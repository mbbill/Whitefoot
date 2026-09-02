//! Finite one-shot completion actualization for direct system operations.
//!
//! This is deliberately narrower than the compute hand-out path. Only a
//! compiler-owned direct file operation reaches the typed adapter; a
//! Whitefoot function, wrapper, callback, or source spelling never crosses
//! that boundary. Source-order progress is carried by ordinary result and
//! loan dependencies, so unrelated pending operations have no common join.

use std::fmt::Write;

use super::parallel::{HandedOut, par_done_label};
use super::system::{CompletionFileOperation, completion_file_operation, completion_mapper_symbol};
use super::*;

/// The finite completion core contract embedded in the compiler.
pub const COMPLETION_CONTRACT_HEADER: &str = include_str!("../completion/contract.h");
/// The typed file-adapter contract embedded in the compiler.
pub const COMPLETION_FILE_ADAPTER_HEADER: &str = include_str!("../completion/file_adapter.h");
/// The compiler-owned file-completion bridge contract embedded in the compiler.
pub const COMPLETION_BRIDGE_HEADER: &str = include_str!("../completion/bridge.h");
/// The stackless writer scheduler ABI embedded in the compiler.
pub const WRITER_SCHEDULER_HEADER: &str = include_str!("../completion/writer_scheduler.h");
/// The target-guarded Linux io_uring adapter contract embedded in the compiler.
pub const COMPLETION_LINUX_IO_URING_HEADER: &str = include_str!("../completion/linux_io_uring.h");
/// The target-private completion ABI shared by the Windows core and IOCP adapter.
pub const COMPLETION_WINDOWS_NATIVE_API_HEADER: &str =
    include_str!("../completion/native_completion_api.h");
/// The Windows completion core contract embedded in the compiler.
pub const COMPLETION_WINDOWS_HEADER: &str = include_str!("../completion/windows_completion.h");
/// The Windows IOCP adapter contract embedded in the compiler.
pub const COMPLETION_WINDOWS_IOCP_HEADER: &str = include_str!("../completion/windows_iocp.h");
/// The Windows bounded blocking-worker contract embedded in the compiler.
pub const COMPLETION_WINDOWS_BLOCKING_HEADER: &str =
    include_str!("../completion/windows_blocking.h");
/// The finite completion core implementation embedded in the compiler.
pub const COMPLETION_RUNTIME_SOURCE: &str = include_str!("../completion/runtime.c");
/// The typed file-adapter implementation embedded in the compiler.
pub const COMPLETION_FILE_ADAPTER_SOURCE: &str = include_str!("../completion/file_adapter.c");
/// The compiler-owned file-completion bridge embedded in the compiler.
pub const COMPLETION_BRIDGE_SOURCE: &str = include_str!("../completion/bridge.c");
/// The bounded ready-frame scheduler implementation. It creates no thread.
pub const WRITER_SCHEDULER_SOURCE: &str = include_str!("../completion/writer_scheduler.c");
/// The target-guarded Linux io_uring adapter embedded in the compiler.
pub const COMPLETION_LINUX_IO_URING_SOURCE: &str = include_str!("../completion/linux_io_uring.c");
/// The Windows completion core embedded in the compiler.
pub const COMPLETION_WINDOWS_SOURCE: &str = include_str!("../completion/windows_completion.c");
/// The Windows IOCP adapter embedded in the compiler.
pub const COMPLETION_WINDOWS_IOCP_SOURCE: &str = include_str!("../completion/windows_iocp.c");
/// The Windows bounded blocking-worker adapter embedded in the compiler.
pub const COMPLETION_WINDOWS_BLOCKING_SOURCE: &str =
    include_str!("../completion/windows_blocking.c");
/// The compiler-owned Windows completion bridge embedded in the compiler.
pub const COMPLETION_WINDOWS_BRIDGE_SOURCE: &str = include_str!("../completion/windows_bridge.c");
/// The Windows bounded ready-frame scheduler embedded in the compiler.
pub const WRITER_SCHEDULER_WINDOWS_SOURCE: &str =
    include_str!("../completion/writer_scheduler_windows.c");

/// The marker definition carried only by a module that actualizes a typed
/// target operation through completion.
const COMPLETION_MARKER: &str = "define weak i32 @wf__completion_file_read_submit(i32 %descriptor, ptr %buffer, i64 %count, ptr %token)";

/// Hard completion ABI for COFF modules.
///
/// Windows has no optional completion backend: the compiler driver supplies
/// the native core and IOCP bridge, and omitting either is a link error.  The
/// capacity wait is part of that same contract.  It lets a submitter which
/// owns no earlier token wait for another source owner to retire core
/// capacity without interpreting pressure as permission to run the operation
/// directly.
pub(crate) const COMPLETION_WINDOWS_RUNTIME_DECLARATIONS: &str = concat!(
    "declare i32 @wf__completion_file_read_submit(i32, ptr, i64, ptr)\n",
    "declare i32 @wf__completion_file_pread_submit(i32, ptr, i64, i64, ptr)\n",
    "declare i32 @wf__completion_file_write_submit(i32, ptr, i64, ptr)\n",
    "declare i32 @wf__completion_file_open_at_submit(i32, ptr, i32, i32, i32, i32, i32, ptr)\n",
    "declare i32 @wf__completion_file_status_submit(i32, ptr)\n",
    "declare i32 @wf__completion_file_close_submit(i32, ptr)\n",
    "declare i32 @wf__completion_directory_next_submit(i32, ptr, i64, ptr, ptr)\n",
    "declare void @wf__completion_file_join(ptr, ptr, ptr)\n",
    "declare void @wf__completion_file_open_join(ptr, ptr, ptr, ptr)\n",
    "declare void @wf__completion_wait_core_capacity()\n",
);

/// Hard Windows declaration for the staged completion-window query.
pub(crate) const COMPLETION_WINDOWS_WINDOW_DECLARATION: &str =
    "declare i64 @wf__completion_window(i64, i64, i64)\n";

/// Validates a dynamically chosen ring element before a caller makes LLVM's
/// `inbounds` promise about it.  Keeping the branch in this helper leaves the
/// caller's predecessor labels intact: completion joins and source block phis
/// can continue to name their real submission blocks.
pub(crate) const COMPLETION_SLOT_CHECKER: &str = "define private i64 @wf__completion_checked_slot(i64 %slot, i64 %slots) {\nentry:\n  %in.range = icmp ult i64 %slot, %slots\n  br i1 %in.range, label %valid, label %invalid\nvalid:\n  ret i64 %slot\ninvalid:\n  call void @abort()\n  unreachable\n}\n\n";

const COMPLETION_WINDOWS_MARKER: &str =
    "declare i32 @wf__completion_file_read_submit(i32, ptr, i64, ptr)";

