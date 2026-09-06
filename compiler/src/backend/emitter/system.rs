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
    ORIGIN_DIRECTORY_OPEN, ORIGIN_NONE, ORIGIN_READ, ORIGIN_SOCKET_ACCEPT, ORIGIN_SOCKET_CONNECT,
    ORIGIN_SOCKET_LISTEN, ORIGIN_WRITE, ProgramKind, Qualification, ReleaseImplementation,
    ResourceRepresentation, SystemTarget, qualified_representation,
};
use super::completion::{
    CompletionRetirement, DIRECTORY_NEXT_SUBMIT, FILE_JOIN, SOCKET_ACCEPT_JOIN,
    SOCKET_ACCEPT_SUBMIT, SOCKET_CONNECT_SUBMIT, SOCKET_LISTEN_SUBMIT, SOCKET_RECEIVE_SUBMIT,
    SOCKET_SEND_SUBMIT, SOCKET_SHUTDOWN_SUBMIT, WRAPPER_RAW_ERROR, WRAPPER_RAW_OUTCOME,
    WRAPPER_RAW_VALUE, WRAPPER_RECORD, completion_retirement, completion_submit_call,
    completion_transfer_target, completion_wrapper_reservation,
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
const RESERVE_HANDLE: u8 = 15;
const CLOSE_READ: u8 = 16;
const CLOSE_DIRECTORY: u8 = 17;
const CLOSE_DIRECTORY_SOURCE: u8 = 18;
const READ_NEXT: u8 = 19;
const SOCKET_ADDRESS_V4: u8 = 20;
const SOCKET_ADDRESS_V6: u8 = 21;
const TCP_LISTEN: u8 = 22;
const TCP_ACCEPT: u8 = 23;
const TCP_CONNECT: u8 = 24;
const RECEIVE_NEXT: u8 = 25;
const SEND_ONCE: u8 = 26;
const CLOSE_CONNECTION: u8 = 27;
const CLOSE_LISTENER: u8 = 28;

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
    /// One transfer attempt on one direction of one connection [SYS-18].
    ///
    /// They are their own members rather than `Read` and `Write` because the
    /// request they submit is its own kind on both engines; what a completion
    /// of one *means* is the same, which is why they share the two transfer
    /// mappers below.
    Receive,
    Send,
}

