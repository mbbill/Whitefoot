use std::collections::{HashMap, HashSet};

use crate::{
    IrArrayRoot, IrFlatElement, IrFunction, IrInstruction, IrNominalId, IrNominalKind, IrOperation,
    IrProgram, IrTargetDomainObligation, IrType, IrValueId, SystemIntegerResultBound,
};

use super::qualification::Qualification;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetObject {
    Representation,
    RuntimeSizedAllocation,
    Static,
    FunctionAbi,
    StackFrame,
    ParallelLaneFrame,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetLayoutFailure {
    UnsupportedHost,
    InvalidIr,
    Unrepresentable(TargetObject),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TargetLayout {
    triple: &'static str,
    data_layout: &'static str,
    address_index_max: u64,
    allocator_parameter_max: u64,
    allocator_alignment: u64,
    stack_probe: &'static str,
}

impl TargetLayout {
    pub(super) fn host() -> Result<Self, TargetLayoutFailure> {
        #[cfg(all(target_arch = "aarch64", target_os = "macos"))]
        {
            return Ok(Self {
                triple: "aarch64-apple-darwin",
                stack_probe: "__chkstk_darwin",
                data_layout: "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128-Fn32",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
                allocator_alignment: 8,
            });
        }
        #[cfg(all(target_arch = "x86_64", target_os = "macos"))]
        {
            return Ok(Self {
                triple: "x86_64-apple-darwin",
                stack_probe: "__chkstk_darwin",
                data_layout: "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
                allocator_alignment: 8,
            });
        }
        #[cfg(all(target_arch = "aarch64", target_os = "linux"))]
        {
            return Ok(Self {
                triple: "aarch64-unknown-linux-gnu",
                stack_probe: "inline-asm",
                data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
                allocator_alignment: 8,
            });
        }
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            return Ok(Self {
                triple: "x86_64-unknown-linux-gnu",
                stack_probe: "inline-asm",
                data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128",
                address_index_max: i64::MAX as u64,
                allocator_parameter_max: u64::MAX,
                allocator_alignment: 8,
            });
        }
        #[allow(unreachable_code)]
        Err(TargetLayoutFailure::UnsupportedHost)
    }

    pub(super) const fn triple(self) -> &'static str {
        self.triple
    }

    pub(super) const fn data_layout(self) -> &'static str {
        self.data_layout
    }

    /// The `probe-stack` value every generated function carries: the
    /// ABI-mandated helper an Apple target already names from its own C
    /// translation units, and the target-independent spelling — an inline
    /// page walk — anywhere else.
    ///
    /// A frame larger than the guard region must touch each page on its way
    /// down. Without that, the frame's first store can land past the guard in
    /// whatever is mapped below — on a pool build, another lane's live stack —
    /// and the write succeeds silently. The backend emits the walk only for a
    /// frame past the page threshold, so an ordinary frame pays nothing.
    pub(super) const fn stack_probe(self) -> &'static str {
        self.stack_probe
    }

    pub(super) const fn address_index_max(self) -> u64 {
        self.address_index_max
    }

    pub(super) const fn runtime_allocation_max(self) -> u64 {
        if self.address_index_max < self.allocator_parameter_max {
            self.address_index_max
        } else {
            self.allocator_parameter_max
        }
    }

    /// Minimum alignment guaranteed by the selected target's heap allocator.
    /// Current source representations require at most eight-byte alignment;
    /// keeping the guarantee explicit makes a future wider representation a
    /// target-layout decision rather than an unchecked emitter assumption.
    pub(super) const fn runtime_allocation_alignment(self) -> u64 {
        self.allocator_alignment
    }

    /// Retains the selected target ABI while replacing only the heap-domain
    /// limits used by exact boundary tests.
    #[cfg(test)]
    pub(super) const fn with_runtime_allocation_limits_for_test(
        mut self,
        byte_maximum: u64,
        alignment: u64,
    ) -> Self {
        self.address_index_max = byte_maximum;
        self.allocator_parameter_max = byte_maximum;
        self.allocator_alignment = alignment;
        self
    }

    /// Retains the selected target ABI while replacing only the address-index
    /// domain used by exact aggregate-layout boundary tests.
    #[cfg(test)]
    pub(super) const fn with_address_index_max_for_test(mut self, maximum: u64) -> Self {
        self.address_index_max = maximum;
        self
    }
}

#[derive(Clone, Copy)]
struct Layout {
    size: u64,
    align: u64,
}

