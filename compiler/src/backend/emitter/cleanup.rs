use std::collections::HashSet;
use std::fmt::Write;

use super::{
    BackendFailure, IrNominalId, IrNominalKind, IrProgram, IrType, llvm_type, nominal_symbol,
    variant_field_base,
};

pub(super) fn emit_resource_drop_helpers(
    program: &IrProgram<'_, '_, '_>,
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
        writeln!(
            output,
            "define private void @{}({aggregate_ty} %value) {{\nentry:\n  %tag = extractvalue {aggregate_ty} %value, 0",
            drop_helper_symbol(nominal.id())
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
            writeln!(output, "variant.{}:", variant.tag())
                .map_err(|_| BackendFailure::TextEmission)?;
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
            emit_cleanup_jobs(program, &mut output, &mut temporary, jobs)?;
            output.push_str("  br label %done\n");
        }

        output.push_str("invalid:\n  call void @abort()\n  unreachable\ndone:\n  ret void\n}\n\n");
    }
    Ok(output)
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
                    IrNominalKind::Box { .. } => return Ok(true),
                }
            }
            IrType::Unit
            | IrType::Bool
            | IrType::Integer { .. }
            | IrType::Float { .. }
            | IrType::Array { .. }
            | IrType::Slice { .. }
            | IrType::NominalAddress(_)
            | IrType::GuardedArrayIndex { .. }
            | IrType::GuardedBufferIndex { .. }
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
    output: &mut String,
    temporary: &mut u32,
    ty: IrType,
    operand: String,
) -> Result<(), BackendFailure> {
    emit_cleanup_jobs(
        program,
        output,
        temporary,
        vec![CleanupJob::Value { ty, operand }],
    )
}

fn emit_cleanup_jobs(
    program: &IrProgram<'_, '_, '_>,
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
                IrType::Buffer { .. } => {
                    let pointer = next_temporary(temporary)?;
                    writeln!(
                        output,
                        "  %{pointer} = extractvalue {} {operand}, 0\n  call void @free(ptr %{pointer})",
                        llvm_type(program, ty)?
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
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
                                writeln!(
                                    output,
                                    "  call void @{}({} {operand})",
                                    drop_helper_symbol(id),
                                    nominal_symbol(id)
                                )
                                .map_err(|_| BackendFailure::TextEmission)?;
                            }
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
                    }
                }
                IrType::Unit
                | IrType::Bool
                | IrType::Integer { .. }
                | IrType::Float { .. }
                | IrType::Array { .. }
                | IrType::Slice { .. }
                | IrType::NominalAddress(_)
                | IrType::GuardedArrayIndex { .. }
                | IrType::GuardedBufferIndex { .. } => {}
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
