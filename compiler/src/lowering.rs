//! Target-independent lowering from the semantically checked active Whitefoot specification.
//!
//! The private IR records exact value types, nominal construction/projection,
//! direct calls, retained claims, and explicit control-flow edges. It performs
//! no source admission, label lookup, exhaustiveness decision, or ownership
//! judgment.

use crate::semantic::{
    CheckedBooleanOperation, CheckedEnumType, CheckedFlatElement, CheckedFloatOperation,
    CheckedIntegerOperation, CheckedLayoutCeiling, CheckedLayoutMagnitude, CheckedNumericType,
    CheckedProgram, CheckedRuntimeTargetObligations, CheckedTargetDomainObligation, CheckedType,
    ClaimSite,
};
use crate::{SystemRelease, SystemResourceContract};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

/// The declaration family that gave one nominal its identity before lowering.
///
/// LLVM shape is deliberately not type identity: a source enum, a prelude
/// `Result`, and a system outcome can have identical fields while remaining
/// non-interchangeable. The backend retains this compact origin so its
/// target-facing signature checks fail closed on malformed IR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrNominalIdentity {
    /// A source nominal or a prelude nominal other than `Result`.
    Ordinary,
    /// One concrete prelude `Result<T, E>` instance.
    PreludeResult,
    /// One exact [SYS-2] nominal-table row.
    System(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrNominal {
    id: IrNominalId,
    identity: IrNominalIdentity,
    kind: IrNominalKind,
}

impl IrNominal {
    pub const fn id(&self) -> IrNominalId {
        self.id
    }

    /// The declaration family retained independently of representation.
    pub const fn identity(&self) -> IrNominalIdentity {
        self.identity
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
pub struct IrClaimSite {
    pub(crate) rule_id: &'static str,
    pub(crate) message: String,
    pub(crate) function: String,
    pub(crate) node_path: Vec<u32>,
}

impl From<ClaimSite> for IrClaimSite {
    fn from(value: ClaimSite) -> Self {
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
        /// Compiler-owned execution and completion contract.
        target_action: crate::TargetAction,
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
    /// One claim-aware wide-probe step over a `u8` buffer.
    ///
    /// Computes how many upcoming iterations of a recognized byte-walk loop
    /// are provably no-ops: the count of leading bytes at `index ..` that
    /// match no needle, but only when `index + 16 <= min(limit, length)`
    /// bounds both the walk's exit guard and every skipped read; otherwise 0.
    /// Every byte at which anything observable can happen — a needle hit,
    /// the exit bound, or any retained `claim` —
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
    /// One permitted counted loop [PAR-2 candidate], actualized as a recursive
    /// split of its index range.
    ///
    /// The whole loop is one instruction here because it has exactly two
    /// renderings and the choice between them is the world, not the source. The
    /// overlapped world asks the runtime what a split of this span may afford
    /// and calls `splitter`; the sequential world calls `chunk`, which *is* the
    /// loop, so that world runs the code the loop always had. Both take the
    /// accumulator's incoming value as their first argument and return the
    /// whole fold, so the site has nothing to recombine and the two renderings
    /// are one call each.
    ///
    /// `splitter` and `chunk` are ordinary synthesized [`IrFunction`]s: the
    /// splitter's two recursive calls are one ordinary overlap group, so the
    /// hand-out, the thunk, the deque, and the join are the ones every other
    /// permitted pair uses.
    LoopSplit {
        splitter: u32,
        chunk: u32,
        /// The accumulator's value on entry. It folds into the leftmost chunk,
        /// which is what keeps the fold's leaf order the source's own.
        seed: IrValueId,
        lower: IrValueId,
        upper: IrValueId,
        /// The values the body reads from the enclosing scope, in the order the
        /// two synthesized functions declare them.
        captures: Vec<IrValueId>,
        /// The static cost estimate of one iteration, which the runtime
        /// allowance multiplies by the span. A cost over the emitted IR, never
        /// a name, a signature, or a source shape.
        weight: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrInstruction {
    Define {
        result: IrValueId,
        ty: IrType,
        operation: IrOperation,
    },
    Claim {
        condition: IrValueId,
        site: IrClaimSite,
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
/// `Claim`. Their order inside one edge is the checked program's reverse
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
    /// A checker-proved uninhabited function body. The function keeps its
    /// ordinary ABI but has no source-derived executable path.
    Unreachable,
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

/// Whether lowering actualizes the permission judgment's overlap groups.
///
/// The judgment itself is pure and always runs: `--par-ledger` reports the same
/// verdicts either way, and no accepted program changes. This selects only
/// whether a permitted group reaches the IR as an overlap group, and therefore
/// whether the backend outlines a call, offers a lane, and joins it.
///
/// `Completion` is the shipped default: it actualizes only compiler-owned
/// finite target operations and leaves pure compute output byte-identical to
/// `Off`. Compute outlining remains opt-in because it is not free. The batch
/// audit measured that lowering alone — no runtime linked, `WF_WORKERS` unset —
/// at about 1.2x on the layout demo and 2.1x on `fib(38)`: an outlined call
/// passes its arguments through a memory frame, is reached through a function
/// pointer, and so cannot be inlined. `Off` remains only the exact sequential
/// reference; `On` adds eligible compute groups to the default completion set.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlapLowering {
    /// Consult no permission group. Retained for exact sequential-reference
    /// tests; this is not the shipped default.
    Off,
    /// Actualize only direct finite target operations. Pure compute output is
    /// therefore byte-identical to `Off`, while completion I/O needs no flag.
    #[default]
    Completion,
    /// Actualize completion operations and eligible compute groups.
    On,
}

/// One group of sibling calls whose evaluations may be overlapped [PAR-1
/// candidate].
///
/// The members are the values those calls define, in source order, all in one
/// block of one function. A compute group may hand out every member but the
/// last. A supported completion group reserves bounded storage all-or-none and
/// dispatches every member, including the source-last one. Every dispatched member
/// is joined at the last definition before any value use or block exit.
///
/// The group is a permission the target stage may take, never an obligation:
/// a target that hands nothing out emits exactly the sequential code, because
/// the handed-out call and the inline fallback call the same monomorphized
/// function on the same arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrOverlap {
    members: Vec<IrValueId>,
    ordered_attribution: Option<crate::SystemAuthorityAttribution>,
    dispatch_last: bool,
}

impl IrOverlap {
    /// Every source member in order, including the source-last join site.
    pub fn members(&self) -> &[IrValueId] {
        &self.members
    }

    /// The value whose definition is the group's join site: the last member,
    /// which runs on the calling thread.
    pub fn join_site(&self) -> Option<IrValueId> {
        self.members.last().copied()
    }

    /// The members that may be handed to a worker lane, in source order.
    pub fn handed_out(&self) -> &[IrValueId] {
        self.members
            .split_last()
            .map_or(&[][..], |(_, earlier)| earlier)
    }

    /// Every member dispatched before the join. Ordered completion groups
    /// include the source-last member because it may not run ahead inline.
    pub fn dispatched(&self) -> &[IrValueId] {
        if self.ordered_attribution.is_some() || self.dispatch_last {
            &self.members
        } else {
            self.handed_out()
        }
    }

    /// Family attribution whose order this actualization must honor.
    pub const fn ordered_attribution(&self) -> Option<crate::SystemAuthorityAttribution> {
        self.ordered_attribution
    }

    /// Returns the target-completion view in which the source-last member is
    /// submitted before the common join rather than run inline.
    pub(crate) fn dispatch_every_member(mut self) -> Self {
        self.dispatch_last = true;
        self
    }
}

/// One source-order family reservation edge retained in IR whether a supported
/// actualizer consumes it or the target conservatively declines the overlap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IrAuthorityOrder {
    earlier: IrValueId,
    later: IrValueId,
    family: crate::SystemAuthorityFamily,
    earlier_fragment: crate::SystemAuthorityFragment,
    later_fragment: crate::SystemAuthorityFragment,
    attribution: crate::SystemAuthorityAttribution,
}

impl IrAuthorityOrder {
    /// The earlier call result in source reservation order.
    pub const fn earlier(self) -> IrValueId {
        self.earlier
    }

    /// The later call result in source reservation order.
    pub const fn later(self) -> IrValueId {
        self.later
    }

    /// Family which owns this pair relation.
    pub const fn family(self) -> crate::SystemAuthorityFamily {
        self.family
    }

    /// Earlier fragment in the ordered pair.
    pub const fn earlier_fragment(self) -> crate::SystemAuthorityFragment {
        self.earlier_fragment
    }

    /// Later fragment in the ordered pair.
    pub const fn later_fragment(self) -> crate::SystemAuthorityFragment {
        self.later_fragment
    }

    /// Attribution identity carried by this ordered pair.
    pub const fn attribution(self) -> crate::SystemAuthorityAttribution {
        self.attribution
    }
}

/// Greatest number of source-ordered OutputSequence members one bounded
/// runtime root reservation can admit atomically.
pub(crate) const ORDERED_OUTPUT_BATCH_MEMBERS: usize = 16;
pub(crate) const FREE_COMPLETION_BATCH_MEMBERS: usize = 64;

/// How large a lane frame a handed-out call is granted, in bytes.
///
/// This restates `WF_PAR_FRAME_BYTES` in `backend/par_runtime.c`, because the
/// decision to emit a [`IrOperation::LoopSplit`] at all has to be made long
/// before a runtime exists — and a split whose frame is over the bound would be
/// refused every lane at run time and sequentialize with no report. The two
/// numbers live in two languages and are pinned to each other by
/// `the_compile_time_frame_bound_is_the_runtimes`.
pub const LANE_FRAME_BYTES: u64 = 256;

/// Why a function exists, for the one consumer that has to tell the two worlds
/// apart: a source function is emitted into both, while the two halves of a
/// [`IrOperation::LoopSplit`] each belong to exactly one.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IrSynthesis {
    /// The recursive range splitter. It exists only in the overlapped world —
    /// the sequential world calls the chunk directly — so cloning it would emit
    /// a second, unreachable caller of the chunk's clone and cost that clone
    /// the single-call-site inlining the sequential world depends on.
    Splitter,
    /// The loop over a subrange, seeded by its first parameter. Both worlds
    /// reach it, so it is cloned like a source function and each world's copy
    /// has exactly one caller.
    Chunk,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IrFunction {
    name: String,
    parameters: Vec<(IrValueId, IrType)>,
    result: IrType,
    values: Vec<IrType>,
    blocks: Vec<IrBlock>,
    overlaps: Vec<IrOverlap>,
    authority_orders: Vec<IrAuthorityOrder>,
    synthesis: Option<IrSynthesis>,
    target_action: crate::TargetAction,
}

impl IrFunction {
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Why this function exists, or `None` for a source function.
    pub const fn synthesis(&self) -> Option<IrSynthesis> {
        self.synthesis
    }

    /// Conservative compiler-owned suspension summary.
    pub const fn target_action(&self) -> crate::TargetAction {
        self.target_action
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

    /// The permission-derived overlap groups of this function's body, in
    /// source order and pairwise disjoint in their members.
    pub fn overlaps(&self) -> &[IrOverlap] {
        &self.overlaps
    }

    /// Ordered family reservation edges retained for a future actualizer.
    pub fn authority_orders(&self) -> &[IrAuthorityOrder] {
        &self.authority_orders
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

/// The [FN-7] entry form the program starts with [PROG-3].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrEntry {
    /// A `command` entry: program start supplies exactly the standard inputs
    /// these table ordinals select, in this order, and maps the returned
    /// `ExitStatus` to the host process status.
    Command { inputs: Vec<u8> },
}

#[derive(Debug)]
pub struct IrProgram<'classified, 'lexed, 'source> {
    _checked: CheckedProgram<'classified, 'lexed, 'source>,
    nominals: Vec<IrNominal>,
    constants: Vec<IrGlobalConstant>,
    functions: Vec<IrFunction>,
    main: u32,
    entry: IrEntry,
    actualization: Vec<String>,
}

impl IrProgram<'_, '_, '_> {
    pub fn nominals(&self) -> &[IrNominal] {
        &self.nominals
    }

    /// The entry form program start must implement.
    pub const fn entry(&self) -> &IrEntry {
        &self.entry
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

    /// Test-only fault injection for runtime-claim evidence.
    ///
    /// The source must first pass the complete claim judgment with a genuine
    /// residual and must define an ordinary `False()` value in the same
    /// function before that claim. This mutator changes only the lowered
    /// claim operand selected by the stable source function/name identity; it
    /// does not create a writer-visible escape or participate in checking.
    #[cfg(test)]
    pub(crate) fn force_claim_false_for_test(
        &mut self,
        function_name: &str,
        claim_name: &str,
    ) -> bool {
        let Some(function) = self
            .functions
            .iter_mut()
            .find(|function| function.name == function_name)
        else {
            return false;
        };
        let Some(false_value) = function.blocks.iter().find_map(|block| {
            block.instructions.iter().find_map(|instruction| {
                let IrInstruction::Define {
                    result,
                    operation: IrOperation::Constant(IrConstant::Bool(false)),
                    ..
                } = instruction
                else {
                    return None;
                };
                Some(*result)
            })
        }) else {
            return false;
        };
        for block in &mut function.blocks {
            for instruction in &mut block.instructions {
                let IrInstruction::Claim { condition, site } = instruction else {
                    continue;
                };
                if site.message == claim_name {
                    *condition = false_value;
                    return true;
                }
            }
        }
        false
    }

    /// The non-normative ledger lines this lowering added: one per permitted
    /// counted loop it either actualized or declined to.
    ///
    /// The judgment's own ledger states what [PAR-2] permits and is the same
    /// with or without `--par`. These state what *this* lowering did with a
    /// permission, which is a different fact and exists only where actualization
    /// was asked for. Both are developer output on the caller's channel; neither
    /// participates in acceptance or in any mandatory [DIAG-3] record.
    pub fn actualization_ledger(&self) -> &[String] {
        &self.actualization
    }

    /// Test-only malformed-IR probe: retypes one command parameter while
    /// keeping the function's local value table internally consistent.
    #[cfg(test)]
    pub(crate) fn retype_main_parameter_for_test(&mut self, parameter: usize, ty: IrType) -> bool {
        let Some(main) = self.functions.get_mut(self.main as usize) else {
            return false;
        };
        let Some((value, declared)) = main.parameters.get_mut(parameter) else {
            return false;
        };
        *declared = ty;
        let Some(stored) = main.values.get_mut(value.index()) else {
            return false;
        };
        *stored = ty;
        true
    }

    /// Test-only malformed-IR probe: retypes the command result while leaving
    /// its semantic entry identity unchanged.
    #[cfg(test)]
    pub(crate) fn retype_main_result_for_test(&mut self, ty: IrType) -> bool {
        let Some(main) = self.functions.get_mut(self.main as usize) else {
            return false;
        };
        main.result = ty;
        true
    }

    /// Test-only malformed-IR probe: retypes one argument of the first
    /// system call without changing its semantic operation identity.
    #[cfg(test)]
    pub(crate) fn retype_first_system_argument_for_test(
        &mut self,
        argument: usize,
        ty: IrType,
    ) -> bool {
        for function in &mut self.functions {
            let selected = function.blocks.iter().find_map(|block| {
                block.instructions.iter().find_map(|instruction| {
                    let IrInstruction::Define {
                        operation: IrOperation::SystemCall { arguments, .. },
                        ..
                    } = instruction
                    else {
                        return None;
                    };
                    arguments.get(argument).copied()
                })
            });
            let Some(value) = selected else {
                continue;
            };
            let Some(stored) = function.values.get_mut(value.index()) else {
                return false;
            };
            *stored = ty;
            return true;
        }
        false
    }

    /// Test-only malformed-IR probe: retypes the first system call's result
    /// while preserving the operation identity and local SSA agreement.
    #[cfg(test)]
    pub(crate) fn retype_first_system_result_for_test(&mut self, ty: IrType) -> bool {
        for function in &mut self.functions {
            let selected = function
                .blocks
                .iter()
                .enumerate()
                .find_map(|(block, data)| {
                    data.instructions
                        .iter()
                        .enumerate()
                        .find_map(|(instruction, value)| {
                            let IrInstruction::Define {
                                result,
                                operation: IrOperation::SystemCall { .. },
                                ..
                            } = value
                            else {
                                return None;
                            };
                            Some((block, instruction, *result))
                        })
                });
            let Some((block, instruction, result)) = selected else {
                continue;
            };
            let IrInstruction::Define { ty: declared, .. } =
                &mut function.blocks[block].instructions[instruction]
            else {
                return false;
            };
            *declared = ty;
            let Some(stored) = function.values.get_mut(result.index()) else {
                return false;
            };
            *stored = ty;
            return true;
        }
        false
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
