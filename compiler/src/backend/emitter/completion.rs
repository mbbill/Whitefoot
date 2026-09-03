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
/// The stackless writer scheduler ABI embedded in the compiler.
pub const WRITER_SCHEDULER_HEADER: &str = include_str!("../completion/writer_scheduler.h");
/// The target-guarded Linux io_uring adapter contract embedded in the compiler.
pub const COMPLETION_LINUX_IO_URING_HEADER: &str = include_str!("../completion/linux_io_uring.h");
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

/// The marker definition carried only by a module that actualizes a typed
/// target operation through completion.
const COMPLETION_MARKER: &str = "define weak i32 @wf__completion_file_read_submit(i32 %descriptor, ptr %buffer, i64 %count, ptr %token)";

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
        || module.contains("@wf__completion_file_pread_submit_writer")
        || module.contains("@wf__completion_file_write_submit_writer")
        || module.contains("@wf__completion_file_pread_direct")
        || module.contains("@wf__completion_file_write_direct")
        || module.contains("@wf__completion_file_open_at_direct")
        || module.contains("@wf__completion_file_status_direct")
        || module.contains("@wf__completion_file_close_direct")
        || module.contains("@wf__completion_directory_next_direct")
}

#[derive(Clone, Debug)]
pub(crate) struct CompletionHandedOut {
    result: IrValueId,
    result_type: IrType,
    operation: CompletionFileOperation,
    token: CompletionStorage,
    result_slot: CompletionStorage,
    raw_value: CompletionStorage,
    raw_error: CompletionStorage,
    submitted: CompletionCaptured,
    mapping: CompletionMapping,
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

