//! Target-independent lowering from the semantically checked active Whitefoot specification.
//!
//! The private IR records exact value types, nominal construction/projection,
//! direct calls, retained checks, and explicit control-flow edges. It performs
//! no source admission, label lookup, exhaustiveness decision, or ownership
//! judgment.

use crate::semantic::{
    CheckedBooleanOperation, CheckedEnumType, CheckedFlatElement, CheckedFloatOperation,
    CheckedIntegerOperation, CheckedLayoutCeiling, CheckedLayoutMagnitude, CheckedNumericType,
    CheckedProgram, CheckedRuntimeTargetObligations, CheckedTargetDomainObligation, CheckedType,
    TrapSite,
};
use crate::{SystemRelease, SystemResourceContract};

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
    Integer {
        width: u8,
        signed: bool,
    },
    Float {
        width: u8,
    },
    TagOnlyNominal(IrNominalId),
    /// One affine aggregate element: a non-copy nominal stored by value.
    /// Only `buffer` element positions carry this variant [TYPE-2].
    Nominal(IrNominalId),
}

impl IrFlatElement {
    pub const fn ty(self) -> IrType {
        match self {
            Self::Unit => IrType::Unit,
            Self::Bool => IrType::Bool,
            Self::Integer { width, signed } => IrType::Integer { width, signed },
            Self::Float { width } => IrType::Float { width },
            Self::TagOnlyNominal(id) | Self::Nominal(id) => IrType::Nominal(id),
        }
    }
}

/// The referent of an [`IrType::Address`]: directly stored content that a
/// borrow addresses.
///
/// Descriptor values (`buffer`, `slice`) and opaque handles (`box`, system
/// resources) are already their own borrow and never appear here.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrAddressed {
    Unit,
    Bool,
    Integer { width: u8, signed: bool },
    Float { width: u8 },
    Nominal(IrNominalId),
}

impl IrAddressed {
    pub const fn ty(self) -> IrType {
        match self {
            Self::Unit => IrType::Unit,
            Self::Bool => IrType::Bool,
            Self::Integer { width, signed } => IrType::Integer { width, signed },
            Self::Float { width } => IrType::Float { width },
            Self::Nominal(id) => IrType::Nominal(id),
        }
    }

