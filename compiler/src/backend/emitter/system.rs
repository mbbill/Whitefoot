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
    ApprovedImplementation, ORIGIN_DIRECTORY_OPEN, ORIGIN_NONE, ORIGIN_READ, ORIGIN_WRITE,
    ProgramKind, Qualification, ReleaseImplementation, SystemTarget, qualified_representation,
};
use super::*;
use crate::ACTIVE_KERNEL_SPEC_VERSION;

/// The status a start failure ends the process with.
///
/// [PROG-3]: when the selected target cannot supply a declared standard input
/// or the [QUAL-2] backing guarantee, start fails before the entry is invoked,
/// no source statement executes, and no `ExitStatus` is produced. The value is
/// this bootstrap's own operating-system-error convention and is deliberately
/// not an `ExitStatus`: the language defines no process status for a start
/// failure, and this one is never produced by a returned command code path.
const START_FAILURE_STATUS: i32 = 71;

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

/// The longest single path component the [SYS-14] name operations admit.
///
/// Both qualified families cap one component at this many bytes, so the
/// bounded stack slot the directory-relative facility is handed is exactly
/// this size plus its terminator. A longer name is rejected as an invalid
/// component before any host call.
const COMPONENT_LIMIT: u64 = 255;

/// The portable [SYS-14] entry-kind values written into the destination.
const KIND_UNKNOWN: u8 = 0;
const KIND_REGULAR: u8 = 1;
const KIND_DIRECTORY: u8 = 2;
const KIND_SYMLINK: u8 = 3;
const KIND_OTHER: u8 = 4;

/// The portable [SYS-14] entry record header: one kind byte and one name
/// length byte, ahead of the name bytes themselves.
const ENTRY_HEADER: u64 = 2;

/// The private symbol of the shared UTF-8 validator both text-route
/// implementations use [HOST-2].
const UTF8_VALIDATOR: &str = "wf.sys.utf8.valid";

/// The private symbol of the one cold [SYS-7] outcome mapper every failing
/// I/O implementation shares [QUAL-3].
const IO_ERROR_MAPPER: &str = "wf.sys.io.error";

