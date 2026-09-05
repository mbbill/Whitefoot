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

/// One proof-only mathematical integer expression. Each leaf retains
/// its exact source integer type while denoting its value in the mathematical
/// integers; this metadata does not request a runtime conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedAffineExpression {
    pub(crate) node_path: NodePath,
    pub(crate) kind: CheckedAffineExpressionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedAffineExpressionKind {
    Constant {
        value: i128,
        ty: IntegerType,
    },
    Local {
        binding: BindingId,
        ty: IntegerType,
    },
    Add(Box<CheckedAffineExpression>, Box<CheckedAffineExpression>),
    Subtract(Box<CheckedAffineExpression>, Box<CheckedAffineExpression>),
    MultiplyByConstant {
        constant: i128,
        constant_ty: IntegerType,
        value: Box<CheckedAffineExpression>,
    },
}

/// One normalized source-written affine ordered relation. `left - right <=
/// bound` has `bound == 0` for non-strict order and `bound == -1` for strict
/// integer order. The checker has already admitted the expression vocabulary,
/// but this record alone grants no fact: INV-1 or PRF-1 must still prove its
/// owning judgment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedAffineRelation {
    pub(crate) node_path: NodePath,
    pub(crate) left: CheckedAffineExpression,
    pub(crate) right: CheckedAffineExpression,
    pub(crate) bound: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedLoopInvariant {
    pub(crate) loop_id: CheckedLoopId,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) relation: CheckedAffineRelation,
}

/// One source-written `use` in a local invariant certificate.
///
/// `factor` is a positive proof-domain integer. The omitted source spelling is
/// represented as one. This record deliberately contains no accumulating
/// state: every use is checked against the invariant statement's same entering
/// proof context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedProofUse {
    pub(crate) node_path: NodePath,
    pub(crate) factor: i128,
    pub(crate) source: CheckedProofUseSource,
}

/// The source selected by one written `use`.
///
/// A named source is the immutable theorem image published by the resolved
/// invariant declaration. A relation source is independently proved by AUTO
/// in the local invariant's entering context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedProofUseSource {
    Named(DeclarationId),
    Relation(CheckedAffineRelation),
}

/// One erased source-written local invariant. Every `use` and the target are
/// written in the `.wf` source; later analysis proves each use independently,
/// follows the written multipliers, and publishes only the checked target.
///
/// The historical type name remains internal while the parser surface moves
/// from `prove` to `invariant`; it does not grant a separate proof language or
/// runtime operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedSourceProof {
    pub(crate) node_path: NodePath,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) target: CheckedAffineRelation,
    pub(crate) uses: Vec<CheckedProofUse>,
}

/// The checked source production that owns a value initializer. These forms
/// share GIVE-1 typing and lowering, but only `value_if` is an ENT-5 relation
/// carrier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ValueInitializerKind {
    ValueIf,
    ValueMatch,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NominalId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CheckedConstantId(pub(crate) u32);

/// One checker-interned symbolic const operation [`DerivedConst`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DerivedConstId(pub(crate) u32);

/// The five bare const-expression operations of the CONST-1 candidate
/// grammar. Const evaluation happens at monomorphization in the unsigned
/// 64-bit domain under the const-eval overflow policy: a result outside that
/// domain or a zero divisor is a compile-time rejection citing CONST-1, never
/// a runtime trap, so this family is disjoint from the runtime arithmetic
/// modes and excluded from EFF-2's state/allocation effect relation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ConstOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

impl ConstOperation {
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "-",
            Self::Multiply => "*",
            Self::Divide => "/",
            Self::Remainder => "%",
        }
    }
}

/// One interned symbolic const-expression node: exactly one operation over
/// two operands, mirroring the one-operation source grammar. At least one
/// operand is symbolic — a fully concrete operation is evaluated eagerly and
/// never interned — so a value of this shape exists only while a generic
/// template or symbolic validation instance is being checked, and every
/// concrete instantiation evaluates it away.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct DerivedConst {
    pub(crate) operation: ConstOperation,
    pub(crate) left: CheckedConst,
    pub(crate) right: CheckedConst,
}