/// Weak direct-specialization answer for a link without the completion unit.
/// Returning zero selects the already-qualified direct wrapper.  A standard
/// compiler link detects this marker and supplies the strong runtime.
pub(crate) const COMPLETION_RUNTIME_FALLBACK: &str = concat!(
    "define weak i32 @wf__completion_file_read_submit(i32 %descriptor, ptr %buffer, i64 %count, ptr %token) {\nentry:\n  ret i32 0\n}\n\n",
    "define weak i32 @wf__completion_file_pread_submit(i32 %descriptor, ptr %buffer, i64 %count, i64 %file_offset, ptr %token) {\nentry:\n  ret i32 0\n}\n\n",
    "define weak i32 @wf__completion_file_write_submit(i32 %descriptor, ptr %buffer, i64 %count, ptr %token) {\nentry:\n  ret i32 0\n}\n\n",
    "define weak i32 @wf__completion_file_open_at_submit(i32 %directory, ptr %path, i32 %flags, i32 %mode, i32 %has_mode, i32 %expected_kind, ptr %token) {\nentry:\n  ret i32 0\n}\n\n",
    "define weak i32 @wf__completion_file_status_submit(i32 %descriptor, ptr %token) {\nentry:\n  ret i32 0\n}\n\n",
    "define weak i32 @wf__completion_file_close_submit(i32 %descriptor, ptr %token) {\nentry:\n  ret i32 0\n}\n\n",
    "define weak i32 @wf__completion_directory_next_submit(i32 %descriptor, ptr %buffer, i64 %count, ptr %position, ptr %token) {\nentry:\n  ret i32 0\n}\n\n",
    "define weak void @wf__completion_file_join(ptr %token, ptr %value, ptr %error) {\nentry:\n  ret void\n}\n\n",
    "define weak void @wf__completion_file_open_join(ptr %token, ptr %value, ptr %error, ptr %outcome) {\nentry:\n  ret void\n}\n\n",
);

/// Weak window answer for a link without the completion unit.
///
/// One is always a legal window and reproduces the sequential program exactly,
/// so a module that asks for one and finds no runtime to answer stages no loop
/// and still publishes the same bytes.
pub(crate) const COMPLETION_WINDOW_FALLBACK: &str = "define weak i64 @wf__completion_window(i64 %span, i64 %slot_bytes, i64 %ceiling) {\nentry:\n  ret i64 1\n}\n\n";

/// True exactly when this emitted module contains a completion actualization.
pub fn module_requires_completion_runtime(module: &str) -> bool {
    module.contains(COMPLETION_MARKER)
        || module.contains(COMPLETION_WINDOWS_MARKER)
        || module.contains("@wf__completion_file_pread_submit_writer")
        || module.contains("@wf__completion_file_write_submit_writer")
        || module.contains("@wf__completion_file_pread_direct")
        || module.contains("@wf__completion_file_write_direct")
        || module.contains("@wf__completion_file_open_at_direct")
        || module.contains("@wf__completion_file_status_direct")
        || module.contains("@wf__completion_file_close_direct")
        || module.contains("@wf__completion_directory_next_direct")
}

/// True exactly when this emitted module can publish a stackless writer frame.
///
/// Direct completion calls still need the completion runtime, but they never
/// enqueue a continuation for a compute worker to resume.  Testing the actual
/// submit calls, rather than any completion symbol or the weak definitions a
/// stackless module carries, keeps the parallel runtime's hot steal loop free
/// of an empty writer-queue probe for an ordinary direct I/O module.
pub fn module_requires_writer_scheduler(module: &str) -> bool {
    module.contains("call i32 @wf__completion_file_pread_submit_writer(")
        || module.contains("call i32 @wf__completion_file_write_submit_writer(")
}

#[derive(Clone, Debug)]
pub(crate) struct CompletionHandedOut {
    result: IrValueId,
    result_type: IrType,
    operation: CompletionFileOperation,
    /// A carrying region defers the source result to one of its drains.  It
    /// must therefore retain every non-SSA fact the drain needs in the
    /// operation's own record.
    staged: bool,
    token: CompletionStorage,
    result_slot: CompletionStorage,
    raw_value: CompletionStorage,
    raw_error: CompletionStorage,
    submission: CompletionSubmission,
    mapping: CompletionMapping,
}

/// How a later source join learns whether the target still owns the request.
///
/// Optional non-Windows completion keeps the original SSA bit.  Windows uses
/// an addressable bit because core-pressure progress may consume the request
/// before its source join: that path stores the typed result in `result_slot`
/// and clears this bit, turning the eventual join into a plain load.
#[derive(Clone, Debug)]
enum CompletionSubmission {
    Ssa(String),
    WindowsState(CompletionStorage),
    PipelineState(CompletionStorage),
}

/// Where one kind of completion storage belonging to one call site lives.
///
/// The two shapes are the two schedules. Without a ring a site owns exactly one
/// record, its pointer is an entry-block definition, and it dominates every
/// block that can reach the operation — which is how this emitter has always
/// addressed completion storage. With a ring the site owns several, one per
/// operation the region may have in flight, and which of them an operation owns
/// is not known until the program runs: the pointer is materialized from the
/// slot index of whichever block starts or retires the operation, so it is
/// defined in that block and dominates its uses the way a block's own phi
/// does.
#[derive(Clone, Debug)]
struct CompletionStorage {
    /// The entry-block reservation: the record itself where the site owns one,
    /// the whole ring where it owns several.
    reservation: String,
    /// The element type, and with it the fact that the reservation is a ring
    /// to be indexed rather than the record itself. How many elements it holds
    /// is the pipeline's `slots` and is not repeated here.
    ring_element: Option<String>,
}

/// One fact the mapper needs after a target operation has completed.
///
/// An ordinary completion joins in the block that computed its arguments, so
/// its facts can stay in SSA.  A staged one can be retired through another
/// block and another iteration's ring element, so it writes each fact beside
/// its token before the adapter owns the request and reloads it only on the
/// completion path.
#[derive(Clone, Debug)]
enum CompletionFact {
    Ssa {
        llvm_type: String,
        value: String,
    },
    Stored {
        llvm_type: String,
        storage: CompletionStorage,
    },
}

impl CompletionHandedOut {
    /// The call site this operation belongs to.
    pub(super) const fn result(&self) -> IrValueId {
        self.result
    }
}

#[derive(Clone, Debug)]
enum CompletionMapping {
    Open {
        outcome: CompletionStorage,
    },
    Transfer {
        start: CompletionFact,
        extent: CompletionFact,
    },
    DirectoryNext {
        destination: CompletionFact,
        start: CompletionFact,
        extent: CompletionFact,
    },
}

impl FunctionEmitter<'_, '_> {
    fn rendered_system_arguments(
        &self,
        arguments: &[IrValueId],
    ) -> Result<Vec<String>, BackendFailure> {
        let mut rendered = Vec::new();
        for argument in arguments {
            let ty = self
                .value_type(*argument)
                .ok_or(BackendFailure::InvalidIr)?;
            if system::proof_only_resource(self.program, ty)? {
                continue;
            }
            rendered.push(format!(
                "{} {}",
                llvm_type(self.program, ty)?,
                self.value_name(*argument)
            ));
        }
        Ok(rendered)
    }

    /// Joins exactly the named prior operations, leaving every unrelated
    /// target operation in flight.
    pub(super) fn emit_completion_dependencies(
        &mut self,
        dependencies: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        for dependency in dependencies {
            let Some(position) = self.handed_out.iter().position(|pending| {
                matches!(pending, HandedOut::Completion(pending) if pending.result == *dependency)
            }) else {
                continue;
            };
            let HandedOut::Completion(pending) = self.handed_out.remove(position) else {
                return Err(BackendFailure::InvalidIr);
            };
            self.emit_completion_join(*pending)?;
            // A wait set retires an operation inside the carrying region, so
            // the region's exits must stop expecting it. A drain reaches this
            // with the record already taken, and the retain is nothing there.
            self.pipeline_outstanding
                .retain(|(_, carried)| carried.result() != *dependency);
        }
        Ok(())
    }

    /// Consumes every outstanding direct target operation before leaving a
    /// schedule or a control-flow block. Compute-lane hand-outs remain owned
    /// by their existing overlap join.
    pub(super) fn emit_all_completion_joins(&mut self) -> Result<(), BackendFailure> {
        // The drain retires the window on *this* path. What the carrying
        // region handed out stays recorded, because the region's other exits
        // must retire the same operations on theirs.
        let carried = std::mem::take(&mut self.pipeline_outstanding);
        let joined = self.emit_outstanding_completion_joins();
        self.pipeline_outstanding = carried;
        joined
    }

