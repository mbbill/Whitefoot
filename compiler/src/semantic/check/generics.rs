use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, FixedTerminal, LexicalUseRole,
    PreludeDeclarationId, Production, ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind,
    SemanticRule, UnsupportedSemanticFeature,
};

use super::super::goal::{GoalDatum, GoalExpression, GoalOperation};
use super::super::model::{
    CheckedConst, CheckedFlatElement, CheckedGenericRequirement, CheckedType, CheckedValue,
};
use super::{CheckStop, Checker, FunctionSignature, FunctionTemplate, derive_slice_return_ceiling};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum GenericBound {
    Unbounded,
    Int,
    Float,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum GenericParameter {
    Type {
        declaration: DeclarationId,
        bound: GenericBound,
    },
    Const {
        declaration: DeclarationId,
    },
}

impl GenericParameter {
    pub(super) const fn declaration(self) -> DeclarationId {
        match self {
            Self::Type { declaration, .. } | Self::Const { declaration } => declaration,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum GenericArgument {
    Type(CheckedType),
    Const(CheckedConst),
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(super) struct GenericSubstitution {
    bindings: Vec<(DeclarationId, GenericArgument)>,
}

impl GenericSubstitution {
    pub(super) fn from_bindings(
        bindings: Vec<(DeclarationId, GenericArgument)>,
    ) -> Result<Self, SemanticCompilerFailure> {
        for (index, (declaration, _)) in bindings.iter().enumerate() {
            if bindings[..index]
                .iter()
                .any(|(earlier, _)| earlier == declaration)
            {
                return Err(SemanticCompilerFailure::InvalidResolution);
            }
        }
        Ok(Self { bindings })
    }

    pub(super) fn len(&self) -> usize {
        self.bindings.len()
    }

    pub(super) fn type_argument(&self, declaration: DeclarationId) -> Option<CheckedType> {
        self.bindings
            .iter()
            .find_map(|(candidate, argument)| (*candidate == declaration).then_some(argument))
            .and_then(|argument| match argument {
                GenericArgument::Type(ty) => Some(*ty),
                GenericArgument::Const(_) => None,
            })
    }

    pub(super) fn const_argument(&self, declaration: DeclarationId) -> Option<CheckedConst> {
        self.bindings
            .iter()
            .find_map(|(candidate, argument)| (*candidate == declaration).then_some(argument))
            .and_then(|argument| match argument {
                GenericArgument::Const(value) => Some(*value),
                GenericArgument::Type(_) => None,
            })
    }

    /// Whether this is the symbolic validation substitution of its own
    /// template: every parameter stands for itself, which is the shape
    /// [`Checker::validate_generic_templates`] builds to check a written
    /// generic body once.
    pub(super) fn is_symbolic(&self) -> bool {
        !self.bindings.is_empty()
            && self
                .bindings
                .iter()
                .all(|(declaration, argument)| match argument {
                    GenericArgument::Type(
                        CheckedType::Generic(bound)
                        | CheckedType::GenericInt(bound)
                        | CheckedType::GenericFloat(bound),
                    ) => bound == declaration,
                    GenericArgument::Const(CheckedConst::Parameter(bound)) => bound == declaration,
                    GenericArgument::Type(_) | GenericArgument::Const(_) => false,
                })
    }

    pub(super) fn is_concrete(&self) -> bool {
        self.bindings.iter().all(|(_, argument)| match argument {
            GenericArgument::Type(ty) => ty.is_concrete(),
            GenericArgument::Const(value) => value.is_concrete(),
        })
    }

    pub(super) fn entries(&self) -> &[(DeclarationId, GenericArgument)] {
        &self.bindings
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn collect_function_templates(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        self.collect_function_template_inventory(items)?;
        self.reject_generic_call_cycles()?;
        self.validate_generic_templates()?;
        Ok(())
    }

    /// Builds only the source template inventory and exact generic-cycle
    /// judgment needed by the throwaway selector checker. Generic bodies are
    /// ordinary semantic premises and are checked later by the real H0 path.
    pub(super) fn collect_function_templates_for_postconditions(
        &mut self,
        items: &[NodeId],
    ) -> Result<(), CheckStop> {
        let nodes = items
            .iter()
            .copied()
            .filter(|node| {
                self.tree
                    .production(*node)
                    .is_ok_and(|production| production == Production::FnDecl)
            })
            .collect::<Vec<_>>();
        for node in nodes {
            match self.collect_function_template(node) {
                Ok(()) => {}
                Err(CheckStop::Issue(_) | CheckStop::Unsupported(_)) => {
                    let declaration = self.declaration_at(node, DeclarationRole::Function)?.id();
                    self.mark_postcondition_unavailable(declaration);
                }
                Err(stop) => return Err(stop),
            }
        }
        let (unavailable, _) = self.generic_cycle_analysis()?;
        for (index, is_unavailable) in unavailable.into_iter().enumerate() {
            if is_unavailable {
                let declaration = self
                    .function_templates
                    .get(index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .declaration;
                self.mark_postcondition_unavailable(declaration);
            }
        }
        Ok(())
    }

    fn collect_function_template_inventory(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        let nodes = items
            .iter()
            .copied()
            .filter(|node| {
                self.tree
                    .production(*node)
                    .is_ok_and(|production| production == Production::FnDecl)
            })
            .collect::<Vec<_>>();
        for node in nodes {
            self.collect_function_template(node)?;
        }
        Ok(())
    }

    fn collect_function_template(&mut self, node: NodeId) -> Result<(), CheckStop> {
        let declaration = self.declaration_at(node, DeclarationRole::Function)?;
        let template = FunctionTemplate {
            declaration: declaration.id(),
            node,
            name: declaration.spelling().to_owned(),
            generic_parameters: self.parse_generic_parameters(node)?,
        };
        if !template.generic_parameters.is_empty()
            && let Some(regions) = self.tree.first_child_with(node, Production::RegionParams)?
        {
            return self.unsupported(UnsupportedSemanticFeature::Generics, regions);
        }
        let index = self.function_templates.len();
        if self
            .templates_by_declaration
            .insert(template.declaration, index)
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        self.function_templates.push(template);
        Ok(())
    }

    pub(super) fn collect_concrete_function_signatures(&mut self) -> Result<(), CheckStop> {
        self.collect_concrete_function_signatures_with(false)
    }

    /// Scratch-only counterpart used by the FN-9 selector preflight.
    ///
    /// A source-side call that has not completed FN-2 establishes no selector
    /// instance.  The throwaway checker may therefore skip that edge while it
    /// discovers every independently successful instance.  The ordinary
    /// checker keeps the strict path above, so its source diagnostics and
    /// no-`ensures` behavior are unchanged.
    pub(super) fn collect_concrete_function_signatures_for_postconditions(
        &mut self,
    ) -> Result<(), CheckStop> {
        self.collect_concrete_function_signatures_with(true)
    }

    fn collect_concrete_function_signatures_with(
        &mut self,
        tolerate_source_failure: bool,
    ) -> Result<(), CheckStop> {
        for template_index in 0..self.function_templates.len() {
            if tolerate_source_failure
                && self.postcondition_declaration_unavailable(
                    self.function_templates[template_index].declaration,
                )
            {
                continue;
            }
            if self.function_templates[template_index]
                .generic_parameters
                .is_empty()
            {
                let result = if tolerate_source_failure {
                    self.instantiate_function_signature_for_postconditions(
                        template_index,
                        GenericSubstitution::default(),
                    )
                } else {
                    self.instantiate_function_signature(
                        template_index,
                        GenericSubstitution::default(),
                    )
                };
                match result {
                    Ok(()) => {}
                    Err(
                        CheckStop::Issue(_)
                        | CheckStop::Unsupported(_)
                        | CheckStop::PostconditionPrerequisiteUnavailable,
                    ) if tolerate_source_failure => {}
                    Err(stop) => return Err(stop),
                }
            }
        }
        self.discover_called_function_signatures(true, tolerate_source_failure)
    }

    fn discover_called_function_signatures(
        &mut self,
        require_concrete: bool,
        tolerate_source_failure: bool,
    ) -> Result<(), CheckStop> {
        let mut cursor = 0_usize;
        while cursor < self.signatures.len() {
            let signature = self.signatures[cursor].clone();
            for call in self
                .tree
                .descendants_with(signature.node, Production::Call)?
            {
                if self.call_is_inside_postcondition(call)? {
                    continue;
                }
                let Some((template_index, template)) = self.called_function_template(call)? else {
                    continue;
                };
                if tolerate_source_failure
                    && self.postcondition_declaration_unavailable(template.declaration)
                {
                    continue;
                }
                if template.generic_parameters.is_empty() {
                    continue;
                }
                if tolerate_source_failure && !self.postcondition_call_arguments_have_links(call)? {
                    continue;
                }
                if tolerate_source_failure
                    && let Some(targs) = self.tree.first_child_with(call, Production::Targs)?
                {
                    let checkpoint = self.nominal_checkpoint();
                    match self.ensure_nominals_in_node(targs, &signature.substitution) {
                        Ok(()) => {}
                        Err(
                            CheckStop::Issue(_)
                            | CheckStop::Unsupported(_)
                            | CheckStop::PostconditionPrerequisiteUnavailable,
                        ) => {
                            self.restore_nominal_checkpoint(checkpoint)?;
                            continue;
                        }
                        Err(stop) => return Err(stop),
                    }
                }
                let substitution = match self.call_generic_substitution(
                    call,
                    &template,
                    &signature.substitution,
                ) {
                    Ok(substitution) => substitution,
                    Err(
                        CheckStop::Issue(_)
                        | CheckStop::Unsupported(_)
                        | CheckStop::PostconditionPrerequisiteUnavailable,
                    ) if tolerate_source_failure => {
                        continue;
                    }
                    Err(stop) => return Err(stop),
                };
                if require_concrete && !substitution.is_concrete() {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                let already_present = self
                    .functions_by_declaration
                    .get(&template.declaration)
                    .into_iter()
                    .flatten()
                    .any(|id| {
                        self.signatures
                            .get(id.0 as usize)
                            .is_some_and(|instance| instance.substitution == substitution)
                    });
                if !already_present {
                    let result = if tolerate_source_failure {
                        self.instantiate_function_signature_for_postconditions(
                            template_index,
                            substitution,
                        )
                    } else {
                        self.instantiate_function_signature(template_index, substitution)
                    };
                    match result {
                        Ok(()) => {}
                        Err(
                            CheckStop::Issue(_)
                            | CheckStop::Unsupported(_)
                            | CheckStop::PostconditionPrerequisiteUnavailable,
                        ) if tolerate_source_failure => {}
                        Err(stop) => return Err(stop),
                    }
                }
            }
            cursor = cursor
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        }
        Ok(())
    }

    pub(super) fn postcondition_call_arguments_have_links(
        &self,
        call: NodeId,
    ) -> Result<bool, CheckStop> {
        let Some(targs) = self.tree.first_child_with(call, Production::Targs)? else {
            return Ok(true);
        };
        let owner = self.tree.path(targs)?.components();
        if self.resolved.lexical_uses().iter().any(|usage| {
            let path = usage.origin().node().components();
            path.len() >= owner.len()
                && path.starts_with(owner)
                && match usage.target() {
                    ResolvedTarget::Source {
                        declaration,
                        class: DeclarationClass::NamedConst,
                    } => !self.constants.contains_key(&declaration),
                    ResolvedTarget::Source {
                        declaration,
                        class: DeclarationClass::NominalType,
                    } => {
                        self.postcondition_declaration_unavailable(declaration)
                            || !self
                                .nominal_templates_by_declaration
                                .contains_key(&declaration)
                    }
                    _ => false,
                }
        }) {
            return Ok(false);
        }
        for ty in self.tree.descendants_with(targs, Production::Type)? {
            if self
                .tree
                .direct_token_with(ty, crate::TerminalPredicate::TypeIdentifier)?
                .is_some()
            {
                let path = self.tree.path(ty)?;
                if !self.resolved.lexical_uses().iter().any(|usage| {
                    usage.role() == LexicalUseRole::Type && usage.origin().node() == path
                }) {
                    return Ok(false);
                }
            }
        }
        for constant in self.tree.descendants_with(targs, Production::Const)? {
            let identifiers = self.tree.direct_identifiers(constant)?;
            if !identifiers.is_empty() {
                let path = self.tree.path(constant)?;
                let uses = self
                    .resolved
                    .lexical_uses()
                    .iter()
                    .filter(|usage| {
                        usage.role() == LexicalUseRole::Const && usage.origin().node() == path
                    })
                    .collect::<Vec<_>>();
                if uses.len() != identifiers.len() {
                    return Ok(false);
                }
                for usage in uses {
                    if let ResolvedTarget::Source {
                        declaration,
                        class: DeclarationClass::NamedConst,
                    } = usage.target()
                        && !self.constants.contains_key(&declaration)
                    {
                        return Ok(false);
                    }
                }
            }
        }
        for argument in self.tree.children_with(targs, Production::Targ)? {
            if self
                .tree
                .first_child_with(argument, Production::Type)?
                .is_some()
                || self
                    .tree
                    .first_child_with(argument, Production::Const)?
                    .is_some()
            {
                continue;
            }
            let path = self.tree.path(argument)?;
            if !self.resolved.lexical_uses().iter().any(|usage| {
                usage.role() == LexicalUseRole::TypeArgumentRegion && usage.origin().node() == path
            }) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn called_function_template(
        &self,
        call: NodeId,
    ) -> Result<Option<(usize, FunctionTemplate)>, CheckStop> {
        let callee = self
            .tree
            .first_child_with(call, Production::Callee)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let path = self.tree.path(callee)?;
        let Some(usage) = self.resolved.lexical_uses().iter().find(|usage| {
            usage.role() == LexicalUseRole::IdentifierCallee && usage.origin().node() == path
        }) else {
            return Ok(None);
        };
        let declaration = match usage.target() {
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Function,
            } => declaration,
            // A system operation is not a user function template; recursion
            // through it is impossible, so it contributes no cycle edge.
            ResolvedTarget::Operation(_) | ResolvedTarget::System(_) => return Ok(None),
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let Some(index) = self.templates_by_declaration.get(&declaration).copied() else {
            if self.postcondition_declaration_unavailable(declaration) {
                return Ok(None);
            }
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let template = self
            .function_templates
            .get(index)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(Some((index, template)))
    }

    pub(super) fn concrete_function_for_call(
        &self,
        node: NodeId,
        declaration: DeclarationId,
        caller: &GenericSubstitution,
    ) -> Result<super::super::model::FunctionId, CheckStop> {
        let template_index = *self
            .templates_by_declaration
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let template = self
            .function_templates
            .get(template_index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let substitution = self.call_generic_substitution(node, template, caller)?;
        self.functions_by_declaration
            .get(&declaration)
            .into_iter()
            .flatten()
            .copied()
            .find(|id| {
                self.signatures
                    .get(id.0 as usize)
                    .is_some_and(|instance| instance.substitution == substitution)
            })
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    fn instantiate_function_signature(
        &mut self,
        template_index: usize,
        substitution: GenericSubstitution,
    ) -> Result<(), CheckStop> {
        let template = self
            .function_templates
            .get(template_index)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let id = super::super::model::FunctionId(
            u32::try_from(self.signatures.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        self.ensure_nominals_in_function(template.node, &substitution)?;
        let signature = self.build_function_signature(&template, substitution, id)?;
        self.functions_by_declaration
            .entry(template.declaration)
            .or_default()
            .push(id);
        self.signatures.push(signature);
        Ok(())
    }

    fn instantiate_function_signature_for_postconditions(
        &mut self,
        template_index: usize,
        substitution: GenericSubstitution,
    ) -> Result<(), CheckStop> {
        let template = self
            .function_templates
            .get(template_index)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let id = super::super::model::FunctionId(
            u32::try_from(self.signatures.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        if !self.postcondition_function_header_dependencies_available(template.node)? {
            return Err(CheckStop::PostconditionPrerequisiteUnavailable);
        }
        let checkpoint = self.nominal_checkpoint();
        let signature = match self
            .ensure_nominals_in_function_signature(template.node, &substitution)
            .and_then(|()| self.build_function_signature(&template, substitution, id))
        {
            Ok(signature) => signature,
            Err(stop) => {
                self.restore_nominal_checkpoint(checkpoint)?;
                self.pending_nominals.borrow_mut().clear();
                return Err(stop);
            }
        };
        self.functions_by_declaration
            .entry(template.declaration)
            .or_default()
            .push(id);
        self.signatures.push(signature);
        Ok(())
    }

    pub(super) fn postcondition_function_header_dependencies_available(
        &self,
        function: NodeId,
    ) -> Result<bool, CheckStop> {
        for owner in [Production::ParamList, Production::Rtype] {
            let Some(node) = self.tree.first_child_with(function, owner)? else {
                if owner == Production::Rtype {
                    return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
                }
                continue;
            };
            let path = self.tree.path(node)?.components();
            if self.resolved.lexical_uses().iter().any(|usage| {
                let usage_path = usage.origin().node().components();
                usage_path.len() >= path.len()
                    && usage_path.starts_with(path)
                    && match usage.target() {
                        ResolvedTarget::Source {
                            declaration,
                            class: DeclarationClass::NamedConst,
                        } => !self.constants.contains_key(&declaration),
                        ResolvedTarget::Source {
                            declaration,
                            class: DeclarationClass::NominalType,
                        } => {
                            self.postcondition_declaration_unavailable(declaration)
                                || !self
                                    .nominal_templates_by_declaration
                                    .contains_key(&declaration)
                        }
                        _ => false,
                    }
            }) {
                return Ok(false);
            }
            for ty in self.tree.descendants_with(node, Production::Type)? {
                if self
                    .tree
                    .direct_token_with(ty, crate::TerminalPredicate::TypeIdentifier)?
                    .is_some()
                {
                    let path = self.tree.path(ty)?;
                    if !self.resolved.lexical_uses().iter().any(|usage| {
                        usage.role() == LexicalUseRole::Type && usage.origin().node() == path
                    }) {
                        return Ok(false);
                    }
                }
            }
            for constant in self.tree.descendants_with(node, Production::Const)? {
                let identifiers = self.tree.direct_identifiers(constant)?;
                if !identifiers.is_empty() {
                    let path = self.tree.path(constant)?;
                    let uses = self
                        .resolved
                        .lexical_uses()
                        .iter()
                        .filter(|usage| {
                            usage.role() == LexicalUseRole::Const && usage.origin().node() == path
                        })
                        .count();
                    if uses != identifiers.len() {
                        return Ok(false);
                    }
                }
            }
        }
        Ok(true)
    }

    pub(super) fn build_function_signature(
        &self,
        template: &FunctionTemplate,
        substitution: GenericSubstitution,
        id: super::super::model::FunctionId,
    ) -> Result<FunctionSignature, CheckStop> {
        let deny_claims_marker = if self
            .tree
            .direct_token_with(
                template.node,
                crate::TerminalPredicate::Fixed(crate::FixedTerminal::DenyClaims),
            )?
            .is_some()
        {
            Some(self.tree.path(template.node)?.clone())
        } else {
            None
        };
        let region_parameters = self.parse_region_parameters(template.node)?;
        let parameters = self.parse_parameters_with(template.node, &substitution)?;
        let rtype = self
            .tree
            .first_child_with(template.node, Production::Rtype)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let (result_mode, result) = self.parse_rtype_with(rtype, &substitution)?;
        // [STOR-4] a value of type `arena<'r, T>` may not be returned, so a
        // result type naming an arena has no legal producing return and is
        // rejected at the callable boundary.
        if self.arena_instance(result)?.is_some() {
            return self.issue_node(
                SemanticRule::Stor4,
                rtype,
                SemanticIssueKind::ArenaEscape {
                    mechanical_fix: super::ARENA_ESCAPE_RESTRUCTURING,
                },
            );
        }
        if result_mode != super::super::model::CheckedMode::Own {
            if matches!(result, super::super::model::CheckedType::Slice { .. }) {
                return self.issue_node(
                    SemanticRule::Fn1,
                    rtype,
                    SemanticIssueKind::BorrowedSliceResult {
                        mechanical_fix: "return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor",
                    },
                );
            }
            if !self.borrowable_type(result)? {
                return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, rtype);
            }
            self.reject_ambiguous_result_provenance(&parameters, result_mode, result, rtype)?;
        }
        let slice_return_ceiling = derive_slice_return_ceiling(&parameters, result_mode, result);
        let effects = self
            .tree
            .first_child_with(template.node, Production::Effects)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let declared_effects = self.parse_effects(effects)?;
        let symbol = if template.generic_parameters.is_empty() {
            template.name.clone()
        } else {
            format!("{}$instance${}", template.name, id.0)
        };
        Ok(FunctionSignature {
            id,
            declaration: template.declaration,
            node: template.node,
            name: template.name.clone(),
            symbol,
            deny_claims_marker,
            region_parameters,
            parameters,
            result_mode,
            result,
            slice_return_ceiling,
            effects_node: effects,
            declared_effects,
            substitution,
        })
    }

    fn validate_generic_templates(&mut self) -> Result<(), CheckStop> {
        if !self.signatures.is_empty()
            || !self.functions_by_declaration.is_empty()
            || !self.generic_requirements.is_empty()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let nominal_checkpoint = self.nominal_checkpoint();
        // Record only the initial source-canonical symbolic instance for each
        // generic. Transitive discovery below may instantiate another
        // symbolic shape for the same source declaration; those validate the
        // source call graph but are not a second metadata identity.
        let mut canonical_generic_signatures = Vec::new();
        for template_index in 0..self.function_templates.len() {
            let template = self
                .function_templates
                .get(template_index)
                .cloned()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let substitution = self.symbolic_generic_substitution(&template.generic_parameters)?;
            let signature_index = self.signatures.len();
            self.instantiate_function_signature(template_index, substitution)?;
            if !template.generic_parameters.is_empty() {
                canonical_generic_signatures.push((signature_index, template.declaration));
            }
        }
        self.discover_called_function_signatures(false, false)?;
        for index in 0..self.signatures.len() {
            if !self.signatures[index].substitution.bindings.is_empty() {
                // Symbolic generic validation may discover a derived box or
                // prelude nominal (for example the Result produced by a
                // `+checked` requires-local). Use the same deferred-nominal
                // retry loop as concrete checking; the checkpoint below
                // discards these symbolic-only instances afterwards.
                let checked = self.check_function_interning_nominals(index)?;
                if let Some((_, declaration)) = canonical_generic_signatures
                    .iter()
                    .find(|(canonical, _)| *canonical == index)
                {
                    if checked.function.declaration != *declaration {
                        return Err(SemanticCompilerFailure::InvalidResolution.into());
                    }
                    if let Some(requirement) = checked.function.requirement {
                        if !goal_uses_nominal_prefix(&requirement.template.root, nominal_checkpoint)
                            || self
                                .generic_requirements
                                .iter()
                                .any(|entry| entry.declaration == *declaration)
                        {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        self.generic_requirements.push(CheckedGenericRequirement {
                            declaration: *declaration,
                            requirement,
                        });
                    }
                }
            }
        }
        self.signatures.clear();
        self.functions_by_declaration.clear();
        self.restore_nominal_checkpoint(nominal_checkpoint)?;
        Ok(())
    }

    pub(super) fn symbolic_generic_substitution(
        &self,
        parameters: &[GenericParameter],
    ) -> Result<GenericSubstitution, CheckStop> {
        let bindings = parameters
            .iter()
            .copied()
            .map(|parameter| {
                let argument = match parameter {
                    GenericParameter::Type {
                        declaration,
                        bound: GenericBound::Int,
                    } => GenericArgument::Type(CheckedType::GenericInt(declaration)),
                    GenericParameter::Type {
                        declaration,
                        bound: GenericBound::Float,
                    } => GenericArgument::Type(CheckedType::GenericFloat(declaration)),
                    GenericParameter::Type {
                        declaration,
                        bound: GenericBound::Unbounded,
                    } => GenericArgument::Type(CheckedType::Generic(declaration)),
                    GenericParameter::Const { declaration } => {
                        GenericArgument::Const(CheckedConst::Parameter(declaration))
                    }
                };
                (parameter.declaration(), argument)
            })
            .collect();
        GenericSubstitution::from_bindings(bindings).map_err(CheckStop::Compiler)
    }

    fn reject_generic_call_cycles(&self) -> Result<(), CheckStop> {
        let edges = self.generic_call_edges()?;
        // [FN-6] recursion is permitted, and polymorphic recursion is rejected
        // by a syntactic rule. That source judgment is asked before the
        // capability report below so that a program the language rejects is
        // never reported as an unimplemented capability instead.
        if let Some((call, cycle)) = self.first_polymorphic_recursion(&edges)? {
            return self.issue_node(
                SemanticRule::Fn6,
                call,
                SemanticIssueKind::PolymorphicRecursion {
                    cycle,
                    mechanical_fix:
                        "instantiate every call on the cycle at exactly the caller's own type parameters, or move the differently instantiated call off the cycle",
                },
            );
        }
        if let Some(call) = self.generic_cycle_components(&edges).1 {
            return self.unsupported(UnsupportedSemanticFeature::Generics, call);
        }
        Ok(())
    }

    /// The first [FN-6] polymorphic-recursion violation in call order, with
    /// the cycle the rule requires the diagnostic to name.
    ///
    /// FN-6 constrains a call cycle *among generic functions*, so an edge is
    /// judged when its caller and callee are both generic and the callee
    /// reaches the caller again. A cycle through a nongeneric participant is
    /// left to the ordinary path: a nongeneric caller has no type parameter to
    /// write, so every argument it writes is fixed and the cycle's instance
    /// set is finite by construction.
    fn first_polymorphic_recursion(
        &self,
        edges: &[Vec<(usize, NodeId)>],
    ) -> Result<Option<(NodeId, String)>, CheckStop> {
        for (caller, outgoing) in edges.iter().enumerate() {
            let caller_parameters = Self::type_parameters(&self.function_templates[caller]);
            if caller_parameters.is_empty() {
                continue;
            }
            for (callee, call) in outgoing {
                let callee_parameters = &self.function_templates[*callee].generic_parameters;
                if callee_parameters.is_empty() || !Self::graph_reaches(*callee, caller, edges) {
                    continue;
                }
                if self.call_instantiates_caller_parameters(
                    *call,
                    &caller_parameters,
                    callee_parameters,
                )? {
                    continue;
                }
                return Ok(Some((
                    *call,
                    self.render_call_cycle(caller, *callee, edges),
                )));
            }
        }
        Ok(None)
    }

    /// Whether one call writes exactly the caller's own type parameters, in
    /// order, at the callee's type-parameter positions [FN-6].
    ///
    /// The judgment is syntactic, as FN-6 states it is: a written `targ`
    /// satisfies it only as a bare TYPEID carrying no arguments of its own
    /// that resolves to the caller's type parameter at the same position, and
    /// the two type-parameter counts must agree for the lists to be equal at
    /// all. An absent or short argument list is [FN-2]'s violation rather than
    /// this rule's, so it is not attributed here.
    fn call_instantiates_caller_parameters(
        &self,
        call: NodeId,
        caller_parameters: &[DeclarationId],
        callee_parameters: &[GenericParameter],
    ) -> Result<bool, CheckStop> {
        let Some(targs) = self.tree.first_child_with(call, Production::Targs)? else {
            return Ok(true);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        if arguments.len() < callee_parameters.len() {
            return Ok(true);
        }
        let mut position = 0;
        for (parameter, argument) in callee_parameters.iter().zip(&arguments) {
            if !matches!(parameter, GenericParameter::Type { .. }) {
                continue;
            }
            let Some(expected) = caller_parameters.get(position) else {
                return Ok(false);
            };
            position += 1;
            if !self.targ_names_type_parameter(*argument, *expected)? {
                return Ok(false);
            }
        }
        Ok(position == caller_parameters.len())
    }

    /// Whether one `targ` is written as exactly the named type parameter.
    fn targ_names_type_parameter(
        &self,
        argument: NodeId,
        expected: DeclarationId,
    ) -> Result<bool, CheckStop> {
        let Some(ty) = self.tree.first_child_with(argument, Production::Type)? else {
            return Ok(false);
        };
        if self
            .tree
            .direct_token_with(ty, TerminalPredicate::TypeIdentifier)?
            .is_none()
            || self.tree.first_child_with(ty, Production::Targs)?.is_some()
        {
            return Ok(false);
        }
        let path = self.tree.path(ty)?;
        Ok(self.resolved.lexical_uses().iter().any(|usage| {
            usage.role() == LexicalUseRole::Type
                && usage.origin().node() == path
                && matches!(
                    usage.target(),
                    ResolvedTarget::Source {
                        declaration,
                        class: DeclarationClass::GenericType,
                    } if declaration == expected
                )
        }))
    }

    fn type_parameters(template: &FunctionTemplate) -> Vec<DeclarationId> {
        template
            .generic_parameters
            .iter()
            .filter_map(|parameter| match parameter {
                GenericParameter::Type { declaration, .. } => Some(*declaration),
                GenericParameter::Const { .. } => None,
            })
            .collect()
    }

    /// The cycle FN-6 requires the diagnostic to name: the caller, the
    /// shortest call path from this call's callee back to it, and the caller
    /// again, so the reader sees where the offending instantiation sits.
    fn render_call_cycle(
        &self,
        caller: usize,
        callee: usize,
        edges: &[Vec<(usize, NodeId)>],
    ) -> String {
        let mut previous = vec![None; edges.len()];
        let mut seen = vec![false; edges.len()];
        let mut pending = std::collections::VecDeque::from([callee]);
        seen[callee] = true;
        while let Some(node) = pending.pop_front() {
            if node == caller {
                break;
            }
            for (next, _) in &edges[node] {
                if !seen[*next] {
                    seen[*next] = true;
                    previous[*next] = Some(node);
                    pending.push_back(*next);
                }
            }
        }
        // The predecessor chain runs backwards from the caller to the callee,
        // so the rendered cycle reverses it and closes on the caller.
        let mut chain = vec![caller];
        let mut cursor = caller;
        while let Some(node) = previous[cursor] {
            chain.push(node);
            cursor = node;
        }
        let mut names = vec![self.function_templates[caller].name.clone()];
        names.extend(
            chain
                .into_iter()
                .rev()
                .map(|index| self.function_templates[index].name.clone()),
        );
        names.join(" -> ")
    }

    fn generic_call_edges(&self) -> Result<Vec<Vec<(usize, NodeId)>>, CheckStop> {
        let mut edges = vec![Vec::new(); self.function_templates.len()];
        for (caller, template) in self.function_templates.iter().enumerate() {
            for call in self
                .tree
                .descendants_with(template.node, Production::Call)?
            {
                if self.call_is_inside_postcondition(call)? {
                    continue;
                }
                let Some((callee, _)) = self.called_function_template(call)? else {
                    continue;
                };
                edges[caller].push((callee, call));
            }
        }
        Ok(edges)
    }

    fn generic_cycle_analysis(&self) -> Result<(Vec<bool>, Option<NodeId>), CheckStop> {
        let edges = self.generic_call_edges()?;
        Ok(self.generic_cycle_components(&edges))
    }

    fn generic_cycle_components(
        &self,
        edges: &[Vec<(usize, NodeId)>],
    ) -> (Vec<bool>, Option<NodeId>) {
        let mut unavailable = vec![false; self.function_templates.len()];
        let mut first = None;
        for (caller, outgoing) in edges.iter().enumerate() {
            for (callee, call) in outgoing {
                if !Self::graph_reaches(*callee, caller, edges) {
                    continue;
                }
                let generic_component = (0..self.function_templates.len()).any(|candidate| {
                    Self::graph_reaches(caller, candidate, edges)
                        && Self::graph_reaches(candidate, caller, edges)
                        && !self.function_templates[candidate]
                            .generic_parameters
                            .is_empty()
                });
                if generic_component {
                    if first.is_none() {
                        first = Some(*call);
                    }
                    for (candidate, slot) in unavailable.iter_mut().enumerate() {
                        if Self::graph_reaches(caller, candidate, edges)
                            && Self::graph_reaches(candidate, caller, edges)
                        {
                            *slot = true;
                        }
                    }
                }
            }
        }
        (unavailable, first)
    }

    pub(super) fn call_is_inside_postcondition(&self, call: NodeId) -> Result<bool, CheckStop> {
        self.node_is_inside_postcondition(call)
    }

    pub(super) fn node_is_inside_postcondition(&self, node: NodeId) -> Result<bool, CheckStop> {
        let path = self.tree.path(node)?.components();
        Ok(self.resolved.postconditions().iter().any(|record| {
            let block = record.block.components();
            path.len() > block.len() && path.starts_with(block)
        }))
    }

    fn graph_reaches(start: usize, target: usize, edges: &[Vec<(usize, NodeId)>]) -> bool {
        let mut seen = vec![false; edges.len()];
        let mut pending = vec![start];
        while let Some(node) = pending.pop() {
            if node == target {
                return true;
            }
            if seen[node] {
                continue;
            }
            seen[node] = true;
            pending.extend(edges[node].iter().rev().map(|(callee, _)| *callee));
        }
        false
    }

    pub(super) fn parse_generic_parameters(
        &self,
        declaration: NodeId,
    ) -> Result<Vec<GenericParameter>, CheckStop> {
        let Some(generics) = self
            .tree
            .first_child_with(declaration, Production::Generics)?
        else {
            return Ok(Vec::new());
        };
        let mut parameters = Vec::new();
        for node in self.tree.children_with(generics, Production::Gparam)? {
            if self.has_fixed(node, FixedTerminal::Const)? {
                let declaration = self
                    .declaration_at(node, DeclarationRole::ConstGeneric)?
                    .id();
                let ty = self
                    .tree
                    .first_child_with(node, Production::Type)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                if self.integer_type(ty)?.is_none() {
                    return self.issue_node(
                        SemanticRule::Const1,
                        ty,
                        SemanticIssueKind::InvalidConstValue,
                    );
                }
                parameters.push(GenericParameter::Const { declaration });
                continue;
            }
            let declaration = self
                .declaration_at(node, DeclarationRole::GenericType)?
                .id();
            let path = self.tree.path(node)?;
            let bound = match self
                .resolved
                .lexical_uses()
                .iter()
                .find(|usage| {
                    usage.role() == LexicalUseRole::GenericBound && usage.origin().node() == path
                })
                .map(|usage| (usage.target(), usage.origin().coordinate()))
            {
                None => GenericBound::Unbounded,
                Some((ResolvedTarget::Prelude(id), _)) if id == PreludeDeclarationId::new(22) => {
                    GenericBound::Int
                }
                Some((ResolvedTarget::Prelude(id), _)) if id == PreludeDeclarationId::new(23) => {
                    GenericBound::Float
                }
                Some((
                    ResolvedTarget::Source {
                        class: DeclarationClass::Contract,
                        ..
                    },
                    coordinate,
                )) => {
                    return self.issue_at(
                        SemanticRule::Fn3,
                        node,
                        coordinate,
                        SemanticIssueKind::SourceContractGenericBound,
                    );
                }
                Some(_) => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            };
            parameters.push(GenericParameter::Type { declaration, bound });
        }
        Ok(parameters)
    }

    pub(super) fn call_generic_substitution(
        &self,
        node: NodeId,
        template: &FunctionTemplate,
        caller: &GenericSubstitution,
    ) -> Result<GenericSubstitution, CheckStop> {
        // [DIAG-1] a user-generic call's argument list is FN-2's.
        self.generic_substitution(
            node,
            &template.generic_parameters,
            caller,
            true,
            SemanticRule::Fn2,
        )
    }

    pub(super) fn nominal_generic_substitution(
        &self,
        node: NodeId,
        parameters: &[GenericParameter],
        caller: &GenericSubstitution,
    ) -> Result<GenericSubstitution, CheckStop> {
        // [TYPE-5] a generic nominal's construct writes that nominal's
        // arguments, and their absence or a wrong count is TYPE-5's own
        // violation, "at the complete `construct`".
        self.generic_substitution(node, parameters, caller, false, SemanticRule::Type5)
    }

    /// One argument list, read for two callee classes.
    ///
    /// [DIAG-1] selects the cited rule by the callee's class rather than by
    /// the kind of argument problem, and these two classes differ: a
    /// user-generic call cites FN-2, a generic nominal's construct cites
    /// TYPE-5. The rule therefore arrives from the caller that knows its own
    /// class, instead of being chosen here from the shape of the failure.
    fn generic_substitution(
        &self,
        node: NodeId,
        parameters: &[GenericParameter],
        caller: &GenericSubstitution,
        allow_trailing_regions: bool,
        argument_rule: SemanticRule,
    ) -> Result<GenericSubstitution, CheckStop> {
        if parameters.is_empty() {
            if !allow_trailing_regions
                && self
                    .tree
                    .first_child_with(node, Production::Targs)?
                    .is_some()
            {
                return self.issue_node(argument_rule, node, SemanticIssueKind::TypeMismatch);
            }
            return Ok(GenericSubstitution::default());
        }
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return self.issue_node(argument_rule, node, SemanticIssueKind::TypeMismatch);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        if (allow_trailing_regions && arguments.len() < parameters.len())
            || (!allow_trailing_regions && arguments.len() != parameters.len())
        {
            return self.issue_node(argument_rule, node, SemanticIssueKind::TypeMismatch);
        }
        for argument in arguments.iter().take(parameters.len()) {
            self.reject_region_bearing_generic_argument(*argument, caller)?;
        }
        let mut bindings = Vec::with_capacity(parameters.len());
        for (parameter, argument) in parameters.iter().copied().zip(arguments) {
            let value = match parameter {
                GenericParameter::Type { bound, .. } => {
                    let Some(ty) = self.tree.first_child_with(argument, Production::Type)? else {
                        return self.issue_node(
                            argument_rule,
                            argument,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    let ty = self.parse_type_with(ty, caller)?;
                    let satisfies_bound = match bound {
                        GenericBound::Unbounded => true,
                        GenericBound::Int => {
                            matches!(ty, CheckedType::Integer(_) | CheckedType::GenericInt(_))
                        }
                        GenericBound::Float => {
                            matches!(ty, CheckedType::Float(_) | CheckedType::GenericFloat(_))
                        }
                    };
                    if !satisfies_bound {
                        return self.issue_node(
                            SemanticRule::Fn3,
                            argument,
                            SemanticIssueKind::TypeMismatch,
                        );
                    }
                    GenericArgument::Type(ty)
                }
                GenericParameter::Const { .. } => {
                    let Some(value) = self.tree.first_child_with(argument, Production::Const)?
                    else {
                        return self.issue_node(
                            argument_rule,
                            argument,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    GenericArgument::Const(self.parse_const_expression_with(value, caller)?)
                }
            };
            bindings.push((parameter.declaration(), value));
        }
        GenericSubstitution::from_bindings(bindings).map_err(CheckStop::Compiler)
    }
}

fn goal_uses_nominal_prefix(expression: &GoalExpression, checkpoint: usize) -> bool {
    match expression {
        GoalExpression::Datum(datum) => goal_datum_uses_nominal_prefix(datum, checkpoint),
        GoalExpression::Operation {
            row,
            type_arguments,
            result,
            arguments,
            ..
        } => {
            type_arguments
                .iter()
                .copied()
                .all(|ty| type_uses_nominal_prefix(ty, checkpoint))
                && type_uses_nominal_prefix(*result, checkpoint)
                && goal_operation_uses_nominal_prefix(*row, checkpoint)
                && arguments
                    .iter()
                    .all(|argument| goal_uses_nominal_prefix(argument, checkpoint))
        }
    }
}

fn goal_datum_uses_nominal_prefix(datum: &GoalDatum, checkpoint: usize) -> bool {
    match datum {
        GoalDatum::Parameter { ty, .. }
        | GoalDatum::NamedConst { ty, .. }
        | GoalDatum::Place { ty, .. } => type_uses_nominal_prefix(*ty, checkpoint),
        GoalDatum::EphemeralActual {
            captured_type, ty, ..
        } => {
            type_uses_nominal_prefix(*captured_type, checkpoint)
                && type_uses_nominal_prefix(*ty, checkpoint)
        }
        GoalDatum::Literal(value) => value_uses_nominal_prefix(value, checkpoint),
    }
}

fn goal_operation_uses_nominal_prefix(row: GoalOperation, checkpoint: usize) -> bool {
    match row {
        GoalOperation::Integer { operand_type, .. }
        | GoalOperation::Float { operand_type, .. }
        | GoalOperation::EnumEquality { operand_type, .. } => {
            type_uses_nominal_prefix(operand_type, checkpoint)
        }
        GoalOperation::ArrayFill { element, .. }
        | GoalOperation::ArrayLength { element, .. }
        | GoalOperation::BufferLength { element }
        | GoalOperation::BufferFits { element, .. }
        | GoalOperation::SliceLength { element, .. } => {
            flat_element_uses_nominal_prefix(element, checkpoint)
        }
        GoalOperation::NumericConversion { .. }
        | GoalOperation::Reinterpret { .. }
        | GoalOperation::Boolean(_) => true,
    }
}

fn value_uses_nominal_prefix(value: &CheckedValue, checkpoint: usize) -> bool {
    match value {
        CheckedValue::NumericIdentity { ty, .. } => type_uses_nominal_prefix(*ty, checkpoint),
        CheckedValue::Array { ty, elements } => {
            type_uses_nominal_prefix(*ty, checkpoint)
                && elements
                    .iter()
                    .all(|element| value_uses_nominal_prefix(element, checkpoint))
        }
        CheckedValue::Struct { ty, fields } => {
            type_uses_nominal_prefix(*ty, checkpoint)
                && fields
                    .iter()
                    .all(|field| value_uses_nominal_prefix(field, checkpoint))
        }
        CheckedValue::Unit
        | CheckedValue::Bool(_)
        | CheckedValue::Integer { .. }
        | CheckedValue::Float { .. } => true,
    }
}

fn type_uses_nominal_prefix(ty: CheckedType, checkpoint: usize) -> bool {
    match ty {
        CheckedType::Nominal(id) => (id.0 as usize) < checkpoint,
        CheckedType::Array { element, .. }
        | CheckedType::Slice { element, .. }
        | CheckedType::Buffer { element } => flat_element_uses_nominal_prefix(element, checkpoint),
        CheckedType::Unit
        | CheckedType::Bool
        | CheckedType::Integer(_)
        | CheckedType::Float(_)
        | CheckedType::Generic(_)
        | CheckedType::GenericInt(_)
        | CheckedType::GenericFloat(_) => true,
    }
}

fn flat_element_uses_nominal_prefix(element: CheckedFlatElement, checkpoint: usize) -> bool {
    match element {
        CheckedFlatElement::TagOnlyNominal(id) | CheckedFlatElement::Nominal(id) => {
            (id.0 as usize) < checkpoint
        }
        CheckedFlatElement::Unit
        | CheckedFlatElement::Bool
        | CheckedFlatElement::Integer(_)
        | CheckedFlatElement::Float(_)
        | CheckedFlatElement::GenericInt(_)
        | CheckedFlatElement::GenericFloat(_) => true,
    }
}
