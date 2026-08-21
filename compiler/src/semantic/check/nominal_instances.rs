use std::collections::HashSet;

use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationRole, DependentDeclarationRole, LexicalUseRole,
    PreludeDeclarationId, Production, ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind,
    SemanticRule,
};

use super::super::model::{
    CheckedConstructor, CheckedField, CheckedNominal, CheckedNominalKind, CheckedNumericType,
    CheckedType, CheckedVariant, NominalId,
};
use super::generics::GenericSubstitution;
use super::{
    CheckStop, Checker, ConstructorTemplate, NominalInstance, NominalTemplate, PreludeType,
};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn declare_nominals(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        let nodes = items
            .iter()
            .copied()
            .filter(|node| {
                self.tree.production(*node).is_ok_and(|production| {
                    matches!(production, Production::StructDecl | Production::EnumDecl)
                })
            })
            .collect::<Vec<_>>();
        for node in nodes {
            self.declare_nominal_template(node)?;
        }
        for index in 0..self.nominal_templates.len() {
            if self.nominal_templates[index].generic_parameters.is_empty() {
                self.declare_source_nominal_instance(index, GenericSubstitution::default())?;
            }
        }
        Ok(())
    }

    /// Scratch inventory for selector signatures. Invalid source templates
    /// are unavailable only to signatures that name them; unrelated nominal
    /// declarations cannot suppress an independently decidable FN-9 verdict.
    pub(super) fn declare_nominals_for_postconditions(
        &mut self,
        items: &[NodeId],
    ) -> Result<(), CheckStop> {
        let nodes = items
            .iter()
            .copied()
            .filter(|node| {
                self.tree.production(*node).is_ok_and(|production| {
                    matches!(production, Production::StructDecl | Production::EnumDecl)
                })
            })
            .collect::<Vec<_>>();
        for node in nodes {
            match self.declare_nominal_template(node) {
                Ok(()) => {}
                Err(CheckStop::Issue(_) | CheckStop::Unsupported(_)) => {
                    let role = match self.tree.production(node)? {
                        Production::StructDecl => DeclarationRole::Struct,
                        Production::EnumDecl => DeclarationRole::Enum,
                        _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
                    };
                    self.mark_postcondition_unavailable(self.declaration_at(node, role)?.id());
                }
                Err(stop) => return Err(stop),
            }
        }
        // Source instances are created and completed lazily by the exact
        // signature/type helpers. This is the dependency-local equivalent of
        // `complete_nominals`; PRE-1 instances remain ordinary shared setup.
        self.register_prelude_nominals()?;

        // A generic nominal declaration is a usable FN-2 signature premise
        // only after its ordinary symbolic template judgment succeeds. Keep
        // that judgment dependency-local: a bad template is unavailable to
        // headers and call arguments that name it, while an unrelated
        // selector remains independently decidable.
        for template_index in 0..self.nominal_templates.len() {
            let template = self
                .nominal_templates
                .get(template_index)
                .cloned()
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if template.generic_parameters.is_empty()
                || self.postcondition_declaration_unavailable(template.declaration)
            {
                continue;
            }
            let checkpoint = self.nominal_checkpoint();
            let result = (|| {
                let substitution =
                    self.symbolic_generic_substitution(&template.generic_parameters)?;
                self.ensure_source_nominal_instance(template_index, substitution)?;
                self.reject_recursive_nominal_layouts()
            })();
            self.restore_nominal_checkpoint(checkpoint)?;
            match result {
                Ok(()) => {}
                Err(
                    CheckStop::Issue(_)
                    | CheckStop::Unsupported(_)
                    | CheckStop::PostconditionPrerequisiteUnavailable,
                ) => self.mark_postcondition_unavailable(template.declaration),
                Err(stop) => return Err(stop),
            }
        }
        Ok(())
    }

    fn declare_nominal_template(&mut self, node: NodeId) -> Result<(), CheckStop> {
        let role = match self.tree.production(node)? {
            Production::StructDecl => DeclarationRole::Struct,
            Production::EnumDecl => DeclarationRole::Enum,
            _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        };
        let declaration = self.declaration_at(node, role)?;
        let declaration_id = declaration.id();
        // Parse every source-bearing premise before publishing any table
        // entry, so a tolerant scratch failure is atomic.
        let generic_parameters = self.parse_generic_parameters(node)?;
        let variants = if role == DeclarationRole::Enum {
            self.tree.children_with(node, Production::Variant)?
        } else {
            Vec::new()
        };
        let template = NominalTemplate {
            declaration: declaration_id,
            node,
            name: declaration.spelling().to_owned(),
            role,
            generic_parameters,
        };
        let template_index = self.nominal_templates.len();
        if self
            .nominal_templates_by_declaration
            .insert(declaration_id, template_index)
            .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        if role == DeclarationRole::Struct
            && self
                .constructor_templates_by_declaration
                .insert(
                    declaration_id,
                    ConstructorTemplate::Struct {
                        template: template_index,
                    },
                )
                .is_some()
        {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        for (variant, variant_node) in variants.into_iter().enumerate() {
            let declaration = self.declaration_at(variant_node, DeclarationRole::Variant)?;
            let variant =
                u32::try_from(variant).map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            if self
                .constructor_templates_by_declaration
                .insert(
                    declaration.id(),
                    ConstructorTemplate::Enum {
                        template: template_index,
                        variant,
                    },
                )
                .is_some()
            {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        }
        self.nominal_templates.push(template);
        Ok(())
    }

    pub(super) fn complete_nominals(&mut self) -> Result<(), CheckStop> {
        self.register_prelude_nominals()?;
        self.complete_pending_source_nominals()?;
        self.reject_recursive_nominal_layouts()?;
        self.validate_nominal_templates()
    }

    pub(super) fn ensure_nominals_in_node(
        &mut self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        for ty in self.nominal_type_descendants(node)? {
            self.ensure_nominal_type_head(ty, substitution)?;
        }
        for construct in self.tree.descendants_with(node, Production::Construct)? {
            self.ensure_source_constructor_instance(construct, substitution)?;
        }
        self.ensure_implicit_prelude_nominals(node, substitution, false)?;
        self.reject_recursive_nominal_layouts()
    }

    /// Performs the ordinary nominal pre-scan for one function without
    /// entering its FN-9 clause. Ensures entries use private provisional
    /// links and must first pass the clause subset judgment; they are consumed
    /// only by the postcondition checker after that admission.
    pub(super) fn ensure_nominals_in_function(
        &mut self,
        function: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        if self
            .tree
            .descendants_with(function, Production::EnsuresClause)?
            .is_empty()
        {
            return self.ensure_nominals_in_node(function, substitution);
        }
        // Preserve the exact ordinary category order across the retained
        // subtree: every type, then every constructor, then every implicit
        // PRE-1 instance, followed by one recursive-layout judgment.
        for ty in self.nominal_type_descendants(function)? {
            if self.node_is_inside_postcondition(ty)? {
                continue;
            }
            self.ensure_nominal_type_head(ty, substitution)?;
        }
        for construct in self
            .tree
            .descendants_with(function, Production::Construct)?
        {
            if self.node_is_inside_postcondition(construct)? {
                continue;
            }
            self.ensure_source_constructor_instance(construct, substitution)?;
        }
        self.ensure_implicit_prelude_nominals(function, substitution, true)?;
        self.reject_recursive_nominal_layouts()
    }

    /// Interns only nominal instances read by `build_function_signature`.
    /// The throwaway FN-9 preflight must not inspect requires, ensures, or the
    /// executable body before selector admission.
    pub(super) fn ensure_nominals_in_function_signature(
        &mut self,
        function: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        if let Some(parameters) = self
            .tree
            .first_child_with(function, Production::ParamList)?
        {
            self.ensure_nominals_in_node(parameters, substitution)?;
        }
        let result_binding = self
            .tree
            .first_child_with(function, Production::ResultBinding)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let result = self
            .tree
            .first_child_with(result_binding, Production::Rtype)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.ensure_nominals_in_node(result, substitution)
    }

    pub(super) fn ensure_nominal_type(
        &mut self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        for nested in self.nominal_type_descendants(node)? {
            self.ensure_nominal_type_head(nested, substitution)?;
        }
        self.ensure_nominal_type_head(node, substitution)
    }

    fn nominal_type_descendants(&self, node: NodeId) -> Result<Vec<NodeId>, CheckStop> {
        let mut nested = self.tree.descendants_with(node, Production::Type)?;
        nested.sort_by(|left, right| {
            let left_depth = self
                .tree
                .topology()
                .node(*left)
                .map(|record| record.tree_depth);
            let right_depth = self
                .tree
                .topology()
                .node(*right)
                .map(|record| record.tree_depth);
            right_depth
                .cmp(&left_depth)
                .then(left.index().cmp(&right.index()))
        });
        Ok(nested)
    }

    fn ensure_nominal_type_head(
        &mut self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        if self.has_fixed(node, crate::FixedTerminal::Box)? {
            let referent_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.reject_region_bearing_storage_type(referent_node, substitution)?;
            let referent = self.parse_type_with(referent_node, substitution)?;
            self.intern_box_nominal(referent)?;
            return Ok(());
        }
        if self.has_fixed(node, crate::FixedTerminal::Arena)? {
            let content_node = self
                .tree
                .first_child_with(node, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.reject_region_bearing_storage_type(content_node, substitution)?;
            let content = self.parse_type_with(content_node, substitution)?;
            let usage = self.use_at(node, crate::LexicalUseRole::TypeRegion)?;
            let crate::ResolvedTarget::Source {
                declaration: region,
                class: crate::DeclarationClass::Region,
            } = usage.target()
            else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            self.intern_arena_nominal(region, content)?;
            return Ok(());
        }
        if self
            .tree
            .direct_token_with(node, TerminalPredicate::TypeIdentifier)?
            .is_none()
        {
            return Ok(());
        }
        let usage = self.use_at(node, LexicalUseRole::Type)?;
        match usage.target() {
            ResolvedTarget::Prelude(id) if id == PreludeDeclarationId::new(3) => {
                let value = self.option_type_argument_with(node, substitution)?;
                self.intern_prelude_nominal(PreludeType::Option(value))?;
                Ok(())
            }
            ResolvedTarget::Prelude(id) if id == PreludeDeclarationId::new(8) => {
                let (ok, error) = self.result_type_arguments_with(node, substitution)?;
                self.intern_prelude_nominal(PreludeType::Result(ok, error))?;
                Ok(())
            }
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::NominalType,
            } => {
                if self.postcondition_declaration_unavailable(declaration) {
                    return Err(CheckStop::PostconditionPrerequisiteUnavailable);
                }
                let Some(template_index) = self
                    .nominal_templates_by_declaration
                    .get(&declaration)
                    .copied()
                else {
                    return Err(SemanticCompilerFailure::InvalidResolution.into());
                };
                let template = self
                    .nominal_templates
                    .get(template_index)
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let instance = self.nominal_generic_substitution(
                    node,
                    &template.generic_parameters,
                    substitution,
                )?;
                self.ensure_source_nominal_instance(template_index, instance)?;
                Ok(())
            }
            ResolvedTarget::System(id) => {
                if let Some(index) = crate::system_nominal_index(id, self.inventory()) {
                    self.intern_system_nominal(index)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn ensure_source_constructor_instance(
        &mut self,
        node: NodeId,
        caller: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        let usage = self.use_at(node, LexicalUseRole::Construct)?;
        if let ResolvedTarget::System(id) = usage.target() {
            if let Some(index) = crate::system_constructor_index(id, self.inventory()) {
                let owner = crate::SYSTEM_CONSTRUCTORS
                    .get(usize::from(index))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .owner;
                self.intern_system_nominal(owner)?;
            }
            return Ok(());
        }
        // [TYPE-5] a prelude variant constructor writes its nominal's
        // arguments, so the instance it names is interned from those written
        // arguments here, before function checking reads it immutably.
        if let ResolvedTarget::Prelude(id) = usage.target() {
            match id.ordinal() {
                5 | 6 => {
                    let value = self.option_type_argument_with(node, caller)?;
                    self.intern_prelude_nominal(PreludeType::Option(value))?;
                }
                11 | 13 => {
                    let (ok, error) = self.result_type_arguments_with(node, caller)?;
                    self.intern_prelude_nominal(PreludeType::Result(ok, error))?;
                }
                _ => {}
            }
            return Ok(());
        }
        let ResolvedTarget::Source { declaration, .. } = usage.target() else {
            return Ok(());
        };
        let constructor = *self
            .constructor_templates_by_declaration
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let template_index = match constructor {
            ConstructorTemplate::Struct { template }
            | ConstructorTemplate::Enum { template, .. } => template,
        };
        let template = self
            .nominal_templates
            .get(template_index)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let substitution =
            self.nominal_generic_substitution(node, &template.generic_parameters, caller)?;
        self.ensure_source_nominal_instance(template_index, substitution)?;
        Ok(())
    }

    fn ensure_implicit_prelude_nominals(
        &mut self,
        node: NodeId,
        substitution: &GenericSubstitution,
        skip_postconditions: bool,
    ) -> Result<(), CheckStop> {
        // A `propagate_let_rhs` needed its operand's `Result` instance
        // interned from the let's written annotation. [TYPE-5] deletes that
        // annotation and [ERR-3] derives the binder from the operand's own
        // Ok payload instead, so the instance is the callee's and its
        // signature already interned it.

        for call in self.tree.descendants_with(node, Production::Call)? {
            if skip_postconditions && self.node_is_inside_postcondition(call)? {
                continue;
            }
            let callee = self
                .tree
                .first_child_with(call, Production::Callee)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            // A call to an admitted system operation needs its written
            // [SYS-2] parameter and result instances — including `Result`
            // instantiations no source type spells — before function
            // checking reads them immutably.
            let callee_path = self.tree.path(callee)?;
            let system_operation = self
                .resolved
                .lexical_uses()
                .iter()
                .find(|usage| {
                    usage.origin().node() == callee_path
                        && matches!(
                            usage.role(),
                            LexicalUseRole::IdentifierCallee | LexicalUseRole::OperationCallee
                        )
                })
                .and_then(|usage| match usage.target() {
                    ResolvedTarget::System(id) => {
                        crate::system_operation_index(id, self.inventory())
                    }
                    _ => None,
                });
            if let Some(index) = system_operation {
                let operation = crate::SYSTEM_OPERATIONS
                    .get(usize::from(index))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                for parameter in operation.parameters {
                    self.ensure_system_type(parameter.ty)?;
                }
                self.ensure_system_type(operation.result)?;
                continue;
            }
            let spelling = self.tree.direct_spelling(callee)?;
            if spelling == b"cvt" {
                self.ensure_conversion_result(call, substitution)?;
                continue;
            }
            let error = if matches!(
                spelling.as_slice(),
                b"iadd.checked"
                    | b"isub.checked"
                    | b"imul.checked"
                    | b"iabs.checked"
                    | b"ineg.checked"
            ) {
                Some(PreludeType::Overflow)
            } else if matches!(spelling.as_slice(), b"idiv.checked" | b"irem.checked") {
                Some(PreludeType::DivError)
            } else {
                None
            };
            let Some(error) = error else {
                continue;
            };
            let Some(targs) = self.tree.first_child_with(call, Production::Targs)? else {
                continue;
            };
            let arguments = self.tree.children_with(targs, Production::Targ)?;
            let [argument] = arguments.as_slice() else {
                continue;
            };
            let Some(ty_node) = self.tree.first_child_with(*argument, Production::Type)? else {
                continue;
            };
            let operand = self.parse_type_with(ty_node, substitution)?;
            if !matches!(
                operand,
                CheckedType::Integer(_) | CheckedType::GenericInt(_)
            ) {
                continue;
            }
            let error = CheckedType::Nominal(self.prelude_nominal(error)?);
            self.intern_prelude_nominal(PreludeType::Result(operand, error))?;
        }
        Ok(())
    }

    fn ensure_conversion_result(
        &mut self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return Ok(());
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let [source_argument, destination_argument] = arguments.as_slice() else {
            return Ok(());
        };
        let (Some(source_node), Some(destination_node)) = (
            self.tree
                .first_child_with(*source_argument, Production::Type)?,
            self.tree
                .first_child_with(*destination_argument, Production::Type)?,
        ) else {
            return Ok(());
        };
        let (source, destination) = (
            self.parse_type_with(source_node, substitution)?,
            self.parse_type_with(destination_node, substitution)?,
        );
        let source = match source {
            CheckedType::Integer(ty) => CheckedNumericType::Integer(ty),
            CheckedType::Float(ty) => CheckedNumericType::Float(ty),
            _ => return Ok(()),
        };
        let destination = match destination {
            CheckedType::Integer(ty) => CheckedNumericType::Integer(ty),
            CheckedType::Float(ty) => CheckedNumericType::Float(ty),
            _ => return Ok(()),
        };
        if source == destination || source.converts_totally_to(destination) {
            return Ok(());
        }
        let error = CheckedType::Nominal(self.prelude_nominal(PreludeType::NarrowError)?);
        self.intern_prelude_nominal(PreludeType::Result(destination.ty(), error))?;
        Ok(())
    }

    fn declare_source_nominal_instance(
        &mut self,
        template_index: usize,
        substitution: GenericSubstitution,
    ) -> Result<NominalId, CheckStop> {
        let template = self
            .nominal_templates
            .get(template_index)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if let Some(id) = self.source_nominal_instance(template.declaration, &substitution) {
            return Ok(id);
        }
        let id = NominalId(
            u32::try_from(self.nominals.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
        );
        let name = if substitution.len() == 0 {
            template.name.clone()
        } else {
            format!("{}<instance:{}>", template.name, id.0)
        };
        self.nominal_nodes.push(Some(template.node));
        self.nominal_states.push(0);
        self.source_nominal_instances
            .push(Some((template_index, substitution.clone())));
        self.prelude_types.push(None);
        self.nominals.push(CheckedNominal {
            id,
            name,
            kind: match template.role {
                DeclarationRole::Struct => CheckedNominalKind::Struct { fields: Vec::new() },
                DeclarationRole::Enum => CheckedNominalKind::Enum {
                    variants: Vec::new(),
                },
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            },
        });
        self.nominals_by_declaration
            .entry(template.declaration)
            .or_default()
            .push(NominalInstance { id, substitution });
        Ok(id)
    }

    pub(super) fn ensure_source_nominal_instance(
        &mut self,
        template_index: usize,
        substitution: GenericSubstitution,
    ) -> Result<NominalId, CheckStop> {
        let id = self.declare_source_nominal_instance(template_index, substitution)?;
        self.complete_source_nominal_instance(id)?;
        Ok(id)
    }

    fn complete_pending_source_nominals(&mut self) -> Result<(), CheckStop> {
        let mut index = 0_usize;
        while index < self.nominals.len() {
            if self
                .source_nominal_instances
                .get(index)
                .is_some_and(Option::is_some)
            {
                self.complete_source_nominal_instance(NominalId(
                    u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                ))?;
            }
            index = index
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
        }
        Ok(())
    }

    fn validate_nominal_templates(&mut self) -> Result<(), CheckStop> {
        let checkpoint = self.nominal_checkpoint();
        for template_index in 0..self.nominal_templates.len() {
            let parameters = self.nominal_templates[template_index]
                .generic_parameters
                .clone();
            if parameters.is_empty() {
                continue;
            }
            let substitution = self.symbolic_generic_substitution(&parameters)?;
            self.ensure_source_nominal_instance(template_index, substitution)?;
        }
        self.reject_recursive_nominal_layouts()?;
        self.restore_nominal_checkpoint(checkpoint)
    }

    fn complete_source_nominal_instance(&mut self, id: NominalId) -> Result<(), CheckStop> {
        match self
            .nominal_states
            .get(id.0 as usize)
            .copied()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
        {
            2 => return Ok(()),
            1 => return Ok(()),
            0 => {}
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
        self.nominal_states[id.0 as usize] = 1;
        let (template_index, substitution) = self
            .source_nominal_instances
            .get(id.0 as usize)
            .and_then(Clone::clone)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let template = self
            .nominal_templates
            .get(template_index)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let kind = match template.role {
            DeclarationRole::Struct => CheckedNominalKind::Struct {
                fields: self.parse_struct_fields(template.node, &substitution)?,
            },
            DeclarationRole::Enum => CheckedNominalKind::Enum {
                variants: self.parse_enum_variants(template.node, &substitution)?,
            },
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        self.nominals
            .get_mut(id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .kind = kind;
        self.nominal_states[id.0 as usize] = 2;
        Ok(())
    }

    fn parse_struct_fields(
        &mut self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<Vec<CheckedField>, CheckStop> {
        let nodes = self.tree.children_with(node, Production::Field)?;
        let mut seen = HashSet::with_capacity(nodes.len());
        let mut fields = Vec::with_capacity(nodes.len());
        for field in nodes {
            let declaration =
                self.dependent_declaration_at(field, DependentDeclarationRole::Field)?;
            let name = declaration.spelling().to_owned();
            if !seen.insert(name.clone()) {
                return self.issue_node(
                    SemanticRule::Type6,
                    field,
                    SemanticIssueKind::DuplicateFieldLabel { label: name },
                );
            }
            let ty = self
                .tree
                .first_child_with(field, Production::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.reject_region_bearing_storage_type(ty, substitution)?;
            self.ensure_nominal_type(ty, substitution)?;
            let parsed = self.parse_type_with(ty, substitution)?;
            fields.push(CheckedField { name, ty: parsed });
        }
        Ok(fields)
    }

    fn parse_enum_variants(
        &mut self,
        node: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<Vec<CheckedVariant>, CheckStop> {
        let nodes = self.tree.children_with(node, Production::Variant)?;
        let mut variants = Vec::with_capacity(nodes.len());
        for variant_node in nodes {
            let declaration = self.declaration_at(variant_node, DeclarationRole::Variant)?;
            let declaration_id = declaration.id();
            let name = declaration.spelling().to_owned();
            let tag = u32::try_from(variants.len())
                .map_err(|_| SemanticCompilerFailure::CounterOverflow)?;
            let mut fields = Vec::new();
            let mut seen = HashSet::new();
            if let Some(list) = self
                .tree
                .first_child_with(variant_node, Production::VfieldList)?
            {
                for field in self.tree.children_with(list, Production::Vfield)? {
                    let declaration = self
                        .dependent_declaration_at(field, DependentDeclarationRole::VariantField)?;
                    let field_name = declaration.spelling().to_owned();
                    if !seen.insert(field_name.clone()) {
                        return self.issue_node(
                            SemanticRule::Type6,
                            field,
                            SemanticIssueKind::DuplicateFieldLabel { label: field_name },
                        );
                    }
                    let ty = self
                        .tree
                        .first_child_with(field, Production::Type)?
                        .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                    self.reject_region_bearing_storage_type(ty, substitution)?;
                    self.ensure_nominal_type(ty, substitution)?;
                    let parsed = self.parse_type_with(ty, substitution)?;
                    fields.push(CheckedField {
                        name: field_name,
                        ty: parsed,
                    });
                }
            }
            variants.push(CheckedVariant {
                name,
                constructor: CheckedConstructor::Source(declaration_id),
                tag,
                fields,
            });
        }
        Ok(variants)
    }

    pub(super) fn source_nominal_instance(
        &self,
        declaration: crate::DeclarationId,
        substitution: &GenericSubstitution,
    ) -> Option<NominalId> {
        self.nominals_by_declaration
            .get(&declaration)
            .into_iter()
            .flatten()
            .find(|instance| instance.substitution == *substitution)
            .map(|instance| instance.id)
    }

    pub(super) fn source_constructor(
        &self,
        node: NodeId,
        declaration: crate::DeclarationId,
        caller: &GenericSubstitution,
    ) -> Result<super::Constructor, CheckStop> {
        let constructor = *self
            .constructor_templates_by_declaration
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let template_index = match constructor {
            ConstructorTemplate::Struct { template }
            | ConstructorTemplate::Enum { template, .. } => template,
        };
        let template = self
            .nominal_templates
            .get(template_index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let substitution =
            self.nominal_generic_substitution(node, &template.generic_parameters, caller)?;
        let nominal = self
            .source_nominal_instance(template.declaration, &substitution)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        Ok(match constructor {
            ConstructorTemplate::Struct { .. } => super::Constructor::Struct(nominal),
            ConstructorTemplate::Enum { variant, .. } => {
                super::Constructor::Enum { nominal, variant }
            }
        })
    }

    pub(super) fn nominal_checkpoint(&self) -> usize {
        self.nominals.len()
    }

    pub(super) fn restore_nominal_checkpoint(
        &mut self,
        checkpoint: usize,
    ) -> Result<(), CheckStop> {
        if checkpoint > self.nominals.len() {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        self.nominals.truncate(checkpoint);
        self.nominal_nodes.truncate(checkpoint);
        self.nominal_states.truncate(checkpoint);
        self.source_nominal_instances.truncate(checkpoint);
        self.prelude_types.truncate(checkpoint);
        self.nominals_by_declaration.retain(|_, instances| {
            instances.retain(|instance| (instance.id.0 as usize) < checkpoint);
            !instances.is_empty()
        });
        self.prelude_nominals
            .retain(|_, id| (id.0 as usize) < checkpoint);
        self.box_nominals
            .retain(|_, id| (id.0 as usize) < checkpoint);
        self.arena_nominals
            .retain(|_, id| (id.0 as usize) < checkpoint);
        if self
            .arena_storage_nominal
            .is_some_and(|id| (id.0 as usize) >= checkpoint)
        {
            self.arena_storage_nominal = None;
        }
        self.system_nominals
            .retain(|_, id| (id.0 as usize) < checkpoint);
        Ok(())
    }
}
