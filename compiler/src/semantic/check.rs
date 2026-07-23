mod control;
mod expressions;
mod nominals;
mod support;
mod types;

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::{
    DeclarationId, DeclarationRole, ProductionV0_14, ResolvedSyntaxUnit, SemanticCompilerFailure,
    SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRuleV0_14,
    UnsupportedSemanticFeatureV0_14,
};

use super::model::{
    BindingId, CheckedExpression, CheckedFunction, CheckedNominal, CheckedParameter,
    CheckedProgramData, CheckedType, CheckedValue, FunctionId, NominalId,
};
use super::tree::TreeView;
use super::{CheckStop, CheckedProgram};
use control::{ControlCounters, ControlScope};

#[derive(Clone)]
struct ParameterSignature {
    declaration: DeclarationId,
    name: String,
    ty: CheckedType,
}

#[derive(Clone)]
struct FunctionSignature {
    id: FunctionId,
    declaration: DeclarationId,
    node: NodeId,
    name: String,
    parameters: Vec<ParameterSignature>,
    result: CheckedType,
    effects_node: NodeId,
    declared_traps: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LocalBinding {
    binding: BindingId,
    ty: CheckedType,
    live: bool,
    loop_depth: usize,
}

#[derive(Clone, Copy)]
enum Constructor {
    Struct(NominalId),
    Enum { nominal: NominalId, variant: u32 },
}

struct TypedExpression {
    expression: CheckedExpression,
    exhibits_traps: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PreludeType {
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
    prelude_nominals: HashMap<PreludeType, NominalId>,
    prelude_types: Vec<Option<PreludeType>>,
    nominals_by_declaration: HashMap<DeclarationId, NominalId>,
    constructors_by_declaration: HashMap<DeclarationId, Constructor>,
    signatures: Vec<FunctionSignature>,
    functions_by_declaration: HashMap<DeclarationId, FunctionId>,
    constants: HashMap<DeclarationId, CheckedValue>,
}

/// Checks the currently implemented exact-v0.14 semantic family.
///
/// Unsupported language families remain explicit compiler capability results;
/// only a proved numbered-rule violation becomes [`SemanticOutcome::SourceIssue`].
#[must_use]
pub fn check_semantics_v0_14<'classified, 'lexed, 'source>(
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
            prelude_nominals: HashMap::new(),
            prelude_types: Vec::new(),
            nominals_by_declaration: HashMap::new(),
            constructors_by_declaration: HashMap::new(),
            signatures: Vec::new(),
            functions_by_declaration: HashMap::new(),
            constants: HashMap::new(),
        })
    }

    fn check_program(&mut self) -> Result<CheckedProgramData, CheckStop> {
        let items = self.item_declarations()?;
        self.check_main_header(&items)?;
        self.reject_unimplemented_items(&items)?;
        self.collect_nominals(&items)?;
        self.collect_function_signatures(&items)?;
        self.collect_constants(&items)?;
        let main = self.main_id()?;

        let mut functions = Vec::with_capacity(self.signatures.len());
        for index in 0..self.signatures.len() {
            functions.push(self.check_function(index)?);
        }
        Ok(CheckedProgramData {
            nominals: self.nominals.clone(),
            functions,
            main,
        })
    }

    fn item_declarations(&self) -> Result<Vec<NodeId>, CheckStop> {
        let mut declarations = Vec::new();
        for item in self.tree.children(self.tree.root())? {
            if self.tree.production(*item)? != ProductionV0_14::Item {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            declarations.push(self.tree.only_child(*item)?);
        }
        Ok(declarations)
    }

    fn reject_unimplemented_items(&self, items: &[NodeId]) -> Result<(), CheckStop> {
        for item in items {
            let feature = match self.tree.production(*item)? {
                ProductionV0_14::FnDecl
                | ProductionV0_14::ConstDecl
                | ProductionV0_14::StructDecl
                | ProductionV0_14::EnumDecl => continue,
                ProductionV0_14::ContractDecl | ProductionV0_14::ConformDecl => {
                    UnsupportedSemanticFeatureV0_14::ContractsAndConformances
                }
                _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
            };
            return self.unsupported(feature, *item);
        }
        Ok(())
    }

    fn collect_function_signatures(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        for node in items.iter().copied().filter(|node| {
            self.tree
                .production(*node)
                .is_ok_and(|production| production == ProductionV0_14::FnDecl)
        }) {
            if let Some(generics) = self
                .tree
                .first_child_with(node, ProductionV0_14::Generics)?
            {
                return self.unsupported(UnsupportedSemanticFeatureV0_14::Generics, generics);
            }
            if let Some(regions) = self
                .tree
                .first_child_with(node, ProductionV0_14::RegionParams)?
            {
                return self
                    .unsupported(UnsupportedSemanticFeatureV0_14::RegionsAndBorrows, regions);
            }
            if let Some(requires) = self
                .tree
                .first_child_with(node, ProductionV0_14::RequiresBlock)?
            {
                return self.unsupported(UnsupportedSemanticFeatureV0_14::RequiresBlocks, requires);
            }

            let declaration = self.declaration_at(node, DeclarationRole::Function)?;
            let declaration_id = declaration.id();
            let name = declaration.spelling().to_owned();
            let id = FunctionId(
                u32::try_from(self.signatures.len())
                    .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
            );
            let parameters = self.parse_parameters(node)?;
            let rtype = self
                .tree
                .first_child_with(node, ProductionV0_14::Rtype)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let result = self.parse_rtype(rtype)?;
            let effects = self
                .tree
                .first_child_with(node, ProductionV0_14::Effects)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let declared_traps = self.parse_effects(effects)?;
            self.functions_by_declaration.insert(declaration_id, id);
            self.signatures.push(FunctionSignature {
                id,
                declaration: declaration_id,
                node,
                name,
                parameters,
                result,
                effects_node: effects,
                declared_traps,
            });
        }
        Ok(())
    }

    fn collect_constants(&mut self, items: &[NodeId]) -> Result<(), CheckStop> {
        for node in items.iter().copied().filter(|node| {
            self.tree
                .production(*node)
                .is_ok_and(|production| production == ProductionV0_14::ConstDecl)
        }) {
            let declaration = self.declaration_at(node, DeclarationRole::NamedConst)?;
            let ty_node = self
                .tree
                .first_child_with(node, ProductionV0_14::Type)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let ty = self.parse_type(ty_node)?;
            let value_node = self
                .tree
                .first_child_with(node, ProductionV0_14::Cvalue)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let value = self.parse_const_value(value_node)?;
            if value.ty() != ty {
                return self.issue_node(
                    SemanticRuleV0_14::Const2,
                    value_node,
                    SemanticIssueKind::InvalidConstValue,
                );
            }
            self.constants.insert(declaration.id(), value);
        }
        Ok(())
    }

    fn check_main_header(&self, items: &[NodeId]) -> Result<(), CheckStop> {
        let mut main = None;
        for node in items.iter().copied().filter(|node| {
            self.tree
                .production(*node)
                .is_ok_and(|production| production == ProductionV0_14::FnDecl)
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
                rule: SemanticRuleV0_14::Fn7,
                location: SemanticLocation::BundleRoot(
                    self.resolved.syntax().root_extent().to_vec(),
                ),
                kind: SemanticIssueKind::MissingMain,
            }));
        };

        let generics = self
            .tree
            .first_child_with(node, ProductionV0_14::Generics)?;
        let regions = self
            .tree
            .first_child_with(node, ProductionV0_14::RegionParams)?;
        let parameters = self
            .tree
            .first_child_with(node, ProductionV0_14::ParamList)?;
        let rtype = self
            .tree
            .first_child_with(node, ProductionV0_14::Rtype)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mode = self
            .tree
            .first_child_with(rtype, ProductionV0_14::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let ty = self
            .tree
            .first_child_with(rtype, ProductionV0_14::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let effects = self
            .tree
            .first_child_with(node, ProductionV0_14::Effects)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if generics.is_some()
            || regions.is_some()
            || parameters.is_some()
            || !self.has_fixed(mode, crate::FixedTerminalV0_14::Own)?
            || !self.has_fixed(ty, crate::FixedTerminalV0_14::Unit)?
            || !self.main_effects_allowed(effects)?
        {
            return self.issue_node(SemanticRuleV0_14::Fn7, node, SemanticIssueKind::InvalidMain);
        }
        Ok(())
    }

    fn main_effects_allowed(&self, effects: NodeId) -> Result<bool, CheckStop> {
        if self.has_fixed(effects, crate::FixedTerminalV0_14::Pure)? {
            return Ok(true);
        }
        let effects = self.tree.children_with(effects, ProductionV0_14::Effect)?;
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
                    ty: parameter.ty,
                    live: true,
                    loop_depth: 0,
                },
            );
            parameters.push(CheckedParameter {
                name: parameter.name.clone(),
                binding,
                ty: parameter.ty,
            });
        }

        let statements = self
            .tree
            .children_with(signature.node, ProductionV0_14::Stmt)?;
        let mut counters = ControlCounters {
            next_binding: &mut next_binding,
            next_loop: &mut next_loop,
        };
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
                rule: SemanticRuleV0_14::Fn1,
                location: SemanticLocation::SourceNode(
                    self.tree.path(signature.node)?.clone(),
                    self.tree.closing_brace_coordinate(signature.node)?,
                ),
                kind: SemanticIssueKind::FunctionFallthrough,
            }));
        }
        if checked.exhibits_traps != signature.declared_traps {
            return self.issue_node(
                SemanticRuleV0_14::Eff2,
                signature.effects_node,
                SemanticIssueKind::EffectMismatch,
            );
        }
        Ok(CheckedFunction {
            id: signature.id,
            declaration: signature.declaration,
            name: signature.name.clone(),
            parameters,
            result: signature.result,
            declared_traps: signature.declared_traps,
            body: checked.statements,
        })
    }
}
