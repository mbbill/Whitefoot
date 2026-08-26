use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, DeferredUseRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::super::model::{
    CheckedConstructor, CheckedEnumType, CheckedExpression, CheckedField, CheckedMatchArm,
    CheckedMatchBinder, CheckedMode, CheckedNominalKind, CheckedStatement, CheckedType,
};
use super::super::super::tree::ConditionalAlternative;
use super::super::borrows::{BorrowInfo, RequiredReferent};
use super::super::{CheckStop, Checker, EffectSet, FunctionSignature, LocalBinding};
use super::{BlockResult, BreakState, ControlCounters, ControlScope, GiveContext};

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
    /// [GIVE-1] the mode and type the delivery set derived, or `None` for a
    /// statement `match` and for a value initializer that delivers nothing.
    pub(super) delivered: Option<(CheckedMode, CheckedType)>,
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
        value_delivery: bool,
    ) -> Result<MatchResult, CheckStop> {
        let expression_node = self
            .tree
            .first_child_with(node, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let scrutinee =
            self.check_match_expression(function, expression_node, bindings, scope.loops.len())?;
        // [OWN-13] matches an enum value or a place reached through a borrow.
        // A holder written where the enum itself is required — a bare borrow
        // holder, a `borrow_expr`, a reference-returning call, or a `box` of
        // an enum — is the [TYPE-7] implicit read, and this scrutinee's own
        // wrong-type judgment forms no rejection.
        if self.reads_implicitly_through_holder(
            scrutinee.reference_value,
            scrutinee.expression.ty(),
            RequiredReferent::Enum,
        )? {
            return self.issue_node(
                SemanticRule::Type7,
                expression_node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        let descriptor = self.match_descriptor(scrutinee.expression.ty(), expression_node)?;
        let base_bindings = bindings.clone();
        let base_keys = base_bindings.keys().copied().collect::<Vec<_>>();
        let base_key_set = base_keys.iter().copied().collect::<HashSet<_>>();
        let value_match = self.tree.production(node)? == Production::ValueMatch;
        if value_match != value_delivery {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let local_give_context = value_delivery.then(|| GiveContext::empty(&base_key_set, scope));
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
                counters,
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
            self.reject_slice_valued_delivery(
                node,
                local_give_context.as_ref().and_then(GiveContext::delivered),
            )?;
            self.join_states(&base_keys, &give_states, node, bindings)?;
        } else {
            self.join_states(&base_keys, &normal_states, node, bindings)?;
        }
        Ok(MatchResult {
            scrutinee: scrutinee.expression,
            enum_type: descriptor.enum_type,
            arms,
            delivered: local_give_context.as_ref().and_then(GiveContext::delivered),
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

    /// [GRAM-6] the Bool conditional.
    ///
    /// It produces exactly the checked shape the Bool `match` produced before
    /// the spelling changed: a two-armed match over [`CheckedEnumType::Bool`]
    /// with `True` tagged 1 and `False` tagged 0. Lowering, entailment,
    /// cleanup, and drops therefore need no `if` of their own. The arms cannot
    /// come from [`Self::check_match`], which reads `arm` nodes and resolves
    /// each one's variant by constructor name; an `if` owns no arm at all, so
    /// its two are built here from the same descriptor.
    pub(super) fn check_if(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
        value_delivery: bool,
    ) -> Result<MatchResult, CheckStop> {
        if (self.tree.production(node)? == Production::ValueIf) != value_delivery {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        self.check_conditional(function, node, bindings, counters, scope, value_delivery)
    }

    /// The conditional body shared by both forms.
    ///
    /// `opens_delivery` is not "this is a `value_if`": [GIVE-1] gives an
    /// else-if chain one delivery set belonging to the chain's binding, so
    /// only the outermost `value_if` opens the context and every chained one
    /// contributes to it, exactly as a statement `match` propagates `give`s.
    fn check_conditional(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
        scope: ControlScope<'_>,
        opens_delivery: bool,
    ) -> Result<MatchResult, CheckStop> {
        let value_if = self.tree.production(node)? == Production::ValueIf;
        let expression_node = self
            .tree
            .first_child_with(node, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let condition =
            self.check_match_expression(function, expression_node, bindings, scope.loops.len())?;
        // [TYPE-7] exclusivity, which [GRAM-6] keeps: a condition reached
        // through a holder is the implicit read, and its own `own Bool`
        // judgment forms no rejection. `RequiredReferent::Enum` already
        // admits `Bool`, the prelude enum this condition must be.
        if self.reads_implicitly_through_holder(
            condition.reference_value,
            condition.expression.ty(),
            RequiredReferent::Enum,
        )? {
            return self.issue_node(
                SemanticRule::Type7,
                expression_node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        // [GRAM-6] the condition takes [OP-5]'s judgment exactly; every
        // failure that is not TYPE-7's implicit read cites GRAM-6 here.
        if condition.expression.ty() != CheckedType::Bool || condition.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Gram6,
                expression_node,
                SemanticIssueKind::InvalidConditionalForm {
                    mechanical_fix: "give the condition exact value mode and type `own Bool`",
                },
            );
        }
        let blocks = self.tree.conditional_blocks(node)?;
        self.reject_unspellable_else(node, &blocks.alternative, value_if)?;

        let base_bindings = bindings.clone();
        let base_keys = base_bindings.keys().copied().collect::<Vec<_>>();
        let base_key_set = base_keys.iter().copied().collect::<HashSet<_>>();
        let local_give_context = opens_delivery.then(|| GiveContext::empty(&base_key_set, scope));
        let arm_scope = ControlScope {
            loops: scope.loops,
            give_context: local_give_context.as_ref().or(scope.give_context),
        };

        let mut then_bindings = base_bindings.clone();
        let then_checked = self.check_block(
            function,
            &blocks.then_statements,
            &mut then_bindings,
            counters,
            arm_scope,
        )?;
        let mut else_bindings = base_bindings.clone();
        let else_checked = match &blocks.alternative {
            // [ERR-2] the else-free `if` is the empty-alternative form, so its
            // False arm is the empty block rather than a missing one.
            ConditionalAlternative::Absent => {
                self.check_block(function, &[], &mut else_bindings, counters, arm_scope)?
            }
            ConditionalAlternative::Block(statements) => self.check_block(
                function,
                statements,
                &mut else_bindings,
                counters,
                arm_scope,
            )?,
            // An `else if` chain: the nested conditional is the whole
            // alternative and is not wrapped in a `stmt` node. It never opens
            // a delivery context of its own — [GIVE-1] gives the whole chain
            // one delivery set, belonging to the chain's binding.
            ConditionalAlternative::Chain(nested) => {
                let chained = self.check_conditional(
                    function,
                    *nested,
                    &mut else_bindings,
                    counters,
                    arm_scope,
                    false,
                )?;
                BlockResult {
                    statements: vec![CheckedStatement::Match {
                        scrutinee: chained.scrutinee,
                        enum_type: chained.enum_type,
                        arms: chained.arms,
                        continues: chained.can_continue,
                    }],
                    can_continue: chained.can_continue,
                    effects: chained.effects,
                    all_paths_deliver: chained.all_paths_deliver,
                    give_states: chained.give_states,
                    break_states: chained.break_states,
                }
            }
        };

        let mut arms = Vec::with_capacity(2);
        let mut normal_states = Vec::new();
        let mut give_states = Vec::new();
        let mut break_states = Vec::new();
        let mut effects = condition.effects.clone();
        let mut all_paths_deliver = true;
        // The then-block is the `True` arm and the alternative is the `False`
        // arm, tagged from the one Bool descriptor the `match` spelling used
        // so the two spellings cannot drift apart. `bool_descriptor` lists the
        // variants in that order.
        let descriptor = Self::bool_descriptor();
        for (variant, (checked, branch_bindings)) in descriptor
            .variants
            .iter()
            .zip([(then_checked, then_bindings), (else_checked, else_bindings)])
        {
            let fallthrough_drops = if checked.can_continue {
                self.live_affine_drops(&branch_bindings, &base_key_set)?
            } else {
                Vec::new()
            };
            if checked.can_continue {
                normal_states.push(branch_bindings);
            }
            all_paths_deliver &= !checked.can_continue && checked.all_paths_deliver;
            effects = effects.union(checked.effects);
            give_states.extend(checked.give_states);
            break_states.extend(checked.break_states);
            arms.push(CheckedMatchArm {
                tag: variant.tag,
                binders: Vec::new(),
                body: checked.statements,
                fallthrough_drops,
            });
        }
        if opens_delivery {
            if !all_paths_deliver {
                return self.issue_node(SemanticRule::Give1, node, SemanticIssueKind::InvalidGive);
            }
            self.reject_slice_valued_delivery(
                node,
                local_give_context.as_ref().and_then(GiveContext::delivered),
            )?;
            self.join_states(&base_keys, &give_states, node, bindings)?;
        } else {
            self.join_states(&base_keys, &normal_states, node, bindings)?;
        }
        Ok(MatchResult {
            scrutinee: condition.expression,
            enum_type: CheckedEnumType::Bool,
            arms,
            delivered: local_give_context.as_ref().and_then(GiveContext::delivered),
            can_continue: if opens_delivery {
                !give_states.is_empty()
            } else {
                !normal_states.is_empty()
            },
            all_paths_deliver,
            effects,
            give_states: if opens_delivery {
                Vec::new()
            } else {
                give_states
            },
            break_states,
        })
    }

    /// [GRAM-6] the two `else` spellings the rule refuses, each reported at
    /// the node the rule names.
    fn reject_unspellable_else(
        &self,
        node: NodeId,
        alternative: &ConditionalAlternative,
        value_if: bool,
    ) -> Result<(), CheckStop> {
        let ConditionalAlternative::Block(statements) = alternative else {
            return Ok(());
        };
        if statements.is_empty() {
            // A `value_if`'s empty `else` delivers nothing, and that is
            // [GIVE-1]'s empty delivery set rather than this rejection.
            if value_if {
                return Ok(());
            }
            return self.issue_node(
                SemanticRule::Gram6,
                node,
                SemanticIssueKind::InvalidConditionalForm {
                    mechanical_fix: "delete the empty `else` and spell the else-free `if`",
                },
            );
        }
        let [only] = statements.as_slice() else {
            return Ok(());
        };
        let nested = self.tree.only_child(*only)?;
        if self.tree.production(nested)? != Production::IfStmt {
            return Ok(());
        }
        // In a `value_if` whose else block is exactly one else-free `if`, the
        // branch cannot deliver, [GIVE-1] owns that rejection, and the chain
        // form could not be spelled anyway — so GRAM-6 forms no candidate.
        if value_if
            && matches!(
                self.tree.conditional_blocks(nested)?.alternative,
                ConditionalAlternative::Absent
            )
        {
            return Ok(());
        }
        self.issue_node(
            SemanticRule::Gram6,
            nested,
            SemanticIssueKind::InvalidConditionalForm {
                mechanical_fix: "flatten the nested `if` to `else if`",
            },
        )
    }

    fn bool_descriptor() -> MatchDescriptor {
        MatchDescriptor {
            enum_type: CheckedEnumType::Bool,
            variants: vec![
                VariantDescriptor {
                    name: "True".to_owned(),
                    tag: 1,
                    fields: Vec::new(),
                    constructor: CheckedConstructor::Prelude(crate::PreludeDeclarationId::new(1)),
                },
                VariantDescriptor {
                    name: "False".to_owned(),
                    tag: 0,
                    fields: Vec::new(),
                    constructor: CheckedConstructor::Prelude(crate::PreludeDeclarationId::new(2)),
                },
            ],
        }
    }

    fn match_descriptor(
        &self,
        ty: CheckedType,
        node: NodeId,
    ) -> Result<MatchDescriptor, CheckStop> {
        match ty {
            // [GRAM-6] conditional control is type-driven and each form is the
            // sole legal one for its class, so a Bool scrutinee is rejected
            // here whatever its arms spell. Its descriptor survives below for
            // `if`, which is the spelling this class does take.
            CheckedType::Bool => self.issue_node(
                SemanticRule::Gram6,
                node,
                SemanticIssueKind::InvalidConditionalForm {
                    mechanical_fix: "spell the Bool conditional `if`",
                },
            ),
            CheckedType::Nominal(id) => {
                // [TYPE-7]'s implicit read was already excluded by the caller,
                // so a non-enum nominal here is the scrutinee's own mismatch.
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
                ResolvedTarget::System(id) => variant.constructor == CheckedConstructor::System(id),
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
        counters: &mut ControlCounters<'_>,
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
            let binding = Self::allocate_binding(counters.next_binding)?;
            counters
                .binding_names
                .push(declaration.spelling().to_owned());
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
            let capability_origins = self.capability_origins_of_value(scrutinee, bindings)?;
            if bindings
                .insert(
                    declaration.id(),
                    LocalBinding {
                        binding,
                        declaration: declaration.id(),
                        mode,
                        ty: field.ty,
                        capability_origins: self
                            .type_carries_one_capability(field.ty)?
                            .then(|| capability_origins.clone())
                            .flatten(),
                        live: true,
                        loop_depth,
                        compiler_updated: false,
                        borrow,
                        slice: None,
                        slice_loans: Vec::new(),
                        suspended: false,
                    },
                )
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            binders.push(CheckedMatchBinder {
                node_path: self.tree.path(written)?.clone(),
                binding,
                field: u32::try_from(index)
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                mode,
                ty: field.ty,
            });
        }
        // Creating the taken arm's binders from a `uniq`-mode root suspends
        // that root binding [OWN-13, OWN-5]; the binders' own loans carry the
        // exclusivity for the region remainder. Shared-mode roots stay plain
        // overlapping shared borrows without suspension.
        if matches!(mode, CheckedMode::Unique(_))
            && !binders.is_empty()
            && let Some(root) = scrutinee.holder
        {
            bindings
                .get_mut(&root)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .suspended = true;
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

    /// [OWN-5] a slice-valued delivery is prohibited outright, whatever its
    /// mode and whatever the arms do to their bindings.
    ///
    /// It is judged here, before [`Self::join_states`], because the join is a
    /// capability limit and a capability stop must never stand in front of a
    /// source rejection. Judged after it, this rejection was unreachable for
    /// exactly the sources that matter — arms delivering *different* bindings,
    /// which is what a slice-valued join looks like when it is written on
    /// purpose.
    fn reject_slice_valued_delivery(
        &self,
        node: NodeId,
        delivered: Option<(CheckedMode, CheckedType)>,
    ) -> Result<(), CheckStop> {
        if matches!(delivered, Some((_, CheckedType::Slice { .. }))) {
            return self.issue_node(
                SemanticRule::Own5,
                node,
                SemanticIssueKind::SliceValueMatch {
                    mechanical_fix: "use a match or if statement whose branches return the slice directly, or call helpers with direct slice results",
                },
            );
        }
        Ok(())
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
            let mut joined = first
                .get(key)
                .cloned()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            for state in states.iter().skip(1) {
                let candidate = state
                    .get(key)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !joined.same_except_region_claims(candidate) {
                    return self.unsupported(UnsupportedSemanticFeature::OwnershipJoin, node);
                }
                joined.merge_region_claims_from(candidate);
            }
            *bindings
                .get_mut(key)
                .ok_or(SemanticCompilerFailure::InvalidResolution)? = joined;
        }
        Ok(())
    }
}
