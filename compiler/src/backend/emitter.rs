//! Conservative textual LLVM emission for exact Whitefoot v0.14.
//!
//! Emission consumes only target-independent IR. It preserves every retained
//! check, emits no overflow or alias promises, initializes complete aggregate
//! representations, and keeps a defensive abort edge for enum discriminants.

mod operations;

use std::collections::BTreeSet;
use std::fmt::Write;

use crate::{
    IrBlock, IrBlockId, IrBooleanOperation, IrConstant, IrDrop, IrEnumType, IrFunction,
    IrInstruction, IrIntegerOperation, IrNominal, IrNominalId, IrNominalKind, IrOperation,
    IrProgram, IrTerminator, IrTrapSite, IrType, IrValueId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackendFailure {
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

pub fn emit_llvm_v0_14(program: &IrProgram<'_, '_, '_>) -> Result<LlvmModule, BackendFailure> {
    let main = program
        .functions()
        .get(program.main_ordinal() as usize)
        .ok_or(BackendFailure::InvalidIr)?;
    if main.result() != IrType::Unit || !main.parameters().is_empty() {
        return Err(BackendFailure::InvalidIr);
    }

    let mut traps = Vec::new();
    let mut intrinsics = BTreeSet::new();
    let mut functions = String::new();
    for function in program.functions() {
        functions.push_str(
            &FunctionEmitter::new(program, function, &mut traps, &mut intrinsics).emit()?,
        );
    }
    let has_matches = program.functions().iter().any(|function| {
        function
            .blocks()
            .iter()
            .any(|block| matches!(block.terminator(), IrTerminator::Match { .. }))
    });

    let mut text = String::from(
        "; Whitefoot v0.14 conservative nominal-data module\nsource_filename = \"whitefoot\"\n\n",
    );
    emit_nominal_declarations(&mut text, program)?;
    for (index, bytes) in traps.iter().enumerate() {
        writeln!(
            text,
            "@.wf_trap.{index} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            bytes.len(),
            llvm_bytes(bytes)
        )
        .map_err(|_| BackendFailure::TextEmission)?;
    }
    if !traps.is_empty() {
        text.push('\n');
        text.push_str("declare i64 @write(i32, ptr, i64)\n");
    }
    if !traps.is_empty() || has_matches {
        text.push_str("declare void @abort() noreturn\n");
    }
    if !traps.is_empty() {
        text.push_str(
            "\ndefine private void @wf_trap(ptr %message, i64 %length) noreturn {\nentry:\n  br label %write.loop\nwrite.loop:\n  %cursor = phi ptr [ %message, %entry ], [ %next, %write.more ]\n  %remaining = phi i64 [ %length, %entry ], [ %left, %write.more ]\n  %written = call i64 @write(i32 2, ptr %cursor, i64 %remaining)\n  %complete = icmp eq i64 %written, %remaining\n  br i1 %complete, label %abort, label %write.incomplete\nwrite.incomplete:\n  %progress = icmp sgt i64 %written, 0\n  br i1 %progress, label %write.more, label %abort\nwrite.more:\n  %next = getelementptr i8, ptr %cursor, i64 %written\n  %left = sub i64 %remaining, %written\n  br label %write.loop\nabort:\n  call void @abort()\n  unreachable\n}\n\n",
        );
    } else if has_matches {
        text.push('\n');
    }
    for intrinsic in intrinsics {
        match intrinsic {
            IntrinsicDeclaration::Overflow { name, ty } => {
                writeln!(text, "declare {{ {ty}, i1 }} @{name}({ty}, {ty})")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
            IntrinsicDeclaration::Absolute { name, ty } => {
                writeln!(text, "declare {ty} @{name}({ty}, i1)")
                    .map_err(|_| BackendFailure::TextEmission)?;
            }
        }
    }
    if !functions.is_empty() {
        text.push('\n');
        text.push_str(&functions);
    }
    writeln!(
        text,
        "define i32 @main() {{\nentry:\n  %result = call i8 @{}()\n  ret i32 0\n}}",
        source_symbol(main.name())
    )
    .map_err(|_| BackendFailure::TextEmission)?;
    Ok(LlvmModule { text })
}

fn emit_nominal_declarations(
    output: &mut String,
    program: &IrProgram<'_, '_, '_>,
) -> Result<(), BackendFailure> {
    let mut emitted = false;
    for nominal in program.nominals() {
        if nominal.is_tag_only_enum() {
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
    Overflow { name: String, ty: String },
    Absolute { name: String, ty: String },
}

struct FunctionEmitter<'program, 'state> {
    program: &'program IrProgram<'program, 'program, 'program>,
    function: &'program IrFunction,
    traps: &'state mut Vec<Vec<u8>>,
    intrinsics: &'state mut BTreeSet<IntrinsicDeclaration>,
    incoming: Vec<Vec<Incoming>>,
    output: String,
    temporary: u32,
}

impl<'program, 'state> FunctionEmitter<'program, 'state> {
    fn new(
        program: &'program IrProgram<'_, '_, '_>,
        function: &'program IrFunction,
        traps: &'state mut Vec<Vec<u8>>,
        intrinsics: &'state mut BTreeSet<IntrinsicDeclaration>,
    ) -> Self {
        Self {
            program,
            function,
            traps,
            intrinsics,
            incoming: Vec::new(),
            output: String::new(),
            temporary: 0,
        }
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
                value_name(*value)
            )
            .map_err(|_| BackendFailure::TextEmission)?;
        }
        self.output.push_str(") {\n");
        for (index, block) in self.function.blocks().iter().enumerate() {
            let block_id =
                IrBlockId::from_index(index).map_err(|_| BackendFailure::CounterOverflow)?;
            writeln!(self.output, "{}:", block_label(block_id))
                .map_err(|_| BackendFailure::TextEmission)?;
            self.emit_block_parameters(block_id, block)?;
            for (instruction_index, instruction) in block.instructions().iter().enumerate() {
                self.emit_instruction(block_id, instruction_index, instruction)?;
            }
            self.emit_terminator(block_id, block.terminator())?;
        }
        self.output.push_str("}\n\n");
        Ok(self.output)
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
                value_name(*parameter),
                llvm_type(self.program, *ty)?
            )
            .map_err(|_| BackendFailure::TextEmission)?;
            for (edge_index, edge) in incoming.iter().enumerate() {
                let argument = *edge
                    .arguments
                    .get(parameter_index)
                    .ok_or(BackendFailure::InvalidIr)?;
                if self.function.value_type(argument) != Some(*ty) {
                    return Err(BackendFailure::InvalidIr);
                }
                if edge_index != 0 {
                    self.output.push_str(", ");
                }
                write!(
                    self.output,
                    "[ {}, %{} ]",
                    value_name(argument),
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
            } => self.emit_definition(block, index, *result, *ty, operation),
            IrInstruction::Check { condition, trap } => {
                if self.function.value_type(*condition) != Some(IrType::Bool) {
                    return Err(BackendFailure::InvalidIr);
                }
                let trap_id = self.register_trap(trap)?;
                writeln!(
                    self.output,
                    "  br i1 {}, label %{}, label %{}\n{}:\n  call void @wf_trap(ptr @.wf_trap.{trap_id}, i64 {})\n  unreachable\n{}:",
                    value_name(*condition),
                    check_continue_label(block, index),
                    check_trap_label(block, index),
                    check_trap_label(block, index),
                    self.traps[trap_id].len(),
                    check_continue_label(block, index)
                )
                .map_err(|_| BackendFailure::TextEmission)
            }
            IrInstruction::Drop(drop) => self.emit_drop(*drop),
        }
    }

    fn emit_definition(
        &mut self,
        block: IrBlockId,
        index: usize,
        result: IrValueId,
        ty: IrType,
        operation: &IrOperation,
    ) -> Result<(), BackendFailure> {
        if self.function.value_type(result) != Some(ty) {
            return Err(BackendFailure::InvalidIr);
        }
        match operation {
            IrOperation::Constant(constant) => self.emit_constant(result, ty, *constant),
            IrOperation::Call {
                function,
                arguments,
            } => self.emit_call(result, ty, *function, arguments),
            IrOperation::Integer {
                operation,
                operand_type,
                arguments,
                trap,
            } => self.emit_integer(
                block,
                index,
                result,
                ty,
                *operation,
                *operand_type,
                arguments,
                trap.as_ref(),
            ),
            IrOperation::Boolean {
                operation,
                arguments,
            } => self.emit_boolean(result, ty, *operation, arguments),
            IrOperation::EnumEquality {
                equal,
                operand_type,
                arguments,
            } => self.emit_enum_equality(result, ty, *equal, *operand_type, *arguments),
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
                    if self.function.value_type(*argument) != Some(*ty) {
                        return Err(BackendFailure::InvalidIr);
                    }
                }
                self.emit_drops(drops)?;
                writeln!(self.output, "  br label %{}", block_label(*target))
                    .map_err(|_| BackendFailure::TextEmission)
            }
            IrTerminator::Return { value, drops } => {
                if self.function.value_type(*value) != Some(self.function.result()) {
                    return Err(BackendFailure::InvalidIr);
                }
                self.emit_drops(drops)?;
                writeln!(
                    self.output,
                    "  ret {} {}",
                    llvm_type(self.program, self.function.result())?,
                    value_name(*value)
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
                if self.function.value_type(scrutinee) != Some(IrType::Bool) {
                    return Err(BackendFailure::InvalidIr);
                }
                Ok((value_name(scrutinee), "i1".to_owned()))
            }
            IrEnumType::Nominal(nominal) => {
                if self.function.value_type(scrutinee) != Some(IrType::Nominal(nominal)) {
                    return Err(BackendFailure::InvalidIr);
                }
                let data = self.nominal(nominal)?;
                let IrNominalKind::Enum { .. } = data.kind() else {
                    return Err(BackendFailure::InvalidIr);
                };
                if data.is_tag_only_enum() {
                    return Ok((
                        value_name(scrutinee),
                        llvm_type(self.program, IrType::Nominal(nominal))?,
                    ));
                }
                let temporary = self.next_temporary()?;
                writeln!(
                    self.output,
                    "  %{temporary} = extractvalue {} {}, 0",
                    llvm_type(self.program, IrType::Nominal(nominal))?,
                    value_name(scrutinee)
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
        if self.function.value_type(drop.value()) != Some(drop.ty()) {
            return Err(BackendFailure::InvalidIr);
        }
        let IrType::Nominal(nominal) = drop.ty() else {
            return Err(BackendFailure::InvalidIr);
        };
        if self.nominal(nominal)?.is_tag_only_enum() {
            return Err(BackendFailure::InvalidIr);
        }
        writeln!(self.output, "  ; drop {}", value_name(drop.value()))
            .map_err(|_| BackendFailure::TextEmission)
    }

    fn emit_drops(&mut self, drops: &[IrDrop]) -> Result<(), BackendFailure> {
        for drop in drops {
            self.emit_drop(*drop)?;
        }
        Ok(())
    }

    fn next_temporary(&mut self) -> Result<String, BackendFailure> {
        let current = self.temporary;
        self.temporary = self
            .temporary
            .checked_add(1)
            .ok_or(BackendFailure::CounterOverflow)?;
        Ok(format!("t{current}"))
    }
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
        IrType::Nominal(id) => {
            let nominal = program.nominal(id).ok_or(BackendFailure::InvalidIr)?;
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
                operation: IrOperation::Integer { trap: Some(_), .. },
                ..
            } => label = overflow_continue_label(*result),
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

fn check_continue_label(block: IrBlockId, index: usize) -> String {
    format!("check.cont.b{}.i{index}", block.ordinal())
}

fn check_trap_label(block: IrBlockId, index: usize) -> String {
    format!("check.trap.b{}.i{index}", block.ordinal())
}

fn overflow_continue_label(value: IrValueId) -> String {
    format!("overflow.cont.v{}", value.ordinal())
}

fn overflow_trap_label(value: IrValueId) -> String {
    format!("overflow.trap.v{}", value.ordinal())
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

fn invalid_tag_label(block: IrBlockId) -> String {
    format!("invalid.tag.b{}", block.ordinal())
}

fn source_symbol(name: &str) -> String {
    format!("wf_{name}")
}

fn trap_record(trap: &IrTrapSite) -> Vec<u8> {
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

fn json_string(value: &str) -> String {
    let mut encoded = String::from("\"");
    for byte in value.bytes() {
        match byte {
            b'"' => encoded.push_str("\\\""),
            b'\\' => encoded.push_str("\\\\"),
            b'\n' => encoded.push_str("\\n"),
            _ => encoded.push(char::from(byte)),
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
