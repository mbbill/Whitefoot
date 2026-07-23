use crate::{DeclarationId, NodePath, PreludeDeclarationId};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FunctionId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct BindingId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CheckedLoopId(pub(crate) u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NominalId(pub(crate) u32);

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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CheckedType {
    Unit,
    Bool,
    Integer(IntegerType),
    Nominal(NominalId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CheckedValue {
    Unit,
    Bool(bool),
    Integer { ty: IntegerType, bits: u64 },
}

impl CheckedValue {
    pub(crate) const fn ty(self) -> CheckedType {
        match self {
            Self::Unit => CheckedType::Unit,
            Self::Bool(_) => CheckedType::Bool,
            Self::Integer { ty, .. } => CheckedType::Integer(ty),
        }
    }
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
    AbsoluteWrap,
    AbsoluteTrap,
    AbsoluteChecked,
    NegateWrap,
    NegateTrap,
    NegateChecked,
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

impl CheckedIntegerOperation {
    pub(crate) const fn traps(self) -> bool {
        matches!(
            self,
            Self::AddTrap
                | Self::SubtractTrap
                | Self::MultiplyTrap
                | Self::AbsoluteTrap
                | Self::NegateTrap
        )
    }

    pub(crate) const fn operand_count(self) -> usize {
        match self {
            Self::AbsoluteWrap
            | Self::AbsoluteTrap
            | Self::AbsoluteChecked
            | Self::NegateWrap
            | Self::NegateTrap
            | Self::NegateChecked => 1,
            _ => 2,
        }
    }

    pub(crate) const fn signed_only(self) -> bool {
        matches!(
            self,
            Self::AbsoluteWrap
                | Self::AbsoluteTrap
                | Self::AbsoluteChecked
                | Self::NegateWrap
                | Self::NegateTrap
                | Self::NegateChecked
        )
    }

    pub(crate) const fn scalar_result_type(self, operand: IntegerType) -> Option<CheckedType> {
        match self {
            Self::AddChecked
            | Self::SubtractChecked
            | Self::MultiplyChecked
            | Self::DivideChecked
            | Self::RemainderChecked
            | Self::AbsoluteChecked
            | Self::NegateChecked => None,
            Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::LessEqual
            | Self::Greater
            | Self::GreaterEqual => Some(CheckedType::Bool),
            _ => Some(CheckedType::Integer(operand)),
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
        operand_type: IntegerType,
        arguments: Vec<CheckedExpression>,
        result: CheckedType,
        trap: Option<TrapSite>,
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
}

impl CheckedExpression {
    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Constant(value) => value.ty(),
            Self::Binding { ty, .. } | Self::UserCall { result: ty, .. } => *ty,
            Self::IntegerOperation { result, .. } => *result,
            Self::BooleanOperation { .. } | Self::EnumEquality { .. } => CheckedType::Bool,
            Self::ConstructStruct { nominal, .. } | Self::ConstructEnum { nominal, .. } => {
                CheckedType::Nominal(*nominal)
            }
            Self::Project { ty, .. } => *ty,
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
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedMatchArm {
    pub(crate) tag: u32,
    pub(crate) binders: Vec<CheckedMatchBinder>,
    pub(crate) body: Vec<CheckedStatement>,
    pub(crate) fallthrough_drops: Vec<CheckedDrop>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CheckedDrop {
    pub(crate) binding: BindingId,
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
        target: CheckedWritablePlace,
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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedParameter {
    pub(crate) name: String,
    pub(crate) binding: BindingId,
    pub(crate) ty: CheckedType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedFunction {
    pub(crate) id: FunctionId,
    pub(crate) declaration: DeclarationId,
    pub(crate) name: String,
    pub(crate) parameters: Vec<CheckedParameter>,
    pub(crate) result: CheckedType,
    pub(crate) declared_traps: bool,
    pub(crate) body: Vec<CheckedStatement>,
}

#[derive(Debug)]
pub(crate) struct CheckedProgramData {
    pub(crate) nominals: Vec<CheckedNominal>,
    pub(crate) functions: Vec<CheckedFunction>,
    pub(crate) main: FunctionId,
}
