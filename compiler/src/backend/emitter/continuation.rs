//! Experimental switched-resume lowering of the existing checked functions.
//!
//! Function effects select representation; names and source shapes do not.
//! The first host keeps the serial source schedule. Staged task publication,
//! checkpoints, and asynchronous cleanup are separate integration work, so
//! this emitter is currently exposed only for experimental LLVM linking.

use super::*;

// Qualified by the C experiment host's size/alignment assertions. One node
// suffices for this serial schedule; it is never reused while registered.
pub(super) const WAITER_BYTES: u64 = 40;
pub(super) const WAITER_ALIGN: u64 = 8;

pub(super) fn symbol(name: &str) -> String {
    format!("wf__coro_{name}")
}

pub(super) fn tail_label(block: IrBlockId) -> String {
    format!("wf.coro.tail.{}", block.index())
}

pub(super) fn wait_done_label(value: IrValueId) -> String {
    format!("wf.coro.wait.v{}.done", value.ordinal())
}

// The host knows no LLVM frame offsets. Ordinary C entries resume and test
// opaque frames; generated callers alone destroy completed children/roots.
pub(super) const SUPPORT: &str = r"
declare token @llvm.coro.id(i32, ptr, ptr, ptr)
declare i1 @llvm.coro.alloc(token)
declare i64 @llvm.coro.size.i64()
declare ptr @llvm.coro.begin(token, ptr)
declare token @llvm.coro.save(ptr)
declare i8 @llvm.coro.suspend(token, i1)
declare ptr @llvm.coro.free(token, ptr)
declare i1 @llvm.coro.end(ptr, i1, token)
declare ptr @llvm.coro.noop()
declare void @llvm.coro.resume(ptr)
declare void @llvm.coro.destroy(ptr)
declare i1 @llvm.coro.done(ptr)
declare void @llvm.coro.await.suspend.handle(ptr, ptr, ptr)
declare i1 @llvm.coro.await.suspend.bool(ptr, ptr, ptr)
declare i32 @wf__continuation_record_done(ptr)
declare void @wf__continuation_prepare(ptr, ptr)
declare i32 @wf__continuation_arm(ptr, ptr)
declare void @wf__continuation_run(ptr)

define void @wf__continuation_resume(ptr %frame) {
entry:
  call void @llvm.coro.resume(ptr %frame)
  ret void
}
define i32 @wf__continuation_finished(ptr %frame) {
entry:
  %done = call i1 @llvm.coro.done(ptr %frame)
  %answer = zext i1 %done to i32
  ret i32 %answer
}
define private ptr @wf__continuation_transfer(ptr %target, ptr %self) {
entry:
  ret ptr %target
}
define private i1 @wf__continuation_register(ptr %waiter, ptr %self) {
entry:
  %armed = call i32 @wf__continuation_arm(ptr %waiter, ptr %self)
  %suspend = icmp ne i32 %armed, 0
  ret i1 %suspend
}
";

impl FunctionEmitter<'_, '_> {
    pub(super) fn emit_continuation(mut self) -> Result<String, BackendFailure> {
        let result_type = llvm_type(self.program, self.function.result())?;
        let parameters = self
            .function
            .parameters()
            .iter()
            .map(|(value, ty)| {
                Ok(format!(
                    "{} {}",
                    llvm_type(self.program, *ty)?,
                    value_name(*value)
                ))
            })
            .collect::<Result<Vec<_>, BackendFailure>>()?;
        let arguments = parameters.join(", ");
        let suffix = if arguments.is_empty() {
            String::new()
        } else {
            format!(", {arguments}")
        };
        let coro_symbol = symbol(self.function.name());
        let target = TargetLayout::host().map_err(BackendFailure::TargetLayout)?;
        let wrapper_frame = render_named_target_frame(
            self.program,
            self.qualification,
            target,
            &[(
                "%wf.coro.result",
                TargetFrameSlot::natural(TargetStorageType::source(self.function.result())),
            )],
        )?;
        // The cold host call must not turn the root into its own C stack
        // allocation. Nested coroutine calls retain proven frame elision.
        writeln!(self.output,
            "define internal {result_type} @{}({arguments}) {{\nentry:\n{wrapper_frame}  \
             %noop = call ptr @llvm.coro.noop()\n  \
             %handle = call ptr @{coro_symbol}(ptr %wf.coro.result, ptr %noop{suffix}) noinline\n  \
             call void @wf__continuation_run(ptr %handle)\n  \
             call void @llvm.coro.destroy(ptr %handle)\n  \
             %answer = load {result_type}, ptr %wf.coro.result\n  \
             ret {result_type} %answer\n}}\n\n\
             define internal ptr @{coro_symbol}(ptr %wf.coro.out, ptr %wf.coro.parent{suffix}) presplitcoroutine {{\n\
             wf.coro.entry:\n{}  \
             %wf.coro.id = call token @llvm.coro.id(i32 0, ptr null, ptr null, ptr null)\n  \
             %wf.coro.needs.allocate = call i1 @llvm.coro.alloc(token %wf.coro.id)\n  \
             br i1 %wf.coro.needs.allocate, label %wf.coro.allocate, label %wf.coro.begin\n\
             wf.coro.allocate:\n  \
             %wf.coro.size = call i64 @llvm.coro.size.i64()\n  \
             %wf.coro.memory = call ptr @malloc(i64 %wf.coro.size)\n  \
             %wf.coro.missing = icmp eq ptr %wf.coro.memory, null\n  \
             br i1 %wf.coro.missing, label %wf.coro.failure, label %wf.coro.allocated\n\
             wf.coro.failure:\n  call void @wf_resource_abort()\n  unreachable\n\
             wf.coro.allocated:\n  br label %wf.coro.begin\n\
             wf.coro.begin:\n  \
             %wf.coro.memory.selected = phi ptr [ null, %wf.coro.entry ], [ %wf.coro.memory, %wf.coro.allocated ]\n  \
             %wf.coro.handle = call ptr @llvm.coro.begin(token %wf.coro.id, ptr %wf.coro.memory.selected)\n  \
             %wf.coro.initial = call i8 @llvm.coro.suspend(token none, i1 false)\n  \
             switch i8 %wf.coro.initial, label %wf.coro.suspended [ i8 0, label %entry i8 1, label %wf.coro.destroy ]",
            source_symbol(self.function.name()), self.entry_prelude,
        ).map_err(|_| BackendFailure::TextEmission)?;
        for (index, block) in self.function.blocks().iter().enumerate() {
            let id = IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
            writeln!(self.output, "{}:", block_label(id))
                .map_err(|_| BackendFailure::TextEmission)?;
            self.emit_block_parameters(id, block)?;
            for (instruction_index, instruction) in block.instructions().iter().enumerate() {
                self.emit_instruction(id, instruction_index, instruction)?;
            }
            self.emit_terminator(id, block.terminator())?;
        }
        if !self.handed_out.is_empty() {
            return Err(BackendFailure::UnretiredCompletionOperation);
        }
        self.output.push_str(r"
wf.coro.final:
  %wf.coro.final.saved = call token @llvm.coro.save(ptr null)
  call void @llvm.coro.await.suspend.handle(ptr %wf.coro.parent, ptr %wf.coro.handle, ptr @wf__continuation_transfer)
  %wf.coro.final.state = call i8 @llvm.coro.suspend(token %wf.coro.final.saved, i1 true)
  switch i8 %wf.coro.final.state, label %wf.coro.suspended [ i8 1, label %wf.coro.destroy ]
wf.coro.destroy:
  %wf.coro.freed = call ptr @llvm.coro.free(token %wf.coro.id, ptr %wf.coro.handle)
  call void @free(ptr %wf.coro.freed)
  br label %wf.coro.suspended
wf.coro.suspended:
  %wf.coro.end = call i1 @llvm.coro.end(ptr null, i1 false, token none)
  ret ptr %wf.coro.handle
}

");
        Ok(self.output)
    }

