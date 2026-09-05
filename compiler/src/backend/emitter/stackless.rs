//! Selective stackless lowering for one bounded continuation shape.
//!
//! The first executable slice deliberately admits only a single-block root
//! with one may-suspend direct call, whose callee chain consists entirely of
//! zero-state tail wrappers and ends in `read_at` or `write_once`. The root
//! saves exactly the operation arguments and values used after suspension;
//! wrappers forward the final continuation and therefore need no frame bytes.
//! Every other function keeps the existing synchronous ABI.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write;

use super::system::{CompletionFileOperation, completion_file_operation, completion_mapper_symbol};
use super::*;
use crate::IrFlatElement;
use crate::backend::target::{TargetLayout, validate_stackless_root_frame};

const WRITER_HEADER_BYTES: usize = 64;

pub(super) const STACKLESS_RUNTIME_FALLBACK: &str = "define weak i32 @wf__completion_file_pread_submit_writer(i32 %descriptor, ptr %buffer, i64 %count, i64 %offset, ptr %token, ptr %frame) {\nentry:\n  ret i32 0\n}\n\ndefine weak i32 @wf__completion_file_write_submit_writer(i32 %descriptor, ptr %buffer, i64 %count, ptr %token, ptr %frame) {\nentry:\n  ret i32 0\n}\n\ndefine weak void @wf__writer_frame_init(ptr %frame) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__writer_begin_suspend(ptr %frame, ptr %resume) {\nentry:\n  ret void\n}\n\ndefine weak i32 @wf__writer_commit_suspend(ptr %frame) {\nentry:\n  ret i32 0\n}\n\ndefine weak void @wf__writer_cancel_suspend(ptr %frame) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__writer_complete(ptr %frame) {\nentry:\n  ret void\n}\n\ndefine weak void @wf__writer_run_root(ptr %frame) {\nentry:\n  ret void\n}\n\n";

/// Hard Windows ABI for stackless writer continuations.
///
/// Windows completion is mandatory: a missing native bridge must fail at link
/// time instead of silently selecting the direct path through weak bodies.
pub(super) const STACKLESS_WINDOWS_RUNTIME_DECLARATIONS: &str = concat!(
    "declare i32 @wf__completion_file_pread_submit_writer(i32, ptr, i64, i64, ptr, ptr)\n",
    "declare i32 @wf__completion_file_write_submit_writer(i32, ptr, i64, ptr, ptr)\n",
    "declare i32 @wf__completion_file_take(ptr, ptr, ptr)\n",
    "declare void @wf__writer_frame_init(ptr)\n",
    "declare void @wf__writer_begin_suspend(ptr, ptr)\n",
    "declare i32 @wf__writer_commit_suspend(ptr)\n",
    "declare void @wf__writer_cancel_suspend(ptr)\n",
    "declare void @wf__writer_complete(ptr)\n",
    "declare void @wf__writer_run_root(ptr)\n",
);

#[derive(Clone, Debug)]
pub(super) struct StacklessPlan {
    root: u32,
    suspension_index: usize,
    suspension_result: IrValueId,
    outer: u32,
    stages: Vec<TailStage>,
    leaf_operation: CompletionFileOperation,
}

#[derive(Clone, Debug)]
struct TailStage {
    function: u32,
    kind: TailKind,
}

#[derive(Clone, Debug)]
enum TailKind {
    User {
        callee: u32,
        arguments: Vec<IrValueId>,
    },
    System {
        operation: crate::IrSystemOperation,
        arguments: Vec<IrValueId>,
    },
}

