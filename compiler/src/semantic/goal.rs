use crate::{DeclarationId, NodePath};

use super::model::{
    BindingId, CheckedBooleanOperation, CheckedConst, CheckedElement, CheckedFlatElement,
    CheckedFloatOperation, CheckedIntegerOperation, CheckedMeasure, CheckedNumericType,
    CheckedType, CheckedValue, FunctionId, MeasuredKind,
};

/// One function requirement, split into predicate and occurrence identity.
///
/// The requires-clause path belongs to diagnostics and retained metadata. It is
/// deliberately outside [`GoalTemplate`]'s equality, so two requirements with
/// the same alpha-expanded typed predicate compare equal even when their
/// source occurrences differ.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedRequirement {
    pub(crate) template: GoalTemplate,
    /// Exact `requires_clause` occurrence. It is diagnostic/provenance
    /// identity only and never an executable trap record.
    pub(crate) clause: NodePath,
}

/// The finite typed predicate carried by one [FN-8] callable boundary.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct GoalTemplate {
    pub(crate) root: GoalExpression,
}

impl GoalTemplate {
    pub(crate) fn new(root: GoalExpression) -> Self {
        Self { root }
    }
}

/// One fully substituted goal at a concrete call occurrence.
///
/// This wrapper prevents a caller from accidentally treating a formal-bearing
/// template as a caller-state predicate. Construction succeeds only after
/// every formal datum has been replaced by its pre-transfer actual image.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ConcreteGoal {
    pub(crate) root: GoalExpression,
}

impl ConcreteGoal {
    pub(crate) fn new(root: GoalExpression) -> Self {
        Self { root }
    }
}

/// The concrete predicate and source occurrence retained for one user call.
///
/// The callee instance is the surrounding `CheckedExpression::UserCall`
/// function id. The requires-clause path is occurrence/provenance identity only;
/// it is deliberately absent from `ConcreteGoal` equality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedCallRequirement {
    pub(crate) requires_clause: NodePath,
    pub(crate) goal: ConcreteGoal,
}

/// One node of an alpha-expanded requirement predicate.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GoalExpression {
    Datum(GoalDatum),
    Operation {
        row: GoalOperation,
        /// Written operation type arguments after generic substitution.
        type_arguments: Vec<CheckedType>,
        /// Written operation const arguments after generic substitution.
        const_arguments: Vec<CheckedConst>,
        result: CheckedType,
        arguments: Vec<GoalExpression>,
    },
}

impl GoalExpression {
    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Datum(datum) => datum.ty(),
            Self::Operation { result, .. } => *result,
        }
    }

    pub(crate) fn with_projection(
        mut self,
        projection: GoalProjection,
        ty: CheckedType,
    ) -> Option<Self> {
        let Self::Datum(datum) = &mut self else {
            return None;
        };
        datum.projections_mut().push(projection);
        datum.set_ty(ty);
        Some(self)
    }
}

/// One typed leaf in a requirement template or concrete proof goal.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GoalDatum {
    /// A formal is identified by position, not source spelling or declaration.
    Parameter {
        ordinal: u32,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    /// Named constants retain declaration identity instead of collapsing to
    /// their evaluated value.
    NamedConst {
        declaration: DeclarationId,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    /// One caller place after lexical resolution. Borrow and reborrow actuals
    /// use their ultimate referent root rather than the temporary holder.
    Place {
        root: BindingId,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    /// One already-evaluated value that source cannot safely name again.
    ///
    /// The value is scoped to its exact source occurrence and contributes no
    /// place support: it is the immutable mathematical result already
    /// produced at that point, not permission to reread the source expression.
    EvaluatedValue {
        function: FunctionId,
        occurrence: EvaluatedValueOccurrence,
        captured_type: CheckedType,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    /// Literals retain their exact checked type and mathematical/nominal value.
    Literal(CheckedValue),
}

impl GoalDatum {
    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Parameter { ty, .. }
            | Self::NamedConst { ty, .. }
            | Self::Place { ty, .. }
            | Self::EvaluatedValue { ty, .. } => *ty,
            Self::Literal(value) => value.ty(),
        }
    }

    fn projections_mut(&mut self) -> &mut Vec<GoalProjection> {
        match self {
            Self::Parameter { projections, .. }
            | Self::NamedConst { projections, .. }
            | Self::Place { projections, .. }
            | Self::EvaluatedValue { projections, .. } => projections,
            Self::Literal(_) => unreachable!("a literal cannot carry a place projection"),
        }
    }

    fn set_ty(&mut self, new_ty: CheckedType) {
        match self {
            Self::Parameter { ty, .. }
            | Self::NamedConst { ty, .. }
            | Self::Place { ty, .. }
            | Self::EvaluatedValue { ty, .. } => *ty = new_ty,
            Self::Literal(_) => unreachable!("a literal cannot carry a place projection"),
        }
    }
}

/// Structural identity for an already-evaluated, occurrence-local value.
///
/// Call actuals remain distinguishable from proof-obligation operands so
/// FN-8's bind-first diagnostic cannot be selected for OP-2, OP-9, or SYS-8.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum EvaluatedValueOccurrence {
    CallArgument { call: NodePath, argument: u32 },
    ObligationOperand { site: NodePath, operand: u32 },
}

