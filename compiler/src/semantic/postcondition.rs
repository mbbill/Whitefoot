//! Checked FN-9 declaration surface shared with the later callee-proof pass.
//!
//! This module deliberately contains only source-derived, concrete-instance
//! metadata.  The symbolic result is an identity in a relation template, not
//! a declaration, [`super::model::BindingId`], storage location, or executable
//! expression.  Task 0060 consumes these values to prove selected exits; H1
//! stops explicitly before that proof.

use crate::{DeclarationId, NodePath, PreludeDeclarationId, SourceOrigin};

use super::goal::GoalProjection;
use super::model::{
    BindingId, CheckedIntegerOperation, CheckedMeasure, CheckedType, CheckedValue, FunctionId,
};

/// One admitted concrete selector and its resolver-owned source identities.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedPostconditionSelector {
    pub(crate) function: FunctionId,
    pub(crate) block: NodePath,
    pub(crate) selector: NodePath,
    pub(crate) candidate: SourceOrigin,
    /// The declared result ordinal this clause's result datum names
    /// [CALL-4]. A declaration that writes one result has ordinal zero, the
    /// route writes no ordinal binder, and every clause names it.
    pub(crate) ordinal: u32,
    pub(crate) variant: Option<PreludeDeclarationId>,
    pub(crate) field: Option<PostconditionFieldIdentity>,
    pub(crate) result_type: CheckedType,
}

/// Exact PRE-1 field identity and written origin of `Ok(value: result)`.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionFieldIdentity {
    pub(crate) declaration: PreludeDeclarationId,
    pub(crate) origin: SourceOrigin,
}

/// One alpha-expanded, output-bearing L0 relation occurrence.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RelationTemplate {
    pub(crate) operation: CheckedIntegerOperation,
    pub(crate) operands: [RelationDatum; 2],
    pub(crate) normalized: NormalizedRelation,
}

/// Direction-normalized relation shape. Equality denotes its ordinary pair
/// of non-strict bounds; disequality remains its one L0 disequality.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedRelation {
    Equal,
    NotEqual,
    /// `operands[left] <(=) operands[right]` after reversing `>`/`>=`.
    UpperBound {
        left: u8,
        right: u8,
        strict: bool,
    },
}

/// One closed FN-9 RelationTemplate datum after clause-local expansion.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RelationDatum {
    Result {
        /// The declared result ordinal this datum names [CALL-4]. A
        /// declaration writing one result has ordinal zero.
        ordinal: u32,
        ty: CheckedType,
    },
    Parameter {
        ordinal: u32,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    NamedConst {
        declaration: DeclarationId,
        projections: Vec<GoalProjection>,
        ty: CheckedType,
    },
    Literal {
        value: CheckedValue,
        origin: PostconditionConstantOrigin,
    },
    Measure(CheckedMeasure, PostconditionPlace),
}

impl RelationDatum {
    pub(crate) const fn ty(&self) -> CheckedType {
        match self {
            Self::Result { ty, .. } | Self::Parameter { ty, .. } | Self::NamedConst { ty, .. } => {
                *ty
            }
            Self::Literal { value, .. } => value.ty(),
            Self::Measure(..) => CheckedType::Integer(super::model::IntegerType::U64),
        }
    }

    /// Whether this datum names a declared result ordinal [CALL-4], as the
    /// value itself or as a measure over that ordinal's place.
    pub(crate) const fn contains_result(&self) -> bool {
        match self {
            Self::Result { .. } => true,
            Self::Measure(_, place) => {
                matches!(place.root, PostconditionPlaceRoot::Result { .. })
            }
            Self::Parameter { .. } | Self::NamedConst { .. } | Self::Literal { .. } => false,
        }
    }
}

/// Source category of a constant-valued atom.  Concrete substitution never
/// erases whether mathematical one came from a literal, named constant, or a
/// generic source; task 0060's closed S7 admission depends on that distinction.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PostconditionConstantOrigin {
    Literal,
    NamedConst(DeclarationId),
    GenericNumericIdentity {
        type_parameter: DeclarationId,
        one: bool,
    },
    /// One in-scope const generic named as a clause operand [MSR-6]. Its
    /// value lives in the datum itself, concrete or symbolic; this origin
    /// retains only the declaration the spelling resolved to.
    ConstGeneric {
        declaration: DeclarationId,
    },
}

/// A parameter or named-const place accepted below `len`.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PostconditionPlace {
    pub(crate) root: PostconditionPlaceRoot,
    pub(crate) projections: Vec<GoalProjection>,
    pub(crate) ty: CheckedType,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PostconditionPlaceRoot {
    Parameter {
        ordinal: u32,
    },
    /// One declared result ordinal [CALL-4]: a measure over an admitted
    /// result place is an operand with no per-family admission, exactly as a
    /// measure over an admitted formal place is.
    Result {
        ordinal: u32,
    },
}

/// One selected explicit normal return, already classified by H1 so H2 need
/// not re-recognize syntax or infer a Result route.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SelectedPostconditionReturn {
    pub(crate) statement: NodePath,
    /// The datum each declared result ordinal evaluates to at this return, in
    /// written order [CALL-4]. An ordinal the return leaves outside the
    /// [ENT-2] term fragment has no datum; a clause naming it is not selected
    /// at this return. A declaration writing one result has one entry.
    pub(crate) values: Vec<Option<PostconditionReturnDatum>>,
}

/// Exact ENT-2-shaped datum evaluated at a selected return.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PostconditionReturnDatum {
    Place(PostconditionReturnPlace),
    Literal {
        value: CheckedValue,
        origin: PostconditionConstantOrigin,
    },
    Measure(CheckedMeasure, PostconditionReturnPlace),
}

/// Complete checked place identity for a selected result term or `len(P)`.
/// Field and dereference projections remain ordered and the root retains its
/// binding/constant class, so the proof pass never re-walks source syntax.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct PostconditionReturnPlace {
    pub(crate) root: PostconditionReturnPlaceRoot,
    pub(crate) projections: Vec<GoalProjection>,
    pub(crate) ty: CheckedType,
    pub(crate) source: NodePath,
}

impl PartialEq for PostconditionReturnPlace {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.projections == other.projections && self.ty == other.ty
    }
}

impl Eq for PostconditionReturnPlace {}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PostconditionReturnPlaceRoot {
    Binding(BindingId),
    NamedConst(DeclarationId),
}

/// Complete checked H1 handoff for one concrete FN-9 occurrence.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckedPostcondition {
    pub(crate) selector: CheckedPostconditionSelector,
    /// Concrete declaration-to-argument identity in written generic order.
    /// These entries are source-free template identity; occurrence paths live
    /// on the surrounding selector and selected returns instead.
    pub(crate) type_substitutions: Vec<(DeclarationId, CheckedType)>,
    pub(crate) const_substitutions: Vec<(DeclarationId, u64)>,
    pub(crate) relation: RelationTemplate,
    pub(crate) selected_returns: Vec<SelectedPostconditionReturn>,
}
