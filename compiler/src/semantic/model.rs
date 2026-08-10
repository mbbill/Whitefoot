use crate::{DeclarationId, NodePath, PreludeDeclarationId};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FunctionId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ContractId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ConformanceId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BindingId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedMode {
    Own,
    Shared(DeclarationId),
    Unique(DeclarationId),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CheckedLoopId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NominalId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CheckedConstantId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedConst {
    Value(u64),
    Parameter(DeclarationId),
}

impl CheckedConst {
    pub(crate) const fn value(self) -> Option<u64> {
        match self {
            Self::Value(value) => Some(value),
            Self::Parameter(_) => None,
        }
    }

    pub(crate) const fn is_concrete(self) -> bool {
        matches!(self, Self::Value(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum IntegerType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
}

impl IntegerType {
    pub(crate) const fn width(self) -> u8 {
        match self {
            Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
            Self::I32 | Self::U32 => 32,
            Self::I64 | Self::U64 => 64,
        }
    }

    pub(crate) const fn signed(self) -> bool {
        matches!(self, Self::I8 | Self::I16 | Self::I32 | Self::I64)
    }

    pub(crate) const fn converts_totally_to(self, destination: Self) -> bool {
        self.width() < destination.width()
            && (self.signed() == destination.signed() || (!self.signed() && destination.signed()))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum FloatType {
    F32,
    F64,
}

impl FloatType {
    pub(crate) const fn width(self) -> u8 {
        match self {
            Self::F32 => 32,
            Self::F64 => 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedNumericType {
    Integer(IntegerType),
    Float(FloatType),
}

impl CheckedNumericType {
    pub(crate) const fn ty(self) -> CheckedType {
        match self {
            Self::Integer(ty) => CheckedType::Integer(ty),
            Self::Float(ty) => CheckedType::Float(ty),
        }
    }

    pub(crate) const fn converts_totally_to(self, destination: Self) -> bool {
        match (self, destination) {
            (Self::Integer(source), Self::Integer(destination)) => {
                source.converts_totally_to(destination)
            }
            (Self::Integer(source), Self::Float(FloatType::F32)) => source.width() <= 16,
            (Self::Integer(source), Self::Float(FloatType::F64)) => source.width() <= 32,
            (Self::Float(FloatType::F32), Self::Float(FloatType::F64)) => true,
            _ => false,
        }
    }

    pub(crate) const fn reinterprets_to(self, destination: Self) -> bool {
        match (self, destination) {
            (Self::Integer(source), Self::Integer(destination)) => {
                source.width() == destination.width() && source.signed() != destination.signed()
            }
            (Self::Integer(source), Self::Float(destination))
            | (Self::Float(destination), Self::Integer(source)) => {
                source.width() == destination.width()
            }
            (Self::Float(_), Self::Float(_)) => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedFlatElement {
    Unit,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    GenericInt(DeclarationId),
    GenericFloat(DeclarationId),
    TagOnlyNominal(NominalId),
}

impl CheckedFlatElement {
    pub(crate) const fn ty(self) -> CheckedType {
        match self {
            Self::Unit => CheckedType::Unit,
            Self::Bool => CheckedType::Bool,
            Self::Integer(ty) => CheckedType::Integer(ty),
            Self::Float(ty) => CheckedType::Float(ty),
            Self::GenericInt(declaration) => CheckedType::GenericInt(declaration),
            Self::GenericFloat(declaration) => CheckedType::GenericFloat(declaration),
            Self::TagOnlyNominal(id) => CheckedType::Nominal(id),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedType {
    Unit,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    Generic(DeclarationId),
    GenericInt(DeclarationId),
    GenericFloat(DeclarationId),
    Nominal(NominalId),
    Array {
        element: CheckedFlatElement,
        length: CheckedConst,
    },
    Slice {
        region: DeclarationId,
        element: CheckedFlatElement,
    },
    Buffer {
        element: CheckedFlatElement,
    },
}

impl CheckedType {
    pub(crate) const fn is_concrete(self) -> bool {
        match self {
            Self::Generic(_) | Self::GenericInt(_) | Self::GenericFloat(_) => false,
            Self::Array { element, length } => element.ty().is_concrete() && length.is_concrete(),
            Self::Slice { element, .. } => element.ty().is_concrete(),
            Self::Buffer { element } => element.ty().is_concrete(),
            Self::Unit | Self::Bool | Self::Integer(_) | Self::Float(_) | Self::Nominal(_) => true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedValue {
    Unit,
    Bool(bool),
    Integer {
        ty: IntegerType,
        bits: u64,
    },
    Float {
        ty: FloatType,
        bits: u64,
    },
    NumericIdentity {
        ty: CheckedType,
        one: bool,
    },
    Array {
        ty: CheckedType,
        elements: Vec<CheckedValue>,
    },
}

impl CheckedValue {
    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Unit => CheckedType::Unit,
            Self::Bool(_) => CheckedType::Bool,
            Self::Integer { ty, .. } => CheckedType::Integer(*ty),
            Self::Float { ty, .. } => CheckedType::Float(*ty),
            Self::NumericIdentity { ty, .. } => *ty,
            Self::Array { ty, .. } => *ty,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedConstant {
    pub(crate) id: CheckedConstantId,
    pub(crate) name: String,
    pub(crate) ty: CheckedType,
    pub(crate) value: CheckedValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedField {
    pub(crate) name: String,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedVariant {
    pub(crate) name: String,
    pub(crate) constructor: CheckedConstructor,
    pub(crate) tag: u32,
    pub(crate) fields: Vec<CheckedField>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedConstructor {
    Source(DeclarationId),
    Prelude(PreludeDeclarationId),
    System(crate::SystemDeclarationId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedNominalKind {
    Struct {
        fields: Vec<CheckedField>,
    },
    Enum {
        variants: Vec<CheckedVariant>,
    },
    Box {
        referent: CheckedType,
    },
    /// One [SYS-2] opaque resource type, by index into the system
    /// nominal catalog. It has no source-visible content; its
    /// compiler-derived release carries the fixed [SYS-5] row.
    SystemResource {
        nominal: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedNominal {
    pub(crate) id: NominalId,
    pub(crate) name: String,
    pub(crate) kind: CheckedNominalKind,
}

impl CheckedNominal {
    pub(crate) fn is_copy(&self) -> bool {
        matches!(
            &self.kind,
            CheckedNominalKind::Enum { variants }
                if variants.iter().all(|variant| variant.fields.is_empty())
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedIntegerOperation {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedBooleanOperation {
    And,
    Or,
    ExclusiveOr,
    Not,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedFloatOperation {
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

impl CheckedFloatOperation {
    pub(crate) const fn operand_count(self) -> usize {
        match self {
            Self::Infinity | Self::Nan => 0,
            Self::Negate
            | Self::Absolute
            | Self::Floor
            | Self::Ceil
            | Self::Truncate
            | Self::RoundEven
            | Self::SquareRootStrict => 1,
            Self::FusedMultiplyAddStrict => 3,
            _ => 2,
        }
    }

    pub(crate) const fn result_type(self, operand: CheckedType) -> CheckedType {
        match self {
            Self::Equal
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::NotEqual => CheckedType::Bool,
            _ => operand,
        }
    }
}

impl CheckedIntegerOperation {
    pub(crate) const fn traps(self) -> bool {
        matches!(
            self,
            Self::AddTrap
                | Self::SubtractTrap
                | Self::MultiplyTrap
                | Self::AbsoluteTrap
                | Self::NegateTrap
                | Self::DivideTrap
                | Self::RemainderTrap
                | Self::ShiftLeftTrap
                | Self::ShiftRightTrap
        )
    }

    pub(crate) const fn operand_count(self) -> usize {
        match self {
            Self::AbsoluteWrap
            | Self::AbsoluteTrap
            | Self::AbsoluteChecked
            | Self::NegateWrap
            | Self::NegateTrap
            | Self::NegateChecked
            | Self::BitNot
            | Self::PopulationCount
            | Self::LeadingZeros
            | Self::TrailingZeros
            | Self::ByteSwap => 1,
            _ => 2,
        }
    }

    pub(crate) const fn accepts_operand_type(self, operand: CheckedType) -> bool {
        match (self, operand) {
            (
                Self::AbsoluteWrap
                | Self::AbsoluteTrap
                | Self::AbsoluteChecked
                | Self::NegateWrap
                | Self::NegateTrap
                | Self::NegateChecked,
                CheckedType::Integer(operand),
            ) => operand.signed(),
            (Self::ByteSwap, CheckedType::Integer(operand)) => operand.width() >= 16,
            (
                Self::AbsoluteWrap
                | Self::AbsoluteTrap
                | Self::AbsoluteChecked
                | Self::NegateWrap
                | Self::NegateTrap
                | Self::NegateChecked
                | Self::ByteSwap,
                CheckedType::GenericInt(_),
            ) => false,
            (_, CheckedType::Integer(_) | CheckedType::GenericInt(_)) => true,
            _ => false,
        }
    }

    pub(crate) const fn argument_type(
        self,
        operand: CheckedType,
        index: usize,
    ) -> Option<CheckedType> {
        if index >= self.operand_count() {
            return None;
        }
        if index == 1
            && matches!(
                self,
                Self::ShiftLeftWrap
                    | Self::ShiftRightWrap
                    | Self::ShiftLeftTrap
                    | Self::ShiftRightTrap
                    | Self::RotateLeft
                    | Self::RotateRight
            )
        {
            Some(CheckedType::Integer(IntegerType::U32))
        } else {
            Some(operand)
        }
    }

    pub(crate) const fn scalar_result_type(self, operand: CheckedType) -> Option<CheckedType> {
        match self {
            Self::AddChecked
            | Self::SubtractChecked
            | Self::MultiplyChecked
            | Self::DivideChecked
            | Self::RemainderChecked
            | Self::AbsoluteChecked
            | Self::NegateChecked => None,
            Self::PopulationCount | Self::LeadingZeros | Self::TrailingZeros => {
                Some(CheckedType::Integer(IntegerType::U32))
            }
            Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual => Some(CheckedType::Bool),
            _ => Some(operand),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TrapSite {
    pub(crate) rule_id: &'static str,
    pub(crate) message: String,
    pub(crate) function: String,
    pub(crate) node_path: NodePath,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedTargetDomainObligation {
    RuntimeSizedAllocation,
    ElementAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CheckedRuntimeTargetObligations {
    allocation: CheckedTargetDomainObligation,
    element_address: CheckedTargetDomainObligation,
}

impl CheckedRuntimeTargetObligations {
    pub(crate) const fn new() -> Self {
        Self {
            allocation: CheckedTargetDomainObligation::RuntimeSizedAllocation,
            element_address: CheckedTargetDomainObligation::ElementAddress,
        }
    }

    pub(crate) const fn allocation(self) -> CheckedTargetDomainObligation {
        self.allocation
    }

    pub(crate) const fn element_address(self) -> CheckedTargetDomainObligation {
        self.element_address
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedArrayRoot {
    Binding {
        binding: BindingId,
        fields: Vec<u32>,
    },
    Constant(CheckedConstantId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedBufferRoot {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) element: CheckedFlatElement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedSliceRoot {
    pub(crate) binding: BindingId,
    pub(crate) element: CheckedFlatElement,
}

/// One member of the finite static origin set carried by a direct slice value.
///
/// The set is a conservative summary: one runtime slice descriptor points at
/// exactly one source, which must be represented by one member after complete
/// call substitution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedSliceOrigin {
    SourcePlace {
        root: DeclarationId,
        fields: Vec<u32>,
        origin_region: Option<DeclarationId>,
    },
    ImmutableConst,
    FormalSlice {
        parameter: DeclarationId,
        region: DeclarationId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedSliceSource {
    Array {
        root: CheckedArrayRoot,
        length: CheckedConst,
    },
    Buffer(CheckedBufferRoot),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedExpression {
    Constant(CheckedValue),
    Binding {
        binding: BindingId,
        ty: CheckedType,
        slice_origins: Vec<CheckedSliceOrigin>,
    },
    UserCall {
        function: FunctionId,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
        slice_origins: Vec<CheckedSliceOrigin>,
    },
    /// One call to an admitted [SYS-2] system operation, by index into the
    /// system operation catalog. Arguments follow declared parameter order.
    SystemCall {
        operation: u8,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
        /// The [DIAG-3] record for the operation's own runtime condition,
        /// present exactly when the [SYS-2] row classifies it `traps`.
        ///
        /// [SYS-8] validates the caller-written range before any host
        /// transfer, any read of the source, and any write of the
        /// destination; the failing site is this operation `call`, and only
        /// the checked program knows its source coordinate.
        trap: Option<TrapSite>,
    },
    IntegerOperation {
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
        trap: Option<TrapSite>,
    },
    FloatOperation {
        operation: CheckedFloatOperation,
        operand_type: CheckedType,
        arguments: Vec<CheckedExpression>,
    },
    NumericConversion {
        source: CheckedNumericType,
        destination: CheckedNumericType,
        value: Box<CheckedExpression>,
        result: CheckedType,
    },
    Reinterpret {
        source: CheckedNumericType,
        destination: CheckedNumericType,
        value: Box<CheckedExpression>,
    },
    BooleanOperation {
        operation: CheckedBooleanOperation,
        arguments: Vec<CheckedExpression>,
    },
    EnumEquality {
        equal: bool,
        operand_type: CheckedType,
        arguments: Vec<CheckedExpression>,
    },
    ArrayFill {
        ty: CheckedType,
        value: Box<CheckedExpression>,
        target_domain: CheckedTargetDomainObligation,
    },
    ArrayLength {
        root: CheckedArrayRoot,
        length: CheckedConst,
    },
    ArrayIndex {
        root: CheckedArrayRoot,
        element_type: CheckedType,
        length: CheckedConst,
        offset: Box<CheckedExpression>,
        trap: TrapSite,
        target_domain: CheckedTargetDomainObligation,
    },
    BufferFill {
        element: CheckedFlatElement,
        length: Box<CheckedExpression>,
        value: Box<CheckedExpression>,
        trap: TrapSite,
        target_domains: CheckedRuntimeTargetObligations,
    },
    BufferLength {
        root: CheckedBufferRoot,
    },
    BufferIndex {
        root: CheckedBufferRoot,
        offset: Box<CheckedExpression>,
        trap: TrapSite,
        target_domain: CheckedTargetDomainObligation,
    },
    SliceOf {
        source: CheckedSliceSource,
        region: DeclarationId,
        element: CheckedFlatElement,
        origins: Vec<CheckedSliceOrigin>,
    },
    SliceLength {
        root: CheckedSliceRoot,
    },
    SliceIndex {
        root: CheckedSliceRoot,
        offset: Box<CheckedExpression>,
        trap: TrapSite,
        target_domain: CheckedTargetDomainObligation,
    },
    BoxNew {
        nominal: NominalId,
        value: Box<CheckedExpression>,
    },
    BoxDeref {
        nominal: NominalId,
        referent: CheckedType,
        value: Box<CheckedExpression>,
    },
    BorrowBuffer {
        root: CheckedBufferRoot,
    },
    /// A borrow of directly stored content — a scalar, struct, or enum — which
    /// is the address of the borrowed binding's storage [OWN-2, OWN-5].
    BorrowAddressed {
        binding: BindingId,
        ty: CheckedType,
    },
    BorrowBox {
        binding: BindingId,
        nominal: NominalId,
    },
    BorrowSystemResource {
        binding: BindingId,
        nominal: NominalId,
    },
    /// The same address, taken from a binding that already holds one: a borrow
    /// whose place is rooted at another borrow holder [OWN-6, OWN-10].
    ReborrowAddressed {
        binding: BindingId,
        ty: CheckedType,
    },
    /// The referent value read through such a holder [TYPE-7]. The holder
    /// itself stays a distinct expression, so lowering never has to guess
    /// whether a borrow binding is being passed on or read through.
    DerefAddressed {
        binding: BindingId,
        ty: CheckedType,
    },
    ConstructStruct {
        nominal: NominalId,
        fields: Vec<CheckedExpression>,
    },
    ConstructEnum {
        nominal: NominalId,
        variant: u32,
        fields: Vec<CheckedExpression>,
    },
    Project {
        binding: BindingId,
        fields: Vec<u32>,
        ty: CheckedType,
        consume_root: bool,
        residual_drops: Vec<CheckedProjectedDrop>,
    },
    ProjectValue {
        value: Box<CheckedExpression>,
        nominal: NominalId,
        field: u32,
        ty: CheckedType,
    },
}

impl CheckedExpression {
    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Constant(value) => value.ty(),
            Self::Binding { ty, .. }
            | Self::UserCall { result: ty, .. }
            | Self::SystemCall { result: ty, .. } => *ty,
            Self::IntegerOperation { result, .. } | Self::NumericConversion { result, .. } => {
                *result
            }
            Self::Reinterpret { destination, .. } => destination.ty(),
            Self::FloatOperation {
                operation,
                operand_type,
                ..
            } => operation.result_type(*operand_type),
            Self::BooleanOperation { .. } | Self::EnumEquality { .. } => CheckedType::Bool,
            Self::ArrayFill { ty, .. } => *ty,
            Self::ArrayLength { .. } => CheckedType::Integer(IntegerType::U64),
            Self::ArrayIndex { element_type, .. } => *element_type,
            Self::BufferFill { element, .. } => CheckedType::Buffer { element: *element },
            Self::BufferLength { .. } => CheckedType::Integer(IntegerType::U64),
            Self::BufferIndex { root, .. } => root.element.ty(),
            Self::SliceOf {
                region, element, ..
            } => CheckedType::Slice {
                region: *region,
                element: *element,
            },
            Self::SliceLength { .. } => CheckedType::Integer(IntegerType::U64),
            Self::SliceIndex { root, .. } => root.element.ty(),
            Self::BoxNew { nominal, .. } => CheckedType::Nominal(*nominal),
            Self::BoxDeref { referent, .. } => *referent,
            Self::BorrowBuffer { root } => CheckedType::Buffer {
                element: root.element,
            },
            Self::BorrowAddressed { ty, .. }
            | Self::ReborrowAddressed { ty, .. }
            | Self::DerefAddressed { ty, .. } => *ty,
            Self::BorrowBox { nominal, .. } | Self::BorrowSystemResource { nominal, .. } => {
                CheckedType::Nominal(*nominal)
            }
            Self::ConstructStruct { nominal, .. } | Self::ConstructEnum { nominal, .. } => {
                CheckedType::Nominal(*nominal)
            }
            Self::Project { ty, .. } | Self::ProjectValue { ty, .. } => *ty,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedEnumType {
    Bool,
    Nominal(NominalId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedMatchBinder {
    pub(crate) binding: BindingId,
    pub(crate) field: u32,
    pub(crate) mode: CheckedMode,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedMatchArm {
    pub(crate) tag: u32,
    pub(crate) binders: Vec<CheckedMatchBinder>,
    pub(crate) body: Vec<CheckedStatement>,
    pub(crate) fallthrough_drops: Vec<CheckedDrop>,
}

/// One compiler-derived release on a normal control-flow edge [STOR-3].
///
/// The record is explicit in the checked program [DIAG-2] rather than being
/// rederived from the type by every consumer: [EFF-2]'s release contribution
/// reads its row, and lowering carries the same record into the typed IR so a
/// target stage can emit the exact [SYS-5] action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedDrop {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
    pub(crate) release: crate::SystemRelease,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedProjectedDrop {
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
    pub(crate) release: crate::SystemRelease,
}

/// A SET-1 target whose root, path, copy type, and post-RHS writability have
/// all been established by semantic checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedWritablePlace {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedArraySetTarget {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) array_type: CheckedType,
    pub(crate) element_type: CheckedType,
    pub(crate) length: CheckedConst,
    pub(crate) offset: CheckedExpression,
    pub(crate) trap: TrapSite,
    pub(crate) target_domain: CheckedTargetDomainObligation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedBufferSetTarget {
    pub(crate) root: CheckedBufferRoot,
    pub(crate) offset: CheckedExpression,
    pub(crate) trap: TrapSite,
    pub(crate) target_domain: CheckedTargetDomainObligation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedSetTarget {
    Place(CheckedWritablePlace),
    ArrayIndex(Box<CheckedArraySetTarget>),
    BufferIndex(Box<CheckedBufferSetTarget>),
}

impl CheckedSetTarget {
    pub(crate) fn binding(&self) -> BindingId {
        match self {
            Self::Place(target) => target.binding,
            Self::ArrayIndex(target) => target.binding,
            Self::BufferIndex(target) => target.root.binding,
        }
    }

    pub(crate) fn ty(&self) -> CheckedType {
        match self {
            Self::Place(target) => target.ty,
            Self::ArrayIndex(target) => target.element_type,
            Self::BufferIndex(target) => target.root.element.ty(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PropagationContext {
    pub(crate) function: String,
    pub(crate) node_path: NodePath,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedStatement {
    Let {
        binding: BindingId,
        value: CheckedExpression,
    },
    PropagateLet {
        binding: BindingId,
        scrutinee: CheckedExpression,
        result_nominal: NominalId,
        return_nominal: NominalId,
        ok_type: CheckedType,
        error_type: CheckedType,
        error_drops: Vec<CheckedDrop>,
        context: PropagationContext,
    },
    Set {
        target: CheckedSetTarget,
        value: CheckedExpression,
    },
    Evaluate(CheckedExpression),
    /// The discarded result of an expression statement, with the
    /// compiler-derived release it runs [STOR-3].
    DropExpression {
        value: CheckedExpression,
        release: crate::SystemRelease,
    },
    Check {
        condition: CheckedExpression,
        trap: TrapSite,
    },
    /// A named runtime check [CLM-1]: check-else-trap semantics with the
    /// claim name as the DIAG-3 message. The justification STRING is
    /// compile-time review data the checked program retains [DIAG-2]; it
    /// never reaches runtime behavior.
    Claim {
        name: String,
        justification: String,
        condition: CheckedExpression,
        trap: TrapSite,
    },
    Return {
        value: CheckedExpression,
        drops: Vec<CheckedDrop>,
    },
    Match {
        scrutinee: CheckedExpression,
        enum_type: CheckedEnumType,
        arms: Vec<CheckedMatchArm>,
        continues: bool,
    },
    ValueMatchLet {
        binding: BindingId,
        result_type: CheckedType,
        scrutinee: CheckedExpression,
        enum_type: CheckedEnumType,
        arms: Vec<CheckedMatchArm>,
        continues: bool,
    },
    Give {
        value: CheckedExpression,
        drops: Vec<CheckedDrop>,
    },
    Loop {
        id: CheckedLoopId,
        body: Vec<CheckedStatement>,
        backedge_drops: Vec<CheckedDrop>,
    },
    CountedRange {
        id: CheckedLoopId,
        node_path: NodePath,
        binder: BindingId,
        lower: CheckedExpression,
        upper: CheckedExpression,
        body: Vec<CheckedStatement>,
        backedge_drops: Vec<CheckedDrop>,
    },
    Break {
        target: CheckedLoopId,
        drops: Vec<CheckedDrop>,
    },
    Region {
        body: Vec<CheckedStatement>,
        fallthrough_drops: Vec<CheckedDrop>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedParameter {
    pub(crate) name: String,
    pub(crate) binding: BindingId,
    pub(crate) mode: CheckedMode,
    pub(crate) ty: CheckedType,
    pub(crate) slice_origins: Vec<CheckedSliceOrigin>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedFunction {
    pub(crate) id: FunctionId,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) symbol: String,
    pub(crate) parameters: Vec<CheckedParameter>,
    pub(crate) result_mode: CheckedMode,
    pub(crate) result: CheckedType,
    pub(crate) slice_return_ceiling: Vec<CheckedSliceOrigin>,
    pub(crate) declared_traps: bool,
    pub(crate) declared_allocates_heap: bool,
    pub(crate) requires: Vec<CheckedStatement>,
    pub(crate) body: Vec<CheckedStatement>,
    /// Retained dark [ENT] analysis summary [DIAG-2]. No acceptance,
    /// diagnostic, or lowering behavior reads it in this slice; tests
    /// exercise it directly.
    #[allow(dead_code)]
    pub(crate) entailment: super::entailment::FunctionEntailment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedEffectCapabilities {
    pub(crate) reads: Vec<DeclarationId>,
    pub(crate) writes: Vec<DeclarationId>,
    pub(crate) allocates_heap: bool,
    pub(crate) allocates_arenas: Vec<DeclarationId>,
    pub(crate) external: bool,
    pub(crate) blocks: bool,
    pub(crate) traps: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedContractParameter {
    pub(crate) mode: CheckedMode,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedContractMember {
    pub(crate) name: String,
    pub(crate) region_parameters: Vec<DeclarationId>,
    pub(crate) parameters: Vec<CheckedContractParameter>,
    pub(crate) result_mode: CheckedMode,
    pub(crate) result: CheckedType,
    pub(crate) slice_return_ceiling: Vec<CheckedSliceOrigin>,
    pub(crate) effects: CheckedEffectCapabilities,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedContractLawKind {
    Associative,
    Commutative,
    Identity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedLawIdentity {
    Literal(CheckedValue),
    Constant(CheckedConstantId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedContractLaw {
    pub(crate) node_path: NodePath,
    pub(crate) kind: CheckedContractLawKind,
    pub(crate) member: u32,
    pub(crate) identity: Option<CheckedLawIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedContract {
    pub(crate) id: ContractId,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) members: Vec<CheckedContractMember>,
    pub(crate) laws: Vec<CheckedContractLaw>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CheckedConformanceBinding {
    pub(crate) member: u32,
    pub(crate) function: FunctionId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedConformance {
    pub(crate) id: ConformanceId,
    pub(crate) node_path: NodePath,
    pub(crate) subject: CheckedType,
    pub(crate) contract: ContractId,
    pub(crate) bindings: Vec<CheckedConformanceBinding>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedLawDerivation {
    pub(crate) conformance: ConformanceId,
    pub(crate) contract_law: u32,
    pub(crate) function: FunctionId,
    pub(crate) operation: CheckedIntegerOperation,
    pub(crate) domain: IntegerType,
    pub(crate) law: CheckedContractLawKind,
    pub(crate) identity: Option<CheckedLawIdentity>,
}

/// The [FN-7] entry form the checker admitted for one compilation unit.
///
/// Lowering needs the shape rather than the declaration: the two forms take
/// different program-start bootstraps [PROG-3]. The `command` variant carries
/// the standard-input table ordinals the entry selected, in the same order as
/// its declared parameters, because ordinal identity — never type identity —
/// selects each supplied value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedEntryForm {
    /// The unlabelled entry: no program kind, no standard input, `own unit`.
    Unlabelled,
    /// A `command` entry and the standard-input rows it selected.
    Command {
        /// Selected [FN-7] table ordinals in strictly increasing order.
        inputs: Vec<u8>,
        /// Retained conservative alias links between selected inputs.
        aliases: Vec<CheckedResourceAlias>,
    },
}

/// One retained conservative alias link between two standard-input resource
/// owners, by [FN-7] table ordinal.
///
/// [SYS-12] fixes exactly one for the first slice: redirection may make the
/// `command.stdout` and `command.stderr` owners the same sink. v0.18 defines
/// no consumer of the fact and it refuses no program; the checked program
/// retains it [DIAG-2] so a later verified cross-resource reordering fact
/// fails closed on this pair rather than treating two separate `Output`
/// owners as disjoint sinks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CheckedResourceAlias {
    pub(crate) left: u8,
    pub(crate) right: u8,
}

#[derive(Debug)]
pub(crate) struct CheckedProgramData {
    pub(crate) nominals: Vec<CheckedNominal>,
    // Nominal instances discovered by the ordinary function path form this
    // prefix. Later instances exist only to type-check static metadata.
    pub(crate) executable_nominal_count: usize,
    pub(crate) constants: Vec<CheckedConstant>,
    pub(crate) functions: Vec<CheckedFunction>,
    // Deliberately unread by ordinary lowering: FN-3/FN-4 metadata is
    // source-acceptance evidence and grants no executable authority.
    #[allow(dead_code)]
    pub(crate) contracts: Vec<CheckedContract>,
    #[allow(dead_code)]
    pub(crate) conformances: Vec<CheckedConformance>,
    #[allow(dead_code)]
    pub(crate) law_derivations: Vec<CheckedLawDerivation>,
    pub(crate) main: FunctionId,
    pub(crate) entry: CheckedEntryForm,
    /// The required non-rejecting [CLM-2] redundancy advisories, one per
    /// claim whose predicate the closed fact state already derives. The
    /// channel and encoding are implementation-owned in this version; this
    /// list is the compiler's channel, and the CLI prints it to stderr.
    pub(crate) claim_advisories: Vec<ClaimAdvisory>,
}

/// One [CLM-2] redundancy advisory: non-rejecting compile-time review data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ClaimAdvisory {
    /// The enclosing source function IDENT.
    pub(crate) function: String,
    /// The claim's written name.
    pub(crate) name: String,
}
