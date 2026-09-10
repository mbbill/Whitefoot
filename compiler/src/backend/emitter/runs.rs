//! Emission of the [BLK-1] runs: the window, its subscript, and [BLK-3]'s
//! four boundary operations.
//!
//! A run carries no per-slot tag and no runtime discriminant: `len` and `head`
//! are the complete typestate. Boundary operations update these words and the
//! selected element in the result destination; a surviving input value keeps
//! an independent snapshot.
//!
//! The window is `len` slots beginning at `head` modulo `cap`, so a subscript
//! at logical offset `i` reads slot `(head + i) mod cap` [BLK-1]. Because
//! `head < cap` and `i < len <= cap`, the sum is below `2 * cap` and the
//! modulus is one conditional subtract; no division is emitted.

use crate::{IrBoundary, IrElement, IrMeasure};

use super::*;

/// The two shapes a run takes at run time [BLK-1, OP-9].
#[derive(Clone, Copy)]
enum RunShape {
    /// `FixedVector<T, n>`: `n` inline slots, then `len` and `head`. The
    /// capacity is the type constant and is stored nowhere.
    Inline { element: IrElement, length: u64 },
    /// `Vector<'s, T>`: the descriptor `{ pointer, cap, len, head }`.
    Descriptor { element: IrElement },
}

impl RunShape {
    const fn of(ty: IrType) -> Option<Self> {
        match ty {
            IrType::FixedVector { element, length } => Some(Self::Inline { element, length }),
            IrType::Vector { element, .. } => Some(Self::Descriptor { element }),
            _ => None,
        }
    }

    const fn element(self) -> IrElement {
        match self {
            Self::Inline { element, .. } | Self::Descriptor { element } => element,
        }
    }

