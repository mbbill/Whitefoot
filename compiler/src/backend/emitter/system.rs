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
    ApprovedImplementation, ProgramKind, Qualification, ReleaseImplementation, SystemTarget,
    qualified_representation,
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
const EXIT_STATUS: u8 = 10;

/// The private symbol of the shared UTF-8 validator both text-route
/// implementations use [HOST-2].
const UTF8_VALIDATOR: &str = "wf.sys.utf8.valid";

/// The private constant naming the initial working directory.
const WORKING_DIRECTORY: &str = "@.wf.sys.working.directory";

/// Everything the qualified system interface adds to one module.
pub(super) struct SystemEmission {
    /// Private constants the bootstrap needs.
    pub(super) constants: String,
    /// Host and intrinsic declarations the approved implementations call.
    pub(super) declarations: String,
    /// The approved implementations themselves.
    pub(super) definitions: String,
}

/// Emits the approved implementation of every used semantic identity.
pub(super) fn emit_system_interface(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
) -> Result<SystemEmission, BackendFailure> {
    let mut constants = String::new();
    let mut declarations = BTreeSet::new();
    let mut definitions = String::new();
    let mut needs_validator = false;

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
                qualification.target(),
            )?),
            EXIT_STATUS => definitions.push_str(&emit_exit_status(implementation)),
            _ => return Err(BackendFailure::InvalidIr),
        }
        for declaration in operation_declarations(ordinal) {
            declarations.insert(*declaration);
        }
    }
    if needs_validator {
        definitions.push_str(&emit_utf8_validator());
    }

    if releases_natively(program, qualification)? {
        declarations.insert("declare i32 @close(i32)");
    }

    if let IrEntry::Command { inputs, .. } = program.entry() {
        declarations.insert("declare ptr @signal(i32, ptr)");
        declarations.insert("declare void @exit(i32) noreturn");
        if inputs.contains(&1) {
            declarations.insert("declare i32 @open(ptr, i32, ...)");
            constants.push_str(&format!(
                "{WORKING_DIRECTORY} = private unnamed_addr constant [2 x i8] c\".\\00\", align 1\n"
            ));
        }
    }

    let mut declaration_text = String::new();
    for declaration in declarations {
        declaration_text.push_str(declaration);
        declaration_text.push('\n');
    }
    Ok(SystemEmission {
        constants,
        declarations: declaration_text,
        definitions,
    })
}

/// The host and intrinsic symbols one approved implementation calls.
fn operation_declarations(ordinal: u8) -> &'static [&'static str] {
    match ordinal {
        ARG_GET => &["declare i64 @strlen(ptr)"],
        HOST_COPY_BYTES => &["declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)"],
        HOST_COPY_UTF8 => &["declare void @llvm.memcpy.p0.p0.i64(ptr, ptr, i64, i1)"],
        RELATIVE_PATH => &["declare ptr @memchr(ptr, i32, i64)"],
        _ => &[],
    }
}

