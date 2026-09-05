use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;

use crate::{IrFlatElement, IrReleaseClass, IrVariant};

use super::super::qualification::Qualification;
use super::super::target::TargetLayout;
use super::{
    BackendFailure, IrNominalId, IrNominalKind, IrProgram, IrType, llvm_type, nominal_symbol,
    system, variant_field_base,
};

/// One release action per node type of the release graph [PROV-6].
///
/// A type whose release graph has a cycle enters its own release action where
/// the graph closes, so the walk's depth is the value's rather than the
/// type's. That is the owner's ruling of 2026-09-04, which deleted the cycle
/// refusal this emitter used to work around: a cycle can arise only where a
/// heap is allowed, and a heap-allowed program's resource behaviour is a
/// runtime quantity already. The explicit worklist that ran such a walk off
/// the machine stack is gone with it, and so is the one release-path caller of
/// `wf_resource_abort` — the worklist allocated, and an allocation on the
/// release path is a runtime trap the writer never wrote.
pub(super) fn emit_resource_drop_helpers(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    _target: TargetLayout,
) -> Result<String, BackendFailure> {
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
        )?;
        output.push_str("}\n\n");
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
    for (index, ty) in cleanup_run_types(program)?.into_iter().enumerate() {
        emit_run_drop_helper(program, qualification, &mut output, index, ty)?;
    }
    Ok(output)
}