const POINTER_LAYOUT: Layout = Layout { size: 8, align: 8 };

/// One concrete type owned by target lowering rather than by Whitefoot's
/// source type system.
///
/// The emitter consumes this same tree when it renders an LLVM type. Keeping
/// source types and compiler-owned arrays/records in one closed vocabulary is
/// what lets layout and emission share one materialization plan instead of
/// maintaining parallel lists of strings and sizes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum TargetStorageType {
    Source(IrType),
    Pointer,
    Integer(u16),
    Array {
        element: Box<TargetStorageType>,
        length: u64,
    },
    Struct(Vec<TargetStorageType>),
}

impl TargetStorageType {
    pub(super) const fn source(ty: IrType) -> Self {
        Self::Source(ty)
    }

    pub(super) const fn pointer() -> Self {
        Self::Pointer
    }

    pub(super) const fn integer(width: u16) -> Self {
        Self::Integer(width)
    }

    pub(super) fn array(element: Self, length: u64) -> Self {
        Self::Array {
            element: Box::new(element),
            length,
        }
    }

    pub(super) fn bytes(length: u64) -> Self {
        Self::array(Self::integer(8), length)
    }

    pub(super) fn structure(fields: impl IntoIterator<Item = Self>) -> Self {
        Self::Struct(fields.into_iter().collect())
    }
}

/// One logical slot in a generated frame.
///
/// `alignment` is the alignment the emitter will state for the slot's
/// address. It may be stronger than the type's natural alignment for target
/// ABI byte records such as `stat`. The frame constructor inserts explicit
/// byte padding so the stated alignment is true rather than an LLVM hint.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TargetFrameSlot {
    ty: TargetStorageType,
    alignment: Option<u64>,
}

impl TargetFrameSlot {
    pub(super) const fn natural(ty: TargetStorageType) -> Self {
        Self {
            ty,
            alignment: None,
        }
    }

    pub(super) const fn aligned(ty: TargetStorageType, alignment: u64) -> Self {
        Self {
            ty,
            alignment: Some(alignment),
        }
    }

    pub(super) const fn ty(&self) -> &TargetStorageType {
        &self.ty
    }
}

/// The physical struct field which owns one logical slot, plus the exact
/// selected-target offset at which its pointer is formed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TargetFrameField {
    physical_index: u32,
    offset: u64,
}

impl TargetFrameField {
    pub(super) const fn physical_index(self) -> u32 {
        self.physical_index
    }

    #[cfg(test)]
    pub(super) const fn offset(self) -> u64 {
        self.offset
    }
}

/// A complete generated frame whose bytes are materialized as one LLVM
/// struct allocation.
///
/// `physical_fields` includes explicit inter-slot and tail padding. Therefore
/// the LLVM struct rendered from it has exactly `layout`, even for a logical
/// byte array whose requested address alignment is stronger than its natural
/// type alignment. `logical_fields` maps each source/emitter slot, in the
/// caller's order, to the physical field that owns it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TargetFramePlan {
    physical_fields: Vec<TargetStorageType>,
    logical_fields: Vec<TargetFrameField>,
    layout: TargetAggregateLayout,
}

impl TargetFramePlan {
    pub(super) fn physical_fields(&self) -> &[TargetStorageType] {
        &self.physical_fields
    }

    pub(super) fn logical_field(&self, index: usize) -> Option<TargetFrameField> {
        self.logical_fields.get(index).copied()
    }

    pub(super) const fn layout(&self) -> TargetAggregateLayout {
        self.layout
    }

    pub(super) const fn is_empty(&self) -> bool {
        self.logical_fields.is_empty()
    }
}

