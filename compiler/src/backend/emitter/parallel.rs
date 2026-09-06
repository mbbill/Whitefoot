//! Actualization of the permission judgment's overlap groups.
//!
//! For a group of sibling calls the checker permitted to overlap, every member
//! but the last is *handed out*: a lane is acquired, the call's arguments are
//! stored into that lane's frame, and the call — outlined into an internal
//! thunk over the frame — is published to the lane. The remaining member then
//! runs inline on the calling thread, and each handed-out member is joined
//! immediately after it — before the group's values are read and before any
//! exit edge.
//!
//! Both edges of the hand-out call the same monomorphized function on the same
//! arguments, so there is still exactly one lowering of the source call: the
//! thunk and the fallback are the same code reached two ways, and a lane that
//! is never granted computes the sequential result on the sequential schedule.
//! Nothing here consults a fact, a source proof statement, or a row; it
//! consumes the group the checker already judged.
//!
//! **Lane acquisition comes before the frame.** The frame belongs to the lane, not to
//! the calling function, and nothing about it is built until a lane has been
//! granted. An activation that is refused a lane executes a null test and its
//! own call: no stack slot, no argument spills, nothing the sequential
//! lowering did not already do. That is what keeps the recursion depth of a
//! `--par` build the recursion depth of the sequential build, whether the pool
//! is off or merely busy. The earlier shape — a frame in the calling
//! function's entry block — put a slot and its stores in *every* activation of
//! an eligible recursive function, which cost about four times the stack per
//! frame on a small one and turned a recursion that ran into a bare SIGSEGV.
//!
//! Handing a call to another thread does still cost the caller what a
//! parallel schedule costs: the lane handle is live across the inline member,
//! and the thunk is a second caller of the handed-out function whose arguments
//! come out of memory, so an interprocedural fact about those arguments —
//! a constant, say — does not survive into the `--par` build.
//!
//! **Two worlds, selected once.** What the paragraph above describes is a cost
//! of *overlapping*, and a program that asked for lanes and did not get them
//! should not pay it. So a `--par` module carries the lowering above and, for
//! every function on a path from the entry to a handed-out call, a second copy
//! that actualizes nothing — the sequential lowering, byte for byte, so that
//! every transform the default build gets fires on it. The bootstrap asks the
//! runtime once whether this run was asked for a pool, and enters one world or
//! the other; neither ever calls into the other, so nothing below that branch
//! tests anything again. [`sequential_clone_set`] carries why the second copy has to
//! exist, why once-per-process is the only selection that is safe here, and
//! why the set is exactly that closure.
//!
//! **Symbol reservation.** A source function is emitted as `wf_` followed by
//! its own IDENT, and [FORM-3] spells IDENT `[a-z][a-z0-9_]*`, so no source
//! name can produce a symbol whose first character after `wf_` is an
//! underscore. Every symbol this module and the runtime introduce therefore
//! begins `wf__par_`: the prefix is unreachable from source, so a program that
//! declares `fn par_try_fork(...)` still compiles and links exactly as it did
//! before this module existed. That is a reserved namespace, not a name check
//! — nothing here inspects a source function's spelling.

use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use super::{BackendFailure, FunctionEmitter, llvm_type, source_symbol, value_name};
use super::{Qualification, TargetLayout};
use crate::{
    IrCompletionStep, IrFunction, IrInstruction, IrOperation, IrProgram, IrSynthesis, IrType,
    IrValueId,
};

/// The counted loop's index type [FN-1], which fixes every width question the
/// split could otherwise have.
const U64: IrType = IrType::Integer {
    width: 64,
    signed: false,
};

/// One [`IrOperation::LoopSplit`] site, as the two renderings read it.
pub(super) struct LoopSplitSite<'ir> {
    pub(super) splitter: u32,
    pub(super) chunk: u32,
    pub(super) seed: IrValueId,
    pub(super) lower: IrValueId,
    pub(super) upper: IrValueId,
    pub(super) captures: &'ir [IrValueId],
    pub(super) weight: u64,
}

/// The Windows parallel ABI as external obligations.
///
/// COFF modules do not carry the sequential weak definitions below.  A
/// Windows module that hands out work therefore cannot link unless the
/// runtime supplies the strong protocol; this is the compile-time half of the
/// fail-closed backend contract.  That runtime is now the same
/// `sched/entry.c` every other target links -- Windows is done as shared code
/// (design section 7) -- so what this fail-closed choice selects is a staging
/// predicate rather than a second implementation.
pub(crate) const PARALLEL_RUNTIME_DECLARATIONS: &str = "declare ptr @wf__par_acquire_lane(i64)\ndeclare void @wf__par_publish(ptr, ptr)\ndeclare void @wf__par_publish_staged(ptr, ptr)\ndeclare void @wf__par_join(ptr)\ndeclare void @wf__par_release(ptr)\n";

/// The fail-closed Windows declaration of the once-per-process backend query.
pub(crate) const PARALLEL_POOL_QUERY_DECLARATION: &str = "declare i32 @wf__par_pool_active(i32)\n";

/// The fail-closed Windows declaration of the loop split budget query.
pub(crate) const PARALLEL_SPLIT_BUDGET_DECLARATION: &str =
    "declare i64 @wf__par_split_budget(i64, i64)\n";

/// A non-Windows module's own definition of the lane protocol: acquire no lane,
/// ever.
///
/// A non-Windows module that hands work out carries a *weak* sequential answer
/// to every runtime entry point, so it is a complete program on its own: with
/// no runtime linked, every acquisition is refused, so no frame is ever built, no
/// task is ever published, and every handed-out call runs on its own thread at
/// its own fallback edge — exactly today's schedule. Linking the runtime
/// replaces these entries with its strong definitions, and only then can a lane be
/// granted. Windows deliberately takes the external declarations above and
/// cannot link without the scheduler core.
///
/// On the optional-runtime targets, the alternative — plain declarations —
/// would make the runtime a link obligation of every path that ever builds a
/// Whitefoot program rather than an option of the paths that want lanes. The
/// Windows production contract deliberately chooses that obligation.
pub(crate) const PARALLEL_RUNTIME_FALLBACK: &str = "define weak ptr @wf__par_acquire_lane(i64 %bytes) {\nentry:\n  ret ptr null\n}\n\ndefine weak void @wf__par_publish(ptr %frame, ptr %fn) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__par_publish_staged(ptr %frame, ptr %fn) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__par_join(ptr %frame) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__par_release(ptr %frame) {\nentry:\n  ret void\n}\n\n";

