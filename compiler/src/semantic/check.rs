mod borrows;
mod cleanup;
mod control;
mod expressions;
mod generics;
mod nominal_instances;
mod nominals;
mod requires;
mod support;
mod types;

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, ProductionV0_15, ResolvedSyntaxUnit, SemanticCompilerFailure,
    SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRuleV0_15,
    UnsupportedSemanticFeatureV0_15,
};

use super::model::{
    BindingId, CheckedConstant, CheckedConstantId, CheckedExpression, CheckedFunction, CheckedMode,
    CheckedNominal, CheckedParameter, CheckedProgramData, CheckedType, FunctionId, NominalId,
};
use super::tree::TreeView;
use super::{CheckStop, CheckedProgram};
use borrows::BorrowInfo;
use borrows::{AccessKind, ResolvedPlace};
use control::{ControlCounters, ControlScope};
use generics::{GenericParameter, GenericSubstitution};

#[derive(Clone)]
struct ParameterSignature {
    declaration: DeclarationId,
    name: String,
    mode: CheckedMode,
    ty: CheckedType,
}

#[derive(Clone)]
struct FunctionSignature {
    id: FunctionId,
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    symbol: String,
    region_parameters: Vec<DeclarationId>,
    parameters: Vec<ParameterSignature>,
    result_mode: CheckedMode,
    result: CheckedType,
    effects_node: NodeId,
    declared_effects: EffectSet,
    substitution: GenericSubstitution,
}

#[derive(Clone)]
struct FunctionTemplate {
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    generic_parameters: Vec<GenericParameter>,
}

#[derive(Clone)]
struct NominalTemplate {
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    role: DeclarationRole,
    generic_parameters: Vec<GenericParameter>,
}

#[derive(Clone)]
struct NominalInstance {
    id: NominalId,
    substitution: GenericSubstitution,
}