/// Constructs and validates one complete compiler-generated frame before its
/// allocation is rendered.
///
/// This consumes the emitter's actual slot descriptions. It does not inspect
/// generated LLVM and it is not an acceptance replay: the resulting physical
/// field list is the sole input from which the emitter may form that frame.
pub(super) fn plan_target_frame(
    target: TargetLayout,
    qualification: &Qualification,
    program: &IrProgram<'_, '_, '_>,
    slots: &[TargetFrameSlot],
) -> Result<TargetFramePlan, TargetLayoutFailure> {
    let mut layouts = LayoutComputer {
        target,
        qualification,
        program,
        nominal: HashMap::new(),
        visiting: HashSet::new(),
    };
    let mut physical_fields = Vec::new();
    let mut logical_fields = Vec::with_capacity(slots.len());
    let mut size = 0_u64;
    let mut frame_alignment = 1_u64;

    for slot in slots {
        let layout = layouts
            .storage_layout(slot.ty())
            .map_err(|failure| as_object(failure, TargetObject::StackFrame))?;
        let requested = slot.alignment.unwrap_or(layout.align);
        if !requested.is_power_of_two() || requested < layout.align {
            return Err(TargetLayoutFailure::InvalidIr);
        }
        let start = align_up(target, size, requested, TargetObject::StackFrame)?;
        if start != size {
            physical_fields.push(TargetStorageType::bytes(start - size));
        }
        let physical_index = u32::try_from(physical_fields.len())
            .map_err(|_| TargetLayoutFailure::Unrepresentable(TargetObject::StackFrame))?;
        physical_fields.push(slot.ty().clone());
        logical_fields.push(TargetFrameField {
            physical_index,
            offset: start,
        });
        size = checked_add(start, layout.size, target, TargetObject::StackFrame)?;
        frame_alignment = frame_alignment.max(requested);
    }

    let complete = align_up(target, size, frame_alignment, TargetObject::StackFrame)?;
    if complete != size {
        physical_fields.push(TargetStorageType::bytes(complete - size));
    }

    Ok(TargetFramePlan {
        physical_fields,
        logical_fields,
        layout: TargetAggregateLayout {
            size: complete,
            align: frame_alignment,
        },
    })
}

/// Validates one compiler-owned heap record against the selected allocator
/// and address domain. Dynamic arrays of this record separately guard their
/// element count against `runtime_allocation_max`; this function establishes
/// the fixed element stride and alignment that guard relies on.
pub(super) fn validate_runtime_storage(
    target: TargetLayout,
    qualification: &Qualification,
    program: &IrProgram<'_, '_, '_>,
    ty: &TargetStorageType,
) -> Result<TargetAggregateLayout, TargetLayoutFailure> {
    let mut layouts = LayoutComputer {
        target,
        qualification,
        program,
        nominal: HashMap::new(),
        visiting: HashSet::new(),
    };
    let layout = layouts
        .storage_layout(ty)
        .map_err(|failure| as_object(failure, TargetObject::RuntimeSizedAllocation))?;
    if layout.size > target.runtime_allocation_max()
        || layout.align > target.runtime_allocation_alignment()
    {
        return Err(TargetLayoutFailure::Unrepresentable(
            TargetObject::RuntimeSizedAllocation,
        ));
    }
    Ok(TargetAggregateLayout {
        size: layout.size,
        align: layout.align,
    })
}

pub(super) fn validate_static_storage(
    target: TargetLayout,
    qualification: &Qualification,
    program: &IrProgram<'_, '_, '_>,
    ty: &TargetStorageType,
) -> Result<TargetAggregateLayout, TargetLayoutFailure> {
    let mut layouts = LayoutComputer {
        target,
        qualification,
        program,
        nominal: HashMap::new(),
        visiting: HashSet::new(),
    };
    let layout = layouts
        .storage_layout(ty)
        .map_err(|failure| as_object(failure, TargetObject::Static))?;
    if layout.size > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(TargetObject::Static));
    }
    Ok(TargetAggregateLayout {
        size: layout.size,
        align: layout.align,
    })
}

/// The selected-target layout of one fully assembled backend aggregate.
///
/// Each consumer retains the part of the result its emitted form needs: the
/// stackless root states the validated alignment on its `alloca`, while an
/// ordinary parallel hand-out passes the validated byte size to the lane
/// runtime. Tests inspect both values at exact target and runtime boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TargetAggregateLayout {
    size: u64,
    align: u64,
}

impl TargetAggregateLayout {
    pub(super) const fn size(self) -> u64 {
        self.size
    }

    pub(super) const fn align(self) -> u64 {
        self.align
    }
}

/// Alignment the compiler-owned parallel runtime guarantees for the byte
/// storage at the start of every worker slot.
///
/// This is a backend ABI constant, not a source-language limit. The matching
/// C declaration is pinned by a backend test beside the size contract.
pub(super) const PARALLEL_LANE_FRAME_ALIGNMENT: u64 = 16;

