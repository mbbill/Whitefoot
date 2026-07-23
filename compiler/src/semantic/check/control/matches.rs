use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, DeferredUseRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedConstructor, CheckedEnumType, CheckedExpression, CheckedField, CheckedMatchArm,
    CheckedMatchBinder, CheckedMode, CheckedNominalKind, CheckedType,
};
use super::super::borrows::BorrowInfo;
use super::super::{CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding};
use super::{BreakState, ControlCounters, ControlScope, GiveContext};

#[derive(Clone)]
struct VariantDescriptor {
    name: String,
    tag: u32,
    fields: Vec<CheckedField>,
    constructor: CheckedConstructor,
}

struct MatchDescriptor {
    enum_type: CheckedEnumType,
    variants: Vec<VariantDescriptor>,
}

pub(super) struct MatchResult {
    pub(super) scrutinee: CheckedExpression,
    pub(super) enum_type: CheckedEnumType,
    pub(super) arms: Vec<CheckedMatchArm>,
    pub(super) can_continue: bool,
    pub(super) all_paths_deliver: bool,
    pub(super) effects: EffectSet,
    pub(super) give_states: Vec<HashMap<DeclarationId, LocalBinding>>,
    pub(super) break_states: Vec<BreakState>,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_match(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
        value_expected: Option<CheckedType>,
    ) -> Result<MatchResult, CheckStop> {
        let expression_node = self
            .tree
            .first_child_with(node, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let scrutinee =
            self.check_match_expression(function, expression_node, bindings, scope.loops.len())?;
        let descriptor = self.match_descriptor(scrutinee.expression.ty(), expression_node)?;
        let base_bindings = bindings.clone();
        let base_keys = base_bindings.keys().copied().collect::<Vec<_>>();
        let base_key_set = base_keys.iter().copied().collect::<HashSet<_>>();
        let value_match = self.tree.production(node)? == Production::ValueMatch;
        if value_match != value_expected.is_some() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let local_give_context = value_expected.map(|expected| GiveContext {
            expected,
            preserved: base_key_set.clone(),
            enclosing_loops: scope.loops.iter().map(|context| context.id).collect(),
        });
        let arm_scope = ControlScope {
            loops: scope.loops,
            give_context: local_give_context.as_ref().or(scope.give_context),
        };
        let arm_nodes = self.tree.children_with(node, Production::Arm)?;
        let mut seen = HashSet::new();
        let mut duplicate_arm = None;
        let mut resolved_variants = Vec::with_capacity(arm_nodes.len());
        for arm_node in &arm_nodes {
            let variant = self.match_variant(&descriptor, *arm_node)?.clone();
            if !seen.insert(variant.tag) {
                duplicate_arm.get_or_insert(*arm_node);
            }
            resolved_variants.push(variant);
        }
        let missing_variants = descriptor
            .variants
            .iter()
            .filter(|variant| !seen.contains(&variant.tag))
            .map(|variant| variant.name.clone())
            .collect::<Vec<_>>();
        if !missing_variants.is_empty() {
            return self.issue_node(
                SemanticRule::Err2,
                node,
                SemanticIssueKind::NonExhaustiveMatch { missing_variants },
            );
        }
        if let Some(arm) = duplicate_arm {
            return self.unsupported(UnsupportedSemanticFeature::DuplicateMatchArm, arm);
        }

        let mut arms = Vec::with_capacity(arm_nodes.len());
        let mut normal_states = Vec::new();
        let mut give_states = Vec::new();
        let mut break_states = Vec::new();
        let mut effects = scrutinee.effects.clone();
        let mut all_paths_deliver = true;
        for (arm_node, variant) in arm_nodes.into_iter().zip(&resolved_variants) {
            let mut arm_bindings = base_bindings.clone();
            let binders = self.check_match_binders(
                variant,
                arm_node,
                &mut arm_bindings,
                counters.next_binding,
                scope.loops.len(),
                &scrutinee,
            )?;
            let statements = self.tree.children_with(arm_node, Production::Stmt)?;
            let checked = self.check_block(
                function,
                &statements,
                &mut arm_bindings,
                counters,
                arm_scope,
            )?;
            let fallthrough_drops = if checked.can_continue {
                self.live_affine_drops(&arm_bindings, &base_key_set)?
            } else {
                Vec::new()
            };
            if checked.can_continue {
                normal_states.push(arm_bindings);
            }
            all_paths_deliver &= !checked.can_continue && checked.all_paths_deliver;
            effects = effects.union(checked.effects);
            give_states.extend(checked.give_states);
            break_states.extend(checked.break_states);
            arms.push(CheckedMatchArm {
                tag: variant.tag,
                binders,
                body: checked.statements,
                fallthrough_drops,
            });
        }
        if value_match {
            if !all_paths_deliver {
                return self.issue_node(SemanticRule::Give1, node, SemanticIssueKind::InvalidGive);
            }
            self.join_states(&base_keys, &give_states, node, bindings)?;
        } else {
            self.join_states(&base_keys, &normal_states, node, bindings)?;
        }
        Ok(MatchResult {
            scrutinee: scrutinee.expression,
            enum_type: descriptor.enum_type,
            arms,
            can_continue: if value_match {
                !give_states.is_empty()
            } else {
                !normal_states.is_empty()
            },
            all_paths_deliver,
            effects,
            give_states: if value_match { Vec::new() } else { give_states },
            break_states,
        })
    }

    fn match_descriptor(
        &self,
        ty: CheckedType,
        node: NodeId,
    ) -> Result<MatchDescriptor, CheckStop> {
        match ty {
            CheckedType::Bool => Ok(MatchDescriptor {
                enum_type: CheckedEnumType::Bool,
                variants: vec![
                    VariantDescriptor {
                        name: "True".to_owned(),
                        tag: 1,
                        fields: Vec::new(),
                        constructor: CheckedConstructor::Prelude(crate::PreludeDeclarationId::new(
                            1,
                        )),
                    },
                    VariantDescriptor {
                        name: "False".to_owned(),
                        tag: 0,
                        fields: Vec::new(),
                        constructor: CheckedConstructor::Prelude(crate::PreludeDeclarationId::new(
                            2,
                        )),
                    },
                ],
            }),
            CheckedType::Nominal(id) => {
                let CheckedNominalKind::Enum { variants } = &self.nominal(id)?.kind else {
                    return self.issue_node(
                        SemanticRule::Type5,
                        node,
                        SemanticIssueKind::TypeMismatch,
                    );
                };
                let variants = variants
                    .iter()
                    .map(|variant| VariantDescriptor {
                        name: variant.name.clone(),
                        tag: variant.tag,
                        fields: variant.fields.clone(),
                        constructor: variant.constructor,
                    })
                    .collect();
                Ok(MatchDescriptor {
                    enum_type: CheckedEnumType::Nominal(id),
                    variants,
                })
            }
            _ => self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch),
        }
    }

