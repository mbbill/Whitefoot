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
mod runs;
mod slice;
mod system;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write;

use super::qualification::{
    Qualification, QualificationFailure, SystemTarget, qualified_representation, qualify_program,
};
use super::target::{
    TargetAggregateLayout, TargetFramePlan, TargetFrameSlot, TargetLayout, TargetLayoutFailure,
    TargetStorageType, parallel_lane_frame_layout, plan_target_frame, validate_program,
    validate_static_storage,
};
use crate::{
    IrAddressed, IrArrayRoot, IrBlock, IrBlockId, IrBooleanOperation, IrCompletionStep, IrConstant,
    IrDrop, IrEntry, IrEnumType, IrFloatOperation, IrFunction, IrGlobalValue, IrInstruction,
    IrIntegerOperation, IrLayoutCeiling, IrNominal, IrNominalId, IrNominalKind, IrOperation,
    IrOverlap, IrProgram, IrRuntimeTargetObligations, IrTargetDomainObligation, IrTerminator,
    IrType, IrValueId, SystemResourceType,
};
use buffer::{buffer_fill_done_label, buffer_probe_join_label, buffer_vacant_done_label};
use cleanup::{emit_resource_drop_helpers, emit_value_cleanup, type_requires_cleanup};
use completion::completion_offered_label;
pub use completion::{
    COMPLETION_BRIDGE_HEADER, COMPLETION_BRIDGE_SOURCE, COMPLETION_CONTRACT_HEADER,
    COMPLETION_FILE_ADAPTER_HEADER, COMPLETION_FILE_ADAPTER_SOURCE, COMPLETION_FILE_POSIX_HEADER,
    COMPLETION_FILE_POSIX_SOURCE, COMPLETION_FILE_WINDOWS_SOURCE, COMPLETION_LINUX_IO_URING_HEADER,
    COMPLETION_LINUX_IO_URING_SOURCE, COMPLETION_RUNTIME_SOURCE, COMPLETION_SOCKET_ADDRESS_HEADER,
    COMPLETION_WAIT_HOST_SOURCE, COMPLETION_WAIT_WINDOWS_SOURCE, COMPLETION_WINDOWS_IOCP_HEADER,
    COMPLETION_WINDOWS_IOCP_SOURCE, SCHED_CORE_HEADER, SCHED_CORE_SOURCE, SCHED_ENTRY_HEADER,
    SCHED_ENTRY_SOURCE, SCHED_PRIM_HEADER, SCHED_PRIM_HOST_SOURCE, SCHED_PRIM_WINDOWS_SOURCE,
    SCHED_SWITCH_HEADER, module_requires_completion_runtime,
};
use floor::FLOOR_RUNTIME_FALLBACK;
pub use floor::FLOOR_STACK_BYTES;
pub use floor::{FLOOR_RUNTIME_SOURCE, FLOOR_WINDOWS_RUNTIME_SOURCE};
pub use parallel::module_requires_parallel_runtime;
use parallel::{
    HandedOut, LoopSplitSite, PARALLEL_POOL_QUERY_DECLARATION, PARALLEL_POOL_QUERY_FALLBACK,
    PARALLEL_RUNTIME_DECLARATIONS, PARALLEL_RUNTIME_FALLBACK, PARALLEL_SPLIT_BUDGET_DECLARATION,
    PARALLEL_SPLIT_BUDGET_FALLBACK, ParallelThunks, compute_join_order, par_done_label,
    sequential_clone_set, sequential_clone_symbol,
};
pub use system::{WINDOWS_RUNTIME_HEADER, WINDOWS_RUNTIME_SOURCE};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendFailure {
    TargetLayout(TargetLayoutFailure),
    /// The [QUAL-1] target-qualification table has no approved implementation
    /// for a facility the program uses on the selected target and program
    /// kind, or a required [QUAL-2] target guarantee is unmet. Like a
    /// target-layout failure this is not a source-language rejection and cites
    /// no language rule [DIAG-1].
    TargetQualification(QualificationFailure),
    /// Lowering assigned the same static completion site twice without an
    /// intervening drain. No source-derived driver has this form; reaching it
    /// is an internal compiler defect, not a source-language rejection.
    SecondOutstandingCompletionOperation,
    /// Emission reached the end of a compiler-generated schedule before its
    /// generated drain consumed every target operation. This is an internal
    /// compiler defect, not a source-language rejection.
    UnretiredCompletionOperation,
    /// A generated issue or drain block reached ring storage without the
    /// `u64` slot value lowering assigned to it. The bounded-batch constructor
    /// proves the range in its own CFG; reaching this state is an internal
    /// compiler defect, not a second proof layer or source rejection.
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
    let system = system::emit_system_interface(program, &qualification, target)?;

    let mut intrinsics = BTreeSet::new();
    let mut thunks = ParallelThunks::default();
    let mut completion_used = false;
    let mut functions = String::new();
    for function in program.functions() {
        let emitter = FunctionEmitter::new(
            program,
            &qualification,
            target,
            function,
            ModuleState {
                intrinsics: &mut intrinsics,
                parallel: &mut thunks,
                completion_used: &mut completion_used,
                sequential_clones: None,
            },
        )?;
        functions.push_str(&emitter.emit()?);
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
                    target,
                    function,
                    ModuleState {
                        intrinsics: &mut intrinsics,
                        parallel: &mut thunks,
                        completion_used: &mut completion_used,
                        sequential_clones: Some(&clones),
                    },
                )?
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
    let drop_helpers = emit_resource_drop_helpers(program, &qualification, target)?;
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
    let heap_record_type = TargetStorageType::bytes(
        u64::try_from(HEAP_RECORD.len()).map_err(|_| BackendFailure::CounterOverflow)?,
    );
    if has_heap_storage {
        validate_static_storage(target, &qualification, program, &heap_record_type)
            .map_err(BackendFailure::TargetLayout)?;
    }
    let mut text = format!(
        "; Whitefoot conservative module\nsource_filename = \"whitefoot\"\ntarget datalayout = \"{}\"\ntarget triple = \"{}\"\n\n",
        target.data_layout(),
        target.triple(),
    );
    emit_nominal_declarations(&mut text, program)?;
    emit_global_constants(&mut text, program)?;
    text.push_str(&system.constants);
    // An allocation this host refuses is the heap twin of an exhausted stack,
    // and it gets the same treatment: one record naming the resource class,
    // written once, before a defined abort. The bytes carry no `rule_id`, no
    // function, and no node path because resource availability is not a
    // source-code failure.
    if has_heap_storage {
        writeln!(
            text,
            "@.wf_resource.heap = private unnamed_addr constant {} c\"{}\", align 1",
            llvm_storage_type(program, &heap_record_type)?,
            llvm_bytes(HEAP_RECORD.as_bytes())
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    // Heap availability is outside source proof. If this module can allocate,
    // it carries one resource-record writer for allocator refusal.
    let writes_a_record = has_heap_storage;
    // A latch decides between threads, so it belongs only to a module that has
    // more than one. `thunks.is_used()` is exactly "this module hands a call
    // out to a worker lane": false for every default build, and false for a
    // `--par` build that actualizes nothing. A lone thread races no one, so
    // those modules emit the sequential resource path.
    let latched_resource_record = writes_a_record && thunks.is_used();
    if latched_resource_record {
        validate_static_storage(
            target,
            &qualification,
            program,
            &TargetStorageType::integer(32),
        )
        .map_err(BackendFailure::TargetLayout)?;
        text.push_str(RESOURCE_RECORD_LATCH);
    }
    // The resource record and the qualified system interface can need the
    // same host symbol; one module declares it once.
    let mut system_declarations = system.declarations;
    if writes_a_record {
        text.push('\n');
        if system_target.is_windows() {
            text.push_str("declare i64 @wf__windows_diagnostic_write(ptr, i64)\n");
            system_declarations.remove("declare i64 @wf__windows_diagnostic_write(ptr, i64)");
        } else {
            text.push_str("declare i64 @write(i32, ptr, i64)\n");
            system_declarations.remove("declare i64 @write(i32, ptr, i64)");
        }
    }
    if writes_a_record || has_matches || (system_target.is_windows() && completion_used) {
        text.push_str("declare void @abort() noreturn\n");
        system_declarations.remove("declare void @abort() noreturn");
    }
    // A general store's run takes its backing from the allocator and gives it
    // back at the release [PROV-1, BLK-2], so a module holding one declares
    // the two symbols even where nothing else on this list allocates. It does
    // not write a resource record: a refused take is the row's own `None`
    // arm and never an abort.
    if has_heap_storage || cleanup::program_has_general_run(program) {
        text.push_str("declare ptr @malloc(i64)\ndeclare void @free(ptr)\n");
    }
    for declaration in &system_declarations {
        text.push_str(declaration);
        text.push('\n');
    }
    if latched_resource_record {
        text.push_str(RESOURCE_RECORD_LATCH_FALLBACK);
        text.push_str(if system_target.is_windows() {
            WINDOWS_LATCHED_RESOURCE_RECORD_WRITER
        } else {
            LATCHED_RESOURCE_RECORD_WRITER
        });
    } else if writes_a_record {
        text.push_str(if system_target.is_windows() {
            WINDOWS_SEQUENTIAL_RESOURCE_RECORD_WRITER
        } else {
            SEQUENTIAL_RESOURCE_RECORD_WRITER
        });
    } else if has_matches {
        text.push('\n');
    }
    if has_heap_storage {
        writeln!(
            text,
            "define private void @wf_resource_abort() noreturn {{\nentry:\n  call void @wf_resource_record_abort(ptr @.wf_resource.heap, i64 {})\n  unreachable\n}}\n",
            HEAP_RECORD.len()
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
    if thunks.is_used() {
        text.push('\n');
        text.push_str(if system_target.is_windows() {
            PARALLEL_RUNTIME_DECLARATIONS
        } else {
            PARALLEL_RUNTIME_FALLBACK
        });
        if !clones.is_empty() {
            text.push_str(if system_target.is_windows() {
                PARALLEL_POOL_QUERY_DECLARATION
            } else {
                PARALLEL_POOL_QUERY_FALLBACK
            });
        }
        if thunks.queries_split_budget() {
            text.push_str(if system_target.is_windows() {
                PARALLEL_SPLIT_BUDGET_DECLARATION
            } else {
                PARALLEL_SPLIT_BUDGET_FALLBACK
            });
        }
        text.push_str(thunks.definitions());
    }
    if completion_used {
        text.push('\n');
        // Declarations on both platforms now: a submit answers nothing, so
        // there is no weak body that could stand in for the runtime, and a
        // link that omits it is an unresolved symbol rather than a program
        // that silently runs a second arm (design §8).
        //
        // The qualified wrappers reach the same entries, and declared the ones
        // they name through the system interface above, so the two sets
        // overlap wherever a module both hands an operation out and calls a
        // wrapper. One module declares each entry once, the same way the
        // resource record and the system interface share a host symbol.
        let runtime = if system_target.is_windows() {
            completion::COMPLETION_WINDOWS_RUNTIME_DECLARATIONS
        } else {
            completion::COMPLETION_RUNTIME_DECLARATIONS
        };
        for declaration in runtime.lines() {
            if system_declarations.contains(declaration) {
                continue;
            }
            text.push_str(declaration);
            text.push('\n');
        }
        // Emitted only where a module actually asks for a window, exactly as
        // the split budget's fallback is, so a module that stages no loop
        // names no such symbol at all.
        if functions.contains("@wf__completion_window(") {
            text.push_str(if system_target.is_windows() {
                completion::COMPLETION_WINDOWS_WINDOW_DECLARATION
            } else {
                completion::COMPLETION_WINDOW_FALLBACK
            });
        }
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
/// and nothing else. It carries no rule identifier, function, or node path
/// because an allocation the host refused is the trusted computing base
/// reaching its limit, not a source proof obligation.
const HEAP_RECORD: &str = "{\"resource\":\"heap\"}\n";

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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum FunctionSlot {
    /// The frame slot one run operation writes its aggregate through
    /// [BLK-1]: a frame-resident run's slots are inline, so indexing one at a
    /// computed offset goes through storage.
    RunStorage(IrValueId),
    /// The bump extent one [BLK-2] reservation lays out in the reserving
    /// activation's own frame, at the byte extent and alignment its two type
    /// constants fix.
    ExtentStorage(IrValueId),
    ArrayFillValue(IrValueId),
    ArrayFillIndex(IrValueId),
    ArrayRoot(IrValueId),
    InsertArray(IrValueId),
    SliceRoot(IrValueId),
    Address(IrValueId),
    ArenaList(IrValueId),
    Completion(IrValueId, CompletionSlot),
    /// The lane frame each in-flight iteration of a staged loop was granted,
    /// or null where it was refused one. This is what the pipeline slot holds
    /// for the lane form, exactly as the record block is what it holds for a
    /// submitted system operation.
    StagedFrame(IrValueId),
    /// Each in-flight iteration's own answer, written where the call ran and
    /// read by the drain.
    StagedResult(IrValueId),
    /// One issue-stage value per in-flight iteration, keyed by the value the
    /// issue stage defines.
    StagedCarry(IrValueId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum CompletionSlot {
    Record,
    Result,
    RawValue,
    RawError,
    OpenOutcome,
    Component,
    Cursor,
    Submitted,
    Start,
    Extent,
    Destination,
}

struct PlannedFunctionSlot {
    logical_index: usize,
    pointer: String,
}

/// The one physical frame an ordinary generated function owns.
///
/// Planning walks the already-selected IR schedule before emission and gives
/// every actual materialization a semantic key. Target layout then turns the
/// logical slots into one explicitly padded struct. Emission can only obtain a
/// pointer by that key; it has no string-shaped `alloca` escape hatch.
struct FunctionFramePlan {
    target: TargetFramePlan,
    slots: HashMap<FunctionSlot, PlannedFunctionSlot>,
    ordered: Vec<FunctionSlot>,
}

impl FunctionFramePlan {
    fn build(
        target: TargetLayout,
        program: &IrProgram<'_, '_, '_>,
        qualification: &Qualification,
        function: &IrFunction,
        completion_steps: &HashMap<IrValueId, IrCompletionStep>,
        pipeline: Option<&crate::IrCompletionPipeline>,
        staged_lane: Option<&parallel::StagedLane>,
    ) -> Result<Self, BackendFailure> {
        let mut specifications = Vec::new();
        let mut ordered = Vec::new();
        for (block_index, block) in function.blocks().iter().enumerate() {
            let block_id =
                IrBlockId::from_index(block_index).map_err(|_| BackendFailure::CounterOverflow)?;
            for instruction in block.instructions() {
                let IrInstruction::Define {
                    result,
                    ty,
                    operation,
                } = instruction
                else {
                    continue;
                };
                match operation {
                    IrOperation::ArrayFill { .. } => {
                        push_function_slot(
                            &mut specifications,
                            &mut ordered,
                            FunctionSlot::ArrayFillValue(*result),
                            TargetStorageType::source(*ty),
                            None,
                        )?;
                        push_function_slot(
                            &mut specifications,
                            &mut ordered,
                            FunctionSlot::ArrayFillIndex(*result),
                            TargetStorageType::integer(64),
                            None,
                        )?;
                    }
                    IrOperation::ArrayIndex {
                        root: IrArrayRoot::Value(value),
                        ..
                    } => {
                        let root_type = function
                            .value_type(*value)
                            .ok_or(BackendFailure::InvalidIr)?;
                        push_function_slot(
                            &mut specifications,
                            &mut ordered,
                            FunctionSlot::ArrayRoot(*result),
                            TargetStorageType::source(root_type),
                            None,
                        )?;
                    }
                    IrOperation::InsertArray { .. } => push_function_slot(
                        &mut specifications,
                        &mut ordered,
                        FunctionSlot::InsertArray(*result),
                        TargetStorageType::source(*ty),
                        None,
                    )?,
                    // [BLK-1] a frame-resident run's element access goes
                    // through storage, because the slot index is computed.
                    IrOperation::RunIndex { run, .. }
                    | IrOperation::RunTaken { run, .. }
                    | IrOperation::RunStore { run, .. }
                    | IrOperation::RunBoundary { run, .. }
                    // [VIEW-2] a view of an inline run takes the address of
                    // its slots, so it needs the same frame slot an element
                    // access of one needs.
                    | IrOperation::SliceFromRun { run } => {
                        let run_type =
                            function.value_type(*run).ok_or(BackendFailure::InvalidIr)?;
                        // Read the storage the run's own type names rather
                        // than naming the run types that keep slots inline
                        // here: the emission that consumes this slot decides
                        // the same question through the shape table, and two
                        // readings of one fact go out of step at the third
                        // run storage.
                        if runs::run_keeps_its_slots_inline(run_type) {
                            push_function_slot(
                                &mut specifications,
                                &mut ordered,
                                FunctionSlot::RunStorage(*result),
                                TargetStorageType::source(run_type),
                                None,
                            )?;
                        }
                    }
                    IrOperation::SliceFromArray {
                        array: IrArrayRoot::Value(value),
                    } => {
                        let array_type = function
                            .value_type(*value)
                            .ok_or(BackendFailure::InvalidIr)?;
                        push_function_slot(
                            &mut specifications,
                            &mut ordered,
                            FunctionSlot::SliceRoot(*result),
                            TargetStorageType::source(array_type),
                            None,
                        )?;
                    }
                    IrOperation::AddressOf { referent, .. } => push_function_slot(
                        &mut specifications,
                        &mut ordered,
                        FunctionSlot::Address(*result),
                        TargetStorageType::source(referent.ty()),
                        None,
                    )?,
                    IrOperation::ArenaListNew => push_function_slot(
                        &mut specifications,
                        &mut ordered,
                        FunctionSlot::ArenaList(*result),
                        TargetStorageType::pointer(),
                        None,
                    )?,
                    // [BLK-2] the reserved extent itself. Its alignment is
                    // the store's own type constant, which is what makes the
                    // bump cursor a multiple of it at every program point.
                    IrOperation::ArenaFrame { bytes, align } => {
                        if ordered.contains(&FunctionSlot::ExtentStorage(*result)) {
                            return Err(BackendFailure::InvalidIr);
                        }
                        specifications.push(TargetFrameSlot::aligned(
                            TargetStorageType::bytes(*bytes),
                            *align,
                        ));
                        ordered.push(FunctionSlot::ExtentStorage(*result));
                    }
                    IrOperation::SystemCall {
                        operation,
                        arguments,
                        ..
                    } if completion_steps
                        .get(result)
                        .is_some_and(IrCompletionStep::submit) =>
                    {
                        plan_completion_slots(
                            &mut specifications,
                            &mut ordered,
                            qualification,
                            function,
                            pipeline,
                            block_id,
                            *result,
                            *ty,
                            *operation,
                            arguments,
                        )?;
                    }
                    _ => {}
                }
            }
        }
        // The staged lane hand-out's ring, reserved once for the whole loop.
        // Its elements are a *plan* of the emission rather than an instruction
        // of the IR, so they are pushed here rather than found in the walk
        // above: the frame the iteration was granted, the answer it produced,
        // and the issue-stage values its drain reads back.
        if let Some(staged) = staged_lane {
            let slots = staged.slots;
            if staged.frame_bytes.is_some() {
                push_function_slot(
                    &mut specifications,
                    &mut ordered,
                    FunctionSlot::StagedFrame(staged.result),
                    TargetStorageType::array(TargetStorageType::pointer(), slots),
                    None,
                )?;
            }
            push_function_slot(
                &mut specifications,
                &mut ordered,
                FunctionSlot::StagedResult(staged.result),
                TargetStorageType::array(TargetStorageType::source(staged.result_type), slots),
                None,
            )?;
            for (origin, _) in &staged.carries {
                let carried = function
                    .value_type(*origin)
                    .ok_or(BackendFailure::InvalidIr)?;
                push_function_slot(
                    &mut specifications,
                    &mut ordered,
                    FunctionSlot::StagedCarry(*origin),
                    TargetStorageType::array(TargetStorageType::source(carried), slots),
                    None,
                )?;
            }
        }

        let target_plan = plan_target_frame(target, qualification, program, &specifications)
            .map_err(BackendFailure::TargetLayout)?;
        let mut slots = HashMap::with_capacity(ordered.len());
        for (logical_index, key) in ordered.iter().copied().enumerate() {
            let pointer = match key {
                FunctionSlot::Address(result) | FunctionSlot::ArenaList(result) => {
                    value_name(result)
                }
                _ => format!("%wf.slot.{logical_index}"),
            };
            if slots
                .insert(
                    key,
                    PlannedFunctionSlot {
                        logical_index,
                        pointer,
                    },
                )
                .is_some()
            {
                return Err(BackendFailure::InvalidIr);
            }
        }
        Ok(Self {
            target: target_plan,
            slots,
            ordered,
        })
    }

    fn slot(&self, key: FunctionSlot) -> Result<String, BackendFailure> {
        self.slots
            .get(&key)
            .map(|slot| slot.pointer.clone())
            .ok_or(BackendFailure::InvalidIr)
    }

    fn render(&self, program: &IrProgram<'_, '_, '_>) -> Result<String, BackendFailure> {
        if self.target.is_empty() {
            return Ok(String::new());
        }
        let fields = self
            .target
            .physical_fields()
            .iter()
            .map(|field| llvm_storage_type(program, field))
            .collect::<Result<Vec<_>, _>>()?;
        let frame_type = format!("{{ {} }}", fields.join(", "));
        let mut output = String::new();
        writeln!(
            output,
            "  %wf.frame = alloca {frame_type}, align {}",
            self.target.layout().align()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        for key in &self.ordered {
            let slot = self.slots.get(key).ok_or(BackendFailure::InvalidIr)?;
            let field = self
                .target
                .logical_field(slot.logical_index)
                .ok_or(BackendFailure::InvalidIr)?;
            writeln!(
                output,
                "  {} = getelementptr inbounds {frame_type}, ptr %wf.frame, i32 0, i32 {}",
                slot.pointer,
                field.physical_index()
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        Ok(output)
    }
}

/// Reserves one logical frame slot under its semantic key.
///
/// `alignment` is `None` for the ordinary case, where the storage type's own
/// natural alignment is the slot's. It is `Some` only where an external
/// contract states the alignment the reservation must have, which a byte
/// block's natural alignment of one would otherwise understate.
fn push_function_slot(
    specifications: &mut Vec<TargetFrameSlot>,
    ordered: &mut Vec<FunctionSlot>,
    key: FunctionSlot,
    ty: TargetStorageType,
    alignment: Option<u64>,
) -> Result<(), BackendFailure> {
    if ordered.contains(&key) {
        return Err(BackendFailure::InvalidIr);
    }
    specifications.push(match alignment {
        Some(alignment) => TargetFrameSlot::aligned(ty, alignment),
        None => TargetFrameSlot::natural(ty),
    });
    ordered.push(key);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn plan_completion_slots(
    specifications: &mut Vec<TargetFrameSlot>,
    ordered: &mut Vec<FunctionSlot>,
    qualification: &Qualification,
    function: &IrFunction,
    pipeline: Option<&crate::IrCompletionPipeline>,
    block: IrBlockId,
    result: IrValueId,
    result_type: IrType,
    operation: crate::IrSystemOperation,
    arguments: &[IrValueId],
) -> Result<(), BackendFailure> {
    let operation =
        system::completion_file_operation(operation).ok_or(BackendFailure::InvalidIr)?;
    let uses_ring =
        pipeline.is_some_and(|pipeline| pipeline.slots() > 1 && pipeline.carries(block));
    let slot_count = if uses_ring {
        pipeline.map_or(1, crate::IrCompletionPipeline::slots)
    } else {
        1
    };
    let mut add = |role, element, alignment| {
        push_function_slot(
            specifications,
            ordered,
            FunctionSlot::Completion(result, role),
            TargetStorageType::array(element, slot_count),
            alignment,
        )
    };
    // The opaque record block, reserved by the two ABI constants the
    // completion contract states rather than by any layout this compiler
    // knows. A byte block's natural alignment is one, so the contract's
    // alignment is the reservation's; the C side asserts its own record
    // against the same two numbers.
    add(
        CompletionSlot::Record,
        TargetStorageType::bytes(completion::COMPLETION_RECORD_BYTES),
        Some(completion::COMPLETION_RECORD_ALIGN),
    )?;
    add(
        CompletionSlot::Result,
        TargetStorageType::source(result_type),
        None,
    )?;
    add(
        CompletionSlot::RawValue,
        TargetStorageType::integer(64),
        None,
    )?;
    add(
        CompletionSlot::RawError,
        TargetStorageType::integer(32),
        None,
    )?;

    match operation {
        system::CompletionFileOperation::OpenRead
        | system::CompletionFileOperation::OpenDirectory
        | system::CompletionFileOperation::OpenDirectorySource
        | system::CompletionFileOperation::OpenFile => {
            add(
                CompletionSlot::OpenOutcome,
                TargetStorageType::integer(32),
                None,
            )?;
        }
        system::CompletionFileOperation::Read
        | system::CompletionFileOperation::Write
        | system::CompletionFileOperation::Receive
        | system::CompletionFileOperation::Send
        | system::CompletionFileOperation::DirectoryNext => {}
    }
    if matches!(
        operation,
        system::CompletionFileOperation::OpenDirectory | system::CompletionFileOperation::OpenFile
    ) {
        let bytes = qualification
            .target()
            .component_limit()
            .checked_add(if qualification.target().is_windows() {
                2
            } else {
                1
            })
            .ok_or(BackendFailure::CounterOverflow)?;
        add(
            CompletionSlot::Component,
            TargetStorageType::bytes(bytes),
            None,
        )?;
    }
    if operation == system::CompletionFileOperation::DirectoryNext {
        add(CompletionSlot::Cursor, TargetStorageType::integer(64), None)?;
    }
    if !uses_ring && !qualification.target().is_windows() {
        return Ok(());
    }

    add(
        CompletionSlot::Submitted,
        TargetStorageType::integer(1),
        None,
    )?;
    if !uses_ring {
        return Ok(());
    }
    match operation {
        system::CompletionFileOperation::Read
        | system::CompletionFileOperation::Write
        | system::CompletionFileOperation::Receive
        | system::CompletionFileOperation::Send => {
            add(CompletionSlot::Start, TargetStorageType::integer(64), None)?;
            add(CompletionSlot::Extent, TargetStorageType::integer(64), None)?;
        }
        system::CompletionFileOperation::DirectoryNext => {
            let [_, destination, _, _] = arguments else {
                return Err(BackendFailure::InvalidIr);
            };
            let destination_type = function
                .value_type(*destination)
                .ok_or(BackendFailure::InvalidIr)?;
            add(
                CompletionSlot::Destination,
                TargetStorageType::source(destination_type),
                None,
            )?;
            add(CompletionSlot::Start, TargetStorageType::integer(64), None)?;
            add(CompletionSlot::Extent, TargetStorageType::integer(64), None)?;
        }
        system::CompletionFileOperation::OpenRead
        | system::CompletionFileOperation::OpenDirectory
        | system::CompletionFileOperation::OpenDirectorySource
        | system::CompletionFileOperation::OpenFile => {}
    }
    Ok(())
}

struct FunctionEmitter<'program, 'state> {
    program: &'program IrProgram<'program, 'program, 'program>,
    /// The [QUAL-1] table lookup this build already performed. Every emission
    /// site reads the resolved row; none consults the table again.
    qualification: &'program Qualification,
    function: &'program IrFunction,
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
    /// The validated physical frame which supplied `entry_prelude` and every
    /// pointer returned to an operation emitter.
    frame: FunctionFramePlan,
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
    /// Selected-target layouts of the exact `{ arguments..., result }`
    /// aggregates this world may place in runtime lane storage.
    ///
    /// This map is built before any function text is emitted. Its membership
    /// is therefore also the proof that the aggregate fits the runtime's
    /// 256-byte, 16-byte-aligned slot and the target's address-index domain.
    ordinary_lane_frames: HashMap<IrValueId, TargetAggregateLayout>,
    /// Source-ordered direct completion steps, keyed by the call value each
    /// step reaches.  Their wait sets contain only ordinary prior result/loan
    /// dependencies retained by lowering.
    completion_steps: HashMap<IrValueId, crate::IrCompletionStep>,
    /// The handed-out completion results whose lowering opens LLVM blocks of
    /// its own.
    ///
    /// Exactly one shape does: an open by component name, the one operation
    /// that can reach an outcome without submitting, so its submit ends in a
    /// `completion.offered` selecting the route and its join ends in a
    /// `par.done` selecting the value. Every other shape is straight-line in
    /// both places (design §8), and `block_exit_label` has to say so or a phi
    /// would name a block that is not its predecessor.
    branching_completions: HashSet<IrValueId>,
    /// Hand-outs emitted and not yet joined.
    ///
    /// Ordinarily this is empty at every terminator, because a block joins
    /// everything it handed out before it ends. A function carrying a staged
    /// loop pipeline is the exception: its selected operation remains here
    /// until the exact compiler-generated drain block retires it. Blocks that
    /// merely occur between feeder and drain in emission order do not own it.
    handed_out: Vec<HandedOut>,
    /// The staged loop pipeline of the function being emitted, or `None`.
    ///
    /// Lowering supplies this only for a source loop whose staged permission
    /// has a complete one-slot or bounded-batch driver. Native file targets
    /// use typed completion; another qualified target executes a bounded
    /// batch with a direct-call window of one.
    pipeline: Option<&'program crate::IrCompletionPipeline>,
    /// The staged lane hand-out this world emits for this function's driven
    /// loop, or `None` where the loop's staged call is a submitted system
    /// operation and where the function drives no such loop at all.
    staged_lane: Option<parallel::StagedLane>,
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
    /// Whether the block being emitted is the exact compiler-generated drain
    /// for the driven pipeline.
    block_drains: bool,
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
/// These travel together because they are all module-scope: intrinsic
/// declarations and thunks are collected across every function and rendered
/// once at the top, and the clone set is the same set for every function of the
/// module. Passing them as one named group keeps the emitter's own arguments —
/// the program, qualification, and the function — the ones a reader has to
/// think about.
struct ModuleState<'state> {
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
        target: TargetLayout,
        function: &'program IrFunction,
        module: ModuleState<'state>,
    ) -> Result<Self, BackendFailure> {
        let ModuleState {
            intrinsics,
            parallel,
            completion_used,
            sequential_clones,
        } = module;
        // The staged call of a driven [PAR-3] loop, when that call is a
        // may-suspend user call. It has a hand-out form of its own — the lane
        // frame — so it is not a call the emitter has to withdraw the
        // submission of.
        let staged_lane_call = function
            .driven_completion_pipeline()
            .filter(|pipeline| pipeline.lane_handout())
            .and_then(crate::IrCompletionPipeline::driven_result);
        // A sequential clone suppresses compute hand-outs only. Target
        // completion is independent of the compute-pool choice and remains
        // active in both worlds.
        let completion_steps: HashMap<_, _> =
            if qualification.target().supports_posix_file_completion() {
                function
                    .completion_steps()
                    .iter()
                    .cloned()
                    .map(|step| {
                        // A step submits only where the typed adapter has a
                        // hand-out form for that exact operation. Every other
                        // permitted may-suspend call keeps its qualified
                        // wrapper, which is the same one lowering — submit,
                        // then join, through the frame's own record — and
                        // differs only in holding one operation of that site
                        // rather than several. Selecting the wrapper here is
                        // an emission choice over one accepted program; it
                        // changes no judgment, no outcome, and no published
                        // byte (`backend/qualification.rs`, the [PAR-1]
                        // review: "a may-suspend record selects completion
                        // lowering only when the backend has a typed adapter
                        // for the exact operation").
                        let handed_out = matches!(
                            definition_operation(function, step.call()),
                            Some(IrOperation::SystemCall { operation, .. })
                                if system::completion_file_operation(*operation).is_some()
                        ) || staged_lane_call == Some(step.call());
                        let step = if handed_out {
                            step
                        } else {
                            step.without_submission()
                        };
                        (step.call(), step)
                    })
                    .collect()
            } else {
                HashMap::new()
            };
        let mut overlaps = Vec::new();
        let mut ordinary_lane_frames = HashMap::new();
        if sequential_clones.is_none() {
            for overlap in function.overlaps() {
                // A group that also carries completion steps is not excluded
                // here: a mixed group hands its compute members to lanes and
                // its submitting members to the completion source, and
                // `ordinary_overlap_lane_frames` is the one place that decides
                // whether every member of this group can be handed out at all.
                let Some(frames) = ordinary_overlap_lane_frames(
                    program,
                    qualification,
                    target,
                    function,
                    overlap,
                    &completion_steps,
                )?
                else {
                    continue;
                };
                for (result, layout) in frames {
                    if ordinary_lane_frames.insert(result, layout).is_some() {
                        return Err(BackendFailure::InvalidIr);
                    }
                }
                overlaps.push(overlap.clone());
            }
        }
        let branching_completions = function
            .blocks()
            .iter()
            .flat_map(crate::IrBlock::instructions)
            .filter_map(|instruction| match instruction {
                IrInstruction::Define {
                    result,
                    operation: IrOperation::SystemCall { operation, .. },
                    ..
                } if completion_steps.contains_key(result) => {
                    system::completion_file_operation(*operation)
                        .filter(|operation| completion::completion_may_skip_submission(*operation))
                        .map(|_| *result)
                }
                _ => None,
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
        let supports_completion = qualification.target().supports_posix_file_completion();
        let pipeline = function.driven_completion_pipeline().filter(|pipeline| {
            if !supports_completion {
                return pipeline.planned_batch_driver().is_some_and(|driver| {
                    function
                        .completion_steps()
                        .iter()
                        .find(|step| step.call() == driver.result())
                        .is_some_and(crate::IrCompletionStep::submit)
                });
            }
            let planned_result = pipeline
                .planned_driver()
                .map(|driver| driver.result())
                .or_else(|| {
                    pipeline
                        .planned_batch_driver()
                        .map(|driver| driver.result())
                });
            planned_result.is_none_or(|result| {
                completion_steps
                    .get(&result)
                    .is_some_and(crate::IrCompletionStep::submit)
            })
        });
        let staged_lane = parallel::staged_lane_plan(
            program,
            qualification,
            target,
            function,
            pipeline,
            &completion_steps,
            sequential_clones.is_none(),
        )?;
        let frame = FunctionFramePlan::build(
            target,
            program,
            qualification,
            function,
            &completion_steps,
            pipeline,
            staged_lane.as_ref(),
        )?;
        let entry_prelude = frame.render(program)?;
        Ok(Self {
            program,
            qualification,
            function,
            intrinsics,
            incoming: Vec::new(),
            output: String::new(),
            entry_prelude,
            frame,
            temporary: 0,
            parallel,
            overlaps,
            overlap_handed_out,
            overlap_join_sites,
            ordinary_lane_frames,
            completion_steps,
            branching_completions,
            handed_out: Vec::new(),
            pipeline,
            staged_lane,
            block_slot: None,
            block_carries: false,
            block_drains: false,
            completion_used,
            sequential_clones,
        })
    }

    fn is_overlap_join_site(&self, value: IrValueId) -> bool {
        self.overlap_join_sites.contains(&value)
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
            self.block_drains = self
                .pipeline
                .is_some_and(|pipeline| pipeline.drains(block_id));
            self.emit_completion_window(block_id)?;
            if self.block_drains {
                self.emit_staged_lane_retirement()?;
            }
            for (instruction_index, instruction) in block.instructions().iter().enumerate() {
                self.emit_instruction(block_id, instruction_index, instruction)?;
            }
            self.emit_terminator(block_id, block.terminator())?;
        }
        // Every ordinary operation is joined at its block boundary. A driven
        // operation remains protected across emission until the exact drain
        // named by its lowering-owned pipeline consumes it.
        if self
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
                // The same rule `emit_terminator` applies: on a target without
                // native completion a gate's pending-exit edge is emitted as
                // `unreachable`, so it is no predecessor of the drain.
                if !self.qualification.target().supports_posix_file_completion()
                    && self
                        .pipeline
                        .is_some_and(|pipeline| pipeline.pending_exit_edge(predecessor))
                {
                    continue;
                }
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
                        &self.completion_steps,
                        &self.branching_completions,
                        self.pipeline,
                        self.staged_lane.as_ref(),
                    )
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            self.output.push('\n');
        }
        Ok(())
    }

    /// One instruction, and then the ring element any value it defines owes a
    /// staged loop's drain.
    ///
    /// The store is separate from the definition because the definition is an
    /// ordinary one: what the drain needs is not a second lowering of the
    /// value but a copy of it kept per in-flight iteration.
    fn emit_instruction(
        &mut self,
        block: IrBlockId,
        index: usize,
        instruction: &IrInstruction,
    ) -> Result<(), BackendFailure> {
        self.emit_instruction_body(block, index, instruction)?;
        if let IrInstruction::Define { result, .. } = instruction {
            self.store_staged_carry(*result)?;
        }
        Ok(())
    }

    fn emit_instruction_body(
        &mut self,
        _block: IrBlockId,
        _index: usize,
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
                match operation {
                    IrOperation::SystemCall {
                        operation,
                        target_action,
                        arguments,
                    } => {
                        if !target_action.may_suspend() {
                            return Err(BackendFailure::InvalidIr);
                        }
                        self.emit_handed_out_system_call(*result, *ty, *operation, arguments)?;
                    }
                    // A staged may-suspend user call: the lane frame is what
                    // the slot holds for this iteration, and the exact drain
                    // is where its answer is read.
                    IrOperation::Call {
                        function,
                        arguments,
                    } => self.emit_staged_lane_call(*result, *ty, *function, arguments)?,
                    _ => return Err(BackendFailure::InvalidIr),
                }
            } else if self.is_overlap_join_site(*result) {
                // A mixed group's join site is an ordinary step of the same
                // schedule. Its definition is emitted exactly as below, and
                // then the group's joins run through `compute_join_order`:
                // compute members newest first, completion members where they
                // were published (design section 4). `emit_overlap_joins`
                // takes the whole queue, so the `finish` drain below finds no
                // member of this group left and joins only what is outside it.
                self.emit_definition_then_join(instruction, *result)?;
            } else {
                // A compute member of the group is handed out by
                // `emit_definition` itself, through the same
                // `overlap_handed_out` test every other call takes.
                self.emit_definition(*result, *ty, operation)?;
            }
            // Ordinary completion steps still finish at their source-owned
            // boundary. The dependency helper protects a driven result until
            // the exact generated drain, even if this block is merely emitted
            // between its feeder and drain.
            if step.finish() {
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
            IrInstruction::StoreBuffer {
                buffer,
                index,
                value,
            } => self.emit_buffer_store(*buffer, *index, *value),
            IrInstruction::StoreSlice {
                slice,
                index,
                value,
            } => self.emit_slice_store(*slice, *index, *value),
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
            IrOperation::BufferMeasure { buffer } => self.emit_buffer_length(result, ty, *buffer),
            IrOperation::FixedVector => self.emit_fixed_vector(result, ty),
            IrOperation::ArenaFrame { bytes, align } => {
                self.emit_arena_frame(result, ty, *bytes, *align)
            }
            IrOperation::StoreTake(take) => self.emit_store_take(result, ty, *take),
            IrOperation::StoreBox(cell) => self.emit_store_box(result, ty, *cell),
            IrOperation::ContainerMeasure { measure, container } => {
                self.emit_container_measure(result, ty, *measure, *container)
            }
            IrOperation::RunIndex {
                run,
                offset,
                target_domain,
            } => self.emit_run_index(result, ty, *run, *offset, *target_domain),
            IrOperation::RunStore {
                run,
                offset,
                value,
                target_domain,
            } => self.emit_run_store(result, ty, *run, *offset, *value, *target_domain),
            IrOperation::RunTaken { row, run } => self.emit_run_taken(result, ty, *row, *run),
            IrOperation::RunBoundary { row, run, value } => {
                self.emit_run_boundary(result, ty, *row, *run, *value)
            }
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
            IrOperation::SliceFromRun { run } => self.emit_slice_from_run(result, ty, *run),
            IrOperation::SliceMeasure { slice } => self.emit_slice_length(result, ty, *slice),
            IrOperation::SliceIndex {
                slice,
                offset,
                target_domain,
            } => self.emit_slice_index(result, ty, *slice, *offset, *target_domain),
            IrOperation::BoxNew { nominal, value } => {
                self.emit_box_new(result, ty, *nominal, *value)
            }
            IrOperation::BoxTake { nominal, value } => {
                self.emit_box_take(result, ty, *nominal, *value)
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
        let name = match pipeline.window_value() {
            Some(value) => self.value_name(value).trim_start_matches('%').to_owned(),
            None => self.next_temporary()?,
        };
        // A target without the native completion adapter keeps the same
        // compiler-generated CFG but admits exactly one issue before every
        // drain. The direct call's SSA result is therefore the result of that
        // one iteration, and source order is preserved without target-owned
        // storage or a runtime query.
        if !self.qualification.target().supports_posix_file_completion()
            && pipeline.planned_batch_driver().is_some()
        {
            writeln!(self.output, "  %{name} = add i64 0, 1")
                .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        }
        // A world that offers no lane carries no iteration in flight, so it
        // asks for no depth: one issue before every drain is the sequential
        // schedule, iteration by iteration, which is what the sequential clone
        // exists to be and what a frame the runtime's slot cannot hold gets.
        if self
            .staged_lane
            .as_ref()
            .is_some_and(|staged| staged.frame_bytes.is_none())
        {
            writeln!(self.output, "  %{name} = add i64 0, 1")
                .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        }
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
        // Ordinary completion work is block-scoped. A driven result is the
        // one exception: `emit_completion_dependencies` leaves it outstanding
        // until `block_drains` names this exact generated drain.
        self.emit_all_completion_joins()?;
        // A target without native completion admits one issue before every
        // drain, so a prologue gate never leaves with an operation in flight:
        // the edge that would drain first is unreachable there, and saying so
        // keeps the drain's one submission the only definition it reads.
        if !self.qualification.target().supports_posix_file_completion()
            && self
                .pipeline
                .is_some_and(|pipeline| pipeline.pending_exit_edge(block))
        {
            return writeln!(self.output, "  unreachable")
                .map_err(|_| BackendFailure::TextEmission);
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

    fn emit_drop(&mut self, drop: IrDrop) -> Result<(), BackendFailure> {
        if self.value_type(drop.value()) != Some(drop.ty()) {
            return Err(BackendFailure::InvalidIr);
        }
        let value_name = self.value_name(drop.value());
        match drop.ty() {
            IrType::Array { .. } | IrType::Slice { .. } => {}
            // [STOR-3] a run's and a provider's release actions. A
            // frame-resident run reclaims no storage of its own; a
            // store-resident run's own run is reclaimed by its store's
            // release [PROV-6]; and a bump extent's action resets its cursor,
            // which at a frame extent's scope exit is the frame itself. Each
            // still walks its elements when one derives a release action.
            IrType::FixedVector { .. } | IrType::Vector { .. } | IrType::Provider => {
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

    /// Returns the address assigned by the already validated physical frame.
    /// No operation emitter can create storage of its own.
    fn entry_slot(&self, key: FunctionSlot) -> Result<String, BackendFailure> {
        self.frame.slot(key)
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

/// Selected-target lane layouts for every member which would leave the
/// calling thread, or `None` when the permission remains sequential.
///
/// A possibly-suspending Whitefoot wrapper is declined here; only its direct
/// compiler-owned file operation can enter completion, and such a function
/// keeps the synchronous ABI of the sequential world (design section 8). An
/// ordinary call additionally has to fit the runtime lane slot as one
/// complete `{ arguments..., result }` aggregate.
fn ordinary_overlap_lane_frames(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    target: TargetLayout,
    function: &IrFunction,
    overlap: &IrOverlap,
    completion_steps: &HashMap<IrValueId, IrCompletionStep>,
) -> Result<Option<Vec<(IrValueId, TargetAggregateLayout)>>, BackendFailure> {
    // A submitting completion member carries the typed completion protocol and
    // takes no lane frame at all, so it is the one member allowed to suspend.
    // Every other member is an ordinary call this thread hands to a lane, and
    // one that may suspend still declines the whole group.
    let submits = |member: &IrValueId| {
        completion_steps
            .get(member)
            .is_some_and(IrCompletionStep::submit)
    };
    // The group's joins ride the join site's own definition, which the submit
    // arm of `emit_instruction` does not reach. A group whose last member
    // submits therefore keeps today's pure completion lowering.
    if overlap.join_site().is_some_and(|site| submits(&site)) {
        return Ok(None);
    }
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
            Some(IrOperation::SystemCall { target_action, .. }) => {
                target_action.may_suspend() && !submits(member)
            }
            _ => false,
        })
    {
        return Ok(None);
    }
    let mut frames = Vec::with_capacity(overlap.handed_out().len());
    for member in overlap.handed_out() {
        if submits(member) {
            continue;
        }
        let Some(IrOperation::Call {
            function: callee, ..
        }) = definition_operation(function, *member)
        else {
            return Ok(None);
        };
        let callee = program
            .functions()
            .get(*callee as usize)
            .ok_or(BackendFailure::InvalidIr)?;
        if callee.target_action().may_suspend() {
            return Ok(None);
        }
        let Some(layout) = parallel_lane_frame_layout(target, qualification, program, callee)
            .map_err(BackendFailure::TargetLayout)?
        else {
            return Ok(None);
        };
        frames.push((*member, layout));
    }
    Ok(Some(frames))
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

fn llvm_storage_type(
    program: &IrProgram<'_, '_, '_>,
    ty: &TargetStorageType,
) -> Result<String, BackendFailure> {
    match ty {
        TargetStorageType::Source(ty) => llvm_type(program, *ty),
        TargetStorageType::Pointer => Ok("ptr".to_owned()),
        TargetStorageType::Integer(width) if matches!(width, 1 | 8 | 16 | 32 | 64) => {
            Ok(format!("i{width}"))
        }
        TargetStorageType::Integer(_) => Err(BackendFailure::InvalidIr),
        TargetStorageType::Array { element, length } => Ok(format!(
            "[{length} x {}]",
            llvm_storage_type(program, element)?
        )),
    }
}

/// Renders one compiler-generated helper frame from the same typed plan target
/// layout validated. The returned text is inserted immediately after that
/// helper's `entry:` label.
fn render_named_target_frame(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    target: TargetLayout,
    slots: &[(&str, TargetFrameSlot)],
) -> Result<String, BackendFailure> {
    let specifications = slots
        .iter()
        .map(|(_, slot)| slot.clone())
        .collect::<Vec<_>>();
    let plan = plan_target_frame(target, qualification, program, &specifications)
        .map_err(BackendFailure::TargetLayout)?;
    if plan.is_empty() {
        return Ok(String::new());
    }
    let fields = plan
        .physical_fields()
        .iter()
        .map(|field| llvm_storage_type(program, field))
        .collect::<Result<Vec<_>, _>>()?;
    let frame_type = format!("{{ {} }}", fields.join(", "));
    let mut output = String::new();
    writeln!(
        output,
        "  %wf.frame = alloca {frame_type}, align {}",
        plan.layout().align()
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    for (logical_index, (name, _)) in slots.iter().enumerate() {
        let field = plan
            .logical_field(logical_index)
            .ok_or(BackendFailure::InvalidIr)?;
        writeln!(
            output,
            "  {name} = getelementptr inbounds {frame_type}, ptr %wf.frame, i32 0, i32 {}",
            field.physical_index()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    Ok(output)
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
        // A `Vector` descriptor is `{ pointer, cap, len, head }`; a
        // `FixedVector` is its slots followed by `len` and `head`; a provider
        // is proof-only and carries at most its own base and cursor
        // [BLK-1, PROV-1, OP-9].
        IrType::Vector { .. } => Ok("{ ptr, i64, i64, i64 }".to_owned()),
        IrType::FixedVector { element, length } => Ok(format!(
            "{{ [{length} x {}], i64, i64 }}",
            llvm_type(program, element.ty())?
        )),
        IrType::Provider => Ok("{ ptr, i64 }".to_owned()),
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

/// The member of the overlap group `result` joins whose join settles the
/// block's label, if `result` is a join site at all. Its `par.done` block is
/// where the block continues.
///
/// The group's members are joined in [`compute_join_order`], so the last one
/// is that order's last member and not the publish queue's — with two or more
/// compute members it is the *first* published compute member. `emit_overlap_joins`
/// reads the same order from the same function, so the label named here is the
/// one emission actually ends at.
///
/// A member is a completion member exactly when it is a completion step that
/// submits, which is the test `emit_instruction` itself takes before it can
/// reach a compute hand-out; `completion_steps` is passed in rather than
/// guessed at from the value id.
///
/// Not every join opens a block, so the answer is the last member that
/// *settles* a label rather than the last member joined. A compute member
/// always ends at its own `par.done`. A completion member ends at one only
/// when it is a branching completion — the single route that can reach an
/// outcome without submitting — and every other completion join is
/// straight-line and leaves the label where the join before it left it. A
/// mixed group whose join order ends in a plain completion member therefore
/// continues at the `par.done` of the last compute member before it.
///
/// `overlaps` is the emitting world's set, never `IrFunction::overlaps`: a
/// clone actualizes nothing, so it emits no `par.done` block and must not name
/// one either.
fn overlap_join_tail(
    overlaps: &[IrOverlap],
    completion_steps: &HashMap<IrValueId, IrCompletionStep>,
    branching_completions: &HashSet<IrValueId>,
    result: IrValueId,
) -> Option<IrValueId> {
    let members = overlaps
        .iter()
        .find(|overlap| overlap.join_site() == Some(result))?
        .handed_out()
        .to_vec();
    let submits = |member: &IrValueId| {
        completion_steps
            .get(member)
            .is_some_and(IrCompletionStep::submit)
    };
    let mut order = compute_join_order(members, |member| !submits(member));
    order.retain(|member| !submits(member) || branching_completions.contains(member));
    order.last().copied()
}

/// Where a block's terminator is actually emitted.
///
/// Emission is one pass over the blocks in order, so a block's phis are written
/// before the blocks that reach it. A phi therefore has to name the label an
/// incoming block *will* end at, and every operation that opens a new LLVM
/// block moves that label away from the plain `bbN` header. This function is
/// the one model of that: it walks a block's instructions and reports the
/// label the terminator lands in.
///
/// The two hand-out mechanisms both leave the block somewhere else. A compute
/// overlap settles on its group's `par.done` when its join site runs. A direct
/// completion step that can reach an outcome without submitting -- an open by
/// component name, and nothing else -- submits into `completion.offered` and
/// settles its join on that operation's own `par.done`; every other completion
/// step is straight-line at both points and moves the label nowhere. Ordinary
/// outstanding work is joined before the terminator; a driven result is
/// preserved until the exact drain block, so this walk uses the same owner and
/// queue order as `emit_completion_dependencies`.
fn block_exit_label(
    block_id: IrBlockId,
    block: &IrBlock,
    overlaps: &[IrOverlap],
    completion_steps: &HashMap<IrValueId, IrCompletionStep>,
    branching_completions: &HashSet<IrValueId>,
    pipeline: Option<&crate::IrCompletionPipeline>,
    staged_lane: Option<&parallel::StagedLane>,
) -> String {
    let mut label = block_label(block_id);
    let driven_result = pipeline
        .and_then(crate::IrCompletionPipeline::driven_result)
        .filter(|result| {
            completion_steps
                .get(result)
                .is_some_and(IrCompletionStep::submit)
        });
    let drains_pipeline = pipeline.is_some_and(|pipeline| pipeline.drains(block_id));
    let protected_result = driven_result.filter(|_| !drains_pipeline);
    // A staged lane retirement is emitted before this block's own
    // instructions, so the drain starts where that retirement left it. Only a
    // world that offers a lane opens a block there; the sequential form is one
    // load and moves the label nowhere.
    if let Some(staged) = staged_lane
        && drains_pipeline
        && staged.frame_bytes.is_some()
    {
        label = parallel::par_staged_done_label(staged.result);
    }
    // The direct completion hand-outs this block has submitted and not yet
    // joined, in `FunctionEmitter::handed_out` order. The exact drain starts
    // with the pipeline result its feeder left outstanding on the incoming
    // edge; unrelated blocks deliberately do not inherit that state.
    let mut outstanding: Vec<IrValueId> = driven_result
        .filter(|_| drains_pipeline)
        .into_iter()
        .collect();
    for (index, instruction) in block.instructions().iter().enumerate() {
        // `emit_instruction` checks the completion step first and returns, so
        // a step's call never reaches the compute-overlap join below; a step
        // that is also its group's join site carries that join in this arm
        // instead.
        if let IrInstruction::Define { result, .. } = instruction
            && let Some(step) = completion_steps.get(result)
        {
            drain_completions(
                &mut outstanding,
                step.wait_for(),
                branching_completions,
                &mut label,
            );
            if step.submit() {
                // A staged lane hand-out is not an outstanding completion
                // operation: nothing joins it at a block boundary, and its own
                // drain reads it from the ring rather than from this queue.
                match staged_lane.filter(|staged| staged.result == *result) {
                    Some(staged) => {
                        if staged.frame_bytes.is_some() {
                            label = parallel::par_staged_offered_label(*result);
                        }
                    }
                    None => {
                        outstanding.push(*result);
                        if branching_completions.contains(result) {
                            label = completion_offered_label(*result);
                        }
                    }
                }
            } else {
                definition_exit_label(block_id, index, instruction, &mut label);
                // A mixed group's join site is an ordinary step, and its
                // definition carries the group's joins. Those joins consume
                // every member of the group, its completion members included,
                // so they leave the outstanding set here as well as the label.
                if let Some(overlap) = overlaps
                    .iter()
                    .find(|overlap| overlap.join_site() == Some(*result))
                {
                    outstanding.retain(|held| !overlap.handed_out().contains(held));
                    if let Some(last) = overlap_join_tail(
                        overlaps,
                        completion_steps,
                        branching_completions,
                        *result,
                    ) {
                        label = par_done_label(last);
                    }
                }
            }
            if step.finish() {
                drain_all_completions_except(
                    &mut outstanding,
                    protected_result,
                    branching_completions,
                    &mut label,
                );
            }
            continue;
        }
        definition_exit_label(block_id, index, instruction, &mut label);
        // The overlap join rides its last member's own emission, so it settles
        // the label after whatever that emission left.
        if let IrInstruction::Define { result, .. } = instruction
            && let Some(last) =
                overlap_join_tail(overlaps, completion_steps, branching_completions, *result)
        {
            label = par_done_label(last);
        }
    }
    drain_all_completions_except(
        &mut outstanding,
        protected_result,
        branching_completions,
        &mut label,
    );
    label
}

/// Replays `emit_completion_dependencies`: each named operation still
/// outstanding is joined, and a join that selects between two routes leaves
/// the block at its `par.done`. A one-route join emits no block of its own and
/// leaves the label where it was.
fn drain_completions(
    outstanding: &mut Vec<IrValueId>,
    wanted: &[IrValueId],
    branching_completions: &HashSet<IrValueId>,
    label: &mut String,
) {
    for value in wanted {
        if let Some(position) = outstanding.iter().position(|held| held == value) {
            outstanding.remove(position);
            if branching_completions.contains(value) {
                *label = par_done_label(*value);
            }
        }
    }
}

/// Replays a block boundary while preserving the one pipeline result whose
/// generated drain occurs elsewhere in emission order.
fn drain_all_completions_except(
    outstanding: &mut Vec<IrValueId>,
    protected: Option<IrValueId>,
    branching_completions: &HashSet<IrValueId>,
    label: &mut String,
) {
    let mut last = None;
    outstanding.retain(|value| {
        if Some(*value) == protected {
            true
        } else {
            if branching_completions.contains(value) {
                last = Some(*value);
            }
            false
        }
    });
    if let Some(last) = last {
        *label = par_done_label(last);
    }
}

/// The label one ordinary instruction's own emission leaves the block at, for
/// the operations whose lowering opens a further LLVM block.
/// The block one cell formation joins at S39.
pub(super) fn store_box_join_label(result: IrValueId) -> String {
    format!("box.join.v{}", result.ordinal())
}

fn definition_exit_label(
    _block_id: IrBlockId,
    _index: usize,
    instruction: &IrInstruction,
    label: &mut String,
) {
    match instruction {
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
        // S39 a cell formation branches on the store's answer and joins,
        // so the block a successor's phi names is that join.
        IrInstruction::Define {
            result,
            operation: IrOperation::StoreBox { .. },
            ..
        } => *label = store_box_join_label(*result),
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

/// The heap-resource record writer of a module with one thread.
///
/// The thread that reaches it writes its complete record to standard error and
/// aborts the process without unwinding. There is no one to arbitrate with, so
/// there is no latch: these are the bytes every module emitted before the
/// overlapped world existed, and they are what a default build still gets.
const SEQUENTIAL_RESOURCE_RECORD_WRITER: &str = "\ndefine private void @wf_resource_record_abort(ptr %message, i64 %length) noreturn {\nentry:\n  br label %write.loop\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @write(i32 2, ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\n}\n\n";

/// Windows twin of [`SEQUENTIAL_RESOURCE_RECORD_WRITER`]. The private runtime
/// call writes the same bytes to the process diagnostic channel without
/// importing the POSIX file-descriptor ABI into a COFF module.
const WINDOWS_SEQUENTIAL_RESOURCE_RECORD_WRITER: &str = "\ndefine private void @wf_resource_record_abort(ptr %message, i64 %length) noreturn {\nentry:\n  br label %write.loop\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @wf__windows_diagnostic_write(ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\n}\n\n";

/// The module's own answer for the shared record latch, and the only state the
/// resource-record path carries.
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
const RESOURCE_RECORD_LATCH: &str = "@.wf_resource_record.latch = private global i32 0, align 4\n";

/// The module's standalone definition of the shared latch's accessor.
const RESOURCE_RECORD_LATCH_FALLBACK: &str = "\ndefine weak ptr @wf__floor_record_latch() {\nentry:\n  ret ptr @.wf_resource_record.latch\n}\n";

/// [`SEQUENTIAL_RESOURCE_RECORD_WRITER`]'s work under a first-writer-wins latch,
/// emitted
/// where the module can have more than one thread inside it — that is, where
/// it writes a heap-resource record and hands a call out.
///
/// The first thread to arrive takes the shared latch and owns the record: it
/// writes its complete bytes to standard error and aborts the process without
/// unwinding. Every other thread that arrives while the latch is taken parks,
/// and the winner's abort takes it down with the process. The latch is the
/// floor runtime's, not this module's, so the parking also holds between this
/// writer and the floor's own — an execution that runs out of stack on one
/// thread while another is refused an allocation still produces exactly one
/// well-formed record rather than two interleaved ones. [PAR-1]'s
/// erroneous-execution guarantee is met by construction.
///
/// *Which* record wins may depend on the schedule, and that is the whole of
/// what a permitted overlap can change about one. The bytes are fixed by the
/// resource class that wins — no worker, thread, or dynamic stack appears in
/// them — so a run that exhausts nothing observes nothing here.
///
/// The park spins on a *volatile* load rather than an empty loop, so no
/// optimizer may delete the loop and let a losing thread fall through into a
/// second record.
const LATCHED_RESOURCE_RECORD_WRITER: &str = "\ndefine private void @wf_resource_record_abort(ptr %message, i64 %length) noreturn {\nentry:\n  %latch = call ptr @wf__floor_record_latch()\n  %acquired = cmpxchg ptr %latch, i32 0, i32 1 seq_cst seq_cst\n  %won = extractvalue { i32, i1 } %acquired, 1\n  br i1 %won, label %write.loop, label %park\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @write(i32 2, ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\npark:\n  %parked = load volatile i32, ptr %latch, align 4\n  br label %park\n}\n\n";

/// Windows twin of [`LATCHED_RESOURCE_RECORD_WRITER`], sharing the floor
/// runtime's first-writer latch while using the native diagnostic channel.
const WINDOWS_LATCHED_RESOURCE_RECORD_WRITER: &str = "\ndefine private void @wf_resource_record_abort(ptr %message, i64 %length) noreturn {\nentry:\n  %latch = call ptr @wf__floor_record_latch()\n  %acquired = cmpxchg ptr %latch, i32 0, i32 1 seq_cst seq_cst\n  %won = extractvalue { i32, i1 } %acquired, 1\n  br i1 %won, label %write.loop, label %park\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @wf__windows_diagnostic_write(ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\npark:\n  %parked = load volatile i32, ptr %latch, align 4\n  br label %park\n}\n\n";

fn llvm_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 3);
    for byte in bytes {
        let _ = write!(encoded, "\\{byte:02X}");
    }
    encoded
}