/// Computes the selected-target layout of the exact aggregate an ordinary
/// handed-out call would place in a runtime lane: every declared argument in
/// ABI order, followed by the result.
///
/// `Some` means both address formation and the runtime slot contract can hold
/// the aggregate. `None` is an ordinary optimization decline for a layout
/// which is representable on the selected target but wider or more aligned
/// than the lane slot. An address-domain failure remains a target-layout
/// failure, and malformed IR remains a compiler failure.
pub(super) fn parallel_lane_frame_layout(
    target: TargetLayout,
    qualification: &Qualification,
    program: &IrProgram<'_, '_, '_>,
    function: &IrFunction,
) -> Result<Option<TargetAggregateLayout>, TargetLayoutFailure> {
    let mut layouts = LayoutComputer {
        target,
        qualification,
        program,
        nominal: HashMap::new(),
        visiting: HashSet::new(),
    };
    let mut fields = Vec::with_capacity(function.parameters().len() + 1);
    for (_, ty) in function.parameters() {
        fields.push(
            layouts
                .layout(*ty)
                .map_err(|failure| as_object(failure, TargetObject::ParallelLaneFrame))?,
        );
    }
    fields.push(
        layouts
            .layout(function.result())
            .map_err(|failure| as_object(failure, TargetObject::ParallelLaneFrame))?,
    );
    let layout = layouts.aggregate_layout(fields, TargetObject::ParallelLaneFrame)?;
    if layout.size > crate::LANE_FRAME_BYTES || layout.align > PARALLEL_LANE_FRAME_ALIGNMENT {
        return Ok(None);
    }
    Ok(Some(TargetAggregateLayout {
        size: layout.size,
        align: layout.align,
    }))
}

/// Validates the exact ordered field list the stackless emitter will render
/// for its root frame.
///
/// This runs after stackless planning has selected the live values, but before
/// the emitter writes the frame's LLVM definition. Each field is laid out by
/// the selected target rules, then the complete aggregate is checked with all
/// inter-field and tail padding included. No separately maintained field list
/// participates in acceptance.
pub(super) fn validate_stackless_root_frame(
    target: TargetLayout,
    qualification: &Qualification,
    program: &IrProgram<'_, '_, '_>,
    fields: &[IrType],
) -> Result<TargetAggregateLayout, TargetLayoutFailure> {
    let mut layouts = LayoutComputer {
        target,
        qualification,
        program,
        nominal: HashMap::new(),
        visiting: HashSet::new(),
    };
    let mut field_layouts = Vec::with_capacity(fields.len());
    for field in fields {
        field_layouts.push(
            layouts
                .layout(*field)
                .map_err(|failure| as_object(failure, TargetObject::StackFrame))?,
        );
    }
    let layout = layouts.aggregate_layout(field_layouts, TargetObject::StackFrame)?;
    Ok(TargetAggregateLayout {
        size: layout.size,
        align: layout.align,
    })
}

pub(super) fn validate_program(
    target: TargetLayout,
    qualification: &Qualification,
    program: &IrProgram<'_, '_, '_>,
) -> Result<(), TargetLayoutFailure> {
    let mut layouts = LayoutComputer {
        target,
        qualification,
        program,
        nominal: HashMap::new(),
        visiting: HashSet::new(),
    };

    for nominal in program.nominals() {
        layouts.layout(IrType::Nominal(nominal.id()))?;
    }
    for nominal in program.nominals() {
        if let IrNominalKind::Box { referent } = nominal.kind() {
            layouts.layout(*referent)?;
        }
    }
    for constant in program.constants() {
        layouts
            .layout(constant.ty())
            .map_err(|failure| as_object(failure, TargetObject::Static))?;
    }
    for function in program.functions() {
        validate_function(&mut layouts, function)?;
    }
    Ok(())
}

fn validate_function(
    layouts: &mut LayoutComputer<'_, '_, '_, '_>,
    function: &IrFunction,
) -> Result<(), TargetLayoutFailure> {
    layouts
        .layout(function.result())
        .map_err(|failure| as_object(failure, TargetObject::FunctionAbi))?;
    for (_, ty) in function.parameters() {
        layouts
            .layout(*ty)
            .map_err(|failure| as_object(failure, TargetObject::FunctionAbi))?;
    }
    let integer_upper_bounds = target_integer_result_bounds(layouts, function)?;

    for block in function.blocks() {
        for (_, ty) in block.parameters() {
            layouts.layout(*ty)?;
        }
        for instruction in block.instructions() {
            let IrInstruction::Define {
                result: _,
                ty,
                operation,
            } = instruction
            else {
                continue;
            };
            layouts.layout(*ty)?;
            validate_target_obligation(layouts, function, &integer_upper_bounds, *ty, operation)?;
        }
    }
    Ok(())
}

