//! Emission of the [BLK-1] runs: the window, its subscript, and [BLK-3]'s
//! four boundary operations.
//!
//! A run carries no per-slot tag and no runtime discriminant: `len` and `head`
//! are the complete typestate, so every operation here is boundary arithmetic
//! over two descriptor words plus at most one element load or store.
//!
//! The window is `len` slots beginning at `head` modulo `cap`, so a subscript
//! at logical offset `i` reads slot `(head + i) mod cap` [BLK-1]. Because
//! `head < cap` and `i < len <= cap`, the sum is below `2 * cap` and the
//! modulus is one conditional subtract; no division is emitted.

use crate::{IrBoundary, IrFlatElement, IrMeasure};

use super::*;

/// The two shapes a run takes at run time [BLK-1, OP-9].
#[derive(Clone, Copy)]
enum RunShape {
    /// `FixedVector<T, n>`: `n` inline slots, then `len` and `head`. The
    /// capacity is the type constant and is stored nowhere.
    Inline { element: IrFlatElement, length: u64 },
    /// `Vector<'s, T>`: the descriptor `{ pointer, cap, len, head }`.
    Descriptor { element: IrFlatElement },
}

impl RunShape {
    const fn of(ty: IrType) -> Option<Self> {
        match ty {
            IrType::FixedVector { element, length } => Some(Self::Inline { element, length }),
            IrType::Vector { element } => Some(Self::Descriptor { element }),
            _ => None,
        }
    }

    const fn element(self) -> IrFlatElement {
        match self {
            Self::Inline { element, .. } | Self::Descriptor { element } => element,
        }
    }

    /// The aggregate field index of `len`.
    const fn length_field(self) -> u32 {
        match self {
            Self::Inline { .. } => 1,
            Self::Descriptor { .. } => 2,
        }
    }