/// The first line of [`PARALLEL_RUNTIME_FALLBACK`], and so the marker a
/// non-Windows link path reads.
pub(crate) const PARALLEL_LANE_ACQUISITION_SYMBOL: &str =
    "define weak ptr @wf__par_acquire_lane(i64 %bytes)";

/// The first Windows declaration of the same ABI, used as that target's link
/// marker.
pub(crate) const PARALLEL_LANE_ACQUISITION_DECLARATION: &str =
    "declare ptr @wf__par_acquire_lane(i64)";

/// True when this emitted module hands work out, so linking the parallel
/// runtime would let it take lanes.
///
/// A module with no permitted overlap group names none of the runtime's
/// symbols, so nothing of the runtime — not one thread, not one atomic —
/// reaches a program that has no use for it. On the optional POSIX path, a
/// module that does hand work out still links and runs sequentially without
/// the runtime. On Windows, the same predicate recognizes external
/// declarations and is a hard link obligation.
pub fn module_requires_parallel_runtime(module: &str) -> bool {
    module.contains(PARALLEL_LANE_ACQUISITION_SYMBOL)
        || module.contains(PARALLEL_LANE_ACQUISITION_DECLARATION)
        || module.contains(super::checkpoint::DECLARATION)
}

/// The runtime's answer to "was this run asked for a pool", put once per
/// process, and a non-Windows module's own weak answer of "no".
///
/// The optional-runtime path carries this for the same reason it carries the
/// task protocol entry points: with no runtime linked, no pool can ever start, so the
/// honest answer is a constant zero and the program is complete on its own.
/// Windows emits the external declaration above instead. The query is not part
/// of the lane protocol — it takes no frame, moves no work, and starts nothing
/// — so it remains separate from the task protocol signatures.
pub(crate) const PARALLEL_POOL_QUERY_FALLBACK: &str =
    "define weak i32 @wf__par_pool_active(i32 %minimum_workers) {\nentry:\n  ret i32 0\n}\n\n";

/// CPU hand-outs need two workers to buy overlap; a staged may-suspend call
/// can make progress alongside another such call on one scheduler worker.
/// Only reachable functions in the clone closure affect the bootstrap, so an
/// unused I/O function cannot change a compute program's one-worker path.
pub(crate) fn overlap_minimum_workers(
    program: &IrProgram<'_, '_, '_>,
    clones: &HashSet<u32>,
) -> Option<u32> {
    if clones.is_empty() {
        return None;
    }
    let stages_io = clones.iter().any(|ordinal| {
        program.functions()[*ordinal as usize]
            .driven_completion_pipeline()
            .is_some_and(crate::IrCompletionPipeline::lane_handout)
    });
    Some(if stages_io { 1 } else { 2 })
}

/// The runtime's answer to "how many times may a split of this span halve",
/// and a non-Windows module's own weak answer of "not at all".
///
/// Carried by the optional-runtime path for the same reason as the query above:
/// with no runtime linked there are no lanes, so the honest allowance is zero
/// and a splitter that gets it descends straight to its leaf — one call, then
/// the loop. Windows leaves the external query unresolved until native link.
///
/// It is a separate definition rather than another lane-protocol entry point
/// because it takes no frame, publishes nothing, and moves no work; keeping it
/// apart also leaves the task protocol signatures' bytes exactly as they were.
pub(crate) const PARALLEL_SPLIT_BUDGET_FALLBACK: &str =
    "define weak i64 @wf__par_split_budget(i64 %span, i64 %weight) {\nentry:\n  ret i64 0\n}\n\n";

/// The symbol one function's sequential clone is emitted under.
///
/// It lives in the same reserved `wf__par_` namespace as the runtime's own
/// symbols, which [FORM-3] puts out of reach of any source IDENT, so cloning a
/// function can never collide with a function the writer declared.
pub(crate) fn sequential_clone_symbol(name: &str) -> String {
    format!("wf__par_seq_{name}")
}