/// The private constant naming the initial working directory.
const WORKING_DIRECTORY: &str = "@.wf.sys.working.directory";

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
) -> Result<SystemEmission, BackendFailure> {
    let mut constants = String::new();
    let mut declarations: BTreeSet<String> = BTreeSet::new();
    let mut definitions = String::new();
    let mut needs_validator = false;
    // The command bootstrap and `open_list` both name the self component, so
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
            ARG_GET => definitions.push_str(&emit_arg_get(program, implementation, result)?),
            HOST_BYTES_LEN => definitions.push_str(&emit_host_bytes_len(implementation)),
            HOST_COPY_BYTES => {
                definitions.push_str(&emit_host_copy_bytes(program, implementation, result)?);
            }
            HOST_UTF8_LEN => {
                needs_validator = true;
                definitions.push_str(&emit_host_utf8_len(program, implementation, result)?);
            }
            HOST_COPY_UTF8 => {
                needs_validator = true;
                definitions.push_str(&emit_host_copy_utf8(program, implementation, result)?);
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
                definitions.push_str(&emit_open_read(implementation, &shape, target)?);
            }
            READ_ONCE => {
                let shape = read_outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.failed_type)?;
                definitions.push_str(&emit_read_once(program, implementation, &shape, target)?);
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
                    implementation,
                    &shape,
                    target,
                )?);
            }
            OPEN_LIST => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                needs_working_directory = true;
                definitions.push_str(&emit_open_list(implementation, &shape, target)?);
            }
            LIST_ONCE => {
                let shape = list_outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.failed_type)?;
                definitions.push_str(&emit_list_once(program, implementation, &shape, target)?);
            }
            OPEN_FILE => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_open_file(program, implementation, &shape, target)?);
            }
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

    // Which facility a close or a directory open reaches is the target column
    // of the [QUAL-1] row, so both symbols come from the qualification rather
    // than from a fixed name here.
    if let Some(symbol) = native_release_symbol(program, qualification)? {
        declarations.insert(format!("declare i32 @{symbol}(i32)"));
    }

    if let IrEntry::Command { inputs, .. } = program.entry() {
        declarations.insert("declare ptr @signal(i32, ptr)".to_owned());
        declarations.insert("declare void @exit(i32) noreturn".to_owned());
        if inputs.contains(&1) {
            declarations.insert(format!(
                "declare i32 @{}(ptr, i32, ...)",
                qualification.target().directory_open_symbol()
            ));
            needs_working_directory = true;
        }
    }
    if needs_working_directory {
        constants.push_str(&format!(
            "{WORKING_DIRECTORY} = private unnamed_addr constant [2 x i8] c\".\\00\", align 1\n"
        ));
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
        ARG_GET => &["declare i64 @strlen(ptr)"],
        HOST_COPY_BYTES | HOST_COPY_UTF8 => {
            &["declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)"]
        }
        RELATIVE_PATH => &["declare ptr @memchr(ptr, i32, i64)"],
        // [PATH-2]: the target's own directory-relative facility, never a
        // prefix concatenated onto a path and resolved against an ambient
        // working directory.
        OPEN_READ | OPEN_LIST => {
            return Ok(vec![format!(
                "declare i32 @{}(i32, ptr, i32, ...)",
                target.file_open_symbol()
            )]);
        }
        OPEN_DIRECTORY | OPEN_FILE => {
            return Ok(vec![
                format!(
                    "declare i32 @{}(i32, ptr, i32, ...)",
                    target.file_open_symbol()
                ),
                "declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)".to_owned(),
            ]);
        }
        READ_ONCE => {
            return Ok(vec![format!(
                "declare i64 @{}(i32, ptr, i64)",
                target.read_symbol()
            )]);
        }
        WRITE_ONCE => {
            return Ok(vec![format!(
                "declare i64 @{}(i32, ptr, i64)",
                target.write_symbol()
            )]);
        }
        // The target's own enumeration facility [SYS-14]; qualification
        // already refused a target that supplies none.
        LIST_ONCE => {
            let enumeration = target
                .directory_enumeration()
                .ok_or(BackendFailure::InvalidIr)?;
            return Ok(vec![enumeration.declaration().to_owned()]);
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
         %length = call i64 @strlen(ptr %text)\n  \
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

fn emit_host_bytes_len(implementation: ApprovedImplementation) -> String {
    let lease = representation(SystemResourceType::HostString);
    format!(
        "define private i64 @{}({lease} %value) alwaysinline {{\n\
         entry:\n  \
         %length = extractvalue {lease} %value, 1\n  \
         ret i64 %length\n\
         }}\n\n",
        implementation.symbol()
    )
}

fn emit_host_utf8_len(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    result: IrType,
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
/// obligation proves `end <= len(buffer)`, so this wrapper has no check or
/// trap fallback.
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
    // The lossless route transfers the target's own code units with no
    // validation and no Unicode restriction [HOST-2]; its only recoverable
    // failure is a destination too small for the exact length, which leaves
    // the whole destination buffer unchanged [SYS-8].
    Ok(format!(
        "define private {llvm} @{symbol}({lease} %value, {buffer} %destination, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         %required = extractvalue {lease} %value, 1\n  \
         %room = icmp ule i64 %required, %extent\n  \
         br i1 %room, label %transfer, label %small\n\
         transfer:\n  \
         %source = extractvalue {lease} %value, 0\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %target = getelementptr inbounds i8, ptr %base, i64 %start\n  \
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
    implementation: ApprovedImplementation,
    result: IrType,
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
    let entry = range_entry("");
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
         %target = getelementptr inbounds i8, ptr %base, i64 %start\n  \
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

/// Resolves the closed thirty-class [SYS-7] set in one program's `IoError`.
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
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
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
    // [PATH-2]: the path is resolved against the capability's own directory
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
    Ok(format!(
        "define private {llvm} @{symbol}({directory} %root, {path} %path) alwaysinline {{\n\
         entry:\n  \
         %text = extractvalue {path} %path, 0\n  \
         %descriptor = call {file} (i32, ptr, i32, ...) @{open}({directory} %root, ptr %text, \
         i32 {flags})\n  \
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
    ))
}

fn emit_read_once(
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
        end_tag,
        failed_tag,
        failed_index,
        failed_llvm,
        ..
    } = shape;
    let entry = range_entry("");
    let (read_error, error) = native_error(target, "failure");
    // The two call-site SYS-8 goals authorize this half-open range. A
    // zero-length range reports `next = start` and issues no host transfer,
    // and is never
    // reported as `ReadEnd`. A nonempty range makes at most one host transfer
    // attempt: reported progress is returned immediately and never hidden by a
    // second attempt, so `ReadBytes(next)` advances beyond start only for
    // positive host progress,
    // than zero, only `ReadEnd` states that no byte was available at the
    // observed end, and a reported interruption reaches source as
    // `Interrupted` rather than being retried. The host advances the file
    // cursor by exactly `next - start` [SYS-11], and exactly `[start, next)`
    // of the requested range may have changed.
    Ok(format!(
        "define private {llvm} @{symbol}({file} %file, {buffer} %destination, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         %vacant = icmp eq i64 %extent, 0\n  \
         br i1 %vacant, label %empty, label %transfer\n\
         empty:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %empty.outcome = insertvalue {llvm} %empty.tag, i64 %start, {bytes_index}\n  \
         ret {llvm} %empty.outcome\n\
         transfer:\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %target = getelementptr inbounds i8, ptr %base, i64 %start\n  \
         %transferred = call i64 @{read}({file} %file, ptr %target, i64 %extent)\n  \
         %progress = icmp sgt i64 %transferred, 0\n  \
         br i1 %progress, label %bytes, label %quiet\n\
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
         failure:\n\
         {read_error}  \
         %failure.error = call {failed_llvm} @{IO_ERROR_MAPPER}(i32 {error}, i8 {ORIGIN_READ})\n  \
         %failed.tag = insertvalue {llvm} zeroinitializer, i32 {failed_tag}, 0\n  \
         %failed.outcome = insertvalue {llvm} %failed.tag, {failed_llvm} %failure.error, \
         {failed_index}\n  \
         ret {llvm} %failed.outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        read = target.read_symbol()
    ))
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
        err_tag,
        err_index,
        err_llvm,
        ..
    } = shape;
    let entry = range_entry("");
    let (read_error, error) = native_error(target, "failure");
    // A host zero-length write is `Err(WriteZero())`, which no native error
    // code produced: [SYS-7] leaves both detail fields zero when the target
    // supplies no value for them.
    let (refused_value, refused_error) =
        io_error_value(err_llvm, refused, "refused", "0", &ORIGIN_NONE.to_string());
    // At most one host output attempt [SYS-12]. A zero-length range reports
    // `next = start` and issues no host transfer; otherwise `Ok(next)` means
    // exactly that the host accepted `[start, next)`, promising neither line
    // atomicity nor durability. A closed destination arrives as
    // the recoverable `BrokenPipe` class because the bootstrap installed the
    // ignored write-to-closed-pipe disposition once, before entry [QUAL-3];
    // this path performs no per-call signal-disposition operation.
    Ok(format!(
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
         %progress = icmp sgt i64 %accepted, 0\n  \
         br i1 %progress, label %ok, label %quiet\n\
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
         failure:\n\
         {read_error}  \
         %failure.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 {error}, i8 {ORIGIN_WRITE})\n  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err.outcome = insertvalue {llvm} %err.tag, {err_llvm} %failure.error, {err_index}\n  \
         ret {llvm} %err.outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        write = target.write_symbol()
    ))
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
fn component_validation(buffer: &str, root: u32) -> String {
    format!(
        "measure:\n  \
         %oversize = icmp ugt i64 %extent, {COMPONENT_LIMIT}\n  \
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
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    emit_open_by_name(
        program,
        implementation,
        shape,
        target,
        SystemResourceType::DirectoryRead,
        target.directory_open_flags(),
    )
}

/// Emits the approved implementation of the candidate `open_file` [SYS-11].
///
/// It differs from `open_directory` in exactly the two places the two rows
/// differ: the flags the target's own directory-relative facility is handed,
/// and the resource the returned descriptor becomes. Everything the shared
/// emitter performs — the statically discharged [SYS-8] range entry, component
/// validation, bounded terminating slot, one host call, and one cold mapper —
/// is the same because [SYS-11] states it by mirroring [SYS-14].
fn emit_open_file(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    emit_open_by_name(
        program,
        implementation,
        shape,
        target,
        SystemResourceType::ReadFile,
        target.file_open_flags(),
    )
}

/// Emits one open-by-name implementation [SYS-11, SYS-14].
///
/// The name arrives as caller-owned bytes and never becomes a path value, so
/// [HOST-3]'s command-lifetime backing and [PATH-1]'s inline lease are
/// untouched. The validated component is copied into one bounded stack slot
/// only to terminate it for the target's own directory-relative facility,
/// which then resolves it against the capability's directory object exactly
/// as `open_read` does [PATH-2].
fn emit_open_by_name(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
    opened: SystemResourceType,
    flags: i32,
) -> Result<String, BackendFailure> {
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
    let slot = COMPONENT_LIMIT + 1;
    let entry = range_entry(&format!("  %component = alloca [{slot} x i8], align 1\n"));
    let component = component_validation(&buffer, u32::from(target.root_prefix()));
    let (read_error, error) = native_error(target, "failure");
    let (invalid_value, invalid_error) = invalid_component(program, err_llvm, *err_type)?;
    Ok(format!(
        "define private {llvm} @{symbol}({directory} %root, {buffer} %name, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         {component}\
         open:\n  \
         call void @llvm.memcpy.p0.p0.i64(ptr %component, ptr %text, i64 %extent, i1 false)\n  \
         %terminator = getelementptr inbounds i8, ptr %component, i64 %extent\n  \
         store i8 0, ptr %terminator, align 1\n  \
         %descriptor = call {opened} (i32, ptr, i32, ...) @{open}({directory} %root, \
         ptr %component, i32 {flags})\n  \
         %opened = icmp sge {opened} %descriptor, 0\n  \
         br i1 %opened, label %live, label %failure\n\
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

/// Emits the approved implementation of `open_list` [SYS-14].
///
/// One enumeration handle is an independent descriptor opened against the
/// capability's own directory object through the same directory-relative
/// facility [PATH-2], named by the self component. It therefore carries its
/// own cursor and aliases the capability no more than `open_read`'s
/// `ReadFile` does [SYS-10].
fn emit_open_list(
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let directory = representation(SystemResourceType::DirectoryRead);
    let list = representation(SystemResourceType::DirectoryList);
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
    Ok(format!(
        "define private {llvm} @{symbol}({directory} %directory) alwaysinline {{\n\
         entry:\n  \
         %descriptor = call {list} (i32, ptr, i32, ...) @{open}({directory} %directory, \
         ptr {WORKING_DIRECTORY}, i32 {flags})\n  \
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
    ))
}

/// Emits the approved implementation of `list_once` [SYS-14].
///
/// The shape is [SYS-8]'s: enter the statically authorized range, make at most
/// one host call, then one outcome check and a cold mapper [QUAL-3]. The one addition
/// is normalization — the facility writes its own records into the caller's
/// range, and the shim rewrites them in place as the portable
/// `[kind][name length][name bytes]` sequence [SYS-14]. The rewrite moves
/// every byte strictly toward the front, because a portable record's two-byte
/// header is smaller than any native record's, so no unread byte is ever
/// overwritten. Every native header field is validated against the reported
/// extent before it is used, so a record the facility mis-sizes ends the walk
/// instead of reading past the range.
fn emit_list_once(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &ListOutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let list = representation(SystemResourceType::DirectoryList);
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
    let entry = range_entry("  %position = alloca i64, align 8\n");
    let (read_error, error) = native_error(target, "failure");
    let name_offset = enumeration.name_offset();
    let record_length_offset = enumeration.record_length_offset();
    let name_length_offset = enumeration.name_length_offset();
    let entry_type_offset = enumeration.entry_type_offset();
    let native_regular = enumeration.native_regular();
    let native_directory = enumeration.native_directory();
    let native_symlink = enumeration.native_symlink();
    let native_unknown = enumeration.native_unknown();
    Ok(format!(
        "define private {llvm} @{symbol}({list} %list, {buffer} %destination, i64 %start, \
         i64 %end) alwaysinline {{\n\
         {entry}\
         measure:\n  \
         store i64 0, ptr %position, align 8\n  \
         %empty.range = icmp eq i64 %extent, 0\n  \
         br i1 %empty.range, label %empty, label %transfer\n\
         empty:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %empty.count = insertvalue {llvm} %empty.tag, i64 %start, {bytes_index}\n  \
         %empty.outcome = insertvalue {llvm} %empty.count, i64 0, {entries_index}\n  \
         ret {llvm} %empty.outcome\n\
         transfer:\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %window = getelementptr inbounds i8, ptr %base, i64 %start\n  \
         %filled = call i64 @{enumerate}({list} %list, ptr %window, i64 %extent, \
         ptr %position)\n  \
         %progress = icmp sgt i64 %filled, 0\n  \
         br i1 %progress, label %normalize, label %quiet\n\
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
         normalize:\n  \
         br label %walk\n\
         walk:\n  \
         %source = phi i64 [ 0, %normalize ], [ %source.next, %step ]\n  \
         %written = phi i64 [ 0, %normalize ], [ %written.next, %step ]\n  \
         %entries = phi i64 [ 0, %normalize ], [ %entries.next, %step ]\n  \
         %remaining = sub i64 %filled, %source\n  \
         %headerless = icmp ult i64 %remaining, {name_offset}\n  \
         br i1 %headerless, label %done, label %header\n\
         header:\n  \
         %entry.record = getelementptr inbounds i8, ptr %window, i64 %source\n  \
         %record.extent.at = getelementptr inbounds i8, ptr %entry.record, i64 {record_length_offset}\n  \
         %record.extent.native = load i16, ptr %record.extent.at, align 1\n  \
         %record.extent = zext i16 %record.extent.native to i64\n  \
         %named.at = getelementptr inbounds i8, ptr %entry.record, i64 {name_length_offset}\n  \
         %named.native = load i16, ptr %named.at, align 1\n  \
         %named = zext i16 %named.native to i64\n  \
         %kind.at = getelementptr inbounds i8, ptr %entry.record, i64 {entry_type_offset}\n  \
         %kind.native = load i8, ptr %kind.at, align 1\n  \
         %kind.value = zext i8 %kind.native to i64\n  \
         %needed = add i64 {name_offset}, %named\n  \
         %sized = icmp uge i64 %record.extent, %needed\n  \
         %bounded = icmp ule i64 %record.extent, %remaining\n  \
         %advancing = icmp uge i64 %record.extent, 1\n  \
         %nameable = icmp ule i64 %named, {COMPONENT_LIMIT}\n  \
         %naming = icmp uge i64 %named, 1\n  \
         %named.usable = and i1 %nameable, %naming\n  \
         %consistent = and i1 %sized, %bounded\n  \
         %progressive = and i1 %advancing, %named.usable\n  \
         %usable = and i1 %consistent, %progressive\n  \
         br i1 %usable, label %room, label %done\n\
         room:\n  \
         %portable = add i64 {ENTRY_HEADER}, %named\n  \
         %after = add i64 %written, %portable\n  \
         %fits = icmp ule i64 %after, %extent\n  \
         br i1 %fits, label %record.header, label %done\n\
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
         %target.named = getelementptr inbounds i8, ptr %target.record, i64 1\n  \
         %named.byte = trunc i64 %named to i8\n  \
         store i8 %named.byte, ptr %target.named, align 1\n  \
         %target.name = getelementptr inbounds i8, ptr %target.record, i64 {ENTRY_HEADER}\n  \
         %source.name = getelementptr inbounds i8, ptr %entry.record, i64 {name_offset}\n  \
         br label %copy\n\
         copy:\n  \
         %copied = phi i64 [ 0, %record.header ], [ %copied.next, %copy.step ]\n  \
         %copy.done = icmp uge i64 %copied, %named\n  \
         br i1 %copy.done, label %step, label %copy.step\n\
         copy.step:\n  \
         %copy.from = getelementptr inbounds i8, ptr %source.name, i64 %copied\n  \
         %copy.byte = load i8, ptr %copy.from, align 1\n  \
         %copy.to = getelementptr inbounds i8, ptr %target.name, i64 %copied\n  \
         store i8 %copy.byte, ptr %copy.to, align 1\n  \
         %copied.next = add i64 %copied, 1\n  \
         br label %copy\n\
         step:\n  \
         %source.next = add i64 %source, %record.extent\n  \
         %written.next = add i64 %written, %portable\n  \
         %entries.next = add i64 %entries, 1\n  \
         br label %walk\n\
         done:\n  \
         %final.written = phi i64 [ %written, %walk ], [ %written, %header ], \
         [ %written, %room ]\n  \
         %final.entries = phi i64 [ %entries, %walk ], [ %entries, %header ], \
         [ %entries, %room ]\n  \
         %next = add nuw i64 %start, %final.written\n  \
         %bytes.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %bytes.count = insertvalue {llvm} %bytes.tag, i64 %next, {bytes_index}\n  \
         %bytes.outcome = insertvalue {llvm} %bytes.count, i64 %final.entries, \
         {entries_index}\n  \
         ret {llvm} %bytes.outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        enumerate = enumeration.symbol()
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
/// For a `command` this is the [QUAL-3] bootstrap; for the unlabelled entry it
/// is the unchanged wrapper that produces no status [FN-7, PROG-3].
pub(super) fn emit_entry(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    main: &IrFunction,
    target_layout: TargetLayout,
    traps: &mut Vec<Vec<u8>>,
    intrinsics: &mut BTreeSet<IntrinsicDeclaration>,
) -> Result<String, BackendFailure> {
    let IrEntry::Command { inputs, .. } = program.entry() else {
        if qualification.kind() != ProgramKind::Unlabelled
            || main.result() != IrType::Unit
            || !main.parameters().is_empty()
        {
            return Err(BackendFailure::InvalidIr);
        }
        if let Some(goal) = program.entry_goal() {
            let emitted = FunctionEmitter::new(
                program,
                qualification,
                main,
                target_layout,
                traps,
                intrinsics,
            )
            .with_entry_goal(goal, Vec::new())?
            .emit_entry_goal()?;
            return Ok(format!(
                "define i32 @main() {{\nentry:\n{}  br i1 {}, label %enter, label %entry.requirement.trap\nentry.requirement.trap:\n  call void @wf_trap(ptr @.wf_trap.{}, i64 {})\n  unreachable\nenter:\n  %result = call i8 @{}()\n  ret i32 0\n}}\n",
                emitted.definitions,
                emitted.condition,
                emitted.trap,
                emitted.trap_length,
                source_symbol(main.name())
            ));
        }
        return Ok(format!(
            "define i32 @main() {{\nentry:\n  %result = call i8 @{}()\n  ret i32 0\n}}\n",
            source_symbol(main.name())
        ));
    };
    if qualification.kind() != ProgramKind::Command || main.parameters().len() != inputs.len() {
        return Err(BackendFailure::InvalidIr);
    }
    let status = representation(SystemResourceType::ExitStatus);
    if llvm_type(program, main.result())? != status {
        return Err(BackendFailure::InvalidIr);
    }
    let target = qualification.target();
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
    let mut supplied_names = Vec::with_capacity(inputs.len());
    let mut opens_directory = false;
    for (ordinal, (_, ty)) in inputs.iter().zip(main.parameters()) {
        let expected = expected_input(*ordinal)?;
        if llvm_type(program, *ty)? != representation(expected) {
            return Err(BackendFailure::InvalidIr);
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
                supplied_names.push("%args".to_owned());
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
                supplied_names.push("%cwd".to_owned());
            }
            // The standard output and standard error entry bindings supply
            // separate affine owners over the invocation's own descriptors
            // [SYS-12]; neither is a shared global sink and neither carries a
            // lock.
            2 => {
                supplied.push("i32 1".to_owned());
                supplied_names.push("1".to_owned());
            }
            3 => {
                supplied.push("i32 2".to_owned());
                supplied_names.push("2".to_owned());
            }
            _ => return Err(BackendFailure::InvalidIr),
        }
    }
    let entry_goal = program
        .entry_goal()
        .map(|goal| {
            FunctionEmitter::new(
                program,
                qualification,
                main,
                target_layout,
                traps,
                intrinsics,
            )
            .with_entry_goal(goal, supplied_names)?
            .emit_entry_goal()
        })
        .transpose()?;
    let post_setup = if entry_goal.is_some() {
        "entry.goal"
    } else {
        "enter"
    };
    if opens_directory {
        // [PROG-3]: supplying each declared standard input is a start-time
        // obligation of the selected target; when it cannot supply one, start
        // fails before the entry is invoked.
        writeln!(
            body,
            "  %cwd.opened = icmp sge i32 %cwd, 0\n  br i1 %cwd.opened, label %{post_setup}, label %start.failure"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    } else {
        writeln!(body, "  br label %{post_setup}").map_err(|_| BackendFailure::TextEmission)?;
    }
    if let Some(emitted) = entry_goal {
        write!(
            body,
            "entry.goal:\n{}  br i1 {}, label %enter, label %entry.requirement.trap\nentry.requirement.trap:\n  call void @wf_trap(ptr @.wf_trap.{}, i64 {})\n  unreachable\n",
            emitted.definitions, emitted.condition, emitted.trap, emitted.trap_length
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    writeln!(
        body,
        "enter:\n  \
         %status = call {status} @{}({})\n  \
         %code = zext {status} %status to i32\n  \
         ret i32 %code\n\
         start.failure:\n  \
         call void @exit(i32 {START_FAILURE_STATUS})\n  \
         unreachable",
        source_symbol(main.name()),
        supplied.join(", ")
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    Ok(format!(
        "define i32 @main(i32 %argc, ptr %argv) {{\n{body}}}\n"
    ))
}

/// The [FN-7] standard-input row one table ordinal selects.
fn expected_input(ordinal: u8) -> Result<SystemResourceType, BackendFailure> {
    match ordinal {
        0 => Ok(SystemResourceType::Args),
        1 => Ok(SystemResourceType::DirectoryRead),
        2 | 3 => Ok(SystemResourceType::Output),
        _ => Err(BackendFailure::InvalidIr),
    }
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
            let rendered_type = llvm_type(self.program, argument_type)?;
            if rendered_type != catalog_llvm_type(parameter.ty)? {
                return Err(BackendFailure::InvalidIr);
            }
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

/// The emitted type of one [SYS-2] table type.
fn catalog_llvm_type(ty: crate::SystemTypeRef) -> Result<String, BackendFailure> {
    Ok(match ty {
        crate::SystemTypeRef::U8 => "i8".to_owned(),
        crate::SystemTypeRef::U32 => "i32".to_owned(),
        crate::SystemTypeRef::U64 => "i64".to_owned(),
        crate::SystemTypeRef::BufferU8 => "{ ptr, i64 }".to_owned(),
        crate::SystemTypeRef::Nominal(nominal) => {
            let contract =
                crate::system_resource_contract(nominal).ok_or(BackendFailure::InvalidIr)?;
            representation(contract.resource).to_owned()
        }
        // No [SYS-2] parameter is an outcome type.
        crate::SystemTypeRef::Result { .. } => return Err(BackendFailure::InvalidIr),
    })
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