/// Ordered projection identity within one formal or named-constant datum.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GoalProjection {
    Deref,
    Field(u32),
}

/// One structural goal row and its exact selected type/domain identity.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GoalOperation {
    Integer {
        operation: CheckedIntegerOperation,
        operand_type: CheckedType,
    },
    Float {
        operation: CheckedFloatOperation,
        operand_type: CheckedType,
    },
    NumericConversion {
        source: CheckedNumericType,
        destination: CheckedNumericType,
    },
    Reinterpret {
        source: CheckedNumericType,
        destination: CheckedNumericType,
    },
    Boolean(CheckedBooleanOperation),
    EnumEquality {
        equal: bool,
        operand_type: CheckedType,
    },
    /// Pure, total `array_new`. FN-8's copy-only clause-local rule keeps this
    /// out of GoalTemplates, but ENT-3 body-origin expansion may retain it.
    ArrayFill {
        element: CheckedFlatElement,
        length: CheckedConst,
    },
    ArrayMeasure {
        measure: CheckedMeasure,
        element: CheckedFlatElement,
        length: CheckedConst,
    },
    /// One array element value whose own OP-4 obligation has already been
    /// discharged before this expression is used as a proof operand.
    ArrayIndex {
        element: CheckedFlatElement,
        length: CheckedConst,
    },
    BufferMeasure {
        measure: CheckedMeasure,
        element: CheckedFlatElement,
    },
    /// One buffer element value whose own OP-4 obligation has already been
    /// discharged before this expression is used as a proof operand.
    BufferIndex {
        element: CheckedFlatElement,
    },
    /// Canonical total allocation-domain predicate [OP-9]. The ceiling is
    /// part of the row identity so a proof cannot be reused across a layout
    /// rule change or across distinct element representations.
    BufferFits {
        element: CheckedType,
        maximum_length: u64,
    },
    SliceMeasure {
        measure: CheckedMeasure,
        region: DeclarationId,
        element: CheckedFlatElement,
    },
    /// One [MSR-1] measure of a run [BLK-1] or a bump extent [PROV-1]. The
    /// measured kind is part of the row identity because the measure table
    /// gives each its own row, and the written constant is what a
    /// `FixedVector`'s capacity and an `Arena`'s byte extent are [MSR-2].
    ContainerMeasure {
        measure: CheckedMeasure,
        measured: MeasuredKind,
        /// The element type of a run; a bump extent has none.
        element: Option<CheckedElement>,
        /// A `FixedVector`'s capacity or an `Arena`'s byte extent; a
        /// `Vector`'s capacity is a descriptor word and has none.
        constant: Option<CheckedConst>,
    },
    /// One run element value whose own [OP-4] obligation has already been
    /// discharged before this expression is used as a proof operand.
    RunIndex {
        measured: MeasuredKind,
        element: CheckedElement,
        constant: Option<CheckedConst>,
    },
    /// One slice element value whose own OP-4 obligation has already been
    /// discharged before this expression is used as a proof operand.
    SliceIndex {
        region: DeclarationId,
        element: CheckedFlatElement,
    },
}

/// First occurrence-local actual value in structural operand order, when a
/// call goal needs FN-8's stronger bind-then-prove restructuring.
pub(crate) fn first_ephemeral_argument(expression: &GoalExpression) -> Option<u32> {
    match expression {
        GoalExpression::Datum(GoalDatum::EvaluatedValue {
            occurrence: EvaluatedValueOccurrence::CallArgument { argument, .. },
            ..
        }) => Some(*argument),
        GoalExpression::Operation { arguments, .. } => {
            arguments.iter().find_map(first_ephemeral_argument)
        }
        GoalExpression::Datum(_) => None,
    }
}