/// Attaches selected-target integer bounds to the exact SSA values that carry
/// them. Qualified system-operation rows contribute their declared bounds;
/// buffer lengths contribute the representation invariant established by
/// target validation. This metadata never becomes an ambient source fact.
fn target_integer_result_bounds(
    layouts: &mut LayoutComputer<'_, '_, '_, '_>,
    function: &IrFunction,
) -> Result<HashMap<IrValueId, u64>, TargetLayoutFailure> {
    let mut bounds = HashMap::new();
    let u64_type = IrType::Integer {
        width: 64,
        signed: false,
    };
    for block in function.blocks() {
        for instruction in block.instructions() {
            let IrInstruction::Define {
                result,
                ty,
                operation,
            } = instruction
            else {
                continue;
            };
            let upper_bound = match operation {
                IrOperation::SystemCall { operation, .. } => {
                    let implementation = layouts
                        .qualification
                        .operation(*operation)
                        .map_err(|_| TargetLayoutFailure::InvalidIr)?;
                    implementation
                        .integer_result_bound()
                        .map(|bound| match bound {
                            SystemIntegerResultBound::AddressIndexMaximum => {
                                layouts.target.address_index_max()
                            }
                        })
                }
                IrOperation::BufferLength { buffer } => {
                    let Some(IrType::Buffer { element }) = function.value_type(*buffer) else {
                        return Err(TargetLayoutFailure::InvalidIr);
                    };
                    let stride = flat_element_stride(layouts, element)?;
                    Some(element_count_max(
                        layouts.target.runtime_allocation_max(),
                        stride,
                    ))
                }
                _ => None,
            };
            let Some(upper_bound) = upper_bound else {
                continue;
            };
            if *ty != u64_type || function.value_type(*result) != Some(u64_type) {
                return Err(TargetLayoutFailure::InvalidIr);
            }
            if bounds.insert(*result, upper_bound).is_some() {
                return Err(TargetLayoutFailure::InvalidIr);
            }
        }
    }
    Ok(bounds)
}

fn flat_element_stride(
    layouts: &mut LayoutComputer<'_, '_, '_, '_>,
    element: IrFlatElement,
) -> Result<u64, TargetLayoutFailure> {
    let layout = layouts.flat_element(element)?;
    align_up(
        layouts.target,
        layout.size,
        layout.align,
        TargetObject::Representation,
    )
}

const fn element_count_max(byte_maximum: u64, stride: u64) -> u64 {
    match byte_maximum.checked_div(stride) {
        Some(maximum) => maximum,
        None => u64::MAX,
    }
}