/// The functions that need a sequential clone: every function on some path
/// from the entry to a handed-out call.
///
/// **Why a second copy exists at all.** The hand-out rejoins its granted and
/// refused edges through a phi, so the callee's result flows into a phi rather
/// than into the caller's return. That is invisible on a heavy call and
/// decisive on a light one: it takes `fib`'s second recursion out of tail
/// position, and with it LLVM's accumulator tail-recursion elimination, which
/// turns the sequential build's second call into a loop. The measured price
/// with the pool off was 2.96x on `fib(38)` — a program paying for
/// parallelism it never activated. No rearrangement of one lowering can serve
/// both: the phi is what actualization *is*, and the transform requires its
/// absence. So the module carries both lowerings and selects between them.
///
/// **Why the selection is safe.** It is made once per process, from whether the
/// run asked for a pool, and never again. Without one every acquisition is refused for
/// the whole process, so the two worlds compute on exactly the same schedule and
/// the choice between them is a choice of machine code, not of semantics. On
/// the optional-runtime path, a run that asks for a pool and cannot start one
/// has every acquisition refused; Windows instead terminates when the native pool is
/// first required. A *per-task* demand signal would be a different thing
/// entirely, and was measured killing the scheduler it was meant to help: the
/// shared word it needs costs two contended read-modify-writes per task, which
/// took the fine-grain oracle cell from 0.4905 s to 0.9254 s. Nothing here reads
/// a per-task signal, and the two worlds never call each other. The optional
/// runtime path may answer that no pool started; a Windows parallel binary is
/// instead required to initialize its linked native pool or terminate at its
/// first pool operation.
///
/// **Why this set and not another.** A function outside it has the same body
/// in both worlds — no hand-out is reachable from it, so nothing about its
/// lowering depends on which world called it — and both worlds call the one
/// copy. Cloning it would be bytes with no reader. The set is a property of
/// the call graph and the permission judgment, never of a name or a source
/// shape.
///
/// Empty when no hand-out is reachable from the entry, which includes every
/// default compilation: the default build carries no overlap group at all, so
/// there is one world and this changes nothing about it.
pub(crate) fn sequential_clone_set(program: &IrProgram<'_, '_, '_>) -> HashSet<u32> {
    let functions = program.functions();
    let mut callees: Vec<Vec<u32>> = vec![Vec::new(); functions.len()];
    let mut hands_out = Vec::with_capacity(functions.len());
    for (ordinal, function) in functions.iter().enumerate() {
        // A driven [PAR-3] loop whose staged call is handed to a lane is a
        // hand-out like any other, so the function that carries it needs the
        // second copy for the world that was not asked for a pool.
        let stages_a_lane = function
            .driven_completion_pipeline()
            .is_some_and(crate::IrCompletionPipeline::lane_handout);
        hands_out.push(
            stages_a_lane
                || function.overlaps().iter().any(|overlap| {
                    overlap.handed_out().iter().any(|member| {
                        function.blocks().iter().any(|block| {
                            block.instructions().iter().any(|instruction| {
                                matches!(
                                    instruction,
                                    IrInstruction::Define {
                                        result,
                                        operation: IrOperation::Call { .. },
                                        ..
                                    } if result == member
                                )
                            })
                        })
                    })
                }),
        );
        for block in function.blocks() {
            for instruction in block.instructions() {
                let IrInstruction::Define { operation, .. } = instruction else {
                    continue;
                };
                match operation {
                    IrOperation::Call { function, .. } => callees[ordinal].push(*function),
                    // A split reaches both halves: the overlapped world calls
                    // the splitter and the sequential world calls the chunk, so
                    // each world's reachability has to hold the one it uses.
                    IrOperation::LoopSplit {
                        splitter, chunk, ..
                    } => {
                        callees[ordinal].push(*splitter);
                        callees[ordinal].push(*chunk);
                    }
                    _ => {}
                }
            }
        }
    }

    let mut reachable = HashSet::new();
    let mut pending = vec![program.main_ordinal()];
    while let Some(ordinal) = pending.pop() {
        if !reachable.insert(ordinal) {
            continue;
        }
        if let Some(called) = callees.get(ordinal as usize) {
            pending.extend(called.iter().copied());
        }
    }

    // The other direction: a function needs a clone when a hand-out is
    // reachable *from* it, so the walk runs the call graph backwards from the
    // functions that hand work out.
    let mut callers: Vec<Vec<u32>> = vec![Vec::new(); functions.len()];
    for (ordinal, called) in callees.iter().enumerate() {
        let Ok(ordinal) = u32::try_from(ordinal) else {
            continue;
        };
        for callee in called {
            if let Some(entry) = callers.get_mut(*callee as usize) {
                entry.push(ordinal);
            }
        }
    }
    let mut reaches_hand_out = HashSet::new();
    let mut pending: Vec<u32> = hands_out
        .iter()
        .enumerate()
        .filter(|(_, hands_out)| **hands_out)
        .filter_map(|(ordinal, _)| u32::try_from(ordinal).ok())
        .collect();
    while let Some(ordinal) = pending.pop() {
        if !reaches_hand_out.insert(ordinal) {
            continue;
        }
        if let Some(calling) = callers.get(ordinal as usize) {
            pending.extend(calling.iter().copied());
        }
    }

    // A chunk reaches no hand-out of its own, and still needs a clone: it is
    // what the sequential world calls at a split site, and giving that world
    // its own copy leaves each copy with exactly one caller, which is what lets
    // the loop be inlined back into it and makes the sequential world free.
    //
    // A splitter is the other way round. It exists only in the overlapped
    // world, so its clone would be unreachable — and would be a second caller
    // of the chunk's clone, costing exactly the inlining above.
    reachable
        .iter()
        .copied()
        .filter(|ordinal| {
            match functions
                .get(*ordinal as usize)
                .and_then(IrFunction::synthesis)
            {
                Some(IrSynthesis::Splitter) => false,
                Some(IrSynthesis::Chunk) => true,
                None => reaches_hand_out.contains(ordinal),
            }
        })
        .collect()
}

/// The outlined thunks of one module, in emission order.
#[derive(Debug, Default)]
pub(crate) struct ParallelThunks {
    definitions: String,
    count: u32,
    /// Whether any emitted function asked the runtime for a split allowance, so
    /// a module that splits no loop names that symbol nowhere.
    queries_split_budget: bool,
}

impl ParallelThunks {
    /// The thunk definitions this module needs, or empty when it hands out
    /// nothing.
    pub(crate) fn definitions(&self) -> &str {
        &self.definitions
    }

    pub(crate) const fn is_used(&self) -> bool {
        self.count != 0
    }

    pub(crate) const fn queries_split_budget(&self) -> bool {
        self.queries_split_budget
    }

    /// Records one thunk body and returns the symbol that names it.
    fn register(&mut self, body: impl FnOnce(&str) -> String) -> Result<String, BackendFailure> {
        let symbol = format!("@wf__par_thunk_{}", self.count);
        self.count = self
            .count
            .checked_add(1)
            .ok_or(BackendFailure::CounterOverflow)?;
        self.definitions.push_str(&body(&symbol));
        Ok(symbol)
    }
}

/// One hand-out awaiting its join, in the order the group hands them out.
#[derive(Clone, Debug)]
pub(crate) struct ComputeHandedOut {
    /// The value the joined call defines: a granted lane's result read out of
    /// the frame, or a refused lane's own call, whichever edge ran.
    result: IrValueId,
    result_type: IrType,
    /// The frame's LLVM struct type, `{ arguments..., result }`.
    frame_type: String,
    /// The acquired lane's frame, or null when no lane was granted. It is both
    /// the storage the thunk reads and the handle the join names.
    frame: String,
    /// The frame field the result occupies: the argument count.
    result_field: usize,
    /// The call the refused edge makes: the same symbol on the same operands
    /// the thunk calls, rendered once so the two edges cannot drift apart.
    callee: String,
    arguments: String,
}

