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
//! **Symbol reservation.** A source function is emitted as `wf_` followed by
//! its own IDENT, and [FORM-3] spells IDENT `[a-z][a-z0-9_]*`, so no source
//! name can produce a symbol whose first character after `wf_` is an
//! underscore. Every symbol this module and the runtime introduce therefore
//! begins `wf__par_`: the prefix is unreachable from source, so a program that
//! declares `fn par_try_fork(...)` still compiles and links exactly as it did
//! before this module existed. That is a reserved namespace, not a name check
//! — nothing here inspects a source function's spelling.

use std::fmt::Write;

use super::{BackendFailure, FunctionEmitter, llvm_type, source_symbol, value_name};
use crate::{IrType, IrValueId};

/// The C source of the parallel runtime, embedded so that every path that
/// links a Whitefoot executable links the same bytes.
pub const PARALLEL_RUNTIME_SOURCE: &str = include_str!("../par_runtime.c");

/// The module's own definition of the lane protocol: claim no lane, ever.
///
/// A module that hands work out carries a *weak* sequential answer to every
/// runtime entry point, so it is a complete program on its own: with no
/// runtime linked, every claim is refused, so no frame is ever built, no task
/// is ever published, and every handed-out call runs on its own thread at its
/// own fallback edge — exactly today's schedule. Linking the runtime replaces
/// all four with its strong definitions, and only then can a lane be granted.
///
/// The alternative — plain declarations — would make the runtime a link
/// obligation of every path that ever builds a Whitefoot program rather than
/// an option of the paths that want lanes, and would turn a program that
/// merely *could* overlap into one that cannot be linked without it. The
/// permission is never an obligation, so neither is its runtime.
pub(crate) const PARALLEL_RUNTIME_FALLBACK: &str = "define weak ptr @wf__par_claim(i64 %bytes) {\nentry:\n  ret ptr null\n}\n\ndefine weak void @wf__par_publish(ptr %frame, ptr %fn) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__par_join(ptr %frame) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__par_release(ptr %frame) {\nentry:\n  ret void\n}\n\n";

/// The first line of [`PARALLEL_RUNTIME_FALLBACK`], and so the marker a link
/// path reads: one definition, so the text a module carries and the text a
/// linker looks for cannot drift apart.
pub(crate) const PARALLEL_CLAIM_SYMBOL: &str = "define weak ptr @wf__par_claim(i64 %bytes)";

/// True when this emitted module hands work out, so linking the parallel
/// runtime would let it take lanes.
///
/// A module with no permitted overlap group names none of the runtime's
/// symbols, so nothing of the runtime — not one thread, not one atomic —
/// reaches a program that has no use for it. A module that does hand work out
/// still links and still runs correctly without the runtime; this only says
/// that linking it is what makes the lanes reachable.
pub fn module_requires_parallel_runtime(module: &str) -> bool {
    module.contains(PARALLEL_CLAIM_SYMBOL)
}

/// The outlined thunks of one module, in emission order.
#[derive(Debug, Default)]
pub(crate) struct ParallelThunks {
    definitions: String,
    count: u32,
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
pub(crate) struct HandedOut {
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
        self.handed_out.push(HandedOut {
            result,
            result_type: ty,
            frame_type,
            frame,
            result_field,
            callee,
            arguments: operands.join(", "),
        });
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
            let condition = format!("%{}", self.next_temporary()?);
            let refused = format!("%{}", self.next_temporary()?);
            let waited = format!("%{}", self.next_temporary()?);
            let field = format!("%{}", self.next_temporary()?);
            let inline = par_inline_label(pending.result);
            let wait = par_wait_label(pending.result);
            let done = par_done_label(pending.result);
            let result_type = llvm_type(self.program, pending.result_type)?;
            let HandedOut {
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
