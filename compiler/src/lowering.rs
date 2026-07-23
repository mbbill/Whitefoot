//! Target-independent lowering from the semantically checked active Whitefoot specification.
//!
//! The private IR records exact value types, nominal construction/projection,
//! direct calls, retained checks, and explicit control-flow edges. It performs
//! no source admission, label lookup, exhaustiveness decision, or ownership
//! judgment.

use crate::semantic::{
    CheckedBooleanOperation, CheckedEnumType, CheckedFlatElement, CheckedFloatOperation,
    CheckedIntegerOperation, CheckedNumericType, CheckedProgram, CheckedRuntimeTargetObligations,
    CheckedTargetDomainObligation, CheckedType, TrapSite,
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
pub struct IrConstantId(u32);

impl IrConstantId {
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.0
    }

    const fn index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrFlatElement {
    Unit,
    Bool,
    Integer { width: u8, signed: bool },
    Float { width: u8 },
    TagOnlyNominal(IrNominalId),
}

impl IrFlatElement {
    pub const fn ty(self) -> IrType {
        match self {
            Self::Unit => IrType::Unit,
            Self::Bool => IrType::Bool,
            Self::Integer { width, signed } => IrType::Integer { width, signed },
            Self::Float { width } => IrType::Float { width },
            Self::TagOnlyNominal(id) => IrType::Nominal(id),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrType {
    Unit,
    Bool,
    Integer { width: u8, signed: bool },
    Float { width: u8 },
    Nominal(IrNominalId),
    NominalAddress(IrNominalId),
    Array { element: IrFlatElement, length: u64 },
    Buffer { element: IrFlatElement },
    Slice { element: IrFlatElement },
    GuardedArrayIndex { length: u64 },
    GuardedBufferIndex { element: IrFlatElement },
}

const fn lower_flat_element(value: CheckedFlatElement) -> Result<IrFlatElement, LoweringFailure> {
    Ok(match value {
        CheckedFlatElement::Unit => IrFlatElement::Unit,
        CheckedFlatElement::Bool => IrFlatElement::Bool,
        CheckedFlatElement::Integer(integer) => IrFlatElement::Integer {
            width: integer.width(),
            signed: integer.signed(),
        },
        CheckedFlatElement::Float(float) => IrFlatElement::Float {
            width: float.width(),
        },
        CheckedFlatElement::GenericInt(_) => {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        CheckedFlatElement::TagOnlyNominal(id) => IrFlatElement::TagOnlyNominal(IrNominalId(id.0)),
    })
}

fn lower_type(value: CheckedType) -> Result<IrType, LoweringFailure> {
    Ok(match value {
        CheckedType::Unit => IrType::Unit,
        CheckedType::Bool => IrType::Bool,
        CheckedType::Integer(integer) => IrType::Integer {
            width: integer.width(),
            signed: integer.signed(),
        },
        CheckedType::Float(float) => IrType::Float {
            width: float.width(),
        },
        CheckedType::Generic(_) | CheckedType::GenericInt(_) => {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        CheckedType::Nominal(id) => IrType::Nominal(IrNominalId(id.0)),
        CheckedType::Array { element, length } => IrType::Array {
            element: lower_flat_element(element)?,
            length: length
                .value()
                .ok_or(LoweringFailure::InvalidCheckedProgram)?,
        },
        CheckedType::Buffer { element } => IrType::Buffer {
            element: lower_flat_element(element)?,
        },
        CheckedType::Slice { element, .. } => IrType::Slice {
            element: lower_flat_element(element)?,
        },
    })
}

const fn lower_numeric_type(value: CheckedNumericType) -> IrType {
    match value {
        CheckedNumericType::Integer(integer) => IrType::Integer {
            width: integer.width(),
            signed: integer.signed(),
        },
        CheckedNumericType::Float(float) => IrType::Float {
            width: float.width(),
        },
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
    Box { referent: IrType },
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrFloatOperation {
    AddStrict,
    SubtractStrict,
    MultiplyStrict,
    DivideStrict,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    NotEqual,
    Negate,
    Absolute,
    CopySign,
    Minimum,
    Maximum,
    Floor,
    Ceil,
    Truncate,
    RoundEven,
    Remainder,
    SquareRootStrict,
    FusedMultiplyAddStrict,
    Infinity,
    Nan,
}

impl From<CheckedFloatOperation> for IrFloatOperation {
    fn from(value: CheckedFloatOperation) -> Self {
        match value {
            CheckedFloatOperation::AddStrict => Self::AddStrict,
            CheckedFloatOperation::SubtractStrict => Self::SubtractStrict,
            CheckedFloatOperation::MultiplyStrict => Self::MultiplyStrict,
            CheckedFloatOperation::DivideStrict => Self::DivideStrict,
            CheckedFloatOperation::Equal => Self::Equal,
            CheckedFloatOperation::Less => Self::Less,
            CheckedFloatOperation::LessEqual => Self::LessEqual,
            CheckedFloatOperation::Greater => Self::Greater,
            CheckedFloatOperation::GreaterEqual => Self::GreaterEqual,
            CheckedFloatOperation::NotEqual => Self::NotEqual,
            CheckedFloatOperation::Negate => Self::Negate,
            CheckedFloatOperation::Absolute => Self::Absolute,
            CheckedFloatOperation::CopySign => Self::CopySign,
            CheckedFloatOperation::Minimum => Self::Minimum,
            CheckedFloatOperation::Maximum => Self::Maximum,
            CheckedFloatOperation::Floor => Self::Floor,
            CheckedFloatOperation::Ceil => Self::Ceil,
            CheckedFloatOperation::Truncate => Self::Truncate,
            CheckedFloatOperation::RoundEven => Self::RoundEven,
            CheckedFloatOperation::Remainder => Self::Remainder,
            CheckedFloatOperation::SquareRootStrict => Self::SquareRootStrict,
            CheckedFloatOperation::FusedMultiplyAddStrict => Self::FusedMultiplyAddStrict,
            CheckedFloatOperation::Infinity => Self::Infinity,
            CheckedFloatOperation::Nan => Self::Nan,
        }
    }
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
    Float { ty: IrType, bits: u64 },
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
pub enum IrGlobalValue {
    Scalar(IrConstant),
    Array(Vec<IrConstant>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrGlobalConstant {
    id: IrConstantId,
    name: String,
    ty: IrType,
    value: IrGlobalValue,
}

impl IrGlobalConstant {
    pub const fn id(&self) -> IrConstantId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn ty(&self) -> IrType {
        self.ty
    }

    pub const fn value(&self) -> &IrGlobalValue {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrArrayRoot {
    Value(IrValueId),
    Constant(IrConstantId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrTargetDomainObligation {
    RuntimeSizedAllocation,
    ElementAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrRuntimeTargetObligations {
    allocation: IrTargetDomainObligation,
    element_address: IrTargetDomainObligation,
}

impl From<CheckedRuntimeTargetObligations> for IrRuntimeTargetObligations {
    fn from(value: CheckedRuntimeTargetObligations) -> Self {
        Self {
            allocation: value.allocation().into(),
            element_address: value.element_address().into(),
        }
    }
}

impl IrRuntimeTargetObligations {
    pub(crate) const fn is_complete(self) -> bool {
        matches!(
            (self.allocation, self.element_address),
            (
                IrTargetDomainObligation::RuntimeSizedAllocation,
                IrTargetDomainObligation::ElementAddress
            )
        )
    }
}

impl From<CheckedTargetDomainObligation> for IrTargetDomainObligation {
    fn from(value: CheckedTargetDomainObligation) -> Self {
        match value {
            CheckedTargetDomainObligation::RuntimeSizedAllocation => Self::RuntimeSizedAllocation,
            CheckedTargetDomainObligation::ElementAddress => Self::ElementAddress,
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
    Float {
        operation: IrFloatOperation,
        operand_type: IrType,
        arguments: Vec<IrValueId>,
    },
    NumericConversion {
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
    },
    Reinterpret {
        source_type: IrType,
        destination_type: IrType,
        value: IrValueId,
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
    ArrayFill {
        value: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    ArrayIndex {
        root: IrArrayRoot,
        offset: IrValueId,
        trap: IrTrapSite,
        target_domain: IrTargetDomainObligation,
    },
    ArrayBoundsCheck {
        offset: IrValueId,
        trap: IrTrapSite,
        target_domain: IrTargetDomainObligation,
    },
    InsertArray {
        aggregate: IrValueId,
        index: IrValueId,
        value: IrValueId,
    },
    BufferFill {
        length: IrValueId,
        value: IrValueId,
        trap: IrTrapSite,
        target_domains: IrRuntimeTargetObligations,
    },
    BufferLength {
        buffer: IrValueId,
    },
    BufferIndex {
        buffer: IrValueId,
        offset: IrValueId,
        trap: IrTrapSite,
        target_domain: IrTargetDomainObligation,
    },
    BufferBoundsCheck {
        buffer: IrValueId,
        offset: IrValueId,
        trap: IrTrapSite,
        target_domain: IrTargetDomainObligation,
    },
    SliceFromArray {
        array: IrArrayRoot,
    },
    SliceFromBuffer {
        buffer: IrValueId,
    },
    SliceLength {
        slice: IrValueId,
    },
    SliceIndex {
        slice: IrValueId,
        offset: IrValueId,
        trap: IrTrapSite,
        target_domain: IrTargetDomainObligation,
    },
    BoxNew {
        nominal: IrNominalId,
        value: IrValueId,
    },
    BoxDeref {
        nominal: IrNominalId,
        value: IrValueId,
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
    AddressOfNominal {
        value: IrValueId,
        nominal: IrNominalId,
    },
    LoadNominal {
        address: IrValueId,
        nominal: IrNominalId,
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
    StoreBuffer {
        buffer: IrValueId,
        index: IrValueId,
        value: IrValueId,
    },
    StoreNominal {
        address: IrValueId,
        value: IrValueId,
        nominal: IrNominalId,
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

    pub(crate) fn contains_buffer(&self) -> bool {
        self.values
            .iter()
            .any(|ty| matches!(ty, IrType::Buffer { .. }))
    }

    pub(crate) fn value_type(&self, value: IrValueId) -> Option<IrType> {
        self.values.get(value.index()).copied()
    }
}

#[derive(Debug)]
pub struct IrProgram<'classified, 'lexed, 'source> {
    _checked: CheckedProgram<'classified, 'lexed, 'source>,
    nominals: Vec<IrNominal>,
    constants: Vec<IrGlobalConstant>,
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

    pub fn constants(&self) -> &[IrGlobalConstant] {
        &self.constants
    }

    pub fn constant(&self, id: IrConstantId) -> Option<&IrGlobalConstant> {
        self.constants.get(id.index())
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

pub use builder::lower_checked;
