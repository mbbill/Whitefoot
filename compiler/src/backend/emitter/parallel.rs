//! Actualization of the permission judgment's overlap groups.
//!
//! For a group of sibling calls the checker permitted to overlap, every member
//! but the last is *handed out*: a lane is claimed, the call's arguments are
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
//! Nothing here consults a fact, a claim disposition, or a row; it consumes
//! the group the checker already judged.
//!
//! **The claim comes before the frame.** The frame belongs to the lane, not to
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

use std::collections::HashSet;
use std::fmt::Write;

use super::{BackendFailure, FunctionEmitter, llvm_type, source_symbol, value_name};
use crate::{IrFunction, IrInstruction, IrOperation, IrProgram, IrSynthesis, IrType, IrValueId};

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

/// The C source of the parallel runtime, embedded so that every path that
/// links a Whitefoot executable links the same bytes.
pub const PARALLEL_RUNTIME_SOURCE: &str = include_str!("../par_runtime.c");

/// The same runtime with stackless writer-frame stealing compiled in. Link
/// paths select these bytes only when the emitted module can publish a writer
/// frame, so pure compute and ordinary direct completion pay no empty queue
/// probe.
pub const PARALLEL_COMPLETION_RUNTIME_SOURCE: &str = concat!(
    "#define WF_PAR_WITH_WRITER_SCHEDULER 1\n",
    include_str!("../par_runtime.c")
);

/// The native Windows worker-pool runtime for a pure-compute module.
///
/// Although the Windows driver currently links the completion units in every
/// executable, this form does not probe their empty writer queue in the hot
/// steal loop.
pub const PARALLEL_WINDOWS_RUNTIME_SOURCE: &str = include_str!("../par_runtime_windows.c");

/// The native Windows worker-pool runtime with writer-frame helping enabled.
/// A module that actualizes compute work and can suspend a stackless writer
/// selects these bytes so ready writers run on the same scheduler lanes.
pub const PARALLEL_WINDOWS_COMPLETION_RUNTIME_SOURCE: &str = concat!(
    "#define WF_PAR_WITH_WRITER_SCHEDULER 1\n",
    include_str!("../par_runtime_windows.c")
);

/// The Windows parallel ABI as external obligations.
///
/// COFF modules do not carry the sequential weak definitions below.  A
/// Windows module that hands out work therefore cannot link unless the native
/// runtime supplies the strong protocol; this is the compile-time half of the
/// fail-closed backend contract.
pub(crate) const PARALLEL_RUNTIME_DECLARATIONS: &str = "declare ptr @wf__par_claim(i64)\ndeclare void @wf__par_publish(ptr, ptr)\ndeclare void @wf__par_join(ptr)\ndeclare void @wf__par_release(ptr)\n";

/// The fail-closed Windows declaration of the once-per-process backend query.
pub(crate) const PARALLEL_POOL_QUERY_DECLARATION: &str = "declare i32 @wf__par_pool_active()\n";

/// The fail-closed Windows declaration of the loop split budget query.
pub(crate) const PARALLEL_SPLIT_BUDGET_DECLARATION: &str =
    "declare i64 @wf__par_split_budget(i64, i64)\n";

/// A non-Windows module's own definition of the lane protocol: claim no lane,
/// ever.
///
/// A non-Windows module that hands work out carries a *weak* sequential answer
/// to every runtime entry point, so it is a complete program on its own: with
/// no runtime linked, every claim is refused, so no frame is ever built, no
/// task is ever published, and every handed-out call runs on its own thread at
/// its own fallback edge — exactly today's schedule. Linking the runtime
/// replaces all four with its strong definitions, and only then can a lane be
/// granted. Windows deliberately takes the external declarations above and
/// cannot link without its native pool.
///
/// On the optional-runtime targets, the alternative — plain declarations —
/// would make the runtime a link obligation of every path that ever builds a
/// Whitefoot program rather than an option of the paths that want lanes. The
/// Windows production contract deliberately chooses that obligation.
pub(crate) const PARALLEL_RUNTIME_FALLBACK: &str = "define weak ptr @wf__par_claim(i64 %bytes) {\nentry:\n  ret ptr null\n}\n\ndefine weak void @wf__par_publish(ptr %frame, ptr %fn) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__par_join(ptr %frame) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__par_release(ptr %frame) {\nentry:\n  ret void\n}\n\n";

