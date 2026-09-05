//! Finite completion actualization for direct system operations.
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
/// The bounded ready-frame writer scheduler ABI embedded in the compiler.
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

/// The scheduler core's contract embedded in the compiler.
///
/// The completion record begins with a `wf_sched_record` and every publication
/// goes through `wf_sched_complete`, so a link that carries the completion
/// runtime carries the core beside it
/// (`research/investigations/io-model/PARK-ON-MISS.md` §5, §7).
pub const SCHED_CORE_HEADER: &str = include_str!("../sched/core.h");
/// The scheduler core embedded in the compiler.
pub const SCHED_CORE_SOURCE: &str = include_str!("../sched/core.c");
/// The seven primitives the core reaches shared state through.
pub const SCHED_PRIM_HEADER: &str = include_str!("../sched/prim.h");
/// The host's implementation of those primitives.
pub const SCHED_PRIM_HOST_SOURCE: &str = include_str!("../sched/prim_host.c");
/// The one stack switch, shared by the host primitives and the enumerator.
pub const SCHED_SWITCH_HEADER: &str = include_str!("../sched/switch.h");

/// Size in bytes of the opaque record block an emitted frame reserves for one
/// outstanding completion operation.
///
/// This is the emitter's half of `WF_COMPLETION_RECORD_BYTES`: the frame
/// reserves the block and passes its address to submit and to join, and the
/// runtime owns whatever it keeps there. Neither side may state the number
/// alone, so `the_record_block_abi_constants_agree_with_the_contract_header`
/// below reads the header's own text and refuses a compilation in which the
/// two have drifted apart.
pub(crate) const COMPLETION_RECORD_BYTES: u64 = 128;

/// Alignment of that same block, the emitter's half of
/// `WF_COMPLETION_RECORD_ALIGN`. A byte block's natural alignment is one, so
/// the reservation states this one explicitly.
pub(crate) const COMPLETION_RECORD_ALIGN: u64 = 8;

/// The element type an emitted frame reserves per outstanding operation.
///
/// It is a byte block and not a typed record on purpose: the emitted module
/// holds one opaque pointer into it and never learns the layout.
pub(crate) fn completion_record_element_type() -> String {
    format!("[{COMPLETION_RECORD_BYTES} x i8]")
}

/// The emitted call spellings of the seven submit entries.
///
/// A module that contains one of these has handed an operation to the
/// completion runtime and cannot run without it. There is nothing weaker to
/// look for any more: the weak fallback definitions this used to key on are
/// gone with the inline arm they selected, so a link without the runtime is an
/// unresolved symbol rather than a program that silently runs the other arm
/// (`research/investigations/io-model/PARK-ON-MISS.md` §8).
const COMPLETION_SUBMIT_CALLS: [&str; 7] = [
    "call void @wf__completion_file_read_submit(",
    "call void @wf__completion_file_pread_submit(",
    "call void @wf__completion_file_write_submit(",
    "call void @wf__completion_file_open_at_submit(",
    "call void @wf__completion_file_status_submit(",
    "call void @wf__completion_file_close_submit(",
    "call void @wf__completion_directory_next_submit(",
];

/// The completion ABI an emitted module names.
///
/// Every submit answers nothing: the runtime either accepted the operation or
/// executed it itself, and either way the record it was given is published and
/// will be joined (design §7, "never with a 0 the caller must interpret").
/// There is no verdict, and therefore no second lowering to select with one.
pub(crate) const COMPLETION_RUNTIME_DECLARATIONS: &str = concat!(
    "declare void @wf__completion_file_read_submit(i32, ptr, i64, ptr)\n",
    "declare void @wf__completion_file_pread_submit(i32, ptr, i64, i64, ptr)\n",
    "declare void @wf__completion_file_write_submit(i32, ptr, i64, ptr)\n",
    "declare void @wf__completion_file_open_at_submit(i32, ptr, i32, i32, i32, i32, ptr)\n",
    "declare void @wf__completion_file_status_submit(i32, ptr, i64, ptr)\n",
    "declare void @wf__completion_file_close_submit(i32, ptr)\n",
    "declare void @wf__completion_directory_next_submit(i32, ptr, i64, ptr, ptr)\n",
    "declare void @wf__completion_file_join(ptr, ptr, ptr)\n",
    "declare void @wf__completion_file_open_join(ptr, ptr, ptr, ptr)\n",
);

