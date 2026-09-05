use std::collections::HashSet;

use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    DeclarationClass, DeclarationRole, DependentDeclarationRole, LexicalUseRole,
    PreludeDeclarationId, Production, ResolvedTarget, SemanticCompilerFailure, SemanticIssueKind,
    SemanticRule,
};

use super::super::model::{
    CheckedConstructor, CheckedElement, CheckedField, CheckedFlatElement, CheckedNominal,
    CheckedNominalKind, CheckedNumericType, CheckedType, CheckedVariant, NominalId,
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
            if self.nominal_templates[index].generic_parameters.is_empty()
                && self.nominal_templates[index].region_parameters.is_empty()
            {
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
            if (template.generic_parameters.is_empty() && template.region_parameters.is_empty())
                || self.postcondition_declaration_unavailable(template.declaration)
            {
                continue;
            }
            let checkpoint = self.nominal_checkpoint();
            let result = (|| {
                let substitution = self.symbolic_nominal_substitution(
                    &template.generic_parameters,
                    &template.region_parameters,
                )?;
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
        let linear = self.declaration_is_linear(node)?;
        // [S20, GRAM-2] a nominal's `region_params` are its own, exactly as a
        // function's are, and each is a component of its type name.
        let region_parameters = self.parse_region_parameters(node)?;
        let template = NominalTemplate {
            declaration: declaration_id,
            node,
            name: declaration.spelling().to_owned(),
            role,
            generic_parameters,
            region_parameters,
            linear,
            constructors: Vec::new(),
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
        self.validate_nominal_templates()?;
        self.validate_linear_modifiers()
    }

    /// [PROV-6] the `linear` modifier is admitted only on a nominal [OWN-1]
    /// classifies as affine; a tag-only enum is copy and the modifier would
    /// mark a value the language duplicates.
    fn validate_linear_modifiers(&self) -> Result<(), CheckStop> {
        for index in 0..self.nominals.len() {
            let id = NominalId(
                u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            let Some(node) = self.nominal_nodes.get(index).copied().flatten() else {
                continue;
            };
            self.check_linear_modifier_admission(id, node)?;
        }
        Ok(())
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
            self.ensure_nominals_in_node(function, substitution)?;
            return self.ensure_result_list_nominal(function, substitution);
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
        self.ensure_result_list_nominal(function, substitution)?;
        self.reject_recursive_nominal_layouts()
    }

    /// Interns the compiler-owned result-list nominal of a `fn_decl` that
    /// writes an ordered result list [GRAM-2, CALL-4].
    ///
    /// It is the callable's result type, so every pre-scan that precedes
    /// `build_function_signature` runs this one walk; a single-result
    /// declaration has no list and nothing is interned.
    pub(super) fn ensure_result_list_nominal(
        &mut self,
        function: NodeId,
        substitution: &GenericSubstitution,
    ) -> Result<(), CheckStop> {
        if self
            .tree
            .children_with(function, Production::ResultBinding)?
            .len()
            < 2
        {
            return Ok(());
        }
        let Some(results) = self.result_list_fields(function, substitution)? else {
            return Ok(());
        };
        self.intern_result_list_nominal(&results).map(|_| ())
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
        // [GRAM-2] a `fn_decl` writes one result or an ordered result list;
        // every ordinal's `rtype` is a signature type, and a list also needs
        // the compiler-owned nominal that carries the ordinals [CALL-4].
        let result_bindings = self
            .tree
            .children_with(function, Production::ResultBinding)?;
        if result_bindings.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        for binding in &result_bindings {
            let result = self
                .tree
                .first_child_with(*binding, Production::Rtype)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.ensure_nominals_in_node(result, substitution)?;
        }
        self.ensure_result_list_nominal(function, substitution)
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
            let region = self.type_region(node)?;
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
                    &template.region_parameters,
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
        // [FORM-8] a construct whose field operands determine a region
        // parameter does not write it, so this pre-scan cannot read the
        // instance off the written list: it is formed while the operands are
        // checked, and interned then through the deferred-nominal route.
        if self
            .constructor_shape(declaration)?
            .is_some_and(|site| site.shape.determining_field.iter().any(Option::is_some))
        {
            return Ok(());
        }
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
        let substitution = self.nominal_generic_substitution(
            node,
            &template.generic_parameters,
            &template.region_parameters,
            caller,
        )?;
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
            linear: template.linear,
        });
        self.nominals_by_declaration
            .entry(template.declaration)
            .or_default()
            .push(NominalInstance { id, substitution });
        Ok(id)
    }

    /// [TYPE-6] whether this nominal is an instance of that source
    /// declaration.
    pub(super) fn nominal_instantiates(
        &self,
        nominal: crate::NominalId,
        declaration: crate::DeclarationId,
    ) -> Result<bool, CheckStop> {
        let Some(template) = self.nominal_templates_by_declaration.get(&declaration) else {
            return Ok(false);
        };
        Ok(self
            .source_nominal_instances
            .get(nominal.0 as usize)
            .and_then(|entry| entry.as_ref())
            .is_some_and(|(index, _)| index == template))
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
            let region_parameters = self.nominal_templates[template_index]
                .region_parameters
                .clone();
            if parameters.is_empty() && region_parameters.is_empty() {
                continue;
            }
            let substitution =
                self.symbolic_nominal_substitution(&parameters, &region_parameters)?;
            let id = self.ensure_source_nominal_instance(template_index, substitution)?;
            // [FORM-8] the one place a declaration's fields can be read
            // against its own region parameters: this instance carries each
            // region parameter as its own argument, so a field type naming
            // one is visibly that parameter and not some caller's actual.
            if !region_parameters.is_empty() {
                let constructors = self.constructor_shapes(id, &region_parameters)?;
                self.nominal_templates
                    .get_mut(template_index)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .constructors = constructors;
            }
        }
        self.reject_recursive_nominal_layouts()?;
        self.restore_nominal_checkpoint(checkpoint)
    }

    /// One declaration's constructors, read off its symbolic instance
    /// [FORM-8].
    ///
    /// A field determines a region parameter exactly when its declared type
    /// names that region — the same relation [FORM-8] uses at a call, where a
    /// parameter whose type names a formal region determines it from the
    /// actual. Where two fields name one region parameter the first decides
    /// it and the rest are the ordinary [TYPE-5] equality against the formed
    /// instance, exactly as a call's second store operand is.
    fn constructor_shapes(
        &self,
        id: NominalId,
        region_parameters: &[crate::DeclarationId],
    ) -> Result<Vec<super::ConstructorShape>, CheckStop> {
        let variants: Vec<&[super::super::model::CheckedField]> = match &self.nominal(id)?.kind {
            CheckedNominalKind::Struct { fields } => vec![fields.as_slice()],
            CheckedNominalKind::Enum { variants } => variants
                .iter()
                .map(|variant| variant.fields.as_slice())
                .collect(),
            _ => return Ok(Vec::new()),
        };
        let mut constructors = Vec::with_capacity(variants.len());
        for fields in variants {
            let mut determining_field = vec![None; region_parameters.len()];
            for (index, field) in fields.iter().enumerate() {
                let Some(region) = self.written_type_region(field.ty)? else {
                    continue;
                };
                let Some(slot) = region_parameters
                    .iter()
                    .position(|parameter| *parameter == region)
                else {
                    continue;
                };
                if determining_field[slot].is_none() {
                    determining_field[slot] = Some(index);
                }
            }
            constructors.push(super::ConstructorShape {
                fields: fields.iter().map(|field| field.name.clone()).collect(),
                determining_field,
            });
        }
        Ok(constructors)
    }

    /// The constructor shape one `construct` names, when its declaration
    /// carries `region_params` [FORM-8]; `None` for every declaration that
    /// does not, where a construct writes no region argument at all and the
    /// instance is formed from the written list alone.
    pub(super) fn constructor_shape(
        &self,
        declaration: crate::DeclarationId,
    ) -> Result<Option<super::ConstructorSite>, CheckStop> {
        let constructor = *self
            .constructor_templates_by_declaration
            .get(&declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let (template_index, variant) = match constructor {
            ConstructorTemplate::Struct { template } => (template, None),
            ConstructorTemplate::Enum { template, variant } => (template, Some(variant)),
        };
        let template = self
            .nominal_templates
            .get(template_index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let Some(shape) = template
            .constructors
            .get(variant.unwrap_or(0) as usize)
            .cloned()
        else {
            return Ok(None);
        };
        Ok(Some(super::ConstructorSite {
            template: template_index,
            variant,
            generic_parameters: template.generic_parameters.clone(),
            region_parameters: template.region_parameters.clone(),
            shape,
        }))
    }

    /// The instance one `construct` names, at the regions its own field
    /// operands determined and the ones it wrote [FORM-8, TYPE-5].
    ///
    /// No position of the writer's text need spell that instance — a
    /// construct whose every region parameter a field determines writes no
    /// region argument at all — so the interning pass cannot have found it,
    /// and a miss is the ordinary deferred-nominal report the driver repairs.
    pub(super) fn constructed_nominal(
        &self,
        node: NodeId,
        site: &super::ConstructorSite,
        determined: &[(crate::DeclarationId, crate::DeclarationId)],
        caller: &GenericSubstitution,
    ) -> Result<NominalId, CheckStop> {
        let substitution = self.nominal_generic_substitution_with(
            node,
            &site.generic_parameters,
            &site.region_parameters,
            determined,
            caller,
        )?;
        let declaration = self
            .nominal_templates
            .get(site.template)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .declaration;
        if let Some(existing) = self.source_nominal_instance(declaration, &substitution) {
            return Ok(existing);
        }
        self.pending_nominals
            .borrow_mut()
            .push(super::PendingNominal::SourceInstance {
                template: site.template,
                substitution,
            });
        Err(CheckStop::DeferredNominal)
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
            self.reject_confined_type_without_store(parsed, ty)?;
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
                    self.reject_confined_type_without_store(parsed, ty)?;
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

    /// [S20] where each nominal instance's region axis leaves the program.
    ///
    /// Two instances of one declaration whose type and const arguments agree
    /// and whose regions differ are two checked types and one representation:
    /// a region names a store for the proof and nothing at run time, exactly
    /// as `Vector<'a, T>` and `Vector<'b, T>` are one lowered run. The first
    /// such instance is the one they all lower as, so a callee's own
    /// formal-region instance and a caller's actual-region instance meet as
    /// one IR nominal at the boundary between them.
    pub(super) fn nominal_lowering_aliases(&self) -> Result<Vec<NominalId>, CheckStop> {
        let mut aliases = Vec::with_capacity(self.nominals.len());
        for index in 0..self.nominals.len() {
            let id = NominalId(
                u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            let mut alias = id;
            for earlier in 0..index {
                let candidate = NominalId(
                    u32::try_from(earlier).map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                );
                if self.nominals_differ_only_in_region(id, candidate)? {
                    alias = candidate;
                    break;
                }
            }
            aliases.push(alias);
        }
        Ok(aliases)
    }

    /// The actual one formal region of a call denotes, or the region itself
    /// when this call substitutes nothing for it [FORM-8].
    fn substituted_region(
        regions: &[(crate::DeclarationId, crate::DeclarationId)],
        region: crate::DeclarationId,
    ) -> crate::DeclarationId {
        regions
            .iter()
            .find_map(|(formal, actual)| (*formal == region).then_some(*actual))
            .unwrap_or(region)
    }

    /// One checked type with each formal region replaced by the actual a call
    /// determined for it [FN-2, FORM-8, PROV-1].
    ///
    /// A call's region arguments are fixed by its operands and by the `::`
    /// members [FORM-8] leaves it to write, and [FN-2] substitutes them into
    /// every position of the callee's signature — a result position exactly as
    /// much as a parameter position. Two calls of one declaration at two
    /// stores therefore hand back two types, which is what [PROV-1]'s
    /// invariant store region means where the value is produced rather than
    /// consumed.
    ///
    /// The walk is structural and closed over the checked type domain, so a
    /// region is reached wherever it occurs: under `Option` and `Result`, in
    /// a source nominal's own instance and in that instance's type arguments,
    /// in a run's element position, and in the compiler-owned result-list
    /// nominal a multi-result callable hands back [CALL-4]. A run's release
    /// class is a function of its region's declaration [PROV-6], so it is
    /// re-read from the substituted region rather than carried across.
    ///
    /// An instance the substitution names may exist nowhere in the caller's
    /// written text — no position of `let held = make(store: &uniq one);`
    /// spells the result's type — so a miss is the ordinary deferred-nominal
    /// report the driver repairs, exactly as a derived `box<T>` is.
    pub(super) fn substitute_type_regions(
        &self,
        ty: CheckedType,
        regions: &[(crate::DeclarationId, crate::DeclarationId)],
    ) -> Result<CheckedType, CheckStop> {
        if regions.is_empty() {
            return Ok(ty);
        }
        Ok(match ty {
            CheckedType::Unit
            | CheckedType::Bool
            | CheckedType::Integer(_)
            | CheckedType::Float(_)
            | CheckedType::Generic(_)
            | CheckedType::GenericInt(_)
            | CheckedType::GenericFloat(_)
            | CheckedType::Array { .. }
            | CheckedType::Buffer { .. } => ty,
            CheckedType::Nominal(id) => self.substitute_nominal_regions(id, regions)?,
            CheckedType::Slice {
                region,
                element,
                strength,
            } => CheckedType::Slice {
                region: Self::substituted_region(regions, region),
                element,
                strength,
            },
            CheckedType::FixedVector { element, length } => CheckedType::FixedVector {
                element: self.substitute_element_regions(element, regions)?,
                length,
            },
            CheckedType::Vector {
                region, element, ..
            } => {
                let region = Self::substituted_region(regions, region);
                CheckedType::Vector {
                    region,
                    element,
                    release: self.vector_release_class(region)?,
                }
            }
            CheckedType::Heap { region } => CheckedType::Heap {
                region: Self::substituted_region(regions, region),
            },
            CheckedType::Extent {
                region,
                bytes,
                align,
            } => CheckedType::Extent {
                region: Self::substituted_region(regions, region),
                bytes,
                align,
            },
        })
    }

    /// One slot's content with the same substitution [BLK-1].
    fn substitute_element_regions(
        &self,
        element: CheckedElement,
        regions: &[(crate::DeclarationId, crate::DeclarationId)],
    ) -> Result<CheckedElement, CheckStop> {
        Ok(match element {
            CheckedElement::Flat(_) | CheckedElement::FixedVector { .. } => element,
            CheckedElement::Vector {
                region, element, ..
            } => {
                let region = Self::substituted_region(regions, region);
                CheckedElement::Vector {
                    region,
                    element,
                    release: self.vector_release_class(region)?,
                }
            }
        })
    }

    /// One nominal instance with the same substitution, reported as a
    /// deferred nominal when the instance it names is not interned yet.
    fn substitute_nominal_regions(
        &self,
        id: NominalId,
        regions: &[(crate::DeclarationId, crate::DeclarationId)],
    ) -> Result<CheckedType, CheckStop> {
        if let Some(prelude) = self.prelude_type(id) {
            let substituted = match prelude {
                PreludeType::Option(value) => {
                    PreludeType::Option(self.substitute_type_regions(value, regions)?)
                }
                PreludeType::Result(ok, error) => PreludeType::Result(
                    self.substitute_type_regions(ok, regions)?,
                    self.substitute_type_regions(error, regions)?,
                ),
                PreludeType::Overflow | PreludeType::DivError | PreludeType::NarrowError => prelude,
            };
            if substituted == prelude {
                return Ok(CheckedType::Nominal(id));
            }
            return Ok(CheckedType::Nominal(self.prelude_nominal(substituted)?));
        }
        if let Some((template, instance)) = self.source_nominal_instance_entry(id)? {
            let mut changed = false;
            let mut bindings = Vec::with_capacity(instance.entries().len());
            for (declaration, argument) in instance.entries() {
                bindings.push((
                    *declaration,
                    match argument {
                        super::generics::GenericArgument::Type(ty) => {
                            let substituted = self.substitute_type_regions(*ty, regions)?;
                            changed |= substituted != *ty;
                            super::generics::GenericArgument::Type(substituted)
                        }
                        super::generics::GenericArgument::Const(value) => {
                            super::generics::GenericArgument::Const(*value)
                        }
                    },
                ));
            }
            let mut axis = Vec::with_capacity(instance.region_arguments().len());
            for (formal, actual) in instance.region_arguments() {
                let substituted = Self::substituted_region(regions, *actual);
                changed |= substituted != *actual;
                axis.push((*formal, substituted));
            }
            if !changed {
                return Ok(CheckedType::Nominal(id));
            }
            let target = GenericSubstitution::from_bindings(bindings)?.with_regions(axis);
            let declaration = self
                .nominal_templates
                .get(template)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .declaration;
            if let Some(existing) = self.source_nominal_instance(declaration, &target) {
                return Ok(CheckedType::Nominal(existing));
            }
            self.pending_nominals
                .borrow_mut()
                .push(super::PendingNominal::SourceInstance {
                    template,
                    substitution: target,
                });
            return Err(CheckStop::DeferredNominal);
        }
        if let Some((results, _)) = self
            .result_list_nominals
            .iter()
            .find(|(_, candidate)| **candidate == id)
        {
            let mut changed = false;
            let mut substituted = Vec::with_capacity(results.len());
            for (name, ty) in results {
                let ordinal = self.substitute_type_regions(*ty, regions)?;
                changed |= ordinal != *ty;
                substituted.push((name.clone(), ordinal));
            }
            if !changed {
                return Ok(CheckedType::Nominal(id));
            }
            let Some(existing) = self.result_list_nominal(&substituted) else {
                self.pending_nominals
                    .borrow_mut()
                    .push(super::PendingNominal::ResultList(substituted));
                return Err(CheckStop::DeferredNominal);
            };
            return Ok(CheckedType::Nominal(existing));
        }
        match &self.nominal(id)?.kind {
            CheckedNominalKind::Box { referent, .. } => {
                let substituted = self.substitute_type_regions(*referent, regions)?;
                if substituted == *referent {
                    return Ok(CheckedType::Nominal(id));
                }
                let Some(existing) = self.box_nominals.get(&substituted).copied() else {
                    self.pending_nominals
                        .borrow_mut()
                        .push(super::PendingNominal::Box(substituted));
                    return Err(CheckStop::DeferredNominal);
                };
                Ok(CheckedType::Nominal(existing))
            }
            CheckedNominalKind::Arena { region, content } => {
                let substituted_region = Self::substituted_region(regions, *region);
                let substituted_content = self.substitute_type_regions(*content, regions)?;
                if substituted_region == *region && substituted_content == *content {
                    return Ok(CheckedType::Nominal(id));
                }
                let key = (substituted_region, substituted_content);
                let Some(existing) = self.arena_nominals.get(&key).copied() else {
                    self.pending_nominals
                        .borrow_mut()
                        .push(super::PendingNominal::Arena(key.0, key.1));
                    return Err(CheckStop::DeferredNominal);
                };
                Ok(CheckedType::Nominal(existing))
            }
            CheckedNominalKind::Struct { .. }
            | CheckedNominalKind::Enum { .. }
            | CheckedNominalKind::ArenaStorage
            | CheckedNominalKind::SystemResource { .. } => Ok(CheckedType::Nominal(id)),
        }
    }

    /// [S20] whether two nominals are two names for one representation that
    /// differ in their regions alone.
    ///
    /// The same name shape — one source declaration at two regions, one
    /// prelude shape over two region-blind-equal arguments, one result list
    /// [CALL-4] with the same ordinal names, one `box` or one `arena`
    /// [STOR-2] — and the same lowered content: a region names a store for
    /// the proof, so two such nominals are two checked types and one IR
    /// nominal. The content comparison is what keeps a difference the run
    /// time *can* see out of the relation — a run's release class is read off
    /// its region's own declaration [PROV-6], so two instances whose classes
    /// differ are two representations and are not related here.
    ///
    /// The relation reaches beyond a source instance because a call's region
    /// substitution does: a callee returning `own Option<BlockPool<'s>>`
    /// hands its caller an `Option<BlockPool<'a>>` [FN-2], and a multi-result
    /// callee hands back its result-list nominal with every ordinal
    /// substituted, so those two classes meet at the same boundary a source
    /// instance does.
    pub(super) fn nominals_differ_only_in_region(
        &self,
        left: NominalId,
        right: NominalId,
    ) -> Result<bool, CheckStop> {
        if left == right {
            return Ok(false);
        }
        self.nominals_are_region_blind_equal(left, right, 0)
    }

    fn nominals_are_region_blind_equal(
        &self,
        left: NominalId,
        right: NominalId,
        depth: usize,
    ) -> Result<bool, CheckStop> {
        if left == right {
            return Ok(true);
        }
        if depth > 16 {
            return Ok(false);
        }
        // [STOR-2] a box and an arena carry their whole content in the kind
        // rather than in fields, so the content comparison below has nothing
        // to read for them.
        match (&self.nominal(left)?.kind, &self.nominal(right)?.kind) {
            (
                CheckedNominalKind::Box { referent: left, .. },
                CheckedNominalKind::Box { referent: right, .. },
            )
            | (
                CheckedNominalKind::Arena { content: left, .. },
                CheckedNominalKind::Arena { content: right, .. },
            ) => {
                return self.types_are_region_blind_equal(*left, *right, depth.saturating_add(1));
            }
            (CheckedNominalKind::Box { .. } | CheckedNominalKind::Arena { .. }, _)
            | (_, CheckedNominalKind::Box { .. } | CheckedNominalKind::Arena { .. }) => {
                return Ok(false);
            }
            _ => {}
        }
        if !self.nominal_names_are_region_blind_equal(left, right, depth)? {
            return Ok(false);
        }
        self.nominal_content_is_region_blind_equal(left, right, depth)
    }

    /// Whether two nominals name the same shape once every region is erased:
    /// the half of the relation that is about identity rather than layout.
    fn nominal_names_are_region_blind_equal(
        &self,
        left: NominalId,
        right: NominalId,
        depth: usize,
    ) -> Result<bool, CheckStop> {
        let (left_source, right_source) = (
            self.source_nominal_instance_entry(left)?,
            self.source_nominal_instance_entry(right)?,
        );
        if let (Some((left_template, left_instance)), Some((right_template, right_instance))) =
            (left_source, right_source)
        {
            return Ok(left_template == right_template
                && left_instance.entries() == right_instance.entries()
                && !left_instance.region_arguments().is_empty());
        }
        if left_source.is_some() || right_source.is_some() {
            return Ok(false);
        }
        let (left_prelude, right_prelude) = (self.prelude_type(left), self.prelude_type(right));
        if let (Some(left_prelude), Some(right_prelude)) = (left_prelude, right_prelude) {
            return self.prelude_types_are_region_blind_equal(left_prelude, right_prelude, depth);
        }
        if left_prelude.is_some() || right_prelude.is_some() {
            return Ok(false);
        }
        let (left_list, right_list) = (
            self.result_list_ordinal_names(left),
            self.result_list_ordinal_names(right),
        );
        if let (Some(left_list), Some(right_list)) = (&left_list, &right_list) {
            return Ok(left_list == right_list);
        }
        Ok(false)
    }

    /// One prelude instance's arguments, compared with every region erased.
    fn prelude_types_are_region_blind_equal(
        &self,
        left: PreludeType,
        right: PreludeType,
        depth: usize,
    ) -> Result<bool, CheckStop> {
        Ok(match (left, right) {
            (PreludeType::Option(left), PreludeType::Option(right)) => {
                self.types_are_region_blind_equal(left, right, depth.saturating_add(1))?
            }
            (
                PreludeType::Result(left_ok, left_error),
                PreludeType::Result(right_ok, right_error),
            ) => {
                self.types_are_region_blind_equal(left_ok, right_ok, depth.saturating_add(1))?
                    && self.types_are_region_blind_equal(
                        left_error,
                        right_error,
                        depth.saturating_add(1),
                    )?
            }
            (left, right) => left == right,
        })
    }

    /// The ordinal names of a compiler-owned result-list nominal [CALL-4],
    /// absent for every nominal that is not one.
    fn result_list_ordinal_names(&self, id: NominalId) -> Option<Vec<String>> {
        self.result_list_nominals
            .iter()
            .find(|(_, candidate)| **candidate == id)
            .map(|(results, _)| results.iter().map(|(name, _)| name.clone()).collect())
    }

    /// The two instances' fields or variant payloads, compared with every
    /// region erased and every region-derived datum kept [S20, PROV-6].
    fn nominal_content_is_region_blind_equal(
        &self,
        left: NominalId,
        right: NominalId,
        depth: usize,
    ) -> Result<bool, CheckStop> {
        if depth > 16 {
            return Ok(false);
        }
        let (left_nominal, right_nominal) = (self.nominal(left)?, self.nominal(right)?);
        if left_nominal.linear != right_nominal.linear {
            return Ok(false);
        }
        let (left_types, right_types) = (
            Self::nominal_content_types(&left_nominal.kind),
            Self::nominal_content_types(&right_nominal.kind),
        );
        let (Some(left_types), Some(right_types)) = (left_types, right_types) else {
            return Ok(false);
        };
        if left_types.len() != right_types.len() {
            return Ok(false);
        }
        for (left_type, right_type) in left_types.into_iter().zip(right_types) {
            if !self.types_are_region_blind_equal(left_type, right_type, depth)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn nominal_content_types(kind: &CheckedNominalKind) -> Option<Vec<CheckedType>> {
        match kind {
            CheckedNominalKind::Struct { fields } => {
                Some(fields.iter().map(|field| field.ty).collect())
            }
            CheckedNominalKind::Enum { variants } => Some(
                variants
                    .iter()
                    .flat_map(|variant| variant.fields.iter().map(|field| field.ty))
                    .collect(),
            ),
            _ => None,
        }
    }

    /// One slot's content [BLK-1], compared with every region erased and every
    /// region-derived datum kept [S20].
    fn elements_are_region_blind_equal(
        &self,
        left: CheckedElement,
        right: CheckedElement,
        depth: usize,
    ) -> Result<bool, CheckStop> {
        Ok(match (left, right) {
            (CheckedElement::Flat(left), CheckedElement::Flat(right)) => {
                self.flat_elements_are_region_blind_equal(left, right, depth)?
            }
            (
                CheckedElement::FixedVector {
                    element: left_element,
                    length: left_length,
                },
                CheckedElement::FixedVector {
                    element: right_element,
                    length: right_length,
                },
            ) => {
                left_length == right_length
                    && self.flat_elements_are_region_blind_equal(
                        left_element,
                        right_element,
                        depth,
                    )?
            }
            (
                CheckedElement::Vector {
                    element: left_element,
                    release: left_release,
                    ..
                },
                CheckedElement::Vector {
                    element: right_element,
                    release: right_release,
                    ..
                },
            ) => {
                left_release == right_release
                    && self.flat_elements_are_region_blind_equal(
                        left_element,
                        right_element,
                        depth,
                    )?
            }
            _ => false,
        })
    }

    fn flat_elements_are_region_blind_equal(
        &self,
        left: CheckedFlatElement,
        right: CheckedFlatElement,
        depth: usize,
    ) -> Result<bool, CheckStop> {
        Ok(match (left, right) {
            (CheckedFlatElement::Nominal(left), CheckedFlatElement::Nominal(right))
            | (
                CheckedFlatElement::TagOnlyNominal(left),
                CheckedFlatElement::TagOnlyNominal(right),
            ) => self.types_are_region_blind_equal(
                CheckedType::Nominal(left),
                CheckedType::Nominal(right),
                depth,
            )?,
            _ => left == right,
        })
    }

    fn types_are_region_blind_equal(
        &self,
        left: CheckedType,
        right: CheckedType,
        depth: usize,
    ) -> Result<bool, CheckStop> {
        Ok(match (left, right) {
            (CheckedType::Nominal(left), CheckedType::Nominal(right)) => {
                self.nominals_are_region_blind_equal(left, right, depth.saturating_add(1))?
            }
            (
                CheckedType::Vector {
                    element: left_element,
                    release: left_release,
                    ..
                },
                CheckedType::Vector {
                    element: right_element,
                    release: right_release,
                    ..
                },
            ) => {
                left_release == right_release
                    && self.elements_are_region_blind_equal(left_element, right_element, depth)?
            }
            (
                CheckedType::FixedVector {
                    element: left_element,
                    length: left_length,
                },
                CheckedType::FixedVector {
                    element: right_element,
                    length: right_length,
                },
            ) => {
                left_length == right_length
                    && self.elements_are_region_blind_equal(left_element, right_element, depth)?
            }
            (
                CheckedType::Slice {
                    element: left_element,
                    strength: left_strength,
                    ..
                },
                CheckedType::Slice {
                    element: right_element,
                    strength: right_strength,
                    ..
                },
            ) => left_element == right_element && left_strength == right_strength,
            (CheckedType::Heap { .. }, CheckedType::Heap { .. }) => true,
            (
                CheckedType::Extent {
                    bytes: left_bytes,
                    align: left_align,
                    ..
                },
                CheckedType::Extent {
                    bytes: right_bytes,
                    align: right_align,
                    ..
                },
            ) => left_bytes == right_bytes && left_align == right_align,
            _ => left == right,
        })
    }

    /// The template index and instance arguments of one source nominal, when
    /// it is a source declaration's instance rather than a compiler-owned
    /// nominal [S20].
    pub(super) fn source_nominal_instance_entry(
        &self,
        id: NominalId,
    ) -> Result<Option<(usize, &GenericSubstitution)>, CheckStop> {
        Ok(self
            .source_nominal_instances
            .get(id.0 as usize)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?
            .as_ref()
            .map(|(template, substitution)| (*template, substitution)))
    }

    /// One nominal instance's region axis [S20], absent for every nominal that
    /// is not a source declaration's instance.
    pub(super) fn nominal_region_axis(
        &self,
        id: NominalId,
    ) -> Result<Option<&[(crate::DeclarationId, crate::DeclarationId)]>, CheckStop> {
        Ok(self
            .source_nominal_instance_entry(id)?
            .map(|(_, substitution)| substitution.region_arguments()))
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
        let substitution = self.nominal_generic_substitution(
            node,
            &template.generic_parameters,
            &template.region_parameters,
            caller,
        )?;
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
        self.result_list_nominals
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