/// Evaluates one const operation in the u64 const-eval domain.
///
/// `None` is the const-eval overflow policy's rejection premise: the
/// mathematical result is outside the domain, or the divisor is zero.
pub(crate) const fn evaluate_const_operation(
    operation: ConstOperation,
    left: u64,
    right: u64,
) -> Option<u64> {
    match operation {
        ConstOperation::Add => left.checked_add(right),
        ConstOperation::Subtract => left.checked_sub(right),
        ConstOperation::Multiply => left.checked_mul(right),
        ConstOperation::Divide => left.checked_div(right),
        ConstOperation::Remainder => left.checked_rem(right),
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedConst {
    Value(u64),
    Parameter(DeclarationId),
    /// One symbolic const operation, by checker-interned identity. Structural
    /// identity is id identity because interning is hash-consed.
    Derived(DerivedConstId),
}

impl CheckedConst {
    pub(crate) const fn value(self) -> Option<u64> {
        match self {
            Self::Value(value) => Some(value),
            Self::Parameter(_) | Self::Derived(_) => None,
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
    /// One affine aggregate element type: a region-free non-copy nominal
    /// stored by value. [TYPE-2] admits this element domain for `buffer`
    /// formation only; arrays and slices keep the flat copy domain, so
    /// their element constructors never produce this variant.
    Nominal(NominalId),
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
            Self::TagOnlyNominal(id) | Self::Nominal(id) => CheckedType::Nominal(id),
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    /// One struct-typed constant value [CONST-2 candidate]: the nominal
    /// instance plus its complete field values in declared order.
    Struct {
        ty: CheckedType,
        fields: Vec<CheckedValue>,
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
            Self::Struct { ty, .. } => *ty,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedConstant {
    pub(crate) id: CheckedConstantId,
    /// Resolved declaration identity retained for named-const goal leaves.
    pub(crate) declaration: DeclarationId,
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
    /// One `arena<'r, T>` instance [STOR-1, STOR-2]. The region is part of
    /// the type's identity, so `arena<'r, T>` and `arena<'s, T>` are two
    /// nominals. Its storage is released with its region rather than with an
    /// owner scope [STOR-3, STOR-4], so the value itself derives no drop.
    Arena {
        region: DeclarationId,
        content: CheckedType,
    },
    /// The compiler-owned allocation list one region block carries when it
    /// has arena allocations: a pointer-shaped cell whose compiler-derived
    /// drop walks and frees every registered allocation, which is exactly
    /// the region's [STOR-3] storage release.
    ArenaStorage,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedIntegerOperation {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedBooleanOperation {
    And,
    Or,
    ExclusiveOr,
    Not,
}

impl CheckedBooleanOperation {
    /// The [OP-1] spelling of each Bool row, exhaustive by construction.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::And => "band",
            Self::Or => "bor",
            Self::ExclusiveOr => "bxor",
            Self::Not => "bnot",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    /// The [OP-1] spelling of each float operation the compiler models,
    /// exhaustive by construction and locked against the specification table
    /// by `semantic::tests::operation_table`.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::AddStrict => "fadd.strict",
            Self::SubtractStrict => "fsub.strict",
            Self::MultiplyStrict => "fmul.strict",
            Self::DivideStrict => "fdiv.strict",
            Self::Equal => "feq",
            Self::Less => "flt",
            Self::LessEqual => "fle",
            Self::Greater => "fgt",
            Self::GreaterEqual => "fge",
            Self::NotEqual => "fne",
            Self::Negate => "fneg",
            Self::Absolute => "fabs",
            Self::CopySign => "fcopysign",
            Self::Minimum => "fmin",
            Self::Maximum => "fmax",
            Self::Floor => "ffloor",
            Self::Ceil => "fceil",
            Self::Truncate => "ftrunc",
            Self::RoundEven => "froundeven",
            Self::Remainder => "frem",
            Self::SquareRootStrict => "fsqrt.strict",
            Self::FusedMultiplyAddStrict => "ffma.strict",
            Self::Infinity => "finf",
            Self::Nan => "fnan",
        }
    }

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

/// The [PRE-1] error type a checked [OP-1] integer row reports.
///
/// The row's `signature` cell names it, so the choice is table data rather
/// than a property of the operation's semantics; this enum exists so that one
/// place decides it and an extraction lock can compare that decision against
/// the specification's own cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedIntegerErrorClass {
    Overflow,
    DivError,
}

impl CheckedIntegerErrorClass {
    /// The exact PRE-1 spelling the `wf-ops` `signature` cell writes.
    ///
    /// Read only by the extraction lock: nothing in the compiler's own path
    /// needs the name, because the class already selects the prelude type.
    #[cfg(test)]
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::Overflow => "Overflow",
            Self::DivError => "DivError",
        }
    }
}

impl CheckedIntegerOperation {
    /// The [OP-1] spelling of each integer operation the compiler models.
    ///
    /// Exhaustive by construction: a new variant is a compile error here,
    /// which is the point — the row it belongs to must be named before it can
    /// be checked. `semantic::tests::operation_table` locks this map against
    /// the specification's own `wf-ops` table in both directions, so every
    /// diagnostic that renders an operation in source terms renders the
    /// spelling the specification fixes.
    pub(crate) const fn spelling(self) -> &'static str {
        match self {
            Self::AddWrap => "+wrap",
            Self::SubtractWrap => "-wrap",
            Self::MultiplyWrap => "*wrap",
            Self::AddExact => "+",
            Self::SubtractExact => "-",
            Self::MultiplyExact => "*",
            Self::AddDefined => "+defined",
            Self::SubtractDefined => "-defined",
            Self::MultiplyDefined => "*defined",
            Self::AddChecked => "+checked",
            Self::SubtractChecked => "-checked",
            Self::MultiplyChecked => "*checked",
            Self::DivideExact => "/",
            Self::RemainderExact => "%",
            Self::DivideDefined => "/defined",
            Self::RemainderDefined => "%defined",
            Self::DivideChecked => "/checked",
            Self::RemainderChecked => "%checked",
            Self::AbsoluteWrap => "iabs.wrap",
            Self::AbsoluteExact => "iabs",
            Self::AbsoluteDefined => "iabs.defined",
            Self::AbsoluteChecked => "iabs.checked",
            Self::NegateWrap => "ineg.wrap",
            Self::NegateExact => "ineg",
            Self::NegateDefined => "ineg.defined",
            Self::NegateChecked => "ineg.checked",
            Self::BitAnd => "iand",
            Self::BitOr => "ior",
            Self::BitXor => "ixor",
            Self::BitNot => "inot",
            Self::ShiftLeftWrap => "ishl.wrap",
            Self::ShiftRightWrap => "ishr.wrap",
            Self::ShiftLeftExact => "ishl",
            Self::ShiftRightExact => "ishr",
            Self::ShiftLeftDefined => "ishl.defined",
            Self::ShiftRightDefined => "ishr.defined",
            Self::RotateLeft => "irotl",
            Self::RotateRight => "irotr",
            Self::PopulationCount => "ipopcount",
            Self::LeadingZeros => "iclz",
            Self::TrailingZeros => "ictz",
            Self::ByteSwap => "ibswap",
            Self::MultiplyHigh => "imulhi",
            Self::AddSaturating => "+sat",
            Self::SubtractSaturating => "-sat",
            Self::MultiplySaturating => "*sat",
            Self::Minimum => "imin",
            Self::Maximum => "imax",
            Self::Equal => "==",
            Self::NotEqual => "!=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::Greater => ">",
            Self::GreaterEqual => ">=",
        }
    }

    pub(crate) const fn is_exact(self) -> bool {
        matches!(
            self,
            Self::AddExact
                | Self::SubtractExact
                | Self::MultiplyExact
                | Self::DivideExact
                | Self::RemainderExact
                | Self::AbsoluteExact
                | Self::NegateExact
                | Self::ShiftLeftExact
                | Self::ShiftRightExact
        )
    }

    pub(crate) const fn defined_query(self) -> Option<Self> {
        Some(match self {
            Self::AddExact => Self::AddDefined,
            Self::SubtractExact => Self::SubtractDefined,
            Self::MultiplyExact => Self::MultiplyDefined,
            Self::DivideExact => Self::DivideDefined,
            Self::RemainderExact => Self::RemainderDefined,
            Self::AbsoluteExact => Self::AbsoluteDefined,
            Self::NegateExact => Self::NegateDefined,
            Self::ShiftLeftExact => Self::ShiftLeftDefined,
            Self::ShiftRightExact => Self::ShiftRightDefined,
            _ => return None,
        })
    }

    pub(crate) const fn is_defined_query(self) -> bool {
        matches!(
            self,
            Self::AddDefined
                | Self::SubtractDefined
                | Self::MultiplyDefined
                | Self::DivideDefined
                | Self::RemainderDefined
                | Self::AbsoluteDefined
                | Self::NegateDefined
                | Self::ShiftLeftDefined
                | Self::ShiftRightDefined
        )
    }

    pub(crate) const fn operand_count(self) -> usize {
        match self {
            Self::AbsoluteWrap
            | Self::AbsoluteExact
            | Self::AbsoluteDefined
            | Self::AbsoluteChecked
            | Self::NegateWrap
            | Self::NegateExact
            | Self::NegateDefined
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
                | Self::AbsoluteExact
                | Self::AbsoluteDefined
                | Self::AbsoluteChecked
                | Self::NegateWrap
                | Self::NegateExact
                | Self::NegateDefined
                | Self::NegateChecked,
                CheckedType::Integer(operand),
            ) => operand.signed(),
            (Self::ByteSwap, CheckedType::Integer(operand)) => operand.width() >= 16,
            (
                Self::AbsoluteWrap
                | Self::AbsoluteExact
                | Self::AbsoluteDefined
                | Self::AbsoluteChecked
                | Self::NegateWrap
                | Self::NegateExact
                | Self::NegateDefined
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
                    | Self::ShiftLeftExact
                    | Self::ShiftRightExact
                    | Self::ShiftLeftDefined
                    | Self::ShiftRightDefined
                    | Self::RotateLeft
                    | Self::RotateRight
            )
        {
            Some(CheckedType::Integer(IntegerType::U32))
        } else {
            Some(operand)
        }
    }

    /// The error type of a checked row, or `None` for a row whose result is a
    /// scalar. Exactly the complement of [`Self::scalar_result_type`]'s `None`:
    /// a row either produces a scalar or produces `Result<T, E>`.
    pub(crate) const fn checked_error(self) -> Option<CheckedIntegerErrorClass> {
        match self {
            Self::AddChecked
            | Self::SubtractChecked
            | Self::MultiplyChecked
            | Self::AbsoluteChecked
            | Self::NegateChecked => Some(CheckedIntegerErrorClass::Overflow),
            Self::DivideChecked | Self::RemainderChecked => {
                Some(CheckedIntegerErrorClass::DivError)
            }
            _ => None,
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
            | Self::GreaterEqual
            | Self::AddDefined
            | Self::SubtractDefined
            | Self::MultiplyDefined
            | Self::DivideDefined
            | Self::RemainderDefined
            | Self::AbsoluteDefined
            | Self::NegateDefined
            | Self::ShiftLeftDefined
            | Self::ShiftRightDefined => Some(CheckedType::Bool),
            _ => Some(operand),
        }
    }
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
    /// Tightest target-independent length ceiling retained at this source
    /// allocation site. Entailment installs it after proving OP-9; lowering
    /// must not proceed while it is absent.
    source_length_upper_bound: Option<u64>,
}

/// Target-independent upper bounds for one stored value's representation.
/// The backend must qualify its concrete layout against all three cells
/// before it may use the source-level `buffer_fits<T>` proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedLayoutMagnitude {
    Finite(u64),
    AboveU64,
}

impl CheckedLayoutMagnitude {
    /// The exact largest element count admitted by OP-9 for this stride.
    /// Every stride represented by `AboveU64` is greater than U64_MAX, so
    /// only the zero-length allocation can fit its u64 byte-count domain.
    pub(crate) const fn allocation_limit(self) -> u64 {
        match self {
            Self::Finite(stride) => {
                assert!(stride >= 1, "a layout stride ceiling is always positive");
                u64::MAX / stride
            }
            Self::AboveU64 => 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CheckedLayoutCeiling {
    pub(crate) size: CheckedLayoutMagnitude,
    pub(crate) align: u64,
    pub(crate) stride: CheckedLayoutMagnitude,
}

impl CheckedRuntimeTargetObligations {
    pub(crate) const fn new() -> Self {
        Self {
            allocation: CheckedTargetDomainObligation::RuntimeSizedAllocation,
            element_address: CheckedTargetDomainObligation::ElementAddress,
            source_length_upper_bound: None,
        }
    }

    pub(crate) const fn allocation(self) -> CheckedTargetDomainObligation {
        self.allocation
    }

    pub(crate) const fn element_address(self) -> CheckedTargetDomainObligation {
        self.element_address
    }

    pub(crate) const fn source_length_upper_bound(self) -> Option<u64> {
        self.source_length_upper_bound
    }

    /// Installs the conclusion of the source allocation proof on the checked
    /// allocation node. This copies an already-derived fact; it performs no
    /// second proof or replay.
    pub(crate) fn install_source_length_upper_bound(&mut self, upper_bound: u64) {
        self.source_length_upper_bound = Some(upper_bound);
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

/// The finite set of caller-formal state paths a checked value may carry.
///
/// An empty formal set means every identity in the value is invocation-local;
/// the value's affine type, not this attribution summary, says whether such an
/// identity exists.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CheckedStateOrigins {
    pub(crate) unknown: bool,
    pub(crate) formals: Vec<CheckedStateOrigin>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CheckedStateOrigin {
    /// Path inside the value carrying this origin.
    pub(crate) value_fields: Vec<u32>,
    /// Selected enum variant while the origin is still correlated with a
    /// direct constructor or match. `None` is the conservative whole-enum
    /// route used at callable boundaries.
    pub(crate) variant: Option<u32>,
    /// Exact incoming formal state leaf supplying it.
    pub(crate) source: CheckedStatePath,
}

impl CheckedStateOrigins {
    pub(crate) fn fresh() -> Self {
        Self {
            unknown: false,
            formals: Vec::new(),
        }
    }

    pub(crate) fn formal_leaves(formal: DeclarationId, leaves: Vec<Vec<u32>>) -> Self {
        Self {
            unknown: false,
            formals: leaves
                .into_iter()
                .map(|fields| CheckedStateOrigin {
                    value_fields: fields.clone(),
                    variant: None,
                    source: CheckedStatePath {
                        root: formal,
                        fields,
                    },
                })
                .collect(),
        }
    }

    pub(crate) fn union(&mut self, other: &Self) {
        self.unknown |= other.unknown;
        for formal in &other.formals {
            if !self.formals.contains(formal) {
                self.formals.push(formal.clone());
            }
        }
        self.formals.sort();
    }

    pub(crate) fn unknown() -> Self {
        Self {
            unknown: true,
            formals: Vec::new(),
        }
    }

    pub(crate) fn projected(mut self, fields: &[u32]) -> Self {
        self.formals.retain_mut(|formal| {
            if !formal.value_fields.starts_with(fields) {
                return false;
            }
            formal.value_fields.drain(..fields.len());
            true
        });
        self
    }

    pub(crate) fn enum_payload(mut self, variant: u32, field: u32) -> Self {
        self.formals.retain_mut(|origin| match origin.variant {
            Some(actual) if actual != variant => false,
            Some(_) => {
                if origin.value_fields.first() != Some(&field) {
                    return false;
                }
                origin.value_fields.remove(0);
                origin.variant = None;
                true
            }
            // A formal whole-enum route cannot distinguish variants. Keep it
            // conservatively, but expose it as the selected payload's root.
            None => {
                origin.value_fields.clear();
                true
            }
        });
        self
    }

    pub(crate) fn replace_path(mut self, fields: &[u32], replacement: Option<Self>) -> Self {
        if fields.is_empty() {
            return replacement.unwrap_or_else(Self::fresh);
        }
        self.formals
            .retain(|origin| !origin.value_fields.starts_with(fields));
        if let Some(mut replacement) = replacement {
            self.unknown |= replacement.unknown;
            for mut origin in replacement.formals.drain(..) {
                let mut value_fields = fields.to_vec();
                value_fields.extend_from_slice(&origin.value_fields);
                origin.value_fields = value_fields;
                if !self.formals.contains(&origin) {
                    self.formals.push(origin);
                }
            }
            self.formals.sort();
        }
        self
    }
}

/// Closed-world origin summary for one concrete function result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedResultStateOrigin {
    /// The complete result type carries no opaque state identity.
    NoState,
    /// Every result state identity comes from this finite origin set.
    Finite {
        /// Formal paths which may supply the returned state.
        formals: Vec<CheckedResultStatePath>,
    },
    /// The current compiler could not close the result-origin equation.
    Unknown,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct CheckedResultStatePath {
    pub(crate) result_fields: Vec<u32>,
    pub(crate) result_variant: Option<u32>,
    pub(crate) parameter: u32,
    pub(crate) parameter_fields: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedSliceSource {
    Array {
        root: CheckedArrayRoot,
        length: CheckedConst,
    },
    Buffer(CheckedBufferRoot),
    /// An array reached in `arena<'r, T>` content through `deref` [OWN-5,
    /// OWN-10]. Semantic checking admits it; the arena runtime lowering is
    /// not implemented yet, and the temporary arena-parameter implementation
    /// stop keeps it from reaching lowering.
    ArenaContent {
        binding: BindingId,
        fields: Vec<u32>,
        length: CheckedConst,
    },
}

/// Source category retained only for integer-operation operands whose exact
/// written constant class affects an ENT-3 source. This is checked metadata,
/// not a second expression tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedIntegerArgumentSource {
    TypedLiteral,
    GenericNumericIdentity,
    NamedConstant { declaration: DeclarationId },
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedIntegerArgument {
    pub(crate) node_path: NodePath,
    pub(crate) source: CheckedIntegerArgumentSource,
}

/// The caller-side root a bound borrow-mode call result reads and writes
/// through: the resolved place of the single provenance-candidate actual
/// [OWN-6, ENT-5]. The record deliberately keeps the complete actual place
/// even when the callee returned a narrower suffix of it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedResultBorrow {
    pub(crate) binding: BindingId,
    pub(crate) fields: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CheckedExpression {
    Constant(CheckedValue),
    /// A named const read retains declaration identity for exact goal-origin
    /// equality while lowering the same immutable value as before.
    NamedConstant {
        declaration: DeclarationId,
        value: CheckedValue,
    },
    Binding {
        carrier: NodePath,
        binding: BindingId,
        state_origins: Option<CheckedStateOrigins>,
        ty: CheckedType,
        slice_origins: Vec<CheckedSliceOrigin>,
        /// The owning checker admitted this occurrence as an affine consume.
        /// ENT keeps it beside the checked binding because holder mode is not
        /// recoverable from the value type and must not be re-read from syntax.
        consume_root: bool,
    },
    UserCall {
        function: FunctionId,
        /// Exact source call occurrence and declared-order argument atoms.
        call: NodePath,
        argument_nodes: Vec<NodePath>,
        arguments: Vec<CheckedExpression>,
        /// Pre-transfer caller images retained for exact GoalTemplate
        /// substitution after the complete concrete function inventory exists.
        goal_arguments: Vec<super::goal::GoalExpression>,
        /// Concrete caller regions supplied for the callee's formal region
        /// parameters, in declaration order.
        goal_regions: Vec<DeclarationId>,
        /// Filled from the complete phase-A inventory before entailment runs,
        /// in callee `requires_clause` source order.
        requirements: Vec<super::goal::CheckedCallRequirement>,
        result: CheckedType,
        slice_origins: Vec<CheckedSliceOrigin>,
        /// For a borrow-mode result admitted under the reborrow extension:
        /// the caller-side storage the result borrow is conservatively rooted
        /// at — the resolved place of the callee signature's single
        /// provenance-candidate actual. Entailment reads it so a write
        /// through the bound result kills exactly the facts on that storage
        /// [ENT-5]; it is `None` for every own-mode result and whenever the
        /// extension is off.
        result_borrow: Option<CheckedResultBorrow>,
    },
    /// One call to an admitted [SYS-2] system operation, by index into the
    /// system operation catalog. Arguments follow declared parameter order.
    SystemCall {
        operation: u8,
        /// Exact compiler-owned execution metadata selected with the catalog
        /// row. It is not a source effect.
        target_action: crate::TargetAction,
        /// Exact source call occurrence and declared-order argument atoms.
        call: NodePath,
        /// Concrete caller regions supplied for the operation's borrow
        /// parameters, in declaration order.
        regions: Vec<DeclarationId>,
        argument_nodes: Vec<NodePath>,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
    },
    IntegerOperation {
        carrier: NodePath,
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
        argument_metadata: Vec<CheckedIntegerArgument>,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
    },
    FloatOperation {
        carrier: NodePath,
        operation: CheckedFloatOperation,
        operand_type: CheckedType,
        arguments: Vec<CheckedExpression>,
    },
    NumericConversion {
        carrier: NodePath,
        source: CheckedNumericType,
        destination: CheckedNumericType,
        value: Box<CheckedExpression>,
        result: CheckedType,
    },
    Reinterpret {
        carrier: NodePath,
        source: CheckedNumericType,
        destination: CheckedNumericType,
        value: Box<CheckedExpression>,
    },
    BooleanOperation {
        carrier: NodePath,
        operation: CheckedBooleanOperation,
        arguments: Vec<CheckedExpression>,
    },
    EnumEquality {
        carrier: NodePath,
        equal: bool,
        operand_type: CheckedType,
        arguments: Vec<CheckedExpression>,
    },
    ArrayFill {
        carrier: NodePath,
        ty: CheckedType,
        value: Box<CheckedExpression>,
        target_domain: CheckedTargetDomainObligation,
    },
    ArrayLength {
        root: CheckedArrayRoot,
        length: CheckedConst,
    },
    ArrayIndex {
        carrier: NodePath,
        root: CheckedArrayRoot,
        element_type: CheckedType,
        length: CheckedConst,
        offset: Box<CheckedExpression>,
        obligation: NodePath,
        target_domain: CheckedTargetDomainObligation,
    },
    BufferFill {
        carrier: NodePath,
        element: CheckedFlatElement,
        length: Box<CheckedExpression>,
        value: Box<CheckedExpression>,
        layout_ceiling: CheckedLayoutCeiling,
        target_domains: CheckedRuntimeTargetObligations,
    },
    /// One `buffer_vacant::<T>(n)` allocation [OP-1, OP-9]: a flat buffer of
    /// the u64 length whose every element is the compiler-minted `None()`
    /// of the named `Option<T>` instance; no source value is duplicated.
    BufferVacant {
        carrier: NodePath,
        /// The interned `Option<T>` element instance.
        element: NominalId,
        length: Box<CheckedExpression>,
        layout_ceiling: CheckedLayoutCeiling,
        target_domains: CheckedRuntimeTargetObligations,
    },
    /// The canonical total OP-9 allocation-domain predicate. Its Boolean
    /// value is `n <= floor(u64::MAX / stride_ceiling(T))`; it never
    /// allocates and has no partial runtime outcome.
    BufferFits {
        carrier: NodePath,
        element: CheckedType,
        layout_ceiling: CheckedLayoutCeiling,
        length: Box<CheckedExpression>,
    },
    BufferLength {
        root: CheckedBufferRoot,
    },
    BufferIndex {
        carrier: NodePath,
        root: CheckedBufferRoot,
        offset: Box<CheckedExpression>,
        obligation: NodePath,
        target_domain: CheckedTargetDomainObligation,
    },
    SliceOf {
        carrier: NodePath,
        source: CheckedSliceSource,
        region: DeclarationId,
        element: CheckedFlatElement,
        origins: Vec<CheckedSliceOrigin>,
    },
    SliceLength {
        root: CheckedSliceRoot,
    },
    SliceIndex {
        carrier: NodePath,
        root: CheckedSliceRoot,
        offset: Box<CheckedExpression>,
        obligation: NodePath,
        target_domain: CheckedTargetDomainObligation,
    },
    BoxNew {
        carrier: NodePath,
        nominal: NominalId,
        value: Box<CheckedExpression>,
    },
    BoxDeref {
        carrier: NodePath,
        nominal: NominalId,
        referent: CheckedType,
        value: Box<CheckedExpression>,
    },
    /// One `arena_new::<'r, T>(v)` allocation [STOR-2]: the content moves into
    /// region-owned storage registered on the region's allocation list, and
    /// the whole list is released with the region [STOR-3, STOR-4].
    ArenaNew {
        carrier: NodePath,
        nominal: NominalId,
        /// The owning region's hidden allocation-list binding.
        list: BindingId,
        value: Box<CheckedExpression>,
    },
    /// Arena content read through explicit `deref` [STOR-2, TYPE-7].
    ArenaDeref {
        carrier: NodePath,
        nominal: NominalId,
        content: CheckedType,
        value: Box<CheckedExpression>,
    },
    BorrowBuffer {
        carrier: NodePath,
        root: CheckedBufferRoot,
    },
    /// A borrow of directly stored content — a scalar, struct, or enum — which
    /// is the address of the borrowed binding's storage [OWN-2, OWN-5].
    BorrowAddressed {
        carrier: NodePath,
        binding: BindingId,
        ty: CheckedType,
    },
    BorrowBox {
        carrier: NodePath,
        binding: BindingId,
        nominal: NominalId,
    },
    BorrowSystemResource {
        carrier: NodePath,
        binding: BindingId,
        /// The struct-field path from the binding to the borrowed resource,
        /// empty when the binding is the resource itself. A system struct's
        /// direction fields are ordinary field places [SYS-18], so a borrow of
        /// one is this expression with a one-element path.
        fields: Vec<u32>,
        state_origins: Option<CheckedStateOrigins>,
        nominal: NominalId,
    },
    /// The same address, taken from a binding that already holds one: a borrow
    /// whose place is rooted at another borrow holder [OWN-6, OWN-10].
    ReborrowAddressed {
        carrier: NodePath,
        binding: BindingId,
        ty: CheckedType,
    },
    /// The referent value read through such a holder [TYPE-7]. The holder
    /// itself stays a distinct expression, so lowering never has to guess
    /// whether a borrow binding is being passed on or read through.
    DerefAddressed {
        carrier: NodePath,
        binding: BindingId,
        ty: CheckedType,
    },
    ConstructStruct {
        carrier: NodePath,
        nominal: NominalId,
        fields: Vec<CheckedExpression>,
    },
    ConstructEnum {
        carrier: NodePath,
        nominal: NominalId,
        variant: u32,
        fields: Vec<CheckedExpression>,
    },
    Project {
        carrier: NodePath,
        binding: BindingId,
        state_origins: Option<CheckedStateOrigins>,
        fields: Vec<u32>,
        ty: CheckedType,
        consume_root: bool,
        residual_drops: Vec<CheckedProjectedDrop>,
    },
    ProjectValue {
        carrier: NodePath,
        value: Box<CheckedExpression>,
        nominal: NominalId,
        field: u32,
        ty: CheckedType,
    },
}

impl CheckedExpression {
    /// Exact PRV-1 carrier node for a positive explicit-dataflow edge.
    pub(crate) const fn carrier(&self) -> Option<&NodePath> {
        match self {
            Self::Constant(_)
            | Self::NamedConstant { .. }
            | Self::ArrayLength { .. }
            | Self::BufferLength { .. }
            | Self::SliceLength { .. } => None,
            Self::UserCall { call, .. } | Self::SystemCall { call, .. } => Some(call),
            Self::Binding { carrier, .. }
            | Self::IntegerOperation { carrier, .. }
            | Self::FloatOperation { carrier, .. }
            | Self::NumericConversion { carrier, .. }
            | Self::Reinterpret { carrier, .. }
            | Self::BooleanOperation { carrier, .. }
            | Self::EnumEquality { carrier, .. }
            | Self::ArrayFill { carrier, .. }
            | Self::ArrayIndex { carrier, .. }
            | Self::BufferFill { carrier, .. }
            | Self::BufferVacant { carrier, .. }
            | Self::BufferFits { carrier, .. }
            | Self::BufferIndex { carrier, .. }
            | Self::SliceOf { carrier, .. }
            | Self::SliceIndex { carrier, .. }
            | Self::BoxNew { carrier, .. }
            | Self::BoxDeref { carrier, .. }
            | Self::ArenaNew { carrier, .. }
            | Self::ArenaDeref { carrier, .. }
            | Self::BorrowBuffer { carrier, .. }
            | Self::BorrowAddressed { carrier, .. }
            | Self::BorrowBox { carrier, .. }
            | Self::BorrowSystemResource { carrier, .. }
            | Self::ReborrowAddressed { carrier, .. }
            | Self::DerefAddressed { carrier, .. }
            | Self::ConstructStruct { carrier, .. }
            | Self::ConstructEnum { carrier, .. }
            | Self::Project { carrier, .. }
            | Self::ProjectValue { carrier, .. } => Some(carrier),
        }
    }

    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Constant(value) => value.ty(),
            Self::NamedConstant { value, .. } => value.ty(),
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
            Self::BufferVacant { element, .. } => CheckedType::Buffer {
                element: CheckedFlatElement::Nominal(*element),
            },
            Self::BufferFits { .. } => CheckedType::Bool,
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
            Self::BoxNew { nominal, .. } | Self::ArenaNew { nominal, .. } => {
                CheckedType::Nominal(*nominal)
            }
            Self::BoxDeref { referent, .. } => *referent,
            Self::ArenaDeref { content, .. } => *content,
            Self::BorrowBuffer { root, .. } => CheckedType::Buffer {
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
    pub(crate) node_path: NodePath,
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
    pub(crate) state_origins: Option<CheckedStateOrigins>,
    pub(crate) release: crate::SystemRelease,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedProjectedDrop {
    pub(crate) fields: Vec<u32>,
    pub(crate) ty: CheckedType,
    pub(crate) state_origins: Option<CheckedStateOrigins>,
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
    pub(crate) obligation: NodePath,
    pub(crate) target_domain: CheckedTargetDomainObligation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedBufferSetTarget {
    pub(crate) root: CheckedBufferRoot,
    pub(crate) offset: CheckedExpression,
    pub(crate) obligation: NodePath,
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
        node_path: NodePath,
        binding: BindingId,
        value: CheckedExpression,
    },
    PropagateLet {
        /// Complete owning `let_stmt`, shared by Ok delivery and Err return.
        node_path: NodePath,
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
        node_path: NodePath,
        target: CheckedSetTarget,
        value: CheckedExpression,
    },
    /// A [SET-2] affine-place replacement: one read of the previous value
    /// into the fresh binding and one write of the replacement into the
    /// target, with no writer-observable point between them. The target
    /// root stays live; the commit is not a consuming use.
    Replace {
        node_path: NodePath,
        binding: BindingId,
        target: CheckedSetTarget,
        value: CheckedExpression,
    },
    Evaluate(CheckedExpression),
    /// The discarded result of an expression statement, with the
    /// compiler-derived release it runs [STOR-3].
    DropExpression {
        value: CheckedExpression,
        state_origins: Option<CheckedStateOrigins>,
        release: crate::SystemRelease,
    },
    /// A finite source-written local invariant checked before it is published
    /// and erased before lowering. It has no runtime expression, effect,
    /// branch, or trap.
    Proof(CheckedSourceProof),
    Return {
        node_path: NodePath,
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
        node_path: NodePath,
        kind: ValueInitializerKind,
        binding: BindingId,
        result_type: CheckedType,
        scrutinee: CheckedExpression,
        enum_type: CheckedEnumType,
        arms: Vec<CheckedMatchArm>,
        continues: bool,
    },
    Give {
        node_path: NodePath,
        value: CheckedExpression,
        drops: Vec<CheckedDrop>,
    },
    Loop {
        id: CheckedLoopId,
        /// Formed source invariants awaiting the normal semantic proof
        /// checker. Their presence alone grants no authority.
        invariants: Vec<CheckedLoopInvariant>,
        body: Vec<CheckedStatement>,
        backedge_drops: Vec<CheckedDrop>,
    },
    CountedRange {
        id: CheckedLoopId,
        node_path: NodePath,
        binder: BindingId,
        lower: CheckedExpression,
        upper: CheckedExpression,
        /// Formed source invariants awaiting the normal semantic proof
        /// checker. Their presence alone grants no authority.
        invariants: Vec<CheckedLoopInvariant>,
        body: Vec<CheckedStatement>,
        backedge_drops: Vec<CheckedDrop>,
    },
    Break {
        target: CheckedLoopId,
        drops: Vec<CheckedDrop>,
    },
    Region {
        /// The region's hidden arena allocation-list binding, present exactly
        /// when the block allocates into this region [STOR-2]. Lowering
        /// materializes it at region entry; its compiler-derived drop on
        /// every normal exit edge is the region's storage release
        /// [STOR-3, STOR-4].
        arena_list: Option<BindingId>,
        body: Vec<CheckedStatement>,
        fallthrough_drops: Vec<CheckedDrop>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedParameter {
    pub(crate) name: String,
    pub(crate) declaration: DeclarationId,
    /// The complete source `param` node used by checked diagnostics.
    pub(crate) node_path: NodePath,
    pub(crate) binding: BindingId,
    pub(crate) mode: CheckedMode,
    pub(crate) ty: CheckedType,
    pub(crate) slice_origins: Vec<CheckedSliceOrigin>,
}

/// One callable-boundary state identity.
///
/// `root` is a formal value-parameter declaration and `fields` are source
/// struct ordinals.  Lifetimes deliberately do not participate in identity.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct CheckedStatePath {
    pub(crate) root: DeclarationId,
    pub(crate) fields: Vec<u32>,
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
    /// Closed-world state origin of this function's result.
    pub(crate) result_state_origin: CheckedResultStateOrigin,
    pub(crate) slice_return_ceiling: Vec<CheckedSliceOrigin>,
    pub(crate) declared_allocates_heap: bool,
    /// Formal state paths named by `writes(...)`.
    pub(crate) declared_state_writes: Vec<CheckedStatePath>,
    /// Conservative fixed-point summary of every reachable target action.
    pub(crate) target_action: crate::TargetAction,
    /// Callable-boundary predicates in `requires_clause` source order.
    pub(crate) requirements: Vec<super::goal::CheckedRequirement>,
    /// Verified-relation surfaces in `ensures_clause` source order. H1
    /// constructs this metadata; the shared entailment flow proves every
    /// clause at every selected exit.
    pub(crate) postconditions: Vec<super::postcondition::CheckedPostcondition>,
    pub(crate) body: Vec<CheckedStatement>,
    /// Whether the independently established body-entry requirements close to
    /// a contradiction. The contradiction is retained proof metadata.
    pub(crate) body_disposition: CheckedBodyDisposition,
    /// Retained [ENT] analysis summary [DIAG-2]. Semantic acceptance and
    /// diagnostics read it; lowering deliberately does not.
    #[allow(dead_code)]
    pub(crate) entailment: super::entailment::FunctionEntailment,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum CheckedBodyDisposition {
    #[default]
    Inhabited,
    Uninhabited {
        contradiction: super::entailment::DerivationId,
    },
}

/// The one source-canonical symbolic requirement retained for a generic
/// function template.
///
/// This is acceptance metadata only. It deliberately has no [`FunctionId`]
/// and ordinary lowering must not treat it as an executable instance.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedGenericRequirement {
    pub(crate) declaration: DeclarationId,
    pub(crate) requirement: super::goal::CheckedRequirement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedEffects {
    pub(crate) reads: Vec<CheckedStatePath>,
    pub(crate) writes: Vec<CheckedStatePath>,
    pub(crate) allocates_heap: bool,
    pub(crate) allocates_arenas: Vec<DeclarationId>,
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
    pub(crate) effects: CheckedEffects,
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
/// The only admitted entry is `command`. Lowering retains the standard-input
/// table ordinals in declaration order because ordinal identity — never type
/// identity — selects each supplied value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedEntryForm {
    /// Selected [FN-7] table ordinals in strictly increasing order.
    pub(crate) inputs: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct CheckedProgramData {
    /// Which [SYS-2] inventory this unit was resolved and checked against.
    ///
    /// Carried here because a `CheckedConstructor::System` holds a declaration
    /// ordinal, and an ordinal is only meaningful against the inventory that
    /// assigned it: lowering decodes those ordinals and must use this one
    /// rather than the shipped active state. Reading the active state here was
    /// a latent defect that only showed once an inventory state changed the
    /// size of the nominal-record block ahead of the constructor block.
    pub(crate) inventory: crate::Inventory,
    pub(crate) nominals: Vec<CheckedNominal>,
    // Nominal instances discovered by the ordinary function path form this
    // prefix. Later instances exist only to type-check static metadata.
    pub(crate) executable_nominal_count: usize,
    pub(crate) constants: Vec<CheckedConstant>,
    /// Immutable structural table for every symbolic const expression named
    /// by retained schema metadata. `DerivedConstId` is meaningful only
    /// relative to this checked-program-owned table.
    #[allow(dead_code)]
    pub(crate) derived_consts: Vec<DerivedConst>,
    pub(crate) functions: Vec<CheckedFunction>,
    /// Concrete ordinary-call SCCs in deterministic callee-before-caller
    /// order, with component-atomic verified FN-9 summary publication.
    #[allow(dead_code)]
    pub(crate) postcondition_schedule: super::entailment::PostconditionSchedule,
    /// One symbolic requirement per source generic that declares one. These
    /// entries survive symbolic validation without entering the concrete
    /// function inventory or executable lowering path.
    #[allow(dead_code)]
    pub(crate) generic_requirements: Vec<CheckedGenericRequirement>,
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
    /// Read-only [PAR-1 candidate] permission table: which sibling call pairs
    /// may be overlapped, and which of those are actualizable. Acceptance
    /// never reads it, and it is identical facts-on and facts-off. The
    /// permission ledger and the overlap lowering are its consumers.
    pub(crate) permission: super::permission::PermissionMetadata,
    /// The rendered non-normative permission ledger, one line per analyzed
    /// source site in source order, each marked with whether an ordinary
    /// compile reports it. This is developer output only: the driver hands the
    /// whole report to `whitefootc --par-ledger` and the marked subset to every
    /// compile, and no mandatory record, no normative output, and no lowering
    /// decision reads it.
    pub(crate) permission_ledger: Vec<super::permission_ledger::LedgerLine>,
}

/// Every direct subexpression, for uniform recursion.
pub(crate) fn expression_children(expression: &CheckedExpression) -> Vec<&CheckedExpression> {
    match expression {
        CheckedExpression::Constant(_)
        | CheckedExpression::NamedConstant { .. }
        | CheckedExpression::Binding { .. }
        | CheckedExpression::ArrayLength { .. }
        | CheckedExpression::BufferLength { .. }
        | CheckedExpression::SliceLength { .. }
        | CheckedExpression::SliceOf { .. }
        | CheckedExpression::BorrowBuffer { .. }
        | CheckedExpression::BorrowAddressed { .. }
        | CheckedExpression::BorrowBox { .. }
        | CheckedExpression::BorrowSystemResource { .. }
        | CheckedExpression::ReborrowAddressed { .. }
        | CheckedExpression::DerefAddressed { .. }
        | CheckedExpression::Project { .. } => Vec::new(),
        CheckedExpression::UserCall { arguments, .. }
        | CheckedExpression::SystemCall { arguments, .. }
        | CheckedExpression::IntegerOperation { arguments, .. }
        | CheckedExpression::FloatOperation { arguments, .. }
        | CheckedExpression::BooleanOperation { arguments, .. }
        | CheckedExpression::EnumEquality { arguments, .. } => arguments.iter().collect(),
        CheckedExpression::NumericConversion { value, .. }
        | CheckedExpression::Reinterpret { value, .. }
        | CheckedExpression::ArrayFill { value, .. }
        | CheckedExpression::BoxNew { value, .. }
        | CheckedExpression::BoxDeref { value, .. }
        | CheckedExpression::ArenaNew { value, .. }
        | CheckedExpression::ArenaDeref { value, .. }
        | CheckedExpression::ProjectValue { value, .. } => vec![value.as_ref()],
        CheckedExpression::ArrayIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::BufferFill { length, value, .. } => {
            vec![length.as_ref(), value.as_ref()]
        }
        CheckedExpression::BufferVacant { length, .. }
        | CheckedExpression::BufferFits { length, .. } => vec![length.as_ref()],
        CheckedExpression::BufferIndex { offset, .. }
        | CheckedExpression::SliceIndex { offset, .. } => vec![offset.as_ref()],
        CheckedExpression::ConstructStruct { fields, .. }
        | CheckedExpression::ConstructEnum { fields, .. } => fields.iter().collect(),
    }
}