/// The same ABI for COFF modules, whose `open_at` carries one more argument.
///
/// Windows has no optional completion backend: the compiler driver supplies
/// the native core and IOCP bridge, and omitting either is a link error. The
/// capacity wait that used to be part of this contract is gone with the
/// verdict fork that called it; core pressure is now the target runtime's own
/// business and never reaches emitted code (design §8).
pub(crate) const COMPLETION_WINDOWS_RUNTIME_DECLARATIONS: &str = concat!(
    "declare void @wf__completion_file_read_submit(i32, ptr, i64, ptr)\n",
    "declare void @wf__completion_file_pread_submit(i32, ptr, i64, i64, ptr)\n",
    "declare void @wf__completion_file_write_submit(i32, ptr, i64, ptr)\n",
    "declare void @wf__completion_file_open_at_submit(i32, ptr, i32, i32, i32, i32, i32, ptr)\n",
    "declare void @wf__completion_file_status_submit(i32, ptr, i64, ptr)\n",
    "declare void @wf__completion_file_close_submit(i32, ptr)\n",
    "declare void @wf__completion_directory_next_submit(i32, ptr, i64, ptr, ptr)\n",
    "declare void @wf__completion_file_join(ptr, ptr, ptr)\n",
    "declare void @wf__completion_file_open_join(ptr, ptr, ptr, ptr)\n",
);

/// Hard Windows declaration for the staged completion-window query.
pub(crate) const COMPLETION_WINDOWS_WINDOW_DECLARATION: &str =
    "declare i64 @wf__completion_window(i64, i64, i64)\n";

/// Weak window answer for a link without the completion unit.
///
/// One is always a legal window and reproduces the sequential program exactly,
/// so a module that asks for one and finds no runtime to answer stages no loop
/// and still publishes the same bytes. This one stays where the submit
/// fallbacks went, and for a reason they did not have: a module can ask for a
/// window without submitting anything, and such a module does not select the
/// runtime at link time, so nothing else would define this symbol for it.
pub(crate) const COMPLETION_WINDOW_FALLBACK: &str = "define weak i64 @wf__completion_window(i64 %span, i64 %slot_bytes, i64 %ceiling) {\nentry:\n  ret i64 1\n}\n\n";

/// True exactly when this emitted module contains a completion actualization.
pub fn module_requires_completion_runtime(module: &str) -> bool {
    COMPLETION_SUBMIT_CALLS
        .iter()
        .any(|call| module.contains(call))
        || module.contains("@wf__completion_file_pread_direct")
        || module.contains("@wf__completion_file_write_direct")
        || module.contains("@wf__completion_file_open_at_direct")
        || module.contains("@wf__completion_file_status_direct")
        || module.contains("@wf__completion_file_close_direct")
        || module.contains("@wf__completion_directory_next_direct")
}

/// Whether this operation's completion lowering can reach its outcome without
/// a submission, and therefore reserves a result slot and a `submitted` flag.
///
/// Exactly one shape can: an open by component name whose name is empty, over
/// the target family's limit, or carrying a separator. That name never reaches
/// a host call, in either lowering, so there is no operation to submit and the
/// typed invalid-path outcome is built where the name was refused. Every other
/// shape submits on every path it can take, so its `submitted` is the constant
/// true and is not represented at all (design §8).
pub(super) const fn completion_may_skip_submission(operation: CompletionFileOperation) -> bool {
    matches!(
        operation,
        CompletionFileOperation::OpenDirectory | CompletionFileOperation::OpenFile
    )
}

#[derive(Clone, Debug)]
pub(crate) struct CompletionHandedOut {
    result: IrValueId,
    result_type: IrType,
    operation: CompletionFileOperation,
    token: CompletionStorage,
    raw_value: CompletionStorage,
    raw_error: CompletionStorage,
    /// Present only where a route without a submission exists at all.
    not_submitted: Option<CompletionNotSubmitted>,
    mapping: CompletionMapping,
}