fn validate_target_obligation(
    layouts: &mut LayoutComputer<'_, '_, '_, '_>,
    function: &IrFunction,
    integer_upper_bounds: &HashMap<IrValueId, u64>,
    result_type: IrType,
    operation: &IrOperation,
) -> Result<(), TargetLayoutFailure> {
    match operation {
        IrOperation::BoxNew { nominal, value } => {
            if result_type != IrType::Nominal(*nominal) {
                return Err(TargetLayoutFailure::InvalidIr);
            }
            let referent = match layouts
                .program
                .nominal(*nominal)
                .ok_or(TargetLayoutFailure::InvalidIr)?
                .kind()
            {
                IrNominalKind::Box { referent } => *referent,
                _ => return Err(TargetLayoutFailure::InvalidIr),
            };
            if function.value_type(*value) != Some(referent) {
                return Err(TargetLayoutFailure::InvalidIr);
            }
            let allocation = layouts
                .layout(referent)
                .map_err(|failure| as_object(failure, TargetObject::RuntimeSizedAllocation))?;
            if allocation.size > layouts.target.runtime_allocation_max()
                || allocation.align > layouts.target.runtime_allocation_alignment()
            {
                return Err(TargetLayoutFailure::Unrepresentable(
                    TargetObject::RuntimeSizedAllocation,
                ));
            }
        }
        IrOperation::ArenaNew {
            nominal,
            list,
            value,
        } => {
            if result_type != IrType::Nominal(*nominal) {
                return Err(TargetLayoutFailure::InvalidIr);
            }
            let content = match layouts
                .program
                .nominal(*nominal)
                .ok_or(TargetLayoutFailure::InvalidIr)?
                .kind()
            {
                IrNominalKind::Arena { content } => *content,
                _ => return Err(TargetLayoutFailure::InvalidIr),
            };
            if function.value_type(*value) != Some(content) {
                return Err(TargetLayoutFailure::InvalidIr);
            }
            let Some(IrType::Nominal(list_nominal)) = function.value_type(*list) else {
                return Err(TargetLayoutFailure::InvalidIr);
            };
            if !matches!(
                layouts
                    .program
                    .nominal(list_nominal)
                    .ok_or(TargetLayoutFailure::InvalidIr)?
                    .kind(),
                IrNominalKind::ArenaStorage
            ) {
                return Err(TargetLayoutFailure::InvalidIr);
            }

            // Emission allocates exactly `{ ptr, content }`, not the
            // pointer-shaped arena handle. Compute that selected-target
            // structure including the content offset and tail padding before
            // `malloc` is emitted.
            let node = layouts.arena_node_layout(content)?;
            if node.size > layouts.target.runtime_allocation_max()
                || node.align > layouts.target.runtime_allocation_alignment()
            {
                return Err(TargetLayoutFailure::Unrepresentable(
                    TargetObject::RuntimeSizedAllocation,
                ));
            }
        }
        IrOperation::ArrayFill { target_domain, .. }
            if *target_domain == IrTargetDomainObligation::ElementAddress => {}
        IrOperation::BufferFill {
            length,
            target_domains,
            layout_ceiling,
            ..
        }
        | IrOperation::BufferVacant {
            length,
            target_domains,
            layout_ceiling,
            ..
        } if target_domains.is_complete() => {
            let IrType::Buffer { element } = result_type else {
                return Err(TargetLayoutFailure::InvalidIr);
            };
            let actual = layouts.layout(element.ty())?;
            let stride = align_up(
                layouts.target,
                actual.size,
                actual.align,
                TargetObject::Representation,
            )?;
            if !layout_ceiling.size.permits(actual.size)
                || actual.align > layout_ceiling.align
                || !layout_ceiling.stride.permits(stride)
            {
                return Err(TargetLayoutFailure::Unrepresentable(
                    TargetObject::Representation,
                ));
            }
            if actual.align > layouts.target.runtime_allocation_alignment() {
                return Err(TargetLayoutFailure::Unrepresentable(
                    TargetObject::RuntimeSizedAllocation,
                ));
            }
            if function.value_type(*length)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
            {
                return Err(TargetLayoutFailure::InvalidIr);
            }
            let source_upper_bound = target_domains.source_length_upper_bound();
            let length_upper_bound = integer_upper_bounds
                .get(length)
                .copied()
                .map_or(source_upper_bound, |target_upper_bound| {
                    source_upper_bound.min(target_upper_bound)
                });
            let byte_upper_bound = length_upper_bound.checked_mul(stride).ok_or(
                TargetLayoutFailure::Unrepresentable(TargetObject::RuntimeSizedAllocation),
            )?;
            if byte_upper_bound > layouts.target.runtime_allocation_max() {
                return Err(TargetLayoutFailure::Unrepresentable(
                    TargetObject::RuntimeSizedAllocation,
                ));
            }
        }
        IrOperation::ArrayIndex {
            root,
            target_domain,
            ..
        } if *target_domain == IrTargetDomainObligation::ElementAddress => {
            let root_type = match root {
                IrArrayRoot::Value(value) => function
                    .value_type(*value)
                    .ok_or(TargetLayoutFailure::InvalidIr)?,
                IrArrayRoot::Constant(id) => layouts
                    .program
                    .constant(*id)
                    .ok_or(TargetLayoutFailure::InvalidIr)?
                    .ty(),
            };
            layouts.layout(root_type)?;
        }
        IrOperation::BufferIndex { target_domain, .. }
        | IrOperation::SliceIndex { target_domain, .. }
            if *target_domain == IrTargetDomainObligation::ElementAddress => {}
        IrOperation::ArrayFill { .. }
        | IrOperation::BufferFill { .. }
        | IrOperation::BufferVacant { .. }
        | IrOperation::ArrayIndex { .. }
        | IrOperation::BufferIndex { .. }
        | IrOperation::SliceIndex { .. } => {
            return Err(TargetLayoutFailure::InvalidIr);
        }
        _ => {}
    }
    Ok(())
}

struct LayoutComputer<'program, 'classified, 'lexed, 'source> {
    target: TargetLayout,
    qualification: &'program Qualification,
    program: &'program IrProgram<'classified, 'lexed, 'source>,
    nominal: HashMap<IrNominalId, Layout>,
    visiting: HashSet<IrNominalId>,
}

