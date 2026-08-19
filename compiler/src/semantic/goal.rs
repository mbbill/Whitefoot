use crate::{DeclarationId, NodePath};

use super::model::{
    BindingId, CheckedBooleanOperation, CheckedConst, CheckedFlatElement, CheckedFloatOperation,
    CheckedIntegerOperation, CheckedNumericType, CheckedType, CheckedValue, FunctionId, TrapSite,
};

/// One function requirement, split into predicate and occurrence identity.
///
/// The final-check path belongs to diagnostics and retained metadata. It is
/// deliberately outside [`GoalTemplate`]'s equality, so two requirements with
/// the same alpha-expanded typed predicate compare equal even when their
/// source occurrences differ.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedRequirement {
    pub(crate) template: GoalTemplate,
    /// Exact OP-5 dynamic-boundary record of the final source check.
    ///
    /// Its node path is also the requirement occurrence path. Keeping the
    /// complete record lets entry lowering survive removal of the legacy
    /// executable clause statements without re-reading source.
    pub(crate) trap: TrapSite,
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
/// function id. The final-check path is occurrence/provenance identity only;
/// it is deliberately absent from `ConcreteGoal` equality.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedCallRequirement {
    pub(crate) final_check: NodePath,
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

/// A source-stable leaf of a goal template.
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
    /// The already-evaluated pre-transfer value of one direct-subscript
    /// actual. Source cannot name this datum, so it is scoped to its exact call
    /// occurrence and contributes no place support.
    EphemeralActual {
        caller: FunctionId,
        call: NodePath,
        argument: u32,
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
            | Self::EphemeralActual { ty, .. } => *ty,
            Self::Literal(value) => value.ty(),
        }
    }

    fn projections_mut(&mut self) -> &mut Vec<GoalProjection> {
        match self {
            Self::Parameter { projections, .. }
            | Self::NamedConst { projections, .. }
            | Self::Place { projections, .. }
            | Self::EphemeralActual { projections, .. } => projections,
            Self::Literal(_) => unreachable!("a literal cannot carry a place projection"),
        }
    }

    fn set_ty(&mut self, new_ty: CheckedType) {
        match self {
            Self::Parameter { ty, .. }
            | Self::NamedConst { ty, .. }
            | Self::Place { ty, .. }
            | Self::EphemeralActual { ty, .. } => *ty = new_ty,
            Self::Literal(_) => unreachable!("a literal cannot carry a place projection"),
        }
    }
}

/// Ordered projection identity within one formal or named-constant datum.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum GoalProjection {
    Deref,
    Field(u32),
}

/// Exact selected operation-table row and its operand-domain identity.
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
    ArrayLength {
        element: CheckedFlatElement,
        length: CheckedConst,
    },
    BufferLength {
        element: CheckedFlatElement,
    },
    /// Canonical total allocation-domain predicate [OP-9]. The ceiling is
    /// part of the row identity so a proof cannot be reused across a layout
    /// rule change or across distinct element representations.
    BufferFits {
        element: CheckedType,
        maximum_length: u64,
    },
    SliceLength {
        region: DeclarationId,
        element: CheckedFlatElement,
    },
}

/// Deterministic complete diagnostic rendering of one typed goal. The active
/// specification fixes the payload's contents but not a cross-implementation
/// byte encoding; this compiler nevertheless uses one stable structural form.
pub(crate) fn render_goal(expression: &GoalExpression) -> String {
    match expression {
        GoalExpression::Datum(GoalDatum::EphemeralActual {
            caller,
            call,
            argument,
            captured_type,
            projections,
            ty,
        }) => format!(
            "argument #{argument} pre-transfer value \
             (caller={caller:?}, call={call:?}, captured={captured_type:?}, \
             projections={projections:?}, type={ty:?})"
        ),
        GoalExpression::Datum(datum) => format!("{datum:?}"),
        GoalExpression::Operation {
            row,
            type_arguments,
            const_arguments,
            result,
            arguments,
        } => {
            let arguments = arguments
                .iter()
                .map(render_goal)
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{row:?}<types={type_arguments:?}, consts={const_arguments:?}>({arguments}):{result:?}"
            )
        }
    }
}

/// First occurrence-local actual value in structural operand order, when a
/// call goal needs FN-8's stronger bind-then-prove restructuring.
pub(crate) fn first_ephemeral_argument(expression: &GoalExpression) -> Option<u32> {
    match expression {
        GoalExpression::Datum(GoalDatum::EphemeralActual { argument, .. }) => Some(*argument),
        GoalExpression::Operation { arguments, .. } => {
            arguments.iter().find_map(first_ephemeral_argument)
        }
        GoalExpression::Datum(_) => None,
    }
}