/// The two elements the one not-submitted route needs, and nothing else has.
///
/// An open by an invalid component name produces its typed outcome where the
/// name is refused, with no host call and no record: the outcome goes in
/// `result_slot`, `submitted` says which route ran, and the join loads the
/// slot instead of waiting. Every other shape submits on every path, so it
/// carries no flag rather than a phi of one constant (design §8).
#[derive(Clone, Debug)]
struct CompletionNotSubmitted {
    result_slot: CompletionStorage,
    submitted: CompletionCaptured,
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
    /// The entry-block field containing the complete fixed-size array.
    reservation: String,
    element_type: String,
    slots: u64,
}

#[derive(Clone, Debug)]
enum CompletionMapping {
    Open {
        outcome: CompletionStorage,
    },
    Transfer {
        start: CompletionCaptured,
        extent: CompletionCaptured,
    },
    DirectoryNext {
        destination_type: String,
        destination: CompletionCaptured,
        start: CompletionCaptured,
        extent: CompletionCaptured,
    },
}

/// A value needed when one submitted operation is retired.
///
/// A one-slot schedule may keep the defining SSA name. A multi-slot issue
/// loop executes that definition several times before the drain starts, so it
/// stores the value in the operation's ring element and the drain reloads the
/// element selected by its own proved slot index.
#[derive(Clone, Debug)]
enum CompletionCaptured {
    Immediate(String),
    PerSlot {
        ty: String,
        storage: CompletionStorage,
    },
}

impl FunctionEmitter<'_, '_> {
    /// Joins the named prior operations owned by this block, leaving every
    /// unrelated operation and a pipeline result protected for another exact
    /// drain in flight.
    pub(super) fn emit_completion_dependencies(
        &mut self,
        dependencies: &[IrValueId],
    ) -> Result<(), BackendFailure> {
        for dependency in dependencies {
            // A driven pipeline owns one result across its feeder edge. Only
            // the exact generated drain may retire that result; another block
            // can appear between the two in linear emission order without
            // lying on that runtime path.
            if !self.block_drains
                && self
                    .pipeline
                    .and_then(crate::IrCompletionPipeline::driven_result)
                    == Some(*dependency)
            {
                continue;
            }
            let Some(position) = self.handed_out.iter().position(|pending| {
                matches!(pending, HandedOut::Completion(pending) if pending.result == *dependency)
            }) else {
                continue;
            };
            let HandedOut::Completion(pending) = self.handed_out.remove(position) else {
                return Err(BackendFailure::InvalidIr);
            };
            self.emit_completion_join(*pending)?;
        }
        Ok(())
    }