impl LayoutComputer<'_, '_, '_, '_> {
    fn storage_layout(&mut self, ty: &TargetStorageType) -> Result<Layout, TargetLayoutFailure> {
        match ty {
            TargetStorageType::Source(ty) => self.layout(*ty),
            TargetStorageType::Pointer => Ok(POINTER_LAYOUT),
            TargetStorageType::Integer(1) | TargetStorageType::Integer(8) => {
                Ok(Layout { size: 1, align: 1 })
            }
            TargetStorageType::Integer(width) if matches!(width, 16 | 32 | 64) => {
                let bytes = u64::from(width / 8);
                Ok(Layout {
                    size: bytes,
                    align: bytes,
                })
            }
            TargetStorageType::Integer(_) => Err(TargetLayoutFailure::InvalidIr),
            TargetStorageType::Array { element, length } => {
                let element = self.storage_layout(element)?;
                let stride = align_up(
                    self.target,
                    element.size,
                    element.align,
                    TargetObject::StackFrame,
                )?;
                Ok(Layout {
                    size: checked_mul(stride, *length, self.target, TargetObject::StackFrame)?,
                    align: element.align,
                })
            }
            TargetStorageType::Struct(fields) => {
                let mut layouts = Vec::with_capacity(fields.len());
                for field in fields {
                    layouts.push(self.storage_layout(field)?);
                }
                self.aggregate_layout(layouts, TargetObject::StackFrame)
            }
        }
    }

    fn layout(&mut self, ty: IrType) -> Result<Layout, TargetLayoutFailure> {
        match ty {
            IrType::Unit | IrType::Bool => Ok(Layout { size: 1, align: 1 }),
            IrType::Integer { width, .. } if matches!(width, 8 | 16 | 32 | 64) => {
                let bytes = u64::from(width / 8);
                Ok(Layout {
                    size: bytes,
                    align: bytes,
                })
            }
            IrType::Integer { .. } => Err(TargetLayoutFailure::InvalidIr),
            IrType::Float { width } if matches!(width, 32 | 64) => {
                let bytes = u64::from(width / 8);
                Ok(Layout {
                    size: bytes,
                    align: bytes,
                })
            }
            IrType::Float { .. } => Err(TargetLayoutFailure::InvalidIr),
            IrType::Nominal(id) => self.nominal_layout(id),
            IrType::Address(_) => Ok(POINTER_LAYOUT),
            IrType::Array { length: 0, .. } => Ok(Layout { size: 0, align: 1 }),
            IrType::Array { element, length } => {
                let element = self.layout(element.ty())?;
                let stride = align_up(
                    self.target,
                    element.size,
                    element.align,
                    TargetObject::Representation,
                )?;
                let size = checked_mul(stride, length, self.target, TargetObject::Representation)?;
                Ok(Layout {
                    size,
                    align: element.align,
                })
            }
            IrType::Buffer { element } => {
                self.flat_element(element)?;
                Ok(Layout { size: 16, align: 8 })
            }
            IrType::Slice { element } => {
                self.flat_element(element)?;
                Ok(Layout { size: 16, align: 8 })
            }
        }
    }

    fn flat_element(&mut self, element: IrFlatElement) -> Result<Layout, TargetLayoutFailure> {
        self.layout(element.ty())
    }