/// One independently admitted operation awaiting the overlap join.  Compute
/// calls use the lane frame protocol; direct finite file operations use the
/// typed completion protocol and never put writer code on a file helper.
#[derive(Clone, Debug)]
pub(crate) enum HandedOut {
    Compute(ComputeHandedOut),
    Completion(Box<super::completion::CompletionHandedOut>),
}

/// The staged call of a [PAR-3] loop the pipeline drives, where that call is a
/// may-suspend call handed to a compute lane.
///
/// This is the third thing a pipeline slot can hold and the second thing that
/// can be in flight across a loop's back edge. A submitted system operation
/// leaves the target its record's address; a staged lane hand-out leaves the
/// pool its frame's address, and the ring holds that address for the iteration
/// exactly as it holds a record for the other form. The callee then runs on a
/// pool stack — stolen by a worker, or run by the offering thread's own
/// scheduler loop once that thread's stack parks — and parks on its own I/O
/// without holding the loop
/// (`research/investigations/io-model/PARK-ON-MISS.md` §2, §5).
///
/// One thing is rendered here rather than at two sites: the frame's shape.
/// The published edge stores into it, the drain reads the result out of it,
/// and a drift between the two would be a wrong load from a live frame.
#[derive(Clone, Debug)]
pub(crate) struct StagedLane {
    /// The call's result, which the drain defines and the remainder reads.
    pub(super) result: IrValueId,
    pub(super) result_type: IrType,
    pub(super) result_llvm: String,
    /// The callee's ordinal and symbol, and the call's operand values.
    pub(super) callee_ordinal: u32,
    pub(super) callee: String,
    pub(super) arguments: Vec<IrValueId>,
    /// The frame's LLVM struct type, `{ arguments..., result }`, and the field
    /// the result occupies.
    pub(super) field_types: Vec<String>,
    pub(super) frame_type: String,
    pub(super) result_field: usize,
    /// How many iterations the ring holds.
    pub(super) slots: u64,
    /// The lane frame this world offers, in bytes, or `None` where it offers
    /// none: a sequential clone actualizes nothing, and a frame the runtime's
    /// slot cannot hold would be refused every lane at run time anyway. Either
    /// way the call runs where it is written and the ring element holds its
    /// answer, which is the permitted sequential form.
    pub(super) frame_bytes: Option<u64>,
    /// The issue stage's own values the drain reads back, `(origin, reload)`.
    pub(super) carries: Vec<(IrValueId, IrValueId)>,
}

impl FunctionEmitter<'_, '_> {
    /// Hands one member of an overlap group to a worker lane.
    ///
    /// Acquires a lane first and builds the frame only inside the granted edge,
    /// so a refused hand-out leaves nothing behind but a null pointer. Defines
    /// nothing: the call's value comes into existence at the join, which is
    /// the only place it is known to have been computed.
    pub(super) fn emit_handed_out_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        function: u32,
        arguments: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        let target = self
            .program
            .functions()
            .get(function as usize)
            .ok_or(BackendFailure::InvalidIr)?;
        if target.result() != ty || target.parameters().len() != arguments.len() {
            return Err(BackendFailure::InvalidIr);
        }
        let mut field_types = Vec::with_capacity(arguments.len() + 1);
        let mut operands = Vec::with_capacity(arguments.len());
        for (argument, (_, parameter_type)) in arguments.iter().zip(target.parameters()) {
            if self.value_type(*argument) != Some(*parameter_type) {
                return Err(BackendFailure::InvalidIr);
            }
            let parameter = llvm_type(self.program, *parameter_type)?;
            operands.push(format!("{parameter} {}", self.value_name(*argument)));
            field_types.push(parameter);
        }
        let result_type = llvm_type(self.program, ty)?;
        let result_field = field_types.len();
        field_types.push(result_type.clone());
        let frame_type = format!("{{ {} }}", field_types.join(", "));
        let frame_layout = self
            .ordinary_lane_frames
            .get(&result)
            .copied()
            .ok_or(BackendFailure::InvalidIr)?;

        let callee = source_symbol(target.name());
        let thunk = self.parallel.register(|symbol| {
            thunk_definition(symbol, &frame_type, &field_types, &callee, &result_type)
        })?;