pub(super) fn completion_file_operation(
    operation: crate::IrSystemOperation,
) -> Option<CompletionFileOperation> {
    match operation.ordinal() {
        OPEN_READ => Some(CompletionFileOperation::OpenRead),
        // `read_at` and `read_next` publish the same `ReadOutcome` from the
        // same three raw scalars, so they share one completion mapper; what
        // differs is the request they submit, not what its completion means.
        READ_ONCE | READ_NEXT => Some(CompletionFileOperation::Read),
        WRITE_ONCE => Some(CompletionFileOperation::Write),
        OPEN_DIRECTORY => Some(CompletionFileOperation::OpenDirectory),
        OPEN_LIST => Some(CompletionFileOperation::OpenDirectorySource),
        LIST_ONCE => Some(CompletionFileOperation::DirectoryNext),
        OPEN_FILE => Some(CompletionFileOperation::OpenFile),
        RECEIVE_NEXT => Some(CompletionFileOperation::Receive),
        SEND_ONCE => Some(CompletionFileOperation::Send),
        // The five remaining TCP rows have no hand-out form: a listen, an
        // accept, a connect and the two explicit closes keep their qualified
        // wrapper, which is the same submit-then-join lowering through the
        // frame's own record. Nothing weaker is substituted and no judgment
        // changes; a site simply holds one such operation at a time.
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
        // A receive publishes the same `ReadOutcome` from the same three raw
        // scalars a read does, and a send the same `Result<u64, IoError>` a
        // write does, so each shares that operation's one mapper [SYS-8].
        CompletionFileOperation::Receive => READ_COMPLETION_MAPPER,
        CompletionFileOperation::Send => WRITE_COMPLETION_MAPPER,
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

/// Raw target-result mappers shared by the qualified wrapper and the
/// handed-out completion site.  There is one lowering now
/// (`research/investigations/io-model/PARK-ON-MISS.md` §8) and therefore one
/// place a raw target value, error and outcome become a typed Whitefoot
/// outcome: whichever of the two submitted the operation, its join hands the
/// three to the same mapper.
const READ_COMPLETION_MAPPER: &str = "wf.sys.read.completion";
const WRITE_COMPLETION_MAPPER: &str = "wf.sys.write.completion";
const OPEN_READ_COMPLETION_MAPPER: &str = "wf.sys.open_read.completion";
const OPEN_DIRECTORY_COMPLETION_MAPPER: &str = "wf.sys.open_directory.completion";
const OPEN_LIST_COMPLETION_MAPPER: &str = "wf.sys.open_directory_source.completion";
const DIRECTORY_NEXT_COMPLETION_MAPPER: &str = "wf.sys.directory_next.completion";
const OPEN_FILE_COMPLETION_MAPPER: &str = "wf.sys.open_file.completion";
const LISTEN_COMPLETION_MAPPER: &str = "wf.sys.tcp_listen.completion";
const ACCEPT_COMPLETION_MAPPER: &str = "wf.sys.tcp_accept.completion";
const CONNECT_COMPLETION_MAPPER: &str = "wf.sys.tcp_connect.completion";
pub(super) const OPEN_EXPECT_REGULAR: u32 = 1;
pub(super) const OPEN_EXPECT_DIRECTORY: u32 = 2;
pub(super) const WINDOWS_DESCRIPTOR_CLASS_READ_FILE: u32 = 1;
pub(super) const WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT: u32 = 2;
pub(super) const WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE: u32 = 3;

/// The declaration of the open submit this target answers.
///
/// Windows carries one more argument, the descriptor class its runtime opens
/// the handle under; every other argument, and the record address that closes
/// the list, is the same on both families.
fn completion_open_submit_declaration(target: SystemTarget) -> String {
    let symbol = target.file_open_at_submit_symbol();
    if target.is_windows() {
        format!("declare void @{symbol}(i32, ptr, i32, i32, i32, i32, i32, ptr)")
    } else {
        format!("declare void @{symbol}(i32, ptr, i32, i32, i32, i32, ptr)")
    }
}

/// The declaration of the join a transferring or closing record is consumed
/// through.
fn completion_join_declaration(target: SystemTarget) -> String {
    format!("declare void @{}(ptr, ptr, ptr)", target.file_join_symbol())
}

/// The declaration of the join an open's record is consumed through.
fn completion_open_join_declaration(target: SystemTarget) -> String {
    format!(
        "declare void @{}(ptr, ptr, ptr, ptr)",
        target.file_open_join_symbol()
    )
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
    // The one close every derived release and every explicit close reaches.
    let mut needs_close = false;

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
    // `read_at`, `read_next` and `receive_next` publish the same `ReadOutcome`
    // through the one read completion mapper [SYS-8], and `write_once` and
    // `send_once` the same `Result<u64, IoError>` through the one write
    // mapper, so a program that uses several of them emits each mapper once.
    // The shapes are the same interned types by construction.
    let mut read_mapper: Option<ReadOutcomeShape> = None;
    let mut write_mapper: Option<(OutcomeShape, IoErrorClass)> = None;
    // The one half-close every [SYS-18] direction release and
    // `close_connection` reaches.
    let mut needs_half_close = false;
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
                definitions.push_str(&emit_open_read(program, implementation, &shape, target)?);
            }
            READ_ONCE => {
                let shape = read_outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.failed_type)?;
                definitions.push_str(&emit_read_at(program, implementation, &shape, target)?);
                read_mapper = Some(shape);
            }
            WRITE_ONCE => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                let refused = write_zero_class(program, &shape)?;
                definitions.push_str(&emit_write_once(program, implementation, &shape, target)?);
                write_mapper = Some((shape, refused));
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
                    implementation,
                    &shape,
                    target,
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
            READ_NEXT => {
                let shape = read_outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.failed_type)?;
                definitions.push_str(&emit_read_next(program, implementation, &shape, target)?);
                read_mapper = Some(shape);
            }
            SOCKET_ADDRESS_V4 => {
                definitions.push_str(&emit_socket_address_v4(implementation));
            }
            SOCKET_ADDRESS_V6 => {
                definitions.push_str(&emit_socket_address_v6(implementation));
            }
            RESERVE_HANDLE => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_reserve_handle(program, implementation, &shape)?);
            }
            CLOSE_READ => {
                needs_close = true;
                definitions.push_str(&emit_close(implementation, SystemResourceType::ReadFile));
            }
            CLOSE_DIRECTORY => {
                needs_close = true;
                definitions.push_str(&emit_close(
                    implementation,
                    SystemResourceType::DirectoryRead,
                ));
            }
            CLOSE_DIRECTORY_SOURCE => {
                needs_close = true;
                definitions.push_str(&emit_close(
                    implementation,
                    SystemResourceType::DirectorySource,
                ));
            }
            TCP_LISTEN => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_socket_endpoint(
                    program,
                    implementation,
                    &shape,
                    SOCKET_LISTEN_SUBMIT,
                    LISTEN_COMPLETION_MAPPER,
                    ORIGIN_SOCKET_LISTEN,
                )?);
            }
            TCP_ACCEPT => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_tcp_accept(program, implementation, &shape)?);
            }
            TCP_CONNECT => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                definitions.push_str(&emit_socket_endpoint(
                    program,
                    implementation,
                    &shape,
                    SOCKET_CONNECT_SUBMIT,
                    CONNECT_COMPLETION_MAPPER,
                    ORIGIN_SOCKET_CONNECT,
                )?);
            }
            RECEIVE_NEXT => {
                let shape = read_outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.failed_type)?;
                definitions.push_str(&emit_receive_next(program, implementation, &shape)?);
                read_mapper = Some(shape);
            }
            SEND_ONCE => {
                let shape = outcome_shape(program, result)?;
                record_io_error(&mut io_error, shape.err_type)?;
                let refused = write_zero_class(program, &shape)?;
                definitions.push_str(&emit_send_once(program, implementation, &shape)?);
                write_mapper = Some((shape, refused));
            }
            CLOSE_CONNECTION => {
                needs_half_close = true;
                let connection = system_call_parameter_type(program, ordinal, 0)?;
                definitions.push_str(&emit_close_connection(program, implementation, connection)?);
            }
            CLOSE_LISTENER => {
                needs_close = true;
                definitions.push_str(&emit_close(implementation, SystemResourceType::TcpListener));
            }
            _ => return Err(BackendFailure::InvalidIr),
        }
        for declaration in operation_declarations(ordinal, target)? {
            declarations.insert(declaration);
        }
    }
    if let Some(shape) = read_mapper.as_ref() {
        definitions.push_str(&emit_read_completion_mapper(shape));
    }
    if let Some((shape, refused)) = write_mapper.as_ref() {
        definitions.push_str(&emit_write_completion_mapper(shape, refused));
    }
    if needs_validator {
        definitions.push_str(&emit_utf8_validator());
    }
    if let Some(error) = io_error {
        definitions.push_str(&emit_io_error_mapper(program, error, target)?);
    }

    if program_releases_with_close(program, qualification)? {
        needs_close = true;
    }
    if needs_close {
        for declaration in close_declarations(target) {
            declarations.insert(declaration);
        }
        definitions.push_str(&emit_close_helper(target));
    }
    if program_releases_with_direction_close(program, qualification)? {
        needs_half_close = true;
    }
    if needs_half_close {
        for declaration in half_close_declarations() {
            declarations.insert(declaration);
        }
        definitions.push_str(&emit_half_close_helper());
    }

    let IrEntry::Command { inputs, .. } = program.entry();
    if target.is_windows() {
        declarations.insert("declare i32 @wf__windows_stdout_descriptor()".to_owned());
        declarations.insert("declare i32 @wf__windows_stderr_descriptor()".to_owned());
        declarations.insert("declare i32 @wf__windows_stdin_descriptor()".to_owned());
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
            return Ok(vec![
                completion_open_submit_declaration(target),
                completion_open_join_declaration(target),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        OPEN_DIRECTORY | OPEN_FILE => {
            return Ok(vec![
                completion_open_submit_declaration(target),
                completion_open_join_declaration(target),
                "declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)".to_owned(),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        READ_ONCE => {
            return Ok(vec![
                format!(
                    "declare void @{}(i32, ptr, i64, i64, ptr)",
                    target.file_pread_submit_symbol()
                ),
                completion_join_declaration(target),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        WRITE_ONCE => {
            return Ok(vec![
                format!(
                    "declare void @{}(i32, ptr, i64, ptr)",
                    target.file_write_submit_symbol()
                ),
                completion_join_declaration(target),
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
                format!("declare void @{DIRECTORY_NEXT_SUBMIT}(i32, ptr, i64, ptr, ptr)"),
                format!("declare void @{FILE_JOIN}(ptr, ptr, ptr)"),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        READ_NEXT => {
            return Ok(vec![
                format!(
                    "declare void @{}(i32, ptr, i64, ptr)",
                    target.file_read_submit_symbol()
                ),
                completion_join_declaration(target),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        // The factory's credit count lives in the floor, linked into every
        // program [SYS-10].
        RESERVE_HANDLE => &["declare i32 @wf__handle_reserve()"],
        CLOSE_READ | CLOSE_DIRECTORY | CLOSE_DIRECTORY_SOURCE | CLOSE_LISTENER => {
            return Ok(close_declarations(target));
        }
        // The four TCP submits that reach a host object, and the two joins
        // they retire through. None is a target column: a socket is one
        // object with one contract on every host this compiler qualifies.
        TCP_LISTEN => {
            return Ok(vec![
                format!("declare void @{SOCKET_LISTEN_SUBMIT}(i64, i64, i32, ptr)"),
                format!("declare void @{FILE_JOIN}(ptr, ptr, ptr)"),
            ]);
        }
        TCP_CONNECT => {
            return Ok(vec![
                format!("declare void @{SOCKET_CONNECT_SUBMIT}(i64, i64, i32, ptr)"),
                format!("declare void @{FILE_JOIN}(ptr, ptr, ptr)"),
            ]);
        }
        TCP_ACCEPT => {
            return Ok(vec![
                format!("declare void @{SOCKET_ACCEPT_SUBMIT}(i32, ptr)"),
                format!("declare void @{SOCKET_ACCEPT_JOIN}(ptr, ptr, ptr, ptr, ptr, ptr)"),
            ]);
        }
        RECEIVE_NEXT => {
            return Ok(vec![
                format!("declare void @{SOCKET_RECEIVE_SUBMIT}(i32, ptr, i64, ptr)"),
                format!("declare void @{FILE_JOIN}(ptr, ptr, ptr)"),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        SEND_ONCE => {
            return Ok(vec![
                format!("declare void @{SOCKET_SEND_SUBMIT}(i32, ptr, i64, ptr)"),
                format!("declare void @{FILE_JOIN}(ptr, ptr, ptr)"),
                "declare void @abort() noreturn".to_owned(),
            ]);
        }
        CLOSE_CONNECTION => {
            return Ok(half_close_declarations());
        }
        _ => &[],
    };
    Ok(fixed.iter().map(|text| (*text).to_owned()).collect())
}

/// The declarations one close needs: the submit its descriptor is handed to
/// and the join that consumes the record it fills.
fn close_declarations(target: SystemTarget) -> Vec<String> {
    vec![
        format!(
            "declare void @{}(i32, ptr)",
            target.file_close_submit_symbol()
        ),
        completion_join_declaration(target),
    ]
}

/// The declarations one half-close needs: the runtime's own half-close submit
/// and the join that consumes the record it fills.
///
/// Neither is a target column: a connection is one object with one contract on
/// every host this compiler qualifies, and the two engines that carry a socket
/// are inside the runtime behind these names.
fn half_close_declarations() -> Vec<String> {
    vec![
        format!("declare void @{SOCKET_SHUTDOWN_SUBMIT}(i32, i32, ptr)"),
        format!("declare void @{FILE_JOIN}(ptr, ptr, ptr)"),
    ]
}

/// Whether any release this program derives is a direction half-close
/// [SYS-18].
fn program_releases_with_direction_close(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
) -> Result<bool, BackendFailure> {
    for nominal in program.nominals() {
        let IrNominalKind::SystemResource(contract) = nominal.kind() else {
            continue;
        };
        if qualification.resource(contract.resource)?.release()
            == ReleaseImplementation::NativeDirectionClose
        {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether any release this program derives is a native close attempt.
///
/// Every type whose [SYS-5] release is a close attempt reaches the one close
/// helper below, so a program that derives one such release declares the close
/// facility once and defines that helper once.
fn program_releases_with_close(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
) -> Result<bool, BackendFailure> {
    for nominal in program.nominals() {
        let IrNominalKind::SystemResource(contract) = nominal.kind() else {
            continue;
        };
        if qualification.resource(contract.resource)?.release()
            == ReleaseImplementation::NativeClose
        {
            return Ok(true);
        }
    }
    Ok(false)
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

/// The exact IR type of one system operation's parameter.
///
/// One caller: `close_connection` takes the whole `TcpConnection`, and a
/// system struct's emitted type is its own nominal rather than the fixed
/// representation an opaque resource has [SYS-18]. Reading the catalog row and
/// resolving it against the program's retained nominals is the same route
/// every other exact type here takes, so no source name or signature reaches
/// it [QUAL-1].
fn system_call_parameter_type(
    program: &IrProgram<'_, '_, '_>,
    ordinal: u8,
    parameter: usize,
) -> Result<IrType, BackendFailure> {
    let declared = crate::SYSTEM_OPERATIONS
        .get(usize::from(ordinal))
        .and_then(|row| row.parameters.get(parameter))
        .ok_or(BackendFailure::InvalidIr)?;
    catalog_ir_type(program, declared.ty)
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
        crate::SystemTypeRef::U16 => IrType::Integer {
            width: 16,
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
        if declared.is_opaque() {
            let expected =
                crate::system_resource_contract(index).ok_or(BackendFailure::InvalidIr)?;
            return Ok(matches!(
                nominal.kind(),
                IrNominalKind::SystemResource(actual) if *actual == expected
            ));
        }
        // A system struct is an ordinary struct nominal whose fields come from
        // the catalog [SYS-18]; nothing about matching it is special.
        if declared.is_struct() {
            let IrNominalKind::Struct { fields } = nominal.kind() else {
                return Ok(false);
            };
            if fields.len() != declared.fields.len() {
                return Ok(false);
            }
            for (field, expected) in fields.iter().zip(declared.fields) {
                if field.ty() != catalog_ir_type(program, expected.ty)? {
                    return Ok(false);
                }
            }
            return Ok(true);
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
    /// The position of the permit an open outcome hands back beside its
    /// error [SYS-10]; `None` for a plain `Result<T, E>`.
    permit_index: Option<usize>,
    /// The position and type of a second field on the succeeding variant.
    ///
    /// Exactly one outcome in the inventory has one: `Accepted(connection,
    /// peer)` reports the address the target gave for the peer beside the
    /// connection it created [SYS-17]. It is resolved from the program's own
    /// IR like everything else here, and `None` everywhere else.
    ok_extra: Option<(usize, String)>,
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
    // A succeeding variant carries the fresh owner the operation created, and
    // `Accepted` carries the peer's address beside it [SYS-17].
    let (ok_field, ok_second) = match ok.fields() {
        [ok_field] => (ok_field, None),
        [ok_field, peer_field] if matches!(peer_field.ty(), IrType::Nominal(_)) => {
            (ok_field, Some(peer_field))
        }
        _ => return Err(BackendFailure::InvalidIr),
    };
    // A plain `Result<T, E>` carries one field on its failed variant; an open
    // outcome [SYS-10] carries the error and, beside it, the permit it hands
    // back. Both are resolved from the program's own IR, never a spelling.
    let (err_field, carries_permit) = match err.fields() {
        [err_field] => (err_field, false),
        [err_field, permit_field] if matches!(permit_field.ty(), IrType::Nominal(_)) => {
            (err_field, true)
        }
        _ => return Err(BackendFailure::InvalidIr),
    };
    let err_index = variant_field_base(variants, err.tag())?;
    let ok_index = variant_field_base(variants, ok.tag())?;
    let ok_extra = match ok_second {
        Some(field) => Some((
            ok_index
                .checked_add(1)
                .ok_or(BackendFailure::CounterOverflow)?,
            llvm_type(program, field.ty())?,
        )),
        None => None,
    };
    Ok(OutcomeShape {
        llvm: llvm_type(program, ty)?,
        ok_tag: ok.tag(),
        ok_index,
        ok_llvm: llvm_type(program, ok_field.ty())?,
        err_tag: err.tag(),
        err_index,
        err_llvm: llvm_type(program, err_field.ty())?,
        err_type: err_field.ty(),
        permit_index: carries_permit.then(|| err_index.checked_add(1)).flatten(),
        ok_extra,
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
#[derive(Clone, Copy)]
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

/// The open submit every qualified open wrapper renders.
///
/// The kind the row promises is the `expected_kind` argument, and deciding it
/// — including the close of a provisional descriptor whose kind does not match
/// — belongs to whoever answers the submit (design §8). The wrapper states the
/// expectation and reads the outcome the join publishes; it holds no status
/// record and performs no close of its own.
fn open_submit_call(
    target: SystemTarget,
    directory: &str,
    path: &str,
    flags: i32,
    expected_kind: u32,
    descriptor_class: u32,
) -> String {
    let class = windows_descriptor_class_argument(target, descriptor_class);
    completion_submit_call(
        target.file_open_at_submit_symbol(),
        &format!(
            "i32 {directory}, ptr {path}, i32 {flags}, i32 0, i32 0, \
             i32 {expected_kind}{class}, ptr {WRAPPER_RECORD}"
        ),
    )
}

/// The retirement every qualified open wrapper renders: the open join, the
/// raw descriptor, error and outcome it publishes, and the operation's own
/// completion mapper over the three.
fn open_retirement(target: SystemTarget, llvm: &str, mapper: &str) -> String {
    completion_retirement(&CompletionRetirement {
        join: target.file_open_join_symbol(),
        record: WRAPPER_RECORD,
        raw_value: WRAPPER_RAW_VALUE,
        raw_error: WRAPPER_RAW_ERROR,
        open_outcome: Some((WRAPPER_RAW_OUTCOME, "%open.outcome")),
        value: "%raw.descriptor",
        error: "%open.error",
        mapper,
        mapper_arguments: "i64 %raw.descriptor, i32 %open.error, i32 %open.outcome",
        result: "%mapped",
        result_type: llvm,
    })
}

/// The retirement a transferring wrapper renders: the join, the raw value and
/// error it publishes, and the operation's own completion mapper over them and
/// the wrapper's own proved endpoints.
///
/// `trailing` is empty for an operation whose mapper reads nothing but the two
/// raw scalars — a listen and a connect, which carry no range.
fn transfer_retirement(join: &str, llvm: &str, mapper: &str, trailing: &str) -> String {
    let arguments = if trailing.is_empty() {
        "i64 %completed.value, i32 %completed.error".to_owned()
    } else {
        format!("i64 %completed.value, i32 %completed.error, {trailing}")
    };
    completion_retirement(&CompletionRetirement {
        join,
        record: WRAPPER_RECORD,
        raw_value: WRAPPER_RAW_VALUE,
        raw_error: WRAPPER_RAW_ERROR,
        open_outcome: None,
        value: "%completed.value",
        error: "%completed.error",
        mapper,
        mapper_arguments: &arguments,
        result: "%outcome",
        result_type: llvm,
    })
}

fn emit_open_read(
    program: &IrProgram<'_, '_, '_>,
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
    let OutcomeShape { llvm, .. } = shape;
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
    let submit = open_submit_call(
        target,
        "%root",
        "%text",
        target.file_open_flags(),
        OPEN_EXPECT_REGULAR,
        WINDOWS_DESCRIPTOR_CLASS_READ_FILE,
    );
    let retirement = open_retirement(target, llvm, OPEN_READ_COMPLETION_MAPPER);
    let wrapper = format!(
        "define private {llvm} @{symbol}({directory} %root, {path} %path) alwaysinline {{\n\
         entry:\n\
         {reservation}  \
         %text = extractvalue {path} %path, 0\n\
         {submit}{retirement}  \
         ret {llvm} %mapped\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(true),
    );
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
    let permit_index = shape.permit_index.ok_or(BackendFailure::InvalidIr)?;
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
         %open.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n  \
         %open.tag = insertvalue {llvm} %open.base, i32 {err_tag}, 0\n  \
         %open.result = insertvalue {llvm} %open.tag, {err_llvm} %open.error, {err_index}\n  \
         ret {llvm} %open.result\n\
         status.failure:\n  \
         %status.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 \
         {ORIGIN_DESCRIPTOR_STATUS})\n  \
         %status.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n  \
         %status.tag = insertvalue {llvm} %status.base, i32 {err_tag}, 0\n  \
         %status.result = insertvalue {llvm} %status.tag, {err_llvm} %status.error, \
         {err_index}\n  \
         ret {llvm} %status.result\n\
         kind.directory.return:\n\
         {directory_value}  \
         %kind.directory.result.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n  \
         %kind.directory.result.tag = insertvalue {llvm} %kind.directory.result.base, i32 {err_tag}, 0\n  \
         %kind.directory.result = insertvalue {llvm} %kind.directory.result.tag, {err_llvm} \
         {directory_error}, {err_index}\n  \
         ret {llvm} %kind.directory.result\n\
         kind.other.return:\n\
         {other_value}  \
         %kind.other.result.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n  \
         %kind.other.result.tag = insertvalue {llvm} %kind.other.result.base, i32 {err_tag}, 0\n  \
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
    let ReadOutcomeShape { llvm, .. } = shape;
    // The two call-site SYS-8 goals authorize this half-open range, so `sub
    // nuw` needs no check: one obligation proves `start <= end` and the other
    // `end <= len(buffer)`.
    //
    // One lowering and no branch before it (design §8). A zero-length range
    // still reports `next = start` and a file offset the target ABI cannot
    // express still reports the host's own refusal, but both are now the
    // runtime's answers over the submitted record rather than a second arm
    // here: an empty transfer is completed with no external action, and an
    // offset above the signed maximum is published as `EINVAL`. The mapper
    // turns either into the same outcome the wrapper used to build itself.
    let prepared =
        completion_transfer_target(&buffer, "%destination", "%start", "%base", "%target");
    let submit = completion_submit_call(
        target.file_pread_submit_symbol(),
        &format!("{file} %file, ptr %target, i64 %extent, i64 %file_offset, ptr {WRAPPER_RECORD}"),
    );
    let retirement = transfer_retirement(
        target.file_join_symbol(),
        llvm,
        READ_COMPLETION_MAPPER,
        "i64 %start, i64 %extent",
    );
    // The one read completion mapper is emitted once by the caller, because
    // `read_at` and `read_next` share it [SYS-8].
    Ok(format!(
        "define private {llvm} @{symbol}({file} %file, {buffer} %destination, i64 %file_offset, \
         i64 %start, i64 %end) alwaysinline {{\n\
         entry:\n\
         {reservation}  \
         %extent = sub nuw i64 %end, %start\n\
         {prepared}{submit}{retirement}  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
    ))
}

/// Emits `read_next`: one unpositioned transfer attempt at the stream's own
/// position [SYS-15].
///
/// It is `read_at`'s wrapper with the offset removed and the runtime's
/// unpositioned request kind in place of the positioned one, and it publishes
/// the same `ReadOutcome` through the same mapper: an empty range answers
/// `ReadBytes(start)` with no host transfer, a progress-producing attempt
/// answers `ReadBytes(next)`, a host end answers `ReadEnd`, and a refusal
/// answers `ReadFailed`. The stream's position is the descriptor's own, so
/// nothing here carries or advances a cursor of its own.
fn emit_read_next(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &ReadOutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let stream = representation(SystemResourceType::InputStream);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    let ReadOutcomeShape { llvm, .. } = shape;
    let prepared =
        completion_transfer_target(&buffer, "%destination", "%start", "%base", "%target");
    let submit = completion_submit_call(
        target.file_read_submit_symbol(),
        &format!("{stream} %input, ptr %target, i64 %extent, ptr {WRAPPER_RECORD}"),
    );
    let retirement = transfer_retirement(
        target.file_join_symbol(),
        llvm,
        READ_COMPLETION_MAPPER,
        "i64 %start, i64 %extent",
    );
    Ok(format!(
        "define private {llvm} @{symbol}({stream} %input, {buffer} %destination, \
         i64 %start, i64 %end) alwaysinline {{\n\
         entry:\n\
         {reservation}  \
         %extent = sub nuw i64 %end, %start\n\
         {prepared}{submit}{retirement}  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
    ))
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
    let output = representation(SystemResourceType::OutputStream);
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
    let OutcomeShape { llvm, .. } = shape;
    // A host zero-length write is `Err(WriteZero())`, which no native error
    // code produced: [SYS-7] leaves both detail fields zero when the target
    // supplies no value for them.
    // At most one host output attempt [SYS-12]. A zero-length range reports
    // `next = start` and issues no host transfer — the runtime completes such
    // a record with no external action, and the mapper builds that outcome
    // (design §8). Otherwise `Ok(next)` means exactly that the host accepted
    // `[start, next)`, promising neither line atomicity nor durability. A
    // closed destination arrives as the recoverable `BrokenPipe` class because
    // the bootstrap installed the ignored write-to-closed-pipe disposition
    // once, before entry [QUAL-3]; this path performs no per-call
    // signal-disposition operation.
    //
    // The mapper itself is emitted once by the caller, because `write_once`
    // and `send_once` share it [SYS-8].
    let prepared = completion_transfer_target(&buffer, "%source", "%start", "%base", "%target");
    let submit = completion_submit_call(
        target.file_write_submit_symbol(),
        &format!("{output} %output, ptr %target, i64 %extent, ptr {WRAPPER_RECORD}"),
    );
    let retirement = transfer_retirement(
        target.file_join_symbol(),
        llvm,
        WRITE_COMPLETION_MAPPER,
        "i64 %start, i64 %extent",
    );
    Ok(format!(
        "define private {llvm} @{symbol}({output} %output, {buffer} %source, i64 %start, \
         i64 %end) alwaysinline {{\n\
         entry:\n\
         {reservation}  \
         %extent = sub nuw i64 %end, %start\n\
         {prepared}{submit}{retirement}  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
    ))
}

/// The [SYS-7] class one host write of zero bytes reports.
///
/// It is resolved from the program's own `IoError` and no native error code
/// produced it: [SYS-7] leaves both detail fields zero when the target
/// supplies no value for them.
fn write_zero_class(
    program: &IrProgram<'_, '_, '_>,
    shape: &OutcomeShape,
) -> Result<IoErrorClass, BackendFailure> {
    io_error_classes(program, shape.err_type)?
        .into_iter()
        .find(|class| class.spelling == "WriteZero")
        .ok_or(BackendFailure::InvalidIr)
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
    prefix: &str,
) -> Result<(String, String), BackendFailure> {
    let classes = io_error_classes(program, err_type)?;
    let class = classes
        .iter()
        .find(|class| class.spelling == "InvalidPath")
        .ok_or(BackendFailure::InvalidIr)?;
    Ok(io_error_value(
        err_llvm,
        class,
        prefix,
        "0",
        &ORIGIN_NONE.to_string(),
    ))
}

/// The same refused-name outcome, built at a completion hand-out site and
/// stored in that operation's result slot.
///
/// An open by component name is the one completion shape that can reach its
/// outcome without submitting anything: a name that is empty, over the target
/// family's limit, or carrying a separator never becomes a host call. The
/// sequential wrapper answers that from its own `invalid` block, and this
/// builds the identical value from the same two functions rather than calling
/// the wrapper for it, which is what lets a hand-out answer an invalid name
/// without a second lowering
/// (`research/investigations/io-model/PARK-ON-MISS.md` §8). `prefix` makes the
/// names unique per site, because unlike the wrapper's block this one is
/// emitted into a function that may hold several such opens.
pub(super) fn completion_invalid_component_outcome(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
    prefix: &str,
    destination: &str,
) -> Result<String, BackendFailure> {
    let shape = outcome_shape(program, ty)?;
    let permit_index = shape.permit_index.ok_or(BackendFailure::InvalidIr)?;
    let OutcomeShape {
        llvm,
        err_tag,
        err_index,
        err_llvm,
        err_type,
        ..
    } = &shape;
    let (invalid_value, invalid_error) =
        invalid_component(program, err_llvm, *err_type, &format!("{prefix}.path"))?;
    let mut text = invalid_value;
    text.push_str(&format!(
        "  %{prefix}.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n"
    ));
    text.push_str(&format!(
        "  %{prefix}.tag = insertvalue {llvm} %{prefix}.base, i32 {err_tag}, 0\n"
    ));
    text.push_str(&format!(
        "  %{prefix}.outcome = insertvalue {llvm} %{prefix}.tag, {err_llvm} {invalid_error}, {err_index}\n"
    ));
    text.push_str(&format!(
        "  store {llvm} %{prefix}.outcome, ptr {destination}\n"
    ));
    Ok(text)
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
    let (flags, mapper_symbol, expected_kind, descriptor_class) = match opened {
        SystemResourceType::DirectoryRead => (
            target.component_directory_open_flags(),
            OPEN_DIRECTORY_COMPLETION_MAPPER,
            OPEN_EXPECT_DIRECTORY,
            WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT,
        ),
        SystemResourceType::ReadFile => (
            target.component_file_open_flags(),
            OPEN_FILE_COMPLETION_MAPPER,
            OPEN_EXPECT_REGULAR,
            WINDOWS_DESCRIPTOR_CLASS_READ_FILE,
        ),
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
        err_tag,
        err_index,
        err_llvm,
        err_type,
        ..
    } = shape;
    let permit_index = shape.permit_index.ok_or(BackendFailure::InvalidIr)?;
    let terminator_bytes = if target.is_windows() { 2 } else { 1 };
    let slot = target.component_limit() + terminator_bytes;
    let component_align = if target.is_windows() { 2 } else { 1 };
    // One shared buffer per wrapper, and deliberately not the per-outstanding-
    // operation storage the handed-out completion sites use. This wrapper
    // submits its open and joins it before returning, so exactly one operation
    // of this site is ever outstanding inside it and the staged name is read
    // only while that one operation is in flight. A site that hands out
    // several at once indexes its own copy the way
    // `FunctionEmitter::completion_entry_slot` does.
    let frame_slots = [(
        "%component",
        TargetFrameSlot::aligned(TargetStorageType::bytes(slot), component_align),
    )];
    let prologue = render_named_target_frame(program, qualification, target_layout, &frame_slots)?;
    let component = component_validation(&buffer, target);
    let (invalid_value, invalid_error) =
        invalid_component(program, err_llvm, *err_type, "invalid")?;
    let terminator = if target.is_windows() {
        "store i16 0, ptr %terminator, align 1"
    } else {
        "store i8 0, ptr %terminator, align 1"
    };
    // The descriptor-kind check and the close of a provisional descriptor that
    // fails it are the runtime's, decided from the `expected_kind` this submit
    // carries; the wrapper holds no status record and never closes anything
    // (design §8).
    let submit = open_submit_call(
        target,
        "%root",
        "%component",
        flags,
        expected_kind,
        descriptor_class,
    );
    let retirement = open_retirement(target, llvm, mapper_symbol);
    Ok(format!(
        "define private {llvm} @{symbol}({directory} %root, {buffer} %name, i64 %start, \
         i64 %end) alwaysinline {{\n\
         entry:\n\
         {prologue}\
         {reservation}  \
         %extent = sub nuw i64 %end, %start\n  \
         br label %measure\n\
         {component}\
         open:\n  \
         call void @llvm.memcpy.p0.p0.i64(ptr %component, ptr %text, i64 %extent, \
         i1 false)\n  \
         %terminator = getelementptr inbounds i8, ptr %component, i64 %extent\n  \
         {terminator}\n\
         {submit}{retirement}  \
         ret {llvm} %mapped\n\
         invalid:\n\
         {invalid_value}  \
         %rejected.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n  \
         %rejected.tag = insertvalue {llvm} %rejected.base, i32 {err_tag}, 0\n  \
         %rejected.outcome = insertvalue {llvm} %rejected.tag, {err_llvm} \
         {invalid_error}, {err_index}\n  \
         ret {llvm} %rejected.outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(true),
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
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    target: SystemTarget,
) -> Result<String, BackendFailure> {
    let directory = representation(SystemResourceType::DirectoryRead);
    let list = representation(SystemResourceType::DirectorySource);
    if shape.ok_llvm != list {
        return Err(BackendFailure::InvalidIr);
    }
    let OutcomeShape { llvm, .. } = shape;
    let mapper = emit_open_completion_mapper(
        program,
        shape,
        OPEN_LIST_COMPLETION_MAPPER,
        SystemResourceType::DirectorySource,
    )?;
    let submit = open_submit_call(
        target,
        "%directory",
        WORKING_DIRECTORY,
        target.directory_open_flags(),
        OPEN_EXPECT_DIRECTORY,
        WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE,
    );
    let retirement = open_retirement(target, llvm, OPEN_LIST_COMPLETION_MAPPER);
    let wrapper = format!(
        "define private {llvm} @{symbol}({directory} %directory) alwaysinline {{\n\
         entry:\n\
         {reservation}\
         {submit}{retirement}  \
         ret {llvm} %mapped\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(true),
    );
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
/// This is one text, emitted from one place, into the one completion mapper
/// every qualified target reaches. The only target-selected
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
/// One lowering: the batch is submitted with the cursor cell the target
/// family needs, joined, and handed to the operation's own completion mapper,
/// which carries the portable-record normalization
/// ([`emit_directory_record_normalizer`]'s one text). The wrapper therefore
/// holds no normalizer of its own and no second arm for an empty range: such
/// a record is completed by the runtime with no external action and the
/// mapper answers `ListBytes(next: start, entries: 0)` for it (design §8).
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
    let ListOutcomeShape { llvm, .. } = shape;
    let prologue = render_named_target_frame(
        program,
        qualification,
        target_layout,
        &[(
            "%position",
            TargetFrameSlot::natural(TargetStorageType::integer(64)),
        )],
    )?;
    let mapper = emit_directory_next_completion_mapper(program, shape, target)?;
    let prepared =
        completion_transfer_target(&buffer, "%destination", "%start", "%base", "%window");
    // The enumeration facility has no target column of its own, so its submit
    // and the join that consumes the record it filled are the runtime's on
    // every target, exactly as its direct entry was.
    let submit = completion_submit_call(
        DIRECTORY_NEXT_SUBMIT,
        &format!("{list} %list, ptr %window, i64 %extent, ptr %position, ptr {WRAPPER_RECORD}"),
    );
    let retirement = transfer_retirement(
        FILE_JOIN,
        llvm,
        DIRECTORY_NEXT_COMPLETION_MAPPER,
        &format!("{buffer} %destination, i64 %start, i64 %extent"),
    );
    let wrapper = format!(
        "define private {llvm} @{symbol}({list} %list, {buffer} %destination, i64 %start, \
         i64 %end) alwaysinline {{\n\
         entry:\n\
         {prologue}\
         {reservation}  \
         %extent = sub nuw i64 %end, %start\n  \
         store i64 0, ptr %position, align 8\n\
         {prepared}{submit}{retirement}  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
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
        bytes_tag,
        bytes_index,
        end_tag,
        failed_tag,
        failed_index,
        failed_llvm,
        ..
    } = shape;
    let entries_index = bytes_index + 1;
    let normalizer = emit_directory_record_normalizer(shape, target, enumeration);
    // The empty-range arm is the same one the sequential wrapper answers from,
    // and it is here because this mapper is now reached with an empty range.
    // The handed-out lowering used to hold that range back and run the wrapper
    // instead; with one lowering it submits, the runtime completes the record
    // with no external action, and the outcome has to be the wrapper's own
    // `ListBytes(next: start, entries: 0)` rather than the exhaustion a zero
    // count means for a range that was not empty (design section 8).
    Ok(format!(
        "define private {llvm} @{DIRECTORY_NEXT_COMPLETION_MAPPER}(i64 %filled, i32 %error, \
         {buffer} %destination, i64 %start, i64 %extent) alwaysinline {{\n\
         entry:\n  \
         %empty.range = icmp eq i64 %extent, 0\n  \
         br i1 %empty.range, label %vacant, label %nonempty\n\
         vacant:\n  \
         %empty.tag = insertvalue {llvm} zeroinitializer, i32 {bytes_tag}, 0\n  \
         %empty.endpoint = insertvalue {llvm} %empty.tag, i64 %start, {bytes_index}\n  \
         %empty.outcome = insertvalue {llvm} %empty.endpoint, i64 0, {entries_index}\n  \
         ret {llvm} %empty.outcome\n\
         nonempty:\n  \
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

/// The [QUAL-1] representation of one `SocketAddress` [SYS-16].
///
/// Sixteen address bytes in two 64-bit words, then the port in the low sixteen
/// bits of a 32-bit word whose bit 16 selects the family. Byte `i` of the
/// address occupies bits `8 * (i % 8)` of word `i / 8`, so the same rule reads
/// the value on either endianness and an IPv4 address simply leaves bytes 4
/// through 15 zero. Nothing in source observes this layout [SYS-2]; it is the
/// target column of the row, and the runtime routes of slice 2 read it by the
/// same rule.
const SOCKET_ADDRESS_FAMILY_V6: u32 = 1 << 16;

fn emit_socket_address_v4(implementation: ApprovedImplementation) -> String {
    let value = ResourceRepresentation::InternetAddress.llvm();
    // The four bytes are the address in its conventional order, so byte 0 is
    // `a` and byte 3 is `d`; bytes 4 through 15 stay zero and the family bit
    // stays clear.
    let mut body = String::new();
    for (index, name) in ["%a", "%b", "%c", "%d"].into_iter().enumerate() {
        let shift = 8 * index;
        let _ = writeln!(
            body,
            "  {name}.w = zext i8 {name} to i64\n  \
             {name}.s = shl nuw i64 {name}.w, {shift}"
        );
    }
    format!(
        "define private {value} @{symbol}(i8 %a, i8 %b, i8 %c, i8 %d, i16 %port) \
         alwaysinline {{\n\
         entry:\n\
         {body}  \
         %ab = or i64 %a.s, %b.s\n  \
         %abc = or i64 %ab, %c.s\n  \
         %word0 = or i64 %abc, %d.s\n  \
         %port.wide = zext i16 %port to i32\n  \
         %address.0 = insertvalue {value} zeroinitializer, i64 %word0, 0\n  \
         %address.1 = insertvalue {value} %address.0, i64 0, 1\n  \
         %address = insertvalue {value} %address.1, i32 %port.wide, 2\n  \
         ret {value} %address\n\
         }}\n\n",
        symbol = implementation.symbol(),
    )
}

fn emit_socket_address_v6(implementation: ApprovedImplementation) -> String {
    let value = ResourceRepresentation::InternetAddress.llvm();
    // Group `i` occupies address bytes `2i` and `2i + 1`, high byte first,
    // which is the conventional order of an IPv6 group.
    let groups = ["%a", "%b", "%c", "%d", "%e", "%f", "%g", "%h"];
    let mut body = String::new();
    for (index, name) in groups.iter().enumerate() {
        let high_shift = 8 * ((2 * index) % 8);
        let low_shift = 8 * ((2 * index + 1) % 8);
        let _ = writeln!(
            body,
            "  {name}.wide = zext i16 {name} to i64\n  \
             {name}.low = and i64 {name}.wide, 255\n  \
             {name}.high = lshr i64 {name}.wide, 8\n  \
             {name}.hs = shl nuw i64 {name}.high, {high_shift}\n  \
             {name}.ls = shl nuw i64 {name}.low, {low_shift}\n  \
             {name}.packed = or i64 {name}.hs, {name}.ls"
        );
    }
    format!(
        "define private {value} @{symbol}(i16 %a, i16 %b, i16 %c, i16 %d, i16 %e, i16 %f, \
         i16 %g, i16 %h, i16 %port) alwaysinline {{\n\
         entry:\n\
         {body}  \
         %word0.ab = or i64 %a.packed, %b.packed\n  \
         %word0.abc = or i64 %word0.ab, %c.packed\n  \
         %word0 = or i64 %word0.abc, %d.packed\n  \
         %word1.ef = or i64 %e.packed, %f.packed\n  \
         %word1.efg = or i64 %word1.ef, %g.packed\n  \
         %word1 = or i64 %word1.efg, %h.packed\n  \
         %port.wide = zext i16 %port to i32\n  \
         %tagged = or i32 %port.wide, {SOCKET_ADDRESS_FAMILY_V6}\n  \
         %address.0 = insertvalue {value} zeroinitializer, i64 %word0, 0\n  \
         %address.1 = insertvalue {value} %address.0, i64 %word1, 1\n  \
         %address = insertvalue {value} %address.1, i32 %tagged, 2\n  \
         ret {value} %address\n\
         }}\n\n",
        symbol = implementation.symbol(),
    )
}

/// The three scalars one emitted `SocketAddress` value is, extracted so a
/// submit can carry them.
///
/// The runtime reads an address as exactly these three and never as a pointer
/// into an emitted value's storage (`completion/bridge.h`), so the layout is
/// stated once here and once in `wf_socket_address`, and neither side holds a
/// pointer into the other's.
fn socket_address_scalars(value: &str, prefix: &str) -> String {
    let address = ResourceRepresentation::InternetAddress.llvm();
    format!(
        "  %{prefix}.low = extractvalue {address} {value}, 0\n  \
         %{prefix}.high = extractvalue {address} {value}, 1\n  \
         %{prefix}.tag = extractvalue {address} {value}, 2\n"
    )
}

/// Emits `tcp_listen` or `tcp_connect` [SYS-17].
///
/// Both take one address by shared loan and one permit the target ABI erases,
/// both create their own socket inside the runtime, and both publish the same
/// two-variant outcome: the fresh owner, or the host's error beside the very
/// permit the operation took. So they are one wrapper with two submit entries
/// and two outcomes, exactly as the four opens are one `emit_open_by_name`.
fn emit_socket_endpoint(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
    submit: &str,
    mapper: &str,
    origin: u8,
) -> Result<String, BackendFailure> {
    let address = ResourceRepresentation::InternetAddress.llvm();
    let OutcomeShape { llvm, .. } = shape;
    let mapper_text = emit_socket_endpoint_completion_mapper(program, shape, mapper, origin)?;
    let scalars = socket_address_scalars("%address", "address");
    let submit_call = completion_submit_call(
        submit,
        &format!("i64 %address.low, i64 %address.high, i32 %address.tag, ptr {WRAPPER_RECORD}"),
    );
    let retirement = transfer_retirement(FILE_JOIN, llvm, mapper, "");
    let wrapper = format!(
        "define private {llvm} @{symbol}({address} %address) alwaysinline {{\n\
         entry:\n\
         {reservation}\
         {scalars}{submit_call}{retirement}  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
    );
    Ok(format!("{mapper_text}{wrapper}"))
}

/// The mapper `tcp_listen` and `tcp_connect` publish through.
///
/// A descriptor the runtime hands back is the fresh owner; a negative value is
/// the host's own refusal, and the permit comes back inside the failed variant
/// because no handle was taken [SYS-10]. A connection is one descriptor and
/// two owners, so `Connected` carries the pair built out of that one
/// descriptor twice [SYS-18]; a listener is one owner and the pair is absent.
fn emit_socket_endpoint_completion_mapper(
    program: &IrProgram<'_, '_, '_>,
    shape: &OutcomeShape,
    symbol: &str,
    origin: u8,
) -> Result<String, BackendFailure> {
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        ok_llvm,
        err_tag,
        err_index,
        err_llvm,
        err_type,
        ..
    } = shape;
    let permit_index = shape.permit_index.ok_or(BackendFailure::InvalidIr)?;
    let _ = io_error_classes(program, *err_type)?;
    let created = socket_owner_value(ok_llvm, "%raw.descriptor", "fresh")?;
    Ok(format!(
        "define private {llvm} @{symbol}(i64 %raw.descriptor, i32 %error) alwaysinline {{\n\
         entry:\n  \
         %live = icmp sge i64 %raw.descriptor, 0\n  \
         br i1 %live, label %created, label %refused\n\
         created:\n\
         {created}  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {ok_llvm} %fresh, {ok_index}\n  \
         ret {llvm} %ok\n\
         refused:\n  \
         %refused.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 {origin})\n  \
         %refused.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n  \
         %refused.tag = insertvalue {llvm} %refused.base, i32 {err_tag}, 0\n  \
         %refused.result = insertvalue {llvm} %refused.tag, {err_llvm} %refused.error, \
         {err_index}\n  \
         ret {llvm} %refused.result\n\
         }}\n\n"
    ))
}

/// The instructions building one fresh socket owner out of the descriptor the
/// runtime published.
///
/// A listener is that descriptor. A connection is the two-field system struct
/// of [SYS-18], whose two directions name one target object, so both fields
/// are that same descriptor and the runtime's own two-count is what decides
/// which release closes it.
fn socket_owner_value(owner: &str, raw: &str, name: &str) -> Result<String, BackendFailure> {
    let descriptor = representation(SystemResourceType::TcpListener);
    if owner == descriptor {
        return Ok(format!("  %{name} = trunc i64 {raw} to {descriptor}\n"));
    }
    Ok(format!(
        "  %{name}.descriptor = trunc i64 {raw} to {descriptor}\n  \
         %{name}.receive = insertvalue {owner} zeroinitializer, {descriptor} \
         %{name}.descriptor, 0\n  \
         %{name} = insertvalue {owner} %{name}.receive, {descriptor} \
         %{name}.descriptor, 1\n"
    ))
}

/// Emits `tcp_accept` [SYS-17].
///
/// It is the endpoint wrapper with one thing added: the target's own answer
/// about the peer, which the accept join publishes as the three scalars a
/// `SocketAddress` value is, and which the mapper assembles into that value.
fn emit_tcp_accept(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
) -> Result<String, BackendFailure> {
    let listener = representation(SystemResourceType::TcpListener);
    let OutcomeShape { llvm, .. } = shape;
    let mapper = emit_tcp_accept_completion_mapper(program, shape)?;
    let submit = completion_submit_call(
        SOCKET_ACCEPT_SUBMIT,
        &format!("{listener} %listener, ptr {WRAPPER_RECORD}"),
    );
    let wrapper = format!(
        "define private {llvm} @{symbol}({listener} %listener) alwaysinline {{\n\
         entry:\n\
         {reservation}  \
         %peer.low = alloca i64, align 8\n  \
         %peer.high = alloca i64, align 8\n  \
         %peer.tag = alloca i32, align 4\n\
         {submit}  \
         call void @{SOCKET_ACCEPT_JOIN}(ptr {WRAPPER_RECORD}, ptr {WRAPPER_RAW_VALUE}, \
         ptr {WRAPPER_RAW_ERROR}, ptr %peer.low, ptr %peer.high, ptr %peer.tag)\n  \
         %completed.value = load i64, ptr {WRAPPER_RAW_VALUE}\n  \
         %completed.error = load i32, ptr {WRAPPER_RAW_ERROR}\n  \
         %peer.low.value = load i64, ptr %peer.low\n  \
         %peer.high.value = load i64, ptr %peer.high\n  \
         %peer.tag.value = load i32, ptr %peer.tag\n  \
         %outcome = call {llvm} @{ACCEPT_COMPLETION_MAPPER}(i64 %completed.value, \
         i32 %completed.error, i64 %peer.low.value, i64 %peer.high.value, \
         i32 %peer.tag.value)\n  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
    );
    Ok(format!("{mapper}{wrapper}"))
}

fn emit_tcp_accept_completion_mapper(
    program: &IrProgram<'_, '_, '_>,
    shape: &OutcomeShape,
) -> Result<String, BackendFailure> {
    let address = ResourceRepresentation::InternetAddress.llvm();
    let OutcomeShape {
        llvm,
        ok_tag,
        ok_index,
        ok_llvm,
        err_tag,
        err_index,
        err_llvm,
        err_type,
        ..
    } = shape;
    let permit_index = shape.permit_index.ok_or(BackendFailure::InvalidIr)?;
    let (peer_index, peer_llvm) = shape
        .ok_extra
        .as_ref()
        .ok_or(BackendFailure::InvalidIr)?
        .clone();
    if peer_llvm != address {
        return Err(BackendFailure::InvalidIr);
    }
    let _ = io_error_classes(program, *err_type)?;
    let taken = socket_owner_value(ok_llvm, "%raw.descriptor", "fresh")?;
    Ok(format!(
        "define private {llvm} @{ACCEPT_COMPLETION_MAPPER}(i64 %raw.descriptor, i32 %error, \
         i64 %peer.low, i64 %peer.high, i32 %peer.tag) alwaysinline {{\n\
         entry:\n  \
         %live = icmp sge i64 %raw.descriptor, 0\n  \
         br i1 %live, label %taken, label %refused\n\
         taken:\n\
         {taken}  \
         %peer.0 = insertvalue {address} zeroinitializer, i64 %peer.low, 0\n  \
         %peer.1 = insertvalue {address} %peer.0, i64 %peer.high, 1\n  \
         %peer = insertvalue {address} %peer.1, i32 %peer.tag, 2\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok.connection = insertvalue {llvm} %ok.tag, {ok_llvm} %fresh, {ok_index}\n  \
         %ok = insertvalue {llvm} %ok.connection, {address} %peer, {peer_index}\n  \
         ret {llvm} %ok\n\
         refused:\n  \
         %refused.error = call {err_llvm} @{IO_ERROR_MAPPER}(i32 %error, i8 \
         {ORIGIN_SOCKET_ACCEPT})\n  \
         %refused.base = insertvalue {llvm} zeroinitializer, i1 true, {permit_index}\n  \
         %refused.tag = insertvalue {llvm} %refused.base, i32 {err_tag}, 0\n  \
         %refused.result = insertvalue {llvm} %refused.tag, {err_llvm} %refused.error, \
         {err_index}\n  \
         ret {llvm} %refused.result\n\
         }}\n\n"
    ))
}

/// Emits `receive_next`: one transfer attempt on one direction of one
/// connection [SYS-18].
///
/// It is `read_next`'s wrapper with the connection's receiving direction in
/// place of the stream and the runtime's own receive request kind in place of
/// the unpositioned read, and it publishes the same `ReadOutcome` through the
/// same mapper, because a completion of one means exactly what a completion of
/// the other does [SYS-8].
fn emit_receive_next(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &ReadOutcomeShape,
) -> Result<String, BackendFailure> {
    let receive = representation(SystemResourceType::TcpReceive);
    let buffer = llvm_type(
        program,
        IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        },
    )?;
    let ReadOutcomeShape { llvm, .. } = shape;
    let prepared =
        completion_transfer_target(&buffer, "%destination", "%start", "%base", "%target");
    let submit = completion_submit_call(
        SOCKET_RECEIVE_SUBMIT,
        &format!("{receive} %receive, ptr %target, i64 %extent, ptr {WRAPPER_RECORD}"),
    );
    let retirement = transfer_retirement(
        FILE_JOIN,
        llvm,
        READ_COMPLETION_MAPPER,
        "i64 %start, i64 %extent",
    );
    Ok(format!(
        "define private {llvm} @{symbol}({receive} %receive, {buffer} %destination, \
         i64 %start, i64 %end) alwaysinline {{\n\
         entry:\n\
         {reservation}  \
         %extent = sub nuw i64 %end, %start\n\
         {prepared}{submit}{retirement}  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
    ))
}

/// Emits `send_once`: one attempt on the sending direction [SYS-18].
///
/// It is `write_once`'s wrapper with the connection's sending direction in
/// place of the output stream and the runtime's own send request kind in place
/// of the write, publishing the same `Result<u64, IoError>` through the same
/// mapper: `Ok(next)` means the local facility accepted `[start, next)`, a
/// host write of zero is `WriteZero`, and a peer that has gone arrives as
/// `BrokenPipe` through the same signal normalization [SYS-8].
fn emit_send_once(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
) -> Result<String, BackendFailure> {
    let send = representation(SystemResourceType::TcpSend);
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
    let OutcomeShape { llvm, .. } = shape;
    let prepared = completion_transfer_target(&buffer, "%source", "%start", "%base", "%target");
    let submit = completion_submit_call(
        SOCKET_SEND_SUBMIT,
        &format!("{send} %send, ptr %target, i64 %extent, ptr {WRAPPER_RECORD}"),
    );
    let retirement = transfer_retirement(
        FILE_JOIN,
        llvm,
        WRITE_COMPLETION_MAPPER,
        "i64 %start, i64 %extent",
    );
    Ok(format!(
        "define private {llvm} @{symbol}({send} %send, {buffer} %source, i64 %start, \
         i64 %end) alwaysinline {{\n\
         entry:\n\
         {reservation}  \
         %extent = sub nuw i64 %end, %start\n\
         {prepared}{submit}{retirement}  \
         ret {llvm} %outcome\n\
         }}\n\n",
        symbol = implementation.symbol(),
        reservation = completion_wrapper_reservation(false),
    ))
}

/// The private symbol of the one half-close every [SYS-18] direction release
/// and `close_connection` reaches.
const HALF_CLOSE_HELPER: &str = "wf.sys.socket.half_close";

/// The [SYS-18] direction one half-close releases, in the runtime's own
/// spelling (`completion/contract.h`, `wf_socket_direction`).
const DIRECTION_RECEIVE: u32 = 0;
const DIRECTION_SEND: u32 = 1;

/// Emits that one half-close.
///
/// It is the close helper with a direction: the record is reserved in this
/// helper's own entry block, the descriptor and the direction are submitted,
/// and the terminal completion is joined here — exactly one attempt, its
/// diagnostic discarded, never retried [SYS-5]. The runtime keeps the pair's
/// own two-count and releases the target's object on the second of the two
/// releases, so nothing here decides which release closes.
fn emit_half_close_helper() -> String {
    let descriptor = representation(SystemResourceType::TcpReceive);
    let submit = completion_submit_call(
        SOCKET_SHUTDOWN_SUBMIT,
        &format!("{descriptor} %descriptor, i32 %direction, ptr {WRAPPER_RECORD}"),
    );
    format!(
        "define private void @{HALF_CLOSE_HELPER}({descriptor} %descriptor, i32 %direction) \
         alwaysinline {{\n\
         entry:\n\
         {reservation}\
         {submit}  \
         call void @{FILE_JOIN}(ptr {WRAPPER_RECORD}, ptr {WRAPPER_RAW_VALUE}, \
         ptr {WRAPPER_RAW_ERROR})\n  \
         ret void\n\
         }}\n\n",
        reservation = completion_wrapper_reservation(false),
    )
}

/// Emits `close_connection` [SYS-18].
///
/// It consumes the whole pair and performs the same two native attempts
/// derived release of the two directions would perform, with the same
/// discarded diagnostics: one half-close per direction, in field order. The
/// second of them is the one the runtime's two-count turns into the close of
/// the target's object, so the credit the pair held comes back as the one
/// fresh permit this returns, which is the erased bit.
fn emit_close_connection(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    connection: IrType,
) -> Result<String, BackendFailure> {
    let permit = representation(SystemResourceType::HandlePermit);
    let descriptor = representation(SystemResourceType::TcpReceive);
    let owner = llvm_type(program, connection)?;
    Ok(format!(
        "define private {permit} @{symbol}({owner} %connection) alwaysinline {{\n\
         entry:\n  \
         %receive = extractvalue {owner} %connection, 0\n  \
         %send = extractvalue {owner} %connection, 1\n  \
         call void @{HALF_CLOSE_HELPER}({descriptor} %receive, i32 {DIRECTION_RECEIVE})\n  \
         call void @{HALF_CLOSE_HELPER}({descriptor} %send, i32 {DIRECTION_SEND})\n  \
         ret {permit} true\n\
         }}\n\n",
        symbol = implementation.symbol(),
    ))
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

/// The permit is one credit of the factory's capacity [SYS-10]. The runtime
/// keeps the count; this wrapper spends one credit without a host call and
/// answers `Ok(permit)`, or `Err(ResourceExhausted)` when the factory is
/// spent. The permit's own representation stays the erased bit: target open
/// wrappers consume it in the checked program and pass nothing native.
fn emit_reserve_handle(
    program: &IrProgram<'_, '_, '_>,
    implementation: ApprovedImplementation,
    shape: &OutcomeShape,
) -> Result<String, BackendFailure> {
    let permit = representation(SystemResourceType::HandlePermit);
    if shape.ok_llvm != permit {
        return Err(BackendFailure::InvalidIr);
    }
    let classes = io_error_classes(program, shape.err_type)?;
    let exhausted = classes
        .iter()
        .find(|class| class.spelling == "ResourceExhausted")
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
    // A spent factory is a refusal, not a host error: no native code, and the
    // origin that says so [SYS-7].
    let (error_body, error_value) =
        io_error_value(err_llvm, exhausted, "spent", "0", &ORIGIN_NONE.to_string());
    Ok(format!(
        "define private {llvm} @{symbol}() alwaysinline {{\n\
         entry:\n  \
         %granted.native = call i32 @wf__handle_reserve()\n  \
         %granted = icmp eq i32 %granted.native, 1\n  \
         br i1 %granted, label %grant, label %refuse\n\
         grant:\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, {permit} true, {ok_index}\n  \
         ret {llvm} %ok\n\
         refuse:\n\
         {error_body}  \
         %err.tag = insertvalue {llvm} zeroinitializer, i32 {err_tag}, 0\n  \
         %err = insertvalue {llvm} %err.tag, {err_llvm} {error_value}, {err_index}\n  \
         ret {llvm} %err\n\
         }}\n\n",
        symbol = implementation.symbol(),
    ))
}

/// The private symbol of the one close every [SYS-5] derived release and
/// every explicit close reaches.
const CLOSE_HELPER: &str = "wf.sys.close";

/// Emits that one close.
///
/// A close is an operation like every other one: the record is reserved in
/// this helper's own entry block, the descriptor is submitted, and the
/// terminal completion is joined here — exactly one attempt, its diagnostic
/// discarded, never retried [SYS-5]. The helper is `alwaysinline` like every
/// other qualified wrapper, so the record is a block of the frame that
/// released the resource (design §5, §8).
fn emit_close_helper(target: SystemTarget) -> String {
    let descriptor = representation(SystemResourceType::ReadFile);
    let submit = completion_submit_call(
        target.file_close_submit_symbol(),
        &format!("{descriptor} %descriptor, ptr {WRAPPER_RECORD}"),
    );
    format!(
        "define private void @{CLOSE_HELPER}({descriptor} %descriptor) alwaysinline {{\n\
         entry:\n\
         {reservation}\
         {submit}  \
         call void @{join}(ptr {WRAPPER_RECORD}, ptr {WRAPPER_RAW_VALUE}, \
         ptr {WRAPPER_RAW_ERROR})\n  \
         ret void\n\
         }}\n\n",
        reservation = completion_wrapper_reservation(false),
        join = target.file_join_symbol(),
    )
}

/// An explicit close [SYS-10]: the same one native close attempt derived
/// release performs, its diagnostic discarded the same way, and the credit the
/// open held comes back as the fresh permit, which is the erased bit. The
/// factory's count is untouched: the permit value is the credit.
fn emit_close(implementation: ApprovedImplementation, resource: SystemResourceType) -> String {
    let permit = representation(SystemResourceType::HandlePermit);
    let owner = representation(resource);
    format!(
        "define private {permit} @{symbol}({owner} %owner) alwaysinline {{\n\
         entry:\n  \
         call void @{CLOSE_HELPER}({owner} %owner)\n  \
         ret {permit} true\n\
         }}\n\n",
        symbol = implementation.symbol(),
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
            // HandleFactory is a proof-only affine entry value. Supplying it
            // performs no host allocation and carries no native handle.
            4 => {
                supplied.push("i1 true".to_owned());
            }
            // The standard input binding supplies one affine owner over the
            // invocation's own descriptor 0 [SYS-15]. Like the two output
            // sinks it is a handle the invocation already holds, so the
            // bootstrap opens nothing and the factory's capacity already
            // excludes it.
            5 => {
                supplied.push("i32 0".to_owned());
            }
            6 => supplied.push(format!("{} zeroinitializer", PROVIDER_REPRESENTATION)),
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
            5 => {
                body.push_str(
                    "  %stdin = call i32 @wf__windows_stdin_descriptor()\n  \
                     %stdin.available = icmp sge i32 %stdin, 0\n",
                );
                supplied.push("i32 %stdin".to_owned());
                available.push("%stdin.available");
            }
            6 => supplied.push(format!("{} zeroinitializer", PROVIDER_REPRESENTATION)),
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
        2 | 3 => Ok(Some(SystemResourceType::OutputStream)),
        4 => Ok(Some(SystemResourceType::HandleFactory)),
        5 => Ok(Some(SystemResourceType::InputStream)),
        6 => Ok(None),
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
/// effect. A native close attempt is exactly one close, submitted and joined
/// through the one close helper, whose diagnostic is discarded and which never
/// retries an ambiguous close.
pub(super) fn emit_resource_release(
    qualification: &Qualification,
    output: &mut String,
    contract: crate::SystemResourceContract,
    operand: &str,
) -> Result<(), BackendFailure> {
    match qualification.resource(contract.resource)?.release() {
        ReleaseImplementation::NoCode => Ok(()),
        ReleaseImplementation::NativeClose => {
            // The helper reserves the record and joins the close inside the
            // frame this release runs in, so a release inside a loop reserves
            // nothing per iteration.
            writeln!(
                output,
                "  call void @{CLOSE_HELPER}({} {operand})",
                representation(contract.resource)
            )
            .map_err(|_| BackendFailure::TextEmission)
        }
        // One direction's half-close [SYS-18]: exactly one attempt through the
        // one half-close helper, whose diagnostic is discarded and which never
        // retries. The runtime keeps the pair's own two-count, so this
        // release does not decide whether it is the one that releases the
        // target's object — and the release order of the two directions is
        // the program's own and changes no outcome it can observe.
        ReleaseImplementation::NativeDirectionClose => {
            let direction = match contract.resource {
                SystemResourceType::TcpReceive => DIRECTION_RECEIVE,
                SystemResourceType::TcpSend => DIRECTION_SEND,
                // No other resource carries this release row.
                _ => return Err(BackendFailure::InvalidIr),
            };
            writeln!(
                output,
                "  call void @{HALF_CLOSE_HELPER}({} {operand}, i32 {direction})",
                representation(contract.resource)
            )
            .map_err(|_| BackendFailure::TextEmission)
        }
    }
}
