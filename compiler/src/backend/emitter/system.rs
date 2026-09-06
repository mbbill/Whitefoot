//! Native emission for the qualified [SYS-2] system interface.
//!
//! [QUAL-3] fixes the emitted shape for a natively compiled command: selection
//! is static for the whole build, so this module emits one private definition
//! per approved implementation and one direct call to its private ABI symbol
//! at each use site. The emitted program therefore contains no runtime
//! operation-ID switch, target tag, per-call dispatch table, instance handle
//! table, or handle lookup. Each wrapper is `alwaysinline`, which is the
//! condition of qualification [QUAL-3] states: the compiler wrapper is inlined
//! rather than left as a call on a transfer path.
//!
//! One-time per-invocation normalization belongs to the command bootstrap
//! before entry, never to a transfer, so the bootstrap owns the process,
//! establishes the [QUAL-2] command-lifetime argument backing, installs the
//! ignored disposition for the write-to-closed-pipe signal once, supplies the
//! [FN-7] standard inputs, invokes the entry once, and maps the returned
//! `ExitStatus` onto the host process status exactly [PROG-3].

use super::super::qualification::{
    ApprovedImplementation, DirectoryEnumeration, EntryNameLength, ORIGIN_DESCRIPTOR_STATUS,
    ORIGIN_DIRECTORY_OPEN, ORIGIN_NONE, ORIGIN_READ, ORIGIN_WRITE, ProgramKind, Qualification,
    ReleaseImplementation, SystemTarget, qualified_representation,
};
use super::*;
use crate::ACTIVE_KERNEL_SPEC_VERSION;

/// The compiler-owned Windows system-runtime contract embedded in the driver.
pub const WINDOWS_RUNTIME_HEADER: &str = include_str!("../windows_runtime.h");
/// The compiler-owned Windows system runtime embedded in the driver.
pub const WINDOWS_RUNTIME_SOURCE: &str = include_str!("../windows_runtime.c");

/// The status a start failure ends the process with.
///
/// [PROG-3]: when the selected target cannot supply a declared standard input
/// or the [QUAL-2] backing guarantee, start fails before the entry is invoked,
/// no source statement executes, and no `ExitStatus` is produced. The value is
/// this bootstrap's own operating-system-error convention and is deliberately
/// not an `ExitStatus`: the language defines no process status for a start
/// failure, and this one is never produced by a returned command code path.
const START_FAILURE_STATUS: i32 = 71;

/// The symbol carrying the bootstrap and the call into the program.
///
/// `@main` keeps the host's entry signature and does one thing: hand this
/// function to the floor runtime, which runs it on a stack of the compiler's
/// own size. The floor's translation unit calls this symbol by name, so it has
/// external linkage; the module's weak fallback calls it directly for a link
/// that supplies no floor.
const ENTRY_BODY_SYMBOL: &str = "wf__main_body";

/// The [SYS-2] inventory ordinals this module emits code for.
const ARGS_COUNT: u8 = 0;
const ARG_GET: u8 = 1;
const HOST_BYTES_LEN: u8 = 2;
const HOST_COPY_BYTES: u8 = 3;
const HOST_UTF8_LEN: u8 = 4;
const HOST_COPY_UTF8: u8 = 5;
const RELATIVE_PATH: u8 = 6;
const OPEN_READ: u8 = 7;
const READ_ONCE: u8 = 8;
const WRITE_ONCE: u8 = 9;
const EXIT_STATUS: u8 = 10;
const OPEN_DIRECTORY: u8 = 11;
const OPEN_LIST: u8 = 12;
const LIST_ONCE: u8 = 13;
const OPEN_FILE: u8 = 14;
const RESERVE_FILE: u8 = 15;

/// The finite system operations the first typed file adapter can actualize.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CompletionFileOperation {
    OpenRead,
    Read,
    Write,
    OpenDirectory,
    OpenDirectorySource,
    DirectoryNext,
    OpenFile,
}

pub(super) fn completion_file_operation(
    operation: crate::IrSystemOperation,
) -> Option<CompletionFileOperation> {
    match operation.ordinal() {
        OPEN_READ => Some(CompletionFileOperation::OpenRead),
        READ_ONCE => Some(CompletionFileOperation::Read),
        WRITE_ONCE => Some(CompletionFileOperation::Write),
        OPEN_DIRECTORY => Some(CompletionFileOperation::OpenDirectory),
        OPEN_LIST => Some(CompletionFileOperation::OpenDirectorySource),
        LIST_ONCE => Some(CompletionFileOperation::DirectoryNext),
        OPEN_FILE => Some(CompletionFileOperation::OpenFile),
        _ => None,
    }
}

pub(super) const fn completion_mapper_symbol(operation: CompletionFileOperation) -> &'static str {
    match operation {
        CompletionFileOperation::OpenRead => OPEN_READ_COMPLETION_MAPPER,
        CompletionFileOperation::Read => READ_COMPLETION_MAPPER,
        CompletionFileOperation::Write => WRITE_COMPLETION_MAPPER,
        CompletionFileOperation::OpenDirectory => OPEN_DIRECTORY_COMPLETION_MAPPER,
        CompletionFileOperation::OpenDirectorySource => OPEN_LIST_COMPLETION_MAPPER,
        CompletionFileOperation::DirectoryNext => DIRECTORY_NEXT_COMPLETION_MAPPER,
        CompletionFileOperation::OpenFile => OPEN_FILE_COMPLETION_MAPPER,
    }
}

/// The portable [SYS-14] entry-kind values written into the destination.
const KIND_UNKNOWN: u8 = 0;
const KIND_REGULAR: u8 = 1;
const KIND_DIRECTORY: u8 = 2;
const KIND_SYMLINK: u8 = 3;
const KIND_OTHER: u8 = 4;

/// The portable [SYS-14] entry record header: one kind byte and one
/// little-endian `u16` name length, ahead of the name bytes themselves.
const ENTRY_HEADER: u64 = 3;

/// The private symbol of the shared UTF-8 validator both text-route
/// implementations use [HOST-2].
const UTF8_VALIDATOR: &str = "wf.sys.utf8.valid";

/// The private symbol of the one cold [SYS-7] outcome mapper every failing
/// I/O implementation shares [QUAL-3].
const IO_ERROR_MAPPER: &str = "wf.sys.io.error";

/// Raw target-result mappers shared by the direct specialization and the
/// finite completion route.  Keeping one mapper is what makes the two
/// execution choices produce the same qualified Whitefoot outcome.
const READ_COMPLETION_MAPPER: &str = "wf.sys.read.completion";
const WRITE_COMPLETION_MAPPER: &str = "wf.sys.write.completion";
const OPEN_READ_COMPLETION_MAPPER: &str = "wf.sys.open_read.completion";
const OPEN_DIRECTORY_COMPLETION_MAPPER: &str = "wf.sys.open_directory.completion";
const OPEN_LIST_COMPLETION_MAPPER: &str = "wf.sys.open_directory_source.completion";
const DIRECTORY_NEXT_COMPLETION_MAPPER: &str = "wf.sys.directory_next.completion";
const OPEN_FILE_COMPLETION_MAPPER: &str = "wf.sys.open_file.completion";
pub(super) const OPEN_EXPECT_REGULAR: u32 = 1;
pub(super) const OPEN_EXPECT_DIRECTORY: u32 = 2;
pub(super) const WINDOWS_DESCRIPTOR_CLASS_READ_FILE: u32 = 1;
pub(super) const WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT: u32 = 2;
pub(super) const WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE: u32 = 3;

fn completion_open_declaration(target: SystemTarget, symbol: &str) -> String {
    if target.is_windows() {
        format!("declare i32 @{symbol}(i32, ptr, i32, i32, i32, i32, i32, ptr, ptr)")
    } else {
        format!("declare i32 @{symbol}(i32, ptr, i32, i32, i32, i32, ptr, ptr)")
    }
}

fn windows_descriptor_class_argument(target: SystemTarget, descriptor_class: u32) -> String {
    if target.is_windows() {
        format!(", i32 {descriptor_class}")
    } else {
        String::new()
    }
}

/// The private constant naming the initial working directory.
pub(super) const WORKING_DIRECTORY: &str = "@.wf.sys.working.directory";

/// Everything the qualified system interface adds to one module.
pub(super) struct SystemEmission {
    /// Private constants the bootstrap needs.
    pub(super) constants: String,
    /// Host and intrinsic declarations the approved implementations call.
    ///
    /// These are owned strings because a host facility's symbol is the target
    /// column of its [QUAL-1] row, not a name fixed in this module.
    pub(super) declarations: BTreeSet<String>,
    /// The approved implementations themselves.
    pub(super) definitions: String,
}

/// Emits the approved implementation of every used semantic identity.
pub(super) fn emit_system_interface(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    target_layout: TargetLayout,
) -> Result<SystemEmission, BackendFailure> {
    let mut constants = String::new();
    let mut declarations: BTreeSet<String> = BTreeSet::new();
    let mut definitions = String::new();
    let mut needs_validator = false;
    // The command bootstrap and `open_directory_source` both name the self component, so
    // the constant is emitted once for whichever of them the program uses.
    let mut needs_working_directory = false;

    // [QUAL-3] establishes the emitted shape by inspection of emitted code and
    // symbols, so the module records which approved implementation each
    // semantic identity and each opaque resource resolved to.
    let mut record = String::new();
    for (ordinal, implementation) in qualification.used_operations() {
        writeln!(
            record,
            "; QUAL-1 semantic id {ordinal} -> @{} implementation version {}",
            implementation.symbol(),
            implementation.version()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    for nominal in program.nominals() {
        let IrNominalKind::SystemResource(contract) = nominal.kind() else {
            continue;
        };
        let resource = qualification.resource(contract.resource)?;
        writeln!(
            record,
            "; QUAL-1 resource {:?} -> {:?} implementation version {}",
            contract.resource,
            resource.representation(),
            resource.version()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    if !record.is_empty() {
        writeln!(
            definitions,
            "; QUAL-1 qualification: specification {ACTIVE_KERNEL_SPEC_VERSION}, program kind {:?}",
            qualification.kind()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        definitions.push_str(&record);
        definitions.push('\n');
    }

    let results = system_call_results(program)?;
    let target = qualification.target();
    // The three failing I/O implementations share one cold [SYS-7] mapper, so
    // the module resolves the `IoError` type once from whichever outcome the
    // program actually uses.
    let mut io_error = None;
    for (ordinal, implementation) in qualification.used_operations() {
        let result = results
            .get(usize::from(ordinal))
            .copied()
            .flatten()
            .ok_or(BackendFailure::InvalidIr)?;
        match ordinal {
            ARGS_COUNT => definitions.push_str(&emit_args_count(implementation)),
            ARG_GET => {
                definitions.push_str(&emit_arg_get(program, implementation, result, target)?)
            }
            HOST_BYTES_LEN => definitions.push_str(&emit_host_bytes_len(implementation, target)),
            HOST_COPY_BYTES => {
                definitions.push_str(&emit_host_copy_bytes(
                    program,
                    implementation,
                    result,
                    target,
                )?);
            }
            HOST_UTF8_LEN => {
                needs_validator = !target.is_windows();
                definitions.push_str(&emit_host_utf8_len(
                    program,
                    qualification,
                    implementation,
                    result,
                    target,
                    target_layout,
                )?);
            }
            HOST_COPY_UTF8 => {
                needs_validator = !target.is_windows();
                definitions.push_str(&emit_host_copy_utf8(
                    program,
                    qualification,
                    implementation,
                    result,
                    target,
                    target_layout,
                )?);
            }
            RELATIVE_PATH => definitions.push_str(&emit_relative_path(
                program,
                implementation,
                result,
                target,
            )?),
            OPEN_READ => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_open_read(
                    program,
                    qualification,
                    implementation,
                    &shape,
                    target,
                    target_layout,
                )?);
            }
            READ_ONCE => {
                let shape = read_outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.failed_type)?;
                definitions.push_str(&emit_read_at(program, implementation, &shape, target)?);
            }
            WRITE_ONCE => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_write_once(program, implementation, &shape, target)?);
            }
            EXIT_STATUS => definitions.push_str(&emit_exit_status(implementation)),
            OPEN_DIRECTORY => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_open_directory(
                    program,
                    qualification,
                    implementation,
                    &shape,
                    target,
                    target_layout,
                )?);
            }
            OPEN_LIST => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                needs_working_directory = true;
                definitions.push_str(&emit_open_directory_source(
                    program,
                    qualification,
                    implementation,
                    &shape,
                    target,
                    target_layout,
                )?);
            }
            LIST_ONCE => {
                let shape = list_outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.failed_type)?;
                definitions.push_str(&emit_directory_next(
                    program,
                    qualification,
                    implementation,
                    &shape,
                    target,
                    target_layout,
                )?);
            }
            OPEN_FILE => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_open_file(
                    program,
                    qualification,
                    implementation,
                    &shape,
                    target,
                    target_layout,
                )?);
            }
            RESERVE_FILE => definitions.push_str(&emit_reserve_file(implementation)),
            _ => return Err(BackendFailure::InvalidIr),
        }
        for declaration in operation_declarations(ordinal, target)? {
            declarations.insert(declaration);
        }
    }
    if needs_validator {
        definitions.push_str(&emit_utf8_validator());
    }
    if let Some(error) = io_error {
        declarations.insert(target.errno_declaration().to_owned());
        definitions.push_str(&emit_io_error_mapper(program, error, target)?);
    }

    if let Some(symbol) = native_release_symbol(program, qualification)? {
        declarations.insert(format!("declare i32 @{symbol}(i32)"));
    }

    let IrEntry::Command { inputs, .. } = program.entry();
    if target.is_windows() {
        declarations.insert("declare i32 @wf__windows_stdout_descriptor()".to_owned());
        declarations.insert("declare i32 @wf__windows_stderr_descriptor()".to_owned());
    } else {
        declarations.insert("declare ptr @signal(i32, ptr)".to_owned());
    }
    declarations.insert("declare void @exit(i32) noreturn".to_owned());
    if inputs.contains(&1) {
        declarations.insert(format!(
            "declare i32 @{}(ptr, i32, ...)",
            qualification.target().directory_open_symbol()
        ));
        needs_working_directory = true;
    }
    if needs_working_directory {
        let storage = if target.is_windows() {
            TargetStorageType::array(TargetStorageType::integer(16), 2)
        } else {
            TargetStorageType::bytes(2)
        };
        validate_static_storage(target_layout, qualification, program, &storage)
            .map_err(BackendFailure::TargetLayout)?;
        if target.is_windows() {
            constants.push_str(&format!(
                "{WORKING_DIRECTORY} = private unnamed_addr constant {} \
                 [i16 46, i16 0], align 2\n",
                llvm_storage_type(program, &storage)?
            ));
        } else {
            constants.push_str(&format!(
                "{WORKING_DIRECTORY} = private unnamed_addr constant {} c\".\\00\", align 1\n",
                llvm_storage_type(program, &storage)?
            ));
        }
    }

    Ok(SystemEmission {
        constants,
        declarations,
        definitions,
    })
}

/// Records the one `IoError` type this program's I/O outcomes carry.
///
/// Every [SYS-2] outcome carrying `IoError` names the one interned nominal, so
/// two different types here would be an inconsistent IR rather than two
/// mappers.
fn record_io_error(recorded: &mut Option<IrType>, ty: IrType) -> Result<(), BackendFailure> {
    match recorded {
        Some(existing) if *existing != ty => Err(BackendFailure::InvalidIr),
        Some(_) => Ok(()),
        None => {
            *recorded = Some(ty);
            Ok(())
        }
    }
}

