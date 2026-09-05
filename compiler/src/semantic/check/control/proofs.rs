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
    CheckedAffineExpression, CheckedAffineExpressionKind, CheckedAffineRelation, CheckedMode,
    CheckedProofUse, CheckedProofUseSource, CheckedSourceProof, CheckedStatement, CheckedType,
    CheckedValue, IntegerType,
};
use super::super::{CheckStop, Checker, EffectSet, LocalBinding};
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
            AffineProofOwner::InvariantTarget,
        )?;
        let premise_nodes = self.tree.children_with(node, Production::ProofUse)?;
        let mut uses = Vec::with_capacity(premise_nodes.len());
        for premise_node in premise_nodes {
            let factor = self.invariant_use_factor(premise_node)?;
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
                factor,
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

    /// Reads the optional proof-domain multiplier on one `use`.
    ///
    /// The lexer already classifies bare `[0-9]+` as `digits`; typed runtime
    /// literals are deliberately a different terminal. This keeps the
    /// multiplier independent of machine integer types. Omission is the
    /// canonical spelling of factor one; an explicit `1 *` is rejected.
    fn invariant_use_factor(&self, node: NodeId) -> Result<i128, CheckStop> {
        let Some(token) = self
            .tree
            .direct_token_with(node, crate::syntax::terminal::TerminalPredicate::Digits)?
        else {
            return Ok(1);
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
        Ok(factor)
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
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineRelation, CheckStop> {
        let normalization = self.ordered_relation_normalization(owner, node)?;
        let mut relation = self.form_affine_relation(node, bindings, allowed_values, owner)?;
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
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineRelation, CheckStop> {
        let expressions = self.tree.children_with(node, Production::AffineExpr)?;
        let [left_node, right_node] = expressions.as_slice() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        let left = self.check_affine_expression(*left_node, bindings, allowed_values, owner)?;
        let right = self.check_affine_expression(*right_node, bindings, allowed_values, owner)?;
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
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineExpression, CheckStop> {
        let children = self.tree.children(node)?;
        let Some(first) = children.first().copied() else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        if self.tree.production(first)? != Production::AffineTerm {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let mut expression = self.check_affine_term(first, bindings, allowed_values, owner)?;
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
            let right = self.check_affine_term(term, bindings, allowed_values, owner)?;
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
        owner: AffineProofOwner,
    ) -> Result<CheckedAffineExpression, CheckStop> {
        let factors = self.tree.children_with(node, Production::AffineFactor)?;
        match factors.as_slice() {
            [factor] => self
                .check_affine_factor(*factor, bindings, allowed_values, owner)
                .map(|(expression, _)| expression),
            [left_node, right_node] => {
                let (left, left_literal) =
                    self.check_affine_factor(*left_node, bindings, allowed_values, owner)?;
                let (right, right_literal) =
                    self.check_affine_factor(*right_node, bindings, allowed_values, owner)?;
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
    /// this and stays inadmissible: an affine factor is a number, and a
    /// symbolic constant would need an atom of its own.
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
        owner: AffineProofOwner,
    ) -> Result<(CheckedAffineExpression, Option<(i128, IntegerType)>), CheckStop> {
        if let Some(literal) = self
            .tree
            .direct_token_with(node, crate::syntax::terminal::TerminalPredicate::Literal)?
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

        if !self.tree.direct_identifiers(node)?.is_empty() {
            let usage = self.use_at(node, owner.value_role())?;
            // A named integer const denotes one closed value, so it folds to
            // the constant it is. It reads the same in a relation as it does
            // in the body, which is the whole reason to admit it: a limit
            // written once is stated once.
            if let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::NamedConst,
            } = usage.target()
            {
                let (value, ty) = self.affine_named_const(declaration, node, owner)?;
                return Ok((
                    CheckedAffineExpression {
                        node_path: self.tree.path(node)?.clone(),
                        kind: CheckedAffineExpressionKind::Constant { value, ty },
                    },
                    Some((value, ty)),
                ));
            }
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Value,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
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
            return Ok((
                CheckedAffineExpression {
                    node_path: self.tree.path(node)?.clone(),
                    kind: CheckedAffineExpressionKind::Local {
                        binding: local.binding,
                        ty,
                    },
                },
                None,
            ));
        }

        let nested = self
            .tree
            .first_child_with(node, Production::AffineExpr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression = self.check_affine_expression(nested, bindings, allowed_values, owner)?;
        Ok((expression, None))
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
fn checked_affine_expression(source: &CheckedAffineExpression) -> Option<AffineExpression> {
    let mut pending = vec![AffineConversion::Visit(source)];
    let mut values = Vec::new();
    while let Some(next) = pending.pop() {
        match next {
            AffineConversion::Visit(expression) => match &expression.kind {
                CheckedAffineExpressionKind::Constant { value, ty: _ } => {
                    values.push(AffineExpression::Constant(*value));
                }
                CheckedAffineExpressionKind::Local { binding, ty: _ } => {
                    values.push(AffineExpression::Term(AffineTermId::from_index(binding.0)));
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