        // Target layout already computed the exact complete aggregate before
        // this function emitted any text. Passing that proved constant avoids
        // forming an address from `null` merely to ask LLVM for the same size.
        let frame = format!("%{}", self.next_temporary()?);
        let granted = format!("%{}", self.next_temporary()?);
        let offer = par_offer_label(result);
        let offered = par_offered_label(result);
        writeln!(
            self.output,
            "  {frame} = call ptr @wf__par_acquire_lane(i64 {})\n  {granted} = icmp ne ptr {frame}, null\n  br i1 {granted}, label %{offer}, label %{offered}\n{offer}:",
            frame_layout.size()
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        for (index, operand) in operands.iter().enumerate() {
            let field = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {field} = getelementptr inbounds {frame_type}, ptr {frame}, i32 0, i32 {index}\n  store {operand}, ptr {field}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        writeln!(
            self.output,
            "  call void @wf__par_publish(ptr {frame}, ptr {thunk})\n  br label %{offered}\n{offered}:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.handed_out.push(HandedOut::Compute(ComputeHandedOut {
            result,
            result_type: ty,
            frame_type,
            frame,
            result_field,
            callee,
            arguments: operands.join(", "),
        }));
        Ok(())
    }

    /// The ring element this block addresses for one staged-lane reservation.
    ///
    /// The array is an entry-block reservation and the index is the slot the
    /// block being emitted owns — the issue stage's count for a submission,
    /// the drain's own slot for a retirement — so a hand-out addresses the
    /// element its iteration took and a join addresses the element it is
    /// retiring, exactly as a completion ring is addressed.
    fn staged_ring_element(
        &mut self,
        key: super::FunctionSlot,
        element_type: &str,
        slots: u64,
    ) -> Result<String, BackendFailure> {
        let array = self.frame.slot(key)?;
        let slot = match self.block_slot {
            Some(value) => self.value_name(value),
            None if slots == 1 => "0".to_owned(),
            None => return Err(BackendFailure::MisaddressedCompletionSlot),
        };
        let element = format!("%{}", self.next_temporary()?);
        writeln!(
            self.output,
            "  {element} = getelementptr inbounds [{slots} x {element_type}], ptr {array}, \
             i64 0, i64 {slot}"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(element)
    }

    /// Stores one issue-stage value into the ring element its iteration owns.
    ///
    /// A carrying block is emitted once and reached once per iteration, so a
    /// value the prologue defines is gone by the time the drain runs that
    /// iteration's remainder. This is where it is kept, and the drain's own
    /// load is the only reader.
    pub(super) fn store_staged_carry(&mut self, value: IrValueId) -> Result<(), BackendFailure> {
        let Some(plan) = self.staged_lane.clone() else {
            return Ok(());
        };
        if !plan.carries.iter().any(|(origin, _)| *origin == value) {
            return Ok(());
        }
        let ty = self.value_type(value).ok_or(BackendFailure::InvalidIr)?;
        let rendered = llvm_type(self.program, ty)?;
        let element = self.staged_ring_element(
            super::FunctionSlot::StagedCarry(value),
            &rendered,
            plan.slots,
        )?;
        writeln!(
            self.output,
            "  store {rendered} {}, ptr {element}",
            self.value_name(value)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(())
    }

    /// Hands the staged call of a driven [PAR-3] loop to a compute lane.
    ///
    /// Acquires a frame first and fills it only inside the granted edge, as
    /// every hand-out does; what differs is where the frame's address goes and
    /// where the answer is read. The address goes into the iteration's ring
    /// element, because the loop's back edge is crossed with the call still
    /// running, and the answer is read in the exact drain rather than here.
    /// A refused acquisition runs the same call on the same operands where it
    /// is written and leaves its answer in the same ring element, so the drain
    /// has one thing to do either way and the two edges cannot drift apart.
    pub(super) fn emit_staged_lane_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        function: u32,
        arguments: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        let plan = self.staged_lane.clone().ok_or(BackendFailure::InvalidIr)?;
        if plan.result != result
            || plan.callee_ordinal != function
            || plan.arguments != arguments
            || plan.result_type != ty
        {
            return Err(BackendFailure::InvalidIr);
        }
        let target = self
            .program
            .functions()
            .get(function as usize)
            .ok_or(BackendFailure::InvalidIr)?;
        if target.result() != ty || target.parameters().len() != arguments.len() {
            return Err(BackendFailure::InvalidIr);
        }
        let mut operands = Vec::with_capacity(arguments.len());
        for (argument, (_, parameter_type)) in arguments.iter().zip(target.parameters()) {
            if self.value_type(*argument) != Some(*parameter_type) {
                return Err(BackendFailure::InvalidIr);
            }
            operands.push(format!(
                "{} {}",
                llvm_type(self.program, *parameter_type)?,
                self.value_name(*argument)
            ));
        }
        let rendered_arguments = operands.join(", ");
        let result_type = plan.result_llvm.clone();
        let answer = self.staged_ring_element(
            super::FunctionSlot::StagedResult(result),
            &result_type,
            plan.slots,
        )?;
        let callee = plan.callee.clone();
        let Some(frame_bytes) = plan.frame_bytes else {
            let inline = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {inline} = call {result_type} @{callee}({rendered_arguments})\n  \
                 store {result_type} {inline}, ptr {answer}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        };
        let thunk = self.parallel.register(|symbol| {
            thunk_definition(
                symbol,
                &plan.frame_type,
                &plan.field_types,
                &callee,
                &result_type,
            )
        })?;
        let held =
            self.staged_ring_element(super::FunctionSlot::StagedFrame(result), "ptr", plan.slots)?;
        let frame = format!("%{}", self.next_temporary()?);
        let granted = format!("%{}", self.next_temporary()?);
        let offer = par_staged_offer_label(result);
        let inline = par_staged_inline_label(result);
        let offered = par_staged_offered_label(result);
        writeln!(
            self.output,
            "  {frame} = call ptr @wf__par_acquire_lane(i64 {frame_bytes})\n  \
             store ptr {frame}, ptr {held}\n  \
             {granted} = icmp ne ptr {frame}, null\n  \
             br i1 {granted}, label %{offer}, label %{inline}\n{offer}:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let frame_type = plan.frame_type.clone();
        for (index, operand) in operands.iter().enumerate() {
            let field = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {field} = getelementptr inbounds {frame_type}, ptr {frame}, i32 0, i32 {index}\n  \
                 store {operand}, ptr {field}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        let refused = format!("%{}", self.next_temporary()?);
        writeln!(
            self.output,
            "  call void @wf__par_publish_staged(ptr {frame}, ptr {thunk})\n  \
             br label %{offered}\n\
             {inline}:\n  \
             {refused} = call {result_type} @{callee}({rendered_arguments})\n  \
             store {result_type} {refused}, ptr {answer}\n  \
             br label %{offered}\n\
             {offered}:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(())
    }

    /// Retires the staged lane hand-out of the slot this drain block owns, and
    /// reads back that iteration's own values.
    ///
    /// Emitted before the drain's own instructions, because the remainder they
    /// are is what reads the retired result. A granted frame is joined, read,
    /// and given back; a refused one already left its answer in the ring. The
    /// remainder then runs with the result in iteration order, exactly as it
    /// runs on a submitted operation's outcome.
    pub(super) fn emit_staged_lane_retirement(&mut self) -> Result<(), BackendFailure> {
        let Some(plan) = self.staged_lane.clone() else {
            return Ok(());
        };
        for (origin, reload) in &plan.carries {
            let ty = self.value_type(*origin).ok_or(BackendFailure::InvalidIr)?;
            if self.value_type(*reload) != Some(ty) {
                return Err(BackendFailure::InvalidIr);
            }
            let rendered = llvm_type(self.program, ty)?;
            let element = self.staged_ring_element(
                super::FunctionSlot::StagedCarry(*origin),
                &rendered,
                plan.slots,
            )?;
            writeln!(
                self.output,
                "  {} = load {rendered}, ptr {element}",
                self.value_name(*reload)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        let result_type = plan.result_llvm.clone();
        let answer = self.staged_ring_element(
            super::FunctionSlot::StagedResult(plan.result),
            &result_type,
            plan.slots,
        )?;
        if plan.frame_bytes.is_none() {
            writeln!(
                self.output,
                "  {} = load {result_type}, ptr {answer}",
                value_name(plan.result)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        }
        let held = self.staged_ring_element(
            super::FunctionSlot::StagedFrame(plan.result),
            "ptr",
            plan.slots,
        )?;
        let frame = format!("%{}", self.next_temporary()?);
        let condition = format!("%{}", self.next_temporary()?);
        let direct = format!("%{}", self.next_temporary()?);
        let waited = format!("%{}", self.next_temporary()?);
        let field = format!("%{}", self.next_temporary()?);
        let inline = par_staged_refused_label(plan.result);
        let wait = par_staged_wait_label(plan.result);
        let done = par_staged_done_label(plan.result);
        writeln!(
            self.output,
            "  {frame} = load ptr, ptr {held}\n  \
             {condition} = icmp eq ptr {frame}, null\n  \
             br i1 {condition}, label %{inline}, label %{wait}\n\
             {inline}:\n  {direct} = load {result_type}, ptr {answer}\n  \
             br label %{done}\n\
             {wait}:\n  call void @wf__par_join(ptr {frame})\n  \
             {field} = getelementptr inbounds {}, ptr {frame}, i32 0, i32 {}\n  \
             {waited} = load {result_type}, ptr {field}\n  \
             call void @wf__par_release(ptr {frame})\n  br label %{done}\n\
             {done}:\n  {} = phi {result_type} [ {direct}, %{inline} ], [ {waited}, %{wait} ]",
            plan.frame_type,
            plan.result_field,
            value_name(plan.result),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(())
    }

    /// Renders one permitted counted loop [PAR-2 candidate] into the world
    /// being emitted.
    ///
    /// The sequential world calls the chunk — the loop itself, seeded with the
    /// accumulator's incoming value — and is therefore the code the loop always
    /// had, behind one call its only caller inlines back. The overlapped world
    /// asks the runtime once, at loop entry, how far a split of this span may
    /// descend, and calls the splitter with that allowance. Nothing is asked
    /// per iteration, and neither world tests anything the other does.
    pub(super) fn emit_loop_split(
        &mut self,
        result: IrValueId,
        ty: IrType,
        split: &LoopSplitSite<'_>,
    ) -> Result<(), BackendFailure> {
        let result_type = llvm_type(self.program, ty)?;
        let mut arguments = Vec::with_capacity(split.captures.len() + 4);
        arguments.push(format!("{result_type} {}", self.value_name(split.seed)));
        for endpoint in [split.lower, split.upper] {
            if self.value_type(endpoint) != Some(U64) {
                return Err(BackendFailure::InvalidIr);
            }
            arguments.push(format!("i64 {}", self.value_name(endpoint)));
        }
        for capture in split.captures {
            let capture_type = self.value_type(*capture).ok_or(BackendFailure::InvalidIr)?;
            arguments.push(format!(
                "{} {}",
                llvm_type(self.program, capture_type)?,
                self.value_name(*capture)
            ));
        }

        let target = if self.sequential_clones.is_some() {
            split.chunk
        } else {
            split.splitter
        };
        let function = self
            .program
            .functions()
            .get(target as usize)
            .ok_or(BackendFailure::InvalidIr)?;
        // The site's operands against the callee's declared parameters, exactly
        // as a handed-out call checks its own. One lowering builds both lists
        // from one computation, so a mismatch is a defect in that lowering
        // rather than a shape this has to render; the point of the check is
        // that such a defect stops here instead of reaching the assembler as
        // type-mismatched text.
        let declared = function.parameters();
        // The splitter takes the allowance as one further parameter, which the
        // overlapped world appends below; every other operand is already in
        // `arguments`.
        let expected = declared
            .len()
            .checked_sub(usize::from(self.sequential_clones.is_none()))
            .ok_or(BackendFailure::InvalidIr)?;
        if expected != arguments.len() {
            return Err(BackendFailure::InvalidIr);
        }
        if function.result() != ty
            || declared.first().map(|(_, ty)| *ty) != Some(ty)
            || split
                .captures
                .iter()
                .zip(declared.get(3..).unwrap_or_default())
                .any(|(capture, (_, parameter))| self.value_type(*capture) != Some(*parameter))
        {
            return Err(BackendFailure::InvalidIr);
        }
        let callee = self.callee_symbol(target, function.name());
        if self.sequential_clones.is_some() {
            writeln!(
                self.output,
                "  {} = call {result_type} @{callee}({})",
                value_name(result),
                arguments.join(", ")
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        }

        // The span, computed so an inverted range asks for nothing rather than
        // for a bogus 2^63 one. The splitter guards the same way; this keeps a
        // wrapped width out of the allowance as well as out of the descent.
        let width = format!("%{}", self.next_temporary()?);
        let ascending = format!("%{}", self.next_temporary()?);
        let span = format!("%{}", self.next_temporary()?);
        let budget = format!("%{}", self.next_temporary()?);
        let lower = self.value_name(split.lower);
        let upper = self.value_name(split.upper);
        writeln!(
            self.output,
            "  {width} = sub i64 {upper}, {lower}\n  \
             {ascending} = icmp ugt i64 {upper}, {lower}\n  \
             {span} = select i1 {ascending}, i64 {width}, i64 0\n  \
             {budget} = call i64 @wf__par_split_budget(i64 {span}, i64 {})",
            split.weight
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.parallel.queries_split_budget = true;
        arguments.push(format!("i64 {budget}"));
        writeln!(
            self.output,
            "  {} = call {result_type} @{callee}({})",
            value_name(result),
            arguments.join(", ")
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(())
    }

    /// Completes every hand-out of the group whose last member just ran.
    ///
    /// A granted lane is waited for, read, and given back; a refused one runs
    /// the same call on this thread. Either way the group's values exist from
    /// here on, and no exit edge of the block is reachable before this point.
    /// The order the queue is completed in is [`compute_join_order`]'s and
    /// nothing here decides it.
    pub(super) fn emit_overlap_joins(
        &mut self,
        join_site: IrValueId,
    ) -> Result<(), BackendFailure> {
        if !self.is_overlap_join_site(join_site) {
            return Ok(());
        }
        let queue = compute_join_order(std::mem::take(&mut self.handed_out), |pending| {
            matches!(pending, HandedOut::Compute(_))
        });
        for pending in queue {
            let pending = match pending {
                HandedOut::Compute(pending) => pending,
                HandedOut::Completion(pending) => {
                    self.emit_completion_join(*pending)?;
                    continue;
                }
            };
            let condition = format!("%{}", self.next_temporary()?);
            let refused = format!("%{}", self.next_temporary()?);
            let waited = format!("%{}", self.next_temporary()?);
            let field = format!("%{}", self.next_temporary()?);
            let inline = par_inline_label(pending.result);
            let wait = par_wait_label(pending.result);
            let done = par_done_label(pending.result);
            let result_type = llvm_type(self.program, pending.result_type)?;
            let ComputeHandedOut {
                frame,
                frame_type,
                callee,
                arguments,
                result_field,
                ..
            } = &pending;
            writeln!(
                self.output,
                "  {condition} = icmp eq ptr {frame}, null\n  \
                 br i1 {condition}, label %{inline}, label %{wait}\n\
                 {inline}:\n  {refused} = call {result_type} @{callee}({arguments})\n  \
                 br label %{done}\n\
                 {wait}:\n  call void @wf__par_join(ptr {frame})\n  \
                 {field} = getelementptr inbounds {frame_type}, ptr {frame}, i32 0, i32 {result_field}\n  \
                 {waited} = load {result_type}, ptr {field}\n  \
                 call void @wf__par_release(ptr {frame})\n  br label %{done}\n\
                 {done}:\n  {} = phi {result_type} [ {refused}, %{inline} ], [ {waited}, %{wait} ]",
                value_name(pending.result),
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        Ok(())
    }
}

/// The order a group's members are joined in: its compute members newest
/// first, its completion members exactly where they were published.
///
/// The compute deque is Chase-Lev. Its owner pushes and pops at the newest
/// end while thieves take from the oldest, so what has been stolen is always
/// a prefix of the publish order and what the owner still holds is the
/// suffix. Joining in publish order therefore asks for the oldest entry
/// first, the one entry the owner cannot reach without digging past
/// everything it published after it. Joining the compute members newest
/// first instead — publish J1, J2, J3, join J3, J2, J1 — means that at every
/// compute join the target is either the newest entry of the owner's deque or
/// it has already been stolen, and never present but buried under something
/// newer. The runtime needs no notion of a group: it looks at the newest end
/// once. (The property is stated for a join taken on the target's home lane
/// with nothing else having pushed onto that lane in between; where that
/// fails the join simply parks, which costs one park and nothing else.)
///
/// A completion member holds no deque entry, so the deque places no
/// constraint on where it is joined and it keeps the position it was
/// published at. Only the compute members move, and only among the positions
/// they already occupy: the queue `[C1, IO1, C2, C3]` is joined as
/// `[C3, IO1, C2, C1]`. Join order is not observable — [PAR-1] fixes every
/// value to the source-order result — so this is an emitter choice, and it is
/// made here once. Every site that needs a group's join order consumes this
/// function rather than encoding one of its own.
pub(super) fn compute_join_order<T>(
    mut members: Vec<T>,
    is_compute: impl Fn(&T) -> bool,
) -> Vec<T> {
    let compute: Vec<usize> = members
        .iter()
        .enumerate()
        .filter(|(_, member)| is_compute(member))
        .map(|(position, _)| position)
        .collect();
    // Reverse the compute members in place across the positions they hold,
    // which leaves every other position untouched.
    let mut oldest = 0;
    let mut newest = compute.len();
    while oldest + 1 < newest {
        newest -= 1;
        members.swap(compute[oldest], compute[newest]);
        oldest += 1;
    }
    members
}

/// One outlined call over its frame.
fn thunk_definition(
    symbol: &str,
    frame_type: &str,
    field_types: &[String],
    callee: &str,
    result_type: &str,
) -> String {
    let mut body = format!("define internal void {symbol}(ptr %frame) {{\nentry:\n");
    let mut rendered = Vec::with_capacity(field_types.len() - 1);
    for (index, field_type) in field_types.iter().enumerate().take(field_types.len() - 1) {
        let _ = write!(
            body,
            "  %p{index} = getelementptr inbounds {frame_type}, ptr %frame, i32 0, i32 {index}\n  %a{index} = load {field_type}, ptr %p{index}\n"
        );
        rendered.push(format!("{field_type} %a{index}"));
    }
    let field = field_types.len() - 1;
    let _ = write!(
        body,
        "  %result = call {result_type} @{callee}({})\n  %slot = getelementptr inbounds {frame_type}, ptr %frame, i32 0, i32 {field}\n  store {result_type} %result, ptr %slot\n  ret void\n}}\n\n",
        rendered.join(", ")
    );
    body
}

/// The label a granted lane's frame is filled and published in.
fn par_offer_label(value: IrValueId) -> String {
    format!("par.offer.v{}", value.ordinal())
}

/// The label both edges of lane acquisition continue in, and so the block the inline
/// member of the group runs in.
fn par_offered_label(value: IrValueId) -> String {
    format!("par.offered.v{}", value.ordinal())
}

/// The label a refused lane runs the call in.
fn par_inline_label(value: IrValueId) -> String {
    format!("par.inline.v{}", value.ordinal())
}

/// The label a granted lane is waited for in.
fn par_wait_label(value: IrValueId) -> String {
    format!("par.wait.v{}", value.ordinal())
}

/// The label the joined value is read in. It is the block a later phi names as
/// its predecessor when the join is the last split of its block.
pub(super) fn par_done_label(value: IrValueId) -> String {
    format!("par.done.v{}", value.ordinal())
}

/// The staged hand-out's four labels. They are its own rather than the compute
/// group's because a staged hand-out splits its block twice — once where it is
/// offered and once where it is retired — and the two are different blocks.
fn par_staged_offer_label(value: IrValueId) -> String {
    format!("par.staged.offer.v{}", value.ordinal())
}

/// The label a refused acquisition runs the call in, at the staged point.
fn par_staged_inline_label(value: IrValueId) -> String {
    format!("par.staged.inline.v{}", value.ordinal())
}

/// The label both edges of the acquisition continue in, and so the block the
/// rest of the issue stage runs in.
pub(super) fn par_staged_offered_label(value: IrValueId) -> String {
    format!("par.staged.offered.v{}", value.ordinal())
}

/// The label the drain reads a refused iteration's own answer in.
fn par_staged_refused_label(value: IrValueId) -> String {
    format!("par.staged.refused.v{}", value.ordinal())
}

/// The label the drain joins a granted frame in.
fn par_staged_wait_label(value: IrValueId) -> String {
    format!("par.staged.wait.v{}", value.ordinal())
}

/// The label the retired result is defined in, and so the block the drain's
/// own remainder runs in.
pub(super) fn par_staged_done_label(value: IrValueId) -> String {
    format!("par.staged.done.v{}", value.ordinal())
}

/// The staged lane hand-out this world emits for this function, or `None`
/// where the function drives no such loop.
///
/// The plan is built once, before any text is written, because the frame's
/// shape has to be the same at the offer and at the retirement and those are
/// two different blocks. `hands_out` is the world: a sequential clone
/// actualizes nothing, so it plans no frame and every iteration runs its call
/// where it is written — the permitted sequential form, and the same one a
/// frame too large for the runtime's slot takes.
pub(super) fn staged_lane_plan(
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    target: TargetLayout,
    function: &IrFunction,
    pipeline: Option<&crate::IrCompletionPipeline>,
    completion_steps: &HashMap<IrValueId, IrCompletionStep>,
    hands_out: bool,
) -> Result<Option<StagedLane>, BackendFailure> {
    let Some(pipeline) = pipeline.filter(|pipeline| pipeline.lane_handout()) else {
        return Ok(None);
    };
    let Some(result) = pipeline.driven_result() else {
        return Ok(None);
    };
    // The step is what selects the hand-out, exactly as it selects a typed
    // adapter's submission: a descriptor whose step does not submit keeps the
    // ordinary call.
    if !completion_steps
        .get(&result)
        .is_some_and(crate::IrCompletionStep::submit)
    {
        return Ok(None);
    }
    let Some(IrOperation::Call {
        function: callee_ordinal,
        arguments,
    }) = super::definition_operation(function, result)
    else {
        return Ok(None);
    };
    let callee = program
        .functions()
        .get(*callee_ordinal as usize)
        .ok_or(BackendFailure::InvalidIr)?;
    let result_type = function
        .value_type(result)
        .ok_or(BackendFailure::InvalidIr)?;
    if callee.result() != result_type || callee.parameters().len() != arguments.len() {
        return Err(BackendFailure::InvalidIr);
    }
    let mut field_types = Vec::with_capacity(arguments.len() + 1);
    for (_, parameter_type) in callee.parameters() {
        field_types.push(llvm_type(program, *parameter_type)?);
    }
    let result_llvm = llvm_type(program, result_type)?;
    let result_field = field_types.len();
    field_types.push(result_llvm.clone());
    let frame_bytes = if hands_out {
        super::parallel_lane_frame_layout(target, qualification, program, callee)
            .map_err(BackendFailure::TargetLayout)?
            .map(|layout| layout.size())
    } else {
        None
    };
    Ok(Some(StagedLane {
        result,
        result_type,
        result_llvm,
        callee_ordinal: *callee_ordinal,
        callee: source_symbol(callee.name()),
        arguments: arguments.clone(),
        frame_type: format!("{{ {} }}", field_types.join(", ")),
        field_types,
        result_field,
        slots: pipeline.slots(),
        frame_bytes,
        carries: pipeline.staged_carries().to_vec(),
    }))
}

#[cfg(test)]
mod join_order_tests {
    use super::compute_join_order;

    /// One member of a group's publish queue, kept abstract because
    /// `compute_join_order` is: all it may ask of a member is whether it is a
    /// compute member.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Member {
        Compute(u32),
        Completion(u32),
    }

    fn joined(queue: &[Member]) -> Vec<Member> {
        compute_join_order(queue.to_vec(), |member| {
            matches!(member, Member::Compute(_))
        })
    }

    /// The three queues design §4 states the rule with.
    ///
    /// Compute members are joined newest first; a completion member holds no
    /// deque entry and is joined where it was published, so the permutation
    /// touches only the compute positions.
    #[test]
    fn compute_members_reverse_and_completion_members_hold_their_positions() {
        use Member::{Completion, Compute};

        assert_eq!(
            joined(&[Compute(1), Completion(1), Compute(2), Compute(3)]),
            vec![Compute(3), Completion(1), Compute(2), Compute(1)],
            "the compute members reverse across the positions they hold"
        );
        assert_eq!(
            joined(&[Completion(1), Compute(1), Completion(2)]),
            vec![Completion(1), Compute(1), Completion(2)],
            "one compute member has nothing to reverse with"
        );
        assert_eq!(
            joined(&[Compute(1), Compute(2)]),
            vec![Compute(2), Compute(1)],
            "the newest published compute member is joined first"
        );
    }

    /// The order is a permutation: every member is joined exactly once, and a
    /// queue with no compute member to move is returned as it came.
    #[test]
    fn the_order_joins_every_member_exactly_once() {
        use Member::{Completion, Compute};

        let queue = [
            Compute(1),
            Completion(1),
            Compute(2),
            Completion(2),
            Compute(3),
        ];
        let mut order = joined(&queue);
        assert_eq!(
            order,
            vec![
                Compute(3),
                Completion(1),
                Compute(2),
                Completion(2),
                Compute(1)
            ]
        );
        order.sort_by_key(|member| match member {
            Compute(index) => (0, *index),
            Completion(index) => (1, *index),
        });
        let mut expected = queue.to_vec();
        expected.sort_by_key(|member| match member {
            Compute(index) => (0, *index),
            Completion(index) => (1, *index),
        });
        assert_eq!(order, expected, "no member is dropped or duplicated");

        let completions = [Completion(1), Completion(2)];
        assert_eq!(joined(&completions), completions.to_vec());
        assert_eq!(joined(&[]), Vec::new());
    }
}
