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
    token: String,
    result_slot: String,
    raw_value: String,
    raw_error: String,
    submitted: String,
    mapping: CompletionMapping,
}

#[derive(Clone, Debug)]
enum CompletionMapping {
    Open {
        outcome: String,
    },
    Transfer {
        start: String,
        extent: String,
    },
    DirectoryNext {
        destination: String,
        start: String,
        extent: String,
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
            self.emit_completion_join(pending)?;
        }
        Ok(())
    }

    /// Consumes every outstanding direct target operation before leaving a
    /// schedule or a control-flow block. Compute-lane hand-outs remain owned
    /// by their existing overlap join.
    pub(super) fn emit_all_completion_joins(&mut self) -> Result<(), BackendFailure> {
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
                mapping: CompletionMapping::Transfer {
                    start: self.value_name(*start),
                    extent,
                },
            }));
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
        let token = self.entry_slot("[2 x i64]")?;
        let result_slot = self.entry_slot(&llvm_type(self.program, ty)?)?;
        let raw_value = self.entry_slot("i64")?;
        let raw_error = self.entry_slot("i32")?;
        let open_outcome = self.entry_slot("i32")?;
        let status = format!("%{}", self.next_temporary()?);
        let accepted = format!("%{}", self.next_temporary()?);
        let inline_result = format!("%{}", self.next_temporary()?);
        let submitted = format!("%{}", self.next_temporary()?);
        let implementation = self.qualification.operation(operation)?;
        let rendered_type = llvm_type(self.program, ty)?;
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
             i32 {flags}, i32 0, i32 0, i32 {expected_kind}, ptr {token})\n  \
             {accepted} = icmp eq i32 {status}, 1\n  \
             br i1 {accepted}, label %{offered_label}, label %{inline_label}\n\
             {inline_label}:\n  \
             {inline_result} = call {rendered_type} @{}({})\n  \
             store {rendered_type} {inline_result}, ptr {result_slot}\n  \
             br label %{offered_label}\n\
             {offered_label}:\n  \
             {submitted} = phi i1 [ true, %{request_label} ], [ false, %{inline_label} ]",
            self.value_name(directory),
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
                mapping: CompletionMapping::Open {
                    outcome: open_outcome,
                },
            }));
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
        let component = self.entry_slot(&format!("[{slot} x i8]"))?;
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
        let token = self.entry_slot("[2 x i64]")?;
        let result_slot = self.entry_slot(&llvm_type(self.program, ty)?)?;
        let raw_value = self.entry_slot("i64")?;
        let raw_error = self.entry_slot("i32")?;
        let position = self.entry_slot("i64")?;
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
             i64 {extent}, ptr {position}, ptr {token})\n  \
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
            self.value_name(*destination),
            self.value_name(*start),
            self.value_name(*source),
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
                mapping: CompletionMapping::DirectoryNext {
                    destination: format!("{destination_llvm} {}", self.value_name(*destination)),
                    start: self.value_name(*start),
                    extent,
                },
            }));
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
            CompletionMapping::Transfer { start, extent } => (
                format!(
                    "call void @wf__completion_file_join(ptr {token}, ptr {raw_value}, \
                         ptr {raw_error})"
                ),
                String::new(),
                format!("i64 {completed_value}, i32 {completed_error}, i64 {start}, i64 {extent}"),
            ),
            CompletionMapping::DirectoryNext {
                destination,
                start,
                extent,
            } => (
                format!(
                    "call void @wf__completion_file_join(ptr {token}, ptr {raw_value}, \
                     ptr {raw_error})"
                ),
                String::new(),
                format!(
                    "i64 {completed_value}, i32 {completed_error}, {destination}, i64 {start}, \
                     i64 {extent}"
                ),
            ),
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

fn completion_offered_label(value: IrValueId) -> String {
    format!("completion.offered.v{}", value.ordinal())
}

fn completion_join_inline_label(value: IrValueId) -> String {
    format!("completion.join.inline.v{}", value.ordinal())
}

fn completion_wait_label(value: IrValueId) -> String {
    format!("completion.wait.v{}", value.ordinal())
}