    fn element_type(self, program: &IrProgram<'_, '_, '_>) -> Result<IrType, BackendFailure> {
        program
            .element(self.element())
            .ok_or(BackendFailure::InvalidIr)
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
        let head = self.run_word(run_type, run, shape.head_field())?;
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

    /// [BLK-2] `arena_frame`: one bump extent reserved in this activation's
    /// own frame.
    ///
    /// The provider value is the reservation's base address and its cursor,
    /// and the reservation is where the extent's initial state is
    /// established: the cursor is zero at every activation of the region
    /// block naming its store region, which is the state [BLK-2] gives a
    /// freshly reserved extent.
    pub(super) fn emit_arena_frame(
        &mut self,
        result: IrValueId,
        ty: IrType,
        bytes: u64,
        align: u64,
    ) -> Result<(), BackendFailure> {
        if ty != IrType::Provider {
            return Err(BackendFailure::InvalidIr);
        }
        let _ = (bytes, align);
        let provider = llvm_type(self.program, ty)?;
        let storage = self.entry_slot(FunctionSlot::ExtentStorage(result))?;
        let based = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{based} = insertvalue {provider} zeroinitializer, ptr {storage}, 0\n  {} = insertvalue {provider} %{based}, i64 0, 1",
            self.value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// [BLK-2] one take from a store: the run the store hands out, and the
    /// store's own advanced state written back through the `&uniq` borrow.
    ///
    /// A bump take is `advance<T>(count)` bytes at the extent's cursor
    /// [BLK-0]; the take succeeds exactly when `room_of(store) >=
    /// advance<T>(count)`, which is the relation the refusing row publishes on
    /// its `None` arm. There is no branch: the refusal is a value, so both
    /// arms are computed and the outcome selects between them, and a refused
    /// take leaves the cursor where it was.
    pub(super) fn emit_store_take(
        &mut self,
        result: IrValueId,
        ty: IrType,
        take: crate::IrStoreTake,
    ) -> Result<(), BackendFailure> {
        // The element type, its layout ceilings and the retained count bound
        // are the target stage's own [STOR-6]; emission reads the stride they
        // qualified and nothing else of them.
        let crate::IrStoreTake {
            store,
            count,
            stride,
            extent,
            refusal,
            ..
        } = take;
        if self.value_type(store) != Some(IrType::Address(crate::IrAddressed::Provider))
            || self.value_type(count)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
        {
            return Err(BackendFailure::InvalidIr);
        }
        let run_type = match (ty, refusal) {
            (IrType::Vector { .. }, None) => ty,
            (IrType::Nominal(nominal), Some(refusal)) if nominal == refusal.nominal => {
                self.refusal_payload_type(refusal)?
            }
            _ => return Err(BackendFailure::InvalidIr),
        };
        let provider = llvm_type(self.program, IrType::Provider)?;
        let count = self.value_name(count);
        let address = self.value_name(store);
        let (pointer, capacity, taken) = match extent {
            Some(extent) => self.emit_bump_take(&address, &provider, &count, stride, extent)?,
            None => self.emit_general_take(&address, &count, stride)?,
        };
        // The run itself: the storage the store handed out, `count` slots of
        // capacity, and the empty window [BLK-2] publishes.
        let run_llvm = llvm_type(self.program, run_type)?;
        let based = self.next_temporary()?;
        let run = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{based} = insertvalue {run_llvm} zeroinitializer, ptr {pointer}, 0\n  %{run} = insertvalue {run_llvm} %{based}, i64 {capacity}, 1",
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let Some(refusal) = refusal else {
            writeln!(
                self.output,
                "  {} = insertvalue {run_llvm} %{run}, i64 0, 2",
                self.value_name(result),
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        };
        let complete = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{complete} = insertvalue {run_llvm} %{run}, i64 0, 2",
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let outcome = llvm_type(self.program, ty)?;
        let variants = self.refusal_variants(refusal)?;
        let payload = variant_field_base(&variants, refusal.made)?;
        let tag = self.next_temporary()?;
        let tagged = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{tag} = select i1 {taken}, i32 {}, i32 {}\n  %{tagged} = insertvalue {outcome} zeroinitializer, i32 %{tag}, 0\n  {} = insertvalue {outcome} %{tagged}, {run_llvm} %{complete}, {payload}",
            refusal.made,
            refusal.refused,
            self.value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// S39 one cell formation: the store's take of one cell's bytes, the
    /// move of the value into it, and the outcome that carries either.
    ///
    /// The store's answer decides the arm, so this is the one kernel row
    /// whose emission branches: the value is written into the cell only on
    /// the arm where the store gave one, and it is handed back in the
    /// refusal's own payload on the other. Both payload fields of the
    /// outcome are written and the tag decides which the release walk and
    /// every reader select [PRE-1].
    pub(super) fn emit_store_box(
        &mut self,
        result: IrValueId,
        ty: IrType,
        cell: crate::IrStoreBox,
    ) -> Result<(), BackendFailure> {
        let crate::IrStoreBox {
            store,
            value,
            bytes,
            extent,
            outcome,
            ..
        } = cell;
        if self.value_type(store) != Some(IrType::Address(crate::IrAddressed::Provider)) {
            return Err(BackendFailure::InvalidIr);
        }
        if ty != IrType::Nominal(outcome.nominal) {
            return Err(BackendFailure::InvalidIr);
        }
        let referent_type = llvm_type(
            self.program,
            self.value_type(value).ok_or(BackendFailure::InvalidIr)?,
        )?;
        let provider = llvm_type(self.program, IrType::Provider)?;
        let address = self.value_name(store);
        // A zero-byte cell still needs one distinct address to store into.
        let request = bytes.max(1);
        let (pointer, taken) = match extent {
            Some(extent) => {
                let count = "1".to_owned();
                let (pointer, _, taken) =
                    self.emit_bump_take(&address, &provider, &count, request, extent)?;
                (pointer, taken)
            }
            None => {
                let raw = self.next_temporary()?;
                let supplied = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{raw} = call ptr @malloc(i64 {request})\n  %{supplied} = icmp ne ptr %{raw}, null",
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                (format!("%{raw}"), format!("%{supplied}"))
            }
        };
        let made = format!("box.made.v{}", result.ordinal());
        let refused = format!("box.refused.v{}", result.ordinal());
        let joined = super::store_box_join_label(result);
        writeln!(
            self.output,
            "  br i1 {taken}, label %{made}, label %{refused}\n\
             {made}:\n  store {referent_type} {}, ptr {pointer}\n  br label %{joined}\n\
             {refused}:\n  br label %{joined}\n\
             {joined}:",
            self.value_name(value),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let outcome_type = llvm_type(self.program, ty)?;
        let variants = self.refusal_variants(outcome)?;
        let ok_field = variant_field_base(&variants, outcome.made)?;
        let err_field = variant_field_base(&variants, outcome.refused)?;
        let tag = self.next_temporary()?;
        let tagged = self.next_temporary()?;
        let carried = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{tag} = phi i32 [ {}, %{made} ], [ {}, %{refused} ]\n  \
             %{tagged} = insertvalue {outcome_type} zeroinitializer, i32 %{tag}, 0\n  \
             %{carried} = insertvalue {outcome_type} %{tagged}, ptr {pointer}, {ok_field}\n  \
             {} = insertvalue {outcome_type} %{carried}, {referent_type} {}, {err_field}",
            outcome.made,
            outcome.refused,
            self.value_name(result),
            self.value_name(value),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// The bump take: the cursor advance, the refusal condition, and the
    /// store's own new state.
    ///
    /// `advance<T>(count)` is `round_up(size_ceiling(T) * count, align)`
    /// [BLK-0]. The product cannot overflow, `fits::<T>(count)` having been
    /// discharged at the call [OP-9]; the rounding is guarded anyway, so an
    /// unrepresentable advance refuses instead of wrapping into an accepted
    /// take.
    fn emit_bump_take(
        &mut self,
        address: &str,
        provider: &str,
        count: &str,
        stride: u64,
        extent: crate::IrExtentConstants,
    ) -> Result<(String, String, String), BackendFailure> {
        let align = extent.align.max(1);
        let mask = (!(align - 1)) as i64;
        let state = self.next_temporary()?;
        let base = self.next_temporary()?;
        let cursor = self.next_temporary()?;
        let raw = self.next_temporary()?;
        let padded = self.next_temporary()?;
        let advance = self.next_temporary()?;
        let representable = self.next_temporary()?;
        let room = self.next_temporary()?;
        let fits = self.next_temporary()?;
        let taken = self.next_temporary()?;
        let pointer = self.next_temporary()?;
        let next = self.next_temporary()?;
        let moved = self.next_temporary()?;
        let advanced = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{state} = load {provider}, ptr {address}\n  \
             %{base} = extractvalue {provider} %{state}, 0\n  \
             %{cursor} = extractvalue {provider} %{state}, 1\n  \
             %{raw} = mul nuw i64 {count}, {stride}\n  \
             %{padded} = add i64 %{raw}, {}\n  \
             %{advance} = and i64 %{padded}, {mask}\n  \
             %{representable} = icmp uge i64 %{padded}, %{raw}\n  \
             %{room} = sub i64 {}, %{cursor}\n  \
             %{fits} = icmp uge i64 %{room}, %{advance}\n  \
             %{taken} = and i1 %{representable}, %{fits}\n  \
             %{pointer} = getelementptr inbounds i8, ptr %{base}, i64 %{cursor}\n  \
             %{next} = add i64 %{cursor}, %{advance}\n  \
             %{moved} = select i1 %{taken}, i64 %{next}, i64 %{cursor}\n  \
             %{advanced} = insertvalue {provider} %{state}, i64 %{moved}, 1\n  \
             store {provider} %{advanced}, ptr {address}",
            align - 1,
            extent.bytes,
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let capacity = self.next_temporary()?;
        let handed = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{capacity} = select i1 %{taken}, i64 {count}, i64 0\n  %{handed} = select i1 %{taken}, ptr %{pointer}, ptr null",
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok((
            format!("%{handed}"),
            format!("%{capacity}"),
            format!("%{taken}"),
        ))
    }

    /// The general-store take: the host is asked for the run's bytes and its
    /// refusal is the row's `None` arm [BLK-2, L6].
    fn emit_general_take(
        &mut self,
        _address: &str,
        count: &str,
        stride: u64,
    ) -> Result<(String, String, String), BackendFailure> {
        let bytes = self.next_temporary()?;
        let pointer = self.next_temporary()?;
        let empty = self.next_temporary()?;
        let supplied = self.next_temporary()?;
        let taken = self.next_temporary()?;
        let capacity = self.next_temporary()?;
        let handed = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{bytes} = mul nuw i64 {count}, {stride}\n  \
             %{pointer} = call ptr @malloc(i64 %{bytes})\n  \
             %{empty} = icmp eq i64 %{bytes}, 0\n  \
             %{supplied} = icmp ne ptr %{pointer}, null\n  \
             %{taken} = or i1 %{empty}, %{supplied}\n  \
             %{capacity} = select i1 %{taken}, i64 {count}, i64 0\n  \
             %{handed} = select i1 %{taken}, ptr %{pointer}, ptr null",
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok((
            format!("%{handed}"),
            format!("%{capacity}"),
            format!("%{taken}"),
        ))
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
        match field.ty() {
            ty @ IrType::Vector { .. } => Ok(ty),
            _ => Err(BackendFailure::InvalidIr),
        }
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
        // A bump extent has one measure word, its cursor [MSR-1]: its byte
        // extent is the type constant and its `room_of` is the complement the
        // lowering already formed, so neither reaches emission, and it has no
        // window at all.
        if container_type == IrType::Provider {
            if measure != IrMeasure::Length {
                return Err(BackendFailure::InvalidIr);
            }
            let cursor = self.run_word(container_type, container, 1)?;
            return writeln!(
                self.output,
                "  {} = add i64 {cursor}, 0",
                self.value_name(result)
            )
            .map_err(|_| BackendFailure::TextEmission);
        }
        let Some(shape) = RunShape::of(container_type) else {
            return Err(BackendFailure::InvalidIr);
        };
        let value = match measure {
            IrMeasure::Length => self.run_word(container_type, container, shape.length_field())?,
            IrMeasure::Head => self.run_word(container_type, container, shape.head_field())?,
            // A `FixedVector`'s capacity is the type constant and never
            // reaches emission; a `Vector`'s is the descriptor word.
            IrMeasure::Capacity => match shape {
                RunShape::Inline { .. } => return Err(BackendFailure::InvalidIr),
                RunShape::Descriptor { .. } => self.run_word(container_type, container, 1)?,
            },
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
        let IrType::Slice { element } = ty else {
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
        let head = self.run_word(run_type, run, shape.head_field())?;
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
        let head = self.run_word(run_type, run, shape.head_field())?;
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
        let run_type = self.value_type(run).ok_or(BackendFailure::InvalidIr)?;
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
                if self.value_type(value) != Some(shape.element_type(self.program)?) {
                    return Err(BackendFailure::InvalidIr);
                }
            }
            (false, None) => {}
            _ => return Err(BackendFailure::InvalidIr),
        }
        let llvm = llvm_type(self.program, run_type)?;
        let length = self.run_word(run_type, run, shape.length_field())?;
        let head = self.run_word(run_type, run, shape.head_field())?;
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
        if self.storage.slot(result).is_some() {
            let destination = self.value_place(result)?;
            let length_address = self.aggregate_field_pointer(
                run_type,
                &destination,
                shape.length_field() as usize,
            )?;
            let head_address =
                self.aggregate_field_pointer(run_type, &destination, shape.head_field() as usize)?;
            return writeln!(self.output, "  store i64 %{new_length}, ptr {length_address}\n  store i64 {new_head}, ptr {head_address}")
                .map_err(|_| BackendFailure::TextEmission);
        }
        // Only an external-backing descriptor remains an SSA aggregate.
        let base = self.value_name(run);
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
        _result: IrValueId,
        shape: RunShape,
        run_type: IrType,
        run: IrValueId,
        physical: &str,
    ) -> Result<String, BackendFailure> {
        let element_type = llvm_type(self.program, shape.element_type(self.program)?)?;
        let pointer = self.next_temporary()?;
        match shape {
            RunShape::Inline { .. } => {
                let llvm = llvm_type(self.program, run_type)?;
                let slot = self.run_storage(run)?.ok_or(BackendFailure::InvalidIr)?;
                writeln!(
                    self.output,
                    "  %{pointer} = getelementptr inbounds {llvm}, ptr {slot}, i64 0, i32 0, i64 {physical}",
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            RunShape::Descriptor { .. } => {
                let llvm = llvm_type(self.program, run_type)?;
                let base = self.next_temporary()?;
                if let Some(address) = self.run_storage(run)? {
                    let field = self.aggregate_field_pointer(run_type, &address, 0)?;
                    writeln!(self.output, "  %{base} = load ptr, ptr {field}\n  %{pointer} = getelementptr inbounds {element_type}, ptr %{base}, i64 {physical}")
                        .map_err(|_| BackendFailure::TextEmission)?;
                    return Ok(pointer);
                }
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
