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
            Self::Generic(_) | Self::GenericInt(_) => false,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedNominalKind {
    Struct { fields: Vec<CheckedField> },
    Enum { variants: Vec<CheckedVariant> },
    Box { referent: CheckedType },
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

    pub(crate) const fn result_type(self, operand: FloatType) -> CheckedType {
        match self {
            Self::Equal
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual
            | Self::NotEqual => CheckedType::Bool,
            _ => CheckedType::Float(operand),
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
    },
    UserCall {
        function: FunctionId,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
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
        operand_type: FloatType,
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
    BorrowStruct {
        binding: BindingId,
        nominal: NominalId,
    },
    BorrowBox {
        binding: BindingId,
        nominal: NominalId,
    },
    ReborrowStruct {
        binding: BindingId,
        nominal: NominalId,
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
            Self::Binding { ty, .. } | Self::UserCall { result: ty, .. } => *ty,
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
            Self::BorrowStruct { nominal, .. } | Self::ReborrowStruct { nominal, .. } => {
                CheckedType::Nominal(*nominal)
            }
            Self::BorrowBox { nominal, .. } => CheckedType::Nominal(*nominal),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedDrop {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedProjectedDrop {
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
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
    DropExpression(CheckedExpression),
    Check {
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
    pub(crate) declared_traps: bool,
    pub(crate) declared_allocates_heap: bool,
    pub(crate) requires: Vec<CheckedStatement>,
    pub(crate) body: Vec<CheckedStatement>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedEffectCapabilities {
    pub(crate) reads: Vec<DeclarationId>,
    pub(crate) writes: Vec<DeclarationId>,
    pub(crate) allocates_heap: bool,
    pub(crate) allocates_arenas: Vec<DeclarationId>,
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
}