        let token = self.completion_entry_slot(result, CompletionSlot::Token, "[2 x i64]")?;
        let result_slot = self.completion_entry_slot(
            result,
            CompletionSlot::Result,
            &llvm_type(self.program, ty)?,
        )?;
        let raw_value = self.completion_entry_slot(result, CompletionSlot::RawValue, "i64")?;
        let raw_error = self.completion_entry_slot(result, CompletionSlot::RawError, "i32")?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let result_pointer = self.completion_storage_pointer(&result_slot)?;
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
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}\n  \
             {status} = call i32 @{submit_symbol}({submit_arguments})\n  \
             {accepted} = icmp eq i32 {status}, 1\n  \
             br i1 {accepted}, label %{offered_label}, label %{inline_label}\n\
             {inline_label}:\n  \
             {inline_result} = call {rendered_type} @{}({})\n  \
             store {rendered_type} {inline_result}, ptr {result_pointer}\n  \
             br label %{offered_label}\n\
             {offered_label}:\n  \
             {submitted} = phi i1 [ true, %{submit_label} ], [ false, %{inline_label} ]",
            self.value_name(*end),
            self.value_name(*start),
            self.value_name(*buffer),
            self.value_name(*start),
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;

        let submitted =
            self.capture_completion_value(result, CompletionSlot::Submitted, "i1", submitted)?;
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
                result_slot,
                raw_value,
                raw_error,
                submitted,
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
        let inline_label = completion_inline_label(result);
        let offered_label = completion_offered_label(result);
        let rendered_type = llvm_type(self.program, ty)?;
        // A component path may branch directly to the inline fallback before
        // reaching `request_label`. Ring element pointers used by both paths
        // must therefore be defined before path preparation opens that branch.
        let token = self.completion_entry_slot(result, CompletionSlot::Token, "[2 x i64]")?;
        let result_slot =
            self.completion_entry_slot(result, CompletionSlot::Result, &rendered_type)?;
        let raw_value = self.completion_entry_slot(result, CompletionSlot::RawValue, "i64")?;
        let raw_error = self.completion_entry_slot(result, CompletionSlot::RawError, "i32")?;
        let open_outcome =
            self.completion_entry_slot(result, CompletionSlot::OpenOutcome, "i32")?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let result_pointer = self.completion_storage_pointer(&result_slot)?;
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
        let status = format!("%{}", self.next_temporary()?);
        let accepted = format!("%{}", self.next_temporary()?);
        let inline_result = format!("%{}", self.next_temporary()?);
        let submitted = format!("%{}", self.next_temporary()?);
        let implementation = self.qualification.operation(operation)?;
        let rendered_arguments = self.rendered_system_arguments(arguments)?;
        let expected_kind = match completion {
            CompletionFileOperation::OpenRead | CompletionFileOperation::OpenFile => {
                system::OPEN_EXPECT_REGULAR
            }
            CompletionFileOperation::OpenDirectory
            | CompletionFileOperation::OpenDirectorySource => system::OPEN_EXPECT_DIRECTORY,
            _ => return Err(BackendFailure::InvalidIr),
        };
        writeln!(
            self.output,
            "  {status} = call i32 @wf__completion_file_open_at_submit(i32 {}, ptr {path}, \
             i32 {flags}, i32 0, i32 0, i32 {expected_kind}, ptr {token_pointer})\n  \
             {accepted} = icmp eq i32 {status}, 1\n  \
             br i1 {accepted}, label %{offered_label}, label %{inline_label}\n\
             {inline_label}:\n  \
             {inline_result} = call {rendered_type} @{}({})\n  \
             store {rendered_type} {inline_result}, ptr {result_pointer}\n  \
             br label %{offered_label}\n\
             {offered_label}:\n  \
             {submitted} = phi i1 [ true, %{request_label} ], [ false, %{inline_label} ]",
            self.value_name(directory),
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let submitted =
            self.capture_completion_value(result, CompletionSlot::Submitted, "i1", submitted)?;
        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(Box::new(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                token,
                result_slot,
                raw_value,
                raw_error,
                submitted,
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
        let slot = limit
            .checked_add(1)
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
        let terminating = format!("%{}", self.next_temporary()?);
        let separating = format!("%{}", self.next_temporary()?);
        let refused = format!("%{}", self.next_temporary()?);
        let next = format!("%{}", self.next_temporary()?);
        let scanned = format!("%{}", self.next_temporary()?);
        let terminator = format!("%{}", self.next_temporary()?);
        let scan_entry = format!("completion.component.entry.v{}", result.ordinal());
        let scan = format!("completion.component.scan.v{}", result.ordinal());
        let scan_step = format!("completion.component.step.v{}", result.ordinal());
        let ready = format!("completion.component.ready.v{}", result.ordinal());
        let buffer_ty = self.value_type(*name).ok_or(BackendFailure::InvalidIr)?;
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
        let token = self.completion_entry_slot(result, CompletionSlot::Token, "[2 x i64]")?;
        let result_slot = self.completion_entry_slot(
            result,
            CompletionSlot::Result,
            &llvm_type(self.program, ty)?,
        )?;
        let raw_value = self.completion_entry_slot(result, CompletionSlot::RawValue, "i64")?;
        let raw_error = self.completion_entry_slot(result, CompletionSlot::RawError, "i32")?;
        let cursor = self.completion_entry_slot(result, CompletionSlot::Cursor, "i64")?;
        let token_pointer = self.completion_storage_pointer(&token)?;
        let result_pointer = self.completion_storage_pointer(&result_slot)?;
        let position = self.completion_storage_pointer(&cursor)?;
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
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}\n  \
             {status} = call i32 @wf__completion_directory_next_submit(i32 {}, ptr {target}, \
             i64 {extent}, ptr {position}, ptr {token_pointer})\n  \
             {accepted} = icmp eq i32 {status}, 1\n  \
             br i1 {accepted}, label %{offered_label}, label %{inline_label}\n\
             {inline_label}:\n  \
             {inline_result} = call {rendered_type} @{}({})\n  \
             store {rendered_type} {inline_result}, ptr {result_pointer}\n  \
             br label %{offered_label}\n\
             {offered_label}:\n  \
             {submitted} = phi i1 [ true, %{submit_label} ], [ false, %{inline_label} ]",
            self.value_name(*end),
            self.value_name(*start),
            self.value_name(*destination),
            self.value_name(*start),
            self.value_name(*source),
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let submitted =
            self.capture_completion_value(result, CompletionSlot::Submitted, "i1", submitted)?;
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
                result_slot,
                raw_value,
                raw_error,
                submitted,
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
            result_slot,
            raw_value,
            raw_error,
            submitted,
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
        let submitted = self.load_completion_value(submitted)?;
        let direct = format!("%{}", self.next_temporary()?);
        let completed_value = format!("%{}", self.next_temporary()?);
        let completed_error = format!("%{}", self.next_temporary()?);
        let completed = format!("%{}", self.next_temporary()?);
        let inline_label = completion_join_inline_label(result);
        let wait_label = completion_wait_label(result);
        let done_label = par_done_label(result);
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
             {completed} = call {result_llvm} @{}({mapper_arguments})\n  \
             br label %{done_label}\n\
             {done_label}:\n  \
             {} = phi {result_llvm} [ {direct}, %{inline_label} ], [ {completed}, %{wait_label} ]",
            completion_mapper_symbol(operation),
            value_name(result),
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}

fn completion_submit_label(value: IrValueId) -> String {
    format!("completion.submit.v{}", value.ordinal())
}

fn completion_inline_label(value: IrValueId) -> String {
    format!("completion.inline.v{}", value.ordinal())
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
