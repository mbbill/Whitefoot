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
mod conversion;
mod floating;
mod floor;
mod frontier;
mod integer;
mod operations;
mod parallel;
pub(super) mod places;
mod reinterpret;
mod runs;
mod slice;

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::Write;

use super::abi::FunctionAbi;
pub use super::runtime::*;
use super::storage::{FunctionStoragePlan, operation_operands};
use super::target::{
    TargetAggregateLayout, TargetFramePlan, TargetFrameSlot, TargetLayout, TargetLayoutFailure,
    TargetStorageType, parallel_lane_frame_layout, plan_target_frame, validate_program,
    validate_static_storage,
};
use crate::{
    IrAddressed, IrArrayRoot, IrBlock, IrBlockId, IrBooleanOperation, IrConstant, IrDrop,
    IrDropSubject, IrEnumType, IrFloatOperation, IrFunction, IrGlobalValue, IrInstruction,
    IrIntegerOperation, IrLayoutCeiling, IrNominal, IrNominalId, IrNominalKind, IrOperation,
    IrOverlap, IrProgram, IrRuntimeTargetObligations, IrTargetDomainObligation, IrTerminator,
    IrType, IrValueId,
};
use buffer::{buffer_fill_done_label, buffer_probe_join_label, buffer_vacant_done_label};
use cleanup::{emit_resource_drop_helpers, emit_value_cleanup, type_requires_cleanup};
use floor::FLOOR_RUNTIME_FALLBACK;
pub use floor::FLOOR_STACK_BYTES;
pub use floor::{FLOOR_RUNTIME_SOURCE, FLOOR_WINDOWS_RUNTIME_SOURCE};
pub(crate) use frontier::is_recursion_budget_symbol;
use frontier::{Grain, RecursiveFrontiers, recursion_budget_symbol};
pub use parallel::module_requires_parallel_runtime;
use parallel::{
    HandedOut, LoopSplitSite, PARALLEL_POOL_QUERY_DECLARATION, PARALLEL_POOL_QUERY_FALLBACK,
    PARALLEL_RECURSION_BUDGET_DECLARATION, PARALLEL_RECURSION_BUDGET_FALLBACK,
    PARALLEL_RUNTIME_DECLARATIONS, PARALLEL_RUNTIME_FALLBACK, PARALLEL_SPLIT_BUDGET_DECLARATION,
    PARALLEL_SPLIT_BUDGET_FALLBACK, ParallelThunks, par_done_label, sequential_clone_set,
    sequential_clone_symbol,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendFailure {
    TargetLayout(TargetLayoutFailure),
    InvalidIr,
    CounterOverflow,
    TextEmission,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LlvmModule {
    text: String,
    ledger: Vec<String>,
}

impl LlvmModule {
    #[must_use]
    pub fn into_string(self) -> String {
        self.text
    }

    /// The non-normative report of what this emission actualized that lowering
    /// could not report for it, in the order `--par-ledger` prints it.
    ///
    /// The recursion budget reads the call graph after the clone sets are
    /// known, which happens here and not in lowering, so its lines reach the
    /// ledger through this channel rather than through a second analysis. It
    /// says nothing about acceptance: a module with an empty report and a
    /// module with a full one are both programs the checker already admitted.
    #[must_use]
    pub fn actualization_ledger(&self) -> &[String] {
        &self.ledger
    }
}

pub fn emit_llvm(program: &IrProgram<'_, '_, '_>) -> Result<LlvmModule, BackendFailure> {
    let target = TargetLayout::host().map_err(BackendFailure::TargetLayout)?;
    emit_llvm_with_layout(program, target)
}

/// The executable builder may choose the no-pool world once at startup. The
/// set depends on ordinary calls and physical lane fit, never on an entry kind.
pub(crate) fn sequential_entry_symbol(
    program: &IrProgram<'_, '_, '_>,
    name: &str,
) -> Result<Option<String>, BackendFailure> {
    let clones = sequential_clone_set(program);
    let selected_is_cloned = program
        .functions()
        .iter()
        .enumerate()
        .any(|(index, function)| {
            function.name() == name
                && u32::try_from(index).is_ok_and(|index| clones.contains(&index))
        });
    if !selected_is_cloned {
        return Ok(None);
    }
    for function in program.functions() {
        for overlap in function.overlaps() {
            if ordinary_overlap_lane_frames(
                program,
                TargetLayout::host().map_err(BackendFailure::TargetLayout)?,
                function,
                overlap,
                &|_| false,
            )?
            .is_some()
            {
                return Ok(Some(sequential_clone_symbol(name)));
            }
        }
    }
    Ok(None)
}

/// Emits the same ordinary callable ABI with a selected physical target layout.
pub(super) fn emit_llvm_with_layout(
    program: &IrProgram<'_, '_, '_>,
    target: TargetLayout,
) -> Result<LlvmModule, BackendFailure> {
    validate_program(target, program).map_err(BackendFailure::TargetLayout)?;
    let mut intrinsics = BTreeSet::new();
    let mut thunks = ParallelThunks::default();
    let refusal_clones = if program.sequential_compute_refusal() {
        sequential_clone_set(program)
    } else {
        HashSet::new()
    };
    // Every compute-actualizing lowering asks the question, because every one
    // of them carries a recursion budget; the set is what decides which
    // components can answer it, since a family's cut lands in a clone.
    let frontier_clones = if program.recursion_budget().is_some() {
        sequential_clone_set(program)
    } else {
        HashSet::new()
    };
    let frontiers = RecursiveFrontiers::new(program, &frontier_clones);
    let mut functions = String::new();
    for (ordinal, function) in program.functions().iter().enumerate() {
        // A member of a budgeted component keeps its ordinary symbol and its
        // ordinary signature, and that symbol obtains the initial budget and
        // enters the family. The body itself is emitted once, below.
        if frontiers.grain(ordinal).is_some() {
            functions.push_str(&emit_recursion_budget_entry(
                program,
                function,
                &frontiers,
                &mut thunks,
            )?);
            continue;
        }
        let emitter = FunctionEmitter::new(
            program,
            target,
            function,
            ModuleState {
                intrinsics: &mut intrinsics,
                parallel: &mut thunks,
                sequential_clones: None,
                refusal_clones: &refusal_clones,
                frontiers: &frontiers,
                grain: None,
            },
        )?;
        functions.push_str(&emitter.emit()?);
    }
    // The budget-carrying half of each family: one variant per member, the
    // same emitter over the same IR as every other function of this module,
    // differing only in the trailing budget it tests and in naming its
    // siblings' variants.
    for (ordinal, function) in program.functions().iter().enumerate() {
        let Some(grain) = frontiers.grain(ordinal) else {
            continue;
        };
        functions.push_str(
            &FunctionEmitter::new(
                program,
                target,
                function,
                ModuleState {
                    intrinsics: &mut intrinsics,
                    parallel: &mut thunks,
                    sequential_clones: None,
                    refusal_clones: &refusal_clones,
                    frontiers: &frontiers,
                    grain: Some(grain),
                },
            )?
            .emit()?,
        );
    }
    // The second world. It exists only where the first one actualizes
    // something, so a build that hands nothing out — every default build among
    // them — emits exactly the module it emitted before this path existed.
    //
    // A build launcher can select the sequential clone of its ordinary entry.
    // The clone set is closed upwards through the module's call graph and
    // does not depend on the presence or spelling of a selected entry.
    let clones = if thunks.is_used() {
        sequential_clone_set(program)
    } else if frontiers.is_used() {
        frontier_clones
    } else {
        HashSet::new()
    };
    for (ordinal, function) in program.functions().iter().enumerate() {
        if u32::try_from(ordinal).is_ok_and(|ordinal| clones.contains(&ordinal)) {
            functions.push_str(
                &FunctionEmitter::new(
                    program,
                    target,
                    function,
                    ModuleState {
                        intrinsics: &mut intrinsics,
                        parallel: &mut thunks,
                        sequential_clones: Some(&clones),
                        refusal_clones: &refusal_clones,
                        frontiers: &frontiers,
                        grain: None,
                    },
                )?
                .emit()?,
            );
        }
    }
    let has_matches = program.functions().iter().any(|function| {
        function
            .blocks()
            .iter()
            .any(|block| matches!(block.terminator(), IrTerminator::Match { .. }))
    });
    let drop_helpers = emit_resource_drop_helpers(program, target)?;
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
        validate_static_storage(target, program, &heap_record_type)
            .map_err(BackendFailure::TargetLayout)?;
    }
    let mut text = format!(
        "; Whitefoot conservative module\nsource_filename = \"whitefoot\"\ntarget datalayout = \"{}\"\ntarget triple = \"{}\"\n\n",
        target.data_layout(),
        target.triple(),
    );
    emit_nominal_declarations(&mut text, program)?;
    emit_global_constants(&mut text, program)?;
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
        validate_static_storage(target, program, &TargetStorageType::integer(32))
            .map_err(BackendFailure::TargetLayout)?;
        text.push_str(RESOURCE_RECORD_LATCH);
    }
    let windows = target.triple().contains("windows");
    if writes_a_record {
        text.push_str(if windows {
            "declare i64 @wf__windows_diagnostic_write(ptr, i64)\n"
        } else {
            "declare i64 @write(i32, ptr, i64)\n"
        });
    }
    if writes_a_record || has_matches {
        text.push_str("declare void @abort() noreturn\n");
    }
    if has_heap_storage || cleanup::program_has_general_run(program)? {
        text.push_str("declare ptr @malloc(i64)\ndeclare void @free(ptr)\n");
    }
    if latched_resource_record {
        text.push_str(RESOURCE_RECORD_LATCH_FALLBACK);
        text.push_str(if windows {
            WINDOWS_LATCHED_RESOURCE_RECORD_WRITER
        } else {
            LATCHED_RESOURCE_RECORD_WRITER
        });
    } else if writes_a_record {
        text.push_str(if windows {
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
    for intrinsic in intrinsics {
        match intrinsic {
            IntrinsicDeclaration::MemoryMove => {
                writeln!(
                    text,
                    "declare void @llvm.memmove.p0.p0.i64(ptr, ptr, i64, i1 immarg)"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
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
        text.push_str(if windows {
            PARALLEL_RUNTIME_DECLARATIONS
        } else {
            PARALLEL_RUNTIME_FALLBACK
        });
        if !clones.is_empty() {
            text.push_str(if windows {
                PARALLEL_POOL_QUERY_DECLARATION
            } else {
                PARALLEL_POOL_QUERY_FALLBACK
            });
        }
        if thunks.queries_split_budget() {
            text.push_str(if windows {
                PARALLEL_SPLIT_BUDGET_DECLARATION
            } else {
                PARALLEL_SPLIT_BUDGET_FALLBACK
            });
        }
        if thunks.queries_recursion_budget() {
            text.push_str(if windows {
                PARALLEL_RECURSION_BUDGET_DECLARATION
            } else {
                PARALLEL_RECURSION_BUDGET_FALLBACK
            });
        }
        text.push_str(thunks.definitions());
    } else if thunks.queries_recursion_budget() {
        // A family whose every offer was declined for its frame still asks
        // for its budget, and the symbol it names must be answered.
        text.push('\n');
        text.push_str(if windows {
            PARALLEL_RECURSION_BUDGET_DECLARATION
        } else {
            PARALLEL_RECURSION_BUDGET_FALLBACK
        });
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
    Ok(LlvmModule {
        text: attach_stack_probe(&text, target),
        ledger: frontiers.ledger().to_vec(),
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

/// One call's arguments, in the emitting function's own parameters: what a
/// same-signature forward to a clone or a variant passes on.
fn ordinary_call_arguments(
    program: &IrProgram<'_, '_, '_>,
    function: &IrFunction,
    abi: &FunctionAbi,
) -> Result<String, BackendFailure> {
    let mut arguments = Vec::with_capacity(abi.parameters().len() + 1);
    if abi.result().uses_destination() {
        arguments.push("ptr %wf.result".to_owned());
    }
    for ((value, _), parameter) in function.parameters().iter().zip(abi.parameters()) {
        arguments.push(if parameter.is_indirect() {
            format!("ptr %wf.arg.v{}", value.ordinal())
        } else {
            format!(
                "{} {}",
                llvm_type(program, parameter.ty())?,
                value_name(*value)
            )
        });
    }
    Ok(arguments.join(", "))
}

/// A budgeted component's ordinary entry: obtain the initial budget and enter
/// the family with it.
///
/// This is the only place a budget comes from outside the family, and it is
/// asked once per call into the component rather than once per node. The
/// runtime's answer follows the pool width, which is why it is a query and not
/// a constant: the measured best cut is not the same cut at two lanes and at
/// four. A pinned budget — the A/B control — starts from a compile-time value
/// instead, and then this entry names no runtime symbol at all.
///
/// The member keeps its ordinary symbol, signature and result ABI, so nothing
/// outside the component sees the family; what changes is that the body it
/// forwards to is emitted once, with the budget as a trailing parameter.
fn emit_recursion_budget_entry(
    program: &IrProgram<'_, '_, '_>,
    function: &IrFunction,
    frontiers: &RecursiveFrontiers,
    thunks: &mut ParallelThunks,
) -> Result<String, BackendFailure> {
    let abi = FunctionAbi::build(program, function)?;
    let mut output = String::new();
    write!(
        output,
        "define internal {} @{}(",
        if abi.result().uses_destination() {
            "void".to_owned()
        } else {
            llvm_type(program, abi.result().ty())?
        },
        source_symbol(function.name())
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    let mut parameters = Vec::with_capacity(abi.parameters().len() + 1);
    if abi.result().uses_destination() {
        parameters.push("ptr %wf.result".to_owned());
    }
    for ((value, _), parameter) in function.parameters().iter().zip(abi.parameters()) {
        parameters.push(if parameter.is_indirect() {
            format!("ptr %wf.arg.v{}", value.ordinal())
        } else {
            format!(
                "{} {}",
                llvm_type(program, parameter.ty())?,
                value_name(*value)
            )
        });
    }
    output.push_str(&parameters.join(", "));
    output.push_str(") {\nentry:\n");
    let budget = match frontiers.initial().ok_or(BackendFailure::InvalidIr)? {
        crate::RecursionBudget::Off => return Err(BackendFailure::InvalidIr),
        crate::RecursionBudget::Pinned(levels) => levels.get().to_string(),
        crate::RecursionBudget::RuntimeDerived => {
            output.push_str("  %wf.budget = call i64 @wf__par_recursion_budget()\n");
            thunks.queries_recursion_budget = true;
            "%wf.budget".to_owned()
        }
    };
    let mut arguments = ordinary_call_arguments(program, function, &abi)?;
    if !arguments.is_empty() {
        arguments.push_str(", ");
    }
    write!(arguments, "i64 {budget}").map_err(|_| BackendFailure::TextEmission)?;
    let callee = recursion_budget_symbol(function.name());
    if abi.result().uses_destination() {
        write!(
            output,
            "  call void @{callee}({arguments})\n  ret void\n}}\n\n"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    } else {
        let result = llvm_type(program, abi.result().ty())?;
        write!(
            output,
            "  %wf.entry = call {result} @{callee}({arguments})\n  ret {result} %wf.entry\n}}\n\n"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    Ok(output)
}

/// The block a budgeted world tests its remaining levels in, and the block it
/// leaves for the sequential clone from.
const GRAIN_ENTRY_LABEL: &str = "par.grain";
const GRAIN_SPENT_LABEL: &str = "par.grain.spent";

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
            let element_type = program.element(element).ok_or(BackendFailure::InvalidIr)?;
            let llvm_element_type = llvm_type(program, element_type)?;
            for (index, value) in elements.iter().enumerate() {
                if index != 0 {
                    text.push_str(", ");
                }
                write!(
                    text,
                    "{llvm_element_type} {}",
                    global_constant_value(program, value, element_type)?
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
        // Pointer owners and the uniform opaque representation do not need a
        // named aggregate type.
        if nominal.is_tag_only_enum()
            || matches!(
                nominal.kind(),
                IrNominalKind::Box { .. }
                    | IrNominalKind::Arena { .. }
                    | IrNominalKind::ArenaStorage
                    | IrNominalKind::Opaque
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
            | IrNominalKind::Opaque => {
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
    MemoryMove,
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
    /// One immutable aggregate value's planned storage, shared only after
    /// complete control-flow interference checks.
    OwnedValue(usize),
    /// The bump extent one [BLK-2] reservation lays out in the reserving
    /// activation's own frame, at the byte extent and alignment its two type
    /// constants fix.
    ExtentStorage(IrValueId),
    ArrayFillIndex(IrValueId),
    Address(IrValueId),
    ArenaList(IrValueId),
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

/// Storage selected before emission, including values live across calls.
struct FunctionFrameContents<'plan> {
    storage: &'plan FunctionStoragePlan,
    result_slot: Option<usize>,
}

impl FunctionFramePlan {
    fn build(
        target: TargetLayout,
        program: &IrProgram<'_, '_, '_>,
        function: &IrFunction,
        contents: FunctionFrameContents<'_>,
    ) -> Result<Self, BackendFailure> {
        let FunctionFrameContents {
            storage,
            result_slot,
        } = contents;
        let mut specifications = Vec::new();
        let mut ordered = Vec::new();
        for (slot, ty) in storage.slots().iter().copied().enumerate() {
            if Some(slot) != result_slot
                && storage.destination(slot).is_none()
                && storage.field_destination(slot).is_none()
            {
                push_function_slot(
                    &mut specifications,
                    &mut ordered,
                    FunctionSlot::OwnedValue(slot),
                    TargetStorageType::source(ty),
                    None,
                )?;
            }
        }
        for block in function.blocks() {
            for instruction in block.instructions() {
                let IrInstruction::Define {
                    result, operation, ..
                } = instruction
                else {
                    continue;
                };
                match operation {
                    IrOperation::ArrayFill { .. } => {
                        push_function_slot(
                            &mut specifications,
                            &mut ordered,
                            FunctionSlot::ArrayFillIndex(*result),
                            TargetStorageType::integer(64),
                            None,
                        )?;
                    }
                    IrOperation::AddressOf { referent, .. } => {
                        let storage = TargetStorageType::source(referent.ty());
                        let key = FunctionSlot::Address(*result);
                        push_function_slot(&mut specifications, &mut ordered, key, storage, None)?;
                    }
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
                    _ => {}
                }
            }
        }
        let target_plan = plan_target_frame(target, program, &specifications)
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
struct FunctionEmitter<'program, 'state> {
    program: &'program IrProgram<'program, 'program, 'program>,
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
    storage: FunctionStoragePlan,
    result_slot: Option<usize>,
    /// Per-operation snapshots for legacy value consumers. Place operations
    /// read their actual storage directly; a snapshot never becomes an alias.
    materialized: HashMap<IrValueId, String>,
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
    /// Ordinary calls awaiting the group's join.
    handed_out: Vec<HandedOut>,
    /// The functions that have a sequential clone, when this emitter is
    /// rendering one.
    ///
    /// `None` is the ordinary lowering, which is every function of a default
    /// build and the overlapped half of a `--par` build. `Some` renders the
    /// clone world: no group is actualized, and a call to a function that also
    /// has a clone names the clone, so the world a call lands in is the world
    /// it was made from. The experimental refusal edge may enter a clone
    /// from ordinary code without changing its parameters or result ABI.
    sequential_clones: Option<&'state HashSet<u32>>,
    /// Existing clones callable from the opt-in refused-compute edge.
    refusal_clones: &'state HashSet<u32>,
    frontiers: &'state RecursiveFrontiers,
    /// The budgeted world this emission renders, or `None` for a function no
    /// family holds — which is every function of a default build.
    grain: Option<Grain>,
    /// The caller's remaining budget, as an operand: the value a call that
    /// stays inside this component carries. Fixed for the whole emission.
    grain_next: Option<String>,
}

/// What one function's emission shares with the rest of its module, and the
/// one choice that says which of the module's two worlds it is emitting into.
///
/// These travel together because they are all module-scope: intrinsic
/// declarations and thunks are collected across every function and rendered
/// once at the top, and the clone set is the same set for every function of the
/// module. Passing them as one named group keeps the emitter's own arguments —
/// the program and the function — the ones a reader has to
/// think about.
struct ModuleState<'state> {
    intrinsics: &'state mut BTreeSet<IntrinsicDeclaration>,
    parallel: &'state mut ParallelThunks,
    /// `None` emits the ordinary lowering; `Some` emits the sequential clone.
    sequential_clones: Option<&'state HashSet<u32>>,
    /// Existing clones callable from the opt-in refused-compute edge.
    refusal_clones: &'state HashSet<u32>,
    frontiers: &'state RecursiveFrontiers,
    grain: Option<Grain>,
}

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    fn new(
        program: &'program IrProgram<'_, '_, '_>,
        target: TargetLayout,
        function: &'program IrFunction,
        module: ModuleState<'state>,
    ) -> Result<Self, BackendFailure> {
        let ModuleState {
            intrinsics,
            parallel,
            sequential_clones,
            refusal_clones,
            frontiers,
            grain,
        } = module;
        let mut overlaps = Vec::new();
        let mut ordinary_lane_frames = HashMap::new();
        if sequential_clones.is_none() {
            for overlap in function.overlaps() {
                // One ordinary ABI, with a budget field only for a synthesized variant.
                let Some(frames) =
                    ordinary_overlap_lane_frames(program, target, function, overlap, &|callee| {
                        grain.is_some_and(|grain| frontiers.spends(callee, grain))
                    })?
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
        let overlap_handed_out = overlaps
            .iter()
            .flat_map(|overlap| overlap.handed_out().iter().copied())
            .collect();
        let overlap_join_sites = overlaps
            .iter()
            .filter_map(crate::IrOverlap::join_site)
            .collect();
        let storage =
            FunctionStoragePlan::build_in_world(program, function, sequential_clones.is_some())?;
        let result_slot = places::returned_storage_slot(function, &storage);
        let frame = FunctionFramePlan::build(
            target,
            program,
            function,
            FunctionFrameContents {
                storage: &storage,
                result_slot,
            },
        )?;
        let entry_prelude = frame.render(program)?;
        Ok(Self {
            program,
            function,
            intrinsics,
            incoming: Vec::new(),
            output: String::new(),
            entry_prelude,
            frame,
            storage,
            result_slot,
            materialized: HashMap::new(),
            temporary: 0,
            parallel,
            overlaps,
            overlap_handed_out,
            overlap_join_sites,
            ordinary_lane_frames,
            handed_out: Vec::new(),
            sequential_clones,
            refusal_clones,
            frontiers,
            grain,
            grain_next: None,
        })
    }

    fn is_overlap_join_site(&self, value: IrValueId) -> bool {
        self.overlap_join_sites.contains(&value)
    }

    /// The symbol one call names, and the budget operand it carries.
    ///
    /// In the clone world a call to a function that also has a clone names the
    /// clone, which is what keeps a clone's whole dynamic extent inside the
    /// sequential world. Inside a budgeted world a call that stays in the
    /// component names the callee's variant and spends one level; a call that
    /// leaves it enters that callee's ordinary symbol, which obtains a budget
    /// of its own. Other calls use the shared original.
    pub(super) fn callee_target(&self, ordinal: u32, name: &str) -> (String, Option<&str>) {
        match (self.sequential_clones, self.grain) {
            (Some(clones), _) if clones.contains(&ordinal) => (sequential_clone_symbol(name), None),
            (None, Some(grain)) => {
                let (symbol, spends) = self.frontiers.callee(ordinal, name, grain);
                (
                    symbol,
                    spends.then_some(self.grain_next.as_deref()).flatten(),
                )
            }
            _ => (source_symbol(name), None),
        }
    }

    pub(super) fn callee_symbol(&self, ordinal: u32, name: &str) -> String {
        self.callee_target(ordinal, name).0
    }

    /// The budget-carrying variant's entry: test the levels this activation
    /// was handed, and enter the sequential clone where none are left.
    ///
    /// One compare and one branch per activation above the cut, and nothing at
    /// all below it — the clone is the world this module already carries for a
    /// run with no pool, so no node under the cut pays a scheduler test, a
    /// null branch or a phi.
    ///
    /// Returns where the frame prelude belongs, which is this block when there
    /// is one: an `alloca` is promotable only in the entry block.
    fn emit_grain_entry(&mut self, abi: &FunctionAbi) -> Result<Option<usize>, BackendFailure> {
        if self.grain.is_none() {
            return Ok(None);
        }
        // A branch into the body needs the body to be enterable from one more
        // place. It always is: an IR entry block that were a jump target would
        // already carry phis in an LLVM entry block, which is malformed.
        if self.incoming.first().is_some_and(|edges| !edges.is_empty()) {
            return Err(BackendFailure::InvalidIr);
        }
        let arguments = ordinary_call_arguments(self.program, self.function, abi)?;
        let spent = RecursiveFrontiers::exhausted(self.function.name());
        let body = block_label(IrBlockId::from_index(0).map_err(|_| BackendFailure::InvalidIr)?);
        writeln!(self.output, "{GRAIN_ENTRY_LABEL}:").map_err(|_| BackendFailure::TextEmission)?;
        let anchor = self.output.len();
        writeln!(
            self.output,
            "  %wf.budget.next = sub i64 %wf.budget, 1\n  \
             %wf.grain = icmp sgt i64 %wf.budget, 0\n  \
             br i1 %wf.grain, label %{body}, label %{GRAIN_SPENT_LABEL}\n\
             {GRAIN_SPENT_LABEL}:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if abi.result().uses_destination() {
            writeln!(self.output, "  call void @{spent}({arguments})\n  ret void")
                .map_err(|_| BackendFailure::TextEmission)?;
        } else {
            let result = llvm_type(self.program, abi.result().ty())?;
            writeln!(
                self.output,
                "  %wf.spent = call {result} @{spent}({arguments})\n  ret {result} %wf.spent"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        self.grain_next = Some("%wf.budget.next".to_owned());
        Ok(Some(anchor))
    }

    fn emit(mut self) -> Result<String, BackendFailure> {
        let declaration = self.function.blocks().is_empty();
        let reachable = if declaration {
            Vec::new()
        } else {
            self.reachable_blocks()?
        };
        self.incoming = self.collect_incoming(&reachable)?;
        let abi = FunctionAbi::build(self.program, self.function)?;
        let symbol = match (self.sequential_clones, self.grain) {
            (Some(_), _) => sequential_clone_symbol(self.function.name()),
            (None, Some(_)) => recursion_budget_symbol(self.function.name()),
            (None, None) => source_symbol(self.function.name()),
        };
        write!(
            self.output,
            "{} {} @{symbol}(",
            if declaration { "declare" } else { "define" },
            if abi.result().uses_destination() {
                "void".to_owned()
            } else {
                llvm_type(self.program, abi.result().ty())?
            },
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let result_address = abi.result().uses_destination();
        if result_address {
            self.output.push_str("ptr %wf.result");
        }
        for (index, ((value, _), parameter)) in self
            .function
            .parameters()
            .iter()
            .zip(abi.parameters())
            .enumerate()
        {
            if index != 0 || result_address {
                self.output.push_str(", ");
            }
            if parameter.is_indirect() {
                write!(self.output, "ptr %wf.arg.v{}", value.ordinal())
                    .map_err(|_| BackendFailure::TextEmission)?;
                continue;
            }
            write!(
                self.output,
                "{} {}",
                llvm_type(self.program, parameter.ty())?,
                self.value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        if declaration {
            self.output.push_str(")\n\n");
            return Ok(self.output);
        }
        // The variant's hidden trailing budget. Legal because this definition
        // is synthesized and is named by no source call: a writer's own
        // signature is the entry's, which is emitted unchanged.
        if self.grain.is_some() {
            if !self.function.parameters().is_empty() || result_address {
                self.output.push_str(", ");
            }
            self.output.push_str("i64 %wf.budget");
        }
        self.output.push_str(") {\n");
        let mut prelude_anchor = self.emit_grain_entry(&abi)?;
        for (index, block) in self.function.blocks().iter().enumerate() {
            if !reachable[index] {
                continue;
            }
            self.materialized.clear();
            let block_id =
                IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
            writeln!(self.output, "{}:", block_label(block_id))
                .map_err(|_| BackendFailure::TextEmission)?;
            if index == 0 && prelude_anchor.is_none() {
                prelude_anchor = Some(self.output.len());
            }
            self.emit_block_parameters(block_id, block)?;
            if index == 0 {
                // A result can alias any consumed caller input. Snapshot
                // every other indirect input first, then initialize the one
                // entry group using the result. Scalar/address parameters
                // already arrived as SSA values before either pass.
                for writes_result in [false, true] {
                    for ((value, _), parameter) in
                        self.function.parameters().iter().zip(abi.parameters())
                    {
                        let uses_result = self.result_slot.is_some_and(|result_slot| {
                            self.storage.slot(*value).is_some_and(|slot| {
                                self.storage.allocation_root(slot) == result_slot
                            })
                        });
                        if !parameter.is_indirect() || uses_result != writes_result {
                            continue;
                        }
                        let destination = self.value_place(*value)?;
                        self.copy_storage(
                            parameter.ty(),
                            &format!("%wf.arg.v{}", value.ordinal()),
                            &destination,
                        )?;
                    }
                }
            }
            for (instruction_index, instruction) in block.instructions().iter().enumerate() {
                self.emit_instruction(block_id, instruction_index, instruction)?;
            }
            self.emit_terminator(block_id, block.terminator())?;
        }
        self.output.push_str("}\n\n");
        if !self.entry_prelude.is_empty() {
            let anchor = prelude_anchor.ok_or(BackendFailure::InvalidIr)?;
            self.output.insert_str(anchor, &self.entry_prelude);
        }
        Ok(self.output)
    }

    /// Source checking retains the conservative continuation of every loop
    /// [FN-1]. An executable loop with no break has no edge to that block,
    /// which may nevertheless carry lowered parameters and more dead CFG.
    /// Emit only the entry-reachable graph: a predecessor-free phi is not
    /// LLVM, and a dead cycle must not supply an incoming value to a live phi.
    fn reachable_blocks(&self) -> Result<Vec<bool>, BackendFailure> {
        let mut reachable = vec![false; self.function.blocks().len()];
        let mut pending = vec![0_usize];
        while let Some(index) = pending.pop() {
            let visited = reachable.get_mut(index).ok_or(BackendFailure::InvalidIr)?;
            if *visited {
                continue;
            }
            *visited = true;
            match self.function.blocks()[index].terminator() {
                IrTerminator::Jump { target, .. } => pending.push(target.index()),
                IrTerminator::Match { targets, .. } => {
                    pending.extend(targets.iter().map(|target| target.block().index()));
                }
                IrTerminator::Return { .. } | IrTerminator::Unreachable => {}
            }
        }
        Ok(reachable)
    }

    fn collect_incoming(&self, reachable: &[bool]) -> Result<Vec<Vec<Incoming>>, BackendFailure> {
        let mut incoming = vec![Vec::new(); self.function.blocks().len()];
        for (index, block) in self.function.blocks().iter().enumerate() {
            if !reachable[index] {
                continue;
            }
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
            if self.storage.slot(*parameter).is_some() {
                continue;
            }
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
                    )
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            self.output.push('\n');
        }
        Ok(())
    }

    /// Emit one ordinary instruction.
    fn emit_instruction(
        &mut self,
        block: IrBlockId,
        index: usize,
        instruction: &IrInstruction,
    ) -> Result<(), BackendFailure> {
        match instruction {
            IrInstruction::StoreBuffer {
                buffer,
                index,
                value,
            } => {
                self.materialize_operands([*buffer, *index, *value])?;
            }
            IrInstruction::StoreSlice {
                slice,
                index,
                value,
            } => {
                self.materialize_operands([*slice, *index, *value])?;
            }
            IrInstruction::Store { address, value, .. } => {
                self.materialize_operands([*address, *value])?;
            }
            IrInstruction::Drops(_) => {}
            IrInstruction::Define { .. } => {}
        }
        self.emit_instruction_body(block, index, instruction)?;
        Ok(())
    }

    fn emit_instruction_body(
        &mut self,
        _block: IrBlockId,
        _index: usize,
        instruction: &IrInstruction,
    ) -> Result<(), BackendFailure> {
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
            IrInstruction::Drops(drops) => self.emit_drops(drops),
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
        self.materialized.clear();
        if self.emit_place_definition(result, ty, operation)? {
            return Ok(());
        }
        self.materialize_operands(operation_operands(operation))?;
        self.emit_value_definition(result, ty, operation)?;
        if !self.overlap_handed_out.contains(&result) {
            self.save_value_result(result)?;
        }
        Ok(())
    }

    fn emit_value_definition(
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
            IrOperation::FullArrayConversion { value } => {
                self.emit_full_array_conversion(result, ty, *value)
            }
            IrOperation::ArrayIndex {
                root,
                offset,
                target_domain,
            } => self.emit_array_index(result, ty, *root, *offset, *target_domain),
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
            IrOperation::ConstantAddress { constant } => {
                let global = self
                    .program
                    .constant(*constant)
                    .ok_or(BackendFailure::InvalidIr)?;
                if !matches!(ty, IrType::Address(referent) if referent.ty() == global.ty()) {
                    return Err(BackendFailure::InvalidIr);
                }
                writeln!(
                    self.output,
                    "  {} = getelementptr inbounds {}, ptr {}, i64 0",
                    self.value_name(result),
                    llvm_type(self.program, global.ty())?,
                    constant_symbol(*constant)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            IrOperation::ProjectAddress {
                address,
                projection,
            } => self.emit_project_address(result, ty, *address, projection),
            IrOperation::Load { address, referent } => {
                self.emit_load(result, ty, *address, *referent)
            }
        }
    }

    fn emit_terminator(
        &mut self,
        block: IrBlockId,
        terminator: &IrTerminator,
    ) -> Result<(), BackendFailure> {
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
                self.emit_place_edge(*target, arguments, drops)?;
                writeln!(self.output, "  br label %{}", block_label(*target))
                    .map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Return { value, drops } => {
                let abi = FunctionAbi::build(self.program, self.function)?;
                if self.value_type(*value) != Some(abi.result().ty()) {
                    return Err(BackendFailure::InvalidIr);
                }
                if abi.result().uses_destination() {
                    self.store_value_at(*value, "%wf.result")?;
                    self.emit_drops(drops)?;
                    return writeln!(self.output, "  ret void")
                        .map_err(|_| BackendFailure::TextEmission);
                }
                self.emit_drops(drops)?;
                writeln!(
                    self.output,
                    "  ret {} {}",
                    llvm_type(self.program, abi.result().ty())?,
                    self.value_name(*value)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Match {
                scrutinee,
                enum_type,
                targets,
            } => {
                self.materialize_operands([*scrutinee])?;
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

    /// Validate the checked release and capture only content it actually
    /// reads. In particular, a no-op owner node requires no aggregate load.
    fn prepare_drop(&mut self, drop: IrDrop) -> Result<Option<String>, BackendFailure> {
        let actual = match drop.subject() {
            IrDropSubject::Value(value) => self.value_type(value),
            IrDropSubject::Place(address) => match self.value_type(address) {
                Some(IrType::Address(referent)) => Some(referent.ty()),
                _ => return Err(BackendFailure::InvalidIr),
            },
        };
        if actual != Some(drop.ty()) {
            return Err(BackendFailure::InvalidIr);
        }
        let reads_content = match drop.ty() {
            IrType::Slice { .. } => false,
            IrType::Array { .. }
            | IrType::FixedVector { .. }
            | IrType::Vector { .. }
            | IrType::Provider => type_requires_cleanup(self.program, drop.ty())?,
            IrType::Buffer { .. } => true,
            IrType::Nominal(nominal) if !self.nominal(nominal)?.is_tag_only_enum() => {
                match self.nominal(nominal)?.kind() {
                    // The checker supplied separate component records. The
                    // struct node must not recursively release them again.
                    IrNominalKind::Struct { .. } | IrNominalKind::Arena { .. } => false,
                    IrNominalKind::ArenaStorage => true,
                    IrNominalKind::Opaque => false,
                    IrNominalKind::Enum { .. } | IrNominalKind::Box { .. } => {
                        type_requires_cleanup(self.program, drop.ty())?
                    }
                }
            }
            _ => return Err(BackendFailure::InvalidIr),
        };
        if !reads_content {
            return Ok(None);
        }
        match drop.subject() {
            IrDropSubject::Value(value) => self.value_operand(value).map(Some),
            IrDropSubject::Place(address) => {
                let snapshot = format!("%{}", self.next_temporary()?);
                writeln!(
                    self.output,
                    "  {snapshot} = load {}, ptr {}",
                    llvm_type(self.program, drop.ty())?,
                    self.value_name(address)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                Ok(Some(snapshot))
            }
        }
    }

    fn emit_drops(&mut self, drops: &[IrDrop]) -> Result<(), BackendFailure> {
        // Capture the complete edge/group before its first effectful release,
        // just as lowering's former value snapshots did. Phi inputs are also
        // captured before this group; destination writes follow it.
        let snapshots = drops
            .iter()
            .map(|drop| self.prepare_drop(*drop))
            .collect::<Result<Vec<_>, _>>()?;
        for (drop, snapshot) in drops.iter().zip(snapshots) {
            if let Some(value) = snapshot {
                emit_value_cleanup(
                    self.program,
                    &mut self.output,
                    &mut self.temporary,
                    drop.ty(),
                    value,
                )?;
            }
            writeln!(self.output, "  ; drop {}", value_name(drop.operand()))
                .map_err(|_| BackendFailure::TextEmission)?;
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
        self.materialized
            .get(&value)
            .cloned()
            .unwrap_or_else(|| value_name(value))
    }
}

/// Fit every ordinary call's frame before selecting an overlap group.
fn ordinary_overlap_lane_frames(
    program: &IrProgram<'_, '_, '_>,
    target: TargetLayout,
    function: &IrFunction,
    overlap: &IrOverlap,
    carries_budget: &dyn Fn(u32) -> bool,
) -> Result<Option<Vec<(IrValueId, TargetAggregateLayout)>>, BackendFailure> {
    let mut frames = Vec::with_capacity(overlap.handed_out().len());
    for member in overlap.handed_out() {
        let Some(IrOperation::Call {
            function: ordinal, ..
        }) = definition_operation(function, *member)
        else {
            return Ok(None);
        };
        let ordinal = *ordinal;
        let callee = program
            .functions()
            .get(ordinal as usize)
            .ok_or(BackendFailure::InvalidIr)?;
        let Some(layout) =
            parallel_lane_frame_layout(target, program, callee, carries_budget(ordinal))
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

pub(crate) fn llvm_type(
    program: &IrProgram<'_, '_, '_>,
    ty: IrType,
) -> Result<String, BackendFailure> {
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
            llvm_type(
                program,
                program.element(element).ok_or(BackendFailure::InvalidIr)?
            )?
        )),
        IrType::Buffer { .. } | IrType::Slice { .. } => Ok("{ ptr, i64 }".to_owned()),
        // A `Vector` descriptor is `{ pointer, cap, len, head }`; a
        // `FixedVector` is its slots followed by `len` and `head`; a provider
        // is proof-only and carries at most its own base and cursor
        // [BLK-1, PROV-1, OP-9].
        IrType::Vector { .. } => Ok("{ ptr, i64, i64, i64 }".to_owned()),
        IrType::FixedVector { element, length } => Ok(format!(
            "{{ [{length} x {}], i64, i64 }}",
            llvm_type(
                program,
                program.element(element).ok_or(BackendFailure::InvalidIr)?
            )?
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
            if matches!(nominal.kind(), IrNominalKind::Opaque) {
                return Ok("{ i128, i128 }".to_owned());
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
/// where the block continues. Ordinary lane calls join newest first.
fn overlap_join_tail(overlaps: &[IrOverlap], result: IrValueId) -> Option<IrValueId> {
    overlaps
        .iter()
        .find(|overlap| overlap.join_site() == Some(result))?
        .handed_out()
        .first()
        .copied()
}

/// Account for every instruction that opens an LLVM block when naming phis.
fn block_exit_label(block_id: IrBlockId, block: &IrBlock, overlaps: &[IrOverlap]) -> String {
    let mut label = block_label(block_id);
    for (index, instruction) in block.instructions().iter().enumerate() {
        definition_exit_label(block_id, index, instruction, &mut label);
        if let IrInstruction::Define { result, .. } = instruction
            && let Some(last) = overlap_join_tail(overlaps, *result)
        {
            label = par_done_label(last);
        }
    }
    label
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

pub(crate) fn source_symbol(name: &str) -> String {
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