#[derive(Clone, Copy)]
enum ConstructorTemplate {
    Struct { template: usize },
    Enum { template: usize, variant: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LocalBinding {
    binding: BindingId,
    declaration: DeclarationId,
    mode: CheckedMode,
    ty: CheckedType,
    live: bool,
    loop_depth: usize,
    borrow: Option<BorrowInfo>,
}

#[derive(Clone, Copy)]
enum Constructor {
    Struct(NominalId),
    Enum { nominal: NominalId, variant: u32 },
}

struct TypedExpression {
    expression: CheckedExpression,
    mode: CheckedMode,
    borrow: Option<BorrowInfo>,
    holder: Option<DeclarationId>,
    effects: EffectSet,
    accesses: Vec<PlaceAccess>,
}

#[derive(Clone)]
struct PlaceAccess {
    place: ResolvedPlace,
    kind: AccessKind,
}

impl TypedExpression {
    fn owned(expression: CheckedExpression, effects: EffectSet) -> Self {
        Self {
            expression,
            mode: CheckedMode::Own,
            borrow: None,
            holder: None,
            effects,
            accesses: Vec::new(),
        }
    }

    fn owned_with_access(
        expression: CheckedExpression,
        effects: EffectSet,
        place: ResolvedPlace,
        kind: AccessKind,
    ) -> Self {
        Self {
            expression,
            mode: CheckedMode::Own,
            borrow: None,
            holder: None,
            effects,
            accesses: vec![PlaceAccess { place, kind }],
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct EffectSet {
    reads: Vec<DeclarationId>,
    writes: Vec<DeclarationId>,
    allocates_heap: bool,
    traps: bool,
}

impl EffectSet {
    const NONE: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: false,
        traps: false,
    };
    const TRAPS: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: false,
        traps: true,
    };
    const ALLOCATES_HEAP_AND_TRAPS: Self = Self {
        reads: Vec::new(),
        writes: Vec::new(),
        allocates_heap: true,
        traps: true,
    };

    fn union(mut self, other: Self) -> Self {
        for region in other.reads {
            self.add_read(region);
        }
        for region in other.writes {
            self.add_write(region);
        }
        self.allocates_heap |= other.allocates_heap;
        self.traps |= other.traps;
        self
    }

    fn add_read(&mut self, region: DeclarationId) {
        if !self.reads.contains(&region) {
            self.reads.push(region);
            self.reads.sort_unstable();
        }
    }

    fn add_write(&mut self, region: DeclarationId) {
        if !self.writes.contains(&region) {
            self.writes.push(region);
            self.writes.sort_unstable();
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PreludeType {
    Option(CheckedType),
    Result(CheckedType, CheckedType),
    Overflow,
    DivError,
    NarrowError,
}

struct Checker<'unit, 'classified, 'lexed, 'source> {
    resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    tree: TreeView<'unit, 'classified, 'lexed, 'source>,
    nominals: Vec<CheckedNominal>,
    nominal_nodes: Vec<Option<NodeId>>,
    nominal_states: Vec<u8>,
    source_nominal_instances: Vec<Option<(usize, GenericSubstitution)>>,
    prelude_nominals: HashMap<PreludeType, NominalId>,
    prelude_types: Vec<Option<PreludeType>>,
    nominal_templates: Vec<NominalTemplate>,
    nominal_templates_by_declaration: HashMap<DeclarationId, usize>,
    nominals_by_declaration: HashMap<DeclarationId, Vec<NominalInstance>>,
    constructor_templates_by_declaration: HashMap<DeclarationId, ConstructorTemplate>,
    signatures: Vec<FunctionSignature>,
    function_templates: Vec<FunctionTemplate>,
    templates_by_declaration: HashMap<DeclarationId, usize>,
    functions_by_declaration: HashMap<DeclarationId, Vec<FunctionId>>,
    constants: HashMap<DeclarationId, CheckedConstantId>,
    checked_constants: Vec<CheckedConstant>,
}

/// Checks the currently implemented exact-v0.15 semantic family.
///
/// Unsupported language families remain explicit compiler capability results;
/// only a proved numbered-rule violation becomes [`SemanticOutcome::SourceIssue`].
#[must_use]
pub fn check_semantics_v0_15<'classified, 'lexed, 'source>(
    resolved: ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
) -> SemanticOutcome<'classified, 'lexed, 'source> {
    let result = Checker::new(&resolved).and_then(|mut checker| checker.check_program());
    match result {
        Ok(data) => SemanticOutcome::Complete(Box::new(CheckedProgram {
            _resolved: resolved,
            data,
        })),
        Err(CheckStop::Issue(issue)) => SemanticOutcome::SourceIssue { issue },
        Err(CheckStop::Unsupported(unsupported)) => SemanticOutcome::Unsupported { unsupported },
        Err(CheckStop::Compiler(failure)) => SemanticOutcome::CompilerFailure { failure },
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    fn new(
        resolved: &'unit ResolvedSyntaxUnit<'classified, 'lexed, 'source>,
    ) -> Result<Self, CheckStop> {
        Ok(Self {
            resolved,
            tree: TreeView::new(resolved)?,
            nominals: Vec::new(),
            nominal_nodes: Vec::new(),
            nominal_states: Vec::new(),
            source_nominal_instances: Vec::new(),
            prelude_nominals: HashMap::new(),
            prelude_types: Vec::new(),
            nominal_templates: Vec::new(),
            nominal_templates_by_declaration: HashMap::new(),
            nominals_by_declaration: HashMap::new(),
            constructor_templates_by_declaration: HashMap::new(),
            signatures: Vec::new(),
            function_templates: Vec::new(),
            templates_by_declaration: HashMap::new(),
            functions_by_declaration: HashMap::new(),
            constants: HashMap::new(),
            checked_constants: Vec::new(),
        })
    }

    fn check_program(&mut self) -> Result<CheckedProgramData, CheckStop> {
        let items = self.item_declarations()?;
        self.check_main_header(&items)?;
        self.reject_unimplemented_items(&items)?;
        self.declare_nominals(&items)?;
        self.collect_constants(&items)?;
        self.complete_nominals()?;
        self.collect_function_signatures(&items)?;
        let main = self.main_id()?;

        let mut functions = Vec::with_capacity(self.signatures.len());
        for index in 0..self.signatures.len() {
            functions.push(self.check_function(index)?);
        }
        Ok(CheckedProgramData {
            nominals: self.nominals.clone(),
            constants: self.checked_constants.clone(),
            functions,
            main,
        })
    }

    fn item_declarations(&self) -> Result<Vec<NodeId>, CheckStop> {
        let mut declarations = Vec::new();
        for item in self.tree.children(self.tree.root())? {
            if self.tree.production(*item)? != ProductionV0_15::Item {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            declarations.push(self.tree.only_child(*item)?);
        }
        Ok(declarations)
    }

    fn reject_unimplemented_items(&self, items: &[NodeId]) -> Result<(), CheckStop> {
        for item in items {
            let feature = match self.tree.production(*item)? {
                ProductionV0_15::FnDecl
                | ProductionV0_15::ConstDecl
                | ProductionV0_15::StructDecl
                | ProductionV0_15::EnumDecl => continue,
                ProductionV0_15::ContractDecl | ProductionV0_15::ConformDecl => {
                    UnsupportedSemanticFeatureV0_15::ContractsAndConformances
                }
                _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
            };
            return self.unsupported(feature, *item);
        }
        Ok(())
    }

    fn collect_function_signatures(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        self.collect_function_templates(items)?;
        self.collect_concrete_function_signatures()
    }

    fn collect_constants(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        for node in items.iter().copied().filter(|node| {
            self.tree
                .production(*node)
                .is_ok_and(|production| production == ProductionV0_15::ConstDecl)
        }) {
            let declaration = self.declaration_at(node, DeclarationRole::NamedConst)?;
            let declaration_id = declaration.id();
            let name = declaration.spelling().to_owned();
            let ty_node = self
                .tree
                .first_child_with(node, ProductionV0_15::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let ty = self.parse_const_type(ty_node)?;
            let value_node = self
                .tree
                .first_child_with(node, ProductionV0_15::Cvalue)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let value = self.parse_const_value(value_node, ty)?;
            let id = CheckedConstantId(
                u32::try_from(self.checked_constants.len())
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            self.checked_constants.push(CheckedConstant {
                id,
                name,
                ty,
                value,
            });
            self.constants.insert(declaration_id, id);
        }
        Ok(())
    }

    fn check_main_header(&self, items: &[NodeId]) -> Result<(), CheckStop> {
        let mut main = None;
        for node in items.iter().copied().filter(|node| {
            self.tree
                .production(*node)
                .is_ok_and(|production| production == ProductionV0_15::FnDecl)
        }) {
            if self
                .declaration_at(node, DeclarationRole::Function)?
                .spelling()
                == "main"
            {
                main = Some(node);
                break;
            }
        }
        let Some(node) = main else {
            return Err(CheckStop::Issue(SemanticIssue {
                rule: SemanticRuleV0_15::Fn7,
                location: SemanticLocation::BundleRoot(
                    self.resolved.syntax().root_extent().to_vec(),
                ),
                kind: SemanticIssueKind::MissingMain,
            }));
        };

        let generics = self
            .tree
            .first_child_with(node, ProductionV0_15::Generics)?;
        let regions = self
            .tree
            .first_child_with(node, ProductionV0_15::RegionParams)?;
        let parameters = self
            .tree
            .first_child_with(node, ProductionV0_15::ParamList)?;
        let rtype = self
            .tree
            .first_child_with(node, ProductionV0_15::Rtype)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mode = self
            .tree
            .first_child_with(rtype, ProductionV0_15::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let ty = self
            .tree
            .first_child_with(rtype, ProductionV0_15::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let effects = self
            .tree
            .first_child_with(node, ProductionV0_15::Effects)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if generics.is_some()
            || regions.is_some()
            || parameters.is_some()
            || !self.has_fixed(mode, crate::FixedTerminalV0_15::Own)?
            || !self.has_fixed(ty, crate::FixedTerminalV0_15::Unit)?
            || !self.main_effects_allowed(effects)?
        {
            return self.issue_node(SemanticRuleV0_15::Fn7, node, SemanticIssueKind::InvalidMain);
        }
        Ok(())
    }

    fn main_effects_allowed(&self, effects: NodeId) -> Result<bool, CheckStop> {
        if self.has_fixed(effects, crate::FixedTerminalV0_15::Pure)? {
            return Ok(true);
        }
        let effects = self.tree.children_with(effects, ProductionV0_15::Effect)?;
        let spellings = effects
            .iter()
            .map(|effect| self.tree.direct_spelling(*effect))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(matches!(
            spellings.as_slice(),
            [one] if one == b"traps" || one == b"allocates(heap)"
        ) || matches!(
            spellings.as_slice(),
            [first, second] if first == b"allocates(heap)" && second == b"traps"
        ))
    }

    fn main_id(&self) -> Result<FunctionId, CheckStop> {
        self.signatures
            .iter()
            .find(|signature| signature.name == "main")
            .map(|signature| signature.id)
            .ok_or_else(|| SemanticCompilerFailure::InvalidResolution.into())
    }

    fn check_function(&self, index: usize) -> Result<CheckedFunction, CheckStop> {
        let signature = self
            .signatures
            .get(index)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        self.check_function_signature(signature)
    }

    fn check_function_signature(
        &self,
        signature: &FunctionSignature,
    ) -> Result<CheckedFunction, CheckStop> {
        let mut bindings = HashMap::new();
        let mut parameters = Vec::with_capacity(signature.parameters.len());
        let mut next_binding = 0_u32;
        let mut next_loop = 0_u32;
        for parameter in &signature.parameters {
            let binding = BindingId(next_binding);
            next_binding = next_binding
                .checked_add(1)
                .ok_or(SemanticCompilerFailure::CounterOverflow)?;
            bindings.insert(
                parameter.declaration,
                LocalBinding {
                    binding,
                    declaration: parameter.declaration,
                    mode: parameter.mode,
                    ty: parameter.ty,
                    live: true,
                    loop_depth: 0,
                    borrow: self.parameter_borrow(parameter),
                },
            );
            parameters.push(CheckedParameter {
                name: parameter.name.clone(),
                binding,
                mode: parameter.mode,
                ty: parameter.ty,
            });
        }

        let mut counters = ControlCounters {
            next_binding: &mut next_binding,
            next_loop: &mut next_loop,
        };
        let parameter_bindings = bindings.clone();
        let requires = if let Some(node) = self
            .tree
            .first_child_with(signature.node, ProductionV0_15::RequiresBlock)?
        {
            let mut requires_bindings = parameter_bindings.clone();
            Some(self.check_requires(signature, node, &mut requires_bindings, &mut counters)?)
        } else {
            None
        };

        bindings = parameter_bindings;
        let statements = self
            .tree
            .children_with(signature.node, ProductionV0_15::Stmt)?;
        let checked = self.check_block(
            signature,
            &statements,
            &mut bindings,
            &mut counters,
            ControlScope {
                loops: &[],
                give_context: None,
            },
        )?;
        if checked.can_continue {
            return Err(CheckStop::Issue(SemanticIssue {
                rule: SemanticRuleV0_15::Fn1,
                location: SemanticLocation::SourceNode(
                    self.tree.path(signature.node)?.clone(),
                    self.tree.closing_brace_coordinate(signature.node)?,
                ),
                kind: SemanticIssueKind::FunctionFallthrough,
            }));
        }
        let effects = requires.as_ref().map_or_else(
            || checked.effects.clone(),
            |prologue| prologue.effects.clone().union(checked.effects.clone()),
        );
        if effects != signature.declared_effects {
            return self.issue_node(
                SemanticRuleV0_15::Eff2,
                signature.effects_node,
                SemanticIssueKind::EffectMismatch,
            );
        }
        Ok(CheckedFunction {
            id: signature.id,
            declaration: signature.declaration,
            name: signature.name.clone(),
            symbol: signature.symbol.clone(),
            parameters,
            result_mode: signature.result_mode,
            result: signature.result,
            declared_traps: signature.declared_effects.traps,
            declared_allocates_heap: signature.declared_effects.allocates_heap,
            requires: requires
                .map(|prologue| prologue.statements)
                .unwrap_or_default(),
            body: checked.statements,
        })
    }
}