impl StacklessPlan {
    pub(super) fn build(
        program: &IrProgram<'_, '_, '_>,
        qualification: &Qualification,
    ) -> Option<Self> {
        if !qualification.target().supports_posix_file_completion() {
            return None;
        }
        let root = program.main_ordinal();
        let function = program.functions().get(root as usize)?;
        if !function.target_action().may_suspend()
            || function.blocks().len() != 1
            || !function.overlaps().is_empty()
        {
            return None;
        }
        let block = &function.blocks()[0];
        if !block.parameters().is_empty() {
            return None;
        }
        let mut selected = None;
        for (index, instruction) in block.instructions().iter().enumerate() {
            let IrInstruction::Define {
                result,
                operation:
                    IrOperation::Call {
                        function: callee, ..
                    },
                ..
            } = instruction
            else {
                if instruction_may_suspend(program, instruction) {
                    return None;
                }
                continue;
            };
            if program
                .functions()
                .get(*callee as usize)
                .is_some_and(|callee| callee.target_action().may_suspend())
            {
                if selected.is_some() {
                    return None;
                }
                selected = Some((index, *result, *callee));
            }
        }
        let (suspension_index, suspension_result, outer) = selected?;
        // The ordinary function frame is stack storage owned by the start
        // activation. A slice, stored address, or arena list formed from one
        // of its slots may retain a pointer to that storage after the start
        // activation returns on the suspended path. Until those backing slots
        // are planned as fields of the persistent root frame, this shape must
        // keep the synchronous ABI.
        if block.instructions()[..suspension_index]
            .iter()
            .any(instruction_materializes_stack_bound_referent)
        {
            return None;
        }
        if block.instructions()[suspension_index + 1..]
            .iter()
            .any(|instruction| !supported_after_instruction(program, instruction))
        {
            return None;
        }
        let IrTerminator::Return { drops, .. } = block.terminator() else {
            return None;
        };
        if drops
            .iter()
            .any(|drop| drop.release().row.target_action.may_suspend())
        {
            return None;
        }

        let mut stages = Vec::new();
        let mut visiting = HashSet::new();
        let (leaf_operation, _) = collect_tail_chain(program, outer, &mut visiting, &mut stages)?;
        Some(Self {
            root,
            suspension_index,
            suspension_result,
            outer,
            stages,
            leaf_operation,
        })
    }

    pub(super) fn root_ordinal(&self) -> u32 {
        self.root
    }

    pub(super) fn emit_tail_definitions(
        &self,
        program: &IrProgram<'_, '_, '_>,
        qualification: &Qualification,
    ) -> Result<String, BackendFailure> {
        let mut output = String::new();
        for stage in self.stages.iter().rev() {
            emit_tail_stage(&mut output, program, qualification, stage)?;
        }
        Ok(output)
    }
}

fn instruction_materializes_stack_bound_referent(instruction: &IrInstruction) -> bool {
    matches!(
        instruction,
        IrInstruction::Define {
            operation: IrOperation::SliceFromArray {
                array: IrArrayRoot::Value(_),
            } | IrOperation::AddressOf { .. }
                | IrOperation::ArenaListNew,
            ..
        }
    )
}

fn instruction_may_suspend(program: &IrProgram<'_, '_, '_>, instruction: &IrInstruction) -> bool {
    match instruction {
        IrInstruction::Define {
            operation: IrOperation::Call { function, .. },
            ..
        } => program
            .functions()
            .get(*function as usize)
            .is_none_or(|callee| callee.target_action().may_suspend()),
        IrInstruction::Define {
            operation: IrOperation::SystemCall { target_action, .. },
            ..
        } => target_action.may_suspend(),
        IrInstruction::Drop(drop) => drop.release().row.target_action.may_suspend(),
        _ => false,
    }
}

fn supported_after_instruction(
    program: &IrProgram<'_, '_, '_>,
    instruction: &IrInstruction,
) -> bool {
    match instruction {
        IrInstruction::Define {
            operation: IrOperation::Constant(_),
            ..
        } => true,
        IrInstruction::Define {
            operation: IrOperation::SystemCall { target_action, .. },
            ..
        } => !target_action.may_suspend(),
        IrInstruction::Define {
            operation: IrOperation::Call { function, .. },
            ..
        } => program
            .functions()
            .get(*function as usize)
            .is_some_and(|callee| !callee.target_action().may_suspend()),
        IrInstruction::Drop(drop) => !drop.release().row.target_action.may_suspend(),
        _ => false,
    }
}

