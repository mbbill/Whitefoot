use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;

use crate::{IrFlatElement, IrVariant};

use super::super::qualification::Qualification;
use super::super::target::{
    TargetFrameSlot, TargetLayout, TargetStorageType, validate_runtime_storage,
};
use super::{
    BackendFailure, IrNominalId, IrNominalKind, IrProgram, IrType, llvm_storage_type, llvm_type,
    nominal_symbol, render_named_target_frame, system, variant_field_base,
};

pub(super) fn emit_resource_drop_helpers(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    target: TargetLayout,
) -> Result<String, BackendFailure> {
    let mut plan = DropPlan::of(program)?;
    let mut output = String::new();
    for nominal in program.nominals() {
        let IrNominalKind::Enum { variants } = nominal.kind() else {
            continue;
        };
        let ty = IrType::Nominal(nominal.id());
        if !type_requires_cleanup(program, ty)? {
            continue;
        }

        let aggregate_ty = llvm_type(program, ty)?;
        let symbol = drop_helper_symbol(nominal.id());
        if plan.is_recursive(ty) {
            // A drop that can reach its own type again is the one place the
            // depth of this traversal is chosen by the value rather than by
            // the type, so it runs on a worklist instead of the machine
            // stack. The entry point keeps its name and signature; what
            // changes is that it now drives the traversal rather than being
            // one level of it.
            let step = plan.step(ty)?;
            emit_worklist_driver(
                program,
                qualification,
                target,
                &mut output,
                &symbol,
                &aggregate_ty,
                step,
            )?;
        } else {
            writeln!(
                output,
                "define private void @{symbol}({aggregate_ty} %value) {{"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            emit_enum_cleanup_body(
                program,
                qualification,
                &mut output,
                variants,
                ty,
                &aggregate_ty,
                None,
            )?;
            output.push_str("}\n\n");
        }
    }
    for element in cleanup_buffer_element_nominals(program)? {
        // The [STOR-3] affine-element buffer drop: each element's
        // compiler-derived drop in ascending index order, then the one
        // heap free the copy-element buffer already has.
        let element_ty = IrType::Nominal(element);
        let symbol = buffer_drop_helper_symbol(element);
        let aggregate_ty = llvm_type(program, element_ty)?;
        writeln!(
            output,
            "define private void @{symbol}({{ ptr, i64 }} %value) {{\nentry:\n  %pointer = extractvalue {{ ptr, i64 }} %value, 0\n  %length = extractvalue {{ ptr, i64 }} %value, 1\n  br label %head\nhead:\n  %index = phi i64 [ 0, %entry ], [ %next, %body ]\n  %continue = icmp ult i64 %index, %length\n  br i1 %continue, label %body, label %done\nbody:\n  %element.pointer = getelementptr inbounds {aggregate_ty}, ptr %pointer, i64 %index\n  %element = load {aggregate_ty}, ptr %element.pointer"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let mut temporary = 0_u32;
        emit_value_cleanup(
            program,
            qualification,
            &mut output,
            &mut temporary,
            element_ty,
            "%element".to_owned(),
        )?;
        output.push_str(
            "  %next = add i64 %index, 1\n  br label %head\ndone:\n  call void @free(ptr %pointer)\n  ret void\n}\n\n",
        );
    }
    emit_worklist_steps(program, qualification, &mut output, &mut plan)?;
    if plan.is_empty() {
        return Ok(output);
    }
    let entry_type = TargetStorageType::structure([
        TargetStorageType::integer(32),
        TargetStorageType::pointer(),
    ]);
    validate_runtime_storage(target, qualification, program, &entry_type)
        .map_err(BackendFailure::TargetLayout)?;
    let work_type = TargetStorageType::structure([
        TargetStorageType::pointer(),
        TargetStorageType::integer(64),
        TargetStorageType::integer(64),
    ]);
    let mut support = drop_worklist_support(
        target.runtime_allocation_max(),
        &llvm_storage_type(program, &entry_type)?,
        &llvm_storage_type(program, &work_type)?,
    );
    emit_worklist_driver_loop(program, &mut support, &plan)?;
    support.push_str(&output);
    Ok(support)
}

/// Every buffer element nominal in the program whose element drop derives an
/// action, in deterministic nominal order. A buffer type occurs only as a
/// defined value, parameter, or result type or as nominal content, so the
/// flat enumeration below is complete.
fn cleanup_buffer_element_nominals(
    program: &IrProgram<'_, '_, '_>,
) -> Result<Vec<IrNominalId>, BackendFailure> {
    let mut needed = BTreeMap::new();
    for ty in program_types(program) {
        if let IrType::Buffer {
            element: IrFlatElement::Nominal(id),
        } = ty
            && type_requires_cleanup(program, IrType::Nominal(id))?
        {
            needed.insert(id.ordinal(), id);
        }
    }
    Ok(needed.into_values().collect())
}

/// Every type written anywhere in the program: nominal content, and the
/// defined values, parameters, and results of every function.
fn program_types(program: &IrProgram<'_, '_, '_>) -> Vec<IrType> {
    let mut types: Vec<IrType> = Vec::new();
    for nominal in program.nominals() {
        types.push(IrType::Nominal(nominal.id()));
        match nominal.kind() {
            IrNominalKind::Struct { fields } => {
                types.extend(fields.iter().map(|field| field.ty()));
            }
            IrNominalKind::Enum { variants } => {
                types.extend(
                    variants
                        .iter()
                        .flat_map(|variant| variant.fields())
                        .map(|field| field.ty()),
                );
            }
            IrNominalKind::Box { referent } => types.push(*referent),
            IrNominalKind::Arena { content } => types.push(*content),
            IrNominalKind::ArenaStorage | IrNominalKind::SystemResource(_) => {}
        }
    }
    for function in program.functions() {
        types.extend(function.value_types().iter().copied());
        types.extend(function.parameters().iter().map(|(_, ty)| *ty));
        types.push(function.result());
    }
    types
}

pub(super) fn buffer_drop_helper_symbol(element: IrNominalId) -> String {
    format!("wf.drop.buffer.t{}", element.ordinal())
}

pub(super) fn type_requires_cleanup(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<bool, BackendFailure> {
    let mut pending = vec![ty];
    let mut visited = HashSet::new();
    while let Some(current) = pending.pop() {
        match current {
            IrType::Buffer { .. } => return Ok(true),
            // A store-resident run always reclaims its own storage; a
            // frame-resident run reclaims none of its own and needs a walk
            // only when its window holds values that do [STOR-3, BLK-1].
            IrType::Vector { .. } => return Ok(true),
            IrType::FixedVector { element, .. } => pending.push(element.ty()),
            IrType::Provider => {}
            IrType::Nominal(id)
                if matches!(
                    program.nominal(id).map(|nominal| nominal.kind()),
                    Some(IrNominalKind::Box { .. })
                ) =>
            {
                return Ok(true);
            }
            IrType::Nominal(id) if visited.insert(id) => {
                let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
                match nominal.kind() {
                    IrNominalKind::Struct { fields } => {
                        pending.extend(fields.iter().map(|field| field.ty()));
                    }
                    IrNominalKind::Enum { variants } => {
                        pending.extend(
                            variants
                                .iter()
                                .flat_map(|variant| variant.fields())
                                .map(|field| field.ty()),
                        );
                    }
                    // Every [SYS-5] release action is an explicit release the
                    // target stage must emit, including a logical consume that
                    // emits nothing.
                    IrNominalKind::Box { .. }
                    | IrNominalKind::SystemResource(_)
                    // The allocation-list drop is the region's storage
                    // release [STOR-3]: walk and free.
                    | IrNominalKind::ArenaStorage => {
                        return Ok(true);
                    }
                    // An arena value's storage is released with its region,
                    // never by an owner-scope cleanup [STOR-3, STOR-4].
                    IrNominalKind::Arena { .. } => {}
                }
            }
            IrType::Unit
            | IrType::Bool
            | IrType::Integer { .. }
            | IrType::Float { .. }
            | IrType::Array { .. }
            | IrType::Slice { .. }
            | IrType::Address(_)
            | IrType::Nominal(_) => {}
        }
    }
    Ok(false)
}

pub(super) fn drop_helper_symbol(nominal: IrNominalId) -> String {
    format!("wf.drop.t{}", nominal.ordinal())
}

enum CleanupJob {
    Value {
        ty: IrType,
        operand: String,
    },
    Field {
        aggregate_ty: IrType,
        aggregate: String,
        index: usize,
        field_ty: IrType,
    },
    FreePointer(String),
}

pub(super) fn emit_value_cleanup(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    output: &mut String,
    temporary: &mut u32,
    ty: IrType,
    operand: String,
) -> Result<(), BackendFailure> {
    emit_cleanup_jobs(
        program,
        qualification,
        output,
        temporary,
        vec![CleanupJob::Value { ty, operand }],
        None,
    )
}

fn emit_cleanup_jobs(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    output: &mut String,
    temporary: &mut u32,
    mut jobs: Vec<CleanupJob>,
    mut deferral: Option<&mut DropPlan>,
) -> Result<(), BackendFailure> {
    // One entry per deferred edge, in the order the edges are reached. The
    // worklist is last-in first-out, so they are pushed in the reverse of that
    // order, which leaves the subtrees of one node reclaimed in exactly the
    // order the straight-line expansion reclaimed them.
    let mut deferred: Vec<DropEntry> = Vec::new();
    while let Some(job) = jobs.pop() {
        match job {
            CleanupJob::FreePointer(pointer) => {
                writeln!(output, "  call void @free(ptr {pointer})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            CleanupJob::Field {
                aggregate_ty,
                aggregate,
                index,
                field_ty,
            } => {
                let value = next_temporary(temporary)?;
                writeln!(
                    output,
                    "  %{value} = extractvalue {} {aggregate}, {index}",
                    llvm_type(program, aggregate_ty)?
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                jobs.push(CleanupJob::Value {
                    ty: field_ty,
                    operand: format!("%{value}"),
                });
            }
            CleanupJob::Value { ty, operand } => match ty {
                IrType::Buffer { element } => {
                    // An element type whose own drop derives an action makes
                    // the buffer drop the per-element loop plus the free
                    // [STOR-3]; every other element leaves exactly the free.
                    if type_requires_cleanup(program, element.ty())? {
                        let IrFlatElement::Nominal(id) = element else {
                            return Err(BackendFailure::InvalidIr);
                        };
                        if let Some(plan) = deferral.as_deref_mut()
                            && plan.is_recursive(ty)
                        {
                            // Reached from inside a traversal that is already
                            // running on the worklist: the buffer's own drop
                            // joins that worklist rather than starting a
                            // second one underneath it.
                            let step = plan.step(ty)?;
                            writeln!(
                                output,
                                "  call void @{}({{ ptr, i64 }} {operand}, ptr %work)",
                                worklist_step_symbol(step)
                            )
                            .map_err(|_| BackendFailure::TextEmission)?;
                        } else {
                            writeln!(
                                output,
                                "  call void @{}({} {operand})",
                                buffer_drop_helper_symbol(id),
                                llvm_type(program, ty)?
                            )
                            .map_err(|_| BackendFailure::TextEmission)?;
                        }
                    } else {
                        let pointer = next_temporary(temporary)?;
                        writeln!(
                            output,
                            "  %{pointer} = extractvalue {} {operand}, 0\n  call void @free(ptr %{pointer})",
                            llvm_type(program, ty)?
                        )
                        .map_err(|_| BackendFailure::TextEmission)?;
                    }
                }
                IrType::Nominal(id) => {
                    let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
                    match nominal.kind() {
                        IrNominalKind::Struct { fields } => {
                            for (index, field) in fields.iter().enumerate() {
                                if type_requires_cleanup(program, field.ty())? {
                                    jobs.push(CleanupJob::Field {
                                        aggregate_ty: ty,
                                        aggregate: operand.clone(),
                                        index,
                                        field_ty: field.ty(),
                                    });
                                }
                            }
                        }
                        IrNominalKind::Enum { .. } => {
                            if type_requires_cleanup(program, ty)? {
                                if let Some(plan) = deferral.as_deref_mut()
                                    && plan.is_recursive(ty)
                                {
                                    let step = plan.step(ty)?;
                                    writeln!(
                                        output,
                                        "  call void @{}({} {operand}, ptr %work)",
                                        worklist_step_symbol(step),
                                        nominal_symbol(id)
                                    )
                                    .map_err(|_| BackendFailure::TextEmission)?;
                                } else {
                                    writeln!(
                                        output,
                                        "  call void @{}({} {operand})",
                                        drop_helper_symbol(id),
                                        nominal_symbol(id)
                                    )
                                    .map_err(|_| BackendFailure::TextEmission)?;
                                }
                            }
                        }
                        // A resource reached through owned content releases
                        // with its own type's action, exactly as a directly
                        // released owner does [SYS-5].
                        IrNominalKind::SystemResource(contract) => {
                            system::emit_resource_release(
                                qualification,
                                output,
                                temporary,
                                *contract,
                                &operand,
                            )?;
                        }
                        IrNominalKind::Box { referent } => {
                            if let Some(plan) = deferral.as_deref_mut()
                                && plan.defers(ty, *referent)
                            {
                                // One entry names the whole box: the traversal
                                // takes its content, releases the block, and
                                // goes on. Releasing before the content rather
                                // than after is what keeps the pending list
                                // the size of the traversal's frontier instead
                                // of the depth it has reached.
                                let kind = plan.content_kind(*referent)?;
                                deferred.push(DropEntry {
                                    kind,
                                    node: operand,
                                });
                            } else {
                                let loaded = next_temporary(temporary)?;
                                writeln!(
                                    output,
                                    "  %{loaded} = load {}, ptr {operand}",
                                    llvm_type(program, *referent)?
                                )
                                .map_err(|_| BackendFailure::TextEmission)?;
                                jobs.push(CleanupJob::FreePointer(operand));
                                jobs.push(CleanupJob::Value {
                                    ty: *referent,
                                    operand: format!("%{loaded}"),
                                });
                            }
                        }
                        // An arena value's storage is released with its
                        // region, never by an owner-scope cleanup
                        // [STOR-3, STOR-4].
                        IrNominalKind::Arena { .. } => {}
                        // The region's allocation-list drop: walk the list
                        // and free every registered allocation, then leave
                        // the cell empty [STOR-3].
                        IrNominalKind::ArenaStorage => {
                            writeln!(output, "  call void @wf_arena_release(ptr {operand})")
                                .map_err(|_| BackendFailure::TextEmission)?;
                        }
                    }
                }
                // A store-resident run's release is one reclamation of its
                // own run to its store, which for the general store is one
                // free of its descriptor pointer [STOR-3, BLK-1]. A run
                // whose element type derives a release action of its own is
                // an explicit unsupported capability, refused at its type
                // before lowering, so this arm reclaims the run and nothing
                // else.
                IrType::Vector { element } => {
                    if type_requires_cleanup(program, element.ty())? {
                        return Err(BackendFailure::InvalidIr);
                    }
                    let pointer = next_temporary(temporary)?;
                    writeln!(
                        output,
                        "  %{pointer} = extractvalue {} {operand}, 0\n  call void @free(ptr %{pointer})",
                        llvm_type(program, ty)?
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                }
                // A frame-resident run reclaims no storage of its own.
                IrType::FixedVector { element, .. } => {
                    if type_requires_cleanup(program, element.ty())? {
                        return Err(BackendFailure::InvalidIr);
                    }
                }
                IrType::Unit
                | IrType::Bool
                | IrType::Integer { .. }
                | IrType::Float { .. }
                | IrType::Array { .. }
                | IrType::Slice { .. }
                | IrType::Provider
                | IrType::Address(_) => {}
            },
        }
    }
    for entry in deferred.iter().rev() {
        writeln!(
            output,
            "  call void @wf.drop.push(ptr %work, i32 {}, ptr {})",
            entry.kind, entry.node
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    Ok(())
}

fn next_temporary(counter: &mut u32) -> Result<String, BackendFailure> {
    let current = *counter;
    *counter = counter
        .checked_add(1)
        .ok_or(BackendFailure::CounterOverflow)?;
    Ok(format!("drop.{current}"))
}

/// The body of one enum's drop: the tag switch and each variant's field
/// cleanup, from the entry label through the closing `ret`.
fn emit_enum_cleanup_body(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    output: &mut String,
    variants: &[IrVariant],
    ty: IrType,
    aggregate_ty: &str,
    mut deferral: Option<&mut DropPlan>,
) -> Result<(), BackendFailure> {
    writeln!(
        output,
        "entry:\n  %tag = extractvalue {aggregate_ty} %value, 0"
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    writeln!(output, "  switch i32 %tag, label %invalid [")
        .map_err(|_| BackendFailure::TextEmission)?;
    for variant in variants {
        writeln!(
            output,
            "    i32 {}, label %variant.{}",
            variant.tag(),
            variant.tag()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    output.push_str("  ]\n");

    let mut temporary = 0_u32;
    for variant in variants {
        writeln!(output, "variant.{}:", variant.tag()).map_err(|_| BackendFailure::TextEmission)?;
        let base = variant_field_base(variants, variant.tag())?;
        let mut jobs = Vec::new();
        for (field, declaration) in variant.fields().iter().enumerate() {
            if type_requires_cleanup(program, declaration.ty())? {
                jobs.push(CleanupJob::Field {
                    aggregate_ty: ty,
                    aggregate: "%value".to_owned(),
                    index: base
                        .checked_add(field)
                        .ok_or(BackendFailure::CounterOverflow)?,
                    field_ty: declaration.ty(),
                });
            }
        }
        emit_cleanup_jobs(
            program,
            qualification,
            output,
            &mut temporary,
            jobs,
            deferral.as_deref_mut(),
        )?;
        output.push_str("  br label %done\n");
    }

    output.push_str("invalid:\n  call void @abort()\n  unreachable\ndone:\n  ret void\n");
    Ok(())
}

// -------------------------------------------------------------- the worklist

/// One pending reclamation: what to do, and the storage to do it to.
struct DropEntry {
    kind: u32,
    node: String,
}

fn worklist_step_symbol(step: usize) -> String {
    format!("wf.drop.step.{step}")
}

/// What a pending entry of one kind means.
///
/// Both owning indirections a cleanup cycle can close through have a variant.
/// A `box` closes it with one block holding one content, so one entry names
/// the whole edge. A `buffer` closes it with one block holding many elements
/// whose reclamation order [STOR-3] fixes, so it takes one entry per element
/// plus one for the block, and the ordering the rule fixes is carried by the
/// order they are pushed in rather than by a walk that has to resume.
#[derive(Clone, Copy)]
enum DropKind {
    /// The entry's pointer is a heap block this drop owns whose content has
    /// this type: take the content, release the block, drop the content.
    Content(IrType),
    /// The entry's pointer addresses one live element of this type inside a
    /// buffer whose block is still held: take the element and drop it.
    Element(IrType),
    /// The entry's pointer is a buffer block whose elements have all been
    /// taken: release it. [STOR-3] puts this after every element's drop, and
    /// the worklist is last-in first-out, so it is pushed before them.
    Storage,
}

/// Which compiler-derived drops descend a value instead of a type, and the
/// worklist entry kinds their traversal uses.
///
/// A drop is recursive exactly when the type's cleanup can reach the same type
/// again, which is a cycle in the graph below and is decided by strongly
/// connected components rather than by any name, shape, or program. Only the
/// edges *inside* such a component are carried on the worklist: an edge that
/// leaves one can never come back, so its depth is bounded by the type and it
/// stays the straight-line expansion it has always been.
struct DropPlan {
    /// Component index per cleanup-requiring type, for components that can
    /// reach themselves only.
    recursive: HashMap<IrType, usize>,
    /// One per-node drop to emit, in registration order; the position is the
    /// symbol it gets.
    steps: Vec<IrType>,
    step_of: HashMap<IrType, usize>,
    /// One registered entry kind per index.
    kinds: Vec<DropKind>,
    content_of: HashMap<IrType, u32>,
    element_of: HashMap<IrType, u32>,
    storage: Option<u32>,
}

impl DropPlan {
    fn of(program: &IrProgram<'_, '_, '_>) -> Result<Self, BackendFailure> {
        Ok(Self {
            recursive: recursive_cleanup_components(program)?,
            steps: Vec::new(),
            step_of: HashMap::new(),
            kinds: Vec::new(),
            content_of: HashMap::new(),
            element_of: HashMap::new(),
            storage: None,
        })
    }

    fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Whether this type's own cleanup can reach this type again.
    fn is_recursive(&self, ty: IrType) -> bool {
        self.recursive.contains_key(&ty)
    }

    /// Whether an edge through owning indirection closes a cleanup cycle, so
    /// its depth is the value's rather than the type's.
    fn defers(&self, from: IrType, to: IrType) -> bool {
        match (self.recursive.get(&from), self.recursive.get(&to)) {
            (Some(source), Some(target)) => source == target,
            _ => false,
        }
    }

    /// The per-node drop of one type, registering its body for emission.
    fn step(&mut self, ty: IrType) -> Result<usize, BackendFailure> {
        if let Some(step) = self.step_of.get(&ty) {
            return Ok(*step);
        }
        let step = self.steps.len();
        self.steps.push(ty);
        self.step_of.insert(ty, step);
        Ok(step)
    }

    fn content_kind(&mut self, ty: IrType) -> Result<u32, BackendFailure> {
        if let Some(kind) = self.content_of.get(&ty) {
            return Ok(*kind);
        }
        self.step(ty)?;
        let kind = u32::try_from(self.kinds.len()).map_err(|_| BackendFailure::CounterOverflow)?;
        self.kinds.push(DropKind::Content(ty));
        self.content_of.insert(ty, kind);
        Ok(kind)
    }

    fn element_kind(&mut self, ty: IrType) -> Result<u32, BackendFailure> {
        if let Some(kind) = self.element_of.get(&ty) {
            return Ok(*kind);
        }
        self.step(ty)?;
        let kind = u32::try_from(self.kinds.len()).map_err(|_| BackendFailure::CounterOverflow)?;
        self.kinds.push(DropKind::Element(ty));
        self.element_of.insert(ty, kind);
        Ok(kind)
    }

    /// Releasing a buffer block says nothing about what was in it, so every
    /// buffer in the program shares one entry kind.
    fn storage_kind(&mut self) -> Result<u32, BackendFailure> {
        if let Some(kind) = self.storage {
            return Ok(kind);
        }
        let kind = u32::try_from(self.kinds.len()).map_err(|_| BackendFailure::CounterOverflow)?;
        self.kinds.push(DropKind::Storage);
        self.storage = Some(kind);
        Ok(kind)
    }
}

/// One `define` that sets up a worklist, runs one traversal on it, and
/// releases it.
fn emit_worklist_driver(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    target: TargetLayout,
    output: &mut String,
    symbol: &str,
    aggregate_ty: &str,
    step: usize,
) -> Result<(), BackendFailure> {
    let prologue = render_named_target_frame(
        program,
        qualification,
        target,
        &[(
            "%work",
            TargetFrameSlot::natural(TargetStorageType::structure([
                TargetStorageType::pointer(),
                TargetStorageType::integer(64),
                TargetStorageType::integer(64),
            ])),
        )],
    )?;
    writeln!(
        output,
        "define private void @{symbol}({aggregate_ty} %value) {{\nentry:\n{prologue}  store %wf.drop.work zeroinitializer, ptr %work\n  call void @{}({aggregate_ty} %value, ptr %work)\n  call void @wf.drop.run(ptr %work)\n  ret void\n}}\n",
        worklist_step_symbol(step)
    )
    .map_err(|_| BackendFailure::TextEmission)
}

/// The per-node drop of every registered step target, including targets that
/// registration reached while emitting an earlier one.
fn emit_worklist_steps(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    output: &mut String,
    plan: &mut DropPlan,
) -> Result<(), BackendFailure> {
    let mut emitted = 0;
    while emitted < plan.steps.len() {
        let ty = plan.steps[emitted];
        let symbol = worklist_step_symbol(emitted);
        emitted += 1;
        let aggregate_ty = llvm_type(program, ty)?;
        if let IrType::Buffer { element } = ty {
            emit_buffer_worklist_step(program, output, &symbol, &aggregate_ty, element, plan)?;
            continue;
        }
        if let IrType::Nominal(id) = ty
            && let Some(IrNominalKind::Enum { variants }) =
                program.nominal(id).map(|nominal| nominal.kind())
        {
            writeln!(
                output,
                "define private void @{symbol}({aggregate_ty} %value, ptr %work) {{"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            emit_enum_cleanup_body(
                program,
                qualification,
                output,
                variants,
                ty,
                &aggregate_ty,
                Some(plan),
            )?;
            output.push_str("}\n\n");
            continue;
        }
        writeln!(
            output,
            "define private void @{symbol}({aggregate_ty} %value, ptr %work) {{\nentry:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let mut temporary = 0_u32;
        emit_cleanup_jobs(
            program,
            qualification,
            output,
            &mut temporary,
            vec![CleanupJob::Value {
                ty,
                operand: "%value".to_owned(),
            }],
            Some(plan),
        )?;
        output.push_str("  ret void\n}\n\n");
    }
    Ok(())
}

/// The per-node drop of a buffer that is inside a cleanup cycle: hand the
/// block and every live element to the worklist, in the order that makes the
/// traversal take them back in the order [STOR-3] fixes.
///
/// The rule is each element's drop in ascending index order followed by the
/// one heap free. The worklist is last-in first-out, so this pushes the free
/// first and then the elements from the last index down, and nothing here
/// touches an element: the loop only records where they are. That is what
/// keeps a buffer's own walk off the machine stack — the step returns after
/// recording, and the traversal resumes at the entry it takes next rather than
/// at a frame this function would otherwise have to hold open.
fn emit_buffer_worklist_step(
    program: &IrProgram<'_, '_, '_>,
    output: &mut String,
    symbol: &str,
    aggregate_ty: &str,
    element: IrFlatElement,
    plan: &mut DropPlan,
) -> Result<(), BackendFailure> {
    let element_ty = element.ty();
    if !type_requires_cleanup(program, element_ty)? {
        // No element derives an action, so the whole drop is the one free and
        // the cycle through this buffer is unreachable by value.
        let storage = plan.storage_kind()?;
        return writeln!(
            output,
            "define private void @{symbol}({aggregate_ty} %value, ptr %work) {{\nentry:\n  %pointer = extractvalue {aggregate_ty} %value, 0\n  call void @wf.drop.push(ptr %work, i32 {storage}, ptr %pointer)\n  ret void\n}}\n"
        )
        .map_err(|_| BackendFailure::TextEmission);
    }
    let element_llvm = llvm_type(program, element_ty)?;
    let storage = plan.storage_kind()?;
    let kind = plan.element_kind(element_ty)?;
    writeln!(
        output,
        "define private void @{symbol}({aggregate_ty} %value, ptr %work) {{\nentry:\n  %pointer = extractvalue {aggregate_ty} %value, 0\n  %length = extractvalue {aggregate_ty} %value, 1\n  call void @wf.drop.push(ptr %work, i32 {storage}, ptr %pointer)\n  br label %head\nhead:\n  %index = phi i64 [ %length, %entry ], [ %next, %body ]\n  %pending = icmp ugt i64 %index, 0\n  br i1 %pending, label %body, label %done\nbody:\n  %next = sub i64 %index, 1\n  %slot = getelementptr inbounds {element_llvm}, ptr %pointer, i64 %next\n  call void @wf.drop.push(ptr %work, i32 {kind}, ptr %slot)\n  br label %head\ndone:\n  ret void\n}}\n"
    )
    .map_err(|_| BackendFailure::TextEmission)
}

/// The traversal itself: take the newest pending entry and do what its kind
/// says, until none is left.
fn emit_worklist_driver_loop(
    program: &IrProgram<'_, '_, '_>,
    output: &mut String,
    plan: &DropPlan,
) -> Result<(), BackendFailure> {
    output.push_str(DROP_WORKLIST_LOOP_HEAD);
    for kind in 0..plan.kinds.len() {
        writeln!(output, "    i32 {kind}, label %kind.{kind}")
            .map_err(|_| BackendFailure::TextEmission)?;
    }
    output.push_str("  ]\n");
    for (kind, entry) in plan.kinds.iter().enumerate() {
        match entry {
            DropKind::Content(ty) => {
                let aggregate_ty = llvm_type(program, *ty)?;
                let step = *plan.step_of.get(ty).ok_or(BackendFailure::InvalidIr)?;
                // The content is taken before the block is released, so the
                // step reads a value and never the freed storage.
                writeln!(
                    output,
                    "kind.{kind}:\n  %content.{kind} = load {aggregate_ty}, ptr %node\n  call void @free(ptr %node)\n  call void @{}({aggregate_ty} %content.{kind}, ptr %work)\n  br label %loop",
                    worklist_step_symbol(step)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            DropKind::Element(ty) => {
                let aggregate_ty = llvm_type(program, *ty)?;
                let step = *plan.step_of.get(ty).ok_or(BackendFailure::InvalidIr)?;
                // The block this element lives in is released by the `Storage`
                // entry pushed underneath every element of that buffer, so the
                // load here always reads storage the traversal still holds.
                writeln!(
                    output,
                    "kind.{kind}:\n  %element.{kind} = load {aggregate_ty}, ptr %node\n  call void @{}({aggregate_ty} %element.{kind}, ptr %work)\n  br label %loop",
                    worklist_step_symbol(step)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            DropKind::Storage => {
                writeln!(
                    output,
                    "kind.{kind}:\n  call void @free(ptr %node)\n  br label %loop"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
        }
    }
    output.push_str(DROP_WORKLIST_LOOP_TAIL);
    Ok(())
}

/// The worklist's storage and the one operation that grows it.
///
/// The entries are heap-resident because that is the resource the traversal is
/// releasing, and every pending entry names storage the traversal still holds:
/// a `box` entry names a block released as that entry is taken, and a buffer's
/// element entries name slots inside a block whose own entry sits underneath
/// them. The list is therefore bounded by the structure being dismantled —
/// within a small constant factor, since a 16-byte entry can name an
/// eight-byte slot — rather than by the depth reached. A host that refuses the
/// growth writes the heap record through the same latch every other refused
/// allocation uses.
const DROP_WORKLIST_ALLOCATION_MAX: &str = "__WF_DROP_WORKLIST_ALLOCATION_MAX__";
const DROP_ENTRY_TYPE: &str = "__WF_DROP_ENTRY_TYPE__";
const DROP_WORK_TYPE: &str = "__WF_DROP_WORK_TYPE__";

fn drop_worklist_support(runtime_allocation_max: u64, entry_type: &str, work_type: &str) -> String {
    DROP_WORKLIST_SUPPORT
        .replace(
            DROP_WORKLIST_ALLOCATION_MAX,
            &runtime_allocation_max.to_string(),
        )
        .replace(DROP_ENTRY_TYPE, entry_type)
        .replace(DROP_WORK_TYPE, work_type)
}

const DROP_WORKLIST_SUPPORT: &str = "%wf.drop.entry = type __WF_DROP_ENTRY_TYPE__\n%wf.drop.work = type __WF_DROP_WORK_TYPE__\n\ndeclare ptr @realloc(ptr, i64)\n\ndefine private void @wf.drop.push(ptr %work, i32 %kind, ptr %node) {\nentry:\n  %count.slot = getelementptr inbounds %wf.drop.work, ptr %work, i32 0, i32 1\n  %capacity.slot = getelementptr inbounds %wf.drop.work, ptr %work, i32 0, i32 2\n  %count = load i64, ptr %count.slot\n  %capacity = load i64, ptr %capacity.slot\n  %count.in.range = icmp ule i64 %count, %capacity\n  br i1 %count.in.range, label %capacity.check, label %exhausted\ncapacity.check:\n  %full = icmp eq i64 %count, %capacity\n  br i1 %full, label %grow.check, label %store\ngrow.check:\n  %entry.bytes = ptrtoint ptr getelementptr (%wf.drop.entry, ptr null, i64 1) to i64\n  %maximum.entries = udiv i64 __WF_DROP_WORKLIST_ALLOCATION_MAX__, %entry.bytes\n  %half.maximum = lshr i64 %maximum.entries, 1\n  %fresh = icmp eq i64 %capacity, 0\n  %fresh.fits = icmp uge i64 %maximum.entries, 64\n  %double.fits = icmp ule i64 %capacity, %half.maximum\n  %growth.fits = select i1 %fresh, i1 %fresh.fits, i1 %double.fits\n  br i1 %growth.fits, label %grow, label %exhausted\ngrow:\n  %doubled = shl nuw i64 %capacity, 1\n  %wanted = select i1 %fresh, i64 64, i64 %doubled\n  %bytes = mul nuw i64 %wanted, %entry.bytes\n  %previous = load ptr, ptr %work\n  %grown = call ptr @realloc(ptr %previous, i64 %bytes)\n  %refused = icmp eq ptr %grown, null\n  br i1 %refused, label %exhausted, label %ready\nexhausted:\n  call void @wf_resource_abort()\n  unreachable\nready:\n  store ptr %grown, ptr %work\n  store i64 %wanted, ptr %capacity.slot\n  br label %store\nstore:\n  %entries = load ptr, ptr %work\n  %slot = getelementptr inbounds %wf.drop.entry, ptr %entries, i64 %count\n  %node.slot = getelementptr inbounds %wf.drop.entry, ptr %slot, i32 0, i32 1\n  store i32 %kind, ptr %slot\n  store ptr %node, ptr %node.slot\n  %after = add nuw i64 %count, 1\n  store i64 %after, ptr %count.slot\n  ret void\n}\n\n";

const DROP_WORKLIST_LOOP_HEAD: &str = "define private void @wf.drop.run(ptr %work) {\nentry:\n  %count.slot = getelementptr inbounds %wf.drop.work, ptr %work, i32 0, i32 1\n  br label %loop\nloop:\n  %count = load i64, ptr %count.slot\n  %empty = icmp eq i64 %count, 0\n  br i1 %empty, label %done, label %take\ntake:\n  %next = sub i64 %count, 1\n  store i64 %next, ptr %count.slot\n  %entries = load ptr, ptr %work\n  %slot = getelementptr inbounds %wf.drop.entry, ptr %entries, i64 %next\n  %node.slot = getelementptr inbounds %wf.drop.entry, ptr %slot, i32 0, i32 1\n  %kind = load i32, ptr %slot\n  %node = load ptr, ptr %node.slot\n  switch i32 %kind, label %invalid [\n";

const DROP_WORKLIST_LOOP_TAIL: &str = "invalid:\n  unreachable\ndone:\n  %remaining = load ptr, ptr %work\n  call void @free(ptr %remaining)\n  ret void\n}\n\n";

/// The cleanup edges of one type: what its compiler-derived drop reaches, and
/// whether the edge passes through owning indirection.
///
/// Only an indirection edge can close a cycle — a value that contained itself
/// by value would have no finite layout — so the flag is exactly the set of
/// edges a traversal can be asked to carry on a worklist.
fn cleanup_edges(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<Vec<(IrType, bool)>, BackendFailure> {
    let mut edges = Vec::new();
    match ty {
        IrType::Buffer { element } => {
            if type_requires_cleanup(program, element.ty())? {
                edges.push((element.ty(), true));
            }
        }
        IrType::Nominal(id) => {
            let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
            match nominal.kind() {
                IrNominalKind::Struct { fields } => {
                    for field in fields {
                        if type_requires_cleanup(program, field.ty())? {
                            edges.push((field.ty(), false));
                        }
                    }
                }
                IrNominalKind::Enum { variants } => {
                    for field in variants.iter().flat_map(|variant| variant.fields()) {
                        if type_requires_cleanup(program, field.ty())? {
                            edges.push((field.ty(), false));
                        }
                    }
                }
                IrNominalKind::Box { referent } => {
                    if type_requires_cleanup(program, *referent)? {
                        edges.push((*referent, true));
                    }
                }
                IrNominalKind::Arena { .. }
                | IrNominalKind::ArenaStorage
                | IrNominalKind::SystemResource(_) => {}
            }
        }
        // A run's elements are reached through its own storage, which is an
        // owning indirection for the store-resident run and an inline block
        // for the frame-resident one [BLK-1].
        IrType::Vector { element } => {
            if type_requires_cleanup(program, element.ty())? {
                edges.push((element.ty(), true));
            }
        }
        IrType::FixedVector { element, .. } => {
            if type_requires_cleanup(program, element.ty())? {
                edges.push((element.ty(), false));
            }
        }
        IrType::Unit
        | IrType::Bool
        | IrType::Integer { .. }
        | IrType::Float { .. }
        | IrType::Array { .. }
        | IrType::Slice { .. }
        | IrType::Provider
        | IrType::Address(_) => {}
    }
    Ok(edges)
}

/// Every cleanup-requiring type that can reach itself, mapped to the component
/// it reaches itself through.
///
/// Tarjan's algorithm over the cleanup graph, iterative because the graph this
/// analysis exists to find is exactly the one a recursive walk of it would
/// descend.
fn recursive_cleanup_components(
    program: &IrProgram<'_, '_, '_>,
) -> Result<HashMap<IrType, usize>, BackendFailure> {
    let mut types: Vec<IrType> = Vec::new();
    let mut index_of: HashMap<IrType, usize> = HashMap::new();
    let mut edges: Vec<Vec<usize>> = Vec::new();
    let mut indirect: Vec<Vec<bool>> = Vec::new();
    let mut pending: Vec<usize> = Vec::new();
    for ty in program_types(program) {
        if !type_requires_cleanup(program, ty)? {
            continue;
        }
        if index_of.contains_key(&ty) {
            continue;
        }
        index_of.insert(ty, types.len());
        types.push(ty);
        edges.push(Vec::new());
        indirect.push(Vec::new());
        pending.push(types.len() - 1);
    }
    while let Some(node) = pending.pop() {
        for (target, through_indirection) in cleanup_edges(program, types[node])? {
            let index = match index_of.get(&target) {
                Some(index) => *index,
                None => {
                    let index = types.len();
                    index_of.insert(target, index);
                    types.push(target);
                    edges.push(Vec::new());
                    indirect.push(Vec::new());
                    pending.push(index);
                    index
                }
            };
            edges[node].push(index);
            indirect[node].push(through_indirection);
        }
    }

    let count = types.len();
    let mut order = vec![usize::MAX; count];
    let mut low = vec![0_usize; count];
    let mut on_stack = vec![false; count];
    let mut component = vec![usize::MAX; count];
    let mut stack: Vec<usize> = Vec::new();
    let mut frames: Vec<(usize, usize)> = Vec::new();
    let mut next_order = 0_usize;
    let mut components = 0_usize;
    for root in 0..count {
        if order[root] != usize::MAX {
            continue;
        }
        frames.push((root, 0));
        order[root] = next_order;
        low[root] = next_order;
        next_order += 1;
        stack.push(root);
        on_stack[root] = true;
        while let Some((node, cursor)) = frames.last_mut() {
            let node = *node;
            if *cursor < edges[node].len() {
                let target = edges[node][*cursor];
                *cursor += 1;
                if order[target] == usize::MAX {
                    order[target] = next_order;
                    low[target] = next_order;
                    next_order += 1;
                    stack.push(target);
                    on_stack[target] = true;
                    frames.push((target, 0));
                } else if on_stack[target] {
                    low[node] = low[node].min(order[target]);
                }
                continue;
            }
            frames.pop();
            if let Some((parent, _)) = frames.last() {
                low[*parent] = low[*parent].min(low[node]);
            }
            if low[node] == order[node] {
                while let Some(member) = stack.pop() {
                    on_stack[member] = false;
                    component[member] = components;
                    if member == node {
                        break;
                    }
                }
                components += 1;
            }
        }
    }

    // A component with one member is recursive only when that member reaches
    // itself, which for a cleanup graph means an indirection edge to its own
    // type.
    let mut sizes = vec![0_usize; components];
    for node in 0..count {
        sizes[component[node]] += 1;
    }
    let mut recursive = HashMap::new();
    for node in 0..count {
        let members = sizes[component[node]];
        let reaches_itself = members > 1 || edges[node].contains(&node);
        if reaches_itself {
            recursive.insert(types[node], component[node]);
        }
    }
    Ok(recursive)
}
