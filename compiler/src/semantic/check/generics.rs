use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, FixedTerminal, LexicalUseRole,
    PreludeDeclarationId, Production, ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind,
    SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::{CheckedConst, CheckedType};
use super::{CheckStop, Checker, FunctionSignature, FunctionTemplate};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum GenericBound {
    Unbounded,
    Int,
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

    pub(super) fn is_concrete(&self) -> bool {
        self.bindings.iter().all(|(_, argument)| match argument {
            GenericArgument::Type(ty) => ty.is_concrete(),
            GenericArgument::Const(value) => value.is_concrete(),
        })
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn collect_function_templates(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        for node in items.iter().copied().filter(|node| {
            self.tree
                .production(*node)
                .is_ok_and(|production| production == Production::FnDecl)
        }) {
            let declaration = self.declaration_at(node, DeclarationRole::Function)?;
            let template = FunctionTemplate {
                declaration: declaration.id(),
                node,
                name: declaration.spelling().to_owned(),
                generic_parameters: self.parse_generic_parameters(node)?,
            };
            if !template.generic_parameters.is_empty() {
                if let Some(regions) = self.tree.first_child_with(node, Production::RegionParams)? {
                    return self.unsupported(UnsupportedSemanticFeature::Generics, regions);
                }
                if let Some(requires) = self
                    .tree
                    .first_child_with(node, Production::RequiresBlock)?
                {
                    return self.unsupported(UnsupportedSemanticFeature::Generics, requires);
                }
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
        }
        self.reject_generic_call_cycles()?;
        self.validate_generic_templates()?;
        Ok(())
    }

    pub(super) fn collect_concrete_function_signatures(&mut self) -> Result<(), CheckStop> {
        for template_index in 0..self.function_templates.len() {
            if self.function_templates[template_index]
                .generic_parameters
                .is_empty()
            {
                self.instantiate_function_signature(
                    template_index,
                    GenericSubstitution::default(),
                )?;
            }
        }
        self.discover_called_function_signatures(true)
    }

    fn discover_called_function_signatures(
        &mut self,
        require_concrete: bool,
    ) -> Result<(), CheckStop> {
        let mut cursor = 0_usize;
        while cursor < self.signatures.len() {
            let signature = self.signatures[cursor].clone();
            for call in self
                .tree
                .descendants_with(signature.node, Production::Call)?
            {
                let Some((template_index, template)) = self.called_function_template(call)? else {
                    continue;
                };
                if template.generic_parameters.is_empty() {
                    continue;
                }
                let substitution =
                    self.call_generic_substitution(call, &template, &signature.substitution)?;
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
                    self.instantiate_function_signature(template_index, substitution)?;
                }
            }
            cursor = cursor
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        }
        Ok(())
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
            ResolvedTarget::Operation(_) => return Ok(None),
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let index = *self
            .templates_by_declaration
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
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
        self.ensure_nominals_in_node(template.node, &substitution)?;
        let signature = self.build_function_signature(&template, substitution, id)?;
        self.functions_by_declaration
            .entry(template.declaration)
            .or_default()
            .push(id);
        self.signatures.push(signature);
        Ok(())
    }

    fn build_function_signature(
        &self,
        template: &FunctionTemplate,
        substitution: GenericSubstitution,
        id: super::super::model::FunctionId,
    ) -> Result<FunctionSignature, CheckStop> {
        let region_parameters = self.parse_region_parameters(template.node)?;
        let parameters = self.parse_parameters_with(template.node, &substitution)?;
        let rtype = self
            .tree
            .first_child_with(template.node, Production::Rtype)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let (result_mode, result) = self.parse_rtype_with(rtype, &substitution)?;
        if result_mode != super::super::model::CheckedMode::Own {
            return self.unsupported(UnsupportedSemanticFeature::RegionsAndBorrows, rtype);
        }
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
            region_parameters,
            parameters,
            result_mode,
            result,
            effects_node: effects,
            declared_effects,
            substitution,
        })
    }

    fn validate_generic_templates(&mut self) -> Result<(), CheckStop> {
        if !self.signatures.is_empty() || !self.functions_by_declaration.is_empty() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        let nominal_checkpoint = self.nominal_checkpoint();
        for template_index in 0..self.function_templates.len() {
            let template = self
                .function_templates
                .get(template_index)
                .cloned()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let substitution = self.symbolic_generic_substitution(&template.generic_parameters)?;
            self.instantiate_function_signature(template_index, substitution)?;
        }
        self.discover_called_function_signatures(false)?;
        for index in 0..self.signatures.len() {
            if !self.signatures[index].substitution.bindings.is_empty() {
                self.check_function(index)?;
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
        let mut edges = vec![Vec::new(); self.function_templates.len()];
        for (caller, template) in self.function_templates.iter().enumerate() {
            for call in self
                .tree
                .descendants_with(template.node, Production::Call)?
            {
                let Some((callee, _)) = self.called_function_template(call)? else {
                    continue;
                };
                edges[caller].push((callee, call));
            }
        }
        for (caller, outgoing) in edges.iter().enumerate() {
            for (callee, call) in outgoing {
                if !Self::graph_reaches(*callee, caller, &edges) {
                    continue;
                }
                let generic_component = (0..self.function_templates.len()).any(|candidate| {
                    Self::graph_reaches(caller, candidate, &edges)
                        && Self::graph_reaches(candidate, caller, &edges)
                        && !self.function_templates[candidate]
                            .generic_parameters
                            .is_empty()
                });
                if generic_component {
                    return self.unsupported(UnsupportedSemanticFeature::Generics, *call);
                }
            }
        }
        Ok(())
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
                    return self.unsupported(UnsupportedSemanticFeature::FloatingPoint, node);
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
        self.generic_substitution(node, &template.generic_parameters, caller, true)
    }

    pub(super) fn nominal_generic_substitution(
        &self,
        node: NodeId,
        parameters: &[GenericParameter],
        caller: &GenericSubstitution,
    ) -> Result<GenericSubstitution, CheckStop> {
        self.generic_substitution(node, parameters, caller, false)
    }

    fn generic_substitution(
        &self,
        node: NodeId,
        parameters: &[GenericParameter],
        caller: &GenericSubstitution,
        allow_trailing_regions: bool,
    ) -> Result<GenericSubstitution, CheckStop> {
        if parameters.is_empty() {
            if !allow_trailing_regions
                && self
                    .tree
                    .first_child_with(node, Production::Targs)?
                    .is_some()
            {
                return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
            }
            return Ok(GenericSubstitution::default());
        }
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        if (allow_trailing_regions && arguments.len() < parameters.len())
            || (!allow_trailing_regions && arguments.len() != parameters.len())
        {
            return self.issue_node(SemanticRule::Type5, node, SemanticIssueKind::TypeMismatch);
        }
        let mut bindings = Vec::with_capacity(parameters.len());
        for (parameter, argument) in parameters.iter().copied().zip(arguments) {
            let value = match parameter {
                GenericParameter::Type { bound, .. } => {
                    let Some(ty) = self.tree.first_child_with(argument, Production::Type)? else {
                        return self.issue_node(
                            SemanticRule::Type5,
                            argument,
                            SemanticIssueKind::TypeMismatch,
                        );
                    };
                    let ty = self.parse_type_with(ty, caller)?;
                    if bound == GenericBound::Int
                        && !matches!(ty, CheckedType::Integer(_) | CheckedType::GenericInt(_))
                    {
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
                            SemanticRule::Type5,
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