    fn emit_outstanding_completion_joins(&mut self) -> Result<(), BackendFailure> {
        let dependencies = self
            .handed_out
            .iter()
            .filter_map(|pending| match pending {
                HandedOut::Completion(pending) => Some(pending.result),
                HandedOut::Compute(_) => None,
            })
            .collect::<Vec<_>>();
        self.emit_completion_dependencies(&dependencies)
    }

    /// Whether an operation of this call site is still outstanding here.
    ///
    /// `handed_out` holds exactly the hand-outs emitted and not yet joined, so
    /// this reads, at emission, the question the storage reservation depends
    /// on: does a live operation already own this site's storage?
    pub(super) fn completion_operation_is_outstanding(&self, site: IrValueId) -> bool {
        self.handed_out.iter().any(
            |pending| matches!(pending, HandedOut::Completion(pending) if pending.result == site),
        )
    }

    /// How many operations one handed-out call site can have outstanding at
    /// once, and where the element this hand-out owns is addressed from.
    ///
    /// Every completion storage element — the token, the result slot, the raw
    /// value and error, an open's outcome, a directory cursor's position, an
    /// open's staged path — belongs to one *operation*, not to one written
    /// call.  The target writes the result and reads the staged path while the
    /// operation is outstanding, so two operations of one site that are
    /// outstanding together need one element each; sharing would let the newer
    /// hand-out overwrite storage the older one is still being read from.
    ///
    /// Which of the two the block being emitted asks for is decided by the
    /// pipeline, and it decides it for the reason the two schedules differ.
    /// A block outside a carrying region reaches its site once with nothing of
    /// that site's outstanding — `emit_terminator` joins everything still in
    /// flight before it writes any terminator — so one element is exact, and
    /// reserving one is what keeps every module this compiler emitted before
    /// the ring existed byte-identical. A carrying block is emitted once and
    /// reached once per iteration with the previous iteration's operation
    /// still owned by the target, so its site needs the whole ring.
    ///
    /// A carrying block whose descriptor gives it no slot is refused rather
    /// than handed element zero. That refusal is the one that matters: falling
    /// back to a single element there is exactly the silent sharing this whole
    /// reservation exists to prevent, and it would show up only as two
    /// iterations reading one buffer.
    ///
    /// The straight-line refusal below stays either way, and it is not the
    /// ring's business: it catches a *walk* that reaches one site twice with a
    /// hand-out live, which no descriptor makes legal, because the second
    /// hand-out would be emitted with the same slot index in hand as the
    /// first and would take the element the first is using.
    fn completion_entry_slot(
        &mut self,
        site: IrValueId,
        ty: &str,
    ) -> Result<CompletionStorage, BackendFailure> {
        if self.completion_operation_is_outstanding(site) {
            return Err(BackendFailure::SecondOutstandingCompletionOperation);
        }
        let slots = self.pipeline.map_or(1, crate::IrCompletionPipeline::slots);
        if slots > 1 && self.block_carries {
            if self.block_slot.is_none() {
                return Err(BackendFailure::MisaddressedCompletionSlot);
            }
            return Ok(CompletionStorage {
                reservation: self.entry_slot(&format!("[{slots} x {ty}]"))?,
                ring_element: Some(ty.to_owned()),
            });
        }
        Ok(CompletionStorage {
            reservation: self.indexed_entry_slot(ty, 1, 0)?,
            ring_element: None,
        })
    }

