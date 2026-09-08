use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::entailment::affine::{
    AffineCheckError, AffineCheckState, AffineExpression, AffineTermId,
    normalize_bounded_less_equal,
};
use super::super::super::model::{
    CheckedAffineExpression, CheckedAffineExpressionKind, CheckedAffineRelation, CheckedExpression,
    CheckedMode, CheckedProofMultiplicity, CheckedProofUse, CheckedProofUseSource,
    CheckedSourceProof, CheckedStatement, CheckedType, CheckedValue, IntegerType,
};
use super::super::{CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding};
use super::StatementResult;

/// The semantic owner of the shared proof-only affine expression grammar.
/// The syntax and arithmetic limits are identical; only lookup roles and
/// source diagnostics differ.
#[derive(Clone, Copy)]
pub(super) enum AffineProofOwner {
    InvariantTarget,
    ProofUse,
}

impl AffineProofOwner {
    const fn value_role(self) -> LexicalUseRole {
        match self {
            Self::InvariantTarget => LexicalUseRole::InvariantValue,
            Self::ProofUse => LexicalUseRole::ProofValue,
        }
    }
}

#[derive(Clone, Copy)]
struct OrderedRelationNormalization {
    reverse: bool,
    bound: i128,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_local_invariant(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        function: &FunctionSignature,
        loop_depth: usize,
    ) -> Result<StatementResult, CheckStop> {
        let declaration = self.declaration_at(node, crate::DeclarationRole::Invariant)?;
        let identifiers = self.tree.direct_identifiers(node)?;
        let [name_token] = identifiers.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        let name = std::str::from_utf8(self.tree.token_bytes(*name_token)?)
            .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding)?
            .to_owned();
        if name != declaration.spelling() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }

        let allowed_values = bindings.keys().copied().collect::<HashSet<_>>();
        let target = self.check_ordered_affine_relation(
            node,
            bindings,
            &allowed_values,
            function,
            loop_depth,
            AffineProofOwner::InvariantTarget,
        )?;
        let premise_nodes = self.tree.children_with(node, Production::ProofUse)?;
        let mut uses = Vec::with_capacity(premise_nodes.len());
        for premise_node in premise_nodes {
            let multiplicity = self.invariant_use_multiplicity(premise_node, bindings)?;
            // [GRAM-4] the premise the use cites is a `use_premise` node: a
            // relation premise delimits its relation with parentheses and
            // carries two affine expressions around a `compare_op`; a named
            // premise is exactly the one IDENT it cites.
            let premise_children = self
                .tree
                .children_with(premise_node, Production::UsePremise)?;
            let [premise] = premise_children.as_slice() else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            let premise = *premise;
            let relation_form = !self
                .tree
                .children_with(premise, Production::AffineExpr)?
                .is_empty();
            let source = if relation_form {
                CheckedProofUseSource::Relation(self.check_ordered_affine_relation(
                    premise,
                    bindings,
                    &allowed_values,
                    function,
                    loop_depth,
                    AffineProofOwner::ProofUse,
                )?)
            } else {
                let usage = self.use_at(premise, LexicalUseRole::InvariantFact)?;
                let ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::Invariant,
                } = usage.target()
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                CheckedProofUseSource::Named(declaration)
            };
            uses.push(CheckedProofUse {
                node_path: self.tree.path(premise_node)?.clone(),
                multiplicity,
                source,
            });
        }

        Ok(Self::continuing_statement(
            CheckedStatement::Proof(CheckedSourceProof {
                node_path: self.tree.path(node)?.clone(),
                declaration: declaration.id(),
                name,
                target,
                uses,
            }),
            EffectSet::NONE,
        ))
    }

    /// Reads the optional multiplicity on one `use`.
    ///
    /// [GRAM-4] spells it `N times` before the premise, and the two forms it
    /// admits are checked here. A bare decimal is a proof-domain integer: the
    /// lexer classifies `[0-9]+` as `digits` and a typed runtime literal is a
    /// different terminal, which keeps the written multiplicity independent of
    /// machine integer types. A name is a live own local of unsigned integer
    /// type, so a written multiplicity is never negative by construction.
    /// Omission is the canonical spelling of one, and an explicit `1 times`
    /// is rejected.
    fn invariant_use_multiplicity(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedProofMultiplicity, CheckStop> {
        let Some(token) = self
            .tree
            .direct_token_with(node, crate::syntax::terminal::TerminalPredicate::Digits)?
        else {
            return self.invariant_use_value_multiplicity(node, bindings);
        };
        let bytes = self.tree.token_bytes(token)?;
        if bytes == b"0" {
            return self.invalid_affine_proof(
                AffineProofOwner::ProofUse,
                node,
                "a use multiplier is zero",
                "write a positive bare-decimal multiplier, or omit it when it is one",
            );
        }
        if bytes == b"1" {
            return self.invalid_affine_proof(
                AffineProofOwner::ProofUse,
                node,
                "an explicitly written use multiplier one is not canonical",
                "omit `1 *` from this use",
            );
        }
        if bytes.len() > 1 && bytes.first() == Some(&b'0') {
            return self.invalid_affine_proof(
                AffineProofOwner::ProofUse,
                node,
                "a use multiplier is not in canonical decimal form",
                "remove leading zeroes from the positive bare-decimal multiplier",
            );
        }
        let Some(factor) = std::str::from_utf8(bytes)
            .ok()
            .and_then(|digits| digits.parse::<i128>().ok())
        else {
            return self.invalid_affine_proof(
                AffineProofOwner::ProofUse,
                node,
                "a use multiplier exceeds the positive i128 proof domain",
                "write a positive bare-decimal multiplier no greater than 170141183460469231731687303715884105727",
            );
        };
        Ok(CheckedProofMultiplicity::Literal(factor))
    }

    /// The named form of the multiplicity, `use n times X;`.
    ///
    /// A `proof_use` owns at most one direct IDENT and it is exactly this
    /// multiplicity, because the premise it cites is a `use_premise` node of
    /// its own. The value must be readable where the certificate is checked
    /// and unsigned: nonnegativity is what makes scaling a premise sound, and
    /// taking it from the written type keeps it structural.
    fn invariant_use_value_multiplicity(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<CheckedProofMultiplicity, CheckStop> {
        if self.tree.direct_identifiers(node)?.is_empty() {
            return Ok(CheckedProofMultiplicity::Literal(1));
        }
        let usage = self.use_at(node, LexicalUseRole::ProofValue)?;
        if let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::NamedConst,
        } = usage.target()
        {
            let (value, ty) =
                self.affine_named_const(declaration, node, AffineProofOwner::ProofUse)?;
            if ty.signed() || value < 1 {
                return self.invalid_affine_proof(
                    AffineProofOwner::ProofUse,
                    node,
                    "a named use multiplicity is not a positive unsigned integer",
                    "name a live own unsigned integer local, or write a positive bare decimal",
                );
            }
            return Ok(CheckedProofMultiplicity::Literal(value));
        }
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let local = bindings
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !local.live || local.mode != CheckedMode::Own {
            return self.invalid_affine_proof(
                AffineProofOwner::ProofUse,
                node,
                "a use multiplicity reads a moved or borrowed local",
                "name a live own unsigned integer local",
            );
        }
        let CheckedType::Integer(ty) = local.ty else {
            return self.invalid_affine_proof(
                AffineProofOwner::ProofUse,
                node,
                "a use multiplicity does not have a closed integer type",
                "name a live own unsigned integer local",
            );
        };
        if ty.signed() {
            return self.invalid_affine_proof(
                AffineProofOwner::ProofUse,
                node,
                "a use multiplicity is a signed integer, which may scale a premise by a negative number",
                "name an unsigned integer local, or convert the value to an unsigned type before the invariant",
            );
        }
        Ok(CheckedProofMultiplicity::Value {
            binding: local.binding,
            ty,
        })
    }

    /// [INV-1] the `compare_op` between the two affine expressions selects
    /// the proof-domain relation: the four ordered symbols normalize to one
    /// bounded `<=`, and equality or disequality is not an invariant relation.
    fn ordered_relation_normalization(
        &self,
        owner: AffineProofOwner,
        node: NodeId,
    ) -> Result<OrderedRelationNormalization, CheckStop> {
        let operator = self
            .tree
            .first_child_with(node, Production::CompareOp)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let [relation_token] = self.tree.direct_token_indices(operator)? else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        let normalization = match self.tree.token_bytes(*relation_token)? {
            b"<=" => OrderedRelationNormalization {
                reverse: false,
                bound: 0,
            },
            b"<" => OrderedRelationNormalization {
                reverse: false,
                bound: -1,
            },
            b">=" => OrderedRelationNormalization {
                reverse: true,
                bound: 0,
            },
            b">" => OrderedRelationNormalization {
                reverse: true,
                bound: -1,
            },
            _ => {
                return self.invalid_affine_proof(
                    owner,
                    node,
                    "the invariant relation is not an admitted ordered integer relation",
                    "write `<=`, `<`, `>=`, or `>` between the two affine expressions; equality and disequality are not invariant relations",
                );
            }
        };
        Ok(normalization)
    }

    pub(super) fn check_ordered_affine_relation(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        function: &FunctionSignature,
        loop_depth: usize,
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineRelation, CheckStop> {
        let normalization = self.ordered_relation_normalization(owner, node)?;
        let mut relation =
            self.form_affine_relation(node, bindings, allowed_values, function, loop_depth, owner)?;
        if normalization.reverse {
            std::mem::swap(&mut relation.left, &mut relation.right);
        }
        relation.bound = normalization.bound;
        self.validate_affine_relation(node, &relation, owner)?;
        Ok(relation)
    }

    fn form_affine_relation(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        function: &FunctionSignature,
        loop_depth: usize,
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineRelation, CheckStop> {
        let expressions = self.tree.children_with(node, Production::AffineExpr)?;
        let [left_node, right_node] = expressions.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        let left = self.check_affine_expression(
            *left_node,
            bindings,
            allowed_values,
            function,
            loop_depth,
            owner,
        )?;
        let right = self.check_affine_expression(
            *right_node,
            bindings,
            allowed_values,
            function,
            loop_depth,
            owner,
        )?;
        Ok(CheckedAffineRelation {
            node_path: self.tree.path(node)?.clone(),
            left,
            right,
            bound: 0,
        })
    }

    fn validate_affine_relation(
        &self,
        node: NodeId,
        relation: &CheckedAffineRelation,
        owner: AffineProofOwner,
    ) -> Result<(), CheckStop> {
        let affine_left = checked_affine_expression(&relation.left)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let affine_right = checked_affine_expression(&relation.right)
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mut affine_check = AffineCheckState::new();
        match normalize_bounded_less_equal(
            &affine_left,
            &affine_right,
            relation.bound,
            &mut affine_check,
        )
        .map(drop)
        {
            Ok(()) => {}
            Err(AffineCheckError::ArithmeticOverflow) => {
                return self.invalid_affine_proof(
                    owner,
                    node,
                    "the affine coefficients or accumulated constant arithmetic overflow i128",
                    "reduce the affine coefficients and constants until every formation step fits i128",
                );
            }
            Err(AffineCheckError::LimitExceeded(_)) => {
                return self.invalid_affine_proof(
                    owner,
                    node,
                    "the affine relation exceeds the checker's fixed formation capacity",
                    "split the relation into smaller named local invariants",
                );
            }
            Err(_) => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
        Ok(())
    }

    fn check_affine_expression(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        function: &FunctionSignature,
        loop_depth: usize,
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineExpression, CheckStop> {
        let children = self.tree.children(node)?;
        let Some(first) = children.first().copied() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        if self.tree.production(first)? != Production::AffineTerm {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let mut expression =
            self.check_affine_term(first, bindings, allowed_values, function, loop_depth, owner)?;
        let mut cursor = 1;
        while cursor < children.len() {
            let Some(operator) = children.get(cursor).copied() else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            let Some(term) = children.get(cursor + 1).copied() else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            if self.tree.production(operator)? != Production::AffineAddOp
                || self.tree.production(term)? != Production::AffineTerm
            {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            let right = self.check_affine_term(
                term,
                bindings,
                allowed_values,
                function,
                loop_depth,
                owner,
            )?;
            let [operator_token] = self.tree.direct_token_indices(operator)? else {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            };
            let kind = match self.tree.token_bytes(*operator_token)? {
                b"+" => CheckedAffineExpressionKind::Add(Box::new(expression), Box::new(right)),
                b"-" => {
                    CheckedAffineExpressionKind::Subtract(Box::new(expression), Box::new(right))
                }
                _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
            };
            expression = CheckedAffineExpression {
                node_path: self.tree.path(node)?.clone(),
                kind,
            };
            cursor = cursor
                .checked_add(2)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        }
        Ok(expression)
    }

    fn check_affine_term(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        function: &FunctionSignature,
        loop_depth: usize,
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineExpression, CheckStop> {
        let factors = self.tree.children_with(node, Production::AffineFactor)?;
        match factors.as_slice() {
            [factor] => self
                .check_affine_factor(
                    *factor,
                    bindings,
                    allowed_values,
                    function,
                    loop_depth,
                    owner,
                )
                .map(|(expression, _)| expression),
            [left_node, right_node] => {
                let (left, left_literal) = self.check_affine_factor(
                    *left_node,
                    bindings,
                    allowed_values,
                    function,
                    loop_depth,
                    owner,
                )?;
                let (right, right_literal) = self.check_affine_factor(
                    *right_node,
                    bindings,
                    allowed_values,
                    function,
                    loop_depth,
                    owner,
                )?;
                let (constant, constant_ty, value) = match (left_literal, right_literal) {
                    (Some((constant, constant_ty)), _) => (constant, constant_ty, right),
                    (None, Some((constant, constant_ty))) => (constant, constant_ty, left),
                    (None, None) => {
                        return self.invalid_affine_proof(
                            owner,
                            node,
                            "an affine multiplication has no direct integer-literal operand",
                            "multiply one affine factor by a directly written integer literal",
                        );
                    }
                };
                Ok(CheckedAffineExpression {
                    node_path: self.tree.path(node)?.clone(),
                    kind: CheckedAffineExpressionKind::MultiplyByConstant {
                        constant,
                        constant_ty,
                        value: Box::new(value),
                    },
                })
            }
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// The closed integer value of one named const named by a proof relation.
    ///
    /// A const-generic parameter is symbolic rather than closed, so it is not
    /// this: [MSR-6] admits it above as an affine atom of its own instead of
    /// folding it to a number here.
    fn affine_named_const(
        &self,
        declaration: DeclarationId,
        node: NodeId,
        owner: AffineProofOwner,
    ) -> Result<(i128, IntegerType), CheckStop> {
        let Some(constant) = self.constants.get(&declaration).copied() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let constant = self.constant(constant)?;
        let CheckedValue::Integer { ty, bits } = &constant.value else {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine factor names a const that is not an integer",
                "name an integer const, an integer literal, or a live own integer local",
            );
        };
        Ok((affine_integer_value(*ty, *bits), *ty))
    }

    fn check_affine_factor(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        function: &FunctionSignature,
        loop_depth: usize,
        owner: AffineProofOwner,
    ) -> Result<(CheckedAffineExpression, Option<(i128, IntegerType)>), CheckStop> {
        // [GRAM-4, MSR-5] the factor production is shared with a contract
        // clause and carries an `atom`, a `call`, a `construct`, or a
        // parenthesized expression. [INV-1] admits an `atom` only as one
        // bare IDENT place or one integer literal; every other atom shape,
        // and a `construct`, is a rule rejection at this factor and not a
        // parse rejection.
        if let Some(atom) = self.tree.first_child_with(node, Production::Atom)? {
            return self.check_affine_atom(node, atom, bindings, allowed_values, function, owner);
        }
        if self
            .tree
            .first_child_with(node, Production::Construct)?
            .is_some()
        {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine factor is a construction",
                "use only integer literals, live own integer locals, and measure formers",
            );
        }

        // [INV-1, MSR-1] one measure former as an affine factor. The relation
        // evaluates nothing and reads no storage, so the factor reaches the
        // resolved place and the measure row and stops there: no loan access,
        // no effect, and no goal.
        if let Some(call) = self.tree.first_child_with(node, Production::Call)? {
            let expression = self.check_affine_measure(
                call,
                bindings,
                allowed_values,
                function,
                loop_depth,
                owner,
            )?;
            return Ok((
                CheckedAffineExpression {
                    node_path: self.tree.path(node)?.clone(),
                    kind: CheckedAffineExpressionKind::Measure(Box::new(expression)),
                },
                None,
            ));
        }

        let nested = self
            .tree
            .first_child_with(node, Production::AffineExpr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression = self.check_affine_expression(
            nested,
            bindings,
            allowed_values,
            function,
            loop_depth,
            owner,
        )?;
        Ok((expression, None))
    }

    /// [INV-1] the one `atom` an affine factor admits: a bare IDENT place or
    /// an integer literal.
    fn check_affine_atom(
        &self,
        node: NodeId,
        atom: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        function: &FunctionSignature,
        owner: AffineProofOwner,
    ) -> Result<(CheckedAffineExpression, Option<(i128, IntegerType)>), CheckStop> {
        if let Some(literal) = self
            .tree
            .direct_token_with(atom, crate::syntax::terminal::TerminalPredicate::Literal)?
        {
            let bytes = self.tree.token_bytes(literal)?;
            if matches!(bytes, b"0_T" | b"1_T") {
                return self.invalid_affine_proof(
                    owner,
                    node,
                    "an affine literal does not have a closed integer type",
                    "write a concrete integer suffix such as `_u64` on every affine literal",
                );
            }
            let CheckedValue::Integer { ty, bits } = self.parse_literal(node, bytes)? else {
                return self.invalid_affine_proof(
                    owner,
                    node,
                    "an affine factor is not an integer literal",
                    "use only integer literals and live own integer locals",
                );
            };
            let value = affine_integer_value(ty, bits);
            return Ok((
                CheckedAffineExpression {
                    node_path: self.tree.path(node)?.clone(),
                    kind: CheckedAffineExpressionKind::Constant { value, ty },
                },
                Some((value, ty)),
            ));
        }

        let Some(place) = self.tree.first_child_with(atom, Production::Place)? else {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine factor is a borrow rather than a value",
                "use only integer literals, live own integer locals, and measure formers",
            );
        };
        if self.has_fixed(atom, crate::syntax::terminal::FixedTerminal::Move)? {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine factor consumes the value it reads",
                "read the live own integer local without `move`",
            );
        }
        if !self
            .tree
            .children_with(place, Production::Psuffix)?
            .is_empty()
        {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine factor selects a field or an element of a place",
                "bind the integer value with a `let` and use that binding",
            );
        }
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.has_fixed(pbase, crate::syntax::terminal::FixedTerminal::Deref)? {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine factor dereferences a holder",
                "bind the integer value with a `let` and use that binding",
            );
        }

        let usage = self.use_at(pbase, owner.value_role())?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        // [MSR-6] an in-scope const generic is a value wherever a named const
        // is, and [INV-1]'s affine atom is one of those positions: it is a
        // constant of [ENT-2] clause (c) rather than a tracked place, so it
        // needs no liveness and no entry state. A concrete instance reads the
        // mathematical constant [FN-2] fixed for it; the one source-canonical
        // symbolic instance keeps the declaration-anchored constant term.
        if class == DeclarationClass::ConstGeneric {
            let ty = self.const_generic_type(declaration)?;
            let kind = match function.substitution.const_argument(declaration) {
                Some(crate::semantic::CheckedConst::Value(value)) => {
                    CheckedAffineExpressionKind::Constant {
                        value: i128::from(value),
                        ty,
                    }
                }
                // [FN-2] a const parameter this instance's caller supplied
                // from a const parameter of its own is that caller's
                // parameter here, exactly as it is in every other position
                // this instance reads it.
                Some(crate::semantic::CheckedConst::Parameter(supplied)) => {
                    CheckedAffineExpressionKind::ConstGeneric {
                        declaration: supplied,
                        ty,
                        name: self.declaration_spelling(supplied)?,
                    }
                }
                _ => CheckedAffineExpressionKind::ConstGeneric {
                    declaration,
                    ty,
                    name: self.declaration_spelling(declaration)?,
                },
            };
            return Ok((
                CheckedAffineExpression {
                    node_path: self.tree.path(node)?.clone(),
                    kind,
                },
                // A const generic is not an integer literal, so it never
                // supplies [INV-1]'s one admitted non-unit multiplier.
                None,
            ));
        }
        // A named integer const denotes one closed value, so it folds to the
        // constant it is. It reads the same in a relation as it does in the
        // body, which is the whole reason to admit it: a limit written once is
        // stated once. [MSR-6] a const generic above stays symbolic because it
        // has no closed value at the source-canonical instance; a named const
        // always has one.
        if class == DeclarationClass::NamedConst {
            let (value, ty) = self.affine_named_const(declaration, node, owner)?;
            return Ok((
                CheckedAffineExpression {
                    node_path: self.tree.path(node)?.clone(),
                    kind: CheckedAffineExpressionKind::Constant { value, ty },
                },
                Some((value, ty)),
            ));
        }
        if class != DeclarationClass::Value {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        if !allowed_values.contains(&declaration) {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine relation reads a value outside its admitted entry state",
                "use a live integer that exists before this proof point",
            );
        }
        let local = bindings
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if !local.live {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine relation reads a moved local value",
                "use a live own integer local",
            );
        }
        let CheckedType::Integer(ty) = local.ty else {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine local does not have a closed integer type",
                "use only live own integer locals",
            );
        };
        if local.mode != CheckedMode::Own {
            return self.invalid_affine_proof(
                owner,
                node,
                "an affine local is a borrow holder rather than an own integer value",
                "bind the integer value and reference that own binding",
            );
        }
        Ok((
            CheckedAffineExpression {
                node_path: self.tree.path(node)?.clone(),
                kind: CheckedAffineExpressionKind::Local {
                    binding: local.binding,
                    ty,
                },
            },
            None,
        ))
    }

    /// [INV-1] the one `call` an affine factor admits: a measure former over
    /// an admitted measure place.
    fn check_affine_measure(
        &self,
        call: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        allowed_values: &HashSet<DeclarationId>,
        function: &FunctionSignature,
        loop_depth: usize,
        owner: AffineProofOwner,
    ) -> Result<CheckedExpression, CheckStop> {
        let callee = self
            .tree
            .first_child_with(call, Production::Callee)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let usage = self.use_at_roles(
            callee,
            &[
                LexicalUseRole::IdentifierCallee,
                LexicalUseRole::OperationCallee,
            ],
        )?;
        let ResolvedTarget::Operation(operation) = usage.target() else {
            return self.invalid_affine_proof(
                owner,
                call,
                "an affine factor calls something other than a measure former",
                "write len_of(P), cap_of(P), room_of(P) or head_of(P) over a measured place",
            );
        };
        let spelling = crate::operation_family_spelling(operation)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let Some(measure) = super::super::expressions::calls::measure_former(spelling) else {
            return self.invalid_affine_proof(
                owner,
                call,
                "an affine factor calls something other than a measure former",
                "write len_of(P), cap_of(P), room_of(P) or head_of(P) over a measured place",
            );
        };
        self.reject_named_operation_arguments(call, spelling)?;
        self.reject_written_operation_type_argument(call)?;
        let atoms = self.operation_atoms(call, 1)?;
        // [INV-1, MSR-1] a subscript inside the measured place is an ordinary
        // [OP-4] occurrence and its offset an ordinary operand, so it is
        // formed under the enclosing concrete instance and at the enclosing
        // loop depth — the same premise set the same place has anywhere else.
        let place = self.check_indexed_atom_place(atoms[0], bindings, function, loop_depth)?;
        // [INV-1] the place resolves in the same context an IDENT does, and
        // its root is one of the values that context admits.
        if let Some(declaration) = place.root_declaration()
            && !allowed_values.contains(&declaration)
        {
            return self.invalid_affine_proof(
                owner,
                atoms[0],
                "an affine relation reads a value outside its admitted entry state",
                "measure a value that exists before this proof point",
            );
        }
        self.measure_of_indexed_place(measure, place, atoms[0])
    }

    fn invalid_affine_proof<ResultValue>(
        &self,
        owner: AffineProofOwner,
        node: NodeId,
        reason: &'static str,
        mechanical_fix: &'static str,
    ) -> Result<ResultValue, CheckStop> {
        match owner {
            AffineProofOwner::InvariantTarget => {
                self.invalid_invariant(node, reason, mechanical_fix)
            }
            AffineProofOwner::ProofUse => self.issue_node(
                SemanticRule::Prf1,
                node,
                SemanticIssueKind::InvalidSourceProof {
                    reason,
                    mechanical_fix,
                },
            ),
        }
    }
}

const fn affine_integer_value(ty: IntegerType, bits: u64) -> i128 {
    let value = bits as i128;
    if ty.signed() {
        let width = ty.width() as u32;
        let sign_bit = 1_u64 << (width - 1);
        if bits & sign_bit != 0 {
            return value - (1_i128 << width);
        }
    }
    value
}

enum AffineConversion<'expression> {
    Visit(&'expression CheckedAffineExpression),
    Add,
    Subtract,
    MultiplyByConstant(i128),
}

/// Erases source-only paths and integer-type annotations while preserving the
/// exact expression tree consumed by the single affine formation core.
///
/// The term identities here are local to this formation check, which decides
/// only whether the written relation's arithmetic fits the fixed [INV-1]
/// ceilings. A binding takes its own ordinal; a measure factor [MSR-1] takes
/// one from the top of the same space, in the order the walk reaches distinct
/// factors, because the checker has no [ENT-2] term registry at this point and
/// the flow builds the real images itself.
fn checked_affine_expression(source: &CheckedAffineExpression) -> Option<AffineExpression> {
    let mut pending = vec![AffineConversion::Visit(source)];
    let mut values = Vec::new();
    // [INV-1] the two factor shapes with no checker-visible term identity:
    // a measure former's place, and a const generic at the symbolic
    // instance. They share one descending index block so no two of them
    // collide and neither collides with a binding's own ascending index.
    enum Opaque<'expression> {
        Measure(&'expression CheckedExpression),
        ConstGeneric(DeclarationId),
    }
    let mut opaques: Vec<Opaque<'_>> = Vec::new();
    while let Some(next) = pending.pop() {
        match next {
            AffineConversion::Visit(expression) => match &expression.kind {
                CheckedAffineExpressionKind::Constant { value, ty: _ } => {
                    values.push(AffineExpression::Constant(*value));
                }
                CheckedAffineExpressionKind::Local { binding, ty: _ } => {
                    values.push(AffineExpression::Term(AffineTermId::from_index(binding.0)));
                }
                CheckedAffineExpressionKind::Measure(measure) => {
                    let index = opaques
                        .iter()
                        .position(|seen| {
                            matches!(seen, Opaque::Measure(other) if *other == measure.as_ref())
                        })
                        .unwrap_or_else(|| {
                            opaques.push(Opaque::Measure(measure.as_ref()));
                            opaques.len() - 1
                        });
                    let index = u32::MAX.checked_sub(u32::try_from(index).ok()?)?;
                    values.push(AffineExpression::Term(AffineTermId::from_index(index)));
                }
                CheckedAffineExpressionKind::ConstGeneric { declaration, .. } => {
                    let index = opaques
                        .iter()
                        .position(|seen| {
                            matches!(seen, Opaque::ConstGeneric(other) if other == declaration)
                        })
                        .unwrap_or_else(|| {
                            opaques.push(Opaque::ConstGeneric(*declaration));
                            opaques.len() - 1
                        });
                    let index = u32::MAX.checked_sub(u32::try_from(index).ok()?)?;
                    values.push(AffineExpression::Term(AffineTermId::from_index(index)));
                }
                CheckedAffineExpressionKind::Add(left, right) => {
                    pending.push(AffineConversion::Add);
                    pending.push(AffineConversion::Visit(right));
                    pending.push(AffineConversion::Visit(left));
                }
                CheckedAffineExpressionKind::Subtract(left, right) => {
                    pending.push(AffineConversion::Subtract);
                    pending.push(AffineConversion::Visit(right));
                    pending.push(AffineConversion::Visit(left));
                }
                CheckedAffineExpressionKind::MultiplyByConstant {
                    constant,
                    constant_ty: _,
                    value,
                } => {
                    pending.push(AffineConversion::MultiplyByConstant(*constant));
                    pending.push(AffineConversion::Visit(value));
                }
            },
            AffineConversion::Add => {
                let right = values.pop()?;
                let left = values.pop()?;
                values.push(AffineExpression::Add(Box::new(left), Box::new(right)));
            }
            AffineConversion::Subtract => {
                let right = values.pop()?;
                let left = values.pop()?;
                values.push(AffineExpression::Subtract(Box::new(left), Box::new(right)));
            }
            AffineConversion::MultiplyByConstant(constant) => {
                let value = values.pop()?;
                values.push(AffineExpression::MultiplyByConstant {
                    constant,
                    value: Box::new(value),
                });
            }
        }
    }
    let result = values.pop()?;
    values.is_empty().then_some(result)
}