fn collect_tail_chain(
    program: &IrProgram<'_, '_, '_>,
    ordinal: u32,
    visiting: &mut HashSet<u32>,
    stages: &mut Vec<TailStage>,
) -> Option<(CompletionFileOperation, crate::IrSystemOperation)> {
    if !visiting.insert(ordinal) {
        return None;
    }
    let function = program.functions().get(ordinal as usize)?;
    let [block] = function.blocks() else {
        return None;
    };
    if !block.parameters().is_empty() || block.instructions().len() != 1 {
        return None;
    }
    let IrInstruction::Define {
        result,
        ty,
        operation,
    } = &block.instructions()[0]
    else {
        return None;
    };
    let IrTerminator::Return { value, drops } = block.terminator() else {
        return None;
    };
    if value != result || !drops.is_empty() || *ty != function.result() {
        return None;
    }
    match operation {
        IrOperation::Call {
            function: callee,
            arguments,
        } => {
            if arguments.iter().any(|argument| {
                !function
                    .parameters()
                    .iter()
                    .any(|(parameter, _)| parameter == argument)
            }) {
                return None;
            }
            stages.push(TailStage {
                function: ordinal,
                kind: TailKind::User {
                    callee: *callee,
                    arguments: arguments.clone(),
                },
            });
            collect_tail_chain(program, *callee, visiting, stages)
        }
        IrOperation::SystemCall {
            operation,
            target_action,
            arguments,
        } => {
            if !target_action.may_suspend()
                || arguments.iter().any(|argument| {
                    !function
                        .parameters()
                        .iter()
                        .any(|(parameter, _)| parameter == argument)
                })
            {
                return None;
            }
            let completion = completion_file_operation(*operation)?;
            if !matches!(
                completion,
                CompletionFileOperation::Read | CompletionFileOperation::Write
            ) {
                return None;
            }
            stages.push(TailStage {
                function: ordinal,
                kind: TailKind::System {
                    operation: *operation,
                    arguments: arguments.clone(),
                },
            });
            Some((completion, *operation))
        }
        _ => None,
    }
}

fn stackless_start_symbol(ordinal: u32) -> String {
    format!("wf__stackless_start_{ordinal}")
}

