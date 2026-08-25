use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationId, DeclarationRole, DeferredUseRole, DependentDeclarationRole,
    LexicalUseRole, PreludeDeclarationId, Production, ResolvedTarget, SemanticCompilerFailure,
    SemanticIssueKind, SemanticRule,
};

use super::super::model::{
    CheckedConformance, CheckedConformanceBinding, CheckedContract, CheckedContractLaw,
    CheckedContractLawKind, CheckedContractMember, CheckedContractParameter,
    CheckedEffectCapabilities, CheckedExpression, CheckedFlatElement, CheckedFunction,
    CheckedIntegerOperation, CheckedLawDerivation, CheckedLawIdentity, CheckedMode,
    CheckedNominalKind, CheckedStatement, CheckedType, CheckedValue, ConformanceId, ContractId,
    IntegerType,
};
use super::generics::{GenericArgument, GenericSubstitution};
use super::{
    CheckStop, Checker, ContractInfo, EffectSet, FunctionSignature, ParameterSignature,
    derive_slice_return_ceiling,
};

pub(super) struct ContractMemberInfo {
    pub(super) name: String,
    pub(super) region_parameters: Vec<DeclarationId>,
    pub(super) parameters: Vec<ParameterSignature>,
    pub(super) result_mode: CheckedMode,
    pub(super) result: CheckedType,
    pub(super) slice_return_ceiling: Vec<super::super::model::CheckedSliceOrigin>,
    pub(super) effects: EffectSet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NormalizedMode {
    Own,
    Shared(usize),
    Unique(usize),
}

/// One [FN-3] effect normalization. Region-kind equality is checked beside
/// these ordinal sets, and `traps` is the sole payload-free category.
#[derive(Clone, Debug, Eq, PartialEq)]
struct NormalizedEffects {
    reads: Vec<usize>,
    writes: Vec<usize>,
    allocates_heap: bool,
    allocates_arenas: Vec<usize>,
    traps: bool,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn collect_contracts(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        for (item_position, node) in items.iter().copied().enumerate() {
            if self.tree.production(node)? != Production::ContractDecl {
                continue;
            }
            if let Some(generics) = self.tree.first_child_with(node, Production::Generics)? {
                return self.issue_node(
                    SemanticRule::Fn3,
                    generics,
                    SemanticIssueKind::GenericContract,
                );
            }

            let declaration = self.declaration_at(node, DeclarationRole::Contract)?;
            let declaration_id = declaration.id();
            let name = declaration.spelling().to_owned();
            let mut members = Vec::new();
            let mut checked_members = Vec::new();
            for member_node in self.tree.children_with(node, Production::FnSig)? {
                let member = self.contract_member(member_node)?;
                if members
                    .iter()
                    .any(|earlier: &ContractMemberInfo| earlier.name == member.name)
                {
                    return self.issue_node(
                        SemanticRule::Fn3,
                        member_node,
                        SemanticIssueKind::DuplicateContractMember {
                            member: member.name,
                        },
                    );
                }
                checked_members.push(CheckedContractMember {
                    name: member.name.clone(),
                    region_parameters: member.region_parameters.clone(),
                    parameters: member
                        .parameters
                        .iter()
                        .map(|parameter| CheckedContractParameter {
                            mode: parameter.mode,
                            ty: parameter.ty,
                        })
                        .collect(),
                    result_mode: member.result_mode,
                    result: member.result,
                    slice_return_ceiling: member.slice_return_ceiling.clone(),
                    effects: checked_effects(&member.effects),
                });
                members.push(member);
            }

            let mut laws = Vec::new();
            for law in self.tree.children_with(node, Production::Law)? {
                laws.push(self.contract_law(law, &members, item_position, items)?);
            }

            let id = ContractId(
                u32::try_from(self.contracts.len())
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            let checked = CheckedContract {
                id,
                declaration: declaration_id,
                name,
                members: checked_members,
                laws,
            };
            let index = self.contracts.len();
            if self
                .contracts_by_declaration
                .insert(declaration_id, index)
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
            self.contracts.push(ContractInfo { checked, members });
        }
        Ok(())
    }

    fn contract_member(&mut self, node: NodeId) -> Result<ContractMemberInfo, CheckStop> {
        self.ensure_nominals_in_node(node, &GenericSubstitution::default())?;
        let name = self
            .dependent_declaration_at(node, DependentDeclarationRole::ContractMember)?
            .spelling()
            .to_owned();
        let region_parameters = self.parse_region_parameters(node)?;
        let parameters = self.parse_parameters_with(node, &GenericSubstitution::default())?;
        let result_binding = self
            .tree
            .first_child_with(node, Production::ResultBinding)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let rtype = self
            .tree
            .first_child_with(result_binding, Production::Rtype)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let (result_mode, result) =
            self.parse_rtype_with(rtype, &GenericSubstitution::default())?;
        // [STOR-4] an arena result has no legal producing return; a member
        // signature is judged exactly as a top-level callable boundary.
        if self.arena_instance(result)?.is_some() {
            return self.issue_node(
                SemanticRule::Stor4,
                rtype,
                SemanticIssueKind::ArenaEscape {
                    mechanical_fix: super::ARENA_ESCAPE_RESTRUCTURING,
                },
            );
        }
        if result_mode != CheckedMode::Own && matches!(result, CheckedType::Slice { .. }) {
            return self.issue_node(
                SemanticRule::Fn1,
                rtype,
                SemanticIssueKind::BorrowedSliceResult {
                    mechanical_fix: "return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor",
                },
            );
        }
        self.reject_ambiguous_result_provenance(&parameters, result_mode, result, rtype)?;
        let slice_return_ceiling = derive_slice_return_ceiling(&parameters, result_mode, result);
        let effects_node = self
            .tree
            .first_child_with(node, Production::Effects)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let effects = self.parse_effects(effects_node)?;
        Ok(ContractMemberInfo {
            name,
            region_parameters,
            parameters,
            result_mode,
            result,
            slice_return_ceiling,
            effects,
        })
    }

    fn contract_law(
        &self,
        node: NodeId,
        members: &[ContractMemberInfo],
        contract_position: usize,
        items: &[NodeId],
    ) -> Result<CheckedContractLaw, CheckStop> {
        let name = self
            .deferred_use_at(node, DeferredUseRole::LawName)?
            .spelling();
        let arguments = self.tree.children_with(node, Production::LawArg)?;
        let (kind, required_arity) = match name {
            "associative" => (CheckedContractLawKind::Associative, 1),
            "commutative" => (CheckedContractLawKind::Commutative, 1),
            "identity" => (CheckedContractLawKind::Identity, 2),
            _ => return self.invalid_law(node),
        };
        if arguments.len() != required_arity {
            return self.invalid_law(node);
        }
        let member_argument = arguments[0];
        if self
            .tree
            .direct_token_with(member_argument, TerminalPredicate::Identifier)?
            .is_none()
        {
            return self.invalid_law(node);
        }
        let member_name = self
            .deferred_use_at(member_argument, DeferredUseRole::LawArgument)?
            .spelling();
        let Some((member_index, member)) = members
            .iter()
            .enumerate()
            .find(|(_, member)| member.name == member_name)
        else {
            return self.invalid_law(node);
        };
        if !law_member_shape(member) {
            return self.invalid_law(node);
        }

        let identity = if kind == CheckedContractLawKind::Identity {
            Some(self.law_identity(
                node,
                arguments[1],
                member.result_mode,
                member.result,
                contract_position,
                items,
            )?)
        } else {
            None
        };
        Ok(CheckedContractLaw {
            node_path: self.tree.path(node)?.clone(),
            kind,
            member: u32::try_from(member_index)
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            identity,
        })
    }

    fn law_identity(
        &self,
        law: NodeId,
        argument: NodeId,
        mode: CheckedMode,
        ty: CheckedType,
        contract_position: usize,
        items: &[NodeId],
    ) -> Result<CheckedLawIdentity, CheckStop> {
        if mode != CheckedMode::Own || !self.is_copy_type(ty)? {
            return self.invalid_law(law);
        }
        if let Some(literal) = self
            .tree
            .direct_token_with(argument, TerminalPredicate::Literal)?
        {
            let bytes = self.tree.token_bytes(literal)?;
            if bytes.starts_with(b"\"") {
                return self.invalid_law(law);
            }
            let value = self.parse_literal(argument, bytes)?;
            if value.ty() != ty {
                return self.invalid_law(law);
            }
            return Ok(CheckedLawIdentity::Literal(value));
        }

        let spelling = self
            .deferred_use_at(argument, DeferredUseRole::LawArgument)?
            .spelling();
        for item in items.iter().copied().take(contract_position) {
            if self.tree.production(item)? != Production::ConstDecl {
                continue;
            }
            let declaration = self.declaration_at(item, DeclarationRole::NamedConst)?;
            if declaration.spelling() != spelling {
                continue;
            }
            let id = *self
                .constants
                .get(&declaration.id())
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if self.constant(id)?.ty != ty {
                return self.invalid_law(law);
            }
            return Ok(CheckedLawIdentity::Constant(id));
        }
        self.invalid_law(law)
    }

    fn invalid_law<ResultValue>(&self, node: NodeId) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Fn4,
            node,
            SemanticIssueKind::InvalidContractLaw,
        )
    }

