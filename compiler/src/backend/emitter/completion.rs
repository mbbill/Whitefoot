//! Finite one-shot completion actualization for direct system operations.
//!
//! This is deliberately narrower than the compute hand-out path.  Only a
//! compiler-owned `read_at` or `write_once` identity reaches the typed file
//! adapter; a Whitefoot function, wrapper, callback, or source spelling never
//! crosses that boundary.

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
pub(crate) const COMPLETION_RUNTIME_FALLBACK: &str = "define weak i32 @wf__completion_file_read_submit(i32 %descriptor, ptr %buffer, i64 %count, ptr %token) {\nentry:\n  ret i32 0\n}\n\ndefine weak i32 @wf__completion_file_pread_submit(i32 %descriptor, ptr %buffer, i64 %count, i64 %file_offset, ptr %token) {\nentry:\n  ret i32 0\n}\n\ndefine weak i32 @wf__completion_file_write_submit(i32 %descriptor, ptr %buffer, i64 %count, ptr %token) {\nentry:\n  ret i32 0\n}\n\ndefine weak i32 @wf__completion_file_batch_claim(ptr %tokens, i32 %count, i32 %requires_fallback) {\nentry:\n  ret i32 0\n}\n\ndefine weak void @wf__completion_file_pread_submit_reserved(i32 %descriptor, ptr %buffer, i64 %count, i64 %file_offset, ptr %token) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__completion_file_write_submit_reserved(i32 %descriptor, ptr %buffer, i64 %count, ptr %token) {\nentry:\n  ret void\n}\n\ndefine weak i64 @wf__completion_output_batch_begin(i64 %key, i32 %count) {\nentry:\n  ret i64 0\n}\n\ndefine weak void @wf__completion_output_batch_submit(i64 %key, i32 %descriptor, ptr %buffer, i64 %count, ptr %token) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__completion_output_batch_commit(i64 %key) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__completion_file_join(ptr %token, ptr %value, ptr %error) {\nentry:\n  ret void\n}\n\n";

/// True exactly when this emitted module contains a completion actualization.
pub fn module_requires_completion_runtime(module: &str) -> bool {
    module.contains(COMPLETION_MARKER)
        || module.contains("@wf__completion_file_pread_submit_writer")
        || module.contains("@wf__completion_file_write_submit_writer")
        || module.contains("@wf__completion_file_pread_direct")
        || module.contains("@wf__completion_file_write_direct")
        || module.contains("@wf__completion_directory_next_direct")
}

#[derive(Clone, Debug)]
pub(crate) struct CompletionHandedOut {
    result: IrValueId,
    result_type: IrType,
    operation: CompletionFileOperation,
    token: String,
    result_slot: String,
    raw_value: String,
    raw_error: String,
    submitted: String,
    start: String,
    extent: String,
}

#[derive(Clone, Debug)]
pub(crate) struct OrderedOutputEmission {
    join_site: IrValueId,
    key: String,
    active: String,
    expected: usize,
    submitted: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct FreeFileBatchEmission {
    join_site: IrValueId,
    tokens: String,
    active: String,
    expected: usize,
    submitted: usize,
}

impl FunctionEmitter<'_, '_> {
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
        let ordered = self.overlaps.iter().find_map(|overlap| {
            (overlap.ordered_attribution() == Some(crate::SystemAuthorityAttribution::OutputBytes)
                && overlap.dispatched().contains(&result))
            .then(|| {
                (
                    overlap.join_site(),
                    overlap.dispatched().len(),
                    overlap
                        .dispatched()
                        .iter()
                        .position(|value| *value == result),
                )
            })
        });
        if let Some((Some(join_site), expected, Some(position))) = ordered {
            return self.emit_ordered_output_call(
                result, ty, operation, arguments, join_site, expected, position,
            );
        }
        let free_batch = self.overlaps.iter().find_map(|overlap| {
            (overlap.ordered_attribution().is_none()
                && overlap.dispatched().len() == overlap.members().len()
                && overlap.dispatched().contains(&result))
            .then(|| {
                (
                    overlap.join_site(),
                    overlap.dispatched().len(),
                    overlap
                        .dispatched()
                        .iter()
                        .position(|value| *value == result),
                    overlap.dispatched().iter().any(|member| {
                        matches!(
                            definition_operation(self.function, *member),
                            Some(IrOperation::SystemCall { operation, .. })
                                if completion_file_operation(*operation)
                                    == Some(CompletionFileOperation::Write)
                        )
                    }),
                )
            })
        });
        if let Some((Some(join_site), expected, Some(position), requires_fallback)) = free_batch {
            return self.emit_free_file_batch_call(
                result,
                ty,
                operation,
                arguments,
                join_site,
                expected,
                position,
                requires_fallback,
            );
        }
        let completion = completion_file_operation(operation).ok_or(BackendFailure::InvalidIr)?;
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