    const fn of(ty: IrType) -> Option<Self> {
        Some(match ty {
            IrType::Unit => Self::Unit,
            IrType::Bool => Self::Bool,
            IrType::Integer { width, signed } => Self::Integer { width, signed },
            IrType::Float { width } => Self::Float { width },
            IrType::Nominal(id) => Self::Nominal(id),
            IrType::Address(_)
            | IrType::Array { .. }
            | IrType::Buffer { .. }
            | IrType::Slice { .. } => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum IrType {
    Unit,
    Bool,
    Integer { width: u8, signed: bool },
    Float { width: u8 },
    Nominal(IrNominalId),
    Address(IrAddressed),
    Array { element: IrFlatElement, length: u64 },
    Buffer { element: IrFlatElement },
    Slice { element: IrFlatElement },
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
        CheckedFlatElement::GenericFloat(_) => {
            return Err(LoweringFailure::InvalidCheckedProgram);
        }
        CheckedFlatElement::TagOnlyNominal(id) => IrFlatElement::TagOnlyNominal(IrNominalId(id.0)),
        CheckedFlatElement::Nominal(id) => IrFlatElement::Nominal(IrNominalId(id.0)),
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
        CheckedType::Generic(_) | CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => {
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
    Struct {
        fields: Vec<IrField>,
    },
    Enum {
        variants: Vec<IrVariant>,
    },
    Box {
        referent: IrType,
    },
    /// One `arena<'r, T>` instance: a pointer-shaped handle to region-owned
    /// heap content, released with its region rather than with an owner
    /// scope [STOR-3, STOR-4].
    Arena {
        content: IrType,
    },
    /// One region block's compiler-owned arena allocation-list cell; its
    /// drop walks and frees every registered allocation [STOR-3].
    ArenaStorage,
    /// One [SYS-2] opaque system resource type. It has no field, variant, or
    /// source-visible content: its identity is the target-independent
    /// semantic identity [QUAL-1] the contract carries, together with the
    /// [SYS-5] release action, that action's row, and the [HOST-3] backing
    /// class. Every use of a value of this type — a move, a `match` binder, a
    /// struct or enum field, a return, or a call argument — keeps that
    /// identity, because the type is what fixes the release action.
    SystemResource(SystemResourceContract),
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
    AddExact,
    SubtractExact,
    MultiplyExact,
    AddDefined,
    SubtractDefined,
    MultiplyDefined,
    AddChecked,
    SubtractChecked,
    MultiplyChecked,
    DivideChecked,
    RemainderChecked,
    DivideExact,
    RemainderExact,
    DivideDefined,
    RemainderDefined,
    AbsoluteWrap,
    AbsoluteExact,
    AbsoluteDefined,
    AbsoluteChecked,
    NegateWrap,
    NegateExact,
    NegateDefined,
    NegateChecked,
    BitAnd,
    BitOr,
    BitXor,
    BitNot,
    ShiftLeftWrap,
    ShiftRightWrap,
    ShiftLeftExact,
    ShiftRightExact,
    ShiftLeftDefined,
    ShiftRightDefined,
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
            CheckedIntegerOperation::AddExact => Self::AddExact,
            CheckedIntegerOperation::SubtractExact => Self::SubtractExact,
            CheckedIntegerOperation::MultiplyExact => Self::MultiplyExact,
            CheckedIntegerOperation::AddDefined => Self::AddDefined,
            CheckedIntegerOperation::SubtractDefined => Self::SubtractDefined,
            CheckedIntegerOperation::MultiplyDefined => Self::MultiplyDefined,
            CheckedIntegerOperation::AddChecked => Self::AddChecked,
            CheckedIntegerOperation::SubtractChecked => Self::SubtractChecked,
            CheckedIntegerOperation::MultiplyChecked => Self::MultiplyChecked,
            CheckedIntegerOperation::DivideChecked => Self::DivideChecked,
            CheckedIntegerOperation::RemainderChecked => Self::RemainderChecked,
            CheckedIntegerOperation::DivideExact => Self::DivideExact,
            CheckedIntegerOperation::RemainderExact => Self::RemainderExact,
            CheckedIntegerOperation::DivideDefined => Self::DivideDefined,
            CheckedIntegerOperation::RemainderDefined => Self::RemainderDefined,
            CheckedIntegerOperation::AbsoluteWrap => Self::AbsoluteWrap,
            CheckedIntegerOperation::AbsoluteExact => Self::AbsoluteExact,
            CheckedIntegerOperation::AbsoluteDefined => Self::AbsoluteDefined,
            CheckedIntegerOperation::AbsoluteChecked => Self::AbsoluteChecked,
            CheckedIntegerOperation::NegateWrap => Self::NegateWrap,
            CheckedIntegerOperation::NegateExact => Self::NegateExact,
            CheckedIntegerOperation::NegateDefined => Self::NegateDefined,
            CheckedIntegerOperation::NegateChecked => Self::NegateChecked,
            CheckedIntegerOperation::BitAnd => Self::BitAnd,
            CheckedIntegerOperation::BitOr => Self::BitOr,
            CheckedIntegerOperation::BitXor => Self::BitXor,
            CheckedIntegerOperation::BitNot => Self::BitNot,
            CheckedIntegerOperation::ShiftLeftWrap => Self::ShiftLeftWrap,
            CheckedIntegerOperation::ShiftRightWrap => Self::ShiftRightWrap,
            CheckedIntegerOperation::ShiftLeftExact => Self::ShiftLeftExact,
            CheckedIntegerOperation::ShiftRightExact => Self::ShiftRightExact,
            CheckedIntegerOperation::ShiftLeftDefined => Self::ShiftLeftDefined,
            CheckedIntegerOperation::ShiftRightDefined => Self::ShiftRightDefined,
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
    /// One struct-typed rodata constant [CONST-2 candidate]: complete field
    /// values in declared order.
    Struct(Vec<IrGlobalValue>),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrLayoutMagnitude {
    Finite(u64),
    AboveU64,
}

impl IrLayoutMagnitude {
    pub(crate) const fn permits(self, actual: u64) -> bool {
        match self {
            Self::Finite(ceiling) => actual <= ceiling,
            Self::AboveU64 => true,
        }
    }
}

impl From<CheckedLayoutMagnitude> for IrLayoutMagnitude {
    fn from(value: CheckedLayoutMagnitude) -> Self {
        match value {
            CheckedLayoutMagnitude::Finite(value) => Self::Finite(value),
            CheckedLayoutMagnitude::AboveU64 => Self::AboveU64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrLayoutCeiling {
    pub size: IrLayoutMagnitude,
    pub align: u64,
    pub stride: IrLayoutMagnitude,
}

impl From<CheckedLayoutCeiling> for IrLayoutCeiling {
    fn from(value: CheckedLayoutCeiling) -> Self {
        Self {
            size: value.size.into(),
            align: value.align,
            stride: value.stride.into(),
        }
    }
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

/// The target-independent semantic identity of one [SYS-2] system operation
/// [QUAL-1].
///
/// It is the operation's index in the specification's own inventory table, so
/// no source function name or spelling, logical path, project, corpus, test,
/// or signature lookalike can select one. A target stage maps this identity
/// to one approved implementation and one private ABI symbol; the identity
/// itself names no target.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct IrSystemOperation(u8);

impl IrSystemOperation {
    // Read by the target stage that maps this identity through the [QUAL-1]
    // qualification table; nothing before that stage may dispatch on it.
    #[allow(dead_code)]
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrOperation {
    Constant(IrConstant),
    Call {
        function: u32,
        arguments: Vec<IrValueId>,
    },
    /// One call to a [SYS-2] system operation, by semantic identity, with its
    /// value arguments in declared parameter order.
    SystemCall {
        operation: IrSystemOperation,
        arguments: Vec<IrValueId>,
    },
    Integer {
        operation: IrIntegerOperation,
        operand_type: IrType,
        arguments: Vec<IrValueId>,
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
    /// One discharged source subscript read [OP-4]: the checker has already
    /// derived the bounds obligation, so no runtime branch is emitted in any
    /// build mode; the offset is consumed directly.
    ArrayIndex {
        root: IrArrayRoot,
        offset: IrValueId,
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
        layout_ceiling: IrLayoutCeiling,
        target_domains: IrRuntimeTargetObligations,
    },
    /// One `buffer_vacant<T>(n)` allocation [OP-1, OP-9]: the defined value's
    /// buffer type names the `Option<T>` element instance, and every element
    /// is initialized to the compiler-minted `None()` of that instance.
    BufferVacant {
        length: IrValueId,
        layout_ceiling: IrLayoutCeiling,
        target_domains: IrRuntimeTargetObligations,
    },
    BufferFits {
        length: IrValueId,
        maximum_length: u64,
    },
    BufferLength {
        buffer: IrValueId,
    },
    /// One discharged source subscript read [OP-4]; see [`Self::ArrayIndex`].
    BufferIndex {
        buffer: IrValueId,
        offset: IrValueId,
        target_domain: IrTargetDomainObligation,
    },
    /// One check-aware wide-probe step over a `u8` buffer.
    ///
    /// Computes how many upcoming iterations of a recognized byte-walk loop
    /// are provably no-ops: the count of leading bytes at `index ..` that
    /// match no needle, but only when `index + 16 <= min(limit, length)`
    /// bounds both the walk's exit guard and every skipped read; otherwise 0.
    /// Every byte at which anything observable can happen — a needle hit,
    /// the exit bound, or any retained trap such as a `check` or `claim` —
    /// therefore reaches the unchanged scalar body and its own [DIAG-3]
    /// record. The probe itself never traps and never reports; it reads only
    /// bytes its internal guard proves in bounds.
    BufferProbeSkip {
        buffer: IrValueId,
        index: IrValueId,
        limit: IrValueId,
        needles: Vec<IrValueId>,
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
    /// One discharged source subscript read [OP-4]; see [`Self::ArrayIndex`].
    SliceIndex {
        slice: IrValueId,
        offset: IrValueId,
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
    /// One region block's arena allocation-list cell, materialized at region
    /// entry: a stack cell reset to empty, whose address is the operation's
    /// value [STOR-2, STOR-3].
    ArenaListNew,
    /// One `arena_new` allocation: heap storage for the content, registered
    /// on the owning region's allocation list so the region's exit release
    /// frees it [STOR-2, STOR-3, STOR-4]. The value is the content address.
    ArenaNew {
        nominal: IrNominalId,
        list: IrValueId,
        value: IrValueId,
    },
    /// Arena content read through explicit `deref` [STOR-2].
    ArenaDeref {
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
    AddressOf {
        value: IrValueId,
        referent: IrAddressed,
    },
    Load {
        address: IrValueId,
        referent: IrAddressed,
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
    Store {
        address: IrValueId,
        value: IrValueId,
        referent: IrAddressed,
    },
    Drop(IrDrop),
}

/// One compiler-derived release, explicit on the normal control-flow edge
/// that carries it [STOR-3].
///
/// Every drop and every release is represented before lowering, and release
/// actions run only on normal edges: a trap runs none [TRAP-1, EFF-4]. The IR
/// therefore places these records only on `Jump` and `Return` terminators and
/// as `Drop` instructions in straight-line position, never on a trapping
/// `Check`. Their order inside one edge is the checked program's reverse
/// declaration order, and their position relative to surrounding calls is the
/// order [EFF-5] requires of every conforming lowering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrDrop {
    value: IrValueId,
    ty: IrType,
    release: SystemRelease,
}

impl IrDrop {
    pub const fn value(self) -> IrValueId {
        self.value
    }

    pub const fn ty(self) -> IrType {
        self.ty
    }

    /// The exact [SYS-5] release this drop performs: the released value's own
    /// action when it is one system resource, together with the union of the
    /// rows of every system release it may run over owned content.
    pub const fn release(self) -> SystemRelease {
        self.release
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

    /// Every defined value's type, for whole-program type enumeration.
    pub(crate) fn value_types(&self) -> &[IrType] {
        &self.values
    }

    pub(crate) fn value_type(&self, value: IrValueId) -> Option<IrType> {
        self.values.get(value.index()).copied()
    }
}

/// One definition in the private straight-line program-start requirement.
///
/// This is deliberately not a general control-flow function: the entry
/// wrapper owns its source parameters until the final Bool is known, and the
/// closed FN-8 operation subset can only define immutable SSA values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IrEntryGoalDefinition {
    result: IrValueId,
    ty: IrType,
    operation: IrOperation,
}

impl IrEntryGoalDefinition {
    pub(crate) const fn result(&self) -> IrValueId {
        self.result
    }

    pub(crate) const fn ty(&self) -> IrType {
        self.ty
    }

    pub(crate) const fn operation(&self) -> &IrOperation {
        &self.operation
    }
}

/// The one retained [FN-8] goal evaluated by the compiler-owned entry
/// wrapper [PROG-3]. Inputs are the source `main` parameters in declaration
/// order; definitions form one dense, straight-line SSA sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IrEntryGoal {
    inputs: Vec<(IrValueId, IrType)>,
    values: Vec<IrType>,
    definitions: Vec<IrEntryGoalDefinition>,
    condition: IrValueId,
    trap: IrTrapSite,
}

impl IrEntryGoal {
    pub(crate) fn inputs(&self) -> &[(IrValueId, IrType)] {
        &self.inputs
    }

    pub(crate) fn definitions(&self) -> &[IrEntryGoalDefinition] {
        &self.definitions
    }

    pub(crate) const fn condition(&self) -> IrValueId {
        self.condition
    }

    pub(crate) const fn trap(&self) -> &IrTrapSite {
        &self.trap
    }

    pub(crate) fn ty(&self, value: IrValueId) -> Option<IrType> {
        self.values.get(value.index()).copied()
    }
}

/// One retained conservative alias link between two of the entry's
/// standard-input resource owners, by [FN-7] table ordinal.
///
/// [SYS-12] fixes exactly one for the first slice: redirection may make the
/// `command.stdout` and `command.stderr` owners the same sink. v0.18 defines
/// no consumer, so nothing here reads it; it is retained so a later verified
/// cross-resource reordering fact fails closed on this pair rather than
/// treating two separate `Output` owners as disjoint sinks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrResourceAlias {
    left: u8,
    right: u8,
}

impl IrResourceAlias {
    // Deliberately unread: v0.18 defines no consumer of the may-alias fact.
    // The pair is retained so a later cross-resource reordering fact must
    // read it and fail closed rather than inferring separateness.
    #[allow(dead_code)]
    #[must_use]
    pub const fn left(self) -> u8 {
        self.left
    }

    #[allow(dead_code)]
    #[must_use]
    pub const fn right(self) -> u8 {
        self.right
    }
}

/// The [FN-7] entry form the program starts with [PROG-3].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrEntry {
    /// The unlabelled entry: no supplied input and no produced status.
    Unlabelled,
    /// A `command` entry: program start supplies exactly the standard inputs
    /// these table ordinals select, in this order, and maps the returned
    /// `ExitStatus` to the host process status.
    Command {
        inputs: Vec<u8>,
        aliases: Vec<IrResourceAlias>,
    },
}

#[derive(Debug)]
pub struct IrProgram<'classified, 'lexed, 'source> {
    _checked: CheckedProgram<'classified, 'lexed, 'source>,
    nominals: Vec<IrNominal>,
    constants: Vec<IrGlobalConstant>,
    functions: Vec<IrFunction>,
    main: u32,
    entry: IrEntry,
    entry_goal: Option<IrEntryGoal>,
}

impl IrProgram<'_, '_, '_> {
    pub fn nominals(&self) -> &[IrNominal] {
        &self.nominals
    }

    /// The entry form program start must implement.
    pub const fn entry(&self) -> &IrEntry {
        &self.entry
    }

    pub(crate) const fn entry_goal(&self) -> Option<&IrEntryGoal> {
        self.entry_goal.as_ref()
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

#[cfg(test)]
mod tests;

pub use builder::lower_checked;