    /// The aggregate field index of `head`.
    const fn head_field(self) -> u32 {
        match self {
            Self::Inline { .. } => 2,
            Self::Descriptor { .. } => 3,
        }
    }
}

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    /// [BLK-2] `seq_fixed`: the empty window over `n` raw slots.
    ///
    /// The value is the zero aggregate, so both descriptor words start at
    /// zero, which is exactly the row's four published relations.
    pub(super) fn emit_seq_fixed(
        &mut self,
        result: IrValueId,
        ty: IrType,
    ) -> Result<(), BackendFailure> {
        let Some(shape) = RunShape::of(ty) else {
            return Err(BackendFailure::InvalidIr);
        };
        let run_type = llvm_type(self.program, ty)?;
        writeln!(
            self.output,
            "  {} = insertvalue {run_type} zeroinitializer, i64 0, {}",
            self.value_name(result),
            shape.length_field(),
        )
        .map_err(|_| BackendFailure::TextEmission)
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
        let container_type = self
            .value_type(container)
            .ok_or(BackendFailure::InvalidIr)?;
        let Some(shape) = RunShape::of(container_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        let llvm = llvm_type(self.program, container_type)?;
        let operand = self.value_name(container);
        match measure {
            IrMeasure::Length => writeln!(
                self.output,
                "  {} = extractvalue {llvm} {operand}, {}",
                self.value_name(result),
                shape.length_field(),
            )
            .map_err(|_| BackendFailure::TextEmission),
            IrMeasure::Head => writeln!(
                self.output,
                "  {} = extractvalue {llvm} {operand}, {}",
                self.value_name(result),
                shape.head_field(),
            )
            .map_err(|_| BackendFailure::TextEmission),
            // A `FixedVector`'s capacity is the type constant and never
            // reaches emission; a `Vector`'s is the descriptor word.
            IrMeasure::Capacity => match shape {
                RunShape::Inline { .. } => Err(BackendFailure::InvalidIr),
                RunShape::Descriptor { .. } => writeln!(
                    self.output,
                    "  {} = extractvalue {llvm} {operand}, 1",
                    self.value_name(result),
                )
                .map_err(|_| BackendFailure::TextEmission),
            },
            // `room` is the complement [MSR-2] relates to the other two.
            IrMeasure::Room => {
                let length = self.next_temporary()?;
                let capacity = self.run_capacity(shape, container_type, container)?;
                writeln!(
                    self.output,
                    "  %{length} = extractvalue {llvm} {operand}, {}\n  {} = sub i64 {capacity}, %{length}",
                    shape.length_field(),
                    self.value_name(result),
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
        }
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
        let run_type = self.value_type(run).ok_or(BackendFailure::InvalidIr)?;
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        if shape.element().ty() != ty
            || self.value_type(offset)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let head = self.run_word(run_type, run, shape.head_field())?;
        let offset = self.value_name(offset);
        let physical = self.wrap_offset(shape, run_type, run, &head, &offset)?;
        let element_pointer = self.element_pointer(result, shape, run_type, run, &physical)?;
        let element_type = llvm_type(self.program, ty)?;
        writeln!(
            self.output,
            "  {} = load {element_type}, ptr %{element_pointer}",
            self.value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// [SET-1, SET-2, BLK-1] one discharged element-position store at logical
    /// offset `i`.
    ///
    /// The slot is the same `(head + i) mod cap` a subscript read computes,
    /// and the two descriptor words are untouched: an element store changes
    /// what the window holds and never where the window is [MSR-2].
    pub(super) fn emit_run_store(
        &mut self,
        result: IrValueId,
        ty: IrType,
        run: IrValueId,
        offset: IrValueId,
        value: IrValueId,
        target_domain: IrTargetDomainObligation,
    ) -> Result<(), BackendFailure> {
        if target_domain != IrTargetDomainObligation::ElementAddress {
            return Err(BackendFailure::InvalidIr);
        }
        let run_type = self.value_type(run).ok_or(BackendFailure::InvalidIr)?;
        if run_type != ty {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        if self.value_type(value) != Some(shape.element().ty())
            || self.value_type(offset)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let head = self.run_word(run_type, run, shape.head_field())?;
        let offset = self.value_name(offset);
        let physical = self.wrap_offset(shape, run_type, run, &head, &offset)?;
        let element_pointer = self.element_pointer(result, shape, run_type, run, &physical)?;
        let element_type = llvm_type(self.program, shape.element().ty())?;
        let llvm = llvm_type(self.program, run_type)?;
        writeln!(
            self.output,
            "  store {element_type} {}, ptr %{element_pointer}",
            self.value_name(value),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        // A frame-resident run was written through its own frame slot, so the
        // value handed back is the reloaded aggregate; a store-resident run's
        // slots are behind its descriptor pointer and its descriptor is
        // unchanged, so the same two words are reinserted to name the result.
        match shape {
            RunShape::Inline { .. } => {
                let slot = self.entry_slot(FunctionSlot::RunStorage(result))?;
                writeln!(
                    self.output,
                    "  {} = load {llvm}, ptr {slot}",
                    self.value_name(result),
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            RunShape::Descriptor { .. } => {
                let length = self.run_word(run_type, run, shape.length_field())?;
                let with_length = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{with_length} = insertvalue {llvm} {}, i64 {length}, {}\n  {} = insertvalue {llvm} %{with_length}, i64 {head}, {}",
                    self.value_name(run),
                    shape.length_field(),
                    self.value_name(result),
                    shape.head_field(),
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
        }
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
        let run_type = self.value_type(run).ok_or(BackendFailure::InvalidIr)?;
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        if shape.element().ty() != ty {
            return Err(BackendFailure::InvalidIr);
        }
        let physical = self.boundary_slot(shape, run_type, run, row)?;
        let element_pointer = self.element_pointer(result, shape, run_type, run, &physical)?;
        let element_type = llvm_type(self.program, ty)?;
        writeln!(
            self.output,
            "  {} = load {element_type}, ptr %{element_pointer}",
            self.value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// [BLK-3] the run one boundary operation hands back: one store at the
    /// boundary slot for a placement, and the moved boundary for both.
    pub(super) fn emit_run_boundary(
        &mut self,
        result: IrValueId,
        ty: IrType,
        row: IrBoundary,
        run: IrValueId,
        value: Option<IrValueId>,
    ) -> Result<(), BackendFailure> {
        let run_type = self.value_type(run).ok_or(BackendFailure::InvalidIr)?;
        if run_type != ty {
            return Err(BackendFailure::InvalidIr);
        }
        let Some(shape) = RunShape::of(run_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        match (row.places(), value) {
            (true, Some(value)) => {
                if self.value_type(value) != Some(shape.element().ty()) {
                    return Err(BackendFailure::InvalidIr);
                }
            }
            (false, None) => {}
            _ => return Err(BackendFailure::InvalidIr),
        }
        let llvm = llvm_type(self.program, run_type)?;
        let length = self.run_word(run_type, run, shape.length_field())?;
        let head = self.run_word(run_type, run, shape.head_field())?;
        // A placement writes the element at the slot the boundary is about to
        // occupy; a removal has already read it out.
        if let Some(value) = value {
            let physical = self.boundary_slot(shape, run_type, run, row)?;
            let element_pointer = self.element_pointer(result, shape, run_type, run, &physical)?;
            let element_type = llvm_type(self.program, shape.element().ty())?;
            writeln!(
                self.output,
                "  store {element_type} {}, ptr %{element_pointer}",
                self.value_name(value),
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
        // A placement wrote through the frame slot, so the value handed back
        // is the reloaded aggregate with its two words replaced; a run with
        // no inline slots carries nothing else and is rebuilt from the
        // operand.
        let base = match (shape, value.is_some()) {
            (RunShape::Inline { .. }, true) => {
                let slot = self.entry_slot(FunctionSlot::RunStorage(result))?;
                let reloaded = self.next_temporary()?;
                writeln!(self.output, "  %{reloaded} = load {llvm}, ptr {slot}")
                    .map_err(|_| BackendFailure::TextEmission)?;
                format!("%{reloaded}")
            }
            _ => self.value_name(run),
        };
        let with_length = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{with_length} = insertvalue {llvm} {base}, i64 %{new_length}, {}\n  {} = insertvalue {llvm} %{with_length}, i64 {new_head}, {}",
            shape.length_field(),
            self.value_name(result),
            shape.head_field(),
        )
        .map_err(|_| BackendFailure::TextEmission)
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
        let head = self.run_word(run_type, run, shape.head_field())?;
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
        match shape {
            RunShape::Inline { length, .. } => Ok(length.to_string()),
            RunShape::Descriptor { .. } => self.run_word(run_type, run, 1),
        }
    }

    /// The address of one physical slot of a run.
    ///
    /// A frame-resident run's slots are inline, so the aggregate is written to
    /// this operation's own frame slot and indexed there; a store-resident
    /// run's slots are behind its descriptor pointer.
    fn element_pointer(
        &mut self,
        result: IrValueId,
        shape: RunShape,
        run_type: IrType,
        run: IrValueId,
        physical: &str,
    ) -> Result<String, BackendFailure> {
        let element_type = llvm_type(self.program, shape.element().ty())?;
        let pointer = self.next_temporary()?;
        match shape {
            RunShape::Inline { .. } => {
                let llvm = llvm_type(self.program, run_type)?;
                let slot = self.entry_slot(FunctionSlot::RunStorage(result))?;
                writeln!(
                    self.output,
                    "  store {llvm} {}, ptr {slot}\n  %{pointer} = getelementptr inbounds {llvm}, ptr {slot}, i64 0, i32 0, i64 {physical}",
                    self.value_name(run),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            RunShape::Descriptor { .. } => {
                let llvm = llvm_type(self.program, run_type)?;
                let base = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{base} = extractvalue {llvm} {}, 0\n  %{pointer} = getelementptr inbounds {element_type}, ptr %{base}, i64 {physical}",
                    self.value_name(run),
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
        }
        Ok(pointer)
    }
}