/// The first line of [`PARALLEL_RUNTIME_FALLBACK`], and so the marker a
/// non-Windows link path reads.
pub(crate) const PARALLEL_CLAIM_SYMBOL: &str = "define weak ptr @wf__par_claim(i64 %bytes)";

/// The first Windows declaration of the same ABI, used as that target's link
/// marker.
pub(crate) const PARALLEL_CLAIM_DECLARATION: &str = "declare ptr @wf__par_claim(i64)";

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
    module.contains(PARALLEL_CLAIM_SYMBOL) || module.contains(PARALLEL_CLAIM_DECLARATION)
}

/// The runtime's answer to "was this run asked for a pool", put once per
/// process, and a non-Windows module's own weak answer of "no".
///
/// The optional-runtime path carries this for the same reason it carries the
/// four entry points: with no runtime linked, no pool can ever start, so the
/// honest answer is a constant zero and the program is complete on its own.
/// Windows emits the external declaration above instead. The query is not part
/// of the lane protocol — it takes no frame, moves no work, and starts nothing
/// — so it remains separate from the four protocol signatures.
pub(crate) const PARALLEL_POOL_QUERY_FALLBACK: &str =
    "define weak i32 @wf__par_pool_active() {\nentry:\n  ret i32 0\n}\n\n";

/// The runtime's answer to "how many times may a split of this span halve",
/// and a non-Windows module's own weak answer of "not at all".
///
/// Carried by the optional-runtime path for the same reason as the query above:
/// with no runtime linked there are no lanes, so the honest allowance is zero
/// and a splitter that gets it descends straight to its leaf — one call, then
/// the loop. Windows leaves the external query unresolved until native link.
///
/// It is a separate definition rather than a fifth lane-protocol entry point
/// because it takes no frame, publishes nothing, and moves no work; keeping it
/// apart also leaves the four protocol signatures' bytes exactly as they were.
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
/// run asked for a pool, and never again. Without one every claim is refused for
/// the whole process, so the two worlds compute on exactly the same schedule and
/// the choice between them is a choice of machine code, not of semantics. On
/// the optional-runtime path, a run that asks for a pool and cannot start one
/// has every claim refused; Windows instead terminates when the native pool is
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
        hands_out.push(function.overlaps().iter().any(|overlap| {
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
        }));
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
    stackless_runtime: bool,
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

    pub(crate) const fn requires_runtime(&self) -> bool {
        self.is_used() || self.stackless_runtime
    }

    pub(crate) fn request_stackless_runtime(&mut self) {
        self.stackless_runtime = true;
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
    /// The claimed lane's frame, or null when no lane was granted. It is both
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

impl FunctionEmitter<'_, '_> {
    /// Hands one member of an overlap group to a worker lane.
    ///
    /// Claims a lane first and builds the frame only inside the granted edge,
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

        let callee = source_symbol(target.name());
        let thunk = self.parallel.register(|symbol| {
            thunk_definition(symbol, &frame_type, &field_types, &callee, &result_type)
        })?;

        // The frame's size is LLVM's own answer for the frame's type, so the
        // bound the lane checks is the layout the thunk reads, not a number
        // this backend computed beside it.
        let frame = format!("%{}", self.next_temporary()?);
        let granted = format!("%{}", self.next_temporary()?);
        let offer = par_offer_label(result);
        let offered = par_offered_label(result);
        writeln!(
            self.output,
            "  {frame} = call ptr @wf__par_claim(i64 ptrtoint (ptr getelementptr ({frame_type}, ptr null, i32 1) to i64))\n  {granted} = icmp ne ptr {frame}, null\n  br i1 {granted}, label %{offer}, label %{offered}\n{offer}:"
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
    pub(super) fn emit_overlap_joins(
        &mut self,
        join_site: IrValueId,
    ) -> Result<(), BackendFailure> {
        if !self.is_overlap_join_site(join_site) {
            return Ok(());
        }
        for pending in std::mem::take(&mut self.handed_out) {
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

/// The label both edges of the claim continue in, and so the block the inline
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