    /// Consumes ordinary outstanding direct target operations before leaving
    /// a block. A driven pipeline result remains protected until its exact
    /// drain; compute-lane hand-outs remain owned by their overlap join.
    pub(super) fn emit_all_completion_joins(&mut self) -> Result<(), BackendFailure> {
        self.emit_outstanding_completion_joins()
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
    /// The source-derived batch constructor assigns a proved slot to both the
    /// issue and drain blocks. If that compiler invariant is absent, emission
    /// stops instead of silently selecting element zero. The separate
    /// outstanding-site check likewise catches an internal schedule defect
    /// before two operations could share one element.
    fn completion_entry_slot(
        &mut self,
        site: IrValueId,
        role: CompletionSlot,
        ty: &str,
    ) -> Result<CompletionStorage, BackendFailure> {
        if self.completion_operation_is_outstanding(site) {
            return Err(BackendFailure::SecondOutstandingCompletionOperation);
        }
        let slots = if self.block_carries {
            self.pipeline.map_or(1, crate::IrCompletionPipeline::slots)
        } else {
            1
        };
        if slots > 1 && self.block_slot.is_none() {
            return Err(BackendFailure::MisaddressedCompletionSlot);
        }
        Ok(CompletionStorage {
            reservation: self.entry_slot(FunctionSlot::Completion(site, role))?,
            element_type: ty.to_owned(),
            slots,
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
    /// has to be the other.
    fn completion_storage_pointer(
        &mut self,
        storage: &CompletionStorage,
    ) -> Result<String, BackendFailure> {
        let slot = if storage.slots == 1 {
            "0".to_owned()
        } else {
            let slot = self
                .block_slot
                .ok_or(BackendFailure::MisaddressedCompletionSlot)?;
            self.value_name(slot)
        };
        let element = format!("%{}", self.next_temporary()?);
        let array = &storage.reservation;
        let slots = storage.slots;
        let element_type = &storage.element_type;
        writeln!(
            self.output,
            "  {element} = getelementptr inbounds [{slots} x {element_type}], ptr {array}, \
             i64 0, i64 {slot}"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        Ok(element)
    }

    fn capture_completion_value(
        &mut self,
        site: IrValueId,
        role: CompletionSlot,
        ty: &str,
        value: String,
    ) -> Result<CompletionCaptured, BackendFailure> {
        let uses_ring =
            self.pipeline.is_some_and(|pipeline| pipeline.slots() > 1) && self.block_carries;
        if !uses_ring {
            return Ok(CompletionCaptured::Immediate(value));
        }
        let storage = self.completion_entry_slot(site, role, ty)?;
        let pointer = self.completion_storage_pointer(&storage)?;
        writeln!(self.output, "  store {ty} {value}, ptr {pointer}")
            .map_err(|_| BackendFailure::TextEmission)?;
        Ok(CompletionCaptured::PerSlot {
            ty: ty.to_owned(),
            storage,
        })
    }

    fn load_completion_value(
        &mut self,
        captured: CompletionCaptured,
    ) -> Result<String, BackendFailure> {
        match captured {
            CompletionCaptured::Immediate(value) => Ok(value),
            CompletionCaptured::PerSlot { ty, storage } => {
                let pointer = self.completion_storage_pointer(&storage)?;
                let value = format!("%{}", self.next_temporary()?);
                writeln!(self.output, "  {value} = load {ty}, ptr {pointer}")
                    .map_err(|_| BackendFailure::TextEmission)?;
                Ok(value)
            }
        }
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

        let token = self.completion_entry_slot(
            result,
            CompletionSlot::Record,
            &completion_record_element_type(),
        )?;
        let raw_value = self.completion_entry_slot(result, CompletionSlot::RawValue, "i64")?;
        let raw_error = self.completion_entry_slot(result, CompletionSlot::RawError, "i32")?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let extent = format!("%{}", self.next_temporary()?);
        let base = format!("%{}", self.next_temporary()?);
        let target = format!("%{}", self.next_temporary()?);
        let submit_symbol = match (completion, file_offset) {
            (CompletionFileOperation::Read, Some(_)) => "wf__completion_file_pread_submit",
            (CompletionFileOperation::Read, None) => "wf__completion_file_read_submit",
            (CompletionFileOperation::Write, None) => "wf__completion_file_write_submit",
            (CompletionFileOperation::Write, Some(_)) => return Err(BackendFailure::InvalidIr),
            _ => return Err(BackendFailure::InvalidIr),
        };
        // The operation still has to be one this target qualifies, even though
        // the handed-out lowering no longer names its direct wrapper.
        let _qualified = self.qualification.operation(operation)?;
        let rendered_buffer = llvm_type(self.program, buffer_type)?;
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

        // One lowering, and no branch before it (design §8). An empty transfer
        // is completed by the runtime, and a file offset the target ABI cannot
        // express is published by it as the host's own refusal, so neither is
        // a reason to select a second arm here.
        writeln!(
            self.output,
            "  {extent} = sub i64 {}, {}\n  \
             {base} = extractvalue {rendered_buffer} {}, 0\n  \
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}",
            self.value_name(*end),
            self.value_name(*start),
            self.value_name(*buffer),
            self.value_name(*start),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        writeln!(
            self.output,
            "  call void @{submit_symbol}({submit_arguments})"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let captured_start = self.capture_completion_value(
            result,
            CompletionSlot::Start,
            "i64",
            self.value_name(*start),
        )?;
        let captured_extent =
            self.capture_completion_value(result, CompletionSlot::Extent, "i64", extent)?;

        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(Box::new(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                token,
                raw_value,
                raw_error,
                not_submitted: None,
                mapping: CompletionMapping::Transfer {
                    start: captured_start,
                    extent: captured_extent,
                },
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
        let not_submitted_label = completion_not_submitted_label(result);
        let offered_label = completion_offered_label(result);
        let rendered_type = llvm_type(self.program, ty)?;
        // Only an open by component name has a route that reaches an outcome
        // without submitting, and only it reserves the two elements that route
        // needs.
        let refusable = completion_may_skip_submission(completion);
        // A component path may branch to that route before reaching
        // `request_label`. Ring element pointers used by both paths must
        // therefore be defined before path preparation opens that branch.
        let token = self.completion_entry_slot(
            result,
            CompletionSlot::Record,
            &completion_record_element_type(),
        )?;
        let result_slot = if refusable {
            Some(self.completion_entry_slot(result, CompletionSlot::Result, &rendered_type)?)
        } else {
            None
        };
        let raw_value = self.completion_entry_slot(result, CompletionSlot::RawValue, "i64")?;
        let raw_error = self.completion_entry_slot(result, CompletionSlot::RawError, "i32")?;
        let open_outcome =
            self.completion_entry_slot(result, CompletionSlot::OpenOutcome, "i32")?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let result_pointer = match &result_slot {
            Some(slot) => Some(self.completion_storage_pointer(slot)?),
            None => None,
        };
        let (directory, path, flags) = match completion {
            CompletionFileOperation::OpenRead => {
                let [.., directory, path] = arguments else {
                    return Err(BackendFailure::InvalidIr);
                };
                let path_ty = self.value_type(*path).ok_or(BackendFailure::InvalidIr)?;
                let text = format!("%{}", self.next_temporary()?);
                writeln!(
                    self.output,
                    "  {text} = extractvalue {} {}, 0",
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
                    &not_submitted_label,
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
        // The operation still has to be one this target qualifies, even though
        // the handed-out lowering no longer names its direct wrapper.
        let _qualified = self.qualification.operation(operation)?;
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
            "  call void @wf__completion_file_open_at_submit(i32 {}, ptr {path}, \
             i32 {flags}, i32 0, i32 0, i32 {expected_kind}{descriptor_class_argument}, \
             ptr {token_pointer})",
            self.value_name(directory),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let not_submitted = match (result_slot, result_pointer) {
            (Some(result_slot), Some(result_pointer)) => {
                let submitted = format!("%{}", self.next_temporary()?);
                writeln!(
                    self.output,
                    "  br label %{offered_label}\n\
                     {not_submitted_label}:"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                let refusal = system::completion_invalid_component_outcome(
                    self.program,
                    ty,
                    &completion_refusal_prefix(result),
                    &result_pointer,
                )?;
                writeln!(
                    self.output,
                    "{refusal}  \
                     br label %{offered_label}\n\
                     {offered_label}:\n  \
                     {submitted} = phi i1 [ true, %{request_label} ], \
                     [ false, %{not_submitted_label} ]"
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                let submitted = self.capture_completion_value(
                    result,
                    CompletionSlot::Submitted,
                    "i1",
                    submitted,
                )?;
                Some(CompletionNotSubmitted {
                    result_slot,
                    submitted,
                })
            }
            (None, None) => None,
            _ => return Err(BackendFailure::InvalidIr),
        };
        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(Box::new(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                token,
                raw_value,
                raw_error,
                not_submitted,
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
        not_submitted_label: &str,
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
        let staged = self.completion_entry_slot(
            result,
            CompletionSlot::Component,
            &format!("[{slot} x i8]"),
        )?;
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
                 br i1 {unusable}, label %{not_submitted_label}, label %{scan_entry}\n\
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
                 br i1 {refused}, label %{not_submitted_label}, label %{scan_step}\n\
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
                 br i1 {unusable}, label %{not_submitted_label}, label %{scan_entry}\n\
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
                 br i1 {refused}, label %{not_submitted_label}, label %{scan_step}\n\
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
        let token = self.completion_entry_slot(
            result,
            CompletionSlot::Record,
            &completion_record_element_type(),
        )?;
        let raw_value = self.completion_entry_slot(result, CompletionSlot::RawValue, "i64")?;
        let raw_error = self.completion_entry_slot(result, CompletionSlot::RawError, "i32")?;
        let cursor = self.completion_entry_slot(result, CompletionSlot::Cursor, "i64")?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let position = self.completion_storage_pointer(&cursor)?;
        let extent = format!("%{}", self.next_temporary()?);
        let base = format!("%{}", self.next_temporary()?);
        let target = format!("%{}", self.next_temporary()?);
        // The operation still has to be one this target qualifies, even though
        // the handed-out lowering no longer names its direct wrapper.
        let _qualified = self.qualification.operation(operation)?;
        // An empty destination range is submitted like any other and completed
        // by the runtime, so there is no branch and no second arm (design §8).
        writeln!(
            self.output,
            "  {extent} = sub i64 {}, {}\n  \
             store i64 0, ptr {position}, align 8\n  \
             {base} = extractvalue {destination_llvm} {}, 0\n  \
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}",
            self.value_name(*end),
            self.value_name(*start),
            self.value_name(*destination),
            self.value_name(*start),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        writeln!(
            self.output,
            "  call void @wf__completion_directory_next_submit(i32 {}, ptr {target}, \
             i64 {extent}, ptr {position}, ptr {token_pointer})",
            self.value_name(*source),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let captured_destination = self.capture_completion_value(
            result,
            CompletionSlot::Destination,
            &destination_llvm,
            self.value_name(*destination),
        )?;
        let captured_start = self.capture_completion_value(
            result,
            CompletionSlot::Start,
            "i64",
            self.value_name(*start),
        )?;
        let captured_extent =
            self.capture_completion_value(result, CompletionSlot::Extent, "i64", extent)?;
        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(Box::new(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                token,
                raw_value,
                raw_error,
                not_submitted: None,
                mapping: CompletionMapping::DirectoryNext {
                    destination_type: destination_llvm,
                    destination: captured_destination,
                    start: captured_start,
                    extent: captured_extent,
                },
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
            token,
            raw_value,
            raw_error,
            not_submitted,
            mapping,
        } = pending;
        // The element pointers are materialized here rather than carried from
        // the submission, because a retirement need not be in the block that
        // submitted and, under a ring, need not mean the same slot: what it
        // retires is the operation the slot index of *this* block names.
        let token = self.completion_storage_pointer(&token)?;
        let raw_value = self.completion_storage_pointer(&raw_value)?;
        let raw_error = self.completion_storage_pointer(&raw_error)?;
        let not_submitted = match not_submitted {
            Some(CompletionNotSubmitted {
                result_slot,
                submitted,
            }) => {
                let result_slot = self.completion_storage_pointer(&result_slot)?;
                let submitted = self.load_completion_value(submitted)?;
                Some((result_slot, submitted))
            }
            None => None,
        };
        let completed_value = format!("%{}", self.next_temporary()?);
        let completed_error = format!("%{}", self.next_temporary()?);
        let result_llvm = llvm_type(self.program, result_type)?;
        let (join_call, extra_load, mapper_arguments) = match mapping {
            CompletionMapping::Open { outcome } => {
                let outcome = self.completion_storage_pointer(&outcome)?;
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
                let start = self.load_completion_value(start)?;
                let extent = self.load_completion_value(extent)?;
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
                destination_type,
                destination,
                start,
                extent,
            } => {
                let destination = self.load_completion_value(destination)?;
                let start = self.load_completion_value(start)?;
                let extent = self.load_completion_value(extent)?;
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
        let mapper = completion_mapper_symbol(operation);
        let Some((result_slot, submitted)) = not_submitted else {
            // Every path through this operation submitted, so the wait is the
            // whole join: no branch, no phi and no block of its own.
            return writeln!(
                self.output,
                "  {join_call}\n  \
                 {completed_value} = load i64, ptr {raw_value}\n  \
                 {completed_error} = load i32, ptr {raw_error}\n  \
                 {extra_load}\
                 {} = call {result_llvm} @{mapper}({mapper_arguments})",
                value_name(result),
            )
            .map_err(|_| BackendFailure::TextEmission);
        };
        let direct = format!("%{}", self.next_temporary()?);
        let completed = format!("%{}", self.next_temporary()?);
        let inline_label = completion_join_inline_label(result);
        let wait_label = completion_wait_label(result);
        let done_label = par_done_label(result);
        writeln!(
            self.output,
            "  br i1 {submitted}, label %{wait_label}, label %{inline_label}\n\
             {inline_label}:\n  \
             {direct} = load {result_llvm}, ptr {result_slot}\n  \
             br label %{done_label}\n\
             {wait_label}:\n  \
             {join_call}\n  \
             {completed_value} = load i64, ptr {raw_value}\n  \
             {completed_error} = load i32, ptr {raw_error}\n  \
             {extra_load}\
             {completed} = call {result_llvm} @{mapper}({mapper_arguments})\n  \
             br label %{done_label}\n\
             {done_label}:\n  \
             {} = phi {result_llvm} [ {direct}, %{inline_label} ], [ {completed}, %{wait_label} ]",
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}

fn completion_submit_label(value: IrValueId) -> String {
    format!("completion.submit.v{}", value.ordinal())
}

/// The one route that reaches an outcome without a submission: an open whose
/// component name the target family cannot mean.
fn completion_not_submitted_label(value: IrValueId) -> String {
    format!("completion.not_submitted.v{}", value.ordinal())
}

/// The unique name prefix that route's typed outcome is built under.
fn completion_refusal_prefix(value: IrValueId) -> String {
    format!("completion.refused.v{}", value.ordinal())
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

#[cfg(test)]
mod tests {
    use super::{
        COMPLETION_CONTRACT_HEADER, COMPLETION_RECORD_ALIGN, COMPLETION_RECORD_BYTES,
        COMPLETION_WINDOWS_NATIVE_API_HEADER,
    };

    /// The value of one `#define NAME <digits>u` in an embedded C header.
    fn defined_unsigned(header: &str, name: &str) -> u64 {
        let definition = format!("#define {name} ");
        let mut matches = header.match_indices(&definition);
        let (start, _) = matches.next().unwrap_or_else(|| {
            panic!("the embedded header does not define {name}");
        });
        assert!(
            matches.next().is_none(),
            "{name} must be defined exactly once so the two sides cannot disagree"
        );
        let rest = &header[start + definition.len()..];
        let line = rest.lines().next().unwrap_or_default().trim();
        line.trim_end_matches('u')
            .parse()
            .unwrap_or_else(|_| panic!("{name} is not a plain unsigned literal: {line}"))
    }

    /// The emitter's half of the two-sided record-block assertion.
    ///
    /// The C side asserts that the record it stores fits the block and does
    /// not out-align it. That assertion is only as good as the numbers the
    /// emitter actually reserved by, and those live in Rust, so the header's
    /// own text is read here and compared with them. A block reserved by one
    /// number and written by another is a kernel write past the reservation,
    /// which this turns into a failing build.
    #[test]
    fn the_record_block_abi_constants_agree_with_the_contract_header() {
        assert_eq!(
            defined_unsigned(COMPLETION_CONTRACT_HEADER, "WF_COMPLETION_RECORD_BYTES"),
            COMPLETION_RECORD_BYTES
        );
        assert_eq!(
            defined_unsigned(COMPLETION_CONTRACT_HEADER, "WF_COMPLETION_RECORD_ALIGN"),
            COMPLETION_RECORD_ALIGN
        );
        // A Windows translation unit reaches the same two constants through
        // the native mirror of this contract instead, which imports no POSIX
        // threading API. The mirror is a second spelling of one ABI, so it is
        // held to the same numbers.
        assert_eq!(
            defined_unsigned(
                COMPLETION_WINDOWS_NATIVE_API_HEADER,
                "WF_COMPLETION_RECORD_BYTES"
            ),
            COMPLETION_RECORD_BYTES
        );
        assert_eq!(
            defined_unsigned(
                COMPLETION_WINDOWS_NATIVE_API_HEADER,
                "WF_COMPLETION_RECORD_ALIGN"
            ),
            COMPLETION_RECORD_ALIGN
        );
    }
}
