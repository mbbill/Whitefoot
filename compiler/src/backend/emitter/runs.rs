//! Emission of the [TYPE-9] windows: the window itself, its subscript, and
//! the [OP-10] boundary operations over it.
//!
//! A window carries no per-slot tag and no runtime discriminant: `len` and,
//! on a `Ring`, `head` are the complete typestate [WIN-1]. A boundary
//! operation updates those words and the selected slot through the reference
//! the operand names.
//!
//! The layout is header-first, so the inline and the boxed placement of one
//! shape share one address computation
//! (compiler/storage-representation): a `Slots` is `{ len, slots }` and a
//! `Ring` is `{ len, head, slots }`, each with a `cap` word after `len` in
//! the runtime-capacity placement and none at all where the type constant
//! already fixes it.
//!
//! The window is `len` slots beginning at `head` modulo `cap` [WIN-1], so a
//! subscript at logical offset `i` reads slot `(head + i) mod cap`. Because
//! `head < cap` and `i < len <= cap`, the sum is below `2 * cap` and the
//! modulus is one conditional subtract; no division is emitted.

use crate::{IrBoundary, IrElement, IrMeasure, IrWindowShape};

use super::*;

/// One emitted window's field layout [TYPE-9, WIN-1].
#[derive(Clone, Copy)]
struct RunShape {
    shape: IrWindowShape,
    element: IrElement,
    /// `Some` is the constant-capacity placement, whose capacity is the type
    /// constant and is stored nowhere.
    capacity: Option<u64>,
}

impl RunShape {
    const fn of(ty: IrType) -> Option<Self> {
        match ty {
            IrType::Window {
                shape,
                element,
                capacity,
            } => Some(Self {
                shape,
                element,
                capacity,
            }),
            _ => None,
        }
    }

    const fn element(self) -> IrElement {
        self.element
    }

    fn element_type(self, program: &IrProgram<'_, '_, '_>) -> Result<IrType, BackendFailure> {
        program
            .element(self.element())
            .ok_or(BackendFailure::InvalidIr)
    }

    /// The aggregate field index of `len`, which every row of [MSR-1]'s
    /// table has and which the header-first layout puts first.
    const fn length_field(self) -> u32 {
        0
    }

    /// The aggregate field index of `cap`, which only a runtime-capacity
    /// block stores.
    const fn capacity_field(self) -> Option<u32> {
        match self.capacity {
            Some(_) => None,
            None => Some(1),
        }
    }

    /// The aggregate field index of `head`, which only a `Ring` has [WIN-1].
    const fn head_field(self) -> Option<u32> {
        match (self.shape, self.capacity) {
            (IrWindowShape::Slots, _) => None,
            (IrWindowShape::Ring, Some(_)) => Some(1),
            (IrWindowShape::Ring, None) => Some(2),
        }
    }