    pub(super) fn emit_continuation_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        name: &str,
        arguments: &[String],
    ) -> Result<(), BackendFailure> {
        let slot = self.entry_slot(FunctionSlot::ContinuationResult(result))?;
        let args = if arguments.is_empty() {
            String::new()
        } else {
            format!(", {}", arguments.join(", "))
        };
        let prefix = format!("wf.coro.call.v{}", result.ordinal());
        writeln!(self.output,
            "  %{prefix}.child = call ptr @{}(ptr {slot}, ptr %wf.coro.handle{args}) coro_elide_safe\n  \
             %{prefix}.saved = call token @llvm.coro.save(ptr null)\n  \
             call void @llvm.coro.await.suspend.handle(ptr %{prefix}.child, ptr %wf.coro.handle, ptr @wf__continuation_transfer)\n  \
             %{prefix}.state = call i8 @llvm.coro.suspend(token %{prefix}.saved, i1 false)\n  \
             switch i8 %{prefix}.state, label %wf.coro.suspended [ i8 0, label %{prefix}.returned i8 1, label %wf.coro.destroy ]\n\
             {prefix}.returned:\n  call void @llvm.coro.destroy(ptr %{prefix}.child)\n  \
             {} = load {}, ptr {slot}", symbol(name), value_name(result), llvm_type(self.program, ty)?)
            .map_err(|_| BackendFailure::TextEmission)
    }

    pub(super) fn continuation_wait(
        &self,
        record: &str,
        value: IrValueId,
    ) -> Result<String, BackendFailure> {
        if !self.continuation {
            return Ok(String::new());
        }
        let waiter = self.entry_slot(FunctionSlot::ContinuationWaiter)?;
        let prefix = format!("wf.coro.wait.v{}", value.ordinal());
        Ok(format!(
            "  %{prefix}.isdone = call i32 @wf__continuation_record_done(ptr {record})\n  \
             %{prefix}.ready = icmp ne i32 %{prefix}.isdone, 0\n  \
             br i1 %{prefix}.ready, label %{prefix}.done, label %{prefix}.prepare\n\
             {prefix}.prepare:\n  call void @wf__continuation_prepare(ptr {waiter}, ptr {record})\n  \
             %{prefix}.saved = call token @llvm.coro.save(ptr null)\n  \
             %{prefix}.armed = call i1 @llvm.coro.await.suspend.bool(ptr {waiter}, ptr %wf.coro.handle, ptr @wf__continuation_register)\n  \
             br i1 %{prefix}.armed, label %{prefix}.suspend, label %{prefix}.done\n\
             {prefix}.suspend:\n  %{prefix}.state = call i8 @llvm.coro.suspend(token %{prefix}.saved, i1 false)\n  \
             switch i8 %{prefix}.state, label %wf.coro.suspended [ i8 0, label %{prefix}.done i8 1, label %wf.coro.destroy ]\n\
             {prefix}.done:\n"
        ))
    }
}