/// [PROV-6, BLK-1] one run's release: its window is visited, in ascending
/// logical order, and only then is its own backing released.
///
/// The walk is over the window and not over the capacity, because a slot
/// outside the window is raw [BLK-1] and reading it would be an uninitialized
/// read. The physical slot of logical offset `i` is `(head + i) mod cap`,
/// which is the one conditional subtract a subscript already emits.
///
/// The backing release itself is empty in this version and is emitted after
/// the walk: a frame-resident run reclaims no storage of its own, and every
/// store-resident run this version can form is taken from a bump extent, whose
/// region reset reclaims the whole extent [BLK-2]. The general store's free
/// lands with `heap_vector`, and it lands *here*, after the loop, which is what
/// [PROV-6]'s ordering requires.
fn emit_run_drop_helper(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    output: &mut String,
    index: usize,
    ty: IrType,
) -> Result<(), BackendFailure> {
    let run_llvm = llvm_type(program, ty)?;
    let symbol = run_drop_helper_symbol(index);
    writeln!(
        output,
        "define private void @{symbol}({run_llvm} %value) {{\nentry:"
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    let element = match ty {
        // A frame-resident run's slots are inside its own value, so the walk
        // needs an address for it; its capacity is the type constant.
        IrType::FixedVector { element, length } => {
            writeln!(
                output,
                "  %storage = alloca {run_llvm}\n  store {run_llvm} %value, ptr %storage\n  %pointer = getelementptr inbounds {run_llvm}, ptr %storage, i64 0, i32 0, i64 0\n  %capacity = add i64 {length}, 0\n  %length = extractvalue {run_llvm} %value, 1\n  %origin = extractvalue {run_llvm} %value, 2"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            element
        }
        // A store-resident run's slots are behind its descriptor pointer.
        IrType::Vector { element, .. } => {
            writeln!(
                output,
                "  %pointer = extractvalue {run_llvm} %value, 0\n  %capacity = extractvalue {run_llvm} %value, 1\n  %length = extractvalue {run_llvm} %value, 2\n  %origin = extractvalue {run_llvm} %value, 3"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            element
        }
        _ => return Err(BackendFailure::InvalidIr),
    };
    let element_ty = element.ty();
    let element_llvm = llvm_type(program, element_ty)?;
    writeln!(
        output,
        "  br label %walk\nwalk:\n  %index = phi i64 [ 0, %entry ], [ %next, %body ]\n  %continue = icmp ult i64 %index, %length\n  br i1 %continue, label %body, label %done\nbody:\n  %raw = add i64 %origin, %index\n  %over = icmp uge i64 %raw, %capacity\n  %reduced = sub i64 %raw, %capacity\n  %physical = select i1 %over, i64 %reduced, i64 %raw\n  %element.pointer = getelementptr inbounds {element_llvm}, ptr %pointer, i64 %physical\n  %element = load {element_llvm}, ptr %element.pointer"
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    let mut temporary = 0_u32;
    emit_value_cleanup(
        program,
        qualification,
        output,
        &mut temporary,
        element_ty,
        "%element".to_owned(),
    )?;
    output.push_str("  %next = add i64 %index, 1\n  br label %walk\ndone:\n  ret void\n}\n\n");
    Ok(())
}

/// Every run type in the program whose window holds values deriving a release
/// action, in deterministic order.
///
/// The one-level lift [BLK-1] means an element run's own element is flat, so
/// closing the enumeration over element types takes one extra pass and not a
/// fixed point.
fn cleanup_run_types(program: &IrProgram<'_, '_, '_>) -> Result<Vec<IrType>, BackendFailure> {
    let mut candidates: Vec<IrType> = Vec::new();
    for ty in program_types(program) {
        if !matches!(ty, IrType::FixedVector { .. } | IrType::Vector { .. }) {
            continue;
        }
        if !candidates.contains(&ty) {
            candidates.push(ty);
        }
        let (IrType::FixedVector { element, .. } | IrType::Vector { element, .. }) = ty else {
            continue;
        };
        let inner = element.ty();
        if matches!(inner, IrType::FixedVector { .. } | IrType::Vector { .. })
            && !candidates.contains(&inner)
        {
            candidates.push(inner);
        }
    }
    let mut needed = Vec::new();
    for ty in candidates {
        let (IrType::FixedVector { element, .. } | IrType::Vector { element, .. }) = ty else {
            continue;
        };
        if type_requires_cleanup(program, element.ty())? {
            needed.push(ty);
        }
    }
    Ok(needed)
}

fn run_drop_helper_symbol(index: usize) -> String {
    format!("wf.drop.run.{index}")
}

/// The helper one run type's release walk is emitted as, when its window holds
/// values that derive a release action.
fn run_drop_helper(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<Option<String>, BackendFailure> {
    Ok(cleanup_run_types(program)?
        .into_iter()
        .position(|candidate| candidate == ty)
        .map(run_drop_helper_symbol))
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
            // A run's own backing action is its release class [PROV-6]: a
            // general store's run spends that store's provider capability, and
            // a bump extent's run is reclaimed by its own region reset, which
            // is no action at all. A frame-resident run reclaims none of its
            // own either. Every run still needs a walk when its window holds
            // values that derive one, and [PROV-6] visits those elements
            // before the backing is released [STOR-3, BLK-1].
            IrType::Vector {
                release: IrReleaseClass::General,
                ..
            } => return Ok(true),
            IrType::Vector { element, .. } | IrType::FixedVector { element, .. } => {
                pending.push(element.ty());
            }
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
    )
}

fn emit_cleanup_jobs(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    output: &mut String,
    temporary: &mut u32,
    mut jobs: Vec<CleanupJob>,
) -> Result<(), BackendFailure> {
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
                        writeln!(
                            output,
                            "  call void @{}({} {operand})",
                            buffer_drop_helper_symbol(id),
                            llvm_type(program, ty)?
                        )
                        .map_err(|_| BackendFailure::TextEmission)?;
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
                            // The one release action of this node type
                            // [PROV-6]. Where the release graph closes on
                            // itself this is the recursive edge, and the
                            // depth is the value's own.
                            if type_requires_cleanup(program, ty)? {
                                writeln!(
                                    output,
                                    "  call void @{}({} {operand})",
                                    drop_helper_symbol(id),
                                    nominal_symbol(id)
                                )
                                .map_err(|_| BackendFailure::TextEmission)?;
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
                // A run's release visits its window and then releases its own
                // backing [PROV-6, BLK-1], and the helper is what carries
                // that order. A frame-resident run reclaims no storage of its
                // own, and every store-resident run this version can form is
                // taken from a bump extent, whose region reset reclaims the
                // whole extent [BLK-2] — so both backings are empty here and
                // a run whose window derives no action emits nothing at all.
                // The general store's free lands with `heap_vector`, after
                // the walk the helper already performs.
                IrType::Vector { element, .. } | IrType::FixedVector { element, .. } => {
                    if type_requires_cleanup(program, element.ty())?
                        && let Some(symbol) = run_drop_helper(program, ty)?
                    {
                        let run_llvm = llvm_type(program, ty)?;
                        writeln!(output, "  call void @{symbol}({run_llvm} {operand})")
                            .map_err(|_| BackendFailure::TextEmission)?;
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
        emit_cleanup_jobs(program, qualification, output, &mut temporary, jobs)?;
        output.push_str("  br label %done\n");
    }

    output.push_str("invalid:\n  call void @abort()\n  unreachable\ndone:\n  ret void\n");
    Ok(())
}
