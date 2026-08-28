use std::collections::{HashMap, HashSet};

use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, FixedTerminal, LexicalUseRole,
    PreludeDeclarationId, Production, ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind,
    SemanticRule, UnsupportedSemanticFeature,
};

use super::super::goal::{CheckedRequirement, GoalDatum, GoalExpression, GoalOperation};
use super::super::model::{
    CheckedConst, CheckedFlatElement, CheckedGenericRequirement, CheckedNominalKind, CheckedType,
    CheckedValue, FloatType, IntegerType, NominalId,
};
use super::{
    CheckStop, Checker, FunctionSignature, FunctionTemplate, PreludeType,
    derive_slice_return_ceiling,
};

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

/// Nominal-arena-independent identity for a concrete substitution discovered
/// while replaying generic source bodies. Replay intentionally runs in a
/// scratch nominal suffix; only this structural form crosses its rollback.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct StableGenericSubstitution {
    bindings: Vec<(DeclarationId, StableGenericArgument)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum StableGenericArgument {
    Type(StableCheckedType),
    Const(CheckedConst),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum StableCheckedType {
    Scalar(CheckedType),
    SourceNominal {
        template: usize,
        substitution: StableGenericSubstitution,
    },
    Prelude(StablePreludeType),
    Boxed(Box<StableCheckedType>),
    Arena {
        region: DeclarationId,
        content: Box<StableCheckedType>,
    },
    System(u8),
    Array {
        element: StableFlatElement,
        length: CheckedConst,
    },
    Slice {
        region: DeclarationId,
        element: StableFlatElement,
    },
    Buffer {
        element: StableFlatElement,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum StableFlatElement {
    Unit,
    Bool,
    Integer(IntegerType),
    Float(FloatType),
    GenericInt(DeclarationId),
    GenericFloat(DeclarationId),
    TagOnlyNominal(Box<StableCheckedType>),
    Nominal(Box<StableCheckedType>),
}

/// One symbolic generic requirement while its scratch nominal suffix is
/// rolled back. The checked predicate remains exact, but every scratch
/// nominal it mentions has a structural bridge that can be re-interned only
/// after the executable nominal prefix is closed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PendingGenericRequirement {
    declaration: DeclarationId,
    requirement: CheckedRequirement,
    nominal_checkpoint: usize,
    replacements: Vec<(NominalId, StableCheckedType)>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum StablePreludeType {
    Option(Box<StableCheckedType>),
    Result(Box<StableCheckedType>, Box<StableCheckedType>),
    Overflow,
    DivError,
    NarrowError,
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
        let prepared = self.ensure_nominals_in_function_signature(template.node, &substitution);
        let signature = match prepared
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
        let mut header_nodes = self.tree.children_with(function, Production::ParamList)?;
        let result_binding = self
            .tree
            .first_child_with(function, Production::ResultBinding)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        header_nodes.push(result_binding);
        for node in header_nodes {
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
        let result_binding = self
            .tree
            .first_child_with(template.node, Production::ResultBinding)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let rtype = self
            .tree
            .first_child_with(result_binding, Production::Rtype)?
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
        let declared_effects = self.parse_effects(effects, &parameters)?;
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

    pub(super) fn validate_generic_templates(&mut self) -> Result<(), CheckStop> {
        if !self.pending_generic_requirements.is_empty()
            || !self.generic_requirements.is_empty()
            || !self.generic_claim_schemas.is_empty()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        // A closed unit with no generic function declaration has no source
        // schema to validate.  Rechecking every concrete function through a
        // temporary symbolic inventory would duplicate the ordinary ENT and
        // provenance baseline without producing a schema report, and claim
        // residuality would then pay that whole-program cost once more for
        // every mask.
        if self
            .function_templates
            .iter()
            .all(|template| template.generic_parameters.is_empty())
        {
            return Ok(());
        }
        let concrete_signatures = std::mem::take(&mut self.signatures);
        let concrete_functions_by_declaration = std::mem::take(&mut self.functions_by_declaration);
        let concrete_postcondition_selectors = std::mem::take(&mut self.postcondition_selectors);
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
        // Concrete selectors are keyed by the dense FunctionId inventory.
        // Schema validation uses a separate scratch inventory starting at
        // zero, so it must build and later discard its own selector table
        // rather than aliasing the real concrete entries by accident.
        self.admit_postcondition_selectors()?;
        self.derive_result_state_origins()?;
        let mut phase_a = Vec::with_capacity(self.signatures.len());
        for index in 0..self.signatures.len() {
            // Symbolic generic validation may discover a derived box or
            // prelude nominal (for example the Result produced by a
            // `+checked` requires-local). Use the same deferred-nominal
            // retry loop as concrete checking; the checkpoint below
            // discards these symbolic-only instances afterwards. The dense
            // inventory also includes nongeneric callees so FN-8 requirement
            // installation uses the ordinary FunctionId-indexed path.
            phase_a.push(self.check_function_interning_nominals(index)?);
        }
        for (canonical, declaration) in &canonical_generic_signatures {
            let checked = phase_a
                .get(*canonical)
                .filter(|checked| checked.function.declaration == *declaration)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            for requirement in &checked.function.requirements {
                self.pending_generic_requirements
                    .push(self.stabilize_generic_requirement(
                        *declaration,
                        requirement,
                        nominal_checkpoint,
                    )?);
            }
        }
        self.install_call_requirements(&mut phase_a)?;
        let callees = self.entailment_callees()?;
        self.evaluate_generic_claim_schemas(&phase_a, &canonical_generic_signatures, &callees)?;
        self.signatures.clear();
        self.functions_by_declaration.clear();
        self.postcondition_selectors.clear();
        self.restore_nominal_checkpoint(nominal_checkpoint)?;
        self.signatures = concrete_signatures;
        self.functions_by_declaration = concrete_functions_by_declaration;
        self.postcondition_selectors = concrete_postcondition_selectors;
        let replayed_concrete = self.discover_schema_written_concrete_instances()?;
        // The replay above can append a concrete instance that is mentioned
        // only inside an uninstantiated generic body. Rebuild the selector
        // table over the final concrete inventory so those instances receive
        // the same FN-9 judgment as directly discovered instances.
        self.postcondition_selectors.clear();
        self.admit_postcondition_selectors_including(&replayed_concrete)?;
        Ok(())
    }

    /// Replays the generic source-call graph after the symbolic nominal
    /// checkpoint and retains every explicitly concrete substitution it
    /// contains. The replay carries only source template indices and freshly
    /// reconstructed substitutions, so a concrete nominal argument never
    /// leaks a scratch `NominalId` from schema validation into the executable
    /// inventory.
    fn discover_schema_written_concrete_instances(
        &mut self,
    ) -> Result<Vec<super::super::model::FunctionId>, CheckStop> {
        if !self.pending_nominals.borrow().is_empty() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let nominal_checkpoint = self.nominal_checkpoint();
        let discovered = (|| {
            let mut work = Vec::new();
            let mut candidates = Vec::new();
            for template_index in 0..self.function_templates.len() {
                let template = self
                    .function_templates
                    .get(template_index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if template.generic_parameters.is_empty() {
                    continue;
                }
                work.push((
                    template_index,
                    self.symbolic_generic_substitution(&template.generic_parameters)?,
                ));
            }
            let mut cursor = 0_usize;
            while cursor < work.len() {
                let (caller_template_index, caller_substitution) = work
                    .get(cursor)
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let caller = self
                    .function_templates
                    .get(caller_template_index)
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                for call in self.tree.descendants_with(caller.node, Production::Call)? {
                    if self.call_is_inside_postcondition(call)? {
                        continue;
                    }
                    let Some((callee_template_index, callee)) =
                        self.called_function_template(call)?
                    else {
                        continue;
                    };
                    if callee.generic_parameters.is_empty() {
                        continue;
                    }
                    if let Some(targs) = self.tree.first_child_with(call, Production::Targs)? {
                        self.ensure_nominals_in_node(targs, &caller_substitution)?;
                    }
                    let substitution =
                        self.call_generic_substitution(call, &callee, &caller_substitution)?;
                    if !work
                        .iter()
                        .any(|(candidate_template, candidate_substitution)| {
                            *candidate_template == callee_template_index
                                && candidate_substitution == &substitution
                        })
                    {
                        work.push((callee_template_index, substitution.clone()));
                    }
                    if let Some(stable) =
                        self.stabilize_concrete_substitution(&substitution, nominal_checkpoint)?
                    {
                        candidates.push((callee_template_index, callee.declaration, stable));
                    }
                }
                cursor = cursor
                    .checked_add(1)
                    .ok_or(SemanticCompilerFailure::CounterOverflow)?;
            }
            Ok::<_, CheckStop>(candidates)
        })();
        self.restore_nominal_checkpoint(nominal_checkpoint)?;
        if !self.pending_nominals.borrow().is_empty() {
            self.pending_nominals.borrow_mut().clear();
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let mut discovered = discovered?;
        discovered.sort_by(|left, right| {
            left.1
                .cmp(&right.1)
                .then_with(|| format!("{:?}", left.2).cmp(&format!("{:?}", right.2)))
        });
        discovered.dedup_by(|left, right| left.0 == right.0 && left.2 == right.2);

        let mut replayed = Vec::new();
        for (template_index, declaration, stable) in discovered {
            let substitution = self.reify_concrete_substitution(&stable)?;
            let already_present = self
                .functions_by_declaration
                .get(&declaration)
                .into_iter()
                .flatten()
                .any(|id| {
                    self.signatures
                        .get(id.0 as usize)
                        .is_some_and(|signature| signature.substitution == substitution)
                });
            if already_present {
                continue;
            }
            let id = super::super::model::FunctionId(
                u32::try_from(self.signatures.len())
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            self.instantiate_function_signature(template_index, substitution)?;
            replayed.push(id);
        }
        Ok(replayed)
    }

    fn stabilize_concrete_substitution(
        &self,
        substitution: &GenericSubstitution,
        nominal_checkpoint: usize,
    ) -> Result<Option<StableGenericSubstitution>, CheckStop> {
        let mut visiting = HashSet::new();
        let mut bindings = Vec::with_capacity(substitution.bindings.len());
        for (declaration, argument) in &substitution.bindings {
            let stable = match argument {
                GenericArgument::Type(ty) => {
                    let Some(ty) =
                        self.stabilize_concrete_type(*ty, nominal_checkpoint, &mut visiting)?
                    else {
                        return Ok(None);
                    };
                    StableGenericArgument::Type(ty)
                }
                GenericArgument::Const(value) => {
                    let Some(value) = value.value() else {
                        return Ok(None);
                    };
                    StableGenericArgument::Const(CheckedConst::Value(value))
                }
            };
            bindings.push((*declaration, stable));
        }
        Ok(Some(StableGenericSubstitution { bindings }))
    }

    fn stabilize_concrete_type(
        &self,
        ty: CheckedType,
        nominal_checkpoint: usize,
        visiting: &mut HashSet<NominalId>,
    ) -> Result<Option<StableCheckedType>, CheckStop> {
        self.stabilize_type(ty, nominal_checkpoint, visiting, false)
    }

    fn stabilize_schema_type(
        &self,
        ty: CheckedType,
        nominal_checkpoint: usize,
        visiting: &mut HashSet<NominalId>,
    ) -> Result<StableCheckedType, CheckStop> {
        self.stabilize_type(ty, nominal_checkpoint, visiting, true)?
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    fn stabilize_type(
        &self,
        ty: CheckedType,
        nominal_checkpoint: usize,
        visiting: &mut HashSet<NominalId>,
        allow_symbolic: bool,
    ) -> Result<Option<StableCheckedType>, CheckStop> {
        let stable = match ty {
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_) => StableCheckedType::Scalar(ty),
            CheckedType::Generic(_) | CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => {
                if !allow_symbolic {
                    return Ok(None);
                }
                StableCheckedType::Scalar(ty)
            }
            CheckedType::Nominal(id) => {
                if !visiting.insert(id) {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
                let source = self
                    .source_nominal_instances
                    .get(id.0 as usize)
                    .cloned()
                    .flatten();
                let prelude = self.prelude_types.get(id.0 as usize).cloned().flatten();
                let system = self
                    .system_nominals
                    .iter()
                    .find_map(|(index, candidate)| (*candidate == id).then_some(*index));
                let kind = self
                    .nominals
                    .get(id.0 as usize)
                    .map(|nominal| nominal.kind.clone())
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let stable = if let Some((template, substitution)) = source {
                    let Some(substitution) = self.stabilize_substitution_with_visiting(
                        &substitution,
                        nominal_checkpoint,
                        visiting,
                        allow_symbolic,
                    )?
                    else {
                        visiting.remove(&id);
                        return Ok(None);
                    };
                    StableCheckedType::SourceNominal {
                        template,
                        substitution,
                    }
                } else if let Some(prelude) = prelude {
                    let Some(prelude) = self.stabilize_prelude_type(
                        prelude,
                        nominal_checkpoint,
                        visiting,
                        allow_symbolic,
                    )?
                    else {
                        visiting.remove(&id);
                        return Ok(None);
                    };
                    StableCheckedType::Prelude(prelude)
                } else if let Some(system) = system {
                    StableCheckedType::System(system)
                } else {
                    match kind {
                        CheckedNominalKind::Box { referent } => {
                            let Some(referent) = self.stabilize_type(
                                referent,
                                nominal_checkpoint,
                                visiting,
                                allow_symbolic,
                            )?
                            else {
                                visiting.remove(&id);
                                return Ok(None);
                            };
                            StableCheckedType::Boxed(Box::new(referent))
                        }
                        CheckedNominalKind::Arena { region, content } => {
                            let Some(content) = self.stabilize_type(
                                content,
                                nominal_checkpoint,
                                visiting,
                                allow_symbolic,
                            )?
                            else {
                                visiting.remove(&id);
                                return Ok(None);
                            };
                            StableCheckedType::Arena {
                                region,
                                content: Box::new(content),
                            }
                        }
                        CheckedNominalKind::SystemResource { .. } => {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                        CheckedNominalKind::Struct { .. }
                        | CheckedNominalKind::Enum { .. }
                        | CheckedNominalKind::ArenaStorage => {
                            return Err(SemanticCompilerFailure::InvalidResolution.into());
                        }
                    }
                };
                visiting.remove(&id);
                stable
            }
            CheckedType::Array { element, length } => {
                let Some(element) = self.stabilize_flat_element(
                    element,
                    nominal_checkpoint,
                    visiting,
                    allow_symbolic,
                )?
                else {
                    return Ok(None);
                };
                if !allow_symbolic && !length.is_concrete() {
                    return Ok(None);
                }
                StableCheckedType::Array { element, length }
            }
            CheckedType::Slice { region, element } => {
                let Some(element) = self.stabilize_flat_element(
                    element,
                    nominal_checkpoint,
                    visiting,
                    allow_symbolic,
                )?
                else {
                    return Ok(None);
                };
                StableCheckedType::Slice { region, element }
            }
            CheckedType::Buffer { element } => {
                let Some(element) = self.stabilize_flat_element(
                    element,
                    nominal_checkpoint,
                    visiting,
                    allow_symbolic,
                )?
                else {
                    return Ok(None);
                };
                StableCheckedType::Buffer { element }
            }
        };
        Ok(Some(stable))
    }

    fn stabilize_substitution_with_visiting(
        &self,
        substitution: &GenericSubstitution,
        nominal_checkpoint: usize,
        visiting: &mut HashSet<NominalId>,
        allow_symbolic: bool,
    ) -> Result<Option<StableGenericSubstitution>, CheckStop> {
        let mut bindings = Vec::with_capacity(substitution.bindings.len());
        for (declaration, argument) in &substitution.bindings {
            let stable = match argument {
                GenericArgument::Type(ty) => {
                    let Some(ty) =
                        self.stabilize_type(*ty, nominal_checkpoint, visiting, allow_symbolic)?
                    else {
                        return Ok(None);
                    };
                    StableGenericArgument::Type(ty)
                }
                GenericArgument::Const(value) => {
                    if !allow_symbolic && !value.is_concrete() {
                        return Ok(None);
                    }
                    StableGenericArgument::Const(*value)
                }
            };
            bindings.push((*declaration, stable));
        }
        Ok(Some(StableGenericSubstitution { bindings }))
    }

    fn stabilize_flat_element(
        &self,
        element: CheckedFlatElement,
        nominal_checkpoint: usize,
        visiting: &mut HashSet<NominalId>,
        allow_symbolic: bool,
    ) -> Result<Option<StableFlatElement>, CheckStop> {
        Ok(match element {
            CheckedFlatElement::Unit => Some(StableFlatElement::Unit),
            CheckedFlatElement::Bool => Some(StableFlatElement::Bool),
            CheckedFlatElement::Integer(ty) => Some(StableFlatElement::Integer(ty)),
            CheckedFlatElement::Float(ty) => Some(StableFlatElement::Float(ty)),
            CheckedFlatElement::GenericInt(declaration) => {
                allow_symbolic.then_some(StableFlatElement::GenericInt(declaration))
            }
            CheckedFlatElement::GenericFloat(declaration) => {
                allow_symbolic.then_some(StableFlatElement::GenericFloat(declaration))
            }
            CheckedFlatElement::TagOnlyNominal(id) => self
                .stabilize_type(
                    CheckedType::Nominal(id),
                    nominal_checkpoint,
                    visiting,
                    allow_symbolic,
                )?
                .map(|ty| StableFlatElement::TagOnlyNominal(Box::new(ty))),
            CheckedFlatElement::Nominal(id) => self
                .stabilize_type(
                    CheckedType::Nominal(id),
                    nominal_checkpoint,
                    visiting,
                    allow_symbolic,
                )?
                .map(|ty| StableFlatElement::Nominal(Box::new(ty))),
        })
    }

    fn stabilize_prelude_type(
        &self,
        ty: PreludeType,
        nominal_checkpoint: usize,
        visiting: &mut HashSet<NominalId>,
        allow_symbolic: bool,
    ) -> Result<Option<StablePreludeType>, CheckStop> {
        Ok(match ty {
            PreludeType::Option(value) => self
                .stabilize_type(value, nominal_checkpoint, visiting, allow_symbolic)?
                .map(|value| StablePreludeType::Option(Box::new(value))),
            PreludeType::Result(ok, error) => {
                let Some(ok) =
                    self.stabilize_type(ok, nominal_checkpoint, visiting, allow_symbolic)?
                else {
                    return Ok(None);
                };
                let Some(error) =
                    self.stabilize_type(error, nominal_checkpoint, visiting, allow_symbolic)?
                else {
                    return Ok(None);
                };
                Some(StablePreludeType::Result(Box::new(ok), Box::new(error)))
            }
            PreludeType::Overflow => Some(StablePreludeType::Overflow),
            PreludeType::DivError => Some(StablePreludeType::DivError),
            PreludeType::NarrowError => Some(StablePreludeType::NarrowError),
        })
    }

    fn reify_concrete_substitution(
        &mut self,
        substitution: &StableGenericSubstitution,
    ) -> Result<GenericSubstitution, CheckStop> {
        let mut bindings = Vec::with_capacity(substitution.bindings.len());
        for (declaration, argument) in &substitution.bindings {
            let argument = match argument {
                StableGenericArgument::Type(ty) => {
                    GenericArgument::Type(self.reify_concrete_type(ty)?)
                }
                StableGenericArgument::Const(value) => GenericArgument::Const(*value),
            };
            bindings.push((*declaration, argument));
        }
        GenericSubstitution::from_bindings(bindings).map_err(CheckStop::Compiler)
    }

    fn reify_concrete_type(&mut self, ty: &StableCheckedType) -> Result<CheckedType, CheckStop> {
        Ok(match ty {
            StableCheckedType::Scalar(ty) => *ty,
            StableCheckedType::SourceNominal {
                template,
                substitution,
            } => {
                let substitution = self.reify_concrete_substitution(substitution)?;
                CheckedType::Nominal(self.ensure_source_nominal_instance(*template, substitution)?)
            }
            StableCheckedType::Prelude(ty) => {
                let ty = match ty {
                    StablePreludeType::Option(value) => {
                        PreludeType::Option(self.reify_concrete_type(value)?)
                    }
                    StablePreludeType::Result(ok, error) => PreludeType::Result(
                        self.reify_concrete_type(ok)?,
                        self.reify_concrete_type(error)?,
                    ),
                    StablePreludeType::Overflow => PreludeType::Overflow,
                    StablePreludeType::DivError => PreludeType::DivError,
                    StablePreludeType::NarrowError => PreludeType::NarrowError,
                };
                CheckedType::Nominal(self.intern_prelude_nominal(ty)?)
            }
            StableCheckedType::Boxed(referent) => {
                let referent = self.reify_concrete_type(referent)?;
                CheckedType::Nominal(self.intern_box_nominal(referent)?)
            }
            StableCheckedType::Arena { region, content } => {
                let content = self.reify_concrete_type(content)?;
                CheckedType::Nominal(self.intern_arena_nominal(*region, content)?)
            }
            StableCheckedType::System(index) => {
                CheckedType::Nominal(self.intern_system_nominal(*index)?)
            }
            StableCheckedType::Array { element, length } => CheckedType::Array {
                element: self.reify_flat_element(element)?,
                length: *length,
            },
            StableCheckedType::Slice { region, element } => CheckedType::Slice {
                region: *region,
                element: self.reify_flat_element(element)?,
            },
            StableCheckedType::Buffer { element } => CheckedType::Buffer {
                element: self.reify_flat_element(element)?,
            },
        })
    }

    fn reify_flat_element(
        &mut self,
        element: &StableFlatElement,
    ) -> Result<CheckedFlatElement, CheckStop> {
        Ok(match element {
            StableFlatElement::Unit => CheckedFlatElement::Unit,
            StableFlatElement::Bool => CheckedFlatElement::Bool,
            StableFlatElement::Integer(ty) => CheckedFlatElement::Integer(*ty),
            StableFlatElement::Float(ty) => CheckedFlatElement::Float(*ty),
            StableFlatElement::GenericInt(declaration) => {
                CheckedFlatElement::GenericInt(*declaration)
            }
            StableFlatElement::GenericFloat(declaration) => {
                CheckedFlatElement::GenericFloat(*declaration)
            }
            StableFlatElement::TagOnlyNominal(ty) => {
                let CheckedType::Nominal(id) = self.reify_concrete_type(ty)? else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                CheckedFlatElement::TagOnlyNominal(id)
            }
            StableFlatElement::Nominal(ty) => {
                let CheckedType::Nominal(id) = self.reify_concrete_type(ty)? else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                CheckedFlatElement::Nominal(id)
            }
        })
    }

    fn stabilize_generic_requirement(
        &self,
        declaration: DeclarationId,
        requirement: &CheckedRequirement,
        nominal_checkpoint: usize,
    ) -> Result<PendingGenericRequirement, CheckStop> {
        let mut nominals = Vec::new();
        collect_goal_nominals(&requirement.template.root, &mut nominals);
        nominals.sort_by_key(|id| id.0);
        nominals.dedup();

        let mut replacements = Vec::new();
        for nominal in nominals {
            if (nominal.0 as usize) < nominal_checkpoint {
                continue;
            }
            let stable = self.stabilize_schema_type(
                CheckedType::Nominal(nominal),
                nominal_checkpoint,
                &mut HashSet::new(),
            )?;
            replacements.push((nominal, stable));
        }
        Ok(PendingGenericRequirement {
            declaration,
            requirement: requirement.clone(),
            nominal_checkpoint,
            replacements,
        })
    }

    /// Re-interns metadata-only symbolic nominals after the executable prefix
    /// has already been measured. No scratch `NominalId` crosses the schema
    /// checkpoint, and lowering continues to see one contiguous concrete
    /// prefix.
    pub(super) fn materialize_generic_requirements(&mut self) -> Result<(), CheckStop> {
        if !self.generic_requirements.is_empty() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let pending = std::mem::take(&mut self.pending_generic_requirements);
        for mut pending in pending {
            let mut replacements = HashMap::new();
            for (old, stable) in &pending.replacements {
                let CheckedType::Nominal(new) = self.reify_concrete_type(stable)? else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                if replacements.insert(*old, new).is_some() {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                }
            }
            rewrite_goal_nominals(
                &mut pending.requirement.template.root,
                pending.nominal_checkpoint,
                &replacements,
            )?;
            self.generic_requirements.push(CheckedGenericRequirement {
                declaration: pending.declaration,
                requirement: pending.requirement,
            });
        }
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
                return self.issue_node(argument_rule, node, SemanticIssueKind::type_mismatch("no type arguments, because this form declares no generic parameters", "a written `<...>` type-argument list"));
            }
            return Ok(GenericSubstitution::default());
        }
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return self.issue_node(argument_rule, node, SemanticIssueKind::type_mismatch(crate::semantic::written_count(parameters.len(), "type argument"), "no type-argument list"));
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        if (allow_trailing_regions && arguments.len() < parameters.len())
            || (!allow_trailing_regions && arguments.len() != parameters.len())
        {
            return self.issue_node(argument_rule, node, SemanticIssueKind::type_mismatch(crate::semantic::written_count(parameters.len(), "type argument"), crate::semantic::written_count(arguments.len(), "type argument")));
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
                            SemanticIssueKind::type_mismatch("a type in this type-argument position", "a const argument in a type-parameter position"),
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
                            SemanticIssueKind::type_mismatch(
                            match bound {
                                GenericBound::Unbounded => "any type",
                                GenericBound::Int => {
                                    "an integer type, which the parameter's `Int` bound requires"
                                }
                                GenericBound::Float => {
                                    "a float type, which the parameter's `Float` bound requires"
                                }
                            },
                            self.checked_type_name(ty)?,
                        ),
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
                            SemanticIssueKind::type_mismatch("a const argument in this type-argument position", "a type in a const-parameter position"),
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

fn collect_goal_nominals(expression: &GoalExpression, output: &mut Vec<NominalId>) {
    match expression {
        GoalExpression::Datum(datum) => match datum {
            GoalDatum::Parameter { ty, .. }
            | GoalDatum::NamedConst { ty, .. }
            | GoalDatum::Place { ty, .. } => collect_type_nominals(*ty, output),
            GoalDatum::EphemeralActual {
                captured_type, ty, ..
            } => {
                collect_type_nominals(*captured_type, output);
                collect_type_nominals(*ty, output);
            }
            GoalDatum::Literal(value) => collect_value_nominals(value, output),
        },
        GoalExpression::Operation {
            row,
            type_arguments,
            result,
            arguments,
            ..
        } => {
            collect_operation_nominals(*row, output);
            for ty in type_arguments {
                collect_type_nominals(*ty, output);
            }
            collect_type_nominals(*result, output);
            for argument in arguments {
                collect_goal_nominals(argument, output);
            }
        }
    }
}

fn collect_operation_nominals(operation: GoalOperation, output: &mut Vec<NominalId>) {
    match operation {
        GoalOperation::Integer { operand_type, .. }
        | GoalOperation::Float { operand_type, .. }
        | GoalOperation::EnumEquality { operand_type, .. }
        | GoalOperation::BufferFits {
            element: operand_type,
            ..
        } => collect_type_nominals(operand_type, output),
        GoalOperation::ArrayFill { element, .. }
        | GoalOperation::ArrayLength { element, .. }
        | GoalOperation::BufferLength { element }
        | GoalOperation::SliceLength { element, .. } => {
            collect_flat_element_nominals(element, output);
        }
        GoalOperation::NumericConversion { .. }
        | GoalOperation::Reinterpret { .. }
        | GoalOperation::Boolean(_) => {}
    }
}

fn collect_type_nominals(ty: CheckedType, output: &mut Vec<NominalId>) {
    match ty {
        CheckedType::Nominal(id) => output.push(id),
        CheckedType::Array { element, .. }
        | CheckedType::Slice { element, .. }
        | CheckedType::Buffer { element } => collect_flat_element_nominals(element, output),
        CheckedType::Unit
        | CheckedType::Bool
        | CheckedType::Integer(_)
        | CheckedType::Float(_)
        | CheckedType::Generic(_)
        | CheckedType::GenericInt(_)
        | CheckedType::GenericFloat(_) => {}
    }
}

fn collect_flat_element_nominals(element: CheckedFlatElement, output: &mut Vec<NominalId>) {
    match element {
        CheckedFlatElement::TagOnlyNominal(id) | CheckedFlatElement::Nominal(id) => output.push(id),
        CheckedFlatElement::Unit
        | CheckedFlatElement::Bool
        | CheckedFlatElement::Integer(_)
        | CheckedFlatElement::Float(_)
        | CheckedFlatElement::GenericInt(_)
        | CheckedFlatElement::GenericFloat(_) => {}
    }
}

fn collect_value_nominals(value: &CheckedValue, output: &mut Vec<NominalId>) {
    match value {
        CheckedValue::NumericIdentity { ty, .. } => collect_type_nominals(*ty, output),
        CheckedValue::Array { ty, elements } => {
            collect_type_nominals(*ty, output);
            for element in elements {
                collect_value_nominals(element, output);
            }
        }
        CheckedValue::Struct { ty, fields } => {
            collect_type_nominals(*ty, output);
            for field in fields {
                collect_value_nominals(field, output);
            }
        }
        CheckedValue::Unit
        | CheckedValue::Bool(_)
        | CheckedValue::Integer { .. }
        | CheckedValue::Float { .. } => {}
    }
}

fn rewrite_goal_nominals(
    expression: &mut GoalExpression,
    checkpoint: usize,
    replacements: &HashMap<NominalId, NominalId>,
) -> Result<(), CheckStop> {
    match expression {
        GoalExpression::Datum(datum) => match datum {
            GoalDatum::Parameter { ty, .. }
            | GoalDatum::NamedConst { ty, .. }
            | GoalDatum::Place { ty, .. } => rewrite_type_nominals(ty, checkpoint, replacements)?,
            GoalDatum::EphemeralActual {
                captured_type, ty, ..
            } => {
                rewrite_type_nominals(captured_type, checkpoint, replacements)?;
                rewrite_type_nominals(ty, checkpoint, replacements)?;
            }
            GoalDatum::Literal(value) => rewrite_value_nominals(value, checkpoint, replacements)?,
        },
        GoalExpression::Operation {
            row,
            type_arguments,
            result,
            arguments,
            ..
        } => {
            rewrite_operation_nominals(row, checkpoint, replacements)?;
            for ty in type_arguments {
                rewrite_type_nominals(ty, checkpoint, replacements)?;
            }
            rewrite_type_nominals(result, checkpoint, replacements)?;
            for argument in arguments {
                rewrite_goal_nominals(argument, checkpoint, replacements)?;
            }
        }
    }
    Ok(())
}

fn rewrite_operation_nominals(
    operation: &mut GoalOperation,
    checkpoint: usize,
    replacements: &HashMap<NominalId, NominalId>,
) -> Result<(), CheckStop> {
    match operation {
        GoalOperation::Integer { operand_type, .. }
        | GoalOperation::Float { operand_type, .. }
        | GoalOperation::EnumEquality { operand_type, .. }
        | GoalOperation::BufferFits {
            element: operand_type,
            ..
        } => rewrite_type_nominals(operand_type, checkpoint, replacements)?,
        GoalOperation::ArrayFill { element, .. }
        | GoalOperation::ArrayLength { element, .. }
        | GoalOperation::BufferLength { element }
        | GoalOperation::SliceLength { element, .. } => {
            rewrite_flat_element_nominals(element, checkpoint, replacements)?;
        }
        GoalOperation::NumericConversion { .. }
        | GoalOperation::Reinterpret { .. }
        | GoalOperation::Boolean(_) => {}
    }
    Ok(())
}

fn rewrite_type_nominals(
    ty: &mut CheckedType,
    checkpoint: usize,
    replacements: &HashMap<NominalId, NominalId>,
) -> Result<(), CheckStop> {
    match ty {
        CheckedType::Nominal(id) if (id.0 as usize) >= checkpoint => {
            *id = *replacements
                .get(id)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        }
        CheckedType::Array { element, .. }
        | CheckedType::Slice { element, .. }
        | CheckedType::Buffer { element } => {
            rewrite_flat_element_nominals(element, checkpoint, replacements)?;
        }
        CheckedType::Unit
        | CheckedType::Bool
        | CheckedType::Integer(_)
        | CheckedType::Float(_)
        | CheckedType::Generic(_)
        | CheckedType::GenericInt(_)
        | CheckedType::GenericFloat(_)
        | CheckedType::Nominal(_) => {}
    }
    Ok(())
}

fn rewrite_flat_element_nominals(
    element: &mut CheckedFlatElement,
    checkpoint: usize,
    replacements: &HashMap<NominalId, NominalId>,
) -> Result<(), CheckStop> {
    match element {
        CheckedFlatElement::TagOnlyNominal(id) | CheckedFlatElement::Nominal(id)
            if (id.0 as usize) >= checkpoint =>
        {
            *id = *replacements
                .get(id)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        }
        CheckedFlatElement::Unit
        | CheckedFlatElement::Bool
        | CheckedFlatElement::Integer(_)
        | CheckedFlatElement::Float(_)
        | CheckedFlatElement::GenericInt(_)
        | CheckedFlatElement::GenericFloat(_)
        | CheckedFlatElement::TagOnlyNominal(_)
        | CheckedFlatElement::Nominal(_) => {}
    }
    Ok(())
}

fn rewrite_value_nominals(
    value: &mut CheckedValue,
    checkpoint: usize,
    replacements: &HashMap<NominalId, NominalId>,
) -> Result<(), CheckStop> {
    match value {
        CheckedValue::NumericIdentity { ty, .. } => {
            rewrite_type_nominals(ty, checkpoint, replacements)?;
        }
        CheckedValue::Array { ty, elements } => {
            rewrite_type_nominals(ty, checkpoint, replacements)?;
            for element in elements {
                rewrite_value_nominals(element, checkpoint, replacements)?;
            }
        }
        CheckedValue::Struct { ty, fields } => {
            rewrite_type_nominals(ty, checkpoint, replacements)?;
            for field in fields {
                rewrite_value_nominals(field, checkpoint, replacements)?;
            }
        }
        CheckedValue::Unit
        | CheckedValue::Bool(_)
        | CheckedValue::Integer { .. }
        | CheckedValue::Float { .. } => {}
    }
    Ok(())
}