    pub(super) fn check_conformances_and_laws(
        &mut self,
        items: &[NodeId],
        functions: &[CheckedFunction],
    ) -> Result<(Vec<CheckedConformance>, Vec<CheckedLawDerivation>), CheckStop> {
        let mut conformances = Vec::new();
        let mut keys = Vec::new();
        for node in items.iter().copied() {
            if self.tree.production(node)? != Production::ConformDecl {
                continue;
            }
            let subject_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.ensure_nominal_type(subject_node, &GenericSubstitution::default())?;
            let subject = self.parse_type(subject_node)?;
            if !subject.is_concrete() {
                return self.issue_node(
                    SemanticRule::Fn3,
                    subject_node,
                    SemanticIssueKind::NonConcreteConformanceSubject,
                );
            }

            let contract_use = self.use_at(node, LexicalUseRole::ConformanceContract)?;
            let contract_index = match contract_use.target() {
                ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::Contract,
                } => *self
                    .contracts_by_declaration
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?,
                ResolvedTarget::Prelude(id)
                    if id == PreludeDeclarationId::new(22)
                        || id == PreludeDeclarationId::new(23) =>
                {
                    return self.issue_at(
                        SemanticRule::Fn3,
                        node,
                        contract_use.origin().coordinate(),
                        SemanticIssueKind::InvalidConformanceContract,
                    );
                }
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            };
            if let Some(arguments) = self.tree.first_child_with(node, Production::Targs)? {
                return self.issue_node(
                    SemanticRule::Fn3,
                    arguments,
                    SemanticIssueKind::ConformanceContractArguments,
                );
            }
            let contract = self
                .contracts
                .get(contract_index)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if keys.iter().any(|(earlier_subject, earlier_contract)| {
                *earlier_subject == subject && *earlier_contract == contract.checked.id
            }) {
                return self.issue_node(
                    SemanticRule::Fn3,
                    node,
                    SemanticIssueKind::DuplicateConformance,
                );
            }
            keys.push((subject, contract.checked.id));

            let binding_nodes = self.tree.children_with(node, Production::FnBind)?;
            let mut bindings = Vec::with_capacity(binding_nodes.len());
            for (member_index, binding_node) in binding_nodes.iter().copied().enumerate() {
                let expected = contract.members.get(member_index);
                let written_member = self
                    .deferred_use_at(binding_node, DeferredUseRole::ContractBinding)?
                    .spelling();
                if expected.is_none_or(|member| member.name != written_member) {
                    return self.issue_node(
                        SemanticRule::Fn3,
                        binding_node,
                        SemanticIssueKind::InvalidConformanceBinding {
                            expected_member: expected.map(|member| member.name.clone()),
                        },
                    );
                }
                let usage = self.use_at(binding_node, LexicalUseRole::FunctionBinding)?;
                let ResolvedTarget::Source {
                    declaration,
                    class: DeclarationClass::Function,
                } = usage.target()
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let template_index = *self
                    .templates_by_declaration
                    .get(&declaration)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let template = self
                    .function_templates
                    .get(template_index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !template.generic_parameters.is_empty()
                    || self
                        .tree
                        .first_child_with(template.node, Production::ContractBlock)?
                        .is_some()
                {
                    return self.issue_node(
                        SemanticRule::Fn3,
                        binding_node,
                        SemanticIssueKind::IncompatibleConformanceFunction,
                    );
                }
                let function = self
                    .functions_by_declaration
                    .get(&declaration)
                    .into_iter()
                    .flatten()
                    .copied()
                    .next()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let signature = self
                    .signatures
                    .get(function.0 as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !self.signatures_equal(
                    expected.ok_or(SemanticCompilerFailure::InvalidResolution)?,
                    signature,
                )? {
                    return self.issue_node(
                        SemanticRule::Fn3,
                        binding_node,
                        SemanticIssueKind::IncompatibleConformanceFunction,
                    );
                }
                bindings.push(CheckedConformanceBinding {
                    member: u32::try_from(member_index)
                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                    function,
                });
            }
            if binding_nodes.len() < contract.members.len() {
                let missing = &contract.members[binding_nodes.len()].name;
                return self.issue_at(
                    SemanticRule::Fn3,
                    node,
                    self.tree.closing_brace_coordinate(node)?,
                    SemanticIssueKind::MissingConformanceBinding {
                        member: missing.clone(),
                    },
                );
            }

            conformances.push(CheckedConformance {
                id: ConformanceId(
                    u32::try_from(conformances.len())
                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                ),
                node_path: self.tree.path(node)?.clone(),
                subject,
                contract: contract.checked.id,
                bindings,
            });
        }

        let mut derivations = Vec::new();
        for conformance in &conformances {
            let contract = self
                .contracts
                .get(conformance.contract.0 as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            for (law_index, law) in contract.checked.laws.iter().enumerate() {
                let law_node = self
                    .tree
                    .node_with_path(&law.node_path)
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let member_index = usize::try_from(law.member)
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
                let member = contract
                    .members
                    .get(member_index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let binding = conformance
                    .bindings
                    .get(member_index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let function = functions
                    .get(binding.function.0 as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let Some(domain) = discharge_domain(
                    conformance.subject,
                    member,
                    function,
                    law,
                    &self.checked_constants,
                ) else {
                    return self.undischarged_law(law_node);
                };
                derivations.push(CheckedLawDerivation {
                    conformance: conformance.id,
                    contract_law: u32::try_from(law_index)
                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                    function: binding.function,
                    operation: CheckedIntegerOperation::AddSaturating,
                    domain,
                    law: law.kind,
                    identity: law.identity.clone(),
                });
            }
        }
        Ok((conformances, derivations))
    }

    fn undischarged_law<ResultValue>(&self, node: NodeId) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Fn4,
            node,
            SemanticIssueKind::UndischargedContractLaw,
        )
    }
}

fn checked_effects(effects: &EffectSet) -> CheckedEffectCapabilities {
    CheckedEffectCapabilities {
        reads: effects.reads.clone(),
        writes: effects.writes.clone(),
        allocates_heap: effects.allocates_heap,
        allocates_arenas: effects.allocates_arenas.clone(),
        traps: effects.traps,
    }
}

fn law_member_shape(member: &ContractMemberInfo) -> bool {
    let [first, second] = member.parameters.as_slice() else {
        return false;
    };
    member.effects == EffectSet::NONE
        && first.mode == second.mode
        && first.mode == member.result_mode
        && first.ty == second.ty
        && first.ty == member.result
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    fn signatures_equal(
        &self,
        member: &ContractMemberInfo,
        function: &FunctionSignature,
    ) -> Result<bool, CheckStop> {
        if member.region_parameters.len() != function.region_parameters.len()
            || member.parameters.len() != function.parameters.len()
        {
            return Ok(false);
        }
        for (left, right) in member
            .region_parameters
            .iter()
            .zip(&function.region_parameters)
        {
            if self.region_kind(*left) != self.region_kind(*right) {
                return Ok(false);
            }
        }
        if !self.alpha_equivalent_type(
            member.result,
            &member.region_parameters,
            function.result,
            &function.region_parameters,
        )? {
            return Ok(false);
        }
        for (member_parameter, function_parameter) in
            member.parameters.iter().zip(&function.parameters)
        {
            if !self.alpha_equivalent_type(
                member_parameter.ty,
                &member.region_parameters,
                function_parameter.ty,
                &function.region_parameters,
            )? || normalize_mode(member_parameter.mode, &member.region_parameters)?
                != normalize_mode(function_parameter.mode, &function.region_parameters)?
            {
                return Ok(false);
            }
        }
        if normalize_mode(member.result_mode, &member.region_parameters)?
            != normalize_mode(function.result_mode, &function.region_parameters)?
        {
            return Ok(false);
        }
        Ok(
            normalize_effects(&member.effects, &member.region_parameters)?
                == normalize_effects(&function.declared_effects, &function.region_parameters)?,
        )
    }

    fn alpha_equivalent_type(
        &self,
        left: CheckedType,
        left_regions: &[DeclarationId],
        right: CheckedType,
        right_regions: &[DeclarationId],
    ) -> Result<bool, CheckStop> {
        match (left, right) {
            (
                CheckedType::Slice {
                    region: left_region,
                    element: left_element,
                },
                CheckedType::Slice {
                    region: right_region,
                    element: right_element,
                },
            ) => Ok(region_ordinal(left_region, left_regions)?
                == region_ordinal(right_region, right_regions)?
                && self.alpha_equivalent_flat(
                    left_element,
                    left_regions,
                    right_element,
                    right_regions,
                )?),
            (
                CheckedType::Array {
                    element: left_element,
                    length: left_length,
                },
                CheckedType::Array {
                    element: right_element,
                    length: right_length,
                },
            ) => Ok(left_length == right_length
                && self.alpha_equivalent_flat(
                    left_element,
                    left_regions,
                    right_element,
                    right_regions,
                )?),
            (
                CheckedType::Buffer {
                    element: left_element,
                },
                CheckedType::Buffer {
                    element: right_element,
                },
            ) => {
                self.alpha_equivalent_flat(left_element, left_regions, right_element, right_regions)
            }
            (CheckedType::Nominal(left), CheckedType::Nominal(right)) => {
                self.alpha_equivalent_nominal(left, left_regions, right, right_regions)
            }
            _ => Ok(left == right),
        }
    }

    fn alpha_equivalent_flat(
        &self,
        left: CheckedFlatElement,
        left_regions: &[DeclarationId],
        right: CheckedFlatElement,
        right_regions: &[DeclarationId],
    ) -> Result<bool, CheckStop> {
        match (left, right) {
            (
                CheckedFlatElement::TagOnlyNominal(left) | CheckedFlatElement::Nominal(left),
                CheckedFlatElement::TagOnlyNominal(right) | CheckedFlatElement::Nominal(right),
            ) => self.alpha_equivalent_nominal(left, left_regions, right, right_regions),
            _ => Ok(left == right),
        }
    }

    fn alpha_equivalent_nominal(
        &self,
        left: super::super::model::NominalId,
        left_regions: &[DeclarationId],
        right: super::super::model::NominalId,
        right_regions: &[DeclarationId],
    ) -> Result<bool, CheckStop> {
        if left == right {
            return Ok(true);
        }
        let left_nominal = self.nominal(left)?;
        let right_nominal = self.nominal(right)?;
        match (&left_nominal.kind, &right_nominal.kind) {
            (
                CheckedNominalKind::SystemResource {
                    nominal: left_nominal,
                    world_regions: left_world,
                },
                CheckedNominalKind::SystemResource {
                    nominal: right_nominal,
                    world_regions: right_world,
                },
            ) => Ok(left_nominal == right_nominal
                && alpha_equivalent_region_vectors(
                    left_world,
                    left_regions,
                    right_world,
                    right_regions,
                )?),
            (
                CheckedNominalKind::Box {
                    referent: left_referent,
                },
                CheckedNominalKind::Box {
                    referent: right_referent,
                },
            ) => self.alpha_equivalent_type(
                *left_referent,
                left_regions,
                *right_referent,
                right_regions,
            ),
            (
                CheckedNominalKind::Arena {
                    region: left_region,
                    content: left_content,
                },
                CheckedNominalKind::Arena {
                    region: right_region,
                    content: right_content,
                },
            ) => Ok(region_ordinal(*left_region, left_regions)?
                == region_ordinal(*right_region, right_regions)?
                && self.alpha_equivalent_type(
                    *left_content,
                    left_regions,
                    *right_content,
                    right_regions,
                )?),
            (CheckedNominalKind::ArenaStorage, CheckedNominalKind::ArenaStorage) => Ok(true),
            _ => {
                if let (Some(left_prelude), Some(right_prelude)) =
                    (self.prelude_type(left), self.prelude_type(right))
                {
                    return self.alpha_equivalent_prelude(
                        left_prelude,
                        left_regions,
                        right_prelude,
                        right_regions,
                    );
                }
                let left_source = self
                    .source_nominal_instances
                    .get(left.0 as usize)
                    .and_then(Option::as_ref);
                let right_source = self
                    .source_nominal_instances
                    .get(right.0 as usize)
                    .and_then(Option::as_ref);
                let (
                    Some((left_template, left_substitution)),
                    Some((right_template, right_substitution)),
                ) = (left_source, right_source)
                else {
                    return Ok(false);
                };
                if left_template != right_template
                    || left_substitution.entries().len() != right_substitution.entries().len()
                {
                    return Ok(false);
                }
                for ((_, left_argument), (_, right_argument)) in left_substitution
                    .entries()
                    .iter()
                    .zip(right_substitution.entries())
                {
                    match (left_argument, right_argument) {
                        (GenericArgument::Type(left), GenericArgument::Type(right)) => {
                            if !self.alpha_equivalent_type(
                                *left,
                                left_regions,
                                *right,
                                right_regions,
                            )? {
                                return Ok(false);
                            }
                        }
                        (GenericArgument::Const(left), GenericArgument::Const(right))
                            if left == right => {}
                        _ => return Ok(false),
                    }
                }
                Ok(true)
            }
        }
    }

    fn alpha_equivalent_prelude(
        &self,
        left: super::PreludeType,
        left_regions: &[DeclarationId],
        right: super::PreludeType,
        right_regions: &[DeclarationId],
    ) -> Result<bool, CheckStop> {
        match (left, right) {
            (super::PreludeType::Option(left), super::PreludeType::Option(right)) => {
                self.alpha_equivalent_type(left, left_regions, right, right_regions)
            }
            (
                super::PreludeType::Result(left_ok, left_error),
                super::PreludeType::Result(right_ok, right_error),
            ) => Ok(
                self.alpha_equivalent_type(left_ok, left_regions, right_ok, right_regions)?
                    && self.alpha_equivalent_type(
                        left_error,
                        left_regions,
                        right_error,
                        right_regions,
                    )?,
            ),
            _ => Ok(left == right),
        }
    }
}

fn alpha_equivalent_region_vectors(
    left: &[DeclarationId],
    left_regions: &[DeclarationId],
    right: &[DeclarationId],
    right_regions: &[DeclarationId],
) -> Result<bool, CheckStop> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        if region_ordinal(*left, left_regions)? != region_ordinal(*right, right_regions)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn region_ordinal(region: DeclarationId, regions: &[DeclarationId]) -> Result<usize, CheckStop> {
    regions
        .iter()
        .position(|candidate| *candidate == region)
        .ok_or(SemanticCompilerFailure::InvalidResolution.into())
}

fn normalize_mode(
    mode: CheckedMode,
    regions: &[DeclarationId],
) -> Result<NormalizedMode, CheckStop> {
    match mode {
        CheckedMode::Own => Ok(NormalizedMode::Own),
        CheckedMode::Shared(region) => regions
            .iter()
            .position(|candidate| *candidate == region)
            .map(NormalizedMode::Shared)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into()),
        CheckedMode::Unique(region) => regions
            .iter()
            .position(|candidate| *candidate == region)
            .map(NormalizedMode::Unique)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into()),
    }
}

fn normalize_effects(
    effects: &EffectSet,
    regions: &[DeclarationId],
) -> Result<NormalizedEffects, CheckStop> {
    Ok(NormalizedEffects {
        reads: normalize_regions(&effects.reads, regions)?,
        writes: normalize_regions(&effects.writes, regions)?,
        allocates_heap: effects.allocates_heap,
        allocates_arenas: normalize_regions(&effects.allocates_arenas, regions)?,
        traps: effects.traps,
    })
}

fn normalize_regions(
    selected: &[DeclarationId],
    regions: &[DeclarationId],
) -> Result<Vec<usize>, CheckStop> {
    let mut normalized = selected
        .iter()
        .map(|selected| {
            regions
                .iter()
                .position(|candidate| candidate == selected)
                .ok_or(SemanticCompilerFailure::InvalidResolution)
        })
        .collect::<Result<Vec<_>, _>>()?;
    normalized.sort_unstable();
    Ok(normalized)
}

fn discharge_domain(
    subject: CheckedType,
    member: &ContractMemberInfo,
    function: &CheckedFunction,
    law: &CheckedContractLaw,
    constants: &[super::super::model::CheckedConstant],
) -> Option<IntegerType> {
    let CheckedType::Integer(domain) = subject else {
        return None;
    };
    let [first, second] = member.parameters.as_slice() else {
        return None;
    };
    if !member.region_parameters.is_empty()
        || member.effects != EffectSet::NONE
        || first.mode != CheckedMode::Own
        || second.mode != CheckedMode::Own
        || member.result_mode != CheckedMode::Own
        || first.ty != subject
        || second.ty != subject
        || member.result != subject
    {
        return None;
    }
    let [first_parameter, second_parameter] = function.parameters.as_slice() else {
        return None;
    };
    if first_parameter.mode != CheckedMode::Own
        || second_parameter.mode != CheckedMode::Own
        || first_parameter.ty != subject
        || second_parameter.ty != subject
        || function.result_mode != CheckedMode::Own
        || function.result != subject
        || function.declared_traps
        || function.declared_allocates_heap
        || !function.requirements.is_empty()
    {
        return None;
    }
    let [CheckedStatement::Return { value, drops, .. }] = function.body.as_slice() else {
        return None;
    };
    if !drops.is_empty() {
        return None;
    }
    let CheckedExpression::IntegerOperation {
        operation: CheckedIntegerOperation::AddSaturating,
        operand_type,
        arguments,
        result,
        ..
    } = value
    else {
        return None;
    };
    let [
        CheckedExpression::Binding {
            binding: first_binding,
            ty: first_type,
            ..
        },
        CheckedExpression::Binding {
            binding: second_binding,
            ty: second_type,
            ..
        },
    ] = arguments.as_slice()
    else {
        return None;
    };
    if *operand_type != subject
        || *result != subject
        || *first_binding != first_parameter.binding
        || *second_binding != second_parameter.binding
        || *first_type != subject
        || *second_type != subject
    {
        return None;
    }
    let holds = match law.kind {
        CheckedContractLawKind::Associative => !domain.signed(),
        CheckedContractLawKind::Commutative => true,
        CheckedContractLawKind::Identity => law
            .identity
            .as_ref()
            .is_some_and(|identity| identity_is_zero(identity, domain, constants)),
    };
    holds.then_some(domain)
}

fn identity_is_zero(
    identity: &CheckedLawIdentity,
    domain: IntegerType,
    constants: &[super::super::model::CheckedConstant],
) -> bool {
    let value = match identity {
        CheckedLawIdentity::Literal(value) => value,
        CheckedLawIdentity::Constant(id) => match constants.get(id.0 as usize) {
            Some(constant) => &constant.value,
            None => return false,
        },
    };
    matches!(
        value,
        CheckedValue::Integer { ty, bits } if *ty == domain && *bits == 0
    )
}