/// Whether any release the program derives is a native close attempt.
fn releases_natively(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
) -> Result<bool, BackendFailure> {
    for nominal in program.nominals() {
        let IrNominalKind::SystemResource(contract) = nominal.kind() else {
            continue;
        };
        if matches!(
            qualification.resource(contract.resource)?.release(),
            ReleaseImplementation::NativeClose(_)
        ) {
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

/// The [SYS-8] range validation every one-attempt operation performs first.
///
/// Overflow of the mathematical sum of the two range values in u64, an offset
/// beyond the buffer's runtime length, or a range extending past that length
/// traps under the bounds semantics of [OP-4], before any host transfer,
/// before any read of the source value, and before any write of the
/// destination. `%end < %offset` is exactly u64 overflow of the sum, and
/// `%end <= %length` covers both remaining conditions because the capacity is
/// never negative.
fn range_validation(buffer: &str) -> String {
    format!(
        "entry:\n  \
         %end = add i64 %offset, %capacity\n  \
         %wrapped = icmp ult i64 %end, %offset\n  \
         %exact = xor i1 %wrapped, true\n  \
         %length = extractvalue {buffer} %destination, 1\n  \
         %within = icmp ule i64 %end, %length\n  \
         %admitted = and i1 %exact, %within\n  \
         br i1 %admitted, label %measure, label %range.trap\n\
         range.trap:\n  \
         call void @wf_trap(ptr %record, i64 %record.length)\n  \
         unreachable\n"
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
    let validation = range_validation(&buffer);
    // The lossless route transfers the target's own code units with no
    // validation and no Unicode restriction [HOST-2]; its only recoverable
    // failure is a destination too small for the exact length, which leaves
    // the whole destination buffer unchanged [SYS-8].
    Ok(format!(
        "define private {llvm} @{symbol}({lease} %value, {buffer} %destination, i64 %offset, \
         i64 %capacity, ptr %record, i64 %record.length) alwaysinline {{\n\
         {validation}\
         measure:\n  \
         %required = extractvalue {lease} %value, 1\n  \
         %room = icmp ule i64 %required, %capacity\n  \
         br i1 %room, label %transfer, label %small\n\
         transfer:\n  \
         %source = extractvalue {lease} %value, 0\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %target = getelementptr inbounds i8, ptr %base, i64 %offset\n  \
         call void @llvm.memcpy.p0.p0.i64(ptr %target, ptr %source, i64 %required, i1 false)\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, i64 %required, {ok_index}\n  \
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
    let validation = range_validation(&buffer);
    // The text route validates and measures the encoding first and returns
    // the invalid or too-small outcome without writing any byte; only then
    // does it copy the complete encoding [SYS-8, HOST-2].
    Ok(format!(
        "define private {llvm} @{symbol}({lease} %value, {buffer} %destination, i64 %offset, \
         i64 %capacity, ptr %record, i64 %record.length) alwaysinline {{\n\
         {validation}\
         measure:\n  \
         %text = extractvalue {lease} %value, 0\n  \
         %required = extractvalue {lease} %value, 1\n  \
         %valid = call i1 @{UTF8_VALIDATOR}(ptr %text, i64 %required)\n  \
         br i1 %valid, label %fit, label %invalid\n\
         fit:\n  \
         %room = icmp ule i64 %required, %capacity\n  \
         br i1 %room, label %transfer, label %small\n\
         transfer:\n  \
         %base = extractvalue {buffer} %destination, 0\n  \
         %target = getelementptr inbounds i8, ptr %base, i64 %offset\n  \
         call void @llvm.memcpy.p0.p0.i64(ptr %target, ptr %text, i64 %required, i1 false)\n  \
         %ok.tag = insertvalue {llvm} zeroinitializer, i32 {ok_tag}, 0\n  \
         %ok = insertvalue {llvm} %ok.tag, i64 %required, {ok_index}\n  \
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
) -> Result<String, BackendFailure> {
    let IrEntry::Command { inputs, .. } = program.entry() else {
        if qualification.kind() != ProgramKind::Unlabelled
            || main.result() != IrType::Unit
            || !main.parameters().is_empty()
        {
            return Err(BackendFailure::InvalidIr);
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
            }
            1 => {
                opens_directory = true;
                writeln!(
                    body,
                    "  %cwd = call i32 (ptr, i32, ...) @open(ptr {WORKING_DIRECTORY}, i32 {})",
                    target.directory_open_flags()
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                supplied.push("i32 %cwd".to_owned());
            }
            // The standard output and standard error entry bindings supply
            // separate affine owners over the invocation's own descriptors
            // [SYS-12]; neither is a shared global sink and neither carries a
            // lock.
            2 => supplied.push("i32 1".to_owned()),
            3 => supplied.push("i32 2".to_owned()),
            _ => return Err(BackendFailure::InvalidIr),
        }
    }
    if opens_directory {
        // [PROG-3]: supplying each declared standard input is a start-time
        // obligation of the selected target; when it cannot supply one, start
        // fails before the entry is invoked.
        body.push_str(
            "  %cwd.opened = icmp sge i32 %cwd, 0\n  \
             br i1 %cwd.opened, label %enter, label %start.failure\n",
        );
    } else {
        body.push_str("  br label %enter\n");
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
        trap: Option<&IrTrapSite>,
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
        // A trapping row carries the [DIAG-3] record of its own [SYS-8] range
        // validation; the record is per-site data, so the shared approved
        // implementation receives it as an argument and constant folding
        // removes the transfer once the wrapper inlines.
        match (row.traps, trap) {
            (true, Some(trap)) => {
                let trap_id = self.register_trap(trap)?;
                rendered.push(format!("ptr @.wf_trap.{trap_id}"));
                rendered.push(format!("i64 {}", self.traps[trap_id].len()));
            }
            (false, None) => {}
            _ => return Err(BackendFailure::InvalidIr),
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