/// The host and intrinsic symbols one approved implementation calls.
///
/// The intrinsics and the pure libc routines are the same on every qualified
/// target; the three facilities that reach a real operating-system object are
/// the target column of their [QUAL-1] row, so their symbols come from the
/// selected target.
fn operation_declarations(
    ordinal: u8,
    target: SystemTarget,
) -> Result<Vec<String>, BackendFailure> {
    let fixed: &[&str] = match ordinal {
        ARG_GET if target.is_windows() => &["declare i64 @wf__windows_wcslen(ptr)"],
        ARG_GET => &["declare i64 @strlen(ptr)"],
        HOST_COPY_BYTES => &["declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)"],
        HOST_UTF8_LEN if target.is_windows() => {
            &["declare i32 @wf__windows_utf8_measure(ptr, i64, ptr)"]
        }
        HOST_COPY_UTF8 if target.is_windows() => &[
            "declare i32 @wf__windows_utf8_measure(ptr, i64, ptr)",
            "declare i32 @wf__windows_utf8_copy(ptr, i64, ptr, i64)",
            "declare void @abort() noreturn",
        ],
        HOST_COPY_UTF8 => &["declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)"],
        RELATIVE_PATH if target.is_windows() => {
            &["declare i32 @wf__windows_relative_path_valid(ptr, i64)"]
        }
        RELATIVE_PATH => &["declare ptr @memchr(ptr, i32, i64)"],
        // [PATH-2]: the target's own directory-relative facility, never a
        // prefix concatenated onto a path and resolved against an ambient
        // working directory.
        OPEN_READ | OPEN_LIST => {
            let symbol = target.file_open_symbol();
            return Ok(vec![
                if target.uses_typed_completion_file_adapter() {
                    completion_open_declaration(target, symbol)
                } else {
                    format!("declare i32 @{symbol}(i32, ptr, i32, ...)")
                },
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        OPEN_DIRECTORY => {
            let symbol = target.file_open_symbol();
            return Ok(vec![
                if target.uses_typed_completion_file_adapter() {
                    completion_open_declaration(target, symbol)
                } else {
                    format!("declare i32 @{symbol}(i32, ptr, i32, ...)")
                },
                "declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)".to_owned(),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        OPEN_FILE => {
            let open = target.file_open_symbol();
            let mut declarations = vec![
                if target.uses_typed_completion_file_adapter() {
                    completion_open_declaration(target, open)
                } else {
                    format!("declare i32 @{open}(i32, ptr, i32, ...)")
                },
                "declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)".to_owned(),
                "declare void @abort() noreturn".to_owned(),
            ];
            if !target.uses_typed_completion_file_adapter() {
                declarations.push(format!(
                    "declare i32 @{}(i32, ptr)",
                    target.file_status_symbol()
                ));
                declarations.push(format!("declare i32 @{}(i32)", target.close_symbol()));
            }
            return Ok(declarations);
        }
        READ_ONCE => {
            return Ok(vec![
                format!("declare i64 @{}(i32, ptr, i64, i64)", target.pread_symbol()),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        WRITE_ONCE => {
            return Ok(vec![
                format!("declare i64 @{}(i32, ptr, i64)", target.write_symbol()),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        // The target's own enumeration facility [SYS-14]; qualification
        // already refused a target that supplies none.
        LIST_ONCE => {
            let enumeration = target
                .directory_enumeration()
                .ok_or(BackendFailure::InvalidIr)?;
            /* The C adapter is target-guarded and calls exactly one of these
             * two facilities behind the one private ABI symbol declared
             * below. Refuse to reuse that symbol if a future target
             * contributes a third enumeration ABI or declaration. */
            let admitted = matches!(
                (enumeration.symbol(), enumeration.declaration()),
                (
                    "__getdirentries64",
                    "declare i64 @__getdirentries64(i32, ptr, i64, ptr)"
                ) | ("getdents64", "declare i64 @getdents64(i32, ptr, i64)")
                    | (
                        "wf__windows_directory_batch",
                        "declare i64 @wf__windows_directory_batch(i32, ptr, i64, ptr)"
                    )
            );
            if !admitted {
                return Err(BackendFailure::InvalidIr);
            }
            return Ok(vec![
                "declare i64 @wf__completion_directory_next_direct(i32, ptr, i64, ptr)".to_owned(),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        _ => &[],
    };
    Ok(fixed.iter().map(|text| (*text).to_owned()).collect())
}

/// The close facility this program's releases reach, when any release the
/// program derives is a native close attempt.
///
/// Every type whose [SYS-5] release is a close attempt resolves to the one
/// facility the selected target's [QUAL-1] rows name, so a program never
/// declares two close symbols.
fn native_release_symbol(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
) -> Result<Option<&'static str>, BackendFailure> {
    let mut selected = None;
    for nominal in program.nominals() {
        let IrNominalKind::SystemResource(contract) = nominal.kind() else {
            continue;
        };
        let ReleaseImplementation::NativeClose(symbol) =
            qualification.resource(contract.resource)?.release()
        else {
            continue;
        };
        if selected
            .replace(symbol)
            .is_some_and(|other| other != symbol)
        {
            return Err(BackendFailure::InvalidIr);
        }
    }
    Ok(selected)
}

/// The result type each used semantic identity produces in this program.
///
/// Every call of one identity yields the one outcome type [SYS-6] fixes for
/// it, and prelude instantiations are interned, so two different result types
/// for one identity would be an inconsistent IR rather than two wrappers.
fn system_call_results(
    program: &IrProgram<'_, '_, '_>,
) -> Result<Vec<Option<IrType>>, BackendFailure> {
    let mut results = vec![None; crate::SYSTEM_OPERATIONS.len()];
    for function in program.functions() {
        for block in function.blocks() {
            for instruction in block.instructions() {
                let IrInstruction::Define {
                    ty,
                    operation: IrOperation::SystemCall { operation, .. },
                    ..
                } = instruction
                else {
                    continue;
                };
                let row = crate::SYSTEM_OPERATIONS
                    .get(usize::from(operation.ordinal()))
                    .ok_or(BackendFailure::InvalidIr)?;
                if *ty != catalog_ir_type(program, row.result)? {
                    return Err(BackendFailure::InvalidIr);
                }
                let slot = results
                    .get_mut(usize::from(operation.ordinal()))
                    .ok_or(BackendFailure::InvalidIr)?;
                match slot {
                    Some(recorded) if recorded != ty => return Err(BackendFailure::InvalidIr),
                    Some(_) => {}
                    None => *slot = Some(*ty),
                }
            }
        }
    }
    Ok(results)
}

/// Resolves one exact [SYS-2] table type against retained IR identities.
///
/// ABI-equivalent types are intentionally rejected: signed integers, buffer
/// elements, opaque resources, system outcomes, and prelude `Result`
/// instances keep distinct identities even when LLVM renders them alike.
fn catalog_ir_type(
    program: &IrProgram<'_, '_, '_>,
    ty: crate::SystemTypeRef,
) -> Result<IrType, BackendFailure> {
    Ok(match ty {
        crate::SystemTypeRef::U8 => IrType::Integer {
            width: 8,
            signed: false,
        },
        crate::SystemTypeRef::U32 => IrType::Integer {
            width: 32,
            signed: false,
        },
        crate::SystemTypeRef::U64 => IrType::Integer {
            width: 64,
            signed: false,
        },
        // [SYS-8] an operand class has no one IR type: a range-bearing
        // operand is a `MutSlice<u8>` or a `Slice<u8>` descriptor, or, until
        // the old surface retires, a `buffer<u8>` one. Each member renders as
        // the same `{ ptr, i64 }` pair, which is what the approved
        // implementation's ABI takes; membership is checked at the argument.
        crate::SystemTypeRef::DestinationU8 | crate::SystemTypeRef::SourceU8 => {
            return Err(BackendFailure::InvalidIr);
        }
        crate::SystemTypeRef::Nominal(index) => system_nominal_ir_type(program, index)?,
        crate::SystemTypeRef::Result { ok, err } => {
            let ok = match ok {
                crate::SystemResultPayload::U64 => IrType::Integer {
                    width: 64,
                    signed: false,
                },
                crate::SystemResultPayload::Nominal(index) => {
                    system_nominal_ir_type(program, index)?
                }
            };
            let error = system_nominal_ir_type(program, err)?;
            unique_nominal_type(program, |nominal| {
                if nominal.identity() != crate::IrNominalIdentity::PreludeResult {
                    return Ok(false);
                }
                let IrNominalKind::Enum { variants } = nominal.kind() else {
                    return Ok(false);
                };
                let [ok_variant, err_variant] = variants.as_slice() else {
                    return Ok(false);
                };
                Ok(ok_variant.tag() == 0
                    && err_variant.tag() == 1
                    && matches!(ok_variant.fields(), [field] if field.ty() == ok)
                    && matches!(err_variant.fields(), [field] if field.ty() == error))
            })?
        }
    })
}

/// Whether one emitted argument's IR type is a member of one [SYS-2] operand
/// class [SYS-8]. A parameter naming an exact type is not a class and is
/// judged by [`catalog_ir_type`] equality instead.
fn system_operand_admits(
    declared: crate::SystemTypeRef,
    argument: IrType,
) -> Result<bool, BackendFailure> {
    let element = crate::IrFlatElement::Integer {
        width: 8,
        signed: false,
    };
    Ok(match declared {
        crate::SystemTypeRef::DestinationU8 | crate::SystemTypeRef::SourceU8 => matches!(
            argument,
            IrType::Buffer { element: actual } | IrType::Slice { element: actual }
                if actual == element
        ),
        _ => false,
    })
}

fn system_nominal_ir_type(
    program: &IrProgram<'_, '_, '_>,
    index: u8,
) -> Result<IrType, BackendFailure> {
    let declared = crate::SYSTEM_NOMINALS
        .get(usize::from(index))
        .ok_or(BackendFailure::InvalidIr)?;
    unique_nominal_type(program, |nominal| {
        if nominal.identity() != crate::IrNominalIdentity::System(index) {
            return Ok(false);
        }
        if declared.opaque {
            let expected =
                crate::system_resource_contract(index).ok_or(BackendFailure::InvalidIr)?;
            return Ok(matches!(
                nominal.kind(),
                IrNominalKind::SystemResource(actual) if *actual == expected
            ));
        }
        let IrNominalKind::Enum { variants } = nominal.kind() else {
            return Ok(false);
        };
        let constructors = crate::system_constructors(crate::Inventory::ACTIVE)
            .iter()
            .filter(|constructor| constructor.owner == index)
            .collect::<Vec<_>>();
        if variants.len() != constructors.len() {
            return Ok(false);
        }
        for (ordinal, (variant, constructor)) in variants.iter().zip(constructors).enumerate() {
            if variant.tag()
                != u32::try_from(ordinal).map_err(|_| BackendFailure::CounterOverflow)?
                || variant.fields().len() != constructor.fields.len()
            {
                return Ok(false);
            }
            for (field, expected) in variant.fields().iter().zip(constructor.fields) {
                if field.ty() != catalog_ir_type(program, expected.ty)? {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    })
}

fn unique_nominal_type(
    program: &IrProgram<'_, '_, '_>,
    mut matches: impl FnMut(&crate::IrNominal) -> Result<bool, BackendFailure>,
) -> Result<IrType, BackendFailure> {
    let mut selected = None;
    for nominal in program.nominals() {
        if !matches(nominal)? {
            continue;
        }
        if selected.replace(nominal.id()).is_some() {
            return Err(BackendFailure::InvalidIr);
        }
    }
    selected
        .map(IrType::Nominal)
        .ok_or(BackendFailure::InvalidIr)
}

/// The emitted representation of one opaque [SYS-2] resource type.
fn representation(resource: SystemResourceType) -> &'static str {
    qualified_representation(resource).llvm()
}

/// One [SYS-6] two-outcome result type, resolved from the program's own IR.
struct OutcomeShape {
    llvm: String,
    ok_tag: u32,
    ok_index: usize,
    ok_llvm: String,
    err_tag: u32,
    err_index: usize,
    err_llvm: String,
    err_type: IrType,
}

fn variants_of<'program>(
    program: &'program IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<&'program [crate::IrVariant], BackendFailure> {
    let IrType::Nominal(id) = ty else {
        return Err(BackendFailure::InvalidIr);
    };
    let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
    let IrNominalKind::Enum { variants } = nominal.kind() else {
        return Err(BackendFailure::InvalidIr);
    };
    Ok(variants)
}

/// Resolves one `Result<T, E>` instantiation's tags and field positions.
fn outcome_shape(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<OutcomeShape, BackendFailure> {
    let variants = variants_of(program, ty)?;
    let [ok, err] = variants else {
        return Err(BackendFailure::InvalidIr);
    };
    let ([ok_field], [err_field]) = (ok.fields(), err.fields()) else {
        return Err(BackendFailure::InvalidIr);
    };
    Ok(OutcomeShape {
        llvm: llvm_type(program, ty)?,
        ok_tag: ok.tag(),
        ok_index: variant_field_base(variants, ok.tag())?,
        ok_llvm: llvm_type(program, ok_field.ty())?,
        err_tag: err.tag(),
        err_index: variant_field_base(variants, err.tag())?,
        err_llvm: llvm_type(program, err_field.ty())?,
        err_type: err_field.ty(),
    })
}

/// The tag of the one variant of an outcome enum that carries no field.
fn empty_variant_tag(program: &IrProgram<'_, '_, '_>, ty: IrType) -> Result<u32, BackendFailure> {
    let variants = variants_of(program, ty)?;
    let mut selected = None;
    for variant in variants {
        if !variant.fields().is_empty() {
            continue;
        }
        if selected.replace(variant.tag()).is_some() {
            return Err(BackendFailure::InvalidIr);
        }
    }
    selected.ok_or(BackendFailure::InvalidIr)
}

/// The tag and payload position of the one variant carrying a single `u64`.
fn measured_variant(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<(u32, usize), BackendFailure> {
    let variants = variants_of(program, ty)?;
    let measured = IrType::Integer {
        width: 64,
        signed: false,
    };
    let mut selected = None;
    for variant in variants {
        let [field] = variant.fields() else {
            continue;
        };
        if field.ty() != measured {
            continue;
        }
        if selected.replace(variant.tag()).is_some() {
            return Err(BackendFailure::InvalidIr);
        }
    }
    let tag = selected.ok_or(BackendFailure::InvalidIr)?;
    Ok((tag, variant_field_base(variants, tag)?))
}

/// Renders an error value that carries no payload.
///
/// A tag-only enum is its tag, so the value costs no instruction.
fn empty_error(program: &IrProgram<'_, '_, '_>, ty: IrType) -> Result<String, BackendFailure> {
    let IrType::Nominal(id) = ty else {
        return Err(BackendFailure::InvalidIr);
    };
    let tag = empty_variant_tag(program, ty)?;
    let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
    if !nominal.is_tag_only_enum() {
        return Err(BackendFailure::InvalidIr);
    }
    Ok(tag.to_string())
}

fn emit_args_count(implementation: ApprovedImplementation) -> String {
    let args = representation(SystemResourceType::Args);
    format!(
        "define private i64 @{}({args} %args) alwaysinline {{\n\
         entry:\n  \
         %count = extractvalue {args} %args, 1\n  \
         ret i64 %count\n\
         }}\n\n",
        implementation.symbol()
    )
}

fn emit_arg_get(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    result: IrType,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let shape = outcome_shape(program, result)?;
    let args = representation(SystemResourceType::Args);
    let lease = representation(SystemResourceType::HostString);
    if shape.ok_llvm != lease {
        return Err(BackendFailure::InvalidIr);
    }
    let absent = empty_error(program, shape.err_type)?;
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    // The lease is an address and a length taken directly out of the
    // command-lifetime backing: no allocation, no byte copy, and no Unicode
    // restriction on the raw byte route [SYS-9, HOST-3].
    let length = if target.is_windows() {
        "wf__windows_wcslen"
    } else {
        "strlen"
    };
    Ok(format!(
        "define private {llvm} @{symbol}({args} %args, i64 %position) alwaysinline {{\n\
         entry:\n  \
         %count = extractvalue {args} %args, 1\n  \
         %present = icmp ult i64 %position, %count\n  \
         br i1 %present, label %found, label %absent\n\
         found:\n  \
         %base = extractvalue {args} %args, 0\n  \
         %slot = getelementptr inbounds ptr, ptr %base, i64 %position\n  \
         %text = load ptr, ptr %slot\n  \
         %length = call i64 @{length}(ptr %text)\n  \
         %lease.base = insertvalue {lease} zeroinitializer, ptr %text, 0\n  \
         %lease.value = insertvalue {lease} %lease.base, i64 %length, 1\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {lease} %lease.value, {ok_index}\n  \
         ret {llvm} %ok\n\
         absent:\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} {absent}, {err_index}\n  \
         ret {llvm} %err\n\
         }}\n\n",
        symbol = implementation.symbol()
    ))
}

fn emit_host_bytes_len(implementation: ApprovedImplementation, target: SystemTarget) -> String {
    let lease = representation(SystemResourceType::HostString);
    let measure = if target.is_windows() {
        format!(
            "  %units = extractvalue {lease} %value, 1\n  \
             %length = shl i64 %units, 1\n"
        )
    } else {
        format!("  %length = extractvalue {lease} %value, 1\n")
    };
    format!(
        "define private i64 @{}({lease} %value) alwaysinline {{\n\
         entry:\n  \
         {measure}  \
         ret i64 %length\n\
         }}\n\n",
        implementation.symbol()
    )
}

fn emit_host_utf8_len(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    result: IrType,
    target: SystemTarget,
    target_layout: TargetLayout,
) -> Result<String, BackendFailure> {
    let shape = outcome_shape(program, result)?;
    let lease = representation(SystemResourceType::HostString);
    let invalid = empty_error(program, shape.err_type)?;
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    if target.is_windows() {
        let prologue = render_named_target_frame(
            program,
            qualification,
            target_layout,
            &[(
                "%encoded.length",
                TargetFrameSlot::natural(TargetStorageType::integer(64)),
            )],
        )?;
        return Ok(format!(
            "define private {llvm} @{symbol}({lease} %value) alwaysinline {{\n\
             entry:\n\
             {prologue}  \
             %text = extractvalue {lease} %value, 0\n  \
             %units = extractvalue {lease} %value, 1\n  \
             %valid.native = call i32 @wf__windows_utf8_measure(ptr %text, i64 %units, \
             ptr %encoded.length)\n  \
             %valid = icmp eq i32 %valid.native, 1\n  \
             br i1 %valid, label %encoded, label %invalid\n\
             encoded:\n  \
             %length = load i64, ptr %encoded.length, align 8\n  \
             %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
             %ok = insertvalue {llvm} %ok.tag, i64 %length, {ok_index}\n  \
             ret {llvm} %ok\n\
             invalid:\n  \
             %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %err = insertvalue {llvm} %err.tag, {err_llvm} {invalid}, {err_index}\n  \
             ret {llvm} %err\n\
             }}\n\n",
            symbol = implementation.symbol()
        ));
    }
    // On a family whose native code unit is exactly one byte, a valid
    // sequence's UTF-8 encoding is the sequence itself, so the exact encoded
    // length is the byte length [HOST-2, SYS-9]. Validation is complete: the
    // route never emits a replacement code point or a truncated encoding.
    Ok(format!(
        "define private {llvm} @{symbol}({lease} %value) alwaysinline {{\n\
         entry:\n  \
         %text = extractvalue {lease} %value, 0\n  \
         %length = extractvalue {lease} %value, 1\n  \
         %valid = call i1 @{UTF8_VALIDATOR}(ptr %text, i64 %length)\n  \
         br i1 %valid, label %encoded, label %invalid\n\
         encoded:\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, i64 %length, {ok_index}\n  \
         ret {llvm} %ok\n\
         invalid:\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} {invalid}, {err_index}\n  \
         ret {llvm} %err\n\
         }}\n\n",
        symbol = implementation.symbol()
    ))
}

/// Starts one statically discharged half-open range operation. `sub nuw` is
/// justified by SYS-8's exact `start <= end` call-site obligation; the other
/// obligation proves `end <= len_of(buffer)`, so this wrapper has no check or
/// runtime-failure fallback.
fn range_entry(prologue: &str) -> String {
    format!(
        "entry:\n\
         {prologue}  \
         %extent = sub nuw i64 %end, %start\n  \
         br label %measure\n"
    )
}

fn emit_host_copy_bytes(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    result: IrType,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let shape = outcome_shape(program, result)?;
    let lease = representation(SystemResourceType::HostString);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    let (small_tag, small_index) = measured_variant(program, shape.err_type)?;
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    let entry = range_entry("");
    let measure = if target.is_windows() {
        format!(
            "  %units = extractvalue {lease} %value, 1\n  \
             %required = shl i64 %units, 1\n"
        )
    } else {
        format!("  %required = extractvalue {lease} %value, 1\n")
    };
    // The lossless route transfers the target's own code units with no
    // validation and no Unicode restriction [HOST-2]; its only recoverable
    // failure is a destination too small for the exact length, which leaves
    // the whole destination buffer unchanged [SYS-8].
    Ok(format!(
        "define private {llvm} @{symbol}({lease} %value, {buffer} %destination, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         {measure}  \
         %room = icmp ule i64 %required, %extent\n  \
         br i1 %room, label %transfer, label %small\n\
         transfer:\n  \
         %source = extractvalue {lease} %value, 0\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %target = getelementptr i8, ptr %base, i64 %start\n  \
         call void @llvm.memcpy.p0.p0.i64(ptr %target, ptr %source, i64 %required, i1 false)\n  \
         %next = add nuw i64 %start, %required\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, i64 %next, {ok_index}\n  \
         ret {llvm} %ok\n\
         small:\n  \
         %small.tag = insertvalue {err_llvm} zeroinitializer, i32 {small_tag}, 0\n  \
         %small.value = insertvalue {err_llvm} %small.tag, i64 %required, {small_index}\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} %small.value, {err_index}\n  \
         ret {llvm} %err\n\
         }}\n\n",
        symbol = implementation.symbol()
    ))
}

fn emit_host_copy_utf8(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    result: IrType,
    target: SystemTarget,
    target_layout: TargetLayout,
) -> Result<String, BackendFailure> {
    let shape = outcome_shape(program, result)?;
    let lease = representation(SystemResourceType::HostString);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    let (small_tag, small_index) = measured_variant(program, shape.err_type)?;
    let invalid_tag = empty_variant_tag(program, shape.err_type)?;
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    let prologue = if target.is_windows() {
        render_named_target_frame(
            program,
            qualification,
            target_layout,
            &[(
                "%required.slot",
                TargetFrameSlot::natural(TargetStorageType::integer(64)),
            )],
        )?
    } else {
        String::new()
    };
    let entry = range_entry(&prologue);
    if target.is_windows() {
        return Ok(format!(
            "define private {llvm} @{symbol}({lease} %value, {buffer} %destination, i64 %start, \
             i64 %end) alwaysinline {{\n\
             {entry}\
             measure:\n  \
             %text = extractvalue {lease} %value, 0\n  \
             %units = extractvalue {lease} %value, 1\n  \
             %valid.native = call i32 @wf__windows_utf8_measure(ptr %text, i64 %units, \
             ptr %required.slot)\n  \
             %valid = icmp eq i32 %valid.native, 1\n  \
             br i1 %valid, label %fit, label %invalid\n\
             fit:\n  \
             %required = load i64, ptr %required.slot, align 8\n  \
             %room = icmp ule i64 %required, %extent\n  \
             br i1 %room, label %transfer, label %small\n\
             transfer:\n  \
             %base = extractvalue {buffer} %destination, 0\n  \
             %target = getelementptr inbounds i8, ptr %base, i64 %start\n  \
             %copied.native = call i32 @wf__windows_utf8_copy(ptr %text, i64 %units, \
             ptr %target, i64 %required)\n  \
             %copied = icmp eq i32 %copied.native, 1\n  \
             br i1 %copied, label %complete, label %tcb.defect\n\
             complete:\n  \
             %next = add nuw i64 %start, %required\n  \
             %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
             %ok = insertvalue {llvm} %ok.tag, i64 %next, {ok_index}\n  \
             ret {llvm} %ok\n\
             small:\n  \
             %small.tag = insertvalue {err_llvm} zeroinitializer, i32 {small_tag}, 0\n  \
             %small.value = insertvalue {err_llvm} %small.tag, i64 %required, {small_index}\n  \
             %small.err = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %small.outcome = insertvalue {llvm} %small.err, {err_llvm} %small.value, \
             {err_index}\n  \
             ret {llvm} %small.outcome\n\
             invalid:\n  \
             %invalid.value = insertvalue {err_llvm} zeroinitializer, i32 {invalid_tag}, 0\n  \
             %invalid.err = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %invalid.outcome = insertvalue {llvm} %invalid.err, {err_llvm} %invalid.value, \
             {err_index}\n  \
             ret {llvm} %invalid.outcome\n\
             tcb.defect:\n  \
             call void @abort()\n  \
             unreachable\n\
             }}\n\n",
            symbol = implementation.symbol()
        ));
    }
    // The text route validates and measures the encoding first and returns
    // the invalid or too-small outcome without writing any byte; only then
    // does it copy the complete encoding [SYS-8, HOST-2].
    Ok(format!(
        "define private {llvm} @{symbol}({lease} %value, {buffer} %destination, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         %text = extractvalue {lease} %value, 0\n  \
         %required = extractvalue {lease} %value, 1\n  \
         %valid = call i1 @{UTF8_VALIDATOR}(ptr %text, i64 %required)\n  \
         br i1 %valid, label %fit, label %invalid\n\
         fit:\n  \
         %room = icmp ule i64 %required, %extent\n  \
         br i1 %room, label %transfer, label %small\n\
         transfer:\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %target = getelementptr i8, ptr %base, i64 %start\n  \
         call void @llvm.memcpy.p0.p0.i64(ptr %target, ptr %text, i64 %required, i1 false)\n  \
         %next = add nuw i64 %start, %required\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, i64 %next, {ok_index}\n  \
         ret {llvm} %ok\n\
         small:\n  \
         %small.tag = insertvalue {err_llvm} zeroinitializer, i32 {small_tag}, 0\n  \
         %small.value = insertvalue {err_llvm} %small.tag, i64 %required, {small_index}\n  \
         %small.err = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %small.outcome = insertvalue {llvm} %small.err, {err_llvm} %small.value, {err_index}\n  \
         ret {llvm} %small.outcome\n\
         invalid:\n  \
         %invalid.value = insertvalue {err_llvm} zeroinitializer, i32 {invalid_tag}, 0\n  \
         %invalid.err = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %invalid.outcome = insertvalue {llvm} %invalid.err, {err_llvm} %invalid.value, \
         {err_index}\n  \
         ret {llvm} %invalid.outcome\n\
         }}\n\n",
        symbol = implementation.symbol()
    ))
}

fn emit_relative_path(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    result: IrType,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let shape = outcome_shape(program, result)?;
    let lease = representation(SystemResourceType::HostString);
    if shape.ok_llvm != lease {
        return Err(BackendFailure::InvalidIr);
    }
    let rejected = empty_error(program, shape.err_type)?;
    let root = u32::from(target.root_prefix());
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    if target.is_windows() {
        return Ok(format!(
            "define private {llvm} @{symbol}({lease} %value) alwaysinline {{\n\
             entry:\n  \
             %text = extractvalue {lease} %value, 0\n  \
             %length = extractvalue {lease} %value, 1\n  \
             %admitted.native = call i32 @wf__windows_relative_path_valid(ptr %text, \
             i64 %length)\n  \
             %admitted = icmp eq i32 %admitted.native, 1\n  \
             br i1 %admitted, label %admit, label %reject\n\
             admit:\n  \
             %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
             %ok = insertvalue {llvm} %ok.tag, {lease} %value, {ok_index}\n  \
             ret {llvm} %ok\n\
             reject:\n  \
             %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %err = insertvalue {llvm} %err.tag, {err_llvm} {rejected}, {err_index}\n  \
             ret {llvm} %err\n\
             }}\n\n",
            symbol = implementation.symbol()
        ));
    }
    // Construction admits exactly a sequence with no NUL code unit that
    // begins with no target-root prefix; on this family that prefix set is one
    // leading separator [PATH-1]. Success retypes the same inline lease with
    // no allocation, no copy, and no code-unit change.
    Ok(format!(
        "define private {llvm} @{symbol}({lease} %value) alwaysinline {{\n\
         entry:\n  \
         %text = extractvalue {lease} %value, 0\n  \
         %length = extractvalue {lease} %value, 1\n  \
         %empty = icmp eq i64 %length, 0\n  \
         br i1 %empty, label %admit, label %inspect\n\
         inspect:\n  \
         %first = load i8, ptr %text\n  \
         %first.value = zext i8 %first to i32\n  \
         %rooted = icmp eq i32 %first.value, {root}\n  \
         br i1 %rooted, label %reject, label %scan\n\
         scan:\n  \
         %embedded = call ptr @memchr(ptr %text, i32 0, i64 %length)\n  \
         %clean = icmp eq ptr %embedded, null\n  \
         br i1 %clean, label %admit, label %reject\n\
         admit:\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {lease} %value, {ok_index}\n  \
         ret {llvm} %ok\n\
         reject:\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} {rejected}, {err_index}\n  \
         ret {llvm} %err\n\
         }}\n\n",
        symbol = implementation.symbol()
    ))
}

/// One [SYS-6] `ReadOutcome` instantiation's tags and field positions.
struct ReadOutcomeShape {
    llvm: String,
    bytes_tag: u32,
    bytes_index: usize,
    end_tag: u32,
    failed_tag: u32,
    failed_index: usize,
    failed_llvm: String,
    failed_type: IrType,
}

/// Resolves the one [SYS-6] outcome type with more than two outcomes.
///
/// `ReadBytes(next: u64)` is the single measured variant, `ReadEnd()` the
/// single empty one, and `ReadFailed(error: IoError)` the single variant
/// carrying a nominal payload, so the three are resolved from the program's
/// own IR rather than from any spelling.
fn read_outcome_shape(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<ReadOutcomeShape, BackendFailure> {
    let variants = variants_of(program, ty)?;
    if variants.len() != 3 {
        return Err(BackendFailure::InvalidIr);
    }
    let (bytes_tag, bytes_index) = measured_variant(program, ty)?;
    let end_tag = empty_variant_tag(program, ty)?;
    let mut selected = None;
    for variant in variants {
        let [field] = variant.fields() else {
            continue;
        };
        if !matches!(field.ty(), IrType::Nominal(_)) {
            continue;
        }
        if selected.replace((variant.tag(), field.ty())).is_some() {
            return Err(BackendFailure::InvalidIr);
        }
    }
    let (failed_tag, failed_type) = selected.ok_or(BackendFailure::InvalidIr)?;
    Ok(ReadOutcomeShape {
        llvm: llvm_type(program, ty)?,
        bytes_tag,
        bytes_index,
        end_tag,
        failed_tag,
        failed_index: variant_field_base(variants, failed_tag)?,
        failed_llvm: llvm_type(program, failed_type)?,
        failed_type,
    })
}

/// One [SYS-7] class's identity and payload positions in one program's
/// `IoError` value.
struct IoErrorClass {
    spelling: &'static str,
    tag: u32,
    /// The position of the class's `code` field; `origin` is the next one.
    code_index: usize,
}

/// Resolves the closed twenty-eight-class [SYS-7] set in one program's `IoError`.
///
/// [SYS-2] fixes the class set, its declared order, and the two inline detail
/// fields every class carries, and the checked program interns the nominal
/// from that same inventory. Resolution therefore reads inventory data and the
/// program's own IR; no source name, spelling, or signature reaches it
/// [QUAL-1].
fn io_error_classes(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<Vec<IoErrorClass>, BackendFailure> {
    let owner = crate::SYSTEM_NOMINALS
        .iter()
        .position(|nominal| nominal.spelling == "IoError")
        .and_then(|index| u8::try_from(index).ok())
        .ok_or(BackendFailure::InvalidIr)?;
    let declared = crate::SYSTEM_CONSTRUCTORS
        .iter()
        .filter(|constructor| constructor.owner == owner);
    let variants = variants_of(program, ty)?;
    let code = IrType::Integer {
        width: 32,
        signed: false,
    };
    let origin = IrType::Integer {
        width: 8,
        signed: false,
    };
    let mut classes = Vec::with_capacity(variants.len());
    for (ordinal, (constructor, variant)) in declared.zip(variants).enumerate() {
        let tag = u32::try_from(ordinal).map_err(|_| BackendFailure::CounterOverflow)?;
        let [first, second] = variant.fields() else {
            return Err(BackendFailure::InvalidIr);
        };
        if variant.tag() != tag || first.ty() != code || second.ty() != origin {
            return Err(BackendFailure::InvalidIr);
        }
        classes.push(IoErrorClass {
            spelling: constructor.spelling,
            tag,
            code_index: variant_field_base(variants, tag)?,
        });
    }
    if classes.len() != variants.len() {
        return Err(BackendFailure::InvalidIr);
    }
    Ok(classes)
}

/// Renders the instructions building one fixed [SYS-7] class value.
///
/// The detail is copy data: it allocates nothing, owns nothing, and has no
/// release action [SYS-7].
fn io_error_value(
    io: &str,
    class: &IoErrorClass,
    prefix: &str,
    code: &str,
    origin: &str,
) -> (String, String) {
    let IoErrorClass {
        tag, code_index, ..
    } = *class;
    let origin_index = code_index + 1;
    (
        format!(
            "  %{prefix}.tag = insertvalue {io} zeroinitializer, i32 {tag}, 0\n  \
             %{prefix}.code = insertvalue {io} %{prefix}.tag, i32 {code}, {code_index}\n  \
             %{prefix}.error = insertvalue {io} %{prefix}.code, i8 {origin}, {origin_index}\n"
        ),
        format!("%{prefix}.error"),
    )
}

/// Renders the two instructions that read the native error slot.
///
/// The slot is read immediately after the failing facility call and only on
/// the cold path [QUAL-3].
fn native_error(target: SystemTarget, prefix: &str) -> (String, String) {
    (
        format!(
            "  %{prefix}.slot = call ptr @{}()\n  \
             %{prefix}.code = load i32, ptr %{prefix}.slot\n",
            target.errno_location()
        ),
        format!("%{prefix}.code"),
    )
}

fn emit_open_read(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
    target_layout: TargetLayout,
) -> Result<String, BackendFailure> {
    let directory = representation(SystemResourceType::DirectoryRead);
    let path = representation(SystemResourceType::RelativePath);
    let file = representation(SystemResourceType::ReadFile);
    if shape.ok_llvm != file {
        return Err(BackendFailure::InvalidIr);
    }
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    let (read_error, error) = native_error(target, "failure");
    // [PATH-2]: the path is resolved against the supplied value's own directory
    // object through the target's own directory-relative facility. Nothing is
    // concatenated onto it and no ambient working directory is consulted, so a
    // resolved object may lie outside that directory exactly as the process
    // namespace resolves it — the complete promise the type makes.
    //
    // The lease's code units are the C string the facility takes: [HOST-3]
    // fixes the backing as the command-lifetime argument snapshot, whose
    // elements the target terminates, [SYS-9] takes the lease length as that
    // element's exact length, and [PATH-1] admits no embedded NUL and retypes
    // the same lease with no copy. Nothing is allocated or copied here.
    let mapper = emit_open_completion_mapper(
        program,
        shape,
        OPEN_READ_COMPLETION_MAPPER,
        SystemResourceType::ReadFile,
    )?;
    let descriptor_class_argument =
        windows_descriptor_class_argument(target, WINDOWS_DESCRIPTOR_CLASS_READ_FILE);
    let wrapper = if target.uses_typed_completion_file_adapter() {
        let prologue = render_named_target_frame(
            program,
            qualification,
            target_layout,
            &[
                (
                    "%open.error.slot",
                    TargetFrameSlot::natural(TargetStorageType::integer(32)),
                ),
                (
                    "%open.outcome.slot",
                    TargetFrameSlot::natural(TargetStorageType::integer(32)),
                ),
            ],
        )?;
        format!(
            "define private {llvm} @{symbol}({directory} %root, {path} %path) alwaysinline {{\n\
             entry:\n\
             {prologue}  \
             %text = extractvalue {path} %path, 0\n  \
             %descriptor = call {file} @{open}({directory} %root, ptr %text, i32 {flags}, \
             i32 0, i32 0, i32 {OPEN_EXPECT_REGULAR}{descriptor_class_argument}, \
             ptr %open.error.slot, \
             ptr %open.outcome.slot)\n  \
             %raw.descriptor = sext {file} %descriptor to i64\n  \
             %open.error = load i32, ptr %open.error.slot, align 4\n  \
             %open.outcome = load i32, ptr %open.outcome.slot, align 4\n  \
             %mapped = call {llvm} @{OPEN_READ_COMPLETION_MAPPER}(i64 %raw.descriptor, \
             i32 %open.error, i32 %open.outcome)\n  \
             ret {llvm} %mapped\n\
             }}\n\n",
            symbol = implementation.symbol(),
            open = target.file_open_symbol(),
            flags = target.file_open_flags(),
        )
    } else {
        format!(
            "define private {llvm} @{symbol}({directory} %root, {path} %path) alwaysinline {{\n\
         entry:\n  \
         %text = extractvalue {path} %path, 0\n  \
         %descriptor = call {file} @{open}({directory} %root, \
         ptr %text, i32 {flags}, i32 0, i32 0)\n  \
         %opened = icmp sge {file} %descriptor, 0\n  \
         br i1 %opened, label %live, label %failure\n\
         live:\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {file} %descriptor, {ok_index}\n  \
         ret {llvm} %ok\n\
         failure:\n\
         {read_error}  \
         %failure.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 {error}, i8 \
         {ORIGIN_DIRECTORY_OPEN})\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} %failure.error, {err_index}\n  \
         ret {llvm} %err\n\
         }}\n\n",
            symbol = implementation.symbol(),
            open = target.file_open_symbol(),
            flags = target.file_open_flags()
        )
    };
    Ok(format!("{mapper}{wrapper}"))
}

fn emit_open_completion_mapper(
    program: &IrProgram<'_, '_, '_>,
    shape: &OutcomeShape,
    symbol: &str,
    resource: SystemResourceType,
) -> Result<String, BackendFailure> {
    let opened = representation(resource);
    if shape.ok_llvm != opened {
        return Err(BackendFailure::InvalidIr);
    }
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        err_type,
        ..
    } = shape;
    let classes = io_error_classes(program, *err_type)?;
    let directory_class = classes
        .iter()
        .find(|class| class.spelling == "IsDirectory")
        .ok_or(BackendFailure::InvalidIr)?;
    let other_class = classes
        .iter()
        .find(|class| class.spelling == "Other")
        .ok_or(BackendFailure::InvalidIr)?;
    let (directory_value, directory_error) =
        io_error_value(err_llvm, directory_class, "kind.directory", "0", "0");
    let (other_value, other_error) = io_error_value(err_llvm, other_class, "kind.other", "0", "0");
    Ok(format!(
        "define private {llvm} @{symbol}(i64 %raw.descriptor, i32 %error, \
         i32 %open.outcome) alwaysinline {{\n\
         entry:\n  \
         switch i32 %open.outcome, label %tcb.defect [\n  \
           i32 0, label %live\n  \
           i32 1, label %open.failure\n  \
           i32 2, label %status.failure\n  \
           i32 3, label %kind.directory.return\n  \
           i32 4, label %kind.other.return\n  \
         ]\n\
         live:\n  \
         %descriptor = trunc i64 %raw.descriptor to {opened}\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {opened} %descriptor, {ok_index}\n  \
         ret {llvm} %ok\n\
         open.failure:\n  \
         %open.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 \
         {ORIGIN_DIRECTORY_OPEN})\n  \
         %open.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %open.result = insertvalue {llvm} %open.tag, {err_llvm} %open.error, {err_index}\n  \
         ret {llvm} %open.result\n\
         status.failure:\n  \
         %status.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 \
         {ORIGIN_DESCRIPTOR_STATUS})\n  \
         %status.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %status.result = insertvalue {llvm} %status.tag, {err_llvm} %status.error, \
         {err_index}\n  \
         ret {llvm} %status.result\n\
         kind.directory.return:\n\
         {directory_value}  \
         %kind.directory.result.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %kind.directory.result = insertvalue {llvm} %kind.directory.result.tag, {err_llvm} \
         {directory_error}, {err_index}\n  \
         ret {llvm} %kind.directory.result\n\
         kind.other.return:\n\
         {other_value}  \
         %kind.other.result.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %kind.other.result = insertvalue {llvm} %kind.other.result.tag, {err_llvm} \
         {other_error}, {err_index}\n  \
         ret {llvm} %kind.other.result\n\
         tcb.defect:\n  \
         call void @abort()\n  \
         unreachable\n\
         }}\n\n"
    ))
}

fn emit_read_at(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &ReadOutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let file = representation(SystemResourceType::ReadFile);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    let ReadOutcomeShape {
        llvm,
        bytes_tag,
        bytes_index,
        ..
    } = shape;
    let entry = range_entry("");
    let (read_error, error) = native_error(target, "failure");
    let invalid_input = target
        .error_classes()
        .iter()
        .find(|class| class.class == "InvalidInput")
        .and_then(|class| class.codes.first())
        .copied()
        .ok_or(BackendFailure::InvalidIr)?;
    // The two call-site SYS-8 goals authorize this half-open range. A
    // zero-length range reports `next = start` and issues no host transfer.
    // A nonempty range makes at most one positioned transfer attempt. The
    // explicit file offset is checked before handoff and the operation never
    // observes or changes an implicit cursor.
    let mapper = emit_read_completion_mapper(shape);
    let wrapper = format!(
        "define private {llvm} @{symbol}({file} %file, {buffer} %destination, i64 %file_offset, \
         i64 %start, i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         %vacant = icmp eq i64 %extent, 0\n  \
         br i1 %vacant, label %empty, label %offset\n\
         empty:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %empty.outcome = insertvalue {llvm} %empty.tag, i64 %start, {bytes_index}\n  \
         ret {llvm} %empty.outcome\n\
         offset:\n  \
         %offset.fits = icmp ule i64 %file_offset, 9223372036854775807\n  \
         br i1 %offset.fits, label %transfer, label %invalid.offset\n\
         invalid.offset:\n  \
         %invalid.outcome = call {llvm} @{READ_COMPLETION_MAPPER}(i64 -1, i32 {invalid_input}, \
         i64 %start, i64 %extent)\n  \
         ret {llvm} %invalid.outcome\n\
         transfer:\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %target = getelementptr inbounds i8, ptr %base, i64 %start\n  \
         %transferred = call i64 @{pread}({file} %file, ptr %target, i64 %extent, \
         i64 %file_offset)\n  \
         %failed = icmp slt i64 %transferred, 0\n  \
         br i1 %failed, label %failure, label %complete\n\
         failure:\n\
         {read_error}  br label %complete\n\
         complete:\n  \
         %error = phi i32 [ 0, %transfer ], [ {error}, %failure ]\n  \
         %outcome = call {llvm} @{READ_COMPLETION_MAPPER}(i64 %transferred, i32 %error, \
         i64 %start, i64 %extent)\n  \
         ret {llvm} %outcome\n\
        }}\n\n",
        symbol = implementation.symbol(),
        pread = target.pread_symbol()
    );
    Ok(format!("{mapper}{wrapper}"))
}

fn emit_read_completion_mapper(shape: &ReadOutcomeShape) -> String {
    let ReadOutcomeShape {
        llvm,
        bytes_tag,
        bytes_index,
        end_tag,
        failed_tag,
        failed_index,
        failed_llvm,
        ..
    } = shape;
    format!(
        "define private {llvm} @{READ_COMPLETION_MAPPER}(i64 %transferred, i32 %error, \
         i64 %start, i64 %extent) alwaysinline {{\n\
         entry:\n  \
         %empty = icmp eq i64 %extent, 0\n  \
         br i1 %empty, label %vacant, label %nonempty\n\
         vacant:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %empty.outcome = insertvalue {llvm} %empty.tag, i64 %start, {bytes_index}\n  \
         ret {llvm} %empty.outcome\n\
         nonempty:\n  \
         %progress = icmp sgt i64 %transferred, 0\n  \
         br i1 %progress, label %sanitize, label %quiet\n\
         sanitize:\n  \
         %bounded = icmp ule i64 %transferred, %extent\n  \
         br i1 %bounded, label %bytes, label %tcb.defect\n\
         bytes:\n  \
         %next = add nuw i64 %start, %transferred\n  \
         %bytes.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %bytes.outcome = insertvalue {llvm} %bytes.tag, i64 %next, {bytes_index}\n  \
         ret {llvm} %bytes.outcome\n\
         quiet:\n  \
         %ended = icmp eq i64 %transferred, 0\n  \
         br i1 %ended, label %exhausted, label %failure\n\
         exhausted:\n  \
         %exhausted.outcome = insertvalue {llvm} zeroinitializer, i32 {end_tag}, 0\n  \
         ret {llvm} %exhausted.outcome\n\
         failure:\n  \
         %failure.error = call {failed_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 {ORIGIN_READ})\n  \
         %failed.tag = insertvalue {llvm} zeroinitializer, i32 {failed_tag}, 0\n  \
         %failed.outcome = insertvalue {llvm} %failed.tag, {failed_llvm} %failure.error, \
         {failed_index}\n  \
         ret {llvm} %failed.outcome\n\
         tcb.defect:\n  \
         call void @abort()\n  \
         unreachable\n\
         }}\n\n"
    )
}

fn emit_write_once(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let output = representation(SystemResourceType::Output);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    if shape.ok_llvm != "i64" {
        return Err(BackendFailure::InvalidIr);
    }
    let classes = io_error_classes(program, shape.err_type)?;
    let refused = classes
        .iter()
        .find(|class| class.spelling == "WriteZero")
        .ok_or(BackendFailure::InvalidIr)?;
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        ..
    } = shape;
    let entry = range_entry("");
    let (read_error, error) = native_error(target, "failure");
    // A host zero-length write is `Err(WriteZero())`, which no native error
    // code produced: [SYS-7] leaves both detail fields zero when the target
    // supplies no value for them.
    // At most one host output attempt [SYS-12]. A zero-length range reports
    // `next = start` and issues no host transfer; otherwise `Ok(next)` means
    // exactly that the host accepted `[start, next)`, promising neither line
    // atomicity nor durability. A closed destination arrives as
    // the recoverable `BrokenPipe` class because the bootstrap installed the
    // ignored write-to-closed-pipe disposition once, before entry [QUAL-3];
    // this path performs no per-call signal-disposition operation.
    let mapper = emit_write_completion_mapper(shape, refused);
    let wrapper = format!(
        "define private {llvm} @{symbol}({output} %output, {buffer} %source, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         %vacant = icmp eq i64 %extent, 0\n  \
         br i1 %vacant, label %empty, label %transfer\n\
         empty:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %empty.outcome = insertvalue {llvm} %empty.tag, i64 %start, {ok_index}\n  \
         ret {llvm} %empty.outcome\n\
         transfer:\n  \
         %base = extractvalue {buffer} %source, 0\n  \
         %target = getelementptr inbounds i8, ptr %base, i64 %start\n  \
         %accepted = call i64 @{write}({output} %output, ptr %target, i64 %extent)\n  \
         %failed = icmp slt i64 %accepted, 0\n  \
         br i1 %failed, label %failure, label %complete\n\
         failure:\n\
         {read_error}  br label %complete\n\
         complete:\n  \
         %error = phi i32 [ 0, %transfer ], [ {error}, %failure ]\n  \
         %outcome = call {llvm} @{WRITE_COMPLETION_MAPPER}(i64 %accepted, i32 %error, \
         i64 %start, i64 %extent)\n  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        write = target.write_symbol()
    );
    Ok(format!("{mapper}{wrapper}"))
}

fn emit_write_completion_mapper(shape: &OutcomeShape, refused: &IoErrorClass) -> String {
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    let (refused_value, refused_error) =
        io_error_value(err_llvm, refused, "refused", "0", &ORIGIN_NONE.to_string());
    format!(
        "define private {llvm} @{WRITE_COMPLETION_MAPPER}(i64 %accepted, i32 %error, \
         i64 %start, i64 %extent) alwaysinline {{\n\
         entry:\n  \
         %empty = icmp eq i64 %extent, 0\n  \
         br i1 %empty, label %vacant, label %nonempty\n\
         vacant:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %empty.outcome = insertvalue {llvm} %empty.tag, i64 %start, {ok_index}\n  \
         ret {llvm} %empty.outcome\n\
         nonempty:\n  \
         %progress = icmp sgt i64 %accepted, 0\n  \
         br i1 %progress, label %sanitize, label %quiet\n\
         sanitize:\n  \
         %bounded = icmp ule i64 %accepted, %extent\n  \
         br i1 %bounded, label %ok, label %tcb.defect\n\
         ok:\n  \
         %next = add nuw i64 %start, %accepted\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok.outcome = insertvalue {llvm} %ok.tag, i64 %next, {ok_index}\n  \
         ret {llvm} %ok.outcome\n\
         quiet:\n  \
         %refused = icmp eq i64 %accepted, 0\n  \
         br i1 %refused, label %zero, label %failure\n\
         zero:\n\
         {refused_value}  \
         %zero.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %zero.outcome = insertvalue {llvm} %zero.tag, {err_llvm} {refused_error}, {err_index}\n  \
         ret {llvm} %zero.outcome\n\
         failure:\n  \
         %failure.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 {ORIGIN_WRITE})\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err.outcome = insertvalue {llvm} %err.tag, {err_llvm} %failure.error, {err_index}\n  \
         ret {llvm} %err.outcome\n\
         tcb.defect:\n  \
         call void @abort()\n  \
         unreachable\n\
         }}\n\n"
    )
}

/// One [SYS-6] `ListOutcome` instantiation's tags and field positions.
struct ListOutcomeShape {
    llvm: String,
    bytes_tag: u32,
    /// The position of `ListBytes(next:)`; `entries:` is the next one.
    bytes_index: usize,
    end_tag: u32,
    failed_tag: u32,
    failed_index: usize,
    failed_llvm: String,
    failed_type: IrType,
}

/// Resolves the [SYS-14] enumeration outcome from the program's own IR.
///
/// `ListBytes(next: u64, entries: u64)` is the single two-u64 variant,
/// `ListEnd()` the single empty one, and `ListFailed(error: IoError)` the
/// single variant carrying a nominal payload, so the three are resolved by
/// shape rather than by any spelling [QUAL-1].
fn list_outcome_shape(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<ListOutcomeShape, BackendFailure> {
    let variants = variants_of(program, ty)?;
    if variants.len() != 3 {
        return Err(BackendFailure::InvalidIr);
    }
    let counted = IrType::Integer {
        width: 64,
        signed: false,
    };
    let mut measured = None;
    let mut failed = None;
    for variant in variants {
        match variant.fields() {
            [count, entries] if count.ty() == counted && entries.ty() == counted => {
                if measured.replace(variant.tag()).is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
            }
            [field] if matches!(field.ty(), IrType::Nominal(_)) => {
                let previous = failed.replace((variant.tag(), field.ty()));
                if previous.is_some() {
                    return Err(BackendFailure::InvalidIr);
                }
            }
            _ => {}
        }
    }
    let bytes_tag = measured.ok_or(BackendFailure::InvalidIr)?;
    let (failed_tag, failed_type) = failed.ok_or(BackendFailure::InvalidIr)?;
    Ok(ListOutcomeShape {
        llvm: llvm_type(program, ty)?,
        bytes_tag,
        bytes_index: variant_field_base(variants, bytes_tag)?,
        end_tag: empty_variant_tag(program, ty)?,
        failed_tag,
        failed_index: variant_field_base(variants, failed_tag)?,
        failed_llvm: llvm_type(program, failed_type)?,
        failed_type,
    })
}

/// The private [SYS-7] value one rejected component name produces.
///
/// No native facility ran, so both detail fields are zero [SYS-7]; the class
/// is `InvalidPath` because the rejection is exactly that the supplied bytes
/// are not one valid relative path component.
fn invalid_component(
    program: &IrProgram<'_, '_, '_>,
    err_llvm: &str,
    err_type: IrType,
) -> Result<(String, String), BackendFailure> {
    let classes = io_error_classes(program, err_type)?;
    let class = classes
        .iter()
        .find(|class| class.spelling == "InvalidPath")
        .ok_or(BackendFailure::InvalidIr)?;
    Ok(io_error_value(
        err_llvm,
        class,
        "invalid",
        "0",
        &ORIGIN_NONE.to_string(),
    ))
}

/// Emits the component-name validation every [SYS-14] name operation shares.
///
/// The admitted bytes are exactly one relative path component: at least one
/// byte, no more than the target family's component limit, no NUL, and no
/// target separator — so no source-assembled multi-component path reaches the
/// host and [PATH-1]'s deferral of path algebra stands. Validation precedes
/// the copy and therefore precedes the host call.
fn component_validation(buffer: &str, target: SystemTarget) -> String {
    let component_limit = target.component_limit();
    if target.is_windows() {
        return format!(
            "measure:\n  \
             %oversize = icmp ugt i64 %extent, {component_limit}\n  \
             %vacant = icmp eq i64 %extent, 0\n  \
             %width.remainder = and i64 %extent, 1\n  \
             %misaligned = icmp ne i64 %width.remainder, 0\n  \
             %size.unusable = or i1 %oversize, %vacant\n  \
             %unusable = or i1 %size.unusable, %misaligned\n  \
             br i1 %unusable, label %invalid, label %scan.entry\n\
             scan.entry:\n  \
             %base = extractvalue {buffer} %name, 0\n  \
             %text = getelementptr inbounds i8, ptr %base, i64 %start\n  \
             br label %scan\n\
             scan:\n  \
             %index = phi i64 [ 0, %scan.entry ], [ %index.next, %scan.step ]\n  \
             %at = getelementptr inbounds i8, ptr %text, i64 %index\n  \
             %unit = load i16, ptr %at, align 1\n  \
             %terminating = icmp eq i16 %unit, 0\n  \
             %slash = icmp eq i16 %unit, 47\n  \
             %backslash = icmp eq i16 %unit, 92\n  \
             %separator = or i1 %slash, %backslash\n  \
             %refused = or i1 %terminating, %separator\n  \
             br i1 %refused, label %invalid, label %scan.step\n\
             scan.step:\n  \
             %index.next = add i64 %index, 2\n  \
             %scanned = icmp uge i64 %index.next, %extent\n  \
             br i1 %scanned, label %open, label %scan\n"
        );
    }
    let root = u32::from(target.root_prefix());
    format!(
        "measure:\n  \
         %oversize = icmp ugt i64 %extent, {component_limit}\n  \
         %vacant = icmp eq i64 %extent, 0\n  \
         %unusable = or i1 %oversize, %vacant\n  \
         br i1 %unusable, label %invalid, label %scan.entry\n\
         scan.entry:\n  \
         %base = extractvalue {buffer} %name, 0\n  \
         %text = getelementptr inbounds i8, ptr %base, i64 %start\n  \
         br label %scan\n\
         scan:\n  \
         %index = phi i64 [ 0, %scan.entry ], [ %index.next, %scan.step ]\n  \
         %at = getelementptr inbounds i8, ptr %text, i64 %index\n  \
         %byte = load i8, ptr %at, align 1\n  \
         %byte.value = zext i8 %byte to i32\n  \
         %terminating = icmp eq i32 %byte.value, 0\n  \
         %separating = icmp eq i32 %byte.value, {root}\n  \
         %refused = or i1 %terminating, %separating\n  \
         br i1 %refused, label %invalid, label %scan.step\n\
         scan.step:\n  \
         %index.next = add i64 %index, 1\n  \
         %scanned = icmp uge i64 %index.next, %extent\n  \
         br i1 %scanned, label %open, label %scan\n"
    )
}

/// Emits the approved implementation of `open_directory` [SYS-14].
fn emit_open_directory(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
    target_layout: TargetLayout,
) -> Result<String, BackendFailure> {
    let mapper = emit_open_completion_mapper(
        program,
        shape,
        OPEN_DIRECTORY_COMPLETION_MAPPER,
        SystemResourceType::DirectoryRead,
    )?;
    let wrapper = emit_open_by_name(
        program,
        qualification,
        implementation,
        shape,
        target,
        target_layout,
        SystemResourceType::DirectoryRead,
    )?;
    Ok(format!("{mapper}{wrapper}"))
}

/// Emits the approved implementation of active `open_file` [SYS-11].
///
/// The provisional descriptor is opened without following the terminal link
/// and without blocking on a non-regular object, then classified through the
/// target ABI before it becomes a `ReadFile`. A rejected descriptor receives
/// one best-effort close attempt; its diagnostic is discarded without retry,
/// and the selected inspection or classification error is returned unchanged.
fn emit_open_file(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
    target_layout: TargetLayout,
) -> Result<String, BackendFailure> {
    let mapper = emit_open_completion_mapper(
        program,
        shape,
        OPEN_FILE_COMPLETION_MAPPER,
        SystemResourceType::ReadFile,
    )?;
    let wrapper = emit_open_by_name(
        program,
        qualification,
        implementation,
        shape,
        target,
        target_layout,
        SystemResourceType::ReadFile,
    )?;
    Ok(format!("{mapper}{wrapper}"))
}

/// Emits one open-by-name implementation [SYS-11, SYS-14].
///
/// The name arrives as caller-owned bytes and never becomes a path value, so
/// [HOST-3]'s command-lifetime backing and [PATH-1]'s inline lease are
/// untouched. The validated component is copied into one bounded stack slot
/// only to terminate it for the target's own directory-relative facility,
/// which then resolves it against the supplied directory object exactly
/// as `open_read` does [PATH-2].
fn emit_open_by_name(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
    target_layout: TargetLayout,
    opened: SystemResourceType,
) -> Result<String, BackendFailure> {
    let (flags, require_regular) = match opened {
        SystemResourceType::DirectoryRead => (target.component_directory_open_flags(), false),
        SystemResourceType::ReadFile => (target.component_file_open_flags(), true),
        _ => return Err(BackendFailure::InvalidIr),
    };
    let directory = representation(SystemResourceType::DirectoryRead);
    let opened = representation(opened);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    if shape.ok_llvm != opened {
        return Err(BackendFailure::InvalidIr);
    }
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        err_type,
        ..
    } = shape;
    let terminator_bytes = if target.is_windows() { 2 } else { 1 };
    let slot = target.component_limit() + terminator_bytes;
    let component_align = if target.is_windows() { 2 } else { 1 };
    // One shared buffer per wrapper, and deliberately not the per-outstanding-
    // operation storage the handed-out completion sites use. This wrapper's
    // only host call is the synchronous direct open, which resolves the name
    // inside the call and leaves no operation outstanding when it returns; the
    // submitting path stages its own copy in the operation record besides. A
    // wrapper that ever submits instead would have to index this the way
    // `FunctionEmitter::completion_entry_slot` indexes a hand-out's storage.
    let mut frame_slots = vec![(
        "%component",
        TargetFrameSlot::aligned(TargetStorageType::bytes(slot), component_align),
    )];
    // The typed adapter performs the descriptor-kind inspection before it
    // publishes the outcome, so only the direct qualified wrapper owns a
    // status record of its own.
    if require_regular && !target.uses_typed_completion_file_adapter() {
        frame_slots.push((
            "%file.status",
            TargetFrameSlot::aligned(TargetStorageType::bytes(target.file_status_size()), 8),
        ));
    }
    if target.uses_typed_completion_file_adapter() {
        frame_slots.push((
            "%open.error.slot",
            TargetFrameSlot::natural(TargetStorageType::integer(32)),
        ));
        frame_slots.push((
            "%open.outcome.slot",
            TargetFrameSlot::natural(TargetStorageType::integer(32)),
        ));
    }
    let prologue = render_named_target_frame(program, qualification, target_layout, &frame_slots)?;
    let entry = range_entry(&prologue);
    let component = component_validation(&buffer, target);
    let (read_error, error) = native_error(target, "failure");
    let (invalid_value, invalid_error) = invalid_component(program, err_llvm, *err_type)?;
    let opened_target = if require_regular { "inspect" } else { "live" };
    let validation = if require_regular {
        let (inspection_read_error, inspection_error) = native_error(target, "inspection");
        let classes = io_error_classes(program, *err_type)?;
        let directory_class = classes
            .iter()
            .find(|class| class.spelling == "IsDirectory")
            .ok_or(BackendFailure::InvalidIr)?;
        let other_class = classes
            .iter()
            .find(|class| class.spelling == "Other")
            .ok_or(BackendFailure::InvalidIr)?;
        let (directory_value, directory_error) =
            io_error_value(err_llvm, directory_class, "kind.directory", "0", "0");
        let (other_value, other_error) =
            io_error_value(err_llvm, other_class, "kind.other", "0", "0");
        let status = target.file_status_symbol();
        let status_call = if target.uses_typed_completion_file_adapter() {
            format!(
                "call i32 @{status}(i32 %descriptor, ptr %file.status, i64 {})",
                target.file_status_size()
            )
        } else {
            format!("call i32 @{status}(i32 %descriptor, ptr %file.status)")
        };
        format!(
            "inspect:\n  \
             %inspection.result = {status_call}\n  \
             %inspection.ok = icmp eq i32 %inspection.result, 0\n  \
             br i1 %inspection.ok, label %classify, label %inspection.failure\n\
             classify:\n  \
             %mode.at = getelementptr inbounds i8, ptr %file.status, i64 {mode_offset}\n  \
             %mode.native = load i16, ptr %mode.at, align 2\n  \
             %mode = zext i16 %mode.native to i32\n  \
             %file.kind = and i32 %mode, 61440\n  \
             %regular = icmp eq i32 %file.kind, 32768\n  \
             br i1 %regular, label %live, label %kind.failure\n\
             inspection.failure:\n\
             {inspection_read_error}  \
             %inspection.close = call i32 @{close}(i32 %descriptor)\n  \
             br label %inspection.error\n\
             inspection.error:\n  \
             %inspection.mapped = call {err_llvm} @{IO_ERROR_MAPPER}(i32 {inspection_error}, \
             i8 {ORIGIN_DESCRIPTOR_STATUS})\n  \
             %inspection.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %inspection.outcome = insertvalue {llvm} %inspection.tag, {err_llvm} \
             %inspection.mapped, {err_index}\n  \
             ret {llvm} %inspection.outcome\n\
             kind.failure:\n  \
             %kind.directory = icmp eq i32 %file.kind, 16384\n  \
             %kind.close = call i32 @{close}(i32 %descriptor)\n  \
             br label %kind.select\n\
             kind.select:\n  \
             br i1 %kind.directory, label %kind.directory.return, label %kind.other.return\n\
             kind.directory.return:\n\
             {directory_value}  \
             %kind.directory.outcome.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %kind.directory.outcome = insertvalue {llvm} %kind.directory.outcome.tag, \
             {err_llvm} {directory_error}, {err_index}\n  \
             ret {llvm} %kind.directory.outcome\n\
             kind.other.return:\n\
             {other_value}  \
             %kind.other.outcome.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %kind.other.outcome = insertvalue {llvm} %kind.other.outcome.tag, {err_llvm} \
             {other_error}, \
             {err_index}\n  \
             ret {llvm} %kind.other.outcome\n",
            close = target.close_symbol(),
            mode_offset = target.file_status_mode_offset(),
        )
    } else {
        String::new()
    };
    if target.uses_typed_completion_file_adapter() {
        let (mapper, expected_kind) = if require_regular {
            (OPEN_FILE_COMPLETION_MAPPER, OPEN_EXPECT_REGULAR)
        } else {
            (OPEN_DIRECTORY_COMPLETION_MAPPER, OPEN_EXPECT_DIRECTORY)
        };
        let descriptor_class_argument = windows_descriptor_class_argument(
            target,
            if require_regular {
                WINDOWS_DESCRIPTOR_CLASS_READ_FILE
            } else {
                WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT
            },
        );
        let terminator = if target.is_windows() {
            "store i16 0, ptr %terminator, align 1"
        } else {
            "store i8 0, ptr %terminator, align 1"
        };
        return Ok(format!(
            "define private {llvm} @{symbol}({directory} %root, {buffer} %name, i64 %start, \
             i64 %end) alwaysinline {{\n\
             {entry}\
             {component}\
             open:\n  \
             call void @llvm.memcpy.p0.p0.i64(ptr %component, ptr %text, i64 %extent, \
             i1 false)\n  \
             %terminator = getelementptr inbounds i8, ptr %component, i64 %extent\n  \
             {terminator}\n  \
             %descriptor = call {opened} @{open}({directory} %root, ptr %component, i32 {flags}, \
             i32 0, i32 0, i32 {expected_kind}{descriptor_class_argument}, \
             ptr %open.error.slot, \
             ptr %open.outcome.slot)\n  \
             %raw.descriptor = sext {opened} %descriptor to i64\n  \
             %open.error = load i32, ptr %open.error.slot, align 4\n  \
             %open.outcome = load i32, ptr %open.outcome.slot, align 4\n  \
             %mapped = call {llvm} @{mapper}(i64 %raw.descriptor, i32 %open.error, \
             i32 %open.outcome)\n  \
             ret {llvm} %mapped\n\
             invalid:\n\
             {invalid_value}  \
             %rejected.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
             %rejected.outcome = insertvalue {llvm} %rejected.tag, {err_llvm} \
             {invalid_error}, {err_index}\n  \
             ret {llvm} %rejected.outcome\n\
             }}\n\n",
            symbol = implementation.symbol(),
            open = target.file_open_symbol(),
        ));
    }
    Ok(format!(
        "define private {llvm} @{symbol}({directory} %root, {buffer} %name, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         {component}\
         open:\n  \
         call void @llvm.memcpy.p0.p0.i64(ptr %component, ptr %text, i64 %extent, i1 false)\n  \
         %terminator = getelementptr inbounds i8, ptr %component, i64 %extent\n  \
         store i8 0, ptr %terminator, align 1\n  \
         %descriptor = call {opened} @{open}({directory} %root, \
         ptr %component, i32 {flags}, i32 0, i32 0)\n  \
         %opened = icmp sge {opened} %descriptor, 0\n  \
         br i1 %opened, label %{opened_target}, label %failure\n\
         {validation}\
         live:\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {opened} %descriptor, {ok_index}\n  \
         ret {llvm} %ok\n\
         failure:\n\
         {read_error}  \
         %failure.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 {error}, i8 \
         {ORIGIN_DIRECTORY_OPEN})\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} %failure.error, {err_index}\n  \
         ret {llvm} %err\n\
         invalid:\n\
         {invalid_value}  \
         %rejected.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %rejected.outcome = insertvalue {llvm} %rejected.tag, {err_llvm} {invalid_error}, \
         {err_index}\n  \
         ret {llvm} %rejected.outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        open = target.file_open_symbol(),
    ))
}

/// Emits the approved implementation of `open_directory_source` [SYS-14].
///
/// One enumeration handle is an independent descriptor opened against the
/// supplied value's own directory object through the same directory-relative
/// facility [PATH-2], named by the self component. It therefore carries its
/// own cursor and aliases the directory value no more than `open_read`'s
/// `ReadFile` does [SYS-10].
fn emit_open_directory_source(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
    target_layout: TargetLayout,
) -> Result<String, BackendFailure> {
    let directory = representation(SystemResourceType::DirectoryRead);
    let list = representation(SystemResourceType::DirectorySource);
    if shape.ok_llvm != list {
        return Err(BackendFailure::InvalidIr);
    }
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    let (read_error, error) = native_error(target, "failure");
    let mapper = emit_open_completion_mapper(
        program,
        shape,
        OPEN_LIST_COMPLETION_MAPPER,
        SystemResourceType::DirectorySource,
    )?;
    let descriptor_class_argument =
        windows_descriptor_class_argument(target, WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE);
    let wrapper = if target.uses_typed_completion_file_adapter() {
        let prologue = render_named_target_frame(
            program,
            qualification,
            target_layout,
            &[
                (
                    "%open.error.slot",
                    TargetFrameSlot::natural(TargetStorageType::integer(32)),
                ),
                (
                    "%open.outcome.slot",
                    TargetFrameSlot::natural(TargetStorageType::integer(32)),
                ),
            ],
        )?;
        format!(
            "define private {llvm} @{symbol}({directory} %directory) alwaysinline {{\n\
             entry:\n\
             {prologue}  \
             %descriptor = call {list} @{open}({directory} %directory, \
             ptr {WORKING_DIRECTORY}, i32 {flags}, i32 0, i32 0, \
             i32 {OPEN_EXPECT_DIRECTORY}{descriptor_class_argument}, ptr %open.error.slot, \
             ptr %open.outcome.slot)\n  \
             %raw.descriptor = sext {list} %descriptor to i64\n  \
             %open.error = load i32, ptr %open.error.slot, align 4\n  \
             %open.outcome = load i32, ptr %open.outcome.slot, align 4\n  \
             %mapped = call {llvm} @{OPEN_LIST_COMPLETION_MAPPER}(i64 %raw.descriptor, \
             i32 %open.error, i32 %open.outcome)\n  \
             ret {llvm} %mapped\n\
             }}\n\n",
            symbol = implementation.symbol(),
            open = target.file_open_symbol(),
            flags = target.directory_open_flags(),
        )
    } else {
        format!(
            "define private {llvm} @{symbol}({directory} %directory) alwaysinline {{\n\
         entry:\n  \
         %descriptor = call {list} @{open}({directory} %directory, \
         ptr {WORKING_DIRECTORY}, i32 {flags}, i32 0, i32 0)\n  \
         %opened = icmp sge {list} %descriptor, 0\n  \
         br i1 %opened, label %live, label %failure\n\
         live:\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {list} %descriptor, {ok_index}\n  \
         ret {llvm} %ok\n\
         failure:\n\
         {read_error}  \
         %failure.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 {error}, i8 \
         {ORIGIN_DIRECTORY_OPEN})\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} %failure.error, {err_index}\n  \
         ret {llvm} %err\n\
         }}\n\n",
            symbol = implementation.symbol(),
            open = target.file_open_symbol(),
            flags = target.directory_open_flags()
        )
    };
    Ok(format!("{mapper}{wrapper}"))
}

/// Emits the one portable-record normalizer both `directory_next` routes
/// embed [SYS-14].
///
/// The facility writes its own records into the caller's range, and the shim
/// rewrites them in place as the portable
/// `[kind][little-endian u16 name length][name bytes]` sequence [SYS-14]. The
/// rewrite moves every byte strictly toward the front, because a portable
/// record's three-byte header is smaller than any qualified target's native
/// one, so no unread byte is ever overwritten. Every native header field is
/// validated against the reported extent before it is used, so a record the
/// facility mis-sizes ends the walk instead of reading past the range.
///
/// This is one text, emitted from one place, for both the direct route and the
/// completion mapper and for every qualified target. The only target-selected
/// part is the `header` block's tail, which answers exactly one question:
/// where the name's byte length comes from. Darwin's record states it in a
/// field; Linux's states none and NUL-terminates the name inside the extent
/// `d_reclen` reports, so the length is derived by one scan bounded by that
/// extent. The scan reads the name before any byte of this record is
/// rewritten, and the rewrite target is strictly behind the record's own name,
/// so measuring and copying never observe a byte the walk already moved.
fn emit_directory_record_normalizer(
    shape: &ListOutcomeShape,
    target: SystemTarget,
    enumeration: DirectoryEnumeration,
) -> String {
    let ListOutcomeShape {
        llvm,
        bytes_tag,
        bytes_index,
        ..
    } = shape;
    let entries_index = bytes_index + 1;
    let name_offset = enumeration.name_offset();
    let record_length_offset = enumeration.record_length_offset();
    let entry_type_offset = enumeration.entry_type_offset();
    let native_regular = enumeration.native_regular();
    let native_directory = enumeration.native_directory();
    let native_symlink = enumeration.native_symlink();
    let native_unknown = enumeration.native_unknown();
    let component_limit = target.component_limit();
    let width_validation = if target.is_windows() {
        "  %named.remainder = and i64 %named, 1\n  \
         %named.even = icmp eq i64 %named.remainder, 0\n  \
         %width.usable = and i1 %naming, %named.even\n"
    } else {
        "  %width.usable = and i1 %naming, true\n"
    };
    let copy = if target.is_windows() {
        "copy:\n  \
         %copied = phi i64 [ 0, %record.header ], [ %copied.next, %copy.store ]\n  \
         %copy.done = icmp uge i64 %copied, %named\n  \
         br i1 %copy.done, label %step, label %copy.step\n\
         copy.step:\n  \
         %copy.from = getelementptr inbounds i8, ptr %source.name, i64 %copied\n  \
         %copy.unit = load i16, ptr %copy.from, align 1\n  \
         %copy.nul = icmp eq i16 %copy.unit, 0\n  \
         %copy.slash = icmp eq i16 %copy.unit, 47\n  \
         %copy.backslash = icmp eq i16 %copy.unit, 92\n  \
         %copy.separator = or i1 %copy.slash, %copy.backslash\n  \
         %copy.invalid = or i1 %copy.nul, %copy.separator\n  \
         br i1 %copy.invalid, label %tcb.defect, label %copy.store\n\
         copy.store:\n  \
         %copy.to = getelementptr inbounds i8, ptr %target.name, i64 %copied\n  \
         store i16 %copy.unit, ptr %copy.to, align 1\n  \
         %copied.next = add i64 %copied, 2\n  \
         br label %copy\n"
            .to_owned()
    } else {
        format!(
            "copy:\n  \
             %copied = phi i64 [ 0, %record.header ], [ %copied.next, %copy.store ]\n  \
             %copy.done = icmp uge i64 %copied, %named\n  \
             br i1 %copy.done, label %step, label %copy.step\n\
             copy.step:\n  \
             %copy.from = getelementptr inbounds i8, ptr %source.name, i64 %copied\n  \
             %copy.byte = load i8, ptr %copy.from, align 1\n  \
             %copy.nul = icmp eq i8 %copy.byte, 0\n  \
             %copy.separator = icmp eq i8 %copy.byte, {}\n  \
             %copy.invalid = or i1 %copy.nul, %copy.separator\n  \
             br i1 %copy.invalid, label %tcb.defect, label %copy.store\n\
             copy.store:\n  \
             %copy.to = getelementptr inbounds i8, ptr %target.name, i64 %copied\n  \
             store i8 %copy.byte, ptr %copy.to, align 1\n  \
             %copied.next = add i64 %copied, 1\n  \
             br label %copy\n",
            target.root_prefix()
        )
    };
    // The `header` block's tail, and every block it needs before `validate`
    // sees one `%named`.
    let measure = match enumeration.name_length() {
        EntryNameLength::Field { offset } => format!(
            "  %named.at = getelementptr inbounds i8, ptr %entry.record, i64 {offset}\n  \
             %named.native = load i16, ptr %named.at, align 1\n  \
             %named = zext i16 %named.native to i64\n  \
             br label %validate\n"
        ),
        // The extent must be inside the reported batch and must hold at least
        // one name byte before a single byte of the name is read, because the
        // scan's bound is the extent itself.
        EntryNameLength::NulTerminated => format!(
            "  %name.bounded = icmp ule i64 %record.extent, %remaining\n  \
             %name.present = icmp ugt i64 %record.extent, {name_offset}\n  \
             %name.scannable = and i1 %name.bounded, %name.present\n  \
             br i1 %name.scannable, label %name.measure, label %tcb.defect\n\
             name.measure:\n  \
             %name.span = sub nuw i64 %record.extent, {name_offset}\n  \
             %name.base = getelementptr inbounds i8, ptr %entry.record, i64 {name_offset}\n  \
             br label %name.scan\n\
             name.scan:\n  \
             %name.scanned = phi i64 [ 0, %name.measure ], \
             [ %name.scanned.next, %name.scan.step ]\n  \
             %name.unterminated = icmp uge i64 %name.scanned, %name.span\n  \
             br i1 %name.unterminated, label %tcb.defect, label %name.scan.step\n\
             name.scan.step:\n  \
             %name.at = getelementptr inbounds i8, ptr %name.base, i64 %name.scanned\n  \
             %name.byte = load i8, ptr %name.at, align 1\n  \
             %name.terminator = icmp eq i8 %name.byte, 0\n  \
             %name.scanned.next = add i64 %name.scanned, 1\n  \
             br i1 %name.terminator, label %name.measured, label %name.scan\n\
             name.measured:\n  \
             %named = phi i64 [ %name.scanned, %name.scan.step ]\n  \
             br label %validate\n"
        ),
    };
    format!(
        "normalize:\n  \
         br label %walk\n\
         walk:\n  \
         %source = phi i64 [ 0, %normalize ], [ %source.next, %step ]\n  \
         %written = phi i64 [ 0, %normalize ], [ %written.next, %step ]\n  \
         %entries = phi i64 [ 0, %normalize ], [ %entries.next, %step ]\n  \
         %complete = icmp eq i64 %source, %filled\n  \
         br i1 %complete, label %done, label %record\n\
         record:\n  \
         %remaining = sub nuw i64 %filled, %source\n  \
         %headerless = icmp ult i64 %remaining, {name_offset}\n  \
         br i1 %headerless, label %tcb.defect, label %header\n\
         header:\n  \
         %entry.record = getelementptr inbounds i8, ptr %window, i64 %source\n  \
         %record.extent.at = getelementptr inbounds i8, ptr %entry.record, \
         i64 {record_length_offset}\n  \
         %record.extent.native = load i16, ptr %record.extent.at, align 1\n  \
         %record.extent = zext i16 %record.extent.native to i64\n  \
         %kind.at = getelementptr inbounds i8, ptr %entry.record, i64 {entry_type_offset}\n  \
         %kind.native = load i8, ptr %kind.at, align 1\n  \
         %kind.value = zext i8 %kind.native to i64\n\
         {measure}\
         validate:\n  \
         %needed = add i64 {name_offset}, %named\n  \
         %sized = icmp uge i64 %record.extent, %needed\n  \
         %bounded = icmp ule i64 %record.extent, %remaining\n  \
         %advancing = icmp uge i64 %record.extent, 1\n  \
         %nameable = icmp ule i64 %named, {component_limit}\n  \
         %naming = icmp uge i64 %named, 1\n  \
         {width_validation}  \
         %named.usable = and i1 %nameable, %width.usable\n  \
         %consistent = and i1 %sized, %bounded\n  \
         %progressive = and i1 %advancing, %named.usable\n  \
         %usable = and i1 %consistent, %progressive\n  \
         br i1 %usable, label %room, label %tcb.defect\n\
         room:\n  \
         %portable = add i64 {ENTRY_HEADER}, %named\n  \
         %after = add i64 %written, %portable\n  \
         %fits = icmp ule i64 %after, %extent\n  \
         br i1 %fits, label %record.header, label %tcb.defect\n\
         record.header:\n  \
         %regular = icmp eq i64 %kind.value, {native_regular}\n  \
         %directory = icmp eq i64 %kind.value, {native_directory}\n  \
         %symlink = icmp eq i64 %kind.value, {native_symlink}\n  \
         %unclassified = icmp eq i64 %kind.value, {native_unknown}\n  \
         %kind.other = select i1 %regular, i8 {KIND_REGULAR}, i8 {KIND_OTHER}\n  \
         %kind.directory = select i1 %directory, i8 {KIND_DIRECTORY}, i8 %kind.other\n  \
         %kind.symlink = select i1 %symlink, i8 {KIND_SYMLINK}, i8 %kind.directory\n  \
         %kind.portable = select i1 %unclassified, i8 {KIND_UNKNOWN}, i8 %kind.symlink\n  \
         %target.record = getelementptr inbounds i8, ptr %window, i64 %written\n  \
         store i8 %kind.portable, ptr %target.record, align 1\n  \
         %target.named.low = getelementptr inbounds i8, ptr %target.record, i64 1\n  \
         %target.named.high = getelementptr inbounds i8, ptr %target.record, i64 2\n  \
         %named.short = trunc i64 %named to i16\n  \
         %named.low = trunc i16 %named.short to i8\n  \
         %named.high.part = lshr i16 %named.short, 8\n  \
         %named.high = trunc i16 %named.high.part to i8\n  \
         store i8 %named.low, ptr %target.named.low, align 1\n  \
         store i8 %named.high, ptr %target.named.high, align 1\n  \
         %target.name = getelementptr inbounds i8, ptr %target.record, i64 {ENTRY_HEADER}\n  \
         %source.name = getelementptr inbounds i8, ptr %entry.record, i64 {name_offset}\n  \
         br label %copy\n\
         {copy}  \
         step:\n  \
         %source.next = add i64 %source, %record.extent\n  \
         %written.next = add i64 %written, %portable\n  \
         %entries.next = add i64 %entries, 1\n  \
         br label %walk\n\
         done:\n  \
         %next = add nuw i64 %start, %written\n  \
         %bytes.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %bytes.endpoint = insertvalue {llvm} %bytes.tag, i64 %next, {bytes_index}\n  \
         %bytes.outcome = insertvalue {llvm} %bytes.endpoint, i64 %entries, {entries_index}\n  \
         ret {llvm} %bytes.outcome\n\
         tcb.defect:\n  \
         call void @abort()\n  \
         unreachable\n",
    )
}

/// Emits the approved implementation of `directory_next` [SYS-14].
///
/// The shape is [SYS-8]'s: enter the statically authorized range, obtain at
/// most one progress-producing host transfer through the target-progress
/// wrapper, then one outcome check and a cold mapper [QUAL-3]. Interruption and
/// readiness refusal remain inside that wrapper. The one addition is
/// normalization, which is [`emit_directory_record_normalizer`]'s one text.
fn emit_directory_next(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    implementation: ApprovedImplementation,
    shape: &ListOutcomeShape,
    target: SystemTarget,
    target_layout: TargetLayout,
) -> Result<String, BackendFailure> {
    let list = representation(SystemResourceType::DirectorySource);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    let enumeration = target
        .directory_enumeration()
        .ok_or(BackendFailure::InvalidIr)?;
    let ListOutcomeShape {
        llvm,
        bytes_tag,
        bytes_index,
        end_tag,
        failed_tag,
        failed_index,
        failed_llvm,
        ..
    } = shape;
    let entries_index = bytes_index + 1;
    let prologue = render_named_target_frame(
        program,
        qualification,
        target_layout,
        &[(
            "%position",
            TargetFrameSlot::natural(TargetStorageType::integer(64)),
        )],
    )?;
    let entry = range_entry(&prologue);
    let (read_error, error) = native_error(target, "failure");
    let normalizer = emit_directory_record_normalizer(shape, target, enumeration);
    let mapper = emit_directory_next_completion_mapper(program, shape, target)?;
    let wrapper = format!(
        "define private {llvm} @{symbol}({list} %list, {buffer} %destination, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         store i64 0, ptr %position, align 8\n  \
         %empty.range = icmp eq i64 %extent, 0\n  \
         br i1 %empty.range, label %empty, label %transfer\n\
         empty:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %empty.endpoint = insertvalue {llvm} %empty.tag, i64 %start, {bytes_index}\n  \
         %empty.outcome = insertvalue {llvm} %empty.endpoint, i64 0, {entries_index}\n  \
         ret {llvm} %empty.outcome\n\
         transfer:\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %window = getelementptr inbounds i8, ptr %base, i64 %start\n  \
         %filled = call i64 @wf__completion_directory_next_direct({list} %list, ptr %window, i64 %extent, \
         ptr %position)\n  \
         %progress = icmp sgt i64 %filled, 0\n  \
         br i1 %progress, label %sanitize, label %quiet\n\
         sanitize:\n  \
         %bounded.batch = icmp ule i64 %filled, %extent\n  \
         br i1 %bounded.batch, label %normalize, label %tcb.defect\n\
         quiet:\n  \
         %ended = icmp eq i64 %filled, 0\n  \
         br i1 %ended, label %exhausted, label %failure\n\
         exhausted:\n  \
         %exhausted.outcome = insertvalue {llvm} zeroinitializer, i32 {end_tag}, 0\n  \
         ret {llvm} %exhausted.outcome\n\
         failure:\n\
         {read_error}  \
         %failure.error = call {failed_llvm} @{IO_ERROR_MAPPER}(i32 {error}, i8 {ORIGIN_READ})\n  \
         %failed.tag = insertvalue {llvm} zeroinitializer, i32 {failed_tag}, 0\n  \
         %failed.outcome = insertvalue {llvm} %failed.tag, {failed_llvm} %failure.error, \
         {failed_index}\n  \
         ret {llvm} %failed.outcome\n\
         {normalizer}\
         }}\n\n",
        symbol = implementation.symbol(),
    );
    Ok(format!("{mapper}{wrapper}"))
}

fn emit_directory_next_completion_mapper(
    program: &IrProgram<'_, '_, '_>,
    shape: &ListOutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    let enumeration = target
        .directory_enumeration()
        .ok_or(BackendFailure::InvalidIr)?;
    let ListOutcomeShape {
        llvm,
        end_tag,
        failed_tag,
        failed_index,
        failed_llvm,
        ..
    } = shape;
    let normalizer = emit_directory_record_normalizer(shape, target, enumeration);
    Ok(format!(
        "define private {llvm} @{DIRECTORY_NEXT_COMPLETION_MAPPER}(i64 %filled, i32 %error, \
         {buffer} %destination, i64 %start, i64 %extent) alwaysinline {{\n\
         entry:\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %window = getelementptr inbounds i8, ptr %base, i64 %start\n  \
         %progress = icmp sgt i64 %filled, 0\n  \
         br i1 %progress, label %sanitize, label %quiet\n\
         sanitize:\n  \
         %bounded.batch = icmp ule i64 %filled, %extent\n  \
         br i1 %bounded.batch, label %normalize, label %tcb.defect\n\
         quiet:\n  \
         %ended = icmp eq i64 %filled, 0\n  \
         br i1 %ended, label %exhausted, label %failure\n\
         exhausted:\n  \
         %exhausted.outcome = insertvalue {llvm} zeroinitializer, i32 {end_tag}, 0\n  \
         ret {llvm} %exhausted.outcome\n\
         failure:\n  \
         %failure.error = call {failed_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 {ORIGIN_READ})\n  \
         %failed.tag = insertvalue {llvm} zeroinitializer, i32 {failed_tag}, 0\n  \
         %failed.outcome = insertvalue {llvm} %failed.tag, {failed_llvm} %failure.error, \
         {failed_index}\n  \
         ret {llvm} %failed.outcome\n\
         {normalizer}\
         }}\n\n"
    ))
}

/// Emits the one cold [SYS-7] outcome mapper the failing I/O implementations
/// share.
///
/// The class is the sole portable semantic discriminator, so the mapper turns
/// one native error code into exactly one class and carries the native detail
/// through unchanged: `code` value-preservingly in `u32`, and `origin` the
/// target-owned discriminator naming the facility that produced it. The
/// default arm is the closed set's own rule rather than a wildcard — a native
/// error with no portable distinction in this set is `Other`.
fn emit_io_error_mapper(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let io = llvm_type(program, ty)?;
    let classes = io_error_classes(program, ty)?;
    let other = classes
        .iter()
        .find(|class| class.spelling == "Other")
        .ok_or(BackendFailure::InvalidIr)?;
    let mut cases = String::new();
    let mut arms = String::new();
    let mut mapped = BTreeSet::new();
    for row in target.error_classes() {
        let class = classes
            .iter()
            .find(|class| class.spelling == row.class)
            .ok_or(BackendFailure::InvalidIr)?;
        if row.codes.is_empty() {
            continue;
        }
        for code in row.codes {
            // One native error maps onto exactly one class [SYS-7].
            if !mapped.insert(*code) {
                return Err(BackendFailure::InvalidIr);
            }
            writeln!(cases, "    i32 {code}, label %class.{}", class.tag)
                .map_err(|_| BackendFailure::TextEmission)?;
        }
        arms.push_str(&class_arm(&io, class));
    }
    arms.push_str(&class_arm(&io, other));
    Ok(format!(
        "define private {io} @{IO_ERROR_MAPPER}(i32 %code, i8 %origin) noinline cold {{\n\
         entry:\n  \
         switch i32 %code, label %class.{other_tag} [\n\
         {cases}  ]\n\
         {arms}}}\n\n",
        other_tag = other.tag
    ))
}

/// One mapper arm: the class's own value, built from the two detail fields.
fn class_arm(io: &str, class: &IoErrorClass) -> String {
    let prefix = format!("class.{}", class.tag);
    let (value, name) = io_error_value(io, class, &prefix, "%code", "%origin");
    format!("{prefix}:\n{value}  ret {io} {name}\n")
}

fn emit_exit_status(implementation: ApprovedImplementation) -> String {
    let status = representation(SystemResourceType::ExitStatus);
    // Total and pure: every `u8` is a valid command code, so there is no
    // failure outcome, no allocation, no host call, and no external effect
    // [SYS-13].
    format!(
        "define private {status} @{}(i8 %code) alwaysinline {{\n\
         entry:\n  \
         ret {status} %code\n\
         }}\n\n",
        implementation.symbol()
    )
}

/// The reservation value exists only in Whitefoot's ownership proof. The
/// implementation performs no target action and returns one harmless opaque
/// bit; target open wrappers erase the consumed permit before native calls.
fn emit_reserve_file(implementation: ApprovedImplementation) -> String {
    let permit = representation(SystemResourceType::FilePermit);
    format!(
        "define private {permit} @{}() alwaysinline {{\n\
         entry:\n  \
         ret {permit} true\n\
         }}\n\n",
        implementation.symbol()
    )
}

/// The complete UTF-8 validator both text-route implementations share.
///
/// It admits exactly the well-formed encodings: no overlong form, no
/// surrogate, and nothing above U+10FFFF. It reads no byte past the sequence
/// and writes nothing.
fn emit_utf8_validator() -> String {
    format!(
        "define private i1 @{UTF8_VALIDATOR}(ptr %text, i64 %length) {{\n\
         entry:\n  \
         br label %scan\n\
         scan:\n  \
         %index = phi i64 [ 0, %entry ], [ %next.one, %one ], [ %next.two, %two ], \
         [ %next.three, %three ], [ %next.four, %four ]\n  \
         %more = icmp ult i64 %index, %length\n  \
         br i1 %more, label %lead, label %valid\n\
         lead:\n  \
         %remaining = sub i64 %length, %index\n  \
         %lead.pointer = getelementptr inbounds i8, ptr %text, i64 %index\n  \
         %lead.byte = load i8, ptr %lead.pointer\n  \
         %lead.value = zext i8 %lead.byte to i32\n  \
         %ascii = icmp ult i32 %lead.value, 128\n  \
         br i1 %ascii, label %one, label %multibyte\n\
         one:\n  \
         %next.one = add i64 %index, 1\n  \
         br label %scan\n\
         multibyte:\n  \
         %stray = icmp ult i32 %lead.value, 194\n  \
         br i1 %stray, label %invalid, label %pair.or.longer\n\
         pair.or.longer:\n  \
         %is.pair = icmp ult i32 %lead.value, 224\n  \
         br i1 %is.pair, label %pair, label %triple.or.longer\n\
         pair:\n  \
         %pair.fits = icmp uge i64 %remaining, 2\n  \
         br i1 %pair.fits, label %pair.tail, label %invalid\n\
         pair.tail:\n  \
         %pair.pointer = getelementptr inbounds i8, ptr %lead.pointer, i64 1\n  \
         %pair.byte = load i8, ptr %pair.pointer\n  \
         %pair.value = zext i8 %pair.byte to i32\n  \
         %pair.masked = and i32 %pair.value, 192\n  \
         %pair.ok = icmp eq i32 %pair.masked, 128\n  \
         br i1 %pair.ok, label %two, label %invalid\n\
         two:\n  \
         %next.two = add i64 %index, 2\n  \
         br label %scan\n\
         triple.or.longer:\n  \
         %is.triple = icmp ult i32 %lead.value, 240\n  \
         br i1 %is.triple, label %triple, label %quad.or.invalid\n\
         triple:\n  \
         %triple.fits = icmp uge i64 %remaining, 3\n  \
         br i1 %triple.fits, label %triple.first, label %invalid\n\
         triple.first:\n  \
         %triple.pointer = getelementptr inbounds i8, ptr %lead.pointer, i64 1\n  \
         %triple.byte = load i8, ptr %triple.pointer\n  \
         %triple.value = zext i8 %triple.byte to i32\n  \
         %triple.overlong = icmp eq i32 %lead.value, 224\n  \
         %triple.surrogate = icmp eq i32 %lead.value, 237\n  \
         %triple.low = select i1 %triple.overlong, i32 160, i32 128\n  \
         %triple.high = select i1 %triple.surrogate, i32 159, i32 191\n  \
         %triple.above = icmp uge i32 %triple.value, %triple.low\n  \
         %triple.below = icmp ule i32 %triple.value, %triple.high\n  \
         %triple.ok = and i1 %triple.above, %triple.below\n  \
         br i1 %triple.ok, label %triple.second, label %invalid\n\
         triple.second:\n  \
         %triple.pointer.2 = getelementptr inbounds i8, ptr %lead.pointer, i64 2\n  \
         %triple.byte.2 = load i8, ptr %triple.pointer.2\n  \
         %triple.value.2 = zext i8 %triple.byte.2 to i32\n  \
         %triple.masked.2 = and i32 %triple.value.2, 192\n  \
         %triple.ok.2 = icmp eq i32 %triple.masked.2, 128\n  \
         br i1 %triple.ok.2, label %three, label %invalid\n\
         three:\n  \
         %next.three = add i64 %index, 3\n  \
         br label %scan\n\
         quad.or.invalid:\n  \
         %is.quad = icmp ult i32 %lead.value, 245\n  \
         br i1 %is.quad, label %quad, label %invalid\n\
         quad:\n  \
         %quad.fits = icmp uge i64 %remaining, 4\n  \
         br i1 %quad.fits, label %quad.first, label %invalid\n\
         quad.first:\n  \
         %quad.pointer = getelementptr inbounds i8, ptr %lead.pointer, i64 1\n  \
         %quad.byte = load i8, ptr %quad.pointer\n  \
         %quad.value = zext i8 %quad.byte to i32\n  \
         %quad.overlong = icmp eq i32 %lead.value, 240\n  \
         %quad.beyond = icmp eq i32 %lead.value, 244\n  \
         %quad.low = select i1 %quad.overlong, i32 144, i32 128\n  \
         %quad.high = select i1 %quad.beyond, i32 143, i32 191\n  \
         %quad.above = icmp uge i32 %quad.value, %quad.low\n  \
         %quad.below = icmp ule i32 %quad.value, %quad.high\n  \
         %quad.ok = and i1 %quad.above, %quad.below\n  \
         br i1 %quad.ok, label %quad.second, label %invalid\n\
         quad.second:\n  \
         %quad.pointer.2 = getelementptr inbounds i8, ptr %lead.pointer, i64 2\n  \
         %quad.byte.2 = load i8, ptr %quad.pointer.2\n  \
         %quad.value.2 = zext i8 %quad.byte.2 to i32\n  \
         %quad.masked.2 = and i32 %quad.value.2, 192\n  \
         %quad.ok.2 = icmp eq i32 %quad.masked.2, 128\n  \
         br i1 %quad.ok.2, label %quad.third, label %invalid\n\
         quad.third:\n  \
         %quad.pointer.3 = getelementptr inbounds i8, ptr %lead.pointer, i64 3\n  \
         %quad.byte.3 = load i8, ptr %quad.pointer.3\n  \
         %quad.value.3 = zext i8 %quad.byte.3 to i32\n  \
         %quad.masked.3 = and i32 %quad.value.3, 192\n  \
         %quad.ok.3 = icmp eq i32 %quad.masked.3, 128\n  \
         br i1 %quad.ok.3, label %four, label %invalid\n\
         four:\n  \
         %next.four = add i64 %index, 4\n  \
         br label %scan\n\
         valid:\n  \
         ret i1 true\n\
         invalid:\n  \
         ret i1 false\n\
         }}\n\n"
    )
}

/// Emits the process entry for one program.
///
/// This is the one [QUAL-3] command bootstrap [FN-7, PROG-3].
///
/// `two_worlds` says the module carries a sequential clone of the entry
/// function, in which case the bootstrap also makes the one selection between
/// the two lowerings — `parallel::sequential_clone_set` records why the second
/// one exists. It belongs here because here is the only place in the program
/// that runs exactly once and is inside no loop and no recursion: the clone set
/// is closed upwards through the call graph, so the entry function is in it
/// whenever anything is, and one branch at the bootstrap puts the selection out
/// of every hot path in both worlds. Neither world reaches the other
/// afterwards, so nothing below this branch tests anything again.
pub(super) fn emit_entry(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    main: &IrFunction,
    two_worlds: bool,
) -> Result<String, BackendFailure> {
    let IrEntry::Command { inputs, .. } = program.entry();
    if qualification.kind() != ProgramKind::Command || main.parameters().len() != inputs.len() {
        return Err(BackendFailure::InvalidIr);
    }
    let status = representation(SystemResourceType::ExitStatus);
    if main.result() != system_resource_ir_type(program, SystemResourceType::ExitStatus)? {
        return Err(BackendFailure::InvalidIr);
    }
    let target = qualification.target();
    if target.is_windows() {
        return emit_windows_entry(program, main, two_worlds, status);
    }
    let mut body = String::new();
    // [QUAL-2]: a qualified target that cannot establish command-lifetime
    // argument backing for one invocation refuses startup before entry rather
    // than entering with backing that does not meet the guarantee.
    body.push_str(
        "entry:\n  \
         %argv.present = icmp ne ptr %argv, null\n  \
         %argc.counted = icmp sge i32 %argc, 0\n  \
         %backing = and i1 %argv.present, %argc.counted\n  \
         br i1 %backing, label %normalize, label %start.failure\n",
    );
    // [QUAL-3]: the one-time disposition install belongs to the bootstrap, not
    // to any transfer, so a closed output destination reaches source as a
    // recoverable outcome and no transfer performs a per-call
    // signal-disposition operation.
    writeln!(
        body,
        "normalize:\n  \
         %disposition = call ptr @signal(i32 {}, ptr inttoptr (i64 {} to ptr))\n  \
         %installed = icmp ne ptr %disposition, inttoptr (i64 {} to ptr)\n  \
         br i1 %installed, label %inputs, label %start.failure\n\
         inputs:",
        target.broken_pipe_signal(),
        target.ignored_disposition(),
        target.invalid_disposition(),
    )
    .map_err(|_| BackendFailure::TextEmission)?;

    let mut supplied = Vec::with_capacity(inputs.len());
    let mut opens_directory = false;
    for (ordinal, (_, ty)) in inputs.iter().zip(main.parameters()) {
        // [S22] the general store's provider is the one standard input that
        // is not a [SYS-2] resource: it is the proof-only provider value
        // [PROV-1, STOR-1], so it is checked against `IrType::Provider` and
        // supplied as the zero aggregate.
        match expected_input(*ordinal)? {
            Some(expected) if *ty == system_resource_ir_type(program, expected)? => {}
            None if *ty == IrType::Provider => {}
            _ => return Err(BackendFailure::InvalidIr),
        }
        match ordinal {
            0 => {
                let args = representation(SystemResourceType::Args);
                writeln!(
                    body,
                    "  %count = sext i32 %argc to i64\n  \
                     %args.base = insertvalue {args} zeroinitializer, ptr %argv, 0\n  \
                     %args = insertvalue {args} %args.base, i64 %count, 1"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                supplied.push(format!("{args} %args"));
            }
            1 => {
                opens_directory = true;
                writeln!(
                    body,
                    "  %cwd = call i32 (ptr, i32, ...) @{}(ptr {WORKING_DIRECTORY}, i32 {})",
                    target.directory_open_symbol(),
                    target.directory_open_flags()
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                supplied.push("i32 %cwd".to_owned());
            }
            // The standard output and standard error entry bindings supply
            // separate affine owners over the invocation's own descriptors
            // [SYS-12]; neither is a shared global sink and neither carries a
            // lock.
            2 => {
                supplied.push("i32 1".to_owned());
            }
            3 => {
                supplied.push("i32 2".to_owned());
            }
            // FileFactory is a proof-only affine entry value. Supplying it
            // performs no host allocation and carries no native handle.
            4 => {
                supplied.push("i1 true".to_owned());
            }
            5 => supplied.push(format!("{} zeroinitializer", PROVIDER_REPRESENTATION)),
            _ => return Err(BackendFailure::InvalidIr),
        }
    }
    if opens_directory {
        // [PROG-3]: supplying each declared standard input is a start-time
        // obligation of the selected target; when it cannot supply one, start
        // fails before the entry is invoked.
        writeln!(
            body,
            "  %cwd.opened = icmp sge i32 %cwd, 0\n  br i1 %cwd.opened, label %enter, label %start.failure"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    } else {
        writeln!(body, "  br label %enter").map_err(|_| BackendFailure::TextEmission)?;
    }
    let symbol = source_symbol(main.name());
    let arguments = supplied.join(", ");
    if two_worlds {
        // This run either asked for a pool or it did not, and it cannot change
        // its mind afterwards, so this is the whole of the decision.
        writeln!(
            body,
            "enter:\n  \
             %par.pool = call i32 @wf__par_pool_active()\n  \
             %par.requested = icmp ne i32 %par.pool, 0\n  \
             br i1 %par.requested, label %enter.overlapped, label %enter.sequential\n\
             enter.overlapped:\n  \
             %status.overlapped = call {status} @{symbol}({arguments})\n  \
             br label %enter.selected\n\
             enter.sequential:\n  \
             %status.sequential = call {status} @{}({arguments})\n  \
             br label %enter.selected\n\
             enter.selected:\n  \
             %status = phi {status} [ %status.overlapped, %enter.overlapped ], \
             [ %status.sequential, %enter.sequential ]\n  \
             %code = zext {status} %status to i32\n  \
             ret i32 %code\n\
             start.failure:\n  \
             call void @exit(i32 {START_FAILURE_STATUS})\n  \
             unreachable",
            sequential_clone_symbol(main.name()),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    } else {
        writeln!(
            body,
            "enter:\n  \
             %status = call {status} @{symbol}({arguments})\n  \
             %code = zext {status} %status to i32\n  \
             ret i32 %code\n\
             start.failure:\n  \
             call void @exit(i32 {START_FAILURE_STATUS})\n  \
             unreachable",
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    // The host's entry hands the program to the floor, which runs the
    // bootstrap and the program itself on a stack the compiler sized rather
    // than on whatever the environment's limit left behind. Nothing about the
    // program's meaning moves with it: the same blocks run in the same order,
    // one frame lower down.
    Ok(format!(
        "define i32 @{ENTRY_BODY_SYMBOL}(i32 %argc, ptr %argv) {{\n{body}}}\n\
         \n\
         define i32 @main(i32 %argc, ptr %argv) {{\n  \
         %status = call i32 @wf__floor_run(i32 %argc, ptr %argv)\n  \
         ret i32 %status\n}}\n"
    ))
}

/// Emits the Windows `wmain` bootstrap.  The MSVC Unicode entry preserves the
/// command's native UTF-16 argument backing, and the compiler-owned runtime
/// supplies CRT descriptors backed by the process cwd and standard handles.
fn emit_windows_entry(
    program: &IrProgram<'_, '_, '_>,
    main: &IrFunction,
    two_worlds: bool,
    status: &str,
) -> Result<String, BackendFailure> {
    let IrEntry::Command { inputs, .. } = program.entry();
    let mut body = String::from(
        "entry:\n  \
         %argv.present = icmp ne ptr %argv, null\n  \
         %argc.counted = icmp sge i32 %argc, 0\n  \
         %backing = and i1 %argv.present, %argc.counted\n  \
         br i1 %backing, label %inputs, label %start.failure\n\
         inputs:\n",
    );
    let mut supplied = Vec::with_capacity(inputs.len());
    let mut available = Vec::new();
    for (ordinal, (_, ty)) in inputs.iter().zip(main.parameters()) {
        // [S22] the general store's provider is the one standard input that
        // is not a [SYS-2] resource: it is the proof-only provider value
        // [PROV-1, STOR-1], so it is checked against `IrType::Provider` and
        // supplied as the zero aggregate.
        match expected_input(*ordinal)? {
            Some(expected) if *ty == system_resource_ir_type(program, expected)? => {}
            None if *ty == IrType::Provider => {}
            _ => return Err(BackendFailure::InvalidIr),
        }
        match ordinal {
            0 => {
                let args = representation(SystemResourceType::Args);
                writeln!(
                    body,
                    "  %count = sext i32 %argc to i64\n  \
                     %args.base = insertvalue {args} zeroinitializer, ptr %argv, 0\n  \
                     %args = insertvalue {args} %args.base, i64 %count, 1"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                supplied.push(format!("{args} %args"));
            }
            1 => {
                body.push_str(
                    "  %cwd = call i32 (ptr, i32, ...) @wf__windows_open_cwd(ptr null, i32 0)\n  \
                     %cwd.available = icmp sge i32 %cwd, 0\n",
                );
                supplied.push("i32 %cwd".to_owned());
                available.push("%cwd.available");
            }
            2 => {
                body.push_str(
                    "  %stdout = call i32 @wf__windows_stdout_descriptor()\n  \
                     %stdout.available = icmp sge i32 %stdout, 0\n",
                );
                supplied.push("i32 %stdout".to_owned());
                available.push("%stdout.available");
            }
            3 => {
                body.push_str(
                    "  %stderr = call i32 @wf__windows_stderr_descriptor()\n  \
                     %stderr.available = icmp sge i32 %stderr, 0\n",
                );
                supplied.push("i32 %stderr".to_owned());
                available.push("%stderr.available");
            }
            4 => supplied.push("i1 true".to_owned()),
            5 => supplied.push(format!("{} zeroinitializer", PROVIDER_REPRESENTATION)),
            _ => return Err(BackendFailure::InvalidIr),
        }
    }
    let ready = match available.as_slice() {
        [] => "true".to_owned(),
        [only] => (*only).to_owned(),
        [first, rest @ ..] => {
            let mut previous = (*first).to_owned();
            for (index, condition) in rest.iter().enumerate() {
                let next = format!("%standard.inputs.{index}");
                writeln!(body, "  {next} = and i1 {previous}, {condition}")
                    .map_err(|_| BackendFailure::TextEmission)?;
                previous = next;
            }
            previous
        }
    };
    writeln!(body, "  br i1 {ready}, label %enter, label %start.failure")
        .map_err(|_| BackendFailure::TextEmission)?;

    let symbol = source_symbol(main.name());
    let arguments = supplied.join(", ");
    if two_worlds {
        writeln!(
            body,
            "enter:\n  \
             %par.pool = call i32 @wf__par_pool_active()\n  \
             %par.requested = icmp ne i32 %par.pool, 0\n  \
             br i1 %par.requested, label %enter.overlapped, label %enter.sequential\n\
             enter.overlapped:\n  \
             %status.overlapped = call {status} @{symbol}({arguments})\n  \
             br label %enter.selected\n\
             enter.sequential:\n  \
             %status.sequential = call {status} @{}({arguments})\n  \
             br label %enter.selected\n\
             enter.selected:\n  \
             %status = phi {status} [ %status.overlapped, %enter.overlapped ], \
             [ %status.sequential, %enter.sequential ]\n  \
             %code = zext {status} %status to i32\n  \
             ret i32 %code\n\
             start.failure:\n  \
             call void @exit(i32 {START_FAILURE_STATUS})\n  \
             unreachable",
            sequential_clone_symbol(main.name()),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    } else {
        writeln!(
            body,
            "enter:\n  \
             %status = call {status} @{symbol}({arguments})\n  \
             %code = zext {status} %status to i32\n  \
             ret i32 %code\n\
             start.failure:\n  \
             call void @exit(i32 {START_FAILURE_STATUS})\n  \
             unreachable"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    Ok(format!(
        "define i32 @{ENTRY_BODY_SYMBOL}(i32 %argc, ptr %argv) {{\n{body}}}\n\
         \n\
         define i32 @wmain(i32 %argc, ptr %argv) {{\n  \
         %status = call i32 @wf__floor_run(i32 %argc, ptr %argv)\n  \
         ret i32 %status\n}}\n"
    ))
}

/// The [FN-7] standard-input row one table ordinal selects.
/// The [SYS-2] resource one standard input supplies, or `None` for the one
/// row whose value is a provider rather than a system resource [S22].
fn expected_input(ordinal: u8) -> Result<Option<SystemResourceType>, BackendFailure> {
    match ordinal {
        0 => Ok(Some(SystemResourceType::Args)),
        1 => Ok(Some(SystemResourceType::DirectoryRead)),
        2 | 3 => Ok(Some(SystemResourceType::Output)),
        4 => Ok(Some(SystemResourceType::FileFactory)),
        5 => Ok(None),
        _ => Err(BackendFailure::InvalidIr),
    }
}

/// The LLVM representation of `IrType::Provider`, which the entry writes
/// literally because the bootstrap holds no `IrProgram` type table position
/// for it.
const PROVIDER_REPRESENTATION: &str = "{ ptr, i64 }";

fn system_resource_ir_type(
    program: &IrProgram<'_, '_, '_>,
    resource: SystemResourceType,
) -> Result<IrType, BackendFailure> {
    let mut selected = None;
    for nominal in program.nominals() {
        let IrNominalKind::SystemResource(contract) = nominal.kind() else {
            continue;
        };
        if contract.resource != resource {
            continue;
        }
        if selected.replace(nominal.id()).is_some() {
            return Err(BackendFailure::InvalidIr);
        }
    }
    selected
        .map(IrType::Nominal)
        .ok_or(BackendFailure::InvalidIr)
}

impl FunctionEmitter<'_, '_> {
    /// Emits one call to the approved implementation of a semantic identity.
    ///
    /// Selection happened once, at qualification; this site emits one direct
    /// call to the private ABI symbol that lookup fixed [QUAL-1, QUAL-3].
    pub(super) fn emit_system_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: crate::IrSystemOperation,
        arguments: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        let implementation = self.qualification.operation(operation)?;
        let row = crate::SYSTEM_OPERATIONS
            .get(usize::from(operation.ordinal()))
            .ok_or(BackendFailure::InvalidIr)?;
        if row.parameters.len() != arguments.len() {
            return Err(BackendFailure::InvalidIr);
        }
        let mut rendered = Vec::with_capacity(arguments.len() + 2);
        for (argument, parameter) in arguments.iter().zip(row.parameters) {
            let argument_type = self
                .function
                .value_type(*argument)
                .ok_or(BackendFailure::InvalidIr)?;
            if !system_operand_admits(parameter.ty, argument_type)?
                && argument_type != catalog_ir_type(self.program, parameter.ty)?
            {
                return Err(BackendFailure::InvalidIr);
            }
            if proof_only_resource(self.program, argument_type)? {
                continue;
            }
            let rendered_type = llvm_type(self.program, argument_type)?;
            rendered.push(format!("{rendered_type} {}", value_name(*argument)));
        }
        writeln!(
            self.output,
            "  {} = call {} @{}({})",
            value_name(result),
            llvm_type(self.program, ty)?,
            implementation.symbol(),
            rendered.join(", ")
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}

/// Whether one checked system argument has no target ABI representation.
pub(super) fn proof_only_resource(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<bool, BackendFailure> {
    let IrType::Nominal(id) = ty else {
        return Ok(false);
    };
    let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
    let IrNominalKind::SystemResource(contract) = nominal.kind() else {
        return Ok(false);
    };
    Ok(matches!(
        qualified_representation(contract.resource),
        super::super::qualification::ResourceRepresentation::ProofToken
    ))
}

/// Emits one type's compiler-derived [SYS-5] release action.
///
/// A logical consume and a logical source detach emit nothing: they perform no
/// host call, no target call, no handle lookup, no byte copy, and no external
/// effect. A native close attempt is exactly one direct close whose diagnostic
/// is discarded and which never retries an ambiguous close.
pub(super) fn emit_resource_release(
    qualification: &Qualification,
    output: &mut String,
    temporary: &mut u32,
    contract: crate::SystemResourceContract,
    operand: &str,
) -> Result<(), BackendFailure> {
    match qualification.resource(contract.resource)?.release() {
        ReleaseImplementation::NoCode => Ok(()),
        ReleaseImplementation::NativeClose(symbol) => {
            let discarded = *temporary;
            *temporary = temporary
                .checked_add(1)
                .ok_or(BackendFailure::CounterOverflow)?;
            writeln!(
                output,
                "  %release.{discarded} = call i32 @{symbol}({} {operand})",
                representation(contract.resource)
            )
            .map_err(|_| BackendFailure::TextEmission)
        }
    }
}