        let token = self.entry_slot("[2 x i64]")?;
        let result_slot = self.entry_slot(&llvm_type(self.program, ty)?)?;
        let raw_value = self.entry_slot("i64")?;
        let raw_error = self.entry_slot("i32")?;
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
                "i32 {}, ptr {target}, i64 {extent}, i64 {}, ptr {token}",
                self.value_name(*resource),
                self.value_name(*offset)
            )
        } else {
            format!(
                "i32 {}, ptr {target}, i64 {extent}, ptr {token}",
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
             store {rendered_type} {inline_result}, ptr {result_slot}\n  \
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

        *self.completion_used = true;
        self.handed_out
            .push(HandedOut::Completion(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                token,
                result_slot,
                raw_value,
                raw_error,
                submitted,
                start: self.value_name(*start),
                extent,
            }));
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_free_file_batch_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: crate::IrSystemOperation,
        arguments: &[IrValueId],
        join_site: IrValueId,
        expected: usize,
        position: usize,
        requires_fallback: bool,
    ) -> Result<(), BackendFailure> {
        if !(2..=crate::FREE_COMPLETION_BATCH_MEMBERS).contains(&expected) || position >= expected {
            return Err(BackendFailure::InvalidIr);
        }
        let completion = completion_file_operation(operation).ok_or(BackendFailure::InvalidIr)?;
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
            || self
                .value_type(*resource)
                .is_none_or(|resource| llvm_type(self.program, resource).as_deref() != Ok("i32"))
        {
            return Err(BackendFailure::InvalidIr);
        }
        let token_array_type = format!("[{expected} x [2 x i64]]");
        if position == 0 {
            let tokens = self.entry_slot(&token_array_type)?;
            let status = format!("%{}", self.next_temporary()?);
            let active = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {status} = call i32 @wf__completion_file_batch_claim(ptr {tokens}, i32 {expected}, i32 {})\n  \
                 {active} = icmp eq i32 {status}, 1"
                ,
                u8::from(requires_fallback)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            self.free_file_batch = Some(FreeFileBatchEmission {
                join_site,
                tokens,
                active,
                expected,
                submitted: 0,
            });
        }
        let batch = self
            .free_file_batch
            .as_ref()
            .ok_or(BackendFailure::InvalidIr)?;
        if batch.join_site != join_site || batch.expected != expected || batch.submitted != position
        {
            return Err(BackendFailure::InvalidIr);
        }
        let tokens = batch.tokens.clone();
        let active = batch.active.clone();
        let token = format!("%{}", self.next_temporary()?);
        let result_llvm = llvm_type(self.program, ty)?;
        let result_slot = self.entry_slot(&result_llvm)?;
        let raw_value = self.entry_slot("i64")?;
        let raw_error = self.entry_slot("i32")?;
        let extent = format!("%{}", self.next_temporary()?);
        let base = format!("%{}", self.next_temporary()?);
        let target = format!("%{}", self.next_temporary()?);
        let direct = format!("%{}", self.next_temporary()?);
        let submit_label = format!("completion.free.submit.v{}", result.ordinal());
        let inline_label = completion_inline_label(result);
        let offered_label = completion_offered_label(result);
        let implementation = self.qualification.operation(operation)?;
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
        let (submit_symbol, submit_arguments) = match (completion, file_offset) {
            (CompletionFileOperation::Read, Some(offset)) => (
                "wf__completion_file_pread_submit_reserved",
                format!(
                    "i32 {}, ptr {target}, i64 {extent}, i64 {}, ptr {token}",
                    self.value_name(*resource),
                    self.value_name(*offset)
                ),
            ),
            (CompletionFileOperation::Write, None) => (
                "wf__completion_file_write_submit_reserved",
                format!(
                    "i32 {}, ptr {target}, i64 {extent}, ptr {token}",
                    self.value_name(*resource)
                ),
            ),
            _ => return Err(BackendFailure::InvalidIr),
        };
        writeln!(
            self.output,
            "  {token} = getelementptr inbounds {token_array_type}, ptr {tokens}, i32 0, i32 {position}\n  \
             {extent} = sub i64 {}, {}\n  \
             br i1 {active}, label %{submit_label}, label %{inline_label}\n\
             {submit_label}:\n  \
             {base} = extractvalue {} {}, 0\n  \
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}\n  \
             call void @{submit_symbol}({submit_arguments})\n  \
             br label %{offered_label}\n\
             {inline_label}:\n  \
             {direct} = call {result_llvm} @{}({})\n  \
             store {result_llvm} {direct}, ptr {result_slot}\n  \
             br label %{offered_label}\n\
             {offered_label}:",
            self.value_name(*end),
            self.value_name(*start),
            llvm_type(self.program, buffer_type)?,
            self.value_name(*buffer),
            self.value_name(*start),
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.handed_out
            .push(HandedOut::Completion(CompletionHandedOut {
                result,
                result_type: ty,
                operation: completion,
                token,
                result_slot,
                raw_value,
                raw_error,
                submitted: active,
                start: self.value_name(*start),
                extent,
            }));
        *self.completion_used = true;
        if position + 1 == expected {
            self.free_file_batch = None;
        } else if let Some(batch) = self.free_file_batch.as_mut() {
            batch.submitted += 1;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_ordered_output_call(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: crate::IrSystemOperation,
        arguments: &[IrValueId],
        join_site: IrValueId,
        expected: usize,
        position: usize,
    ) -> Result<(), BackendFailure> {
        if completion_file_operation(operation) != Some(CompletionFileOperation::Write)
            || !(2..=crate::ORDERED_OUTPUT_BATCH_MEMBERS).contains(&expected)
            || position >= expected
        {
            return Err(BackendFailure::InvalidIr);
        }
        let [resource, buffer, start, end] = arguments else {
            return Err(BackendFailure::InvalidIr);
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
            || self
                .value_type(*resource)
                .is_none_or(|resource| llvm_type(self.program, resource).as_deref() != Ok("i32"))
        {
            return Err(BackendFailure::InvalidIr);
        }
        if position == 0 {
            let root = self.entry_slot("[1 x i8]")?;
            let logical_root = format!("%{}", self.next_temporary()?);
            let key = format!("%{}", self.next_temporary()?);
            let active = format!("%{}", self.next_temporary()?);
            writeln!(
                self.output,
                "  {logical_root} = ptrtoint ptr {root} to i64\n  \
                 {key} = call i64 @wf__completion_output_batch_begin(i64 {logical_root}, i32 {expected})\n  \
                 {active} = icmp ne i64 {key}, 0"
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            self.ordered_output = Some(OrderedOutputEmission {
                join_site,
                key,
                active,
                expected,
                submitted: 0,
            });
        }
        let batch = self
            .ordered_output
            .as_ref()
            .ok_or(BackendFailure::InvalidIr)?;
        if batch.join_site != join_site || batch.expected != expected || batch.submitted != position
        {
            return Err(BackendFailure::InvalidIr);
        }
        let key = batch.key.clone();
        let active = batch.active.clone();
        let token = self.entry_slot("[2 x i64]")?;
        let result_llvm = llvm_type(self.program, ty)?;
        let result_slot = self.entry_slot(&result_llvm)?;
        let raw_value = self.entry_slot("i64")?;
        let raw_error = self.entry_slot("i32")?;
        let extent = format!("%{}", self.next_temporary()?);
        let base = format!("%{}", self.next_temporary()?);
        let target = format!("%{}", self.next_temporary()?);
        let direct = format!("%{}", self.next_temporary()?);
        let submit_label = format!("completion.ordered.submit.v{}", result.ordinal());
        let inline_label = completion_inline_label(result);
        let offered_label = completion_offered_label(result);
        let implementation = self.qualification.operation(operation)?;
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
        let commit = if position + 1 == expected {
            format!("  call void @wf__completion_output_batch_commit(i64 {key})\n")
        } else {
            String::new()
        };
        writeln!(
            self.output,
            "  {extent} = sub i64 {}, {}\n  \
             br i1 {active}, label %{submit_label}, label %{inline_label}\n\
             {submit_label}:\n  \
             {base} = extractvalue {} {}, 0\n  \
             {target} = getelementptr inbounds i8, ptr {base}, i64 {}\n  \
             call void @wf__completion_output_batch_submit(i64 {key}, i32 {}, ptr {target}, \
             i64 {extent}, ptr {token})\n\
             {commit}  br label %{offered_label}\n\
             {inline_label}:\n  \
             {direct} = call {result_llvm} @{}({})\n  \
             store {result_llvm} {direct}, ptr {result_slot}\n  \
             br label %{offered_label}\n\
             {offered_label}:",
            self.value_name(*end),
            self.value_name(*start),
            llvm_type(self.program, buffer_type)?,
            self.value_name(*buffer),
            self.value_name(*start),
            self.value_name(*resource),
            implementation.symbol(),
            rendered_arguments.join(", "),
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        self.handed_out
            .push(HandedOut::Completion(CompletionHandedOut {
                result,
                result_type: ty,
                operation: CompletionFileOperation::Write,
                token,
                result_slot,
                raw_value,
                raw_error,
                submitted: active,
                start: self.value_name(*start),
                extent,
            }));
        *self.completion_used = true;
        if position + 1 == expected {
            self.ordered_output = None;
        } else if let Some(batch) = self.ordered_output.as_mut() {
            batch.submitted += 1;
        }
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
            start,
            extent,
        } = pending;
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
             {inline_label}:\n  \
             {direct} = load {result_llvm}, ptr {result_slot}\n  \
             br label %{done_label}\n\
             {wait_label}:\n  \
             call void @wf__completion_file_join(ptr {token}, ptr {raw_value}, ptr {raw_error})\n  \
             {completed_value} = load i64, ptr {raw_value}\n  \
             {completed_error} = load i32, ptr {raw_error}\n  \
             {completed} = call {result_llvm} @{}(i64 {completed_value}, i32 {completed_error}, \
             i64 {start}, i64 {extent})\n  \
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

fn completion_offered_label(value: IrValueId) -> String {
    format!("completion.offered.v{}", value.ordinal())
}

fn completion_join_inline_label(value: IrValueId) -> String {
    format!("completion.join.inline.v{}", value.ordinal())
}

fn completion_wait_label(value: IrValueId) -> String {
    format!("completion.wait.v{}", value.ordinal())
}