    fn match_variant<'descriptor>(
        &self,
        descriptor: &'descriptor MatchDescriptor,
        arm: NodeId,
    ) -> Result<&'descriptor VariantDescriptor, CheckStop> {
        let usage = self.use_at(arm, LexicalUseRole::ArmVariant)?;
        descriptor
            .variants
            .iter()
            .find(|variant| match usage.target() {
                ResolvedTarget::Source { declaration, .. } => {
                    variant.constructor == CheckedConstructor::Source(declaration)
                }
                ResolvedTarget::Prelude(id) => {
                    variant.constructor == CheckedConstructor::Prelude(id)
                }
                _ => false,
            })
            .ok_or_else(|| {
                self.issue_value(
                    SemanticRule::Type6,
                    arm,
                    SemanticIssueKind::ForeignMatchVariant,
                )
            })
    }

    fn check_match_binders(
        &self,
        variant: &VariantDescriptor,
        arm: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        next_binding: &mut u32,
        loop_depth: usize,
        scrutinee: &super::super::TypedExpression,
    ) -> Result<Vec<CheckedMatchBinder>, CheckStop> {
        let mode = scrutinee.mode;
        let written =
            if let Some(list) = self.tree.first_child_with(arm, Production::FieldbindList)? {
                self.tree.children_with(list, Production::Fieldbind)?
            } else {
                Vec::new()
            };
        if written.len() != variant.fields.len() {
            return self.invalid_match_fields(variant, arm);
        }
        let mut binders = Vec::with_capacity(written.len());
        for (index, (written, field)) in written.into_iter().zip(&variant.fields).enumerate() {
            if self
                .deferred_use_at(written, DeferredUseRole::MatchField)?
                .spelling()
                != field.name
            {
                return self.invalid_match_fields(variant, written);
            }
            if mode != CheckedMode::Own {
                let box_payload = matches!(
                    field.ty,
                    CheckedType::Nominal(id)
                        if matches!(self.nominal(id)?.kind, CheckedNominalKind::Box { .. })
                );
                if matches!(mode, CheckedMode::Unique(_))
                    && !box_payload
                    && !self.is_copy_type(field.ty)?
                {
                    return self
                        .unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, written);
                }
            }
            let declaration = self.declaration_at(written, DeclarationRole::MatchBinder)?;
            let binding = Self::allocate_binding(next_binding)?;
            let borrow = if mode == CheckedMode::Own {
                None
            } else {
                let parent = scrutinee
                    .borrow
                    .as_ref()
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let mut place = parent.place;
                place.fields.push(
                    u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                );
                Some(BorrowInfo { place, ..parent })
            };
            if bindings
                .insert(
                    declaration.id(),
                    LocalBinding {
                        binding,
                        declaration: declaration.id(),
                        mode,
                        ty: field.ty,
                        live: true,
                        loop_depth,
                        borrow,
                        slice: None,
                    },
                )
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            binders.push(CheckedMatchBinder {
                binding,
                field: u32::try_from(index)
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                mode,
                ty: field.ty,
            });
        }
        Ok(binders)
    }

    fn invalid_match_fields<ResultValue>(
        &self,
        variant: &VariantDescriptor,
        node: NodeId,
    ) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Gram10,
            node,
            SemanticIssueKind::InvalidMatchFields {
                variant: variant.name.clone(),
                declared_fields: variant
                    .fields
                    .iter()
                    .map(|field| field.name.clone())
                    .collect(),
            },
        )
    }

    pub(super) fn join_states(
        &self,
        base_keys: &[DeclarationId],
        states: &[HashMap<DeclarationId, LocalBinding>],
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        let Some(first) = states.first() else {
            return Ok(());
        };
        for key in base_keys {
            let expected = first
                .get(key)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if states
                .iter()
                .skip(1)
                .any(|state| state.get(key) != Some(expected))
            {
                return self.unsupported(UnsupportedSemanticFeature::OwnershipJoin, node);
            }
            *bindings
                .get_mut(key)
                .ok_or(SemanticCompilerFailure::InvalidResolution)? = expected.clone();
        }
        Ok(())
    }
}