    fn nominal_layout(&mut self, id: IrNominalId) -> Result<Layout, TargetLayoutFailure> {
        if let Some(layout) = self.nominal.get(&id) {
            return Ok(*layout);
        }
        if !self.visiting.insert(id) {
            return Err(TargetLayoutFailure::InvalidIr);
        }
        let nominal = self
            .program
            .nominal(id)
            .ok_or(TargetLayoutFailure::InvalidIr)?;
        // [QUAL-1] fixes an opaque system resource's representation in its
        // qualification record, which qualification resolved before layout
        // ran, so the selected target has an exact size and alignment for it.
        if let IrNominalKind::SystemResource(contract) = nominal.kind() {
            let representation = self
                .qualification
                .resource(contract.resource)
                .map_err(|_| TargetLayoutFailure::InvalidIr)?
                .representation();
            let layout = Layout {
                size: representation.size(),
                align: representation.align(),
            };
            self.visiting.remove(&id);
            self.nominal.insert(id, layout);
            return Ok(layout);
        }
        let layout = if matches!(
            nominal.kind(),
            IrNominalKind::Box { .. } | IrNominalKind::Arena { .. } | IrNominalKind::ArenaStorage
        ) {
            POINTER_LAYOUT
        } else if nominal.is_tag_only_enum() {
            let IrNominalKind::Enum { variants } = nominal.kind() else {
                return Err(TargetLayoutFailure::InvalidIr);
            };
            if variants.len() <= 2 {
                Layout { size: 1, align: 1 }
            } else {
                Layout { size: 4, align: 4 }
            }
        } else {
            let mut fields = Vec::new();
            match nominal.kind() {
                IrNominalKind::Struct {
                    fields: declarations,
                } => fields.extend(declarations.iter().map(|field| field.ty())),
                IrNominalKind::Enum { variants } => {
                    fields.push(IrType::Integer {
                        width: 32,
                        signed: false,
                    });
                    fields.extend(
                        variants
                            .iter()
                            .flat_map(|variant| variant.fields())
                            .map(|field| field.ty()),
                    );
                }
                // A box, arena, or allocation list has its own pointer
                // layout above, and an opaque system resource returned with
                // its qualified representation before this match; none
                // reaches the field walk.
                IrNominalKind::Box { .. }
                | IrNominalKind::Arena { .. }
                | IrNominalKind::ArenaStorage
                | IrNominalKind::SystemResource(_) => {
                    return Err(TargetLayoutFailure::InvalidIr);
                }
            }
            self.struct_layout(fields)?
        };
        self.visiting.remove(&id);
        self.nominal.insert(id, layout);
        Ok(layout)
    }

    fn struct_layout(&mut self, fields: Vec<IrType>) -> Result<Layout, TargetLayoutFailure> {
        let mut layouts = Vec::with_capacity(fields.len());
        for field in fields {
            layouts.push(self.layout(field)?);
        }
        self.aggregate_layout(layouts, TargetObject::Representation)
    }

    fn arena_node_layout(&mut self, content: IrType) -> Result<Layout, TargetLayoutFailure> {
        let content = self
            .layout(content)
            .map_err(|failure| as_object(failure, TargetObject::RuntimeSizedAllocation))?;
        self.aggregate_layout(
            [POINTER_LAYOUT, content],
            TargetObject::RuntimeSizedAllocation,
        )
    }

    fn aggregate_layout(
        &self,
        fields: impl IntoIterator<Item = Layout>,
        object: TargetObject,
    ) -> Result<Layout, TargetLayoutFailure> {
        let mut size = 0_u64;
        let mut alignment = 1_u64;
        for field in fields {
            size = align_up(self.target, size, field.align, object)?;
            size = checked_add(size, field.size, self.target, object)?;
            alignment = alignment.max(field.align);
        }
        size = align_up(self.target, size, alignment, object)?;
        Ok(Layout {
            size,
            align: alignment,
        })
    }
}

fn checked_add(
    left: u64,
    right: u64,
    target: TargetLayout,
    object: TargetObject,
) -> Result<u64, TargetLayoutFailure> {
    let value = left
        .checked_add(right)
        .ok_or(TargetLayoutFailure::Unrepresentable(object))?;
    if value > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(object));
    }
    Ok(value)
}

fn checked_mul(
    left: u64,
    right: u64,
    target: TargetLayout,
    object: TargetObject,
) -> Result<u64, TargetLayoutFailure> {
    let value = left
        .checked_mul(right)
        .ok_or(TargetLayoutFailure::Unrepresentable(object))?;
    if value > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(object));
    }
    Ok(value)
}

fn align_up(
    target: TargetLayout,
    value: u64,
    alignment: u64,
    object: TargetObject,
) -> Result<u64, TargetLayoutFailure> {
    let mask = alignment
        .checked_sub(1)
        .ok_or(TargetLayoutFailure::InvalidIr)?;
    let aligned = value
        .checked_add(mask)
        .map(|sum| sum & !mask)
        .ok_or(TargetLayoutFailure::Unrepresentable(object))?;
    if aligned > target.address_index_max() {
        return Err(TargetLayoutFailure::Unrepresentable(object));
    }
    Ok(aligned)
}

fn as_object(failure: TargetLayoutFailure, object: TargetObject) -> TargetLayoutFailure {
    match failure {
        TargetLayoutFailure::Unrepresentable(_) => TargetLayoutFailure::Unrepresentable(object),
        other => other,
    }
}
