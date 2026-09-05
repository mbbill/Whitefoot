use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::super::model::{
    BindingId, CheckedMode, CheckedNominalKind, CheckedStatement, CheckedType, PropagationContext,
};
use super::super::{CheckStop, Checker, FunctionSignature, LocalBinding, PreludeType};
use super::{ControlScope, StatementResult};

// [DIAG-1] the return position asks TYPE-7's implicit read before the
// operand's OWN-1 spelling judgments because TYPE-7 is defined first.
const _: () = assert!(
    SemanticRule::Type7.definition_rank() < SemanticRule::Own1.definition_rank(),
    "check_return_implicit_read precedes the operand's OWN-1 judgments"
);

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// The result-list nominal a checked value carries, when it is one
    /// [GRAM-2, CALL-4].
    ///
    /// A destructuring binder list and a `set` target list are the two places
    /// that name a callee's result ordinals again, and both ask this one
    /// question of the value in front of them rather than of the callee's
    /// spelling or of the statement's shape.
    pub(super) fn result_list_of(
        &self,
        value: &super::super::TypedExpression,
    ) -> Option<crate::NominalId> {
        let CheckedType::Nominal(nominal) = value.expression.ty() else {
            return None;
        };
        (value.mode == CheckedMode::Own
            && self
                .result_list_nominals
                .values()
                .any(|other| *other == nominal))
        .then_some(nominal)
    }

    /// The declared result ordinals of a result-list nominal, in written
    /// order.
    pub(super) fn result_list_ordinals(
        &self,
        nominal: crate::NominalId,
    ) -> Result<Vec<CheckedType>, CheckStop> {
        match &self.nominal(nominal)?.kind {
            CheckedNominalKind::Struct { fields } => {
                Ok(fields.iter().map(|field| field.ty).collect())
            }
            _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
    }

    /// The [TYPE-5] rejection a binder or target list receives when its
    /// right-hand side does not produce exactly that many result ordinals.
    pub(super) fn result_list_shape_rejection<T>(
        &self,
        call: NodeId,
        written: usize,
        value: &super::super::TypedExpression,
    ) -> Result<T, CheckStop> {
        self.issue_node(
            SemanticRule::Type5,
            call,
            SemanticIssueKind::type_mismatch(
                format!("an ordered result list of {written} results"),
                self.checked_value_name(value.mode, value.expression.ty())?,
            ),
        )
    }

    /// Checks `let (a, b) = f(...);` [GRAM-4, TYPE-5, CALL-4].
    ///
    /// The call is evaluated once; binder i is an ordinary fresh `let`
    /// binding of result ordinal i, at that ordinal's declared type and mode.
    pub(super) fn check_destructuring_let(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        call: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut super::ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let declarations = self
            .declarations_at(node, crate::DeclarationRole::Let)?
            .iter()
            .map(|declaration| (declaration.id(), declaration.spelling().to_owned()))
            .collect::<Vec<_>>();
        let value = self.check_call(function, call, bindings, scope.loops.len())?;
        let Some(nominal) = self.result_list_of(&value) else {
            return self.result_list_shape_rejection(call, declarations.len(), &value);
        };
        let ordinals = self.result_list_ordinals(nominal)?;
        if ordinals.len() != declarations.len() {
            return self.result_list_shape_rejection(call, declarations.len(), &value);
        }
        let whole_origins = self.state_origins_of_value(&value, bindings)?;
        let mut binder_ids = Vec::with_capacity(ordinals.len());
        for (ordinal, ((declaration_id, spelling), ty)) in
            declarations.into_iter().zip(ordinals).enumerate()
        {
            let binding = Self::allocate_binding(counters.next_binding)?;
            counters.binding_names.push(spelling);
            let field =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            let state_origins = if self.type_carries_identity(ty)? {
                whole_origins
                    .clone()
                    .map(|origins| origins.projected(&[field]))
            } else {
                None
            };
            if bindings
                .insert(
                    declaration_id,
                    LocalBinding {
                        binding,
                        declaration: declaration_id,
                        mode: CheckedMode::Own,
                        ty,
                        state_origins,
                        live: true,
                        loop_depth: scope.loops.len(),
                        compiler_updated: false,
                        borrow: None,
                        slice: None,
                        slice_loans: Vec::new(),
                        suspended: false,
                    },
                )
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            binder_ids.push((binding, ty));
        }
        Ok(Self::continuing_statement(
            CheckedStatement::DestructuringLet {
                node_path: self.tree.path(node)?.clone(),
                bindings: binder_ids,
                nominal,
                value: value.expression,
            },
            value.effects,
        ))
    }

    /// Checks `let N(f1: b1, ..., fk: bk) = move v;` [GRAM-4, PROV-6].
    ///
    /// The value is consumed whole and every declared field of `N` is bound
    /// in declaration order, so no residual of `v` survives the statement and
    /// nothing here derives a release of the consumed value's own storage.
    pub(super) fn check_destructuring_consume(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        place: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut super::ControlCounters<'_>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let usage = self.use_at(node, LexicalUseRole::Construct)?;
        let written = usage.spelling().to_owned();
        // [S39] the one compiler-owned nominal this statement takes apart is
        // the cell, whose one field is its referent: the destructuring is
        // what takes the value out and releases the cell, and it is the
        // existing statement rather than a new operation.
        let cell = matches!(usage.target(), ResolvedTarget::Container(id)
            if crate::container_nominal(id)
                .is_some_and(|entry| entry.shape == crate::ContainerShape::Box));
        let source_declaration = match usage.target() {
            ResolvedTarget::Source { declaration, .. } => Some(declaration),
            _ if cell => None,
            _ => return self.destructuring_shape_rejection(node, &written),
        };
        let value =
            self.check_consumed_place(function, node, place, bindings, scope.loops.len(), true)?;
        if value.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "destructure an own-mode value; a borrow owns nothing to take apart",
                },
            );
        }
        let CheckedType::Nominal(nominal) = value.expression.ty() else {
            return self.destructuring_shape_rejection(node, &written);
        };
        if let Some(nominal_declaration) = source_declaration
            && !self.nominal_instantiates(nominal, nominal_declaration)?
        {
            return self.destructuring_shape_rejection(node, &written);
        }
        let fields = match &self.nominal(nominal)?.kind {
            CheckedNominalKind::Struct { fields } if !cell => fields.clone(),
            CheckedNominalKind::Box {
                referent,
                region: Some(_),
                ..
            } if cell => vec![super::super::super::model::CheckedField {
                name: "value".to_owned(),
                ty: *referent,
            }],
            _ => return self.destructuring_shape_rejection(node, &written),
        };
        let binders = match self
            .tree
            .first_child_with(node, Production::FieldbindList)?
        {
            Some(list) => self.tree.children_with(list, Production::Fieldbind)?,
            None => Vec::new(),
        };
        if binders.len() != fields.len() {
            return self.invalid_destructuring_fields(&written, &fields, node);
        }
        let whole_origins = self.state_origins_of_value(&value, bindings)?;
        let mut binder_ids = Vec::with_capacity(fields.len());
        for (ordinal, (written_binder, field)) in binders.into_iter().zip(&fields).enumerate() {
            if self
                .deferred_use_at(written_binder, crate::DeferredUseRole::MatchField)?
                .spelling()
                != field.name
            {
                return self.invalid_destructuring_fields(&written, &fields, written_binder);
            }
            let declaration = self.declaration_at(written_binder, crate::DeclarationRole::Let)?;
            let binding = Self::allocate_binding(counters.next_binding)?;
            counters
                .binding_names
                .push(declaration.spelling().to_owned());
            let ordinal =
                u32::try_from(ordinal).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            let state_origins = if self.type_carries_identity(field.ty)? {
                whole_origins
                    .clone()
                    .map(|origins| origins.projected(&[ordinal]))
            } else {
                None
            };
            if bindings
                .insert(
                    declaration.id(),
                    LocalBinding {
                        binding,
                        declaration: declaration.id(),
                        mode: CheckedMode::Own,
                        ty: field.ty,
                        state_origins,
                        live: true,
                        loop_depth: scope.loops.len(),
                        compiler_updated: false,
                        borrow: None,
                        slice: None,
                        slice_loans: Vec::new(),
                        suspended: false,
                    },
                )
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            binder_ids.push((binding, field.ty));
        }
        Ok(Self::continuing_statement(
            CheckedStatement::DestructuringLet {
                node_path: self.tree.path(node)?.clone(),
                bindings: binder_ids,
                nominal,
                value: value.expression,
            },
            value.effects,
        ))
    }

    /// [TYPE-5] the destructuring consume's operand is not a value of the
    /// nominal struct type it writes.
    fn destructuring_shape_rejection<T>(
        &self,
        node: NodeId,
        written: &str,
    ) -> Result<T, CheckStop> {
        self.issue_node(
            SemanticRule::Type5,
            node,
            SemanticIssueKind::type_mismatch(
                format!("an own value of struct type {written}"),
                "a value of another type".to_owned(),
            ),
        )
    }

    /// [GRAM-10] the destructuring consume writes every declared field of its
    /// nominal exactly once, in declared order.
    fn invalid_destructuring_fields<T>(
        &self,
        written: &str,
        fields: &[super::super::super::model::CheckedField],
        node: NodeId,
    ) -> Result<T, CheckStop> {
        self.issue_node(
            SemanticRule::Gram10,
            node,
            SemanticIssueKind::InvalidMatchFields {
                variant: written.to_owned(),
                declared_fields: fields.iter().map(|field| field.name.clone()).collect(),
            },
        )
    }

    /// Checks `return e1, ..., en;` in a declaration that writes an ordered
    /// result list [GRAM-2, GRAM-4, FN-1, CALL-4].
    ///
    /// Expression i produces result ordinal i under exactly the ordinary
    /// return judgment for that ordinal's written `rtype`, and the statement
    /// hands back the one result-list value carrying them in written order.
    /// Nothing below this point sees a second return shape.
    pub(super) fn check_result_list_return(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let nominal = function
            .result_list
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let expressions = self.tree.children_with(node, Production::Expr)?;
        let mut fields = Vec::with_capacity(expressions.len());
        let mut effects = super::super::EffectSet::NONE;
        for (expression_node, declared) in expressions.iter().zip(&function.results) {
            self.check_return_implicit_read(function, *expression_node, bindings)?;
            let value =
                self.check_expression(function, *expression_node, bindings, scope.loops.len())?;
            if value.expression.ty() != declared.ty || value.mode != CheckedMode::Own {
                return Err(CheckStop::source_issue(crate::SemanticIssue {
                    rule: SemanticRule::Fn1,
                    location: crate::SemanticLocation::SourceNode(
                        self.tree.path(node)?.clone(),
                        self.tree.coordinate(*expression_node)?,
                    ),
                    kind: SemanticIssueKind::ReturnMismatch,
                }));
            }
            self.borrow_for_destination(CheckedMode::Own, &value, *expression_node)?;
            effects = effects.union(value.effects);
            fields.push(value.expression);
        }
        let node_path = self.tree.path(node)?.clone();
        Ok(StatementResult {
            statement: CheckedStatement::Return {
                node_path: node_path.clone(),
                value: super::super::super::model::CheckedExpression::ConstructStruct {
                    carrier: node_path,
                    nominal,
                    fields,
                },
                drops: self.live_affine_drops(bindings, &HashSet::new(), node)?,
            },
            can_continue: false,
            effects,
            all_paths_deliver: true,
            direct_give: false,
            give_states: Vec::new(),
            break_states: Vec::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn check_propagate_let(
        &self,
        function: &FunctionSignature,
        let_statement: NodeId,
        propagate: NodeId,
        declaration: DeclarationId,
        binding: BindingId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        scope: ControlScope<'_>,
    ) -> Result<StatementResult, CheckStop> {
        let expression_node = self
            .tree
            .first_child_with(propagate, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // [TYPE-5] a `propagate_let_rhs` binder is derived from the
        // propagated Ok payload [ERR-3], so the operand carries no
        // expectation and the payload is read off its Result type below.
        let value = self.check_consuming_expression(
            function,
            expression_node,
            bindings,
            scope.loops.len(),
        )?;
        let holder_without_deref = value.mode != CheckedMode::Own
            || match value.expression.ty() {
                CheckedType::Nominal(nominal) => {
                    matches!(self.nominal(nominal)?.kind, CheckedNominalKind::Box { .. })
                }
                _ => false,
            };
        if holder_without_deref {
            return self.issue_node(
                SemanticRule::Type7,
                expression_node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        let CheckedType::Nominal(result_nominal) = value.expression.ty() else {
            return self.invalid_propagation(propagate);
        };
        let Some(PreludeType::Result(ok_type, error_type)) = self.prelude_type(result_nominal)
        else {
            return self.invalid_propagation(propagate);
        };
        let CheckedType::Nominal(return_nominal) = function.result else {
            return self.invalid_propagation(propagate);
        };
        let Some(PreludeType::Result(_, return_error_type)) = self.prelude_type(return_nominal)
        else {
            return self.invalid_propagation(propagate);
        };
        if error_type != return_error_type {
            return self.invalid_propagation(propagate);
        }
        let error_drops = self.live_affine_drops(bindings, &HashSet::new(), propagate)?;
        let result_state_origins = self.state_origins_of_value(&value, bindings)?;
        let ok_state_origins = result_state_origins
            .clone()
            .map(|origins| origins.enum_payload(0, 0));
        if bindings
            .insert(
                declaration,
                LocalBinding {
                    binding,
                    declaration,
                    mode: CheckedMode::Own,
                    ty: ok_type,
                    state_origins: self
                        .type_carries_identity(ok_type)?
                        .then_some(ok_state_origins)
                        .flatten(),
                    live: true,
                    loop_depth: scope.loops.len(),
                    compiler_updated: false,
                    borrow: None,
                    slice: None,
                    slice_loans: Vec::new(),
                    suspended: false,
                },
            )
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        Ok(Self::continuing_statement(
            CheckedStatement::PropagateLet {
                node_path: self.tree.path(let_statement)?.clone(),
                binding,
                scrutinee: value.expression,
                result_nominal,
                return_nominal,
                ok_type,
                error_type,
                error_drops,
                context: PropagationContext {
                    function: function.name.clone(),
                    node_path: self.tree.path(propagate)?.clone(),
                },
            },
            value.effects,
        ))
    }

    fn invalid_propagation<ResultValue>(&self, node: NodeId) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Err3,
            node,
            SemanticIssueKind::InvalidPropagation,
        )
    }

    /// [TYPE-7]'s implicit read at return position: a live borrow-mode or
    /// box binding used where the written `rtype` requires its referent
    /// value is rejected citing TYPE-7, and [FN-1] forms no candidate for
    /// that use. TYPE-7's definition precedes OWN-1's spelling judgments,
    /// so this same-node rejection event is cited first
    /// [DIAG-1, `SemanticRule::definition_rank`].
    pub(super) fn check_return_implicit_read(
        &self,
        function: &FunctionSignature,
        expression_node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
    ) -> Result<(), CheckStop> {
        if function.result_mode != CheckedMode::Own {
            return Ok(());
        }
        // An infix expression reads its operands as operands, not as the
        // returned value, so it has no implicit read at return position.
        let Some(atom) = self.tree.sole_expression_child(expression_node)? else {
            return Ok(());
        };
        if self.tree.production(atom)? != Production::Atom {
            return Ok(());
        }
        let Some(place) = self.tree.first_child_with(atom, Production::Place)? else {
            return Ok(());
        };
        let Some(pbase) = self.tree.first_child_with(place, Production::Pbase)? else {
            return Ok(());
        };
        if !self.tree.children(pbase)?.is_empty()
            || !self
                .tree
                .children_with(place, Production::Psuffix)?
                .is_empty()
        {
            return Ok(());
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::Value,
        } = usage.target()
        else {
            return Ok(());
        };
        let Some(local) = bindings.get(&declaration) else {
            return Ok(());
        };
        if !local.live {
            return Ok(());
        }
        let referent = if local.mode != CheckedMode::Own {
            Some(local.ty)
        } else if let CheckedType::Nominal(nominal) = local.ty
            && let CheckedNominalKind::Box { referent, .. } = self.nominal(nominal)?.kind
        {
            Some(referent)
        } else {
            None
        };
        if referent == Some(function.result) {
            return self.issue_node(
                SemanticRule::Type7,
                expression_node,
                SemanticIssueKind::MissingDereference {
                    mechanical_fix: "write `deref(holder)`",
                },
            );
        }
        Ok(())
    }
}
