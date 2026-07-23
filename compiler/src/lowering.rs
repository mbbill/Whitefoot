//! Target-independent lowering from semantically checked Whitefoot v0.14.
//!
//! The private IR records exact value types, nominal construction/projection,
//! direct calls, retained checks, and explicit control-flow edges. It performs
//! no source admission, label lookup, exhaustiveness decision, or ownership
//! judgment.

use crate::semantic::{
    CheckedBooleanOperation, CheckedEnumType, CheckedIntegerOperation, CheckedProgram, CheckedType,
    TrapSite,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IrValueId(u32);

impl IrValueId {
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IrBlockId(u32);

impl IrBlockId {
    pub(crate) fn from_index(index: usize) -> Result<Self, LoweringFailure> {
        Ok(Self(
            u32::try_from(index).map_err(|_| LoweringFailure::CounterOverflow)?,
        ))
    }

    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct IrNominalId(u32);

impl IrNominalId {
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrType {
    Unit,
    Bool,
    Integer { width: u8, signed: bool },
    Nominal(IrNominalId),
}

fn lower_type(value: CheckedType) -> IrType {
    match value {
        CheckedType::Unit => IrType::Unit,
        CheckedType::Bool => IrType::Bool,
        CheckedType::Integer(integer) => IrType::Integer {
            width: integer.width(),
            signed: integer.signed(),
        },
        CheckedType::Nominal(id) => IrType::Nominal(IrNominalId(id.0)),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrField {
    ty: IrType,
}

impl IrField {
    pub const fn ty(&self) -> IrType {
        self.ty
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrVariant {
    tag: u32,
    fields: Vec<IrField>,
}

impl IrVariant {
    pub const fn tag(&self) -> u32 {
        self.tag
    }

    pub fn fields(&self) -> &[IrField] {
        &self.fields
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrNominalKind {
    Struct { fields: Vec<IrField> },
    Enum { variants: Vec<IrVariant> },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrNominal {
    id: IrNominalId,
    kind: IrNominalKind,
}

impl IrNominal {
    pub const fn id(&self) -> IrNominalId {
        self.id
    }

    pub const fn kind(&self) -> &IrNominalKind {
        &self.kind
    }

    pub fn is_tag_only_enum(&self) -> bool {
        matches!(
            &self.kind,
            IrNominalKind::Enum { variants }
                if variants.iter().all(|variant| variant.fields.is_empty())
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrEnumType {
    Bool,
    Nominal(IrNominalId),
}

impl From<CheckedEnumType> for IrEnumType {
    fn from(value: CheckedEnumType) -> Self {
        match value {
            CheckedEnumType::Bool => Self::Bool,
            CheckedEnumType::Nominal(id) => Self::Nominal(IrNominalId(id.0)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrIntegerOperation {
    AddWrap,
    SubtractWrap,
    MultiplyWrap,
    AddTrap,
    SubtractTrap,
    MultiplyTrap,
    AddChecked,
    SubtractChecked,
    MultiplyChecked,
    DivideChecked,
    RemainderChecked,
    DivideTrap,
    RemainderTrap,
    AbsoluteWrap,
    AbsoluteTrap,
    AbsoluteChecked,
    NegateWrap,
    NegateTrap,
    NegateChecked,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeftWrap,
    ShiftRightWrap,
    ShiftLeftTrap,
    ShiftRightTrap,
    RotateLeft,
    RotateRight,
    PopulationCount,
    LeadingZeros,
    TrailingZeros,
    ByteSwap,
    MultiplyHigh,
    AddSaturating,
    SubtractSaturating,
    MultiplySaturating,
    Minimum,
    Maximum,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

impl From<CheckedIntegerOperation> for IrIntegerOperation {
    fn from(value: CheckedIntegerOperation) -> Self {
        match value {
            CheckedIntegerOperation::AddWrap => Self::AddWrap,
            CheckedIntegerOperation::SubtractWrap => Self::SubtractWrap,
            CheckedIntegerOperation::MultiplyWrap => Self::MultiplyWrap,
            CheckedIntegerOperation::AddTrap => Self::AddTrap,
            CheckedIntegerOperation::SubtractTrap => Self::SubtractTrap,
            CheckedIntegerOperation::MultiplyTrap => Self::MultiplyTrap,
            CheckedIntegerOperation::AddChecked => Self::AddChecked,
            CheckedIntegerOperation::SubtractChecked => Self::SubtractChecked,
            CheckedIntegerOperation::MultiplyChecked => Self::MultiplyChecked,
            CheckedIntegerOperation::DivideChecked => Self::DivideChecked,
            CheckedIntegerOperation::RemainderChecked => Self::RemainderChecked,
            CheckedIntegerOperation::DivideTrap => Self::DivideTrap,
            CheckedIntegerOperation::RemainderTrap => Self::RemainderTrap,
            CheckedIntegerOperation::AbsoluteWrap => Self::AbsoluteWrap,
            CheckedIntegerOperation::AbsoluteTrap => Self::AbsoluteTrap,
            CheckedIntegerOperation::AbsoluteChecked => Self::AbsoluteChecked,
            CheckedIntegerOperation::NegateWrap => Self::NegateWrap,
            CheckedIntegerOperation::NegateTrap => Self::NegateTrap,
            CheckedIntegerOperation::NegateChecked => Self::NegateChecked,
            CheckedIntegerOperation::BitAnd => Self::BitAnd,
            CheckedIntegerOperation::BitOr => Self::BitOr,
            CheckedIntegerOperation::BitXor => Self::BitXor,
            CheckedIntegerOperation::BitNot => Self::BitNot,
            CheckedIntegerOperation::ShiftLeftWrap => Self::ShiftLeftWrap,
            CheckedIntegerOperation::ShiftRightWrap => Self::ShiftRightWrap,
            CheckedIntegerOperation::ShiftLeftTrap => Self::ShiftLeftTrap,
            CheckedIntegerOperation::ShiftRightTrap => Self::ShiftRightTrap,
            CheckedIntegerOperation::RotateLeft => Self::RotateLeft,
            CheckedIntegerOperation::RotateRight => Self::RotateRight,
            CheckedIntegerOperation::PopulationCount => Self::PopulationCount,
            CheckedIntegerOperation::LeadingZeros => Self::LeadingZeros,
            CheckedIntegerOperation::TrailingZeros => Self::TrailingZeros,
            CheckedIntegerOperation::ByteSwap => Self::ByteSwap,
            CheckedIntegerOperation::MultiplyHigh => Self::MultiplyHigh,
            CheckedIntegerOperation::AddSaturating => Self::AddSaturating,
            CheckedIntegerOperation::SubtractSaturating => Self::SubtractSaturating,
            CheckedIntegerOperation::MultiplySaturating => Self::MultiplySaturating,
            CheckedIntegerOperation::Minimum => Self::Minimum,
            CheckedIntegerOperation::Maximum => Self::Maximum,
            CheckedIntegerOperation::Equal => Self::Equal,
            CheckedIntegerOperation::NotEqual => Self::NotEqual,
            CheckedIntegerOperation::Less => Self::Less,
            CheckedIntegerOperation::LessEqual => Self::LessEqual,
            CheckedIntegerOperation::Greater => Self::Greater,
            CheckedIntegerOperation::GreaterEqual => Self::GreaterEqual,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrBooleanOperation {
    And,
    Or,
    ExclusiveOr,
    Not,
}

impl From<CheckedBooleanOperation> for IrBooleanOperation {
    fn from(value: CheckedBooleanOperation) -> Self {
        match value {
            CheckedBooleanOperation::And => Self::And,
            CheckedBooleanOperation::Or => Self::Or,
            CheckedBooleanOperation::ExclusiveOr => Self::ExclusiveOr,
            CheckedBooleanOperation::Not => Self::Not,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrConstant {
    Unit,
    Bool(bool),
    Integer { ty: IrType, bits: u64 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrTrapSite {
    pub(crate) rule_id: &'static str,
    pub(crate) message: String,
    pub(crate) function: String,
    pub(crate) node_path: Vec<u32>,
}

impl From<TrapSite> for IrTrapSite {
    fn from(value: TrapSite) -> Self {
        Self {
            rule_id: value.rule_id,
            message: value.message,
            function: value.function,
            node_path: value.node_path.components().to_vec(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrOperation {
    Constant(IrConstant),
    Call {
        function: u32,
        arguments: Vec<IrValueId>,
    },
    Integer {
        operation: IrIntegerOperation,
        operand_type: IrType,
        arguments: Vec<IrValueId>,
        trap: Option<IrTrapSite>,
    },
    Boolean {
        operation: IrBooleanOperation,
        arguments: Vec<IrValueId>,
    },
    EnumEquality {
        equal: bool,
        operand_type: IrType,
        arguments: [IrValueId; 2],
    },
    ConstructStruct {
        nominal: IrNominalId,
        fields: Vec<IrValueId>,
    },
    ConstructEnum {
        nominal: IrNominalId,
        variant: u32,
        fields: Vec<IrValueId>,
    },
    ProjectStruct {
        aggregate: IrValueId,
        nominal: IrNominalId,
        field: u32,
        consume_root: bool,
    },
    InsertStruct {
        aggregate: IrValueId,
        nominal: IrNominalId,
        field: u32,
        value: IrValueId,
    },
    ProjectVariant {
        aggregate: IrValueId,
        nominal: IrNominalId,
        variant: u32,
        field: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrInstruction {
    Define {
        result: IrValueId,
        ty: IrType,
        operation: IrOperation,
    },
    Check {
        condition: IrValueId,
        trap: IrTrapSite,
    },
    Drop(IrDrop),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrDrop {
    value: IrValueId,
    ty: IrType,
}

impl IrDrop {
    pub const fn value(self) -> IrValueId {
        self.value
    }

    pub const fn ty(self) -> IrType {
        self.ty
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrMatchTarget {
    tag: u32,
    block: IrBlockId,
}

impl IrMatchTarget {
    pub const fn tag(self) -> u32 {
        self.tag
    }

    pub const fn block(self) -> IrBlockId {
        self.block
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrTerminator {
    Jump {
        target: IrBlockId,
        arguments: Vec<IrValueId>,
        drops: Vec<IrDrop>,
    },
    Match {
        scrutinee: IrValueId,
        enum_type: IrEnumType,
        targets: Vec<IrMatchTarget>,
    },
    Return {
        value: IrValueId,
        drops: Vec<IrDrop>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrBlock {
    parameters: Vec<(IrValueId, IrType)>,
    instructions: Vec<IrInstruction>,
    terminator: IrTerminator,
}

impl IrBlock {
    pub fn parameters(&self) -> &[(IrValueId, IrType)] {
        &self.parameters
    }

    pub fn instructions(&self) -> &[IrInstruction] {
        &self.instructions
    }

    pub const fn terminator(&self) -> &IrTerminator {
        &self.terminator
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrFunction {
    name: String,
    parameters: Vec<(IrValueId, IrType)>,
    result: IrType,
    values: Vec<IrType>,
    blocks: Vec<IrBlock>,
}

impl IrFunction {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn result(&self) -> IrType {
        self.result
    }

    pub fn parameters(&self) -> &[(IrValueId, IrType)] {
        &self.parameters
    }

    pub fn blocks(&self) -> &[IrBlock] {
        &self.blocks
    }

    pub(crate) fn value_type(&self, value: IrValueId) -> Option<IrType> {
        self.values.get(value.index()).copied()
    }
}

#[derive(Debug)]
pub struct IrProgram<'classified, 'lexed, 'source> {
    _checked: CheckedProgram<'classified, 'lexed, 'source>,
    nominals: Vec<IrNominal>,
    functions: Vec<IrFunction>,
    main: u32,
}

impl IrProgram<'_, '_, '_> {
    pub fn nominals(&self) -> &[IrNominal] {
        &self.nominals
    }

    pub fn nominal(&self, id: IrNominalId) -> Option<&IrNominal> {
        self.nominals.get(id.index())
    }

    pub fn functions(&self) -> &[IrFunction] {
        &self.functions
    }

    pub const fn main_ordinal(&self) -> u32 {
        self.main
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoweringFailure {
    InvalidCheckedProgram,
    CounterOverflow,
}

mod builder;

pub use builder::lower_checked_v0_14;