fn emit_tail_stage(
    output: &mut String,
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    stage: &TailStage,
) -> Result<(), BackendFailure> {
    let function = program
        .functions()
        .get(stage.function as usize)
        .ok_or(BackendFailure::InvalidIr)?;
    let symbol = stackless_start_symbol(stage.function);
    write!(
        output,
        "define internal i1 @{symbol}(ptr %continuation, ptr %result, ptr %token, ptr %raw_value, ptr %raw_error, ptr %start_slot, ptr %extent_slot"
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    for (value, ty) in function.parameters() {
        write!(
            output,
            ", {} {}",
            llvm_type(program, *ty)?,
            value_name(*value)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    output.push_str(") {\nentry:\n");
    match &stage.kind {
        TailKind::User { callee, arguments } => {
            let callee_function = program
                .functions()
                .get(*callee as usize)
                .ok_or(BackendFailure::InvalidIr)?;
            let rendered = render_arguments(program, function, callee_function, arguments)?;
            writeln!(
                output,
                "  %pending = call i1 @{}(ptr %continuation, ptr %result, ptr %token, ptr %raw_value, ptr %raw_error, ptr %start_slot, ptr %extent_slot, {})\n  ret i1 %pending\n}}\n",
                stackless_start_symbol(*callee),
                rendered.join(", ")
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        TailKind::System {
            operation,
            arguments,
        } => emit_system_tail(
            output,
            program,
            qualification,
            function,
            *operation,
            arguments,
        )?,
    }
    Ok(())
}

fn render_arguments(
    program: &IrProgram<'_, '_, '_>,
    caller: &IrFunction,
    callee: &IrFunction,
    arguments: &[IrValueId],
) -> Result<Vec<String>, BackendFailure> {
    if callee.parameters().len() != arguments.len() {
        return Err(BackendFailure::InvalidIr);
    }
    arguments
        .iter()
        .zip(callee.parameters())
        .map(|(argument, (_, expected))| {
            if caller.value_type(*argument) != Some(*expected) {
                return Err(BackendFailure::InvalidIr);
            }
            Ok(format!(
                "{} {}",
                llvm_type(program, *expected)?,
                value_name(*argument)
            ))
        })
        .collect()
}

fn emit_system_tail(
    output: &mut String,
    program: &IrProgram<'_, '_, '_>,
    qualification: &Qualification,
    function: &IrFunction,
    operation: crate::IrSystemOperation,
    arguments: &[IrValueId],
) -> Result<(), BackendFailure> {
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
    let buffer_ty = function
        .value_type(*buffer)
        .ok_or(BackendFailure::InvalidIr)?;
    let buffer_llvm = llvm_type(program, buffer_ty)?;
    let result_llvm = llvm_type(program, function.result())?;
    let implementation = qualification.operation(operation)?;
    let rendered_arguments = arguments
        .iter()
        .map(|argument| {
            let ty = function
                .value_type(*argument)
                .ok_or(BackendFailure::InvalidIr)?;
            Ok(format!(
                "{} {}",
                llvm_type(program, ty)?,
                value_name(*argument)
            ))
        })
        .collect::<Result<Vec<_>, BackendFailure>>()?;
    let submit = match (completion, file_offset) {
        (CompletionFileOperation::Read, Some(_)) => "wf__completion_file_pread_submit_writer",
        (CompletionFileOperation::Write, None) => "wf__completion_file_write_submit_writer",
        _ => return Err(BackendFailure::InvalidIr),
    };
    let submit_args = if let Some(offset) = file_offset {
        format!(
            "i32 {}, ptr %target, i64 %extent, i64 {}, ptr %token, ptr %continuation",
            value_name(*resource),
            value_name(*offset)
        )
    } else {
        format!(
            "i32 {}, ptr %target, i64 %extent, ptr %token, ptr %continuation",
            value_name(*resource)
        )
    };
    let (eligibility, ineligible) = if let Some(offset) = file_offset {
        (
            format!(
                "  %offset_too_large = icmp ugt i64 {}, 9223372036854775807\n  \
                 %ineligible = or i1 %vacant, %offset_too_large\n",
                value_name(*offset)
            ),
            "%ineligible",
        )
    } else {
        (String::new(), "%vacant")
    };
    let verdict = if qualification.target().is_windows() {
        "  %direct_only = icmp eq i32 %status, 0\n  \
         br i1 %direct_only, label %inline, label %submit.accepted\n\
         submit.accepted:\n  %accepted = icmp eq i32 %status, 1\n  \
         br i1 %accepted, label %suspended, label %submit.capacity\n\
         submit.capacity:\n  %capacity = icmp eq i32 %status, 2\n  \
         br i1 %capacity, label %capacity_wait, label %invalid_submit\n\
         capacity_wait:\n  call void @wf__completion_wait_core_capacity()\n  br label %submit\n\
         invalid_submit:\n  call void @abort()\n  unreachable\n"
    } else {
        "  %accepted = icmp eq i32 %status, 1\n  br i1 %accepted, label %suspended, label %inline\n"
    };
    writeln!(
        output,
        "  %extent = sub i64 {}, {}\n  store i64 {}, ptr %start_slot\n  store i64 %extent, ptr %extent_slot\n  %vacant = icmp eq i64 %extent, 0\n{eligibility}  br i1 {ineligible}, label %inline, label %submit\nsubmit:\n  %base = extractvalue {buffer_llvm} {}, 0\n  %target = getelementptr inbounds i8, ptr %base, i64 {}\n  %status = call i32 @{submit}({submit_args})\n{verdict}inline:\n  %direct = call {result_llvm} @{}({})\n  store {result_llvm} %direct, ptr %result\n  ret i1 false\nsuspended:\n  ret i1 true\n}}\n",
        value_name(*end),
        value_name(*start),
        value_name(*start),
        value_name(*buffer),
        value_name(*start),
        implementation.symbol(),
        rendered_arguments.join(", ")
    )
    .map_err(|_| BackendFailure::TextEmission)
}

fn collect_root_live_values(block: &IrBlock, suspension_index: usize) -> BTreeSet<IrValueId> {
    let mut live = BTreeSet::new();
    if let IrInstruction::Define {
        operation: IrOperation::Call { arguments, .. },
        ..
    } = &block.instructions()[suspension_index]
    {
        live.extend(arguments.iter().copied());
    }
    for instruction in &block.instructions()[suspension_index + 1..] {
        match instruction {
            IrInstruction::Define {
                operation:
                    IrOperation::Call { arguments, .. } | IrOperation::SystemCall { arguments, .. },
                ..
            } => live.extend(arguments.iter().copied()),
            IrInstruction::Drop(drop) => {
                live.insert(drop.value());
            }
            _ => {}
        }
    }
    if let IrTerminator::Return { value, drops } = block.terminator() {
        live.insert(*value);
        live.extend(drops.iter().map(|drop| drop.value()));
    }
    let suspension_result = match &block.instructions()[suspension_index] {
        IrInstruction::Define { result, .. } => *result,
        _ => return BTreeSet::new(),
    };
    live.retain(|value| value.ordinal() < suspension_result.ordinal());
    live
}

#[derive(Debug)]
pub(super) struct RootFrame {
    ty: String,
    live_fields: BTreeMap<IrValueId, usize>,
    async_result: usize,
    final_result: usize,
    alignment: u64,
    #[cfg(test)]
    size: u64,
}

impl RootFrame {
    fn build(
        program: &IrProgram<'_, '_, '_>,
        qualification: &Qualification,
        function: &IrFunction,
        block: &IrBlock,
        suspension_index: usize,
        target: TargetLayout,
    ) -> Result<Self, BackendFailure> {
        let live = collect_root_live_values(block, suspension_index);
        let mut fields = vec![
            IrType::Array {
                element: IrFlatElement::Integer {
                    width: 8,
                    signed: false,
                },
                length: u64::try_from(WRITER_HEADER_BYTES)
                    .map_err(|_| BackendFailure::InvalidIr)?,
            },
            IrType::Array {
                element: IrFlatElement::Integer {
                    width: 64,
                    signed: false,
                },
                length: 2,
            },
            IrType::Integer {
                width: 64,
                signed: false,
            },
            IrType::Integer {
                width: 32,
                signed: false,
            },
            IrType::Integer {
                width: 64,
                signed: false,
            },
            IrType::Integer {
                width: 64,
                signed: false,
            },
            IrType::Bool,
            function
                .value_type(match &block.instructions()[suspension_index] {
                    IrInstruction::Define { result, .. } => *result,
                    _ => return Err(BackendFailure::InvalidIr),
                })
                .ok_or(BackendFailure::InvalidIr)?,
            function.result(),
        ];
        let async_result = 7;
        let final_result = 8;
        let mut live_fields = BTreeMap::new();
        for value in live {
            let ty = function
                .value_type(value)
                .ok_or(BackendFailure::InvalidIr)?;
            let index = fields.len();
            fields.push(ty);
            live_fields.insert(value, index);
        }
        let layout = validate_stackless_root_frame(target, qualification, program, &fields)
            .map_err(BackendFailure::TargetLayout)?;
        let mut rendered_fields = Vec::with_capacity(fields.len());
        for field in fields {
            rendered_fields.push(llvm_type(program, field)?);
        }
        Ok(Self {
            ty: format!("{{ {} }}", rendered_fields.join(", ")),
            live_fields,
            async_result,
            final_result,
            alignment: layout.align(),
            #[cfg(test)]
            size: layout.size(),
        })
    }
}

impl FunctionEmitter<'_, '_> {
    pub(super) fn emit_stackless_root(
        mut self,
        plan: &StacklessPlan,
    ) -> Result<String, BackendFailure> {
        let [block] = self.function.blocks() else {
            return Err(BackendFailure::InvalidIr);
        };
        let target = TargetLayout::host().map_err(BackendFailure::TargetLayout)?;
        let frame = RootFrame::build(
            self.program,
            self.qualification,
            self.function,
            block,
            plan.suspension_index,
            target,
        )?;
        let result_llvm = llvm_type(self.program, self.function.result())?;
        let source = source_symbol(self.function.name());
        let start_symbol = format!("wf__stackless_root_start_{}", plan.root);
        let resume_symbol = format!("wf__stackless_root_resume_{}", plan.root);

        write!(self.output, "define internal {result_llvm} @{source}(")
            .map_err(|_| BackendFailure::TextEmission)?;
        for (index, (value, ty)) in self.function.parameters().iter().enumerate() {
            if index != 0 {
                self.output.push_str(", ");
            }
            write!(
                self.output,
                "{} {}",
                llvm_type(self.program, *ty)?,
                value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        write!(
            self.output,
            ") {{\nentry:\n  %frame = alloca {}, align {}\n  call void @wf__writer_frame_init(ptr %frame)\n  %pending = call i1 @{start_symbol}(ptr %frame",
            frame.ty,
            frame.alignment
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        for (value, ty) in self.function.parameters() {
            write!(
                self.output,
                ", {} {}",
                llvm_type(self.program, *ty)?,
                value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        write!(
            self.output,
            ")\n  call void @wf__writer_run_root(ptr %frame)\n  %finalp = getelementptr inbounds {}, ptr %frame, i32 0, i32 {}\n  %final = load {result_llvm}, ptr %finalp\n  ret {result_llvm} %final\n}}\n\n",
            frame.ty,
            frame.final_result
        )
        .map_err(|_| BackendFailure::TextEmission)?;

        self.emit_stackless_start(plan, block, &frame, &start_symbol, &resume_symbol)?;
        self.emit_stackless_resume(plan, block, &frame, &resume_symbol)?;
        *self.completion_used = true;
        self.parallel.request_stackless_runtime();
        Ok(self.output)
    }

    fn emit_stackless_start(
        &mut self,
        plan: &StacklessPlan,
        block: &IrBlock,
        frame: &RootFrame,
        start_symbol: &str,
        resume_symbol: &str,
    ) -> Result<(), BackendFailure> {
        write!(self.output, "define internal i1 @{start_symbol}(ptr %frame")
            .map_err(|_| BackendFailure::TextEmission)?;
        for (value, ty) in self.function.parameters() {
            write!(
                self.output,
                ", {} {}",
                llvm_type(self.program, *ty)?,
                value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        self.output.push_str(") {\nentry:\n");
        let prelude_anchor = self.output.len();
        for (index, instruction) in block.instructions()[..plan.suspension_index]
            .iter()
            .enumerate()
        {
            self.emit_instruction(
                IrBlockId::from_index(0).map_err(|_| BackendFailure::InvalidIr)?,
                index,
                instruction,
            )?;
        }
        for (value, field) in &frame.live_fields {
            let ptr = self.next_temporary()?;
            let ty = self.value_type(*value).ok_or(BackendFailure::InvalidIr)?;
            writeln!(
                self.output,
                "  %{ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 {field}\n  store {} {}, ptr %{ptr}",
                frame.ty,
                llvm_type(self.program, ty)?,
                value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        let token = self.next_temporary()?;
        let raw_value = self.next_temporary()?;
        let raw_error = self.next_temporary()?;
        let start = self.next_temporary()?;
        let extent = self.next_temporary()?;
        let async_result = self.next_temporary()?;
        let submitted = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{token} = getelementptr inbounds {}, ptr %frame, i32 0, i32 1\n  %{raw_value} = getelementptr inbounds {}, ptr %frame, i32 0, i32 2\n  %{raw_error} = getelementptr inbounds {}, ptr %frame, i32 0, i32 3\n  %{start} = getelementptr inbounds {}, ptr %frame, i32 0, i32 4\n  %{extent} = getelementptr inbounds {}, ptr %frame, i32 0, i32 5\n  %{async_result} = getelementptr inbounds {}, ptr %frame, i32 0, i32 {}\n  call void @wf__writer_begin_suspend(ptr %frame, ptr @{resume_symbol})",
            frame.ty,
            frame.ty,
            frame.ty,
            frame.ty,
            frame.ty,
            frame.ty,
            frame.async_result
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let IrInstruction::Define {
            operation: IrOperation::Call { arguments, .. },
            ..
        } = &block.instructions()[plan.suspension_index]
        else {
            return Err(BackendFailure::InvalidIr);
        };
        let outer = self
            .program
            .functions()
            .get(plan.outer as usize)
            .ok_or(BackendFailure::InvalidIr)?;
        let rendered = render_arguments(self.program, self.function, outer, arguments)?;
        writeln!(
            self.output,
            "  %pending = call i1 @{}(ptr %frame, ptr %{async_result}, ptr %{token}, ptr %{raw_value}, ptr %{raw_error}, ptr %{start}, ptr %{extent}, {})\n  %{submitted} = getelementptr inbounds {}, ptr %frame, i32 0, i32 6\n  store i1 %pending, ptr %{submitted}\n  br i1 %pending, label %suspend, label %inline\ninline:\n  call void @wf__writer_cancel_suspend(ptr %frame)\n  call void @{resume_symbol}(ptr %frame)\n  ret i1 false\nsuspend:\n  %committed = call i32 @wf__writer_commit_suspend(ptr %frame)\n  ret i1 true\n}}\n\n",
            stackless_start_symbol(plan.outer),
            rendered.join(", "),
            frame.ty
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        if !self.entry_prelude.is_empty() {
            self.output.insert_str(prelude_anchor, &self.entry_prelude);
            self.entry_prelude.clear();
        }
        Ok(())
    }

    fn emit_stackless_resume(
        &mut self,
        plan: &StacklessPlan,
        block: &IrBlock,
        frame: &RootFrame,
        resume_symbol: &str,
    ) -> Result<(), BackendFailure> {
        writeln!(
            self.output,
            "define internal void @{resume_symbol}(ptr %frame) {{\nentry:"
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        for (value, field) in &frame.live_fields {
            let ptr = self.next_temporary()?;
            let ty = self.value_type(*value).ok_or(BackendFailure::InvalidIr)?;
            writeln!(
                self.output,
                "  %{ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 {field}\n  {} = load {}, ptr %{ptr}",
                frame.ty,
                value_name(*value),
                llvm_type(self.program, ty)?
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        let submitted_ptr = self.next_temporary()?;
        let submitted = self.next_temporary()?;
        let async_ptr = self.next_temporary()?;
        let result_llvm = llvm_type(
            self.program,
            self.value_type(plan.suspension_result)
                .ok_or(BackendFailure::InvalidIr)?,
        )?;
        writeln!(
            self.output,
            "  %{submitted_ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 6\n  %{submitted} = load i1, ptr %{submitted_ptr}\n  %{async_ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 {}\n  br i1 %{submitted}, label %async, label %inline\ninline:\n  %inline_result = load {result_llvm}, ptr %{async_ptr}\n  br label %result_ready\nasync:",
            frame.ty,
            frame.ty,
            frame.async_result
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let token = self.next_temporary()?;
        let raw_value_ptr = self.next_temporary()?;
        let raw_error_ptr = self.next_temporary()?;
        let raw_value = self.next_temporary()?;
        let raw_error = self.next_temporary()?;
        let start_ptr = self.next_temporary()?;
        let extent_ptr = self.next_temporary()?;
        let start = self.next_temporary()?;
        let extent = self.next_temporary()?;
        writeln!(
            self.output,
            "  %{token} = getelementptr inbounds {}, ptr %frame, i32 0, i32 1\n  %{raw_value_ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 2\n  %{raw_error_ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 3\n  call void @wf__completion_file_join(ptr %{token}, ptr %{raw_value_ptr}, ptr %{raw_error_ptr})\n  %{raw_value} = load i64, ptr %{raw_value_ptr}\n  %{raw_error} = load i32, ptr %{raw_error_ptr}\n  %{start_ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 4\n  %{extent_ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 5\n  %{start} = load i64, ptr %{start_ptr}\n  %{extent} = load i64, ptr %{extent_ptr}\n  %mapped_result = call {result_llvm} @{}(i64 %{raw_value}, i32 %{raw_error}, i64 %{start}, i64 %{extent})\n  br label %result_ready\nresult_ready:\n  {} = phi {result_llvm} [ %inline_result, %inline ], [ %mapped_result, %async ]",
            frame.ty,
            frame.ty,
            frame.ty,
            frame.ty,
            frame.ty,
            completion_mapper_symbol(plan.leaf_operation),
            value_name(plan.suspension_result)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        let block_id = IrBlockId::from_index(0).map_err(|_| BackendFailure::InvalidIr)?;
        for (offset, instruction) in block.instructions()[plan.suspension_index + 1..]
            .iter()
            .enumerate()
        {
            self.emit_instruction(block_id, plan.suspension_index + 1 + offset, instruction)?;
        }
        let IrTerminator::Return { value, drops } = block.terminator() else {
            return Err(BackendFailure::InvalidIr);
        };
        self.emit_drops(drops)?;
        let final_ptr = self.next_temporary()?;
        let final_ty = llvm_type(self.program, self.function.result())?;
        writeln!(
            self.output,
            "  %{final_ptr} = getelementptr inbounds {}, ptr %frame, i32 0, i32 {}\n  store {final_ty} {}, ptr %{final_ptr}\n  call void @wf__writer_complete(ptr %frame)\n  ret void\n}}\n\n",
            frame.ty,
            frame.final_result,
            value_name(*value)
        )
        .map_err(|_| BackendFailure::TextEmission)
    }
}

#[cfg(test)]
mod root_frame_layout_tests {
    use super::*;
    use crate::backend::qualification::{SystemTarget, qualify_program};
    use crate::lexer::{LexOutcome, lex};
    use crate::{
        ACTIVE_KERNEL_SPEC_HASH, CanonicalOutcome, CompilerLimits, FinalizeOutcome, IrProgram,
        OverlapLowering, ParseOutcome, ResolutionOutcome, SemanticOutcome, SourceBundle,
        SourceInput, TerminalOutcome, audit_canonical, check_semantics, classify_terminals,
        finalize, lower_checked, parse,
    };

    const ROOT_FRAME_SOURCE: &[u8] = br#"fn publish(output: &uniq Output, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output) contract {
  define ordered = start <= end;
  define capacity = len(deref(source));
  requires ordered;
  requires end <= capacity;
} {
  return write_once(output: move output, source: source, start: start, end: end);
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region {
    let outcome = publish(output: &uniq out, source: &bytes, start: 0_u64, end: 1_u64);
  }
  return exit_status(code: 0_u8);
}
"#;

    fn with_root_frame(
        run: impl for<'classified, 'lexed, 'source> FnOnce(
            &IrProgram<'classified, 'lexed, 'source>,
            &Qualification,
            &IrFunction,
            &IrBlock,
            usize,
            TargetLayout,
        ),
    ) {
        let limits = CompilerLimits::default();
        let inputs = [SourceInput::new("root-frame-layout.wf", ROOT_FRAME_SOURCE)];
        let bundle = SourceBundle::with_limits(&inputs, limits.source).expect("valid test bundle");
        let LexOutcome::Complete(lexed) = lex(&bundle, limits.lexer) else {
            panic!("root-frame source must lex");
        };
        let TerminalOutcome::Complete(classified) =
            classify_terminals(&lexed, ACTIVE_KERNEL_SPEC_HASH, limits.terminals)
        else {
            panic!("root-frame source must classify");
        };
        let ParseOutcome::Complete(parsed) = parse(&classified, limits.parser) else {
            panic!("root-frame source must parse");
        };
        let FinalizeOutcome::Complete(finalized) = finalize(parsed, limits.finalizer) else {
            panic!("root-frame source must finalize");
        };
        let CanonicalOutcome::Complete(canonical) = audit_canonical(finalized, limits.canonical)
        else {
            panic!("root-frame source must be canonical");
        };
        let ResolutionOutcome::Complete(resolved) = crate::resolve(canonical) else {
            panic!("root-frame source must resolve");
        };
        let SemanticOutcome::Complete(checked) = check_semantics(resolved) else {
            panic!("root-frame source must check");
        };
        let program = lower_checked(*checked, OverlapLowering::Completion)
            .expect("checked root-frame source must lower");
        let target = TargetLayout::host().expect("root-frame test requires a supported host");
        let system_target = SystemTarget::for_triple(target.triple())
            .expect("the supported host must have a qualification row");
        let qualification =
            qualify_program(system_target, &program).expect("root-frame source must qualify");
        let plan = StacklessPlan::build(&program, &qualification)
            .expect("root-frame source must select stackless lowering");
        let function = program
            .functions()
            .get(plan.root as usize)
            .expect("the stackless root ordinal must identify a function");
        let [block] = function.blocks() else {
            panic!("the selected stackless root must have one block");
        };
        run(
            &program,
            &qualification,
            function,
            block,
            plan.suspension_index,
            target,
        );
    }

    #[test]
    fn complete_root_frame_accepts_the_exact_selected_target_boundary() {
        with_root_frame(
            |program, qualification, function, block, suspension_index, target| {
                let host_frame = RootFrame::build(
                    program,
                    qualification,
                    function,
                    block,
                    suspension_index,
                    target,
                )
                .expect("the host target must represent the root frame");
                let exact = target.with_address_index_max_for_test(host_frame.size);
                let exact_frame = RootFrame::build(
                    program,
                    qualification,
                    function,
                    block,
                    suspension_index,
                    exact,
                )
                .expect("a target domain equal to the complete frame size must admit it");
                assert_eq!(exact_frame.size, host_frame.size);
                assert_eq!(exact_frame.alignment, 8);
                assert!(exact_frame.ty.starts_with("{ [64 x i8], [2 x i64]"));
            },
        );
    }

    #[test]
    fn complete_root_frame_rejects_a_domain_one_byte_below_its_padded_size() {
        with_root_frame(
            |program, qualification, function, block, suspension_index, target| {
                let host_frame = RootFrame::build(
                    program,
                    qualification,
                    function,
                    block,
                    suspension_index,
                    target,
                )
                .expect("the host target must represent the root frame");
                assert!(host_frame.size > WRITER_HEADER_BYTES as u64 + 1);
                let short = target.with_address_index_max_for_test(host_frame.size - 1);
                let failure = RootFrame::build(
                    program,
                    qualification,
                    function,
                    block,
                    suspension_index,
                    short,
                )
                .expect_err("the complete padded frame exceeds this target domain");
                assert_eq!(
                    failure,
                    BackendFailure::TargetLayout(TargetLayoutFailure::Unrepresentable(
                        crate::backend::target::TargetObject::StackFrame,
                    ))
                );
            },
        );
    }
}
