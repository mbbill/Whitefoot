//! Conservative textual LLVM emission for the active Whitefoot specification.
//!
//! Emission consumes only target-independent IR. It preserves every retained
//! check, emits no overflow or alias promises, initializes complete aggregate
//! representations, and keeps a defensive abort edge for enum discriminants.

mod arena;
mod array;
mod boxes;
mod buffer;
mod cleanup;
mod completion;
mod conversion;
mod floating;
mod floor;
mod integer;
mod operations;
mod parallel;
mod reinterpret;
mod slice;
mod stackless;
mod system;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write;

use super::qualification::{
    Qualification, QualificationFailure, SystemTarget, qualified_representation, qualify_program,
};
use super::target::{TargetLayout, TargetLayoutFailure, validate_program};
use crate::{
    IrAddressed, IrArrayRoot, IrBlock, IrBlockId, IrBooleanOperation, IrClaimSite,
    IrCompletionStep, IrConstant, IrDrop, IrEntry, IrEnumType, IrFloatOperation, IrFunction,
    IrGlobalValue, IrInstruction, IrIntegerOperation, IrLayoutCeiling, IrNominal, IrNominalId,
    IrNominalKind, IrOperation, IrOverlap, IrProgram, IrRuntimeTargetObligations,
    IrTargetDomainObligation, IrTerminator, IrType, IrValueId, SystemResourceType,
};
use buffer::{buffer_fill_done_label, buffer_probe_join_label, buffer_vacant_done_label};
use cleanup::{emit_resource_drop_helpers, emit_value_cleanup, type_requires_cleanup};
use completion::completion_offered_label;
pub use completion::{
    COMPLETION_BRIDGE_HEADER, COMPLETION_BRIDGE_SOURCE, COMPLETION_CONTRACT_HEADER,
    COMPLETION_FILE_ADAPTER_HEADER, COMPLETION_FILE_ADAPTER_SOURCE,
    COMPLETION_LINUX_IO_URING_HEADER, COMPLETION_LINUX_IO_URING_SOURCE, COMPLETION_RUNTIME_SOURCE,
    WRITER_SCHEDULER_HEADER, WRITER_SCHEDULER_SOURCE, module_requires_completion_runtime,
};
use floor::FLOOR_RUNTIME_FALLBACK;
pub use floor::FLOOR_RUNTIME_SOURCE;
pub use floor::FLOOR_STACK_BYTES;
use parallel::{
    HandedOut, LoopSplitSite, PARALLEL_POOL_QUERY_FALLBACK, PARALLEL_RUNTIME_FALLBACK,
    PARALLEL_SPLIT_BUDGET_FALLBACK, ParallelThunks, par_done_label, sequential_clone_set,
    sequential_clone_symbol,
};
pub use parallel::{
    PARALLEL_COMPLETION_RUNTIME_SOURCE, PARALLEL_RUNTIME_SOURCE, module_requires_parallel_runtime,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendFailure {
    TargetLayout(TargetLayoutFailure),
    /// The [QUAL-1] target-qualification table has no approved implementation
    /// for a facility the program uses on the selected target and program
    /// kind, or a required [QUAL-2] target guarantee is unmet. Like a
    /// target-layout failure this is not a source-language rejection and cites
    /// no language rule [DIAG-1].
    TargetQualification(QualificationFailure),
    /// A lowering handed one call site a second operation while its first was
    /// still outstanding. Completion storage is reserved per outstanding
    /// operation and this emitter reserves one element per site, so a second
    /// live operation of one site has nowhere of its own to write. It is
    /// refused rather than given the first operation's element: sharing would
    /// let the newer operation overwrite a result or a staged path the older
    /// one is still being read from, with no compile error and no crash. Like
    /// a target-layout stop this is an emitter capability limit and not a
    /// source-language rejection; it cites no language rule [DIAG-1].
    SecondOutstandingCompletionOperation,
    /// A function ended with a target operation still outstanding. A staged
    /// loop pipeline may leave operations in flight across the blocks it
    /// names, but every path out of its loop reaches a block it does not name,
    /// and that block retires them. A pipeline that named a block on every
    /// exit path would leave an accepted operation owned by nobody — the
    /// target would still write its result into storage the frame no longer
    /// exists to hold. Like the refusal above this is an emitter capability
    /// limit rather than a source-language rejection; it cites no language
    /// rule [DIAG-1].
    UnretiredCompletionOperation,
    /// A staged loop pipeline's ring cannot be addressed by the blocks that
    /// reach it. Exactly three descriptors raise this: a ring with no slots at
    /// all; a slot a block names that is not a `u64` *value of the function*;
    /// and a block that reaches completion storage — reserving a ring or
    /// addressing an element — while naming no slot, which is refused rather
    /// than handed element zero, since that is the silent sharing the ring
    /// exists to prevent. Nothing else is checked. In particular the value's
    /// *dominance* over the block naming it, and its *range* against the ring
    /// width, are trusted exactly as every other operand this emitter renders
    /// is trusted: a driver threads the slot along the edges into its region
    /// and owes the range a static refusal or a proof of its own. Like the two
    /// refusals above this is an emitter capability limit rather than a
    /// source-language rejection; it cites no language rule [DIAG-1].
    MisaddressedCompletionSlot,
    InvalidIr,
    CounterOverflow,
    TextEmission,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LlvmModule {
    text: String,
}

impl LlvmModule {
    #[must_use]
    pub fn into_string(self) -> String {
        self.text
    }
}

pub fn emit_llvm(program: &IrProgram<'_, '_, '_>) -> Result<LlvmModule, BackendFailure> {
    let target = TargetLayout::host().map_err(BackendFailure::TargetLayout)?;
    // [QUAL-1] consults the qualification table after the exact target and ABI
    // are selected and before emitting any use of an operation. It runs before
    // layout because an opaque resource has no target representation until its
    // qualification record fixes one.
    let system_target = SystemTarget::for_triple(target.triple()).ok_or(
        BackendFailure::TargetLayout(TargetLayoutFailure::UnsupportedHost),
    )?;
    emit_llvm_for(program, system_target)
}

/// Emits one program against an explicitly selected system target.
///
/// `emit_llvm` selects the host's native target; a test harness that has
/// selected the deterministic test target calls this instead. Selection stays
/// one decision taken before qualification either way [QUAL-1].
#[cfg(test)]
pub(crate) fn emit_llvm_for_target(
    program: &IrProgram<'_, '_, '_>,
    system_target: SystemTarget,
) -> Result<LlvmModule, BackendFailure> {
    emit_llvm_for(program, system_target)
}

fn emit_llvm_for(
    program: &IrProgram<'_, '_, '_>,
    system_target: SystemTarget,
) -> Result<LlvmModule, BackendFailure> {
    let target = TargetLayout::host().map_err(BackendFailure::TargetLayout)?;
    let qualification = qualify_program(system_target, program)?;
    validate_program(target, &qualification, program).map_err(BackendFailure::TargetLayout)?;
    let main = program
        .functions()
        .get(program.main_ordinal() as usize)
        .ok_or(BackendFailure::InvalidIr)?;
    let system = system::emit_system_interface(program, &qualification)?;

    let mut claim_records = Vec::new();
    let mut intrinsics = BTreeSet::new();
    let mut thunks = ParallelThunks::default();
    let mut completion_used = false;
    let mut functions = String::new();
    let stackless = stackless::StacklessPlan::build(program, &qualification);
    for (ordinal, function) in program.functions().iter().enumerate() {
        let emitter = FunctionEmitter::new(
            program,
            &qualification,
            function,
            target,
            ModuleState {
                claim_records: &mut claim_records,
                intrinsics: &mut intrinsics,
                parallel: &mut thunks,
                completion_used: &mut completion_used,
                sequential_clones: None,
            },
        );
        let emitted = if stackless
            .as_ref()
            .is_some_and(|plan| u32::try_from(ordinal).ok() == Some(plan.root_ordinal()))
        {
            emitter.emit_stackless_root(stackless.as_ref().expect("checked above"))?
        } else {
            emitter.emit()?
        };
        functions.push_str(&emitted);
    }
    if let Some(plan) = &stackless {
        functions.push_str(&plan.emit_tail_definitions(program, &qualification)?);
    }
    // The second world. It exists only where the first one actualizes
    // something, so a build that hands nothing out — every default build among
    // them — emits exactly the module it emitted before this path existed.
    //
    // The bootstrap selects between the two worlds by calling the entry
    // function's clone, so a set without the entry in it would emit a call to
    // a definition that is not there. The set is closed upwards through the
    // call graph and so holds the entry whenever it holds anything; reading
    // that off the set rather than trusting the argument is what makes the
    // module well-formed by construction instead of by that reasoning.
    let mut clones = if thunks.is_used() {
        sequential_clone_set(program)
    } else {
        HashSet::new()
    };
    if !clones.contains(&program.main_ordinal()) {
        clones.clear();
    }
    for (ordinal, function) in program.functions().iter().enumerate() {
        if u32::try_from(ordinal).is_ok_and(|ordinal| clones.contains(&ordinal)) {
            functions.push_str(
                &FunctionEmitter::new(
                    program,
                    &qualification,
                    function,
                    target,
                    ModuleState {
                        claim_records: &mut claim_records,
                        intrinsics: &mut intrinsics,
                        parallel: &mut thunks,
                        completion_used: &mut completion_used,
                        sequential_clones: Some(&clones),
                    },
                )
                .emit()?,
            );
        }
    }
    let entry = system::emit_entry(program, &qualification, main, !clones.is_empty())?;
    let has_matches = program.functions().iter().any(|function| {
        function
            .blocks()
            .iter()
            .any(|block| matches!(block.terminator(), IrTerminator::Match { .. }))
    });
    let drop_helpers = emit_resource_drop_helpers(program, &qualification)?;
    let has_arena_storage = program
        .nominals()
        .iter()
        .any(|nominal| matches!(nominal.kind(), IrNominalKind::ArenaStorage));
    let has_heap_storage = !drop_helpers.is_empty()
        || has_arena_storage
        || program.functions().iter().any(IrFunction::contains_buffer)
        || program.nominals().iter().any(|nominal| {
            matches!(
                nominal.kind(),
                IrNominalKind::Box { .. } | IrNominalKind::Arena { .. }
            )
        });
    // The dynamic target-domain guard is emitted by exactly these two
    // operations, so its record and its abort helper follow the operations
    // rather than the buffer type: a module that carries buffers it never
    // allocates emits no guard and needs neither.
    let has_target_domain_guard = program.functions().iter().any(|function| {
        function.blocks().iter().any(|block| {
            block.instructions().iter().any(|instruction| {
                matches!(
                    instruction,
                    IrInstruction::Define {
                        operation: IrOperation::BufferFill { .. }
                            | IrOperation::BufferVacant { .. },
                        ..
                    }
                )
            })
        })
    });

    let mut text = format!(
        "; Whitefoot conservative module\nsource_filename = \"whitefoot\"\ntarget datalayout = \"{}\"\ntarget triple = \"{}\"\n\n",
        target.data_layout(),
        target.triple(),
    );
    emit_nominal_declarations(&mut text, program)?;
    emit_global_constants(&mut text, program)?;
    text.push_str(&system.constants);
    for (index, bytes) in claim_records.iter().enumerate() {
        writeln!(
            text,
            "@.wf_trap.{index} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            bytes.len(),
            llvm_bytes(bytes)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    // An allocation this host refuses is the heap twin of an exhausted stack,
    // and it gets the same treatment: one record naming the resource class,
    // written once, before a defined abort. The bytes carry no `rule_id`, no
    // function, and no node path, which is what keeps them from being read as
    // a [DIAG-3] trap record — running out of memory is not something the
    // writer did.
    if has_heap_storage {
        writeln!(
            text,
            "@.wf_resource.heap = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            HEAP_RECORD.len(),
            llvm_bytes(HEAP_RECORD.as_bytes())
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    if has_target_domain_guard {
        writeln!(
            text,
            "@.wf_resource.target_domain = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            TARGET_DOMAIN_RECORD.len(),
            llvm_bytes(TARGET_DOMAIN_RECORD.as_bytes())
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    // One writer serves both records: it is already a general "write these
    // bytes, then abort, exactly once in this process" primitive, and sharing
    // it is what makes "no execution produces both records" a mechanism rather
    // than an argument.
    let writes_a_record = !claim_records.is_empty() || has_heap_storage || has_target_domain_guard;
    // A latch decides between threads, so it belongs only to a module that has
    // more than one. `thunks.is_used()` is exactly "this module hands a call
    // out to a worker lane": false for every default build, and false for a
    // `--par` build that actualizes nothing. A lone thread races no one, so
    // those modules emit the trap path they emitted before the latch existed.
    let latched_trap = writes_a_record && thunks.is_used();
    if latched_trap {
        text.push_str(TRAP_LATCH);
    }
    // The mandatory [DIAG-3] record and the qualified system interface can
    // need the same host symbol; one module declares it once.
    let mut system_declarations = system.declarations;
    if writes_a_record {
        text.push('\n');
        text.push_str("declare i64 @write(i32, ptr, i64)\n");
        system_declarations.remove("declare i64 @write(i32, ptr, i64)");
    }
    if writes_a_record || has_matches {
        text.push_str("declare void @abort() noreturn\n");
        system_declarations.remove("declare void @abort() noreturn");
    }
    if has_heap_storage {
        text.push_str("declare ptr @malloc(i64)\ndeclare void @free(ptr)\n");
    }
    for declaration in &system_declarations {
        text.push_str(declaration);
        text.push('\n');
    }
    if latched_trap {
        text.push_str(TRAP_LATCH_FALLBACK);
        text.push_str(LATCHED_TRAP_WRITER);
    } else if writes_a_record {
        text.push_str(SEQUENTIAL_TRAP_WRITER);
    } else if has_matches {
        text.push('\n');
    }
    if has_heap_storage {
        writeln!(
            text,
            "define private void @wf_resource_abort() noreturn {{\nentry:\n  call void @wf_trap(ptr @.wf_resource.heap, i64 {})\n  unreachable\n}}\n",
            HEAP_RECORD.len()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    if has_target_domain_guard {
        writeln!(
            text,
            "define private void @wf_target_domain_abort() noreturn {{\nentry:\n  call void @wf_trap(ptr @.wf_resource.target_domain, i64 {})\n  unreachable\n}}\n",
            TARGET_DOMAIN_RECORD.len()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    if has_arena_storage {
        text.push('\n');
        text.push_str(arena::ARENA_RELEASE_HELPER);
    }
    text.push_str(&drop_helpers);
    text.push_str(&system.definitions);
    for intrinsic in intrinsics {
        match intrinsic {
            IntrinsicDeclaration::Overflow { name, ty } => {
                writeln!(text, "declare {{ {ty}, i1 }} @{name}({ty}, {ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::UnaryWithFlag { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty}, i1)")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::Unary { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::Binary { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty}, {ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::Ternary { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty}, {ty}, {ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::UnaryCast {
                name,
                result_ty,
                argument_ty,
            } => writeln!(text, "declare {result_ty} @{name}({argument_ty})")
                .map_err(|_| BackendFailure::TextEmission)?,
        }
    }
    // Emitted only where a permitted overlap group is actually handed out, so
    // a module that overlaps nothing names no runtime symbol at all.
    if thunks.requires_runtime() {
        text.push('\n');
        text.push_str(PARALLEL_RUNTIME_FALLBACK);
        if !clones.is_empty() {
            text.push_str(PARALLEL_POOL_QUERY_FALLBACK);
        }
        if thunks.queries_split_budget() {
            text.push_str(PARALLEL_SPLIT_BUDGET_FALLBACK);
        }
        text.push_str(thunks.definitions());
    }
    if completion_used {
        text.push('\n');
        text.push_str(completion::COMPLETION_RUNTIME_FALLBACK);
        // Emitted only where a module actually asks for a window, exactly as
        // the split budget's fallback is, so a module that stages no loop
        // names no such symbol at all.
        if functions.contains("@wf__completion_window(") {
            text.push_str(completion::COMPLETION_WINDOW_FALLBACK);
        }
    }
    if stackless.is_some() {
        text.push('\n');
        text.push_str(stackless::STACKLESS_RUNTIME_FALLBACK);
    }
    if !functions.is_empty() {
        text.push('\n');
        text.push_str(&functions);
    }
    // Unconditional, unlike the parallel runtime's: every program can run out
    // of stack, so every module names the floor and carries its own answer for
    // a link that does not supply one.
    text.push('\n');
    text.push_str(FLOOR_RUNTIME_FALLBACK);
    text.push_str(&entry);
    Ok(LlvmModule {
        text: attach_stack_probe(&text, target),
    })
}

/// The bytes an allocation refusal writes before aborting.
///
/// The heap twin of the exhausted-stack record the floor's runtime writes, and
/// fixed the same way and for the same reasons: it names the resource class
/// and nothing else. What it leaves out is what distinguishes it from a
/// [DIAG-3] record — no rule identifier, no function, no node path — because
/// an allocation the host refused is the trusted computing base reaching its
/// limit, not a contract the writer failed to keep.
const HEAP_RECORD: &str = "{\"resource\":\"heap\"}\n";

/// The bytes a refused dynamic target-domain guard writes before aborting.
///
/// A third class rather than a second spelling of `heap`, because the two
/// conditions are different and the class is the only thing the record says.
/// An allocation refusal is memory running out; this guard fires when the byte
/// count a `buffer` asks for has no exact value in the target's allocator or
/// address-index domain, which happens on a machine with memory to spare and
/// would still happen if the machine had more. The specification keeps the two
/// apart everywhere it names them: its compile-time classification lists
/// "resource failure" and "target-layout failure" as separate members of the
/// same non-rejection sum, and it routes this guard down the same
/// non-continuing path without merging it into that class. Folding them here
/// would make the record point a reader at the wrong resource.
const TARGET_DOMAIN_RECORD: &str = "{\"resource\":\"target-domain\"}\n";

/// The attribute group every generated definition carries.
const STACK_PROBE_GROUP: &str = "#0";

/// Gives every definition in the assembled module the target's `probe-stack`
/// attribute, and appends the group it names.
///
/// This runs over the finished module rather than at each `define` site so
/// that what the code establishes is "every generated function" rather than
/// "every site someone remembered": a definition introduced later carries the
/// probe without anyone deciding to give it one. [SCOPE-3] containment under
/// exhaustion is exactly a completeness property — one unprobed large frame
/// is enough to step over the guard region into a neighbouring thread's live
/// stack — so completeness is what the emission establishes.
///
/// A `define` line always ends in ` {`, after any attribute keyword it
/// carries, and a definition is always followed by its body, so the suffix
/// test identifies exactly the definition lines. A rodata constant renders on
/// one line with its bytes escaped, so no constant's contents can look like a
/// definition to this scan.
fn attach_stack_probe(module: &str, target: TargetLayout) -> String {
    let mut text = String::with_capacity(module.len() + 64);
    for line in module.split_inclusive('\n') {
        match line.strip_suffix(" {\n") {
            Some(head) if head.starts_with("define ") => {
                text.push_str(head);
                text.push(' ');
                text.push_str(STACK_PROBE_GROUP);
                text.push_str(" {\n");
            }
            _ => text.push_str(line),
        }
    }
    text.push_str("\nattributes ");
    text.push_str(STACK_PROBE_GROUP);
    text.push_str(" = { \"probe-stack\"=\"");
    text.push_str(target.stack_probe());
    text.push_str("\" }\n");
    text
}

fn emit_global_constants(
    output: &mut String,
    program: &IrProgram<'_, '_, '_>,
) -> Result<(), BackendFailure> {
    for constant in program.constants() {
        writeln!(output, "; const {}", constant.name())
            .map_err(|_| BackendFailure::TextEmission)?;
        write!(
            output,
            "{} = private unnamed_addr constant {} {}",
            constant_symbol(constant.id()),
            llvm_type(program, constant.ty())?,
            global_constant_value(program, constant.value(), constant.ty())?
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        output.push('\n');
    }
    if !program.constants().is_empty() {
        output.push('\n');
    }
    Ok(())
}

/// Renders one rodata constant value of one exact type: a scalar operand, a
/// complete array, or a complete struct aggregate with each field rendered
/// recursively [CONST-2 candidate].
fn global_constant_value(
    program: &IrProgram<'_, '_, '_>,
    value: &IrGlobalValue,
    ty: IrType,
) -> Result<String, BackendFailure> {
    match (value, ty) {
        (IrGlobalValue::Scalar(value), ty) => constant_operand(*value, ty),
        (IrGlobalValue::Array(elements), IrType::Array { element, length }) => {
            if u64::try_from(elements.len()).map_err(|_| BackendFailure::CounterOverflow)? != length
            {
                return Err(BackendFailure::InvalidIr);
            }
            if elements.is_empty() {
                return Ok("zeroinitializer".to_owned());
            }
            let mut text = String::from("[");
            let element_type = element.ty();
            let llvm_element_type = llvm_type(program, element_type)?;
            for (index, value) in elements.iter().enumerate() {
                if index != 0 {
                    text.push_str(", ");
                }
                write!(
                    text,
                    "{llvm_element_type} {}",
                    constant_operand(*value, element_type)?
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            text.push(']');
            Ok(text)
        }
        (IrGlobalValue::Struct(fields), IrType::Nominal(id)) => {
            let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
            let IrNominalKind::Struct { fields: declared } = nominal.kind() else {
                return Err(BackendFailure::InvalidIr);
            };
            if fields.len() != declared.len() {
                return Err(BackendFailure::InvalidIr);
            }
            if fields.is_empty() {
                return Ok("zeroinitializer".to_owned());
            }
            let mut text = String::from("{ ");
            for (index, (value, field)) in fields.iter().zip(declared).enumerate() {
                if index != 0 {
                    text.push_str(", ");
                }
                write!(
                    text,
                    "{} {}",
                    llvm_type(program, field.ty())?,
                    global_constant_value(program, value, field.ty())?
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            text.push_str(" }");
            Ok(text)
        }
        _ => Err(BackendFailure::InvalidIr),
    }
}

fn emit_nominal_declarations(
    output: &mut String,
    program: &IrProgram<'_, '_, '_>,
) -> Result<(), BackendFailure> {
    let mut emitted = false;
    for nominal in program.nominals() {
        // A box is a pointer and an opaque system resource carries the
        // representation its [QUAL-1] qualification record fixes; neither
        // needs a named aggregate type.
        if nominal.is_tag_only_enum()
            || matches!(
                nominal.kind(),
                IrNominalKind::Box { .. }
                    | IrNominalKind::Arena { .. }
                    | IrNominalKind::ArenaStorage
                    | IrNominalKind::SystemResource(_)
            )
        {
            continue;
        }
        emitted = true;
        write!(output, "{} = type {{ ", nominal_symbol(nominal.id()))
            .map_err(|_| BackendFailure::TextEmission)?;
        match nominal.kind() {
            IrNominalKind::Struct { fields } => {
                for (index, field) in fields.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&llvm_type(program, field.ty())?);
                }
            }
            IrNominalKind::Enum { variants } => {
                output.push_str("i32");
                for variant in variants {
                    for field in variant.fields() {
                        output.push_str(", ");
                        output.push_str(&llvm_type(program, field.ty())?);
                    }
                }
            }
            IrNominalKind::Box { .. }
            | IrNominalKind::Arena { .. }
            | IrNominalKind::ArenaStorage
            | IrNominalKind::SystemResource(_) => {
                return Err(BackendFailure::InvalidIr);
            }
        }
        output.push_str(" }\n");
    }
    if emitted {
        output.push('\n');
    }
    Ok(())
}

#[derive(Clone)]
struct Incoming {
    predecessor: IrBlockId,
    arguments: Vec<IrValueId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum IntrinsicDeclaration {
    Overflow {
        name: String,
        ty: String,
    },
    UnaryWithFlag {
        name: String,
        ty: String,
    },
    Unary {
        name: String,
        ty: String,
    },
    Binary {
        name: String,
        ty: String,
    },
    Ternary {
        name: String,
        ty: String,
    },
    UnaryCast {
        name: String,
        result_ty: String,
        argument_ty: String,
    },
}

struct FunctionEmitter<'program, 'state> {
    program: &'program IrProgram<'program, 'program, 'program>,
    /// The [QUAL-1] table lookup this build already performed. Every emission
    /// site reads the resolved row; none consults the table again.
    qualification: &'program Qualification,
    function: &'program IrFunction,
    target: TargetLayout,
    claim_records: &'state mut Vec<Vec<u8>>,
    intrinsics: &'state mut BTreeSet<IntrinsicDeclaration>,
    incoming: Vec<Vec<Incoming>>,
    output: String,
    /// Stack slot declarations hoisted to the top of the function's entry block.
    ///
    /// A slot is requested where it is used, but a repeated `alloca` grows the
    /// frame once per execution, so a slot inside a loop would grow the frame
    /// without bound. Declaring every slot in the entry block, which runs
    /// exactly once per call, keeps frame size a property of the function
    /// rather than of the iteration count. Stores stay at the use site.
    entry_prelude: String,
    temporary: u32,
    /// The module's outlined thunks, shared by every function that hands a
    /// call out.
    parallel: &'state mut ParallelThunks,
    /// The overlap groups *this world* actualizes: the judgment's groups in
    /// the ordinary lowering, and none at all in a sequential clone.
    ///
    /// Every consumer reads this one slice and none reads `function.overlaps()`
    /// again, which is what keeps the blocks a world emits and the labels its
    /// phis name from disagreeing: a `par.done` label can be named only where
    /// the same slice caused the block to be emitted.
    overlaps: Vec<IrOverlap>,
    /// Values whose defining call is handed to a worker lane [PAR-1
    /// candidate], and the values whose definitions are the join sites that
    /// complete them.
    overlap_handed_out: HashSet<IrValueId>,
    overlap_join_sites: HashSet<IrValueId>,
    /// Source-ordered direct completion steps, keyed by the call value each
    /// step reaches.  Their wait sets contain only ordinary prior result/loan
    /// dependencies retained by lowering.
    completion_steps: HashMap<IrValueId, crate::IrCompletionStep>,
    /// Hand-outs emitted and not yet joined.
    ///
    /// Ordinarily this is empty at every terminator, because a block joins
    /// everything it handed out before it ends. A function carrying a staged
    /// loop pipeline is the exception: a block the pipeline names leaves its
    /// operations here, and every block it does not name drains them.
    handed_out: Vec<HandedOut>,
    /// The staged loop pipeline of the function being emitted, or `None`.
    ///
    /// `None` is every function today: `lower_checked` grants no pipeline yet.
    /// It is what keeps the emitted module byte-identical to one built before
    /// this machinery existed.
    pipeline: Option<&'program crate::IrCompletionPipeline>,
    /// The blocks on which the pipeline's carrying region ends: a block the
    /// pipeline does not name that some block it does name can reach in one
    /// edge. These are the drains — the loop's normal exit and every typed
    /// exit out of its prologue — and there is generally more than one.
    pipeline_drains: HashSet<IrBlockId>,
    /// For each drain, the carrying blocks that reach it without leaving the
    /// region first.
    ///
    /// This is what decides which operations one drain retires. It is a fact
    /// about the control-flow graph, so a drain retires the same window
    /// however the blocks around it happen to be numbered, and a carrying
    /// block on a path that cannot reach this drain contributes nothing to it.
    pipeline_feeders: HashMap<IrBlockId, HashSet<IrBlockId>>,
    /// Every completion hand-out the carrying region made, with the carrying
    /// block that made it.
    ///
    /// [`Self::handed_out`] is a straight-line simulation: it holds what one
    /// walk through the blocks in index order has outstanding. That is exact
    /// while every block joins what it handed out, which is every function
    /// today. A carrying region breaks it, because the region has several
    /// exits and each of them must retire the same operations — the walk
    /// reaches the first exit, empties the simulation there, and would leave
    /// the second exit emitting nothing. This is what each exit is seeded
    /// from, and the block recorded beside each hand-out is what lets a drain
    /// take the operations that actually reach it rather than everything the
    /// walk has seen so far.
    pipeline_outstanding: Vec<(IrBlockId, completion::CompletionHandedOut)>,
    /// The slot index the block being emitted addresses its ring through,
    /// where the pipeline gives it one.
    ///
    /// Completion storage addressed while this is `Some` is one element of the
    /// site's ring, chosen at run time; while it is `None` the site owns one
    /// element and its pointer is an entry-block definition, which is every
    /// function emitted before the ring existed and every block of a one-slot
    /// region.
    block_slot: Option<IrValueId>,
    /// Whether the block being emitted is one the pipeline lets end with the
    /// region's operations still in flight.
    ///
    /// It is what decides whether a hand-out emitted here reserves a ring or a
    /// single element: a carrying block is emitted once and reached once per
    /// iteration, so a site in it may own one operation per slot; a site
    /// anywhere else is reached with nothing of its own outstanding and owns
    /// one element, exactly as it did before rings existed.
    block_carries: bool,
    /// Whether any function in this module emitted a typed completion handoff.
    completion_used: &'state mut bool,
    /// The functions that have a sequential clone, when this emitter is
    /// rendering one.
    ///
    /// `None` is the ordinary lowering, which is every function of a default
    /// build and the overlapped half of a `--par` build. `Some` renders the
    /// clone world: no group is actualized, and a call to a function that also
    /// has a clone names the clone, so the world a call lands in is the world
    /// it was made from and neither ever reaches the other.
    sequential_clones: Option<&'state HashSet<u32>>,
}

/// What one function's emission shares with the rest of its module, and the
/// one choice that says which of the module's two worlds it is emitting into.
///
/// These travel together because they are all module-scope: the trap records
/// and the intrinsic declarations are collected across every function and
/// rendered once at the top, the thunks likewise, and the clone set is the same
/// set for every function of the module. Passing them as one named group keeps
/// the emitter's own arguments — the program, the function, the target — the
/// ones a reader has to think about.
struct ModuleState<'state> {
    claim_records: &'state mut Vec<Vec<u8>>,
    intrinsics: &'state mut BTreeSet<IntrinsicDeclaration>,
    parallel: &'state mut ParallelThunks,
    completion_used: &'state mut bool,
    /// `None` emits the ordinary lowering; `Some` emits the sequential clone.
    sequential_clones: Option<&'state HashSet<u32>>,
}

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    fn new(
        program: &'program IrProgram<'_, '_, '_>,
        qualification: &'program Qualification,
        function: &'program IrFunction,
        target: TargetLayout,
        module: ModuleState<'state>,
    ) -> Self {
        let ModuleState {
            claim_records,
            intrinsics,
            parallel,
            completion_used,
            sequential_clones,
        } = module;
        // A sequential clone suppresses compute hand-outs only. Target
        // completion is independent of the compute-pool choice and remains
        // active in both worlds.
        let completion_steps: HashMap<_, _> =
            if qualification.target().supports_posix_file_completion() {
                function
                    .completion_steps()
                    .iter()
                    .cloned()
                    .map(|step| (step.call(), step))
                    .collect()
            } else {
                HashMap::new()
            };
        let overlaps: Vec<IrOverlap> = function
            .overlaps()
            .iter()
            .filter_map(|overlap| {
                if overlap
                    .members()
                    .iter()
                    .any(|member| completion_steps.contains_key(member))
                    || !overlap_is_actualizable(program, function, overlap)
                    || sequential_clones.is_some()
                {
                    return None;
                }
                Some(overlap.clone())
            })
            .collect();
        let overlap_handed_out = overlaps
            .iter()
            .flat_map(|overlap| overlap.handed_out().iter().copied())
            .collect();
        let overlap_join_sites = overlaps
            .iter()
            .filter_map(crate::IrOverlap::join_site)
            .collect();
        Self {
            program,
            qualification,
            function,
            target,
            claim_records,
            intrinsics,
            incoming: Vec::new(),
            output: String::new(),
            entry_prelude: String::new(),
            temporary: 0,
            parallel,
            overlaps,
            overlap_handed_out,
            overlap_join_sites,
            completion_steps,
            handed_out: Vec::new(),
            pipeline: function.completion_pipeline(),
            pipeline_drains: pipeline_drain_blocks(function),
            pipeline_feeders: pipeline_feeder_blocks(function),
            pipeline_outstanding: Vec::new(),
            block_slot: None,
            block_carries: false,
            completion_used,
            sequential_clones,
        }
    }

    fn is_overlap_join_site(&self, value: IrValueId) -> bool {
        self.overlap_join_sites.contains(&value)
    }

    /// Whether this block may end with the pipeline's operations still
    /// outstanding.
    ///
    /// One rule decides both halves of the staged schedule: a block the
    /// pipeline names never joins, and every other block joins everything
    /// outstanding before it ends. The first gives the loop's back edge the
    /// right to carry work across it. The second is the drain — at the loop's
    /// normal exit and at every typed exit out of the prologue alike — and it
    /// needs no separate machinery, because retiring every outstanding
    /// operation in hand-out order is exactly what a block has always done.
    fn block_carries_completion(&self, block: IrBlockId) -> bool {
        self.pipeline
            .is_some_and(|pipeline| pipeline.carries(block))
    }

    /// Refuses a carrying region no exit leaves.
    ///
    /// Every block the pipeline names must reach, in some number of edges, a
    /// block it does not name. A region without that property has a path on
    /// which an accepted operation is never joined: the target would still
    /// write its result into storage the frame no longer exists to hold. It is
    /// a defect of whatever produced the descriptor, so it is refused before a
    /// line of the function is emitted rather than diagnosed by its absence.
    fn validate_pipeline(&self) -> Result<(), BackendFailure> {
        let Some(pipeline) = self.pipeline else {
            return Ok(());
        };
        let blocks = self.function.blocks();
        let mut escapes = vec![false; blocks.len()];
        for (index, escaped) in escapes.iter_mut().enumerate() {
            let id = IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
            *escaped = !pipeline.carries(id);
        }
        let mut changed = true;
        while changed {
            changed = false;
            for (index, block) in blocks.iter().enumerate() {
                if escapes[index] {
                    continue;
                }
                if block_successors(block)
                    .iter()
                    .any(|successor| escapes.get(successor.index()).copied().unwrap_or(false))
                {
                    escapes[index] = true;
                    changed = true;
                }
            }
        }
        if !escapes.iter().all(|escaped| *escaped) {
            return Err(BackendFailure::UnretiredCompletionOperation);
        }
        // A drain retires what the carrying blocks that reach it handed out,
        // and a hand-out exists to be retired only once its own block has been
        // emitted. The walk is in block-index order, so a feeder that hands
        // out an operation and is numbered at or after its own drain would
        // leave that drain emitting nothing for it — silently, because a
        // function with a pipeline is exempt from the straight-line check at
        // the end of emission, a carrying block being free to be the last one
        // emitted. That is the same defect as a region with no drain and it is
        // refused in the same place.
        //
        // A feeder that hands nothing out is not that defect: a loop's latch
        // is numbered after the typed exit its back edge reaches, and where it
        // starts no operation there is nothing for that exit to be missing.
        for (drain, feeders) in &self.pipeline_feeders {
            for feeder in feeders {
                if feeder.index() >= drain.index() && self.block_hands_out_completion(*feeder) {
                    return Err(BackendFailure::UnretiredCompletionOperation);
                }
            }
        }
        self.validate_pipeline_slots(pipeline)
    }

    /// Refuses a ring whose slots cannot be addressed where they are used.
    ///
    /// Two things have to hold, and between them they are the whole of what
    /// makes a run-time-chosen element safe.
    ///
    /// A ring has at least one element. A descriptor claiming none would
    /// reserve a zero-length array and index into it.
    ///
    /// And a slot a block names is a value of the function, of the `u64` the
    /// array is indexed with. The index is rendered straight into a
    /// `getelementptr` the block emits, so a name of the wrong width, or of no
    /// value at all, is a module that does not verify; that is refused here
    /// rather than left to a linker. Whether the value *dominates* the block
    /// naming it is trusted exactly as every other operand this emitter
    /// renders is trusted — a driver threads the slot along the edges into its
    /// region, so the value reaching a carrying block is the loop-carried
    /// parameter that dominates it.
    ///
    /// What is deliberately *not* checked here is which blocks must name a
    /// slot. A carrying block that starts no operation and retires none needs
    /// no index, and demanding one would refuse the ordinary shape where a
    /// loop's comparison and increment blocks carry nothing. The block that
    /// does reach storage is caught where it reaches it: with a ring in force,
    /// both the reservation and the element pointer refuse a block that has no
    /// slot rather than quietly handing back element zero, which is the one
    /// failure that would let two iterations share one operation record.
    fn validate_pipeline_slots(
        &self,
        pipeline: &crate::IrCompletionPipeline,
    ) -> Result<(), BackendFailure> {
        if pipeline.slots() == 0 {
            return Err(BackendFailure::MisaddressedCompletionSlot);
        }
        for index in 0..self.function.blocks().len() {
            let id = IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
            let Some(slot) = pipeline.slot_index(id) else {
                continue;
            };
            if self.value_type(slot)
                != Some(IrType::Integer {
                    width: 64,
                    signed: false,
                })
            {
                return Err(BackendFailure::MisaddressedCompletionSlot);
            }
        }
        Ok(())
    }

    /// Whether this block starts an operation a drain would have to retire.
    ///
    /// It answers the same question `emit_instruction` answers when it decides
    /// to hand a system call to a target, read from the same source-ordered
    /// steps, so the ordering rule above is checked against what the walk will
    /// actually emit.
    fn block_hands_out_completion(&self, block: IrBlockId) -> bool {
        self.function
            .blocks()
            .get(block.index())
            .is_some_and(|block| {
                block.instructions().iter().any(|instruction| {
                    matches!(
                        instruction,
                        IrInstruction::Define { result, .. }
                            if self
                                .completion_steps
                                .get(result)
                                .is_some_and(crate::IrCompletionStep::submit)
                    )
                })
            })
    }

    /// Gives a drain block exactly the operations the carrying region left in
    /// flight on the paths that reach it.
    ///
    /// Which those are is decided by the graph — the carrying blocks that
    /// reach this drain — and not by how much of the region the walk happens
    /// to have passed. The walk arrives here with the straight-line
    /// simulation, which on a branching region is both too little and too
    /// much: an operation handed out on a branch the walk has not passed is
    /// missing, and one handed out on a *sibling* branch that cannot reach
    /// this drain is present. Adding the first without removing the second
    /// would make this exit join an operation that was never started on any
    /// path through it, which is a use of storage no target ever wrote.
    ///
    /// So both halves are done here, for every drain including the first one
    /// the walk reaches. The order is the order they were handed out, so each
    /// exit retires its whole window in that order.
    fn seed_pipeline_drain(&mut self, block: IrBlockId) {
        if !self.pipeline_drains.contains(&block) {
            return;
        }
        let Some(feeders) = self.pipeline_feeders.get(&block).cloned() else {
            return;
        };
        let elsewhere: HashSet<IrValueId> = self
            .pipeline_outstanding
            .iter()
            .filter(|(carrying, _)| !feeders.contains(carrying))
            .map(|(_, carried)| carried.result())
            .collect();
        self.handed_out.retain(|pending| match pending {
            HandedOut::Completion(pending) => !elsewhere.contains(&pending.result()),
            HandedOut::Compute(_) => true,
        });
        for (carrying, pending) in self.pipeline_outstanding.clone() {
            if feeders.contains(&carrying)
                && !self.completion_operation_is_outstanding(pending.result())
            {
                self.handed_out.push(HandedOut::Completion(pending));
            }
        }
    }

    /// Records what a carrying block handed out, so every exit the block
    /// reaches can retire it.
    fn record_pipeline_handouts(&mut self, block: IrBlockId) {
        for pending in &self.handed_out {
            let HandedOut::Completion(pending) = pending else {
                continue;
            };
            if !self
                .pipeline_outstanding
                .iter()
                .any(|(_, carried)| carried.result() == pending.result())
            {
                self.pipeline_outstanding.push((block, pending.clone()));
            }
        }
    }

    /// The symbol one call names.
    ///
    /// In the clone world a call to a function that also has a clone names the
    /// clone, which is what keeps a clone's whole dynamic extent inside the
    /// world the entry selected. Everything else — including every callee with
    /// no hand-out anywhere below it — is the one copy both worlds share.
    pub(super) fn callee_symbol(&self, ordinal: u32, name: &str) -> String {
        match self.sequential_clones {
            Some(clones) if clones.contains(&ordinal) => sequential_clone_symbol(name),
            _ => source_symbol(name),
        }
    }

    fn emit(mut self) -> Result<String, BackendFailure> {
        self.validate_pipeline()?;
        self.incoming = self.collect_incoming()?;
        let symbol = match self.sequential_clones {
            Some(_) => sequential_clone_symbol(self.function.name()),
            None => source_symbol(self.function.name()),
        };
        write!(
            self.output,
            "define internal {} @{symbol}(",
            llvm_type(self.program, self.function.result())?,
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        for (index, (value, ty)) in self.function.parameters().iter().enumerate() {
            if index != 0 {
                self.output.push_str(", ");
            }
            write!(
                self.output,
                "{} {}",
                llvm_type(self.program, *ty)?,
                self.value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        self.output.push_str(") {\n");
        let mut prelude_anchor = None;
        for (index, block) in self.function.blocks().iter().enumerate() {
            let block_id =
                IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
            writeln!(self.output, "{}:", block_label(block_id))
                .map_err(|_| BackendFailure::TextEmission)?;
            if index == 0 {
                prelude_anchor = Some(self.output.len());
            }
            self.emit_block_parameters(block_id, block)?;
            self.block_slot = self
                .pipeline
                .and_then(|pipeline| pipeline.slot_index(block_id));
            self.block_carries = self
                .pipeline
                .is_some_and(|pipeline| pipeline.carries(block_id));
            self.emit_completion_window(block_id)?;
            self.seed_pipeline_drain(block_id);
            for (instruction_index, instruction) in block.instructions().iter().enumerate() {
                self.emit_instruction(block_id, instruction_index, instruction)?;
            }
            self.emit_terminator(block_id, block.terminator())?;
        }
        // Every operation this function handed to a target is joined by some
        // block. Without a pipeline that is the straight-line fact this
        // emitter has always kept: nothing survives its own block's
        // terminator. With one it is the reachability fact checked before the
        // first block was emitted, and the straight-line count says nothing
        // because a carrying block may well be the last one emitted.
        if self.pipeline.is_none()
            && self
                .handed_out
                .iter()
                .any(|pending| matches!(pending, HandedOut::Completion(_)))
        {
            return Err(BackendFailure::UnretiredCompletionOperation);
        }
        self.output.push_str("}\n\n");
        if !self.entry_prelude.is_empty() {
            let anchor = prelude_anchor.ok_or(BackendFailure::InvalidIr)?;
            self.output.insert_str(anchor, &self.entry_prelude);
        }
        Ok(self.output)
    }

    fn collect_incoming(&self) -> Result<Vec<Vec<Incoming>>, BackendFailure> {
        let mut incoming = vec![Vec::new(); self.function.blocks().len()];
        for (index, block) in self.function.blocks().iter().enumerate() {
            if let IrTerminator::Jump {
                target, arguments, ..
            } = block.terminator()
            {
                let predecessor =
                    IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
                incoming
                    .get_mut(target.index())
                    .ok_or(BackendFailure::InvalidIr)?
                    .push(Incoming {
                        predecessor,
                        arguments: arguments.clone(),
                    });
            }
        }
        Ok(incoming)
    }

    fn emit_block_parameters(
        &mut self,
        block_id: IrBlockId,
        block: &IrBlock,
    ) -> Result<(), BackendFailure> {
        if block.parameters().is_empty() {
            return Ok(());
        }
        let incoming = self
            .incoming
            .get(block_id.index())
            .ok_or(BackendFailure::InvalidIr)?;
        if incoming.is_empty()
            || incoming
                .iter()
                .any(|edge| edge.arguments.len() != block.parameters().len())
        {
            return Err(BackendFailure::InvalidIr);
        }
        for (parameter_index, (parameter, ty)) in block.parameters().iter().enumerate() {
            write!(
                self.output,
                "  {} = phi {} ",
                self.value_name(*parameter),
                llvm_type(self.program, *ty)?
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            for (edge_index, edge) in incoming.iter().enumerate() {
                let argument = *edge
                    .arguments
                    .get(parameter_index)
                    .ok_or(BackendFailure::InvalidIr)?;
                if self.value_type(argument) != Some(*ty) {
                    return Err(BackendFailure::InvalidIr);
                }
                if edge_index != 0 {
                    self.output.push_str(", ");
                }
                write!(
                    self.output,
                    "[ {}, %{} ]",
                    self.value_name(argument),
                    block_exit_label(
                        edge.predecessor,
                        self.block(edge.predecessor)?,
                        &self.overlaps,
                        &self.completion_steps
                    )
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            self.output.push('\n');
        }
        Ok(())
    }

    fn emit_instruction(
        &mut self,
        block: IrBlockId,
        index: usize,
        instruction: &IrInstruction,
    ) -> Result<(), BackendFailure> {
        if let IrInstruction::Define {
            result,
            ty,
            operation,
        } = instruction
            && let Some(step) = self.completion_steps.get(result).cloned()
        {
            self.emit_completion_dependencies(step.wait_for())?;
            if step.submit() {
                let IrOperation::SystemCall {
                    operation,
                    target_action,
                    arguments,
                } = operation
                else {
                    return Err(BackendFailure::InvalidIr);
                };
                if !target_action.may_suspend() {
                    return Err(BackendFailure::InvalidIr);
                }
                self.emit_handed_out_system_call(*result, *ty, *operation, arguments)?;
            } else {
                self.emit_definition(*result, *ty, operation)?;
            }
            // A carrying block never joins: the schedule's last member ends
            // it with the operations still owned by the target, and every
            // block the pipeline does not name retires them.
            if self.block_carries_completion(block) {
                self.record_pipeline_handouts(block);
            } else if step.finish() {
                self.emit_all_completion_joins()?;
            }
            return Ok(());
        }
        // A group's join rides the definition of its last member: the members
        // before it were handed out and their values do not exist until here.
        if let IrInstruction::Define { result, .. } = instruction
            && self.is_overlap_join_site(*result)
        {
            self.emit_definition_then_join(instruction, *result)?;
            return Ok(());
        }
        match instruction {
            IrInstruction::Define {
                result,
                ty,
                operation,
            } => self.emit_definition(*result, *ty, operation),
            IrInstruction::Claim { condition, site } => {
                if self.value_type(*condition) != Some(IrType::Bool) {
                    return Err(BackendFailure::InvalidIr);
                }
                let claim_id = self.register_claim(site)?;
                writeln!(
                    self.output,
                    "  br i1 {}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{claim_id}, i64 {})\n  unreachable\n{}:",
                    self.value_name(*condition),
                    claim_continue_label(block, index),
                    claim_trap_label(block, index),
                    claim_trap_label(block, index),
                    self.claim_records[claim_id].len(),
                    claim_continue_label(block, index)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            IrInstruction::StoreBuffer {
                buffer,
                index,
                value,
            } => self.emit_buffer_store(*buffer, *index, *value),
            IrInstruction::Store {
                address,
                value,
                referent,
            } => self.emit_store(*address, *value, *referent),
            IrInstruction::Drop(drop) => self.emit_drop(*drop),
        }
    }

    /// Emits the last member of an overlap group and then its joins.
    fn emit_definition_then_join(
        &mut self,
        instruction: &IrInstruction,
        result: IrValueId,
    ) -> Result<(), BackendFailure> {
        let IrInstruction::Define { ty, operation, .. } = instruction else {
            return Err(BackendFailure::InvalidIr);
        };
        self.emit_definition(result, *ty, operation)?;
        self.emit_overlap_joins(result)
    }

    fn emit_definition(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: &IrOperation,
    ) -> Result<(), BackendFailure> {
        if self.value_type(result) != Some(ty) {
            return Err(BackendFailure::InvalidIr);
        }
        match operation {
            IrOperation::Constant(constant) => self.emit_constant(result, ty, *constant),
            IrOperation::Call {
                function,
                arguments,
            } => {
                if self.overlap_handed_out.contains(&result) {
                    self.emit_handed_out_call(result, ty, *function, arguments)
                } else {
                    self.emit_call(result, ty, *function, arguments)
                }
            }
            IrOperation::LoopSplit {
                splitter,
                chunk,
                seed,
                lower,
                upper,
                captures,
                weight,
            } => self.emit_loop_split(
                result,
                ty,
                &LoopSplitSite {
                    splitter: *splitter,
                    chunk: *chunk,
                    seed: *seed,
                    lower: *lower,
                    upper: *upper,
                    captures,
                    weight: *weight,
                },
            ),
            IrOperation::SystemCall {
                operation,
                arguments,
                ..
            } => {
                if self.overlap_handed_out.contains(&result) {
                    self.emit_handed_out_system_call(result, ty, *operation, arguments)
                } else {
                    self.emit_system_call(result, ty, *operation, arguments)
                }
            }
            IrOperation::Integer {
                operation,
                operand_type,
                arguments,
            } => self.emit_integer(result, ty, *operation, *operand_type, arguments),
            IrOperation::Float {
                operation,
                operand_type,
                arguments,
            } => self.emit_float(result, ty, *operation, *operand_type, arguments),
            IrOperation::NumericConversion {
                source_type,
                destination_type,
                value,
            } => self.emit_numeric_conversion(result, ty, *source_type, *destination_type, *value),
            IrOperation::Reinterpret {
                source_type,
                destination_type,
                value,
            } => self.emit_reinterpret(result, ty, *source_type, *destination_type, *value),
            IrOperation::Boolean {
                operation,
                arguments,
            } => self.emit_boolean(result, ty, *operation, arguments),
            IrOperation::EnumEquality {
                equal,
                operand_type,
                arguments,
            } => self.emit_enum_equality(result, ty, *equal, *operand_type, *arguments),
            IrOperation::ArrayFill {
                value,
                target_domain,
            } => self.emit_array_fill(result, ty, *value, *target_domain),
            IrOperation::ArrayIndex {
                root,
                offset,
                target_domain,
            } => self.emit_array_index(result, ty, *root, *offset, *target_domain),
            IrOperation::InsertArray {
                aggregate,
                index,
                value,
            } => self.emit_array_insertion(result, ty, *aggregate, *index, *value),
            IrOperation::BufferFill {
                length,
                value,
                layout_ceiling,
                target_domains,
            } => self.emit_buffer_fill(
                result,
                ty,
                *length,
                *value,
                *layout_ceiling,
                *target_domains,
            ),
            IrOperation::BufferVacant {
                length,
                layout_ceiling,
                target_domains,
            } => self.emit_buffer_vacant(result, ty, *length, *layout_ceiling, *target_domains),
            IrOperation::BufferFits {
                length,
                maximum_length,
            } => self.emit_buffer_fits(result, ty, *length, *maximum_length),
            IrOperation::BufferLength { buffer } => self.emit_buffer_length(result, ty, *buffer),
            IrOperation::BufferIndex {
                buffer,
                offset,
                target_domain,
            } => self.emit_buffer_index(result, ty, *buffer, *offset, *target_domain),
            IrOperation::BufferProbeSkip {
                buffer,
                index,
                limit,
                needles,
            } => self.emit_buffer_probe_skip(result, ty, *buffer, *index, *limit, needles),
            IrOperation::SliceFromArray { array } => self.emit_slice_from_array(result, ty, *array),
            IrOperation::SliceFromBuffer { buffer } => {
                self.emit_slice_from_buffer(result, ty, *buffer)
            }
            IrOperation::SliceLength { slice } => self.emit_slice_length(result, ty, *slice),
            IrOperation::SliceIndex {
                slice,
                offset,
                target_domain,
            } => self.emit_slice_index(result, ty, *slice, *offset, *target_domain),
            IrOperation::BoxNew { nominal, value } => {
                self.emit_box_new(result, ty, *nominal, *value)
            }
            IrOperation::BoxDeref { nominal, value } => {
                self.emit_box_deref(result, ty, *nominal, *value)
            }
            IrOperation::ArenaListNew => self.emit_arena_list_new(result, ty),
            IrOperation::ArenaNew {
                nominal,
                list,
                value,
            } => self.emit_arena_new(result, ty, *nominal, *list, *value),
            IrOperation::ArenaDeref { nominal, value } => {
                self.emit_arena_deref(result, ty, *nominal, *value)
            }
            IrOperation::ConstructStruct { nominal, fields } => {
                self.emit_struct(result, ty, *nominal, fields)
            }
            IrOperation::ConstructEnum {
                nominal,
                variant,
                fields,
            } => self.emit_enum(result, ty, *nominal, *variant, fields),
            IrOperation::ProjectStruct {
                aggregate,
                nominal,
                field,
                consume_root,
            } => {
                self.emit_struct_projection(result, ty, *aggregate, *nominal, *field, *consume_root)
            }
            IrOperation::InsertStruct {
                aggregate,
                nominal,
                field,
                value,
            } => self.emit_struct_insertion(result, ty, *aggregate, *nominal, *field, *value),
            IrOperation::ProjectVariant {
                aggregate,
                nominal,
                variant,
                field,
            } => self.emit_variant_projection(result, ty, *aggregate, *nominal, *variant, *field),
            IrOperation::AddressOf { value, referent } => {
                self.emit_address_of(result, ty, *value, *referent)
            }
            IrOperation::Load { address, referent } => {
                self.emit_load(result, ty, *address, *referent)
            }
        }
    }

    /// Asks the runtime for this loop's window, once at its entry block.
    ///
    /// The precedent is `wf__par_split_budget`, asked once per loop entry and
    /// never per iteration. The three arguments are bounds the compiler
    /// already knows — the trip count where it is known, the private storage
    /// one in-flight iteration owns, and the compiler's own static cap from
    /// that storage's cost — and the runtime answers from its own capacity.
    /// One is always a legal answer and reproduces the sequential program, so
    /// this query can never make a correct program fail.
    ///
    /// There is no environment variable, attribute, or source spelling for the
    /// answer. The writer never sees it.
    fn emit_completion_window(&mut self, block: IrBlockId) -> Result<(), BackendFailure> {
        let Some(pipeline) = self.pipeline.filter(|pipeline| pipeline.entry() == block) else {
            return Ok(());
        };
        let window = pipeline.window();
        let name = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{name} = call i64 @wf__completion_window(i64 {}, i64 {}, i64 {})",
            window.span(),
            window.slot_bytes(),
            window.ceiling()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        *self.completion_used = true;
        Ok(())
    }

    fn emit_terminator(
        &mut self,
        block: IrBlockId,
        terminator: &IrTerminator,
    ) -> Result<(), BackendFailure> {
        if !self.block_carries_completion(block) {
            self.emit_all_completion_joins()?;
        }
        match terminator {
            IrTerminator::Unreachable => {
                writeln!(self.output, "  unreachable").map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Jump {
                target,
                arguments,
                drops,
            } => {
                let target_block = self.block(*target)?;
                if target_block.parameters().len() != arguments.len() {
                    return Err(BackendFailure::InvalidIr);
                }
                for (argument, (_, ty)) in arguments.iter().zip(target_block.parameters()) {
                    if self.value_type(*argument) != Some(*ty) {
                        return Err(BackendFailure::InvalidIr);
                    }
                }
                self.emit_drops(drops)?;
                writeln!(self.output, "  br label %{}", block_label(*target))
                    .map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Return { value, drops } => {
                if self.value_type(*value) != Some(self.function.result()) {
                    return Err(BackendFailure::InvalidIr);
                }
                self.emit_drops(drops)?;
                writeln!(
                    self.output,
                    "  ret {} {}",
                    llvm_type(self.program, self.function.result())?,
                    self.value_name(*value)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Match {
                scrutinee,
                enum_type,
                targets,
            } => {
                let (tag, tag_ty) = self.match_tag(*scrutinee, *enum_type)?;
                writeln!(
                    self.output,
                    "  switch {tag_ty} {tag}, label %{} [",
                    invalid_tag_label(block)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                let mut seen = BTreeSet::new();
                for target in targets {
                    if !seen.insert(target.tag()) {
                        return Err(BackendFailure::InvalidIr);
                    }
                    writeln!(
                        self.output,
                        "    {tag_ty} {}, label %{}",
                        target.tag(),
                        block_label(target.block())
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                }
                writeln!(
                    self.output,
                    "  ]\n{}:\n  call void @abort()\n  unreachable",
                    invalid_tag_label(block)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
        }
    }

    fn match_tag(
        &mut self,
        scrutinee: IrValueId,
        enum_type: IrEnumType,
    ) -> Result<(String, String), BackendFailure> {
        match enum_type {
            IrEnumType::Bool => {
                if self.value_type(scrutinee) != Some(IrType::Bool) {
                    return Err(BackendFailure::InvalidIr);
                }
                Ok((self.value_name(scrutinee), "i1".to_owned()))
            }
            IrEnumType::Nominal(nominal) => {
                if self.value_type(scrutinee) != Some(IrType::Nominal(nominal)) {
                    return Err(BackendFailure::InvalidIr);
                }
                let data = self.nominal(nominal)?;
                let IrNominalKind::Enum { .. } = data.kind() else {
                    return Err(BackendFailure::InvalidIr);
                };
                if data.is_tag_only_enum() {
                    return Ok((
                        self.value_name(scrutinee),
                        llvm_type(self.program, IrType::Nominal(nominal))?,
                    ));
                }
                let temporary = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{temporary} = extractvalue {} {}, 0",
                    llvm_type(self.program, IrType::Nominal(nominal))?,
                    self.value_name(scrutinee)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                Ok((format!("%{temporary}"), "i32".to_owned()))
            }
        }
    }

    fn nominal(&self, id: IrNominalId) -> Result<&IrNominal, BackendFailure> {
        self.program.nominal(id).ok_or(BackendFailure::InvalidIr)
    }

    fn block(&self, id: IrBlockId) -> Result<&IrBlock, BackendFailure> {
        self.function
            .blocks()
            .get(id.index())
            .ok_or(BackendFailure::InvalidIr)
    }

    fn register_claim(&mut self, site: &IrClaimSite) -> Result<usize, BackendFailure> {
        let index = self.claim_records.len();
        let _ = u32::try_from(index).map_err(|_| BackendFailure::CounterOverflow)?;
        self.claim_records.push(trap_record(site));
        Ok(index)
    }

    fn emit_drop(&mut self, drop: IrDrop) -> Result<(), BackendFailure> {
        if self.value_type(drop.value()) != Some(drop.ty()) {
            return Err(BackendFailure::InvalidIr);
        }
        let value_name = self.value_name(drop.value());
        match drop.ty() {
            IrType::Array { .. } | IrType::Slice { .. } => {}
            IrType::Buffer { .. } => {
                emit_value_cleanup(
                    self.program,
                    self.qualification,
                    &mut self.output,
                    &mut self.temporary,
                    drop.ty(),
                    value_name.clone(),
                )?;
            }
            IrType::Nominal(nominal) if !self.nominal(nominal)?.is_tag_only_enum() => {
                match self.nominal(nominal)?.kind() {
                    IrNominalKind::Struct { .. } => {}
                    // An arena value derives no owner-scope drop action; its
                    // storage is released with its region [STOR-3, STOR-4].
                    IrNominalKind::Arena { .. } => {}
                    // The region's allocation-list drop is that release:
                    // walk the list and free every registered allocation.
                    IrNominalKind::ArenaStorage => {
                        emit_value_cleanup(
                            self.program,
                            self.qualification,
                            &mut self.output,
                            &mut self.temporary,
                            drop.ty(),
                            value_name.clone(),
                        )?;
                    }
                    // The checked program's own [SYS-5] record is the single
                    // source of truth for which action runs here, so a table
                    // row disagreeing with it stops rather than silently
                    // emitting a different release.
                    IrNominalKind::SystemResource(contract) => {
                        if drop.release().action != Some(contract.action) {
                            return Err(BackendFailure::InvalidIr);
                        }
                        let contract = *contract;
                        system::emit_resource_release(
                            self.qualification,
                            &mut self.output,
                            &mut self.temporary,
                            contract,
                            &value_name,
                        )?;
                    }
                    IrNominalKind::Enum { .. } | IrNominalKind::Box { .. } => {
                        if type_requires_cleanup(self.program, drop.ty())? {
                            emit_value_cleanup(
                                self.program,
                                self.qualification,
                                &mut self.output,
                                &mut self.temporary,
                                drop.ty(),
                                value_name.clone(),
                            )?;
                        }
                    }
                }
            }
            _ => return Err(BackendFailure::InvalidIr),
        }
        writeln!(self.output, "  ; drop {value_name}").map_err(|_| BackendFailure::TextEmission)
    }

    fn emit_drops(&mut self, drops: &[IrDrop]) -> Result<(), BackendFailure> {
        for drop in drops {
            self.emit_drop(*drop)?;
        }
        Ok(())
    }

    /// Declares a stack slot under a caller-chosen name in the entry block.
    ///
    /// The name must be an otherwise undefined value name, because the slot
    /// declaration becomes that name's single definition.
    fn declare_entry_slot(&mut self, name: &str, ty: &str) -> Result<(), BackendFailure> {
        writeln!(self.entry_prelude, "  {name} = alloca {ty}")
            .map_err(|_| BackendFailure::TextEmission)
    }

    /// Reserves a fresh stack slot in the entry block and returns its name.
    ///
    /// Each call reserves a distinct slot, so two slots never alias even when
    /// they hold values of the same type and live at the same time.
    fn entry_slot(&mut self, ty: &str) -> Result<String, BackendFailure> {
        let name = format!("%{}", self.next_temporary()?);
        self.declare_entry_slot(&name, ty)?;
        Ok(name)
    }

    /// Reserves `count` stack slots of one type in the entry block and returns
    /// the name of element `index`.
    ///
    /// The storage and the element pointer are both entry-block definitions,
    /// so the returned name dominates every block of the function exactly as a
    /// plain [`Self::entry_slot`] name does and can be used wherever one was.
    fn indexed_entry_slot(
        &mut self,
        ty: &str,
        count: u64,
        index: u64,
    ) -> Result<String, BackendFailure> {
        if index >= count {
            return Err(BackendFailure::InvalidIr);
        }
        let storage = self.entry_slot(&format!("[{count} x {ty}]"))?;
        let element = format!("%{}", self.next_temporary()?);
        writeln!(
            self.entry_prelude,
            "  {element} = getelementptr inbounds [{count} x {ty}], ptr {storage}, i64 0, \
             i64 {index}"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(element)
    }

    fn next_temporary(&mut self) -> Result<String, BackendFailure> {
        let current = self.temporary;
        self.temporary = self
            .temporary
            .checked_add(1)
            .ok_or(BackendFailure::CounterOverflow)?;
        Ok(format!("t{current}"))
    }

    fn value_type(&self, value: IrValueId) -> Option<IrType> {
        self.function.value_type(value)
    }

    fn value_name(&self, value: IrValueId) -> String {
        value_name(value)
    }
}

/// Whether every member which would leave the calling thread has an execution
/// route that preserves its target-action contract.  A possibly-suspending
/// Whitefoot wrapper is intentionally refused until selective stackless frames
/// exist; only its direct compiler-owned file operation can enter completion.
fn overlap_is_actualizable(
    program: &IrProgram<'_, '_, '_>,
    function: &IrFunction,
    overlap: &IrOverlap,
) -> bool {
    if overlap
        .members()
        .iter()
        .any(|member| match definition_operation(function, *member) {
            Some(IrOperation::Call {
                function: callee, ..
            }) => program
                .functions()
                .get(*callee as usize)
                .is_none_or(|callee| callee.target_action().may_suspend()),
            Some(IrOperation::SystemCall { target_action, .. }) => target_action.may_suspend(),
            _ => false,
        })
    {
        return false;
    }
    overlap
        .handed_out()
        .iter()
        .all(|member| match definition_operation(function, *member) {
            Some(IrOperation::Call { function, .. }) => program
                .functions()
                .get(*function as usize)
                .is_some_and(|callee| !callee.target_action().may_suspend()),
            _ => false,
        })
}

fn definition_operation(function: &IrFunction, value: IrValueId) -> Option<&IrOperation> {
    function.blocks().iter().find_map(|block| {
        block
            .instructions()
            .iter()
            .find_map(|instruction| match instruction {
                IrInstruction::Define {
                    result, operation, ..
                } if *result == value => Some(operation),
                _ => None,
            })
    })
}

fn llvm_type(program: &IrProgram<'_, '_, '_>, ty: IrType) -> Result<String, BackendFailure> {
    match ty {
        IrType::Unit => Ok("i8".to_owned()),
        IrType::Bool => Ok("i1".to_owned()),
        IrType::Integer { width: 8, .. } => Ok("i8".to_owned()),
        IrType::Integer { width: 16, .. } => Ok("i16".to_owned()),
        IrType::Integer { width: 32, .. } => Ok("i32".to_owned()),
        IrType::Integer { width: 64, .. } => Ok("i64".to_owned()),
        IrType::Integer { .. } => Err(BackendFailure::InvalidIr),
        IrType::Float { width: 32 } => Ok("float".to_owned()),
        IrType::Float { width: 64 } => Ok("double".to_owned()),
        IrType::Float { .. } => Err(BackendFailure::InvalidIr),
        IrType::Array { length: 0, .. } => Ok("[0 x i8]".to_owned()),
        IrType::Array { element, length } => Ok(format!(
            "[{length} x {}]",
            llvm_type(program, element.ty())?
        )),
        IrType::Buffer { .. } | IrType::Slice { .. } => Ok("{ ptr, i64 }".to_owned()),
        IrType::Address(_) => Ok("ptr".to_owned()),
        IrType::Nominal(id) => {
            let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
            if matches!(
                nominal.kind(),
                IrNominalKind::Box { .. }
                    | IrNominalKind::Arena { .. }
                    | IrNominalKind::ArenaStorage
            ) {
                return Ok("ptr".to_owned());
            }
            // [QUAL-1] fixes an opaque resource's representation in its
            // qualification record. Emission is reached only after
            // qualification accepted the program, so the row this reads is the
            // one qualification already resolved for this resource.
            if let IrNominalKind::SystemResource(contract) = nominal.kind() {
                return Ok(qualified_representation(contract.resource)
                    .llvm()
                    .to_owned());
            }
            if nominal.is_tag_only_enum() {
                let IrNominalKind::Enum { variants } = nominal.kind() else {
                    return Err(BackendFailure::InvalidIr);
                };
                Ok(if variants.len() <= 2 { "i1" } else { "i32" }.to_owned())
            } else {
                Ok(nominal_symbol(id))
            }
        }
    }
}

fn is_tag_only_type(program: &IrProgram<'_, '_, '_>, ty: IrType) -> Result<bool, BackendFailure> {
    match ty {
        IrType::Bool => Ok(true),
        IrType::Nominal(id) => program
            .nominal(id)
            .map(IrNominal::is_tag_only_enum)
            .ok_or(BackendFailure::InvalidIr),
        _ => Ok(false),
    }
}

fn constant_operand(constant: IrConstant, ty: IrType) -> Result<String, BackendFailure> {
    match (constant, ty) {
        (IrConstant::Unit, IrType::Unit) => Ok("0".to_owned()),
        (IrConstant::Bool(value), IrType::Bool) => Ok(u8::from(value).to_string()),
        (
            IrConstant::Integer {
                ty: constant_ty,
                bits,
            },
            actual_ty,
        ) if constant_ty == actual_ty => {
            let IrType::Integer { width, signed } = actual_ty else {
                return Err(BackendFailure::InvalidIr);
            };
            if !matches!(width, 8 | 16 | 32 | 64) {
                return Err(BackendFailure::InvalidIr);
            }
            let mask = if width == 64 {
                u64::MAX
            } else {
                (1_u64 << width) - 1
            };
            let bits = bits & mask;
            Ok(if signed && bits & (1_u64 << (width - 1)) != 0 {
                (i128::from(bits) - (1_i128 << width)).to_string()
            } else {
                bits.to_string()
            })
        }
        (
            IrConstant::Float {
                ty: constant_ty,
                bits,
            },
            actual_ty,
        ) if constant_ty == actual_ty => match actual_ty {
            IrType::Float { width: 32 } => {
                let bits = u32::try_from(bits).map_err(|_| BackendFailure::InvalidIr)?;
                let widened = f64::from(f32::from_bits(bits)).to_bits();
                Ok(format!("0x{widened:016X}"))
            }
            IrType::Float { width: 64 } => Ok(format!("0x{bits:016X}")),
            _ => Err(BackendFailure::InvalidIr),
        },
        _ => Err(BackendFailure::InvalidIr),
    }
}

fn variant_field_base(
    variants: &[crate::IrVariant],
    selected: u32,
) -> Result<usize, BackendFailure> {
    let mut index = 1_usize;
    for variant in variants {
        if variant.tag() == selected {
            return Ok(index);
        }
        index = index
            .checked_add(variant.fields().len())
            .ok_or(BackendFailure::CounterOverflow)?;
    }
    Err(BackendFailure::InvalidIr)
}

/// The last handed-out member of the overlap group `result` joins, if it is a
/// join site at all. Its `par.done` block is where the block continues.
///
/// `overlaps` is the emitting world's set, never `IrFunction::overlaps`: a
/// clone actualizes nothing, so it emits no `par.done` block and must not name
/// one either.
fn overlap_join_tail(overlaps: &[IrOverlap], result: IrValueId) -> Option<IrValueId> {
    overlaps
        .iter()
        .find(|overlap| overlap.join_site() == Some(result))
        .and_then(|overlap| overlap.handed_out().last().copied())
}

/// Where a block's terminator is actually emitted.
///
/// Emission is one pass over the blocks in order, so a block's phis are written
/// before the blocks that reach it. A phi therefore has to name the label an
/// incoming block *will* end at, and every operation that opens a new LLVM
/// block moves that label away from the plain `bbN` header. This function is
/// the one model of that: it replays a block's instructions and reports the
/// label the terminator lands in.
///
/// The two hand-out mechanisms both leave the block somewhere else. A compute
/// overlap settles on its group's `par.done` when its join site runs. A direct
/// completion step submits into `completion.offered` and is joined later, and
/// its join settles on that operation's own `par.done`; whatever is still
/// outstanding when the block ends is joined by `emit_terminator` before the
/// terminator itself, so the replay drains the same queue in the same order
/// that `emit_completion_dependencies` does.
fn block_exit_label(
    block_id: IrBlockId,
    block: &IrBlock,
    overlaps: &[IrOverlap],
    completion_steps: &HashMap<IrValueId, IrCompletionStep>,
) -> String {
    let mut label = block_label(block_id);
    // The direct completion hand-outs this block has submitted and not yet
    // joined, in `FunctionEmitter::handed_out` order.
    let mut outstanding: Vec<IrValueId> = Vec::new();
    for (index, instruction) in block.instructions().iter().enumerate() {
        // `emit_instruction` checks the completion step first and returns, so
        // a step's call never reaches the compute-overlap join below.
        if let IrInstruction::Define { result, .. } = instruction
            && let Some(step) = completion_steps.get(result)
        {
            drain_completions(&mut outstanding, step.wait_for(), &mut label);
            if step.submit() {
                outstanding.push(*result);
                label = completion_offered_label(*result);
            } else {
                definition_exit_label(block_id, index, instruction, &mut label);
            }
            if step.finish() {
                drain_all_completions(&mut outstanding, &mut label);
            }
            continue;
        }
        definition_exit_label(block_id, index, instruction, &mut label);
        // The overlap join rides its last member's own emission, so it settles
        // the label after whatever that emission left.
        if let IrInstruction::Define { result, .. } = instruction
            && let Some(last) = overlap_join_tail(overlaps, *result)
        {
            label = par_done_label(last);
        }
    }
    // `emit_terminator` joins every remaining hand-out before the terminator.
    drain_all_completions(&mut outstanding, &mut label);
    label
}

/// Replays `emit_completion_dependencies`: each named operation still
/// outstanding is joined, and each join leaves the block at its `par.done`.
fn drain_completions(outstanding: &mut Vec<IrValueId>, wanted: &[IrValueId], label: &mut String) {
    for value in wanted {
        if let Some(position) = outstanding.iter().position(|held| held == value) {
            outstanding.remove(position);
            *label = par_done_label(*value);
        }
    }
}

/// Replays `emit_all_completion_joins`, which drains in hand-out order and so
/// leaves the block at the last hand-out's `par.done`.
fn drain_all_completions(outstanding: &mut Vec<IrValueId>, label: &mut String) {
    if let Some(last) = outstanding.last() {
        *label = par_done_label(*last);
    }
    outstanding.clear();
}

/// The label one ordinary instruction's own emission leaves the block at, for
/// the operations whose lowering opens a further LLVM block.
fn definition_exit_label(
    block_id: IrBlockId,
    index: usize,
    instruction: &IrInstruction,
    label: &mut String,
) {
    match instruction {
        IrInstruction::Claim { .. } => *label = claim_continue_label(block_id, index),
        IrInstruction::Define {
            result,
            operation:
                IrOperation::Integer {
                    operation:
                        IrIntegerOperation::DivideChecked | IrIntegerOperation::RemainderChecked,
                    ..
                },
            ..
        } => *label = integer_continue_label(*result),
        IrInstruction::Define {
            result,
            operation: IrOperation::ArrayFill { .. },
            ..
        } => *label = array_fill_done_label(*result),
        IrInstruction::Define {
            result,
            operation: IrOperation::BoxNew { .. },
            ..
        } => *label = box_new_ready_label(*result),
        IrInstruction::Define {
            result,
            operation: IrOperation::ArenaNew { .. },
            ..
        } => *label = arena_new_ready_label(*result),
        IrInstruction::Define {
            result,
            operation: IrOperation::BufferFill { .. },
            ..
        } => *label = buffer_fill_done_label(*result),
        IrInstruction::Define {
            result,
            operation: IrOperation::BufferVacant { .. },
            ..
        } => *label = buffer_vacant_done_label(*result),
        IrInstruction::Define {
            result,
            operation: IrOperation::BufferProbeSkip { .. },
            ..
        } => *label = buffer_probe_join_label(*result),
        _ => {}
    }
}

/// The blocks one block's terminator can transfer control to.
fn block_successors(block: &IrBlock) -> Vec<IrBlockId> {
    match block.terminator() {
        IrTerminator::Jump { target, .. } => vec![*target],
        IrTerminator::Match { targets, .. } => {
            targets.iter().map(|target| target.block()).collect()
        }
        IrTerminator::Return { .. } | IrTerminator::Unreachable => Vec::new(),
    }
}

/// Where a staged loop pipeline's carrying region ends.
///
/// A block the pipeline does not name, reached in one edge from a block it
/// does, is an exit from the region and therefore a drain: it retires every
/// operation the region left outstanding. The loop's normal exit is one of
/// these and every typed exit out of the body is another, so this is a set and
/// not a block.
fn pipeline_drain_blocks(function: &IrFunction) -> HashSet<IrBlockId> {
    let Some(pipeline) = function.completion_pipeline() else {
        return HashSet::new();
    };
    function
        .blocks()
        .iter()
        .enumerate()
        .filter_map(|(index, block)| {
            let id = IrBlockId::from_index(index).ok()?;
            pipeline.carries(id).then(|| block_successors(block))
        })
        .flatten()
        .filter(|successor| !pipeline.carries(*successor))
        .collect()
}

/// Which carrying blocks each drain retires the work of.
///
/// A drain must retire exactly the operations the region can still have in
/// flight when control arrives there, and those are the ones handed out by the
/// carrying blocks that reach it without leaving the region on the way — a
/// fact about the edges, not about the order the blocks are numbered in.
/// Reading it from the graph is what keeps a drain from retiring work that
/// cannot reach it and, together with the ordering rule `validate_pipeline`
/// applies, from missing work that can.
fn pipeline_feeder_blocks(function: &IrFunction) -> HashMap<IrBlockId, HashSet<IrBlockId>> {
    let Some(pipeline) = function.completion_pipeline() else {
        return HashMap::new();
    };
    let blocks = function.blocks();
    let mut predecessors: Vec<Vec<IrBlockId>> = vec![Vec::new(); blocks.len()];
    for (index, block) in blocks.iter().enumerate() {
        let Ok(id) = IrBlockId::from_index(index) else {
            continue;
        };
        for successor in block_successors(block) {
            if let Some(edges) = predecessors.get_mut(successor.index()) {
                edges.push(id);
            }
        }
    }
    let carrying_predecessors = |block: IrBlockId| -> Vec<IrBlockId> {
        predecessors
            .get(block.index())
            .map(|edges| {
                edges
                    .iter()
                    .copied()
                    .filter(|edge| pipeline.carries(*edge))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut feeders = HashMap::new();
    for drain in pipeline_drain_blocks(function) {
        let mut reached: HashSet<IrBlockId> = HashSet::new();
        let mut frontier = carrying_predecessors(drain);
        while let Some(block) = frontier.pop() {
            if !reached.insert(block) {
                continue;
            }
            frontier.extend(carrying_predecessors(block));
        }
        feeders.insert(drain, reached);
    }
    feeders
}

fn block_label(block: IrBlockId) -> String {
    if block.ordinal() == 0 {
        "entry".to_owned()
    } else {
        format!("bb{}", block.ordinal())
    }
}

fn value_name(value: IrValueId) -> String {
    format!("%v{}", value.ordinal())
}

fn nominal_symbol(nominal: IrNominalId) -> String {
    format!("%wf.t{}", nominal.ordinal())
}

fn constant_symbol(constant: crate::IrConstantId) -> String {
    format!("@.wf_const.{}", constant.ordinal())
}

fn claim_continue_label(block: IrBlockId, index: usize) -> String {
    format!("claim.cont.b{}.i{index}", block.ordinal())
}

fn claim_trap_label(block: IrBlockId, index: usize) -> String {
    format!("claim.trap.b{}.i{index}", block.ordinal())
}

fn integer_safe_label(value: IrValueId) -> String {
    format!("integer.safe.v{}", value.ordinal())
}

fn integer_error_label(value: IrValueId) -> String {
    format!("integer.error.v{}", value.ordinal())
}

fn integer_continue_label(value: IrValueId) -> String {
    format!("integer.cont.v{}", value.ordinal())
}

fn box_new_ready_label(value: IrValueId) -> String {
    format!("box.new.ready.v{}", value.ordinal())
}

fn arena_new_ready_label(value: IrValueId) -> String {
    format!("arena.new.ready.v{}", value.ordinal())
}

fn array_fill_head_label(value: IrValueId) -> String {
    format!("array.fill.head.v{}", value.ordinal())
}

fn array_fill_body_label(value: IrValueId) -> String {
    format!("array.fill.body.v{}", value.ordinal())
}

fn array_fill_done_label(value: IrValueId) -> String {
    format!("array.fill.done.v{}", value.ordinal())
}

fn invalid_tag_label(block: IrBlockId) -> String {
    format!("invalid.tag.b{}", block.ordinal())
}

fn source_symbol(name: &str) -> String {
    format!("wf_{name}")
}

/// The overlapped-world symbol a sequential clone was made from, for a caller
/// that has only the two symbols and needs to know they are one function.
///
/// It lives beside [`source_symbol`] because that is the other half of the
/// spelling: a clone is `wf__par_seq_` plus the source name where the ordinary
/// definition is `wf_` plus the same name, and a reader who has to reconstruct
/// that from two files gets it wrong.
pub(crate) fn overlapped_clone_symbol(sequential: &str) -> Option<String> {
    sequential.strip_prefix("wf__par_seq_").map(source_symbol)
}

/// The mandatory [DIAG-3] record writer of a module with one thread, reached
/// from every `claim` that evaluates false, and the sole writer-reachable
/// runtime trap [SCOPE-4].
///
/// The thread that reaches it writes its complete record to standard error and
/// aborts the process without unwinding. There is no one to arbitrate with, so
/// there is no latch: these are the bytes every module emitted before the
/// overlapped world existed, and they are what a default build still gets.
const SEQUENTIAL_TRAP_WRITER: &str = "\ndefine private void @wf_trap(ptr %message, i64 %length) noreturn {\nentry:\n  br label %write.loop\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @write(i32 2, ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\n}\n\n";

/// The module's own answer for the shared record latch, and the only state the
/// trap path carries.
///
/// The latch a record writer takes is the floor runtime's, because the floor's
/// signal handler writes the stack record and this module writes every other
/// one: a latch each would leave the two classes unserialized against each
/// other, and two threads dying of different resources at once could interleave
/// two records on one channel. Asking the floor for the address is what makes
/// "no execution writes a second one" a mechanism rather than an argument.
///
/// The `weak` definition here is the same standalone answer
/// [`floor::FLOOR_RUNTIME_FALLBACK`] gives: an emitted module must link and run
/// without the floor's translation unit, and the real definition replaces this
/// one whenever that unit is linked, which is every ordinary build. Zero until
/// some thread writes a record, and no path outside the writer reads it, so a
/// program that writes none pays nothing for it.
const TRAP_LATCH: &str = "@.wf_trap.latch = private global i32 0, align 4\n";

/// The module's standalone definition of the shared latch's accessor.
const TRAP_LATCH_FALLBACK: &str =
    "\ndefine weak ptr @wf__floor_record_latch() {\nentry:\n  ret ptr @.wf_trap.latch\n}\n";

/// [`SEQUENTIAL_TRAP_WRITER`]'s work under a first-writer-wins latch, emitted
/// where the module can have more than one thread inside it — that is, where
/// it writes any record at all and hands a call out, whether or not it
/// contains a `claim`.
///
/// The first thread to arrive takes the shared latch and owns the record: it
/// writes its complete bytes to standard error and aborts the process without
/// unwinding. Every other thread that arrives while the latch is taken parks,
/// and the winner's abort takes it down with the process. The latch is the
/// floor runtime's, not this module's, so the parking also holds between this
/// writer and the floor's own — an execution that runs out of stack on one
/// thread while another is refused an allocation still produces exactly one
/// well-formed record rather than two interleaved ones. [PAR-1]'s
/// erroneous-execution guarantee is met by construction instead of by refusing
/// to overlap the claim-bearing calls of correct programs.
///
/// *Which* record wins may depend on the schedule, and that is the whole of
/// what a permitted overlap can change about one. The bytes are fixed by the
/// claim [DIAG-3] or the resource class that wins — no worker, thread, or
/// dynamic stack appears in them — so a correct program that exhausts nothing
/// observes nothing here.
///
/// The park spins on a *volatile* load rather than an empty loop, so no
/// optimizer may delete the loop and let a losing thread fall through into a
/// second record.
const LATCHED_TRAP_WRITER: &str = "\ndefine private void @wf_trap(ptr %message, i64 %length) noreturn {\nentry:\n  %latch = call ptr @wf__floor_record_latch()\n  %claimed = cmpxchg ptr %latch, i32 0, i32 1 seq_cst seq_cst\n  %won = extractvalue { i32, i1 } %claimed, 1\n  br i1 %won, label %write.loop, label %park\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @write(i32 2, ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\npark:\n  %parked = load volatile i32, ptr %latch, align 4\n  br label %park\n}\n\n";

pub(super) fn trap_record(site: &IrClaimSite) -> Vec<u8> {
    let components = site
        .node_path
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"rule_id\":{},\"message\":{},\"function\":{},\"node_path\":[{components}]}}\n",
        json_string(site.rule_id),
        json_string(&site.message),
        json_string(&site.function)
    )
    .into_bytes()
}

/// Encodes one [DIAG-3] record field as a JSON string.
///
/// The input is always a Rust `str`, so it is already one well-formed UTF-8
/// sequence; the record must carry those exact bytes. Iteration is therefore
/// over scalar values, not bytes: pushing a `char` writes its complete UTF-8
/// encoding, so a multi-byte scalar survives intact and no record can end
/// inside an encoding. Byte iteration with `char::from` was wrong for exactly
/// this case — it reinterpreted each continuation byte as the Latin-1 scalar
/// of the same value and re-encoded it, doubling every non-ASCII byte. ASCII
/// input is unaffected either way.
///
/// Only `"`, `\`, and newline need escaping here: [FORM-5] admits no other
/// control in a STRING, so no other scalar in a record field requires a JSON
/// escape.
fn json_string(value: &str) -> String {
    let mut encoded = String::from("\"");
    for scalar in value.chars() {
        match scalar {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            _ => encoded.push(scalar),
        }
    }
    encoded.push('"');
    encoded
}

fn llvm_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 3);
    for byte in bytes {
        let _ = write!(encoded, "\\{byte:02X}");
    }
    encoded
}
