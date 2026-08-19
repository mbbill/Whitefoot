//! Conservative textual LLVM emission for the active Whitefoot specification.
//!
//! Emission consumes only target-independent IR. It preserves every retained
//! check, emits no overflow or alias promises, initializes complete aggregate
//! representations, and keeps a defensive abort edge for enum discriminants.

mod arena;
mod array;
mod boxes;
mod buffer;
mod cleanup;
mod conversion;
mod floating;
mod integer;
mod operations;
mod reinterpret;
mod slice;
mod system;

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;

use super::qualification::{
    Qualification, QualificationFailure, SystemTarget, qualified_representation, qualify_program,
};
use super::target::{TargetLayout, TargetLayoutFailure, validate_program};
use crate::{
    IrAddressed, IrArrayRoot, IrBlock, IrBlockId, IrBooleanOperation, IrConstant, IrDrop, IrEntry,
    IrEntryGoal, IrEnumType, IrFloatOperation, IrFunction, IrGlobalValue, IrInstruction,
    IrIntegerOperation, IrNominal, IrNominalId, IrNominalKind, IrOperation, IrProgram,
    IrRuntimeTargetObligations, IrTargetDomainObligation, IrTerminator, IrTrapSite, IrType,
    IrValueId, SystemResourceType,
};
use buffer::{buffer_fill_done_label, buffer_probe_join_label, buffer_vacant_done_label};
use cleanup::{emit_resource_drop_helpers, emit_value_cleanup, type_requires_cleanup};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendFailure {
    TargetLayout(TargetLayoutFailure),
    /// The [QUAL-1] target-qualification table has no approved implementation
    /// for a facility the program uses on the selected target and program
    /// kind, or a required [QUAL-2] target guarantee is unmet. Like a
    /// target-layout failure this is not a source-language rejection and cites
    /// no language rule [DIAG-1].
    TargetQualification(QualificationFailure),
    InvalidIr,
    CounterOverflow,
    TextEmission,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LlvmModule {
    text: String,
}

impl LlvmModule {
    #[must_use]
    pub fn into_string(self) -> String {
        self.text
    }
}

pub fn emit_llvm(program: &IrProgram<'_, '_, '_>) -> Result<LlvmModule, BackendFailure> {
    let target = TargetLayout::host().map_err(BackendFailure::TargetLayout)?;
    // [QUAL-1] consults the qualification table after the exact target and ABI
    // are selected and before emitting any use of an operation. It runs before
    // layout because an opaque resource has no target representation until its
    // qualification record fixes one.
    let system_target = SystemTarget::for_triple(target.triple()).ok_or(
        BackendFailure::TargetLayout(TargetLayoutFailure::UnsupportedHost),
    )?;
    emit_llvm_for(program, system_target)
}

/// Emits one program against an explicitly selected system target.
///
/// `emit_llvm` selects the host's native target; a test harness that has
/// selected the deterministic test target calls this instead. Selection stays
/// one decision taken before qualification either way [QUAL-1].
#[cfg(test)]
pub(crate) fn emit_llvm_for_target(
    program: &IrProgram<'_, '_, '_>,
    system_target: SystemTarget,
) -> Result<LlvmModule, BackendFailure> {
    emit_llvm_for(program, system_target)
}

fn emit_llvm_for(
    program: &IrProgram<'_, '_, '_>,
    system_target: SystemTarget,
) -> Result<LlvmModule, BackendFailure> {
    let target = TargetLayout::host().map_err(BackendFailure::TargetLayout)?;
    let qualification = qualify_program(system_target, program)?;
    validate_program(target, &qualification, program).map_err(BackendFailure::TargetLayout)?;
    let main = program
        .functions()
        .get(program.main_ordinal() as usize)
        .ok_or(BackendFailure::InvalidIr)?;
    let system = system::emit_system_interface(program, &qualification)?;

    let mut traps = Vec::new();
    let mut intrinsics = BTreeSet::new();
    let mut functions = String::new();
    for function in program.functions() {
        functions.push_str(
            &FunctionEmitter::new(
                program,
                &qualification,
                function,
                target,
                &mut traps,
                &mut intrinsics,
            )
            .emit()?,
        );
    }
    // Render the compiler-owned wrapper before declarations are written: an
    // entry-only goal may be the sole user of one trap record or intrinsic.
    let entry = system::emit_entry(
        program,
        &qualification,
        main,
        target,
        &mut traps,
        &mut intrinsics,
    )?;
    let has_matches = program.functions().iter().any(|function| {
        function
            .blocks()
            .iter()
            .any(|block| matches!(block.terminator(), IrTerminator::Match { .. }))
    });
    let drop_helpers = emit_resource_drop_helpers(program, &qualification)?;
    let has_arena_storage = program
        .nominals()
        .iter()
        .any(|nominal| matches!(nominal.kind(), IrNominalKind::ArenaStorage));
    let has_heap_storage = !drop_helpers.is_empty()
        || has_arena_storage
        || program.functions().iter().any(IrFunction::contains_buffer)
        || program.nominals().iter().any(|nominal| {
            matches!(
                nominal.kind(),
                IrNominalKind::Box { .. } | IrNominalKind::Arena { .. }
            )
        });

    let mut text = format!(
        "; Whitefoot conservative module\nsource_filename = \"whitefoot\"\ntarget datalayout = \"{}\"\ntarget triple = \"{}\"\n\n",
        target.data_layout(),
        target.triple(),
    );
    emit_nominal_declarations(&mut text, program)?;
    emit_global_constants(&mut text, program)?;
    text.push_str(&system.constants);
    for (index, bytes) in traps.iter().enumerate() {
        writeln!(
            text,
            "@.wf_trap.{index} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            bytes.len(),
            llvm_bytes(bytes)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    // The mandatory [DIAG-3] record and the qualified system interface can
    // need the same host symbol; one module declares it once.
    let mut system_declarations = system.declarations;
    if !traps.is_empty() {
        text.push('\n');
        text.push_str("declare i64 @write(i32, ptr, i64)\n");
        system_declarations.remove("declare i64 @write(i32, ptr, i64)");
    }
    if !traps.is_empty() || has_matches || has_heap_storage {
        text.push_str("declare void @abort() noreturn\n");
        system_declarations.remove("declare void @abort() noreturn");
    }
    if has_heap_storage {
        text.push_str("declare ptr @malloc(i64)\ndeclare void @free(ptr)\n");
    }
    for declaration in &system_declarations {
        text.push_str(declaration);
        text.push('\n');
    }
    if !traps.is_empty() {
        text.push_str(
            "\ndefine private void @wf_trap(ptr %message, i64 %length) noreturn {\nentry:\n  br label %write.loop\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @write(i32 2, ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\n}\n\n",
        );
    } else if has_matches {
        text.push('\n');
    }
    if has_arena_storage {
        text.push('\n');
        text.push_str(arena::ARENA_RELEASE_HELPER);
    }
    text.push_str(&drop_helpers);
    text.push_str(&system.definitions);
    for intrinsic in intrinsics {
        match intrinsic {
            IntrinsicDeclaration::Overflow { name, ty } => {
                writeln!(text, "declare {{ {ty}, i1 }} @{name}({ty}, {ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::UnaryWithFlag { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty}, i1)")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::Unary { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::Binary { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty}, {ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::Ternary { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty}, {ty}, {ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::UnaryCast {
                name,
                result_ty,
                argument_ty,
            } => writeln!(text, "declare {result_ty} @{name}({argument_ty})")
                .map_err(|_| BackendFailure::TextEmission)?,
        }
    }
    if !functions.is_empty() {
        text.push('\n');
        text.push_str(&functions);
    }
    text.push_str(&entry);
    Ok(LlvmModule { text })
}

fn emit_global_constants(
    output: &mut String,
    program: &IrProgram<'_, '_, '_>,
) -> Result<(), BackendFailure> {
    for constant in program.constants() {
        writeln!(output, "; const {}", constant.name())
            .map_err(|_| BackendFailure::TextEmission)?;
        write!(
            output,
            "{} = private unnamed_addr constant {} {}",
            constant_symbol(constant.id()),
            llvm_type(program, constant.ty())?,
            global_constant_value(program, constant.value(), constant.ty())?
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        output.push('\n');
    }
    if !program.constants().is_empty() {
        output.push('\n');
    }
    Ok(())
}

/// Renders one rodata constant value of one exact type: a scalar operand, a
/// complete array, or a complete struct aggregate with each field rendered
/// recursively [CONST-2 candidate].
fn global_constant_value(
    program: &IrProgram<'_, '_, '_>,
    value: &IrGlobalValue,
    ty: IrType,
) -> Result<String, BackendFailure> {
    match (value, ty) {
        (IrGlobalValue::Scalar(value), ty) => constant_operand(*value, ty),
        (IrGlobalValue::Array(elements), IrType::Array { element, length }) => {
            if u64::try_from(elements.len()).map_err(|_| BackendFailure::CounterOverflow)? != length
            {
                return Err(BackendFailure::InvalidIr);
            }
            if elements.is_empty() {
                return Ok("zeroinitializer".to_owned());
            }
            let mut text = String::from("[");
            let element_type = element.ty();
            let llvm_element_type = llvm_type(program, element_type)?;
            for (index, value) in elements.iter().enumerate() {
                if index != 0 {
                    text.push_str(", ");
                }
                write!(
                    text,
                    "{llvm_element_type} {}",
                    constant_operand(*value, element_type)?
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            text.push(']');
            Ok(text)
        }
        (IrGlobalValue::Struct(fields), IrType::Nominal(id)) => {
            let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
            let IrNominalKind::Struct { fields: declared } = nominal.kind() else {
                return Err(BackendFailure::InvalidIr);
            };
            if fields.len() != declared.len() {
                return Err(BackendFailure::InvalidIr);
            }
            if fields.is_empty() {
                return Ok("zeroinitializer".to_owned());
            }
            let mut text = String::from("{ ");
            for (index, (value, field)) in fields.iter().zip(declared).enumerate() {
                if index != 0 {
                    text.push_str(", ");
                }
                write!(
                    text,
                    "{} {}",
                    llvm_type(program, field.ty())?,
                    global_constant_value(program, value, field.ty())?
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            text.push_str(" }");
            Ok(text)
        }
        _ => Err(BackendFailure::InvalidIr),
    }
}

fn emit_nominal_declarations(
    output: &mut String,
    program: &IrProgram<'_, '_, '_>,
) -> Result<(), BackendFailure> {
    let mut emitted = false;
    for nominal in program.nominals() {
        // A box is a pointer and an opaque system resource carries the
        // representation its [QUAL-1] qualification record fixes; neither
        // needs a named aggregate type.
        if nominal.is_tag_only_enum()
            || matches!(
                nominal.kind(),
                IrNominalKind::Box { .. }
                    | IrNominalKind::Arena { .. }
                    | IrNominalKind::ArenaStorage
                    | IrNominalKind::SystemResource(_)
            )
        {
            continue;
        }
        emitted = true;
        write!(output, "{} = type {{ ", nominal_symbol(nominal.id()))
            .map_err(|_| BackendFailure::TextEmission)?;
        match nominal.kind() {
            IrNominalKind::Struct { fields } => {
                for (index, field) in fields.iter().enumerate() {
                    if index != 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&llvm_type(program, field.ty())?);
                }
            }
            IrNominalKind::Enum { variants } => {
                output.push_str("i32");
                for variant in variants {
                    for field in variant.fields() {
                        output.push_str(", ");
                        output.push_str(&llvm_type(program, field.ty())?);
                    }
                }
            }
            IrNominalKind::Box { .. }
            | IrNominalKind::Arena { .. }
            | IrNominalKind::ArenaStorage
            | IrNominalKind::SystemResource(_) => {
                return Err(BackendFailure::InvalidIr);
            }
        }
        output.push_str(" }\n");
    }
    if emitted {
        output.push('\n');
    }
    Ok(())
}

#[derive(Clone)]
struct Incoming {
    predecessor: IrBlockId,
    arguments: Vec<IrValueId>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum IntrinsicDeclaration {
    Overflow {
        name: String,
        ty: String,
    },
    UnaryWithFlag {
        name: String,
        ty: String,
    },
    Unary {
        name: String,
        ty: String,
    },
    Binary {
        name: String,
        ty: String,
    },
    Ternary {
        name: String,
        ty: String,
    },
    UnaryCast {
        name: String,
        result_ty: String,
        argument_ty: String,
    },
}

struct FunctionEmitter<'program, 'state> {
    program: &'program IrProgram<'program, 'program, 'program>,
    /// The [QUAL-1] table lookup this build already performed. Every emission
    /// site reads the resolved row; none consults the table again.
    qualification: &'program Qualification,
    function: &'program IrFunction,
    entry_goal: Option<&'program IrEntryGoal>,
    entry_value_names: HashMap<IrValueId, String>,
    target: TargetLayout,
    traps: &'state mut Vec<Vec<u8>>,
    intrinsics: &'state mut BTreeSet<IntrinsicDeclaration>,
    incoming: Vec<Vec<Incoming>>,
    output: String,
    /// Stack slot declarations hoisted to the top of the function's entry block.
    ///
    /// A slot is requested where it is used, but a repeated `alloca` grows the
    /// frame once per execution, so a slot inside a loop would grow the frame
    /// without bound. Declaring every slot in the entry block, which runs
    /// exactly once per call, keeps frame size a property of the function
    /// rather than of the iteration count. Stores stay at the use site.
    entry_prelude: String,
    temporary: u32,
}

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    fn new(
        program: &'program IrProgram<'_, '_, '_>,
        qualification: &'program Qualification,
        function: &'program IrFunction,
        target: TargetLayout,
        traps: &'state mut Vec<Vec<u8>>,
        intrinsics: &'state mut BTreeSet<IntrinsicDeclaration>,
    ) -> Self {
        Self {
            program,
            qualification,
            function,
            entry_goal: None,
            entry_value_names: HashMap::new(),
            target,
            traps,
            intrinsics,
            incoming: Vec::new(),
            output: String::new(),
            entry_prelude: String::new(),
            temporary: 0,
        }
    }

    fn with_entry_goal(
        mut self,
        goal: &'program IrEntryGoal,
        input_names: Vec<String>,
    ) -> Result<Self, BackendFailure> {
        if goal.inputs().len() != input_names.len()
            || goal.inputs().len() != self.function.parameters().len()
        {
            return Err(BackendFailure::InvalidIr);
        }
        let mut names = HashMap::new();
        for (((value, ty), (_, parameter_ty)), name) in goal
            .inputs()
            .iter()
            .zip(self.function.parameters())
            .zip(input_names)
        {
            if ty != parameter_ty
                || goal.ty(*value) != Some(*ty)
                || names.insert(*value, name).is_some()
            {
                return Err(BackendFailure::InvalidIr);
            }
        }
        self.entry_goal = Some(goal);
        self.entry_value_names = names;
        Ok(self)
    }

    fn emit(mut self) -> Result<String, BackendFailure> {
        self.incoming = self.collect_incoming()?;
        write!(
            self.output,
            "define internal {} @{}(",
            llvm_type(self.program, self.function.result())?,
            source_symbol(self.function.name())
        )
        .map_err(|_| BackendFailure::TextEmission)?;
        for (index, (value, ty)) in self.function.parameters().iter().enumerate() {
            if index != 0 {
                self.output.push_str(", ");
            }
            write!(
                self.output,
                "{} {}",
                llvm_type(self.program, *ty)?,
                self.value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        self.output.push_str(") {\n");
        let mut prelude_anchor = None;
        for (index, block) in self.function.blocks().iter().enumerate() {
            let block_id =
                IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
            writeln!(self.output, "{}:", block_label(block_id))
                .map_err(|_| BackendFailure::TextEmission)?;
            if index == 0 {
                prelude_anchor = Some(self.output.len());
            }
            self.emit_block_parameters(block_id, block)?;
            for (instruction_index, instruction) in block.instructions().iter().enumerate() {
                self.emit_instruction(block_id, instruction_index, instruction)?;
            }
            self.emit_terminator(block_id, block.terminator())?;
        }
        self.output.push_str("}\n\n");
        if !self.entry_prelude.is_empty() {
            let anchor = prelude_anchor.ok_or(BackendFailure::InvalidIr)?;
            self.output.insert_str(anchor, &self.entry_prelude);
        }
        Ok(self.output)
    }

    fn emit_entry_goal(mut self) -> Result<EntryGoalEmission, BackendFailure> {
        let goal = self.entry_goal.ok_or(BackendFailure::InvalidIr)?;
        for definition in goal.definitions() {
            if !entry_goal_operation(definition.operation()) {
                return Err(BackendFailure::InvalidIr);
            }
            self.emit_definition(definition.result(), definition.ty(), definition.operation())?;
        }
        if !self.entry_prelude.is_empty() || self.value_type(goal.condition()) != Some(IrType::Bool)
        {
            return Err(BackendFailure::InvalidIr);
        }
        let condition = self.value_name(goal.condition());
        let trap = self.register_trap(goal.trap())?;
        Ok(EntryGoalEmission {
            definitions: self.output,
            condition,
            trap,
            trap_length: self.traps[trap].len(),
        })
    }

    fn collect_incoming(&self) -> Result<Vec<Vec<Incoming>>, BackendFailure> {
        let mut incoming = vec![Vec::new(); self.function.blocks().len()];
        for (index, block) in self.function.blocks().iter().enumerate() {
            if let IrTerminator::Jump {
                target, arguments, ..
            } = block.terminator()
            {
                let predecessor =
                    IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
                incoming
                    .get_mut(target.index())
                    .ok_or(BackendFailure::InvalidIr)?
                    .push(Incoming {
                        predecessor,
                        arguments: arguments.clone(),
                    });
            }
        }
        Ok(incoming)
    }

    fn emit_block_parameters(
        &mut self,
        block_id: IrBlockId,
        block: &IrBlock,
    ) -> Result<(), BackendFailure> {
        if block.parameters().is_empty() {
            return Ok(());
        }
        let incoming = self
            .incoming
            .get(block_id.index())
            .ok_or(BackendFailure::InvalidIr)?;
        if incoming.is_empty()
            || incoming
                .iter()
                .any(|edge| edge.arguments.len() != block.parameters().len())
        {
            return Err(BackendFailure::InvalidIr);
        }
        for (parameter_index, (parameter, ty)) in block.parameters().iter().enumerate() {
            write!(
                self.output,
                "  {} = phi {} ",
                self.value_name(*parameter),
                llvm_type(self.program, *ty)?
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            for (edge_index, edge) in incoming.iter().enumerate() {
                let argument = *edge
                    .arguments
                    .get(parameter_index)
                    .ok_or(BackendFailure::InvalidIr)?;
                if self.value_type(argument) != Some(*ty) {
                    return Err(BackendFailure::InvalidIr);
                }
                if edge_index != 0 {
                    self.output.push_str(", ");
                }
                write!(
                    self.output,
                    "[ {}, %{} ]",
                    self.value_name(argument),
                    block_exit_label(edge.predecessor, self.block(edge.predecessor)?)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
            }
            self.output.push('\n');
        }
        Ok(())
    }

    fn emit_instruction(
        &mut self,
        block: IrBlockId,
        index: usize,
        instruction: &IrInstruction,
    ) -> Result<(), BackendFailure> {
        match instruction {
            IrInstruction::Define {
                result,
                ty,
                operation,
            } => self.emit_definition(*result, *ty, operation),
            IrInstruction::Check { condition, trap } => {
                if self.value_type(*condition) != Some(IrType::Bool) {
                    return Err(BackendFailure::InvalidIr);
                }
                let trap_id = self.register_trap(trap)?;
                writeln!(
                    self.output,
                    "  br i1 {}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:",
                    self.value_name(*condition),
                    check_continue_label(block, index),
                    check_trap_label(block, index),
                    check_trap_label(block, index),
                    self.traps[trap_id].len(),
                    check_continue_label(block, index)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            IrInstruction::StoreBuffer {
                buffer,
                index,
                value,
            } => self.emit_buffer_store(*buffer, *index, *value),
            IrInstruction::Store {
                address,
                value,
                referent,
            } => self.emit_store(*address, *value, *referent),
            IrInstruction::Drop(drop) => self.emit_drop(*drop),
        }
    }

    fn emit_definition(
        &mut self,
        result: IrValueId,
        ty: IrType,
        operation: &IrOperation,
    ) -> Result<(), BackendFailure> {
        if self.value_type(result) != Some(ty) {
            return Err(BackendFailure::InvalidIr);
        }
        match operation {
            IrOperation::Constant(constant) => self.emit_constant(result, ty, *constant),
            IrOperation::Call {
                function,
                arguments,
            } => self.emit_call(result, ty, *function, arguments),
            IrOperation::SystemCall {
                operation,
                arguments,
                trap,
            } => self.emit_system_call(result, ty, *operation, arguments, trap.as_ref()),
            IrOperation::Integer {
                operation,
                operand_type,
                arguments,
            } => self.emit_integer(result, ty, *operation, *operand_type, arguments),
            IrOperation::Float {
                operation,
                operand_type,
                arguments,
            } => self.emit_float(result, ty, *operation, *operand_type, arguments),
            IrOperation::NumericConversion {
                source_type,
                destination_type,
                value,
            } => self.emit_numeric_conversion(result, ty, *source_type, *destination_type, *value),
            IrOperation::Reinterpret {
                source_type,
                destination_type,
                value,
            } => self.emit_reinterpret(result, ty, *source_type, *destination_type, *value),
            IrOperation::Boolean {
                operation,
                arguments,
            } => self.emit_boolean(result, ty, *operation, arguments),
            IrOperation::EnumEquality {
                equal,
                operand_type,
                arguments,
            } => self.emit_enum_equality(result, ty, *equal, *operand_type, *arguments),
            IrOperation::ArrayFill {
                value,
                target_domain,
            } => self.emit_array_fill(result, ty, *value, *target_domain),
            IrOperation::ArrayIndex {
                root,
                offset,
                target_domain,
            } => self.emit_array_index(result, ty, *root, *offset, *target_domain),
            IrOperation::InsertArray {
                aggregate,
                index,
                value,
            } => self.emit_array_insertion(result, ty, *aggregate, *index, *value),
            IrOperation::BufferFill {
                length,
                value,
                trap,
                target_domains,
            } => self.emit_buffer_fill(result, ty, *length, *value, trap, *target_domains),
            IrOperation::BufferVacant {
                length,
                trap,
                target_domains,
            } => self.emit_buffer_vacant(result, ty, *length, trap, *target_domains),
            IrOperation::BufferLength { buffer } => self.emit_buffer_length(result, ty, *buffer),
            IrOperation::BufferIndex {
                buffer,
                offset,
                target_domain,
            } => self.emit_buffer_index(result, ty, *buffer, *offset, *target_domain),
            IrOperation::BufferProbeSkip {
                buffer,
                index,
                limit,
                needles,
            } => self.emit_buffer_probe_skip(result, ty, *buffer, *index, *limit, needles),
            IrOperation::SliceFromArray { array } => self.emit_slice_from_array(result, ty, *array),
            IrOperation::SliceFromBuffer { buffer } => {
                self.emit_slice_from_buffer(result, ty, *buffer)
            }
            IrOperation::SliceLength { slice } => self.emit_slice_length(result, ty, *slice),
            IrOperation::SliceIndex {
                slice,
                offset,
                target_domain,
            } => self.emit_slice_index(result, ty, *slice, *offset, *target_domain),
            IrOperation::BoxNew { nominal, value } => {
                self.emit_box_new(result, ty, *nominal, *value)
            }
            IrOperation::BoxDeref { nominal, value } => {
                self.emit_box_deref(result, ty, *nominal, *value)
            }
            IrOperation::ArenaListNew => self.emit_arena_list_new(result, ty),
            IrOperation::ArenaNew {
                nominal,
                list,
                value,
            } => self.emit_arena_new(result, ty, *nominal, *list, *value),
            IrOperation::ArenaDeref { nominal, value } => {
                self.emit_arena_deref(result, ty, *nominal, *value)
            }
            IrOperation::ConstructStruct { nominal, fields } => {
                self.emit_struct(result, ty, *nominal, fields)
            }
            IrOperation::ConstructEnum {
                nominal,
                variant,
                fields,
            } => self.emit_enum(result, ty, *nominal, *variant, fields),
            IrOperation::ProjectStruct {
                aggregate,
                nominal,
                field,
                consume_root,
            } => {
                self.emit_struct_projection(result, ty, *aggregate, *nominal, *field, *consume_root)
            }
            IrOperation::InsertStruct {
                aggregate,
                nominal,
                field,
                value,
            } => self.emit_struct_insertion(result, ty, *aggregate, *nominal, *field, *value),
            IrOperation::ProjectVariant {
                aggregate,
                nominal,
                variant,
                field,
            } => self.emit_variant_projection(result, ty, *aggregate, *nominal, *variant, *field),
            IrOperation::AddressOf { value, referent } => {
                self.emit_address_of(result, ty, *value, *referent)
            }
            IrOperation::Load { address, referent } => {
                self.emit_load(result, ty, *address, *referent)
            }
        }
    }

    fn emit_terminator(
        &mut self,
        block: IrBlockId,
        terminator: &IrTerminator,
    ) -> Result<(), BackendFailure> {
        match terminator {
            IrTerminator::Jump {
                target,
                arguments,
                drops,
            } => {
                let target_block = self.block(*target)?;
                if target_block.parameters().len() != arguments.len() {
                    return Err(BackendFailure::InvalidIr);
                }
                for (argument, (_, ty)) in arguments.iter().zip(target_block.parameters()) {
                    if self.value_type(*argument) != Some(*ty) {
                        return Err(BackendFailure::InvalidIr);
                    }
                }
                self.emit_drops(drops)?;
                writeln!(self.output, "  br label %{}", block_label(*target))
                    .map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Return { value, drops } => {
                if self.value_type(*value) != Some(self.function.result()) {
                    return Err(BackendFailure::InvalidIr);
                }
                self.emit_drops(drops)?;
                writeln!(
                    self.output,
                    "  ret {} {}",
                    llvm_type(self.program, self.function.result())?,
                    self.value_name(*value)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Match {
                scrutinee,
                enum_type,
                targets,
            } => {
                let (tag, tag_ty) = self.match_tag(*scrutinee, *enum_type)?;
                writeln!(
                    self.output,
                    "  switch {tag_ty} {tag}, label %{} [",
                    invalid_tag_label(block)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                let mut seen = BTreeSet::new();
                for target in targets {
                    if !seen.insert(target.tag()) {
                        return Err(BackendFailure::InvalidIr);
                    }
                    writeln!(
                        self.output,
                        "    {tag_ty} {}, label %{}",
                        target.tag(),
                        block_label(target.block())
                    )
                    .map_err(|_| BackendFailure::TextEmission)?;
                }
                writeln!(
                    self.output,
                    "  ]\n{}:\n  call void @abort()\n  unreachable",
                    invalid_tag_label(block)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
        }
    }

    fn match_tag(
        &mut self,
        scrutinee: IrValueId,
        enum_type: IrEnumType,
    ) -> Result<(String, String), BackendFailure> {
        match enum_type {
            IrEnumType::Bool => {
                if self.value_type(scrutinee) != Some(IrType::Bool) {
                    return Err(BackendFailure::InvalidIr);
                }
                Ok((self.value_name(scrutinee), "i1".to_owned()))
            }
            IrEnumType::Nominal(nominal) => {
                if self.value_type(scrutinee) != Some(IrType::Nominal(nominal)) {
                    return Err(BackendFailure::InvalidIr);
                }
                let data = self.nominal(nominal)?;
                let IrNominalKind::Enum { .. } = data.kind() else {
                    return Err(BackendFailure::InvalidIr);
                };
                if data.is_tag_only_enum() {
                    return Ok((
                        self.value_name(scrutinee),
                        llvm_type(self.program, IrType::Nominal(nominal))?,
                    ));
                }
                let temporary = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{temporary} = extractvalue {} {}, 0",
                    llvm_type(self.program, IrType::Nominal(nominal))?,
                    self.value_name(scrutinee)
                )
                .map_err(|_| BackendFailure::TextEmission)?;
                Ok((format!("%{temporary}"), "i32".to_owned()))
            }
        }
    }

    fn nominal(&self, id: IrNominalId) -> Result<&IrNominal, BackendFailure> {
        self.program.nominal(id).ok_or(BackendFailure::InvalidIr)
    }

    fn block(&self, id: IrBlockId) -> Result<&IrBlock, BackendFailure> {
        self.function
            .blocks()
            .get(id.index())
            .ok_or(BackendFailure::InvalidIr)
    }

    fn register_trap(&mut self, trap: &IrTrapSite) -> Result<usize, BackendFailure> {
        let index = self.traps.len();
        let _ = u32::try_from(index).map_err(|_| BackendFailure::CounterOverflow)?;
        self.traps.push(trap_record(trap));
        Ok(index)
    }

    fn emit_drop(&mut self, drop: IrDrop) -> Result<(), BackendFailure> {
        if self.value_type(drop.value()) != Some(drop.ty()) {
            return Err(BackendFailure::InvalidIr);
        }
        let value_name = self.value_name(drop.value());
        match drop.ty() {
            IrType::Array { .. } | IrType::Slice { .. } => {}
            IrType::Buffer { .. } => {
                emit_value_cleanup(
                    self.program,
                    self.qualification,
                    &mut self.output,
                    &mut self.temporary,
                    drop.ty(),
                    value_name.clone(),
                )?;
            }
            IrType::Nominal(nominal) if !self.nominal(nominal)?.is_tag_only_enum() => {
                match self.nominal(nominal)?.kind() {
                    IrNominalKind::Struct { .. } => {}
                    // An arena value derives no owner-scope drop action; its
                    // storage is released with its region [STOR-3, STOR-4].
                    IrNominalKind::Arena { .. } => {}
                    // The region's allocation-list drop is that release:
                    // walk the list and free every registered allocation.
                    IrNominalKind::ArenaStorage => {
                        emit_value_cleanup(
                            self.program,
                            self.qualification,
                            &mut self.output,
                            &mut self.temporary,
                            drop.ty(),
                            value_name.clone(),
                        )?;
                    }
                    // The checked program's own [SYS-5] record is the single
                    // source of truth for which action runs here, so a table
                    // row disagreeing with it stops rather than silently
                    // emitting a different release.
                    IrNominalKind::SystemResource(contract) => {
                        if drop.release().action != Some(contract.action) {
                            return Err(BackendFailure::InvalidIr);
                        }
                        let contract = *contract;
                        system::emit_resource_release(
                            self.qualification,
                            &mut self.output,
                            &mut self.temporary,
                            contract,
                            &value_name,
                        )?;
                    }
                    IrNominalKind::Enum { .. } | IrNominalKind::Box { .. } => {
                        if type_requires_cleanup(self.program, drop.ty())? {
                            emit_value_cleanup(
                                self.program,
                                self.qualification,
                                &mut self.output,
                                &mut self.temporary,
                                drop.ty(),
                                value_name.clone(),
                            )?;
                        }
                    }
                }
            }
            _ => return Err(BackendFailure::InvalidIr),
        }
        writeln!(self.output, "  ; drop {value_name}").map_err(|_| BackendFailure::TextEmission)
    }

    fn emit_drops(&mut self, drops: &[IrDrop]) -> Result<(), BackendFailure> {
        for drop in drops {
            self.emit_drop(*drop)?;
        }
        Ok(())
    }

    /// Declares a stack slot under a caller-chosen name in the entry block.
    ///
    /// The name must be an otherwise undefined value name, because the slot
    /// declaration becomes that name's single definition.
    fn declare_entry_slot(&mut self, name: &str, ty: &str) -> Result<(), BackendFailure> {
        writeln!(self.entry_prelude, "  {name} = alloca {ty}")
            .map_err(|_| BackendFailure::TextEmission)
    }

    /// Reserves a fresh stack slot in the entry block and returns its name.
    ///
    /// Each call reserves a distinct slot, so two slots never alias even when
    /// they hold values of the same type and live at the same time.
    fn entry_slot(&mut self, ty: &str) -> Result<String, BackendFailure> {
        let name = format!("%{}", self.next_temporary()?);
        self.declare_entry_slot(&name, ty)?;
        Ok(name)
    }

    fn next_temporary(&mut self) -> Result<String, BackendFailure> {
        let current = self.temporary;
        self.temporary = self
            .temporary
            .checked_add(1)
            .ok_or(BackendFailure::CounterOverflow)?;
        Ok(if self.entry_goal.is_some() {
            format!("entry.goal.t{current}")
        } else {
            format!("t{current}")
        })
    }

    fn value_type(&self, value: IrValueId) -> Option<IrType> {
        self.entry_goal
            .map_or_else(|| self.function.value_type(value), |goal| goal.ty(value))
    }

    fn value_name(&self, value: IrValueId) -> String {
        if self.entry_goal.is_some() {
            self.entry_value_names
                .get(&value)
                .cloned()
                .unwrap_or_else(|| format!("%entry.goal.v{}", value.ordinal()))
        } else {
            value_name(value)
        }
    }
}

struct EntryGoalEmission {
    definitions: String,
    condition: String,
    trap: usize,
    trap_length: usize,
}

pub(super) fn entry_goal_operation(operation: &IrOperation) -> bool {
    matches!(
        operation,
        IrOperation::Constant(_)
            | IrOperation::Integer { .. }
            | IrOperation::Float { .. }
            | IrOperation::NumericConversion { .. }
            | IrOperation::Reinterpret { .. }
            | IrOperation::Boolean { .. }
            | IrOperation::EnumEquality { .. }
            | IrOperation::BufferLength { .. }
            | IrOperation::SliceLength { .. }
            | IrOperation::BoxDeref { .. }
            | IrOperation::ProjectStruct {
                consume_root: false,
                ..
            }
            | IrOperation::Load { .. }
    )
}

fn llvm_type(program: &IrProgram<'_, '_, '_>, ty: IrType) -> Result<String, BackendFailure> {
    match ty {
        IrType::Unit => Ok("i8".to_owned()),
        IrType::Bool => Ok("i1".to_owned()),
        IrType::Integer { width: 8, .. } => Ok("i8".to_owned()),
        IrType::Integer { width: 16, .. } => Ok("i16".to_owned()),
        IrType::Integer { width: 32, .. } => Ok("i32".to_owned()),
        IrType::Integer { width: 64, .. } => Ok("i64".to_owned()),
        IrType::Integer { .. } => Err(BackendFailure::InvalidIr),
        IrType::Float { width: 32 } => Ok("float".to_owned()),
        IrType::Float { width: 64 } => Ok("double".to_owned()),
        IrType::Float { .. } => Err(BackendFailure::InvalidIr),
        IrType::Array { element, length } => Ok(format!(
            "[{length} x {}]",
            llvm_type(program, element.ty())?
        )),
        IrType::Buffer { .. } | IrType::Slice { .. } => Ok("{ ptr, i64 }".to_owned()),
        IrType::Address(_) => Ok("ptr".to_owned()),
        IrType::Nominal(id) => {
            let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
            if matches!(
                nominal.kind(),
                IrNominalKind::Box { .. }
                    | IrNominalKind::Arena { .. }
                    | IrNominalKind::ArenaStorage
            ) {
                return Ok("ptr".to_owned());
            }
            // [QUAL-1] fixes an opaque resource's representation in its
            // qualification record. Emission is reached only after
            // qualification accepted the program, so the row this reads is the
            // one qualification already resolved for this resource.
            if let IrNominalKind::SystemResource(contract) = nominal.kind() {
                return Ok(qualified_representation(contract.resource)
                    .llvm()
                    .to_owned());
            }
            if nominal.is_tag_only_enum() {
                let IrNominalKind::Enum { variants } = nominal.kind() else {
                    return Err(BackendFailure::InvalidIr);
                };
                Ok(if variants.len() <= 2 { "i1" } else { "i32" }.to_owned())
            } else {
                Ok(nominal_symbol(id))
            }
        }
    }
}

fn is_tag_only_type(program: &IrProgram<'_, '_, '_>, ty: IrType) -> Result<bool, BackendFailure> {
    match ty {
        IrType::Bool => Ok(true),
        IrType::Nominal(id) => program
            .nominal(id)
            .map(IrNominal::is_tag_only_enum)
            .ok_or(BackendFailure::InvalidIr),
        _ => Ok(false),
    }
}

fn constant_operand(constant: IrConstant, ty: IrType) -> Result<String, BackendFailure> {
    match (constant, ty) {
        (IrConstant::Unit, IrType::Unit) => Ok("0".to_owned()),
        (IrConstant::Bool(value), IrType::Bool) => Ok(u8::from(value).to_string()),
        (
            IrConstant::Integer {
                ty: constant_ty,
                bits,
            },
            actual_ty,
        ) if constant_ty == actual_ty => {
            let IrType::Integer { width, signed } = actual_ty else {
                return Err(BackendFailure::InvalidIr);
            };
            if !matches!(width, 8 | 16 | 32 | 64) {
                return Err(BackendFailure::InvalidIr);
            }
            let mask = if width == 64 {
                u64::MAX
            } else {
                (1_u64 << width) - 1
            };
            let bits = bits & mask;
            Ok(if signed && bits & (1_u64 << (width - 1)) != 0 {
                (i128::from(bits) - (1_i128 << width)).to_string()
            } else {
                bits.to_string()
            })
        }
        (
            IrConstant::Float {
                ty: constant_ty,
                bits,
            },
            actual_ty,
        ) if constant_ty == actual_ty => match actual_ty {
            IrType::Float { width: 32 } => {
                let bits = u32::try_from(bits).map_err(|_| BackendFailure::InvalidIr)?;
                let widened = f64::from(f32::from_bits(bits)).to_bits();
                Ok(format!("0x{widened:016X}"))
            }
            IrType::Float { width: 64 } => Ok(format!("0x{bits:016X}")),
            _ => Err(BackendFailure::InvalidIr),
        },
        _ => Err(BackendFailure::InvalidIr),
    }
}

fn variant_field_base(
    variants: &[crate::IrVariant],
    selected: u32,
) -> Result<usize, BackendFailure> {
    let mut index = 1_usize;
    for variant in variants {
        if variant.tag() == selected {
            return Ok(index);
        }
        index = index
            .checked_add(variant.fields().len())
            .ok_or(BackendFailure::CounterOverflow)?;
    }
    Err(BackendFailure::InvalidIr)
}

fn block_exit_label(block_id: IrBlockId, block: &IrBlock) -> String {
    let mut label = block_label(block_id);
    for (index, instruction) in block.instructions().iter().enumerate() {
        match instruction {
            IrInstruction::Check { .. } => label = check_continue_label(block_id, index),
            IrInstruction::Define {
                result,
                operation:
                    IrOperation::Integer {
                        operation:
                            IrIntegerOperation::DivideChecked | IrIntegerOperation::RemainderChecked,
                        ..
                    },
                ..
            } => label = integer_continue_label(*result),
            IrInstruction::Define {
                result,
                operation: IrOperation::ArrayFill { .. },
                ..
            } => label = array_fill_done_label(*result),
            IrInstruction::Define {
                result,
                operation: IrOperation::BoxNew { .. },
                ..
            } => label = box_new_ready_label(*result),
            IrInstruction::Define {
                result,
                operation: IrOperation::ArenaNew { .. },
                ..
            } => label = arena_new_ready_label(*result),
            IrInstruction::Define {
                result,
                operation: IrOperation::BufferFill { .. },
                ..
            } => label = buffer_fill_done_label(*result),
            IrInstruction::Define {
                result,
                operation: IrOperation::BufferVacant { .. },
                ..
            } => label = buffer_vacant_done_label(*result),
            IrInstruction::Define {
                result,
                operation: IrOperation::BufferProbeSkip { .. },
                ..
            } => label = buffer_probe_join_label(*result),
            _ => {}
        }
    }
    label
}

fn block_label(block: IrBlockId) -> String {
    if block.ordinal() == 0 {
        "entry".to_owned()
    } else {
        format!("bb{}", block.ordinal())
    }
}

fn value_name(value: IrValueId) -> String {
    format!("%v{}", value.ordinal())
}

fn nominal_symbol(nominal: IrNominalId) -> String {
    format!("%wf.t{}", nominal.ordinal())
}

fn constant_symbol(constant: crate::IrConstantId) -> String {
    format!("@.wf_const.{}", constant.ordinal())
}

fn check_continue_label(block: IrBlockId, index: usize) -> String {
    format!("check.cont.b{}.i{index}", block.ordinal())
}

fn check_trap_label(block: IrBlockId, index: usize) -> String {
    format!("check.trap.b{}.i{index}", block.ordinal())
}

fn integer_safe_label(value: IrValueId) -> String {
    format!("integer.safe.v{}", value.ordinal())
}

fn integer_error_label(value: IrValueId) -> String {
    format!("integer.error.v{}", value.ordinal())
}

fn integer_continue_label(value: IrValueId) -> String {
    format!("integer.cont.v{}", value.ordinal())
}

fn box_new_ready_label(value: IrValueId) -> String {
    format!("box.new.ready.v{}", value.ordinal())
}

fn arena_new_ready_label(value: IrValueId) -> String {
    format!("arena.new.ready.v{}", value.ordinal())
}

fn array_fill_head_label(value: IrValueId) -> String {
    format!("array.fill.head.v{}", value.ordinal())
}

fn array_fill_body_label(value: IrValueId) -> String {
    format!("array.fill.body.v{}", value.ordinal())
}

fn array_fill_done_label(value: IrValueId) -> String {
    format!("array.fill.done.v{}", value.ordinal())
}

fn invalid_tag_label(block: IrBlockId) -> String {
    format!("invalid.tag.b{}", block.ordinal())
}

fn source_symbol(name: &str) -> String {
    format!("wf_{name}")
}

pub(super) fn trap_record(trap: &IrTrapSite) -> Vec<u8> {
    let components = trap
        .node_path
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"rule_id\":{},\"message\":{},\"function\":{},\"node_path\":[{components}]}}\n",
        json_string(trap.rule_id),
        json_string(&trap.message),
        json_string(&trap.function)
    )
    .into_bytes()
}

/// Encodes one [DIAG-3] record field as a JSON string.
///
/// The input is always a Rust `str`, so it is already one well-formed UTF-8
/// sequence; the record must carry those exact bytes. Iteration is therefore
/// over scalar values, not bytes: pushing a `char` writes its complete UTF-8
/// encoding, so a multi-byte scalar survives intact and no record can end
/// inside an encoding. Byte iteration with `char::from` was wrong for exactly
/// this case — it reinterpreted each continuation byte as the Latin-1 scalar
/// of the same value and re-encoded it, doubling every non-ASCII byte. ASCII
/// input is unaffected either way.
///
/// Only `"`, `\`, and newline need escaping here: [FORM-5] admits no other
/// control in a STRING, so no other scalar in a record field requires a JSON
/// escape.
fn json_string(value: &str) -> String {
    let mut encoded = String::from("\"");
    for scalar in value.chars() {
        match scalar {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            _ => encoded.push(scalar),
        }
    }
    encoded.push('"');
    encoded
}

fn llvm_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 3);
    for byte in bytes {
        let _ = write!(encoded, "\\{byte:02X}");
    }
    encoded
}
