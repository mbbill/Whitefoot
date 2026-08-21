//! Actualization of the permission judgment's overlap groups.
//!
//! For a group of sibling calls the checker permitted to overlap, every member
//! but the last is *handed out*: its arguments are stored into a stack frame,
//! the call itself is outlined into an internal thunk over that frame, and the
//! thunk is offered to a worker lane. The remaining member then runs inline on
//! the calling thread, and each handed-out member is joined immediately after
//! it — before the group's values are read and before any exit edge.
//!
//! Both edges of the hand-out call the same monomorphized function on the same
//! arguments, so there is still exactly one lowering of the source call: the
//! thunk and the fallback are the same code reached two ways, and a lane that
//! is never granted computes the sequential result on the sequential schedule.
//! Nothing here consults a fact, a claim disposition, or a row; it consumes
//! the group the checker already judged.
//!
//! The thunk's frame is an ordinary stack slot of the calling function, so a
//! recursive body that hands out at every level gives each activation its own
//! frame, exactly as its own arguments already are.

use std::fmt::Write;

use super::{BackendFailure, FunctionEmitter, llvm_type, source_symbol, value_name};
use crate::{IrType, IrValueId};

/// The C source of the parallel runtime, embedded so that every path that
/// links a Whitefoot executable links the same bytes.
pub const PARALLEL_RUNTIME_SOURCE: &str = include_str!("../par_runtime.c");

/// The module's own definition of the lane offer: refuse every lane.
///
/// A module that hands work out carries a *weak* sequential answer to both
/// runtime entry points, so it is a complete program on its own: with no
/// runtime linked, every offer is refused, every join is a no-op, and every
/// handed-out call runs on its own thread at its own fallback edge — exactly
/// today's schedule. Linking the runtime replaces both with its strong
/// definitions, and only then can a lane be granted.
///
/// The alternative — plain declarations — would make the runtime a link
/// obligation of every path that ever builds a Whitefoot program rather than
/// an option of the paths that want lanes, and would turn a program that
/// merely *could* overlap into one that cannot be linked without it. The
/// permission is never an obligation, so neither is its runtime.
pub(crate) const PARALLEL_RUNTIME_FALLBACK: &str = "define weak ptr @wf_par_try_fork(ptr %fn, ptr %arg) {\nentry:\n  ret ptr null\n}\n\ndefine weak void @wf_par_join(ptr %handle) {\nentry:\n  ret void\n}\n\n";

/// The first line of [`PARALLEL_RUNTIME_FALLBACK`], and so the marker a link
/// path reads: one definition, so the text a module carries and the text a
/// linker looks for cannot drift apart.
pub(crate) const PARALLEL_TRY_FORK_SYMBOL: &str =
    "define weak ptr @wf_par_try_fork(ptr %fn, ptr %arg)";

/// True when this emitted module hands work out, so linking the parallel
/// runtime would let it take lanes.
///
/// A module with no permitted overlap group names none of the runtime's
/// symbols, so nothing of the runtime — not one thread, not one atomic —
/// reaches a program that has no use for it. A module that does hand work out
/// still links and still runs correctly without the runtime; this only says
/// that linking it is what makes the lanes reachable.
pub fn module_requires_parallel_runtime(module: &str) -> bool {
    module.contains(PARALLEL_TRY_FORK_SYMBOL)
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
        let symbol = format!("@wf_par_thunk_{}", self.count);
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
    /// The value the joined call defines, loaded out of the frame at the join.
    result: IrValueId,
    result_type: IrType,
    /// The frame's LLVM struct type, `{ arguments..., result }`.
    frame_type: String,
    /// The stack slot holding this activation's frame.
    frame: String,
    /// The lane handle, or null when no lane was granted.
    handle: String,
    thunk: String,
    /// The frame field the result occupies: the argument count.
    result_field: usize,
}

impl FunctionEmitter<'_, '_> {
    /// Hands one member of an overlap group to a worker lane.
    ///
    /// Emits the frame stores and the lane offer, and defines nothing: the
    /// call's value comes into existence at the join, which is the only place
    /// it is known to have been computed.
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
        for (argument, (_, parameter_type)) in arguments.iter().zip(target.parameters()) {
            if self.value_type(*argument) != Some(*parameter_type) {
                return Err(BackendFailure::InvalidIr);
            }
            field_types.push(llvm_type(self.program, *parameter_type)?);
        }
        let result_type = llvm_type(self.program, ty)?;
        let result_field = field_types.len();
        field_types.push(result_type.clone());
        let frame_type = format!("{{ {} }}", field_types.join(", "));

        let callee = source_symbol(target.name());
        let thunk = self.parallel.register(|symbol| {
            thunk_definition(symbol, &frame_type, &field_types, &callee, &result_type)
        })?;

        let frame = self.entry_slot(&frame_type)?;
        for (index, argument) in arguments.iter().enumerate() {
            let field = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {field} = getelementptr inbounds {frame_type}, ptr {frame}, i32 0, i32 {index}\n  store {} {}, ptr {field}",
                field_types[index],
                self.value_name(*argument)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        let handle = format!("%{}", self.next_temporary()?);
        writeln!(
            self.output,
            "  {handle} = call ptr @wf_par_try_fork(ptr {thunk}, ptr {frame})"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.handed_out.push(HandedOut {
            result,
            result_type: ty,
            frame_type,
            frame,
            handle,
            thunk,
            result_field,
        });
        Ok(())
    }

    /// Completes every hand-out of the group whose last member just ran.
    ///
    /// A granted lane is waited for; a refused one runs the same thunk on this
    /// thread. Either way the group's values exist from here on, and no exit
    /// edge of the block is reachable before this point.
    pub(super) fn emit_overlap_joins(
        &mut self,
        join_site: IrValueId,
    ) -> Result<(), BackendFailure> {
        if !self.is_overlap_join_site(join_site) {
            return Ok(());
        }
        for pending in std::mem::take(&mut self.handed_out) {
            let condition = format!("%{}", self.next_temporary()?);
            let field = format!("%{}", self.next_temporary()?);
            let inline = par_inline_label(pending.result);
            let wait = par_wait_label(pending.result);
            let done = par_done_label(pending.result);
            let result_type = llvm_type(self.program, pending.result_type)?;
            writeln!(
                self.output,
                "  {condition} = icmp eq ptr {}, null\n  br i1 {condition}, label %{inline}, label %{wait}\n{inline}:\n  call void {}(ptr {})\n  br label %{done}\n{wait}:\n  call void @wf_par_join(ptr {})\n  br label %{done}\n{done}:\n  {field} = getelementptr inbounds {}, ptr {}, i32 0, i32 {}\n  {} = load {result_type}, ptr {field}",
                pending.handle,
                pending.thunk,
                pending.frame,
                pending.handle,
                pending.frame_type,
                pending.frame,
                pending.result_field,
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

/// The label a refused lane runs the thunk in.
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