    /// The aggregate field index of the slots.
    const fn slots_field(self) -> u32 {
        match (self.shape, self.capacity) {
            (IrWindowShape::Slots, Some(_)) => 1,
            (IrWindowShape::Slots, None) | (IrWindowShape::Ring, Some(_)) => 2,
            (IrWindowShape::Ring, None) => 3,
        }
    }
}

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    fn run_value_type(&self, run: IrValueId) -> Result<IrType, BackendFailure> {
        Ok(
            match self.value_type(run).ok_or(BackendFailure::InvalidIr)? {
                IrType::Address(referent) => referent.ty(),
                ty => ty,
            },
        )
    }

    fn run_storage(&mut self, run: IrValueId) -> Result<Option<String>, BackendFailure> {
        if matches!(self.value_type(run), Some(IrType::Address(_))) {
            Ok(Some(self.value_name(run)))
        } else if self.storage.slot(run).is_some() {
            self.value_place(run).map(Some)
        } else {
            Ok(None)
        }
    }

    fn prepare_run_update(
        &mut self,
        result: IrValueId,
        run: IrValueId,
        ty: IrType,
    ) -> Result<IrValueId, BackendFailure> {
        if matches!(self.value_type(run), Some(IrType::Address(_))) {
            return Ok(run);
        }
        if self.storage.slot(result).is_some() {
            let source = self.value_place(run)?;
            let destination = self.value_place(result)?;
            self.copy_storage(ty, &source, &destination)?;
            Ok(result)
        } else {
            Ok(run)
        }
    }

    pub(super) fn run_element_place(
        &mut self,
        run: IrValueId,
        offset: IrValueId,
        element: IrType,
        target_domain: IrTargetDomainObligation,
    ) -> Result<String, BackendFailure> {
        let run_type = self.run_value_type(run)?;
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        if target_domain != IrTargetDomainObligation::ElementAddress
            || shape.element_type(self.program)? != element
            || self.value_type(offset)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let head = self.window_origin(shape, run_type, run)?;
        let physical = self.wrap_offset(shape, run_type, run, &head, &self.value_name(offset))?;
        self.element_pointer(run, shape, run_type, run, &physical)
            .map(|pointer| format!("%{pointer}"))
    }

    /// [BLK-2] `fixed_vector`: the empty window over `n` raw slots.
    ///
    /// The value is the zero aggregate, so both descriptor words start at
    /// zero, which is exactly the row's four published relations.
    pub(super) fn emit_fixed_vector(
        &mut self,
        result: IrValueId,
        ty: IrType,
    ) -> Result<(), BackendFailure> {
        let Some(_) = RunShape::of(ty) else {
            return Err(BackendFailure::InvalidIr);
        };
        let run_type = llvm_type(self.program, ty)?;
        let destination = self.value_place(result)?;
        writeln!(
            self.output,
            "  store {run_type} zeroinitializer, ptr {destination}",
        )
        .map_err(|_| BackendFailure::TextEmission)
    }


    fn refusal_variants(
        &self,
        refusal: crate::IrRefusal,
    ) -> Result<Vec<crate::IrVariant>, BackendFailure> {
        let IrNominalKind::Enum { variants } = self.nominal(refusal.nominal)?.kind() else {
            return Err(BackendFailure::InvalidIr);
        };
        Ok(variants.to_vec())
    }

    fn refusal_payload_type(&self, refusal: crate::IrRefusal) -> Result<IrType, BackendFailure> {
        let variants = self.refusal_variants(refusal)?;
        let made = variants
            .iter()
            .find(|variant| variant.tag() == refusal.made)
            .ok_or(BackendFailure::InvalidIr)?;
        let [field] = made.fields() else {
            return Err(BackendFailure::InvalidIr);
        };
        Ok(field.ty())
    }

    /// [MSR-1] one measure of a run or a bump extent, read at run time.
    pub(super) fn emit_container_measure(
        &mut self,
        result: IrValueId,
        ty: IrType,
        measure: IrMeasure,
        container: IrValueId,
    ) -> Result<(), BackendFailure> {
        if ty
            != (IrType::Integer {
                width: 64,
                signed: false,
            })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let container_type = self.run_value_type(container)?;
        let Some(shape) = RunShape::of(container_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        let value = match measure {
            IrMeasure::Length => self.run_word(container_type, container, shape.length_field())?,
            // `head` is a `Ring`'s alone [WIN-1]; a `Slots` window begins at
            // slot zero and the measure table gives it no cell at all.
            IrMeasure::Head => {
                let field = shape.head_field().ok_or(BackendFailure::InvalidIr)?;
                self.run_word(container_type, container, field)?
            }
            // A constant capacity is the type constant and never reaches
            // emission; a runtime one is the block's own word.
            IrMeasure::Capacity => {
                let field = shape.capacity_field().ok_or(BackendFailure::InvalidIr)?;
                self.run_word(container_type, container, field)?
            }
            // `room` is the complement [MSR-2] relates to the other two.
            IrMeasure::Room => {
                let length = self.run_word(container_type, container, shape.length_field())?;
                let capacity = self.run_capacity(shape, container_type, container)?;
                return writeln!(
                    self.output,
                    "  {} = sub i64 {capacity}, {length}",
                    self.value_name(result),
                )
                .map_err(|_| BackendFailure::TextEmission);
            }
        };
        writeln!(
            self.output,
            "  {} = add i64 {value}, 0",
            self.value_name(result)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// [VIEW-2] one view formed over typed owner storage.
    ///
    /// The window is `len` slots beginning at `head`, and the row's own
    /// requirement `head_of(vector) <= room_of(vector)` is discharged before
    /// this operation exists [BLK-0], so `head + len <= cap` and the window
    /// is one contiguous range: the descriptor is the address of slot `head`
    /// together with `len`, and no modulus is emitted.
    ///
    /// A complete array instead contributes its type's length and the address
    /// of its first slot, without descriptor metadata in the owner.
    /// Both view modes point into the checked owner's stable storage. A
    /// descriptor copy does not create storage or prolong its lifetime.
    pub(super) fn emit_slice_from_run(
        &mut self,
        result: IrValueId,
        ty: IrType,
        run: IrValueId,
    ) -> Result<(), BackendFailure> {
        let IrType::Range { element } = ty else {
            return Err(BackendFailure::InvalidIr);
        };
        let run_type = self.run_value_type(run)?;
        if let IrType::Array {
            element: actual,
            length,
        } = run_type
        {
            if self.program.element(actual) != Some(element.ty())
                || !matches!(self.value_type(run), Some(IrType::Address(_)))
            {
                return Err(BackendFailure::InvalidIr);
            }
            let pointer = self.value_name(run);
            return self.emit_slice_descriptor(result, ty, &pointer, length);
        }
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        if shape.element_type(self.program)? != element.ty() {
            return Err(BackendFailure::InvalidIr);
        }
        let head = self.window_origin(shape, run_type, run)?;
        let length = self.run_word(run_type, run, shape.length_field())?;
        let pointer = self.element_pointer(result, shape, run_type, run, &head)?;
        let descriptor_type = llvm_type(self.program, ty)?;
        let partial = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{partial} = insertvalue {descriptor_type} zeroinitializer, ptr %{pointer}, 0\n  {} = insertvalue {descriptor_type} %{partial}, i64 {length}, 1",
            self.value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// [OP-4, BLK-1] one discharged subscript read at logical offset `i`.
    pub(super) fn emit_run_index(
        &mut self,
        result: IrValueId,
        ty: IrType,
        run: IrValueId,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let run_type = self.run_value_type(run)?;
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        if shape.element_type(self.program)? != ty
            || self.value_type(offset)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let head = self.window_origin(shape, run_type, run)?;
        let offset = self.value_name(offset);
        let physical = self.wrap_offset(shape, run_type, run, &head, &offset)?;
        let element_pointer = self.element_pointer(result, shape, run_type, run, &physical)?;
        self.load_place_result(result, ty, &format!("%{element_pointer}"))
    }

    /// [BLK-3] the element a removal row hands back, read before the boundary
    /// moves.
    pub(super) fn emit_run_taken(
        &mut self,
        result: IrValueId,
        ty: IrType,
        row: IrBoundary,
        run: IrValueId,
    ) -> Result<(), BackendFailure> {
        if row.places() {
            return Err(BackendFailure::InvalidIr);
        }
        let run_type = self.run_value_type(run)?;
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        if shape.element_type(self.program)? != ty {
            return Err(BackendFailure::InvalidIr);
        }
        let physical = self.boundary_slot(shape, run_type, run, row)?;
        let element_pointer = self.element_pointer(result, shape, run_type, run, &physical)?;
        self.load_place_result(result, ty, &format!("%{element_pointer}"))
    }

    /// [BLK-3] update an exclusive run: one store at the boundary slot for a
    /// placement, and the moved boundary for both.
    pub(super) fn emit_run_boundary(
        &mut self,
        result: IrValueId,
        ty: IrType,
        row: IrBoundary,
        run: IrValueId,
        value: Option<IrValueId>,
    ) -> Result<(), BackendFailure> {
        let run_type = self.run_value_type(run)?;
        if ty != IrType::Unit || !matches!(self.value_type(run), Some(IrType::Address(_))) {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        match (row.places(), value) {
            (true, Some(value)) => {
                if self.value_type(value) != Some(shape.element_type(self.program)?) {
                    return Err(BackendFailure::InvalidIr);
                }
            }
            (false, None) => {}
            _ => return Err(BackendFailure::InvalidIr),
        }
        let length = self.run_word(run_type, run, shape.length_field())?;
        let head = self.window_origin(shape, run_type, run)?;
        let updated = self.prepare_run_update(result, run, run_type)?;
        // A placement writes the element at the slot the boundary is about to
        // occupy; a removal has already read it out.
        if let Some(value) = value {
            let physical = self.boundary_slot(shape, run_type, run, row)?;
            let element_pointer =
                self.element_pointer(result, shape, run_type, updated, &physical)?;
            let element_type = llvm_type(self.program, shape.element_type(self.program)?)?;
            let operand = self.value_operand(value)?;
            writeln!(
                self.output,
                "  store {element_type} {}, ptr %{element_pointer}",
                operand,
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        // The new descriptor words. A back operation leaves `head` where it
        // was; a front operation moves it by one, modulo the capacity.
        let new_length = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{new_length} = {} i64 {length}, 1",
            if row.places() { "add" } else { "sub" },
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        // A front row's new window origin is exactly the slot it just
        // touched: a front placement's is the slot it wrote, and a front
        // removal's is one past the slot it read.
        let new_head = if row.front() {
            if row.places() {
                self.boundary_slot(shape, run_type, run, row)?
            } else {
                self.wrap_offset(shape, run_type, run, &head, "1")?
            }
        } else {
            head
        };
        let destination = self.run_storage(run)?.ok_or(BackendFailure::InvalidIr)?;
        let length_address =
            self.aggregate_field_pointer(run_type, &destination, shape.length_field() as usize)?;
        writeln!(self.output, "  store i64 %{new_length}, ptr {length_address}")
            .map_err(|_| BackendFailure::TextEmission)?;
        // Only a `Ring` stores a window origin [WIN-1]; a `Slots` window
        // begins at slot zero and no operation moves it, so a front
        // operation over one is no row of [OP-10] this emitter can serve.
        if let Some(field) = shape.head_field() {
            let head_address =
                self.aggregate_field_pointer(run_type, &destination, field as usize)?;
            writeln!(self.output, "  store i64 {new_head}, ptr {head_address}")
                .map_err(|_| BackendFailure::TextEmission)?;
        } else if row.front() {
            return Err(BackendFailure::InvalidIr);
        }
        self.emit_constant(result, ty, IrConstant::Unit)
    }

    /// The physical slot one boundary operation touches [BLK-1].
    ///
    /// A back operation touches the slot one past the window's last, which is
    /// `(head + len) mod cap` for a placement and `(head + len - 1) mod cap`
    /// for a removal; a front placement touches `(head + cap - 1) mod cap`
    /// and a front removal touches `head` itself.
    fn boundary_slot(
        &mut self,
        shape: RunShape,
        run_type: IrType,
        run: IrValueId,
        row: IrBoundary,
    ) -> Result<String, BackendFailure> {
        let head = self.window_origin(shape, run_type, run)?;
        match row {
            IrBoundary::TakeFront => Ok(head),
            // One slot before the window origin: `head + cap - 1` lies in
            // `[cap - 1, 2 * cap - 1]`, so it never underflows and the
            // modulus is the same one conditional subtract.
            IrBoundary::PlaceFront => {
                let capacity = self.run_capacity(shape, run_type, run)?;
                let raised = self.next_temporary()?;
                let stepped = self.next_temporary()?;
                let over = self.next_temporary()?;
                let wrapped = self.next_temporary()?;
                let physical = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{raised} = add i64 {head}, {capacity}\n  %{stepped} = sub i64 %{raised}, 1\n  %{over} = icmp uge i64 %{stepped}, {capacity}\n  %{wrapped} = sub i64 %{stepped}, {capacity}\n  %{physical} = select i1 %{over}, i64 %{wrapped}, i64 %{stepped}",
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                Ok(format!("%{physical}"))
            }
            IrBoundary::PlaceBack | IrBoundary::TakeBack => {
                let length = self.run_word(run_type, run, shape.length_field())?;
                let offset = if row.places() {
                    length
                } else {
                    let previous = self.next_temporary()?;
                    writeln!(self.output, "  %{previous} = sub i64 {length}, 1")
                        .map_err(|_| BackendFailure::TextEmission)?;
                    format!("%{previous}")
                };
                self.wrap_offset(shape, run_type, run, &head, &offset)
            }
        }
    }

    /// `(base + offset) mod cap`, as the one conditional subtract [BLK-1]
    /// fixes.
    fn wrap_offset(
        &mut self,
        shape: RunShape,
        run_type: IrType,
        run: IrValueId,
        base: &str,
        offset: &str,
    ) -> Result<String, BackendFailure> {
        let capacity = self.run_capacity(shape, run_type, run)?;
        let sum = self.next_temporary()?;
        let over = self.next_temporary()?;
        let wrapped = self.next_temporary()?;
        let physical = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{sum} = add i64 {base}, {offset}\n  %{over} = icmp uge i64 %{sum}, {capacity}\n  %{wrapped} = sub i64 %{sum}, {capacity}\n  %{physical} = select i1 %{over}, i64 %{wrapped}, i64 %{sum}",
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(format!("%{physical}"))
    }

    /// One descriptor word of a run.
    fn run_word(
        &mut self,
        run_type: IrType,
        run: IrValueId,
        field: u32,
    ) -> Result<String, BackendFailure> {
        let llvm = llvm_type(self.program, run_type)?;
        let word = self.next_temporary()?;
        if let Some(address) = self.run_storage(run)? {
            let pointer = self.aggregate_field_pointer(run_type, &address, field as usize)?;
            writeln!(self.output, "  %{word} = load i64, ptr {pointer}")
                .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(format!("%{word}"));
        }
        writeln!(
            self.output,
            "  %{word} = extractvalue {llvm} {}, {field}",
            self.value_name(run),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(format!("%{word}"))
    }

    /// The run's capacity: a `FixedVector`'s type constant, or a `Vector`'s
    /// own descriptor word.
    fn run_capacity(
        &mut self,
        shape: RunShape,
        run_type: IrType,
        run: IrValueId,
    ) -> Result<String, BackendFailure> {
        match (shape.capacity, shape.capacity_field()) {
            (Some(capacity), _) => Ok(capacity.to_string()),
            (None, Some(field)) => self.run_word(run_type, run, field),
            (None, None) => Err(BackendFailure::InvalidIr),
        }
    }

    /// The window origin [WIN-1]: a `Ring`'s stored `head`, and the constant
    /// zero for a `Slots`, whose window begins at slot zero and whose
    /// measure table gives it no `head` cell at all.
    fn window_origin(
        &mut self,
        shape: RunShape,
        run_type: IrType,
        run: IrValueId,
    ) -> Result<String, BackendFailure> {
        match shape.head_field() {
            Some(field) => self.run_word(run_type, run, field),
            None => Ok("0".to_owned()),
        }
    }

    /// The address of one physical slot of a window.
    ///
    /// The slots follow the header in the same block in both placements
    /// (compiler/storage-representation), so one `getelementptr` serves the
    /// inline window and the boxed one alike; only where the block address
    /// comes from differs.
    fn element_pointer(
        &mut self,
        _result: IrValueId,
        shape: RunShape,
        run_type: IrType,
        run: IrValueId,
        physical: &str,
    ) -> Result<String, BackendFailure> {
        let _ = shape;
        let llvm = llvm_type(self.program, run_type)?;
        let slot = self.run_storage(run)?.ok_or(BackendFailure::InvalidIr)?;
        let pointer = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{pointer} = getelementptr inbounds {llvm}, ptr {slot}, i64 0, i32 {}, i64 {physical}",
            shape.slots_field(),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(pointer)
    }
}