    /// The pointer to the element this operation owns, usable where it is
    /// emitted.
    ///
    /// A fixed element is an entry-block definition and needs nothing: naming
    /// it is free and emits no instruction, which is why a function with no
    /// ring is byte-identical to one emitted before rings existed. A ring
    /// element is computed here, in the block asking for it, from that block's
    /// own slot index — so a submission addresses the slot its iteration
    /// took and a retirement addresses the slot it is retiring, and neither
    /// has to be the other. The descriptor validation proves the slot name
    /// dominates this point; the compiler-owned helper proves its dynamic
    /// range before LLVM's `inbounds` promise is made without adding a branch
    /// to the caller's control-flow graph.
    fn completion_storage_pointer(
        &mut self,
        storage: &CompletionStorage,
    ) -> Result<String, BackendFailure> {
        let Some(element_type) = storage.ring_element.as_deref() else {
            return Ok(storage.reservation.clone());
        };
        let slots = self.pipeline.map_or(1, crate::IrCompletionPipeline::slots);
        let slot = self
            .block_slot
            .ok_or(BackendFailure::MisaddressedCompletionSlot)?;
        let checked = format!("%{}", self.next_temporary()?);
        let element = format!("%{}", self.next_temporary()?);
        let array = &storage.reservation;
        writeln!(
            self.output,
            "  {checked} = call i64 @wf__completion_checked_slot(i64 {}, i64 {slots})\n  \
             {element} = getelementptr inbounds [{slots} x {element_type}], ptr {array}, \
             i64 0, i64 {checked}",
            self.value_name(slot),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(element)
    }

    /// Reserves and initializes the state that tells a later join whether its
    /// operation reached the target.  The initialization is deliberately in
    /// the entry prelude, not on the submission path: a Windows pressure scan
    /// can reach any ring element before that element's first submission, and
    /// a branch may never enter its submit arm at all.
    fn completion_submission_state(
        &mut self,
        site: IrValueId,
    ) -> Result<CompletionStorage, BackendFailure> {
        let state = self.completion_entry_slot(site, "i1")?;
        let slots = self.pipeline.map_or(1, crate::IrCompletionPipeline::slots);
        if state.ring_element.is_some() {
            writeln!(
                self.entry_prelude,
                "  store [{slots} x i1] zeroinitializer, ptr {}",
                state.reservation
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        } else {
            writeln!(
                self.entry_prelude,
                "  store i1 false, ptr {}",
                state.reservation
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        Ok(state)
    }

    /// Captures a mapper argument in the operation record only when its join
    /// can run in a different block from its submission.
    fn completion_fact(
        &mut self,
        site: IrValueId,
        llvm_type: &str,
        value: String,
    ) -> Result<CompletionFact, BackendFailure> {
        if !self.block_carries {
            return Ok(CompletionFact::Ssa {
                llvm_type: llvm_type.to_owned(),
                value,
            });
        }
        let storage = self.completion_entry_slot(site, llvm_type)?;
        let pointer = self.completion_storage_pointer(&storage)?;
        writeln!(self.output, "  store {llvm_type} {value}, ptr {pointer}")
            .map_err(|_| BackendFailure::TextEmission)?;
        Ok(CompletionFact::Stored {
            llvm_type: llvm_type.to_owned(),
            storage,
        })
    }

    /// Reads one mapper argument on the completion path.  A stored fact is
    /// never loaded on a declined submission path, so an unvisited submit arm
    /// cannot manufacture an uninitialized source value.
    fn completion_fact_value(&mut self, fact: &CompletionFact) -> Result<String, BackendFailure> {
        match fact {
            CompletionFact::Ssa { value, .. } => Ok(value.clone()),
            CompletionFact::Stored { llvm_type, storage } => {
                let pointer = self.completion_storage_pointer(storage)?;
                let value = format!("%{}", self.next_temporary()?);
                writeln!(self.output, "  {value} = load {llvm_type}, ptr {pointer}")
                    .map_err(|_| BackendFailure::TextEmission)?;
                Ok(value)
            }
        }
    }

    fn completion_fact_type<'fact>(&self, fact: &'fact CompletionFact) -> &'fact str {
        match fact {
            CompletionFact::Ssa { llvm_type, .. } | CompletionFact::Stored { llvm_type, .. } => {
                llvm_type
            }
        }
    }

    /// Branches on the complete Windows submit verdict without collapsing
    /// core pressure into the direct route.
    ///
    /// A non-Windows target retains the original two-way optional-runtime
    /// contract byte for byte.  On Windows, `2` means that the request was not
    /// submitted because the finite core is full.  The source owner first
    /// consumes the oldest earlier request it still owns, materializes that
    /// request's typed result, and retries this exact submission.  If it owns
    /// none, the runtime's unified capacity wait makes progress elsewhere and
    /// the same submission is retried.  No pressure edge reaches `inline`.
    fn emit_completion_submit_verdict(
        &mut self,
        result: IrValueId,
        status: &str,
        accepted: &str,
        submit_label: &str,
        inline_label: &str,
        offered_label: &str,
    ) -> Result<(), BackendFailure> {
        if !self.qualification.target().is_windows() {
            writeln!(
                self.output,
                "  {accepted} = icmp eq i32 {status}, 1\n  \
                 br i1 {accepted}, label %{offered_label}, label %{inline_label}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            return Ok(());
        }

        let direct = format!("%{}", self.next_temporary()?);
        let waiting = format!("%{}", self.next_temporary()?);
        let verdict_label = completion_verdict_label(result);
        let wait_verdict_label = completion_wait_verdict_label(result);
        let invalid_label = completion_invalid_verdict_label(result);
        let capacity_label = completion_capacity_label(result);
        writeln!(
            self.output,
            "  {accepted} = icmp eq i32 {status}, 1\n  \
             br i1 {accepted}, label %{offered_label}, label %{verdict_label}\n\
             {verdict_label}:\n  \
             {direct} = icmp eq i32 {status}, 0\n  \
             br i1 {direct}, label %{inline_label}, label %{wait_verdict_label}\n\
             {wait_verdict_label}:\n  \
             {waiting} = icmp eq i32 {status}, 2\n  \
             br i1 {waiting}, label %{capacity_label}, label %{invalid_label}\n\
             {invalid_label}:\n  \
             call void @abort()\n  \
             unreachable\n\
             {capacity_label}:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;

        let owners = self
            .handed_out
            .iter()
            .filter_map(|pending| match pending {
                HandedOut::Completion(pending) => Some(pending.clone()),
                HandedOut::Compute(_) => None,
            })
            .collect::<Vec<_>>();
        for owner in owners {
            let CompletionSubmission::WindowsState(state) = &owner.submission else {
                return Err(BackendFailure::InvalidIr);
            };
            let state = self.completion_storage_pointer(state)?;
            let target_owned = format!("%{}", self.next_temporary()?);
            let consume_label = completion_capacity_consume_label(result, owner.result);
            let next_label = completion_capacity_next_label(result, owner.result);
            writeln!(
                self.output,
                "  {target_owned} = load i1, ptr {state}\n  \
                 br i1 {target_owned}, label %{consume_label}, label %{next_label}\n\
                 {consume_label}:"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            self.emit_windows_completion_materialization(&owner, &state)?;
            writeln!(
                self.output,
                "  br label %{submit_label}\n\
                 {next_label}:"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        writeln!(
            self.output,
            "  call void @wf__completion_wait_core_capacity()\n  \
             br label %{submit_label}"
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Consumes one target-owned Windows request and changes its later source
    /// join into a load from the call site's existing typed result slot.
    fn emit_windows_completion_materialization(
        &mut self,
        pending: &CompletionHandedOut,
        state: &str,
    ) -> Result<(), BackendFailure> {
        let token = self.completion_storage_pointer(&pending.token)?;
        let result_slot = self.completion_storage_pointer(&pending.result_slot)?;
        let raw_value = self.completion_storage_pointer(&pending.raw_value)?;
        let raw_error = self.completion_storage_pointer(&pending.raw_error)?;
        let completed_value = format!("%{}", self.next_temporary()?);
        let completed_error = format!("%{}", self.next_temporary()?);
        let completed = format!("%{}", self.next_temporary()?);
        let result_llvm = llvm_type(self.program, pending.result_type)?;
        let (join_call, extra_load, mapper_arguments) = match &pending.mapping {
            CompletionMapping::Open { outcome } => {
                let outcome = self.completion_storage_pointer(outcome)?;
                let completed_outcome = format!("%{}", self.next_temporary()?);
                (
                    format!(
                        "call void @wf__completion_file_open_join(ptr {token}, ptr {raw_value}, \
                         ptr {raw_error}, ptr {outcome})"
                    ),
                    format!("  {completed_outcome} = load i32, ptr {outcome}\n"),
                    format!(
                        "i64 {completed_value}, i32 {completed_error}, i32 {completed_outcome}"
                    ),
                )
            }
            CompletionMapping::Transfer { start, extent } => {
                let start = self.completion_fact_value(start)?;
                let extent = self.completion_fact_value(extent)?;
                (
                    format!(
                        "call void @wf__completion_file_join(ptr {token}, ptr {raw_value}, \
                         ptr {raw_error})"
                    ),
                    String::new(),
                    format!(
                        "i64 {completed_value}, i32 {completed_error}, i64 {start}, i64 {extent}"
                    ),
                )
            }
            CompletionMapping::DirectoryNext {
                destination,
                start,
                extent,
            } => {
                let destination_type = self.completion_fact_type(destination).to_owned();
                let destination = self.completion_fact_value(destination)?;
                let start = self.completion_fact_value(start)?;
                let extent = self.completion_fact_value(extent)?;
                (
                    format!(
                        "call void @wf__completion_file_join(ptr {token}, ptr {raw_value}, \
                         ptr {raw_error})"
                    ),
                    String::new(),
                    format!(
                        "i64 {completed_value}, i32 {completed_error}, {destination_type} \
                         {destination}, i64 {start}, i64 {extent}"
                    ),
                )
            }
        };
        writeln!(
            self.output,
            "  {join_call}\n  \
             {completed_value} = load i64, ptr {raw_value}\n  \
             {completed_error} = load i32, ptr {raw_error}\n  \
             {extra_load}\
             {completed} = call {result_llvm} @{}({mapper_arguments})\n  \
             store {result_llvm} {completed}, ptr {result_slot}\n  \
             store i1 false, ptr {state}",
            completion_mapper_symbol(pending.operation),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }

    /// Starts one direct file operation before the remaining independent
    /// members run.  All operation storage is allocated in the function entry
    /// block before the adapter can own the request.
    pub(super) fn emit_handed_out_system_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: crate::IrSystemOperation,
        arguments: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        let completion = completion_file_operation(operation).ok_or(BackendFailure::InvalidIr)?;
        match completion {
            CompletionFileOperation::OpenRead
            | CompletionFileOperation::OpenDirectory
            | CompletionFileOperation::OpenDirectorySource
            | CompletionFileOperation::OpenFile => {
                return self.emit_handed_out_open(result, ty, operation, arguments, completion);
            }
            CompletionFileOperation::DirectoryNext => {
                return self
                    .emit_handed_out_directory_next(result, ty, operation, arguments, completion);
            }
            CompletionFileOperation::Read | CompletionFileOperation::Write => {}
        }
        let (resource, buffer, file_offset, start, end) = match arguments {
            [resource, buffer, start, end] => (resource, buffer, None, start, end),
            [resource, buffer, file_offset, start, end]
                if completion == CompletionFileOperation::Read =>
            {
                (resource, buffer, Some(file_offset), start, end)
            }
            _ => return Err(BackendFailure::InvalidIr),
        };
        let u64_type = IrType::Integer {
            width: 64,
            signed: false,
        };
        let buffer_type = IrType::Buffer {
            element: crate::IrFlatElement::Integer {
                width: 8,
                signed: false,
            },
        };
        if self.value_type(*start) != Some(u64_type)
            || self.value_type(*end) != Some(u64_type)
            || self.value_type(*buffer) != Some(buffer_type)
            || file_offset.is_some_and(|offset| self.value_type(*offset) != Some(u64_type))
        {
            return Err(BackendFailure::InvalidIr);
        }
        let resource_type = self
            .value_type(*resource)
            .ok_or(BackendFailure::InvalidIr)?;
        if llvm_type(self.program, resource_type)? != "i32" {
            return Err(BackendFailure::InvalidIr);
        }

        let staged = self.block_carries;
        let token = self.completion_entry_slot(result, "[2 x i64]")?;
        let result_slot = self.completion_entry_slot(result, &llvm_type(self.program, ty)?)?;
        let raw_value = self.completion_entry_slot(result, "i64")?;
        let raw_error = self.completion_entry_slot(result, "i32")?;
        let submission_state = (self.qualification.target().is_windows() || self.block_carries)
            .then(|| self.completion_submission_state(result))
            .transpose()?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let result_pointer = self.completion_storage_pointer(&result_slot)?;
        let submission_state_pointer = submission_state
            .as_ref()
            .map(|state| self.completion_storage_pointer(state))
            .transpose()?;
        let extent = format!("%{}", self.next_temporary()?);
        let vacant = format!("%{}", self.next_temporary()?);
        let offset_too_large = file_offset
            .map(|_| self.next_temporary().map(|name| format!("%{name}")))
            .transpose()?;
        let ineligible = file_offset
            .map(|_| self.next_temporary().map(|name| format!("%{name}")))
            .transpose()?;
        let base = format!("%{}", self.next_temporary()?);
        let target = format!("%{}", self.next_temporary()?);
        let status = format!("%{}", self.next_temporary()?);
        let accepted = format!("%{}", self.next_temporary()?);
        let inline_result = format!("%{}", self.next_temporary()?);
        let submitted = format!("%{}", self.next_temporary()?);
        let submit_label = completion_submit_label(result);
        let inline_label = completion_inline_label(result);
        let offered_label = completion_offered_label(result);
        let submit_symbol = match (completion, file_offset) {
            (CompletionFileOperation::Read, Some(_)) => "wf__completion_file_pread_submit",
            (CompletionFileOperation::Read, None) => "wf__completion_file_read_submit",
            (CompletionFileOperation::Write, None) => "wf__completion_file_write_submit",
            (CompletionFileOperation::Write, Some(_)) => return Err(BackendFailure::InvalidIr),
            _ => return Err(BackendFailure::InvalidIr),
        };
        let implementation = self.qualification.operation(operation)?;
        let rendered_type = llvm_type(self.program, ty)?;
        let rendered_buffer = llvm_type(self.program, buffer_type)?;
        let rendered_arguments = arguments
            .iter()
            .map(|argument| {
                let argument_type = self
                    .value_type(*argument)
                    .ok_or(BackendFailure::InvalidIr)?;
                Ok(format!(
                    "{} {}",
                    llvm_type(self.program, argument_type)?,
                    self.value_name(*argument)
                ))
            })
            .collect::<Result<Vec<_>, BackendFailure>>()?;
        let (eligibility, ineligible) = match (
            file_offset,
            offset_too_large.as_deref(),
            ineligible.as_deref(),
        ) {
            (Some(offset), Some(offset_too_large), Some(ineligible)) => (
                format!(
                    "  {offset_too_large} = icmp ugt i64 {}, 9223372036854775807\n  \
                     {ineligible} = or i1 {vacant}, {offset_too_large}\n",
                    self.value_name(*offset)
                ),
                ineligible.to_owned(),
            ),
            (None, None, None) => (String::new(), vacant.clone()),
            _ => return Err(BackendFailure::InvalidIr),
        };
        let submit_arguments = if let Some(offset) = file_offset {
            format!(
                "i32 {}, ptr {target}, i64 {extent}, i64 {}, ptr {token_pointer}",
                self.value_name(*resource),
                self.value_name(*offset)
            )
        } else {
            format!(
                "i32 {}, ptr {target}, i64 {extent}, ptr {token_pointer}",
                self.value_name(*resource)
            )
        };

        writeln!(
            self.output,
            "  {extent} = sub i64 {}, {}\n  \
             {vacant} = icmp eq i64 {extent}, 0\n  \
             {eligibility}  \
             br i1 {ineligible}, label %{inline_label}, label %{submit_label}\n\
             {submit_label}:\n  \
             {base} = extractvalue {rendered_buffer} {}, 0\n  \
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}",
            self.value_name(*end),
            self.value_name(*start),
            self.value_name(*buffer),
            self.value_name(*start),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let mapping = CompletionMapping::Transfer {
            start: self.completion_fact(result, "i64", self.value_name(*start))?,
            extent: self.completion_fact(result, "i64", extent.clone())?,
        };
        writeln!(
            self.output,
            "  {status} = call i32 @{submit_symbol}({submit_arguments})"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.emit_completion_submit_verdict(
            result,
            &status,
            &accepted,
            &submit_label,
            &inline_label,
            &offered_label,
        )?;
        writeln!(
            self.output,
            "{inline_label}:\n  \
             {inline_result} = call {rendered_type} @{}({})\n  \
             store {rendered_type} {inline_result}, ptr {result_pointer}\n  \
             br label %{offered_label}\n\
             {offered_label}:\n  \
             {submitted} = phi i1 [ true, %{submit_label} ], [ false, %{inline_label} ]",
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if let Some(state) = &submission_state_pointer {
            writeln!(self.output, "  store i1 {submitted}, ptr {state}")
                .map_err(|_| BackendFailure::TextEmission)?;
        }
        let submission = match submission_state {
            None => CompletionSubmission::Ssa(submitted),
            Some(state) if self.qualification.target().is_windows() => {
                CompletionSubmission::WindowsState(state)
            }
            Some(state) => CompletionSubmission::PipelineState(state),
        };

        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(Box::new(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                staged,
                token,
                result_slot,
                raw_value,
                raw_error,
                submission,
                mapping,
            })));
        Ok(())
    }

    fn emit_handed_out_open(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: crate::IrSystemOperation,
        arguments: &[IrValueId],
        completion: CompletionFileOperation,
    ) -> Result<(), BackendFailure> {
        let request_label = completion_submit_label(result);
        let inline_label = completion_inline_label(result);
        let offered_label = completion_offered_label(result);
        let (directory, path, flags) = match completion {
            CompletionFileOperation::OpenRead => {
                let [.., directory, path] = arguments else {
                    return Err(BackendFailure::InvalidIr);
                };
                let path_ty = self.value_type(*path).ok_or(BackendFailure::InvalidIr)?;
                let text = format!("%{}", self.next_temporary()?);
                writeln!(
                    self.output,
                    "  {text} = extractvalue {} {}, 0\n  br label %{request_label}\n{request_label}:",
                    llvm_type(self.program, path_ty)?,
                    self.value_name(*path)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                (
                    *directory,
                    text,
                    self.qualification.target().file_open_flags(),
                )
            }
            CompletionFileOperation::OpenDirectorySource => {
                let [.., directory] = arguments else {
                    return Err(BackendFailure::InvalidIr);
                };
                writeln!(self.output, "  br label %{request_label}\n{request_label}:")
                    .map_err(|_| BackendFailure::TextEmission)?;
                (
                    *directory,
                    system::WORKING_DIRECTORY.to_owned(),
                    self.qualification.target().directory_open_flags(),
                )
            }
            CompletionFileOperation::OpenDirectory | CompletionFileOperation::OpenFile => self
                .emit_completion_component_path(
                    result,
                    arguments,
                    completion,
                    &request_label,
                    &inline_label,
                )?,
            CompletionFileOperation::Read
            | CompletionFileOperation::Write
            | CompletionFileOperation::DirectoryNext => {
                return Err(BackendFailure::InvalidIr);
            }
        };
        let directory_ty = self
            .value_type(directory)
            .ok_or(BackendFailure::InvalidIr)?;
        if llvm_type(self.program, directory_ty)? != "i32" {
            return Err(BackendFailure::InvalidIr);
        }
        let staged = self.block_carries;
        let token = self.completion_entry_slot(result, "[2 x i64]")?;
        let result_slot = self.completion_entry_slot(result, &llvm_type(self.program, ty)?)?;
        let raw_value = self.completion_entry_slot(result, "i64")?;
        let raw_error = self.completion_entry_slot(result, "i32")?;
        let open_outcome = self.completion_entry_slot(result, "i32")?;
        let submission_state = (self.qualification.target().is_windows() || self.block_carries)
            .then(|| self.completion_submission_state(result))
            .transpose()?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let result_pointer = self.completion_storage_pointer(&result_slot)?;
        let submission_state_pointer = submission_state
            .as_ref()
            .map(|state| self.completion_storage_pointer(state))
            .transpose()?;
        let status = format!("%{}", self.next_temporary()?);
        let accepted = format!("%{}", self.next_temporary()?);
        let inline_result = format!("%{}", self.next_temporary()?);
        let submitted = format!("%{}", self.next_temporary()?);
        let implementation = self.qualification.operation(operation)?;
        let rendered_type = llvm_type(self.program, ty)?;
        let rendered_arguments = self.rendered_system_arguments(arguments)?;
        let (expected_kind, descriptor_class) = match completion {
            CompletionFileOperation::OpenRead | CompletionFileOperation::OpenFile => (
                system::OPEN_EXPECT_REGULAR,
                system::WINDOWS_DESCRIPTOR_CLASS_READ_FILE,
            ),
            CompletionFileOperation::OpenDirectory => (
                system::OPEN_EXPECT_DIRECTORY,
                system::WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_ROOT,
            ),
            CompletionFileOperation::OpenDirectorySource => (
                system::OPEN_EXPECT_DIRECTORY,
                system::WINDOWS_DESCRIPTOR_CLASS_DIRECTORY_SOURCE,
            ),
            _ => return Err(BackendFailure::InvalidIr),
        };
        let descriptor_class_argument = if self.qualification.target().is_windows() {
            format!(", i32 {descriptor_class}")
        } else {
            String::new()
        };
        writeln!(
            self.output,
            "  {status} = call i32 @wf__completion_file_open_at_submit(i32 {}, ptr {path}, \
             i32 {flags}, i32 0, i32 0, i32 {expected_kind}{descriptor_class_argument}, \
             ptr {token_pointer})",
            self.value_name(directory),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.emit_completion_submit_verdict(
            result,
            &status,
            &accepted,
            &request_label,
            &inline_label,
            &offered_label,
        )?;
        writeln!(
            self.output,
            "{inline_label}:\n  \
             {inline_result} = call {rendered_type} @{}({})\n  \
             store {rendered_type} {inline_result}, ptr {result_pointer}\n  \
             br label %{offered_label}\n\
             {offered_label}:\n  \
             {submitted} = phi i1 [ true, %{request_label} ], [ false, %{inline_label} ]",
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if let Some(state) = &submission_state_pointer {
            writeln!(self.output, "  store i1 {submitted}, ptr {state}")
                .map_err(|_| BackendFailure::TextEmission)?;
        }
        let submission = match submission_state {
            None => CompletionSubmission::Ssa(submitted),
            Some(state) if self.qualification.target().is_windows() => {
                CompletionSubmission::WindowsState(state)
            }
            Some(state) => CompletionSubmission::PipelineState(state),
        };
        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(Box::new(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                staged,
                token,
                result_slot,
                raw_value,
                raw_error,
                submission,
                mapping: CompletionMapping::Open {
                    outcome: open_outcome,
                },
            })));
        Ok(())
    }

    fn emit_completion_component_path(
        &mut self,
        result: IrValueId,
        arguments: &[IrValueId],
        completion: CompletionFileOperation,
        request_label: &str,
        inline_label: &str,
    ) -> Result<(IrValueId, String, i32), BackendFailure> {
        let [.., directory, name, start, end] = arguments else {
            return Err(BackendFailure::InvalidIr);
        };
        let limit = self.qualification.target().component_limit();
        let terminator_bytes = if self.qualification.target().is_windows() {
            2
        } else {
            1
        };
        let slot = limit
            .checked_add(terminator_bytes)
            .ok_or(BackendFailure::CounterOverflow)?;
        let staged = self.completion_entry_slot(result, &format!("[{slot} x i8]"))?;
        let component = self.completion_storage_pointer(&staged)?;
        let extent = format!("%{}", self.next_temporary()?);
        let oversize = format!("%{}", self.next_temporary()?);
        let vacant = format!("%{}", self.next_temporary()?);
        let unusable = format!("%{}", self.next_temporary()?);
        let base = format!("%{}", self.next_temporary()?);
        let text = format!("%{}", self.next_temporary()?);
        let index = format!("%{}", self.next_temporary()?);
        let at = format!("%{}", self.next_temporary()?);
        let byte = format!("%{}", self.next_temporary()?);
        let scan_entry = format!("completion.component.entry.v{}", result.ordinal());
        let scan = format!("completion.component.scan.v{}", result.ordinal());
        let scan_step = format!("completion.component.step.v{}", result.ordinal());
        let ready = format!("completion.component.ready.v{}", result.ordinal());
        let buffer_ty = self.value_type(*name).ok_or(BackendFailure::InvalidIr)?;
        if self.qualification.target().is_windows() {
            let width_remainder = format!("%{}", self.next_temporary()?);
            let misaligned = format!("%{}", self.next_temporary()?);
            let size_unusable = format!("%{}", self.next_temporary()?);
            let unit = format!("%{}", self.next_temporary()?);
            let terminating = format!("%{}", self.next_temporary()?);
            let slash = format!("%{}", self.next_temporary()?);
            let backslash = format!("%{}", self.next_temporary()?);
            let separating = format!("%{}", self.next_temporary()?);
            let refused = format!("%{}", self.next_temporary()?);
            let next = format!("%{}", self.next_temporary()?);
            let scanned = format!("%{}", self.next_temporary()?);
            let terminator = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {extent} = sub i64 {}, {}\n  \
                 {oversize} = icmp ugt i64 {extent}, {limit}\n  \
                 {vacant} = icmp eq i64 {extent}, 0\n  \
                 {width_remainder} = and i64 {extent}, 1\n  \
                 {misaligned} = icmp ne i64 {width_remainder}, 0\n  \
                 {size_unusable} = or i1 {oversize}, {vacant}\n  \
                 {unusable} = or i1 {size_unusable}, {misaligned}\n  \
                 br i1 {unusable}, label %{inline_label}, label %{scan_entry}\n\
                 {scan_entry}:\n  \
                 {base} = extractvalue {} {}, 0\n  \
                 {text} = getelementptr inbounds i8, ptr {base}, i64 {}\n  \
                 br label %{scan}\n\
                 {scan}:\n  \
                 {index} = phi i64 [ 0, %{scan_entry} ], [ {next}, %{scan_step} ]\n  \
                 {at} = getelementptr inbounds i8, ptr {text}, i64 {index}\n  \
                 {unit} = load i16, ptr {at}, align 1\n  \
                 {terminating} = icmp eq i16 {unit}, 0\n  \
                 {slash} = icmp eq i16 {unit}, 47\n  \
                 {backslash} = icmp eq i16 {unit}, 92\n  \
                 {separating} = or i1 {slash}, {backslash}\n  \
                 {refused} = or i1 {terminating}, {separating}\n  \
                 br i1 {refused}, label %{inline_label}, label %{scan_step}\n\
                 {scan_step}:\n  \
                 {next} = add i64 {index}, 2\n  \
                 {scanned} = icmp uge i64 {next}, {extent}\n  \
                 br i1 {scanned}, label %{ready}, label %{scan}\n\
                 {ready}:\n  \
                 call void @llvm.memcpy.p0.p0.i64(ptr {component}, ptr {text}, i64 {extent}, \
                 i1 false)\n  \
                 {terminator} = getelementptr inbounds i8, ptr {component}, i64 {extent}\n  \
                 store i16 0, ptr {terminator}, align 1\n  \
                 br label %{request_label}\n\
                 {request_label}:",
                self.value_name(*end),
                self.value_name(*start),
                llvm_type(self.program, buffer_ty)?,
                self.value_name(*name),
                self.value_name(*start),
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        } else {
            let terminating = format!("%{}", self.next_temporary()?);
            let separating = format!("%{}", self.next_temporary()?);
            let refused = format!("%{}", self.next_temporary()?);
            let next = format!("%{}", self.next_temporary()?);
            let scanned = format!("%{}", self.next_temporary()?);
            let terminator = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {extent} = sub i64 {}, {}\n  \
                 {oversize} = icmp ugt i64 {extent}, {limit}\n  \
                 {vacant} = icmp eq i64 {extent}, 0\n  \
                 {unusable} = or i1 {oversize}, {vacant}\n  \
                 br i1 {unusable}, label %{inline_label}, label %{scan_entry}\n\
                 {scan_entry}:\n  \
                 {base} = extractvalue {} {}, 0\n  \
                 {text} = getelementptr inbounds i8, ptr {base}, i64 {}\n  \
                 br label %{scan}\n\
                 {scan}:\n  \
                 {index} = phi i64 [ 0, %{scan_entry} ], [ {next}, %{scan_step} ]\n  \
                 {at} = getelementptr inbounds i8, ptr {text}, i64 {index}\n  \
                 {byte} = load i8, ptr {at}, align 1\n  \
                 {terminating} = icmp eq i8 {byte}, 0\n  \
                 {separating} = icmp eq i8 {byte}, {}\n  \
                 {refused} = or i1 {terminating}, {separating}\n  \
                 br i1 {refused}, label %{inline_label}, label %{scan_step}\n\
                 {scan_step}:\n  \
                 {next} = add i64 {index}, 1\n  \
                 {scanned} = icmp uge i64 {next}, {extent}\n  \
                 br i1 {scanned}, label %{ready}, label %{scan}\n\
                 {ready}:\n  \
                 call void @llvm.memcpy.p0.p0.i64(ptr {component}, ptr {text}, i64 {extent}, \
                 i1 false)\n  \
                 {terminator} = getelementptr inbounds i8, ptr {component}, i64 {extent}\n  \
                 store i8 0, ptr {terminator}, align 1\n  \
                 br label %{request_label}\n\
                 {request_label}:",
                self.value_name(*end),
                self.value_name(*start),
                llvm_type(self.program, buffer_ty)?,
                self.value_name(*name),
                self.value_name(*start),
                self.qualification.target().root_prefix(),
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        let flags = match completion {
            CompletionFileOperation::OpenDirectory => {
                self.qualification.target().component_directory_open_flags()
            }
            CompletionFileOperation::OpenFile => {
                self.qualification.target().component_file_open_flags()
            }
            _ => return Err(BackendFailure::InvalidIr),
        };
        Ok((*directory, component, flags))
    }

    fn emit_handed_out_directory_next(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: crate::IrSystemOperation,
        arguments: &[IrValueId],
        completion: CompletionFileOperation,
    ) -> Result<(), BackendFailure> {
        let [source, destination, start, end] = arguments else {
            return Err(BackendFailure::InvalidIr);
        };
        if completion != CompletionFileOperation::DirectoryNext {
            return Err(BackendFailure::InvalidIr);
        }
        let destination_ty = self
            .value_type(*destination)
            .ok_or(BackendFailure::InvalidIr)?;
        let destination_llvm = llvm_type(self.program, destination_ty)?;
        let staged = self.block_carries;
        let token = self.completion_entry_slot(result, "[2 x i64]")?;
        let result_slot = self.completion_entry_slot(result, &llvm_type(self.program, ty)?)?;
        let raw_value = self.completion_entry_slot(result, "i64")?;
        let raw_error = self.completion_entry_slot(result, "i32")?;
        let cursor = self.completion_entry_slot(result, "i64")?;
        let submission_state = (self.qualification.target().is_windows() || self.block_carries)
            .then(|| self.completion_submission_state(result))
            .transpose()?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let result_pointer = self.completion_storage_pointer(&result_slot)?;
        let position = self.completion_storage_pointer(&cursor)?;
        let submission_state_pointer = submission_state
            .as_ref()
            .map(|state| self.completion_storage_pointer(state))
            .transpose()?;
        let extent = format!("%{}", self.next_temporary()?);
        let vacant = format!("%{}", self.next_temporary()?);
        let base = format!("%{}", self.next_temporary()?);
        let target = format!("%{}", self.next_temporary()?);
        let status = format!("%{}", self.next_temporary()?);
        let accepted = format!("%{}", self.next_temporary()?);
        let inline_result = format!("%{}", self.next_temporary()?);
        let submitted = format!("%{}", self.next_temporary()?);
        let submit_label = completion_submit_label(result);
        let inline_label = completion_inline_label(result);
        let offered_label = completion_offered_label(result);
        let implementation = self.qualification.operation(operation)?;
        let rendered_type = llvm_type(self.program, ty)?;
        let rendered_arguments = self.rendered_system_arguments(arguments)?;
        writeln!(
            self.output,
            "  {extent} = sub i64 {}, {}\n  \
             {vacant} = icmp eq i64 {extent}, 0\n  \
             br i1 {vacant}, label %{inline_label}, label %{submit_label}\n\
             {submit_label}:\n  \
             store i64 0, ptr {position}, align 8\n  \
             {base} = extractvalue {destination_llvm} {}, 0\n  \
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}",
            self.value_name(*end),
            self.value_name(*start),
            self.value_name(*destination),
            self.value_name(*start),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let mapping = CompletionMapping::DirectoryNext {
            destination: self.completion_fact(
                result,
                &destination_llvm,
                self.value_name(*destination),
            )?,
            start: self.completion_fact(result, "i64", self.value_name(*start))?,
            extent: self.completion_fact(result, "i64", extent.clone())?,
        };
        writeln!(
            self.output,
            "  {status} = call i32 @wf__completion_directory_next_submit(i32 {}, ptr {target}, \
             i64 {extent}, ptr {position}, ptr {token_pointer})",
            self.value_name(*source),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.emit_completion_submit_verdict(
            result,
            &status,
            &accepted,
            &submit_label,
            &inline_label,
            &offered_label,
        )?;
        writeln!(
            self.output,
            "{inline_label}:\n  \
             {inline_result} = call {rendered_type} @{}({})\n  \
             store {rendered_type} {inline_result}, ptr {result_pointer}\n  \
             br label %{offered_label}\n\
             {offered_label}:\n  \
             {submitted} = phi i1 [ true, %{submit_label} ], [ false, %{inline_label} ]",
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if let Some(state) = &submission_state_pointer {
            writeln!(self.output, "  store i1 {submitted}, ptr {state}")
                .map_err(|_| BackendFailure::TextEmission)?;
        }
        let submission = match submission_state {
            None => CompletionSubmission::Ssa(submitted),
            Some(state) if self.qualification.target().is_windows() => {
                CompletionSubmission::WindowsState(state)
            }
            Some(state) => CompletionSubmission::PipelineState(state),
        };
        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(Box::new(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                staged,
                token,
                result_slot,
                raw_value,
                raw_error,
                submission,
                mapping,
            })));
        Ok(())
    }

    pub(super) fn emit_completion_join(
        &mut self,
        pending: CompletionHandedOut,
    ) -> Result<(), BackendFailure> {
        let CompletionHandedOut {
            result,
            result_type,
            operation,
            staged,
            token,
            result_slot,
            raw_value,
            raw_error,
            submission,
            mapping,
        } = pending;
        // The element pointers are materialized here rather than carried from
        // the submission, because a retirement need not be in the block that
        // submitted and, under a ring, need not mean the same slot: what it
        // retires is the operation the slot index of *this* block names.
        let token = self.completion_storage_pointer(&token)?;
        let result_slot = self.completion_storage_pointer(&result_slot)?;
        let raw_value = self.completion_storage_pointer(&raw_value)?;
        let raw_error = self.completion_storage_pointer(&raw_error)?;
        let submitted = match submission {
            CompletionSubmission::Ssa(submitted) => submitted,
            CompletionSubmission::WindowsState(state)
            | CompletionSubmission::PipelineState(state) => {
                let state = self.completion_storage_pointer(&state)?;
                let submitted = format!("%{}", self.next_temporary()?);
                writeln!(self.output, "  {submitted} = load i1, ptr {state}")
                    .map_err(|_| BackendFailure::TextEmission)?;
                submitted
            }
        };
        let direct = format!("%{}", self.next_temporary()?);
        let completed_value = format!("%{}", self.next_temporary()?);
        let completed_error = format!("%{}", self.next_temporary()?);
        let completed = format!("%{}", self.next_temporary()?);
        let inline_label = completion_join_inline_label(result);
        let wait_label = completion_wait_label(result);
        let done_label = par_done_label(result);
        let result_llvm = llvm_type(self.program, result_type)?;
        writeln!(
            self.output,
            "  br i1 {submitted}, label %{wait_label}, label %{inline_label}\n\
             {inline_label}:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if staged {
            writeln!(self.output, "  br label %{done_label}")
                .map_err(|_| BackendFailure::TextEmission)?;
        } else {
            writeln!(
                self.output,
                "  {direct} = load {result_llvm}, ptr {result_slot}\n  \
                 br label %{done_label}"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        writeln!(self.output, "{wait_label}:").map_err(|_| BackendFailure::TextEmission)?;
        let (join_call, extra_load, mapper_arguments) = match &mapping {
            CompletionMapping::Open { outcome } => {
                let outcome = self.completion_storage_pointer(outcome)?;
                let completed_outcome = format!("%{}", self.next_temporary()?);
                (
                    format!(
                        "call void @wf__completion_file_open_join(ptr {token}, ptr {raw_value}, \
                         ptr {raw_error}, ptr {outcome})"
                    ),
                    format!("  {completed_outcome} = load i32, ptr {outcome}\n"),
                    format!(
                        "i64 {completed_value}, i32 {completed_error}, i32 {completed_outcome}"
                    ),
                )
            }
            CompletionMapping::Transfer { start, extent } => {
                let start = self.completion_fact_value(start)?;
                let extent = self.completion_fact_value(extent)?;
                (
                    format!(
                        "call void @wf__completion_file_join(ptr {token}, ptr {raw_value}, \
                         ptr {raw_error})"
                    ),
                    String::new(),
                    format!(
                        "i64 {completed_value}, i32 {completed_error}, i64 {start}, i64 {extent}"
                    ),
                )
            }
            CompletionMapping::DirectoryNext {
                destination,
                start,
                extent,
            } => {
                let destination_type = self.completion_fact_type(destination).to_owned();
                let destination = self.completion_fact_value(destination)?;
                let start = self.completion_fact_value(start)?;
                let extent = self.completion_fact_value(extent)?;
                (
                    format!(
                        "call void @wf__completion_file_join(ptr {token}, ptr {raw_value}, \
                         ptr {raw_error})"
                    ),
                    String::new(),
                    format!(
                        "i64 {completed_value}, i32 {completed_error}, {destination_type} \
                         {destination}, i64 {start}, i64 {extent}"
                    ),
                )
            }
        };
        writeln!(
            self.output,
            "  {join_call}\n  \
             {completed_value} = load i64, ptr {raw_value}\n  \
             {completed_error} = load i32, ptr {raw_error}\n  \
             {extra_load}\
             {completed} = call {result_llvm} @{}({mapper_arguments})",
            completion_mapper_symbol(operation),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if staged {
            writeln!(
                self.output,
                "  store {result_llvm} {completed}, ptr {result_slot}\n  \
                 br label %{done_label}\n\
                 {done_label}:"
            )
            .map_err(|_| BackendFailure::TextEmission)
        } else {
            writeln!(
                self.output,
                "  br label %{done_label}\n\
                 {done_label}:\n  \
                 {} = phi {result_llvm} [ {direct}, %{inline_label} ], [ {completed}, %{wait_label} ]",
                value_name(result),
            )
            .map_err(|_| BackendFailure::TextEmission)
        }
    }
}

fn completion_submit_label(value: IrValueId) -> String {
    format!("completion.submit.v{}", value.ordinal())
}

fn completion_inline_label(value: IrValueId) -> String {
    format!("completion.inline.v{}", value.ordinal())
}

fn completion_verdict_label(value: IrValueId) -> String {
    format!("completion.verdict.v{}", value.ordinal())
}

fn completion_wait_verdict_label(value: IrValueId) -> String {
    format!("completion.verdict.wait.v{}", value.ordinal())
}

fn completion_invalid_verdict_label(value: IrValueId) -> String {
    format!("completion.verdict.invalid.v{}", value.ordinal())
}

fn completion_capacity_label(value: IrValueId) -> String {
    format!("completion.capacity.v{}", value.ordinal())
}

fn completion_capacity_consume_label(current: IrValueId, owner: IrValueId) -> String {
    format!(
        "completion.capacity.consume.v{}.v{}",
        current.ordinal(),
        owner.ordinal()
    )
}

fn completion_capacity_next_label(current: IrValueId, owner: IrValueId) -> String {
    format!(
        "completion.capacity.next.v{}.v{}",
        current.ordinal(),
        owner.ordinal()
    )
}

pub(super) fn completion_offered_label(value: IrValueId) -> String {
    format!("completion.offered.v{}", value.ordinal())
}

fn completion_join_inline_label(value: IrValueId) -> String {
    format!("completion.join.inline.v{}", value.ordinal())
}

fn completion_wait_label(value: IrValueId) -> String {
    format!("completion.wait.v{}", value.ordinal())
}
