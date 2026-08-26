//! The [FN-7] entry-form admission judgment over one closed compilation unit.
//!
//! [FN-7] fixes exactly one entry shape. The entry carries the fixed `command`
//! marker, a writer-named `own ExitStatus` result, no contract, and labelled
//! value parameters selected from the closed command-input table.
//!
//! The judgment is whole-unit: no other declaration may carry a `program_kind`
//! or an `input_label` child, and a `call` whose callee resolves to a
//! kind-declaring entry is rejected here because program start is that entry's
//! only invocation [PROG-3]. It runs on resolved facts and before the
//! remaining unsupported system-surface stops, because an unsupported compiler
//! capability establishes no source violation [DIAG-1] and must never mask one.
//!
//! Every rejection uses the exact `SourceNode` [FN-7] names for it: the
//! `program_kind` node, the `input_label` node, the complete `param` node, the
//! `rtype` node, the `effects` node, the `generics` or `region_params` child,
//! the `call` node, or the `fn_decl` node as the stated fallback.

use crate::syntax::NodeId;
use crate::{
    DeclarationClass, DeclarationRole, FixedTerminal, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticRule,
    SystemEntity, system_entity,
};

use super::super::model::{CheckedEntryForm, CheckedResourceAlias};
use super::{CheckStop, Checker, RegionKindConflictOwner};

/// One row of [FN-7]'s closed standard-input table for kind `command`.
struct StandardInput {
    /// Label tail written after the kind IDENT and `.`.
    tail: &'static str,
    /// The row's exact written mode and type.
    written: &'static str,
    /// [SYS-2] nominal spelling the row's written type must resolve to.
    nominal: &'static str,
}

const fn input(tail: &'static str, written: &'static str, nominal: &'static str) -> StandardInput {
    StandardInput {
        tail,
        written,
        nominal,
    }
}

/// [FN-7]'s closed standard-input table for kind `command`, in table-ordinal
/// order. Ordinal identity, never type identity, selects the supplied value:
/// `command.stdout` and `command.stderr` share one type and stay two inputs.
const COMMAND_INPUTS: [StandardInput; 4] = [
    input("args", "own Args", "Args"),
    input("cwd", "own DirectoryRead<'q, 'dh, 'd, 'f>", "DirectoryRead"),
    input("stdout", "own Output<'q, 'o>", "Output"),
    input("stderr", "own Output<'q, 'o>", "Output"),
];

/// The `command.stdout` and `command.stderr` table ordinals.
const COMMAND_STDOUT: u8 = 2;
const COMMAND_STDERR: u8 = 3;

const COMMAND_RESULT: &str = "own ExitStatus";
const COMMAND_RESULT_NOMINAL: &str = "ExitStatus";
const COMMAND_EFFECTS: &str =
    "the exact world reads/writes row, plus `allocates(heap)` and `traps` when exhibited";

/// Returns the conservative alias links the entry's selected inputs carry.
///
/// [SYS-12] fixes exactly one: when the entry selects both `command.stdout`
/// and `command.stderr`, redirection may make those two `Output` owners the
/// same sink. The link is a retained fact only [DIAG-2] — it decides no
/// judgment here and refuses no program — and the first slice declares no
/// duplicate, split, or attenuation operation, so no other pair of system
/// values can alias and nothing beyond ordinary move tracking is needed.
fn command_resource_aliases(inputs: &[u8]) -> Vec<CheckedResourceAlias> {
    if inputs.contains(&COMMAND_STDOUT) && inputs.contains(&COMMAND_STDERR) {
        return vec![CheckedResourceAlias {
            left: COMMAND_STDOUT,
            right: COMMAND_STDERR,
        }];
    }
    Vec::new()
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// Admits the unit's [FN-7] entry and returns the form it admitted.
    pub(super) fn check_entry_form(&self, items: &[NodeId]) -> Result<CheckedEntryForm, CheckStop> {
        let entry = self.entry_declaration(items)?;
        let entry_kind = self.tree.first_child_with(entry, Production::ProgramKind)?;
        self.reject_non_entry_program_kind(entry_kind)?;
        // FN-7's unit-level kind-declaring judgment comes from
        // `syntax::unit_program_kind`. The scan above admits no other
        // declaration carrying a `program_kind`, so after it succeeds the
        // unit judgment and the entry's child must be the same node. [SYS-3]
        // is independent: resolution installs system declarations in every
        // unit before this entry-form check.
        if crate::syntax::unit_program_kind(self.tree.topology()) != entry_kind {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        self.check_entry_parameters(entry)?;
        if self
            .tree
            .first_child_with(entry, Production::ContractBlock)?
            .is_some()
        {
            return self.issue_node(SemanticRule::Fn7, entry, SemanticIssueKind::InvalidMain);
        }
        let Some(kind) = entry_kind else {
            return self.issue_node(SemanticRule::Fn7, entry, SemanticIssueKind::InvalidMain);
        };
        let inputs = self.check_command_entry(entry, kind)?;
        self.reject_foreign_input_labels(entry)?;
        self.reject_calls_to_entry(entry)?;
        let aliases = command_resource_aliases(&inputs);
        Ok(CheckedEntryForm { inputs, aliases })
    }

    /// Selects the unique top-level `fn_decl` named `main`.
    ///
    /// A missing entry is the one [FN-7] rejection located at `BundleRoot`; a
    /// duplicate `main` spelling never reaches here, because [TYPE-6] rejects
    /// it during declaration inventory [DIAG-1].
    fn entry_declaration(&self, items: &[NodeId]) -> Result<NodeId, CheckStop> {
        for node in items.iter().copied().filter(|node| {
            self.tree
                .production(*node)
                .is_ok_and(|production| production == Production::FnDecl)
        }) {
            if self
                .declaration_at(node, DeclarationRole::Function)?
                .spelling()
                == "main"
            {
                return Ok(node);
            }
        }
        Err(CheckStop::source_issue(SemanticIssue {
            rule: SemanticRule::Fn7,
            location: SemanticLocation::BundleRoot(self.resolved.syntax().root_extent().to_vec()),
            kind: SemanticIssueKind::MissingMain,
        }))
    }

    /// Rejects a `program_kind` on any declaration that is not the entry.
    ///
    /// The grammar admits `program_kind` only as the optional second child of a
    /// `fn_decl`, which derives only from a top-level `item` [GRAM-2], so a
    /// total node scan sees exactly the declarations [FN-7] speaks about.
    fn reject_non_entry_program_kind(&self, entry_kind: Option<NodeId>) -> Result<(), CheckStop> {
        for node in self.nodes_with(Production::ProgramKind)? {
            if Some(node) == entry_kind {
                continue;
            }
            let owner = self
                .tree
                .parent(node)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let function = self
                .declaration_at(owner, DeclarationRole::Function)?
                .spelling()
                .to_owned();
            return self.issue_node(
                SemanticRule::Fn7,
                node,
                SemanticIssueKind::NonEntryProgramKind { function },
            );
        }
        Ok(())
    }

    /// Rejects type/const generics and records every entry region for the
    /// complete-unit [FN-7] kind check. Program start supplies only identities
    /// that the ordinary OWN-3 inference establishes as world-kind.
    fn check_entry_parameters(&self, entry: NodeId) -> Result<(), CheckStop> {
        if let Some(child) = self.tree.first_child_with(entry, Production::Generics)? {
            return self.issue_node(SemanticRule::Fn7, child, SemanticIssueKind::InvalidMain);
        }
        for region in self.parse_region_parameters(entry)? {
            let declaration = self
                .resolved
                .declarations()
                .get(region.index())
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let node = self
                .tree
                .node_with_path(declaration.origin().node())
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            self.entry_region_parameters
                .borrow_mut()
                .push((region, node));
        }
        Ok(())
    }

    /// Checks the one command entry and returns its selected input ordinals.
    fn check_command_entry(&self, entry: NodeId, kind_node: NodeId) -> Result<Vec<u8>, CheckStop> {
        if !self.has_fixed(kind_node, FixedTerminal::Command)? {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let inputs = self.check_standard_inputs(entry)?;

        let (rtype, mode, ty) = self.entry_result(entry)?;
        if !self.has_fixed(mode, FixedTerminal::Own)?
            || !self.resolves_to_system_nominal(ty, COMMAND_RESULT_NOMINAL)?
        {
            return self.issue_node(
                SemanticRule::Fn7,
                rtype,
                SemanticIssueKind::InvalidEntryResult {
                    required: COMMAND_RESULT,
                },
            );
        }
        let effects = self.entry_effects(entry)?;
        if !self.command_effects_admitted(effects)? {
            return self.issue_node(
                SemanticRule::Fn7,
                effects,
                SemanticIssueKind::InvalidEntryEffects {
                    admitted: COMMAND_EFFECTS,
                },
            );
        }
        Ok(inputs)
    }

    /// Checks every value parameter of a `command` entry against [FN-7]'s
    /// closed standard-input table and returns the selected ordinals.
    ///
    /// Every parameter carries an `input_label`; its IDENT equals one row's
    /// label tail. Each row is selected at most once and selected rows appear
    /// in strictly increasing table-ordinal order; the written mode and type
    /// equal the row exactly, with no conversion, default, or inferred mode.
    fn check_standard_inputs(&self, entry: NodeId) -> Result<Vec<u8>, CheckStop> {
        let Some(list) = self.tree.first_child_with(entry, Production::ParamList)? else {
            // [FN-7] admits a `command` entry that selects no row; it simply
            // receives no standard input.
            return Ok(Vec::new());
        };
        let mut selected: Vec<u8> = Vec::new();
        let mut command_order = None;
        let mut output_identity = None;
        for parameter in self.tree.children_with(list, Production::Param)? {
            let Some(label) = self
                .tree
                .first_child_with(parameter, Production::InputLabel)?
            else {
                return self.issue_node(
                    SemanticRule::Fn7,
                    parameter,
                    SemanticIssueKind::UnlabelledEntryParameter {
                        parameter: self.identifier(parameter)?,
                    },
                );
            };
            let tail = self.label_identifier(label)?;
            let ordinal = COMMAND_INPUTS
                .iter()
                .position(|row| row.tail == tail)
                .and_then(|ordinal| u8::try_from(ordinal).ok());
            let Some(ordinal) = ordinal else {
                return self.invalid_label(label, &tail);
            };
            if selected.last().is_some_and(|last| *last >= ordinal) {
                // Repeated and out-of-order selections are the same rejection:
                // ordinal identity selects the supplied value, and declared
                // order is the one legal byte sequence [FORM-1, GRAM-8].
                return self.invalid_label(label, &tail);
            }
            let world = self.check_standard_input_binding(parameter, &tail, ordinal)?;
            if let Some(region) = world.first().copied() {
                if let Some(expected) = command_order {
                    if region != expected {
                        return self.issue_node(
                            SemanticRule::Fn7,
                            parameter,
                            SemanticIssueKind::InvalidStandardInput {
                                label: format!("command.{tail}"),
                                declared: COMMAND_INPUTS[usize::from(ordinal)].written,
                            },
                        );
                    }
                } else {
                    command_order = Some(region);
                }
            }
            if matches!(ordinal, COMMAND_STDOUT | COMMAND_STDERR) {
                let identity = *world
                    .get(1)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if let Some(expected) = output_identity {
                    if identity != expected {
                        return self.issue_node(
                            SemanticRule::Fn7,
                            parameter,
                            SemanticIssueKind::InvalidStandardInput {
                                label: format!("command.{tail}"),
                                declared: COMMAND_INPUTS[usize::from(ordinal)].written,
                            },
                        );
                    }
                } else {
                    output_identity = Some(identity);
                }
            }
            selected.push(ordinal);
        }
        Ok(selected)
    }

    /// Requires one selected parameter's written mode and type to equal its
    /// table row exactly.
    fn check_standard_input_binding(
        &self,
        parameter: NodeId,
        tail: &str,
        ordinal: u8,
    ) -> Result<Vec<crate::DeclarationId>, CheckStop> {
        let row = COMMAND_INPUTS
            .get(usize::from(ordinal))
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mode = self
            .tree
            .first_child_with(parameter, Production::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let ty = self
            .tree
            .first_child_with(parameter, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.has_fixed(mode, FixedTerminal::Own)?
            || self.system_nominal_identity(ty, row.nominal)?.is_none()
        {
            return self.issue_node(
                SemanticRule::Fn7,
                parameter,
                SemanticIssueKind::InvalidStandardInput {
                    label: format!("command.{tail}"),
                    declared: row.written,
                },
            );
        }
        self.system_nominal_identity(ty, row.nominal)?
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    /// Rejects an `input_label` anywhere outside the command entry's own
    /// parameters, including one written in a `fn_sig`.
    fn reject_foreign_input_labels(&self, entry: NodeId) -> Result<(), CheckStop> {
        let admitted = match self.tree.first_child_with(entry, Production::ParamList)? {
            None => Vec::new(),
            Some(list) => self.tree.descendants_with(list, Production::InputLabel)?,
        };
        for label in self.nodes_with(Production::InputLabel)? {
            if admitted.contains(&label) {
                continue;
            }
            let tail = self.label_identifier(label)?;
            return self.issue_node(
                SemanticRule::Fn7,
                label,
                SemanticIssueKind::StandardInputLabelOutsideEntry {
                    label: format!("command.{tail}"),
                },
            );
        }
        Ok(())
    }

    /// Rejects a source `call` whose callee resolves to the command entry.
    ///
    /// That entry is invoked exactly once, by program start [PROG-3]: its
    /// standard inputs are supplied there and are neither constructible nor
    /// forgeable by source.
    fn reject_calls_to_entry(&self, entry: NodeId) -> Result<(), CheckStop> {
        let declaration = self.declaration_at(entry, DeclarationRole::Function)?;
        let entry_id = declaration.id();
        let name = declaration.spelling().to_owned();
        for usage in self.resolved.lexical_uses() {
            if usage.role() != LexicalUseRole::IdentifierCallee
                || usage.target()
                    != (ResolvedTarget::Source {
                        declaration: entry_id,
                        class: DeclarationClass::Function,
                    })
            {
                continue;
            }
            let callee = self
                .tree
                .node_with_path(usage.origin().node())
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let call = self
                .tree
                .parent(callee)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            return self.issue_node(
                SemanticRule::Fn7,
                call,
                SemanticIssueKind::CallToKindDeclaringEntry { entry: name },
            );
        }
        Ok(())
    }

    fn invalid_label<ResultValue>(
        &self,
        label: NodeId,
        tail: &str,
    ) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Fn7,
            label,
            SemanticIssueKind::InvalidStandardInputLabel {
                label: format!("command.{tail}"),
                declared_labels: COMMAND_INPUTS
                    .iter()
                    .map(|row| format!("command.{}", row.tail))
                    .collect(),
            },
        )
    }

    /// Returns the entry's `rtype` node with its written mode and type.
    fn entry_result(&self, entry: NodeId) -> Result<(NodeId, NodeId, NodeId), CheckStop> {
        let result_binding = self
            .tree
            .first_child_with(entry, Production::ResultBinding)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let rtype = self
            .tree
            .first_child_with(result_binding, Production::Rtype)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let mode = self
            .tree
            .first_child_with(rtype, Production::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let ty = self
            .tree
            .first_child_with(rtype, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        Ok((rtype, mode, ty))
    }

    fn entry_effects(&self, entry: NodeId) -> Result<NodeId, CheckStop> {
        self.tree
            .first_child_with(entry, Production::Effects)?
            .ok_or_else(|| SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// Reports whether every category written in a `command` entry's row is
    /// admitted by that kind row.
    ///
    /// Reads and writes are admitted only over the entry's world parameters;
    /// allocation is heap-only, and `traps` is the sole payload-free flag.
    fn command_effects_admitted(&self, effects: NodeId) -> Result<bool, CheckStop> {
        if self.has_fixed(effects, FixedTerminal::Pure)? {
            return Ok(true);
        }
        for effect in self.tree.children_with(effects, Production::Effect)? {
            let admitted = if self.has_fixed(effect, FixedTerminal::Reads)?
                || self.has_fixed(effect, FixedTerminal::Writes)?
            {
                // OWN-3 deliberately does not infer a kind from a row. Every
                // entry row region is an entry parameter by resolution; the
                // complete-unit pass later rejects a memory-kind parameter
                // under FN-7 and an unanchored one under OWN-3.
                let _ = self.effect_regions(effect)?;
                true
            } else if self.has_fixed(effect, FixedTerminal::Allocates)? {
                self.has_fixed(effect, FixedTerminal::Heap)?
                    && !self.has_fixed(effect, FixedTerminal::Arena)?
            } else {
                self.has_fixed(effect, FixedTerminal::Traps)?
            };
            if !admitted {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Reports whether a written `type` is a bare TYPEID resolving to the
    /// named [SYS-2] nominal.
    ///
    /// The judgment binds to the resolved system declaration, not to source
    /// bytes: a `targs` child, a non-nominal target, or any other system
    /// nominal fails.
    fn resolves_to_system_nominal(&self, ty: NodeId, nominal: &str) -> Result<bool, CheckStop> {
        Ok(self.system_nominal_identity(ty, nominal)?.is_some())
    }

    /// Returns the exact world vector when a written type resolves to the
    /// requested system nominal with its required arity.
    fn system_nominal_identity(
        &self,
        ty: NodeId,
        nominal: &str,
    ) -> Result<Option<Vec<crate::DeclarationId>>, CheckStop> {
        let Ok(usage) = self.use_at(ty, LexicalUseRole::Type) else {
            return Ok(None);
        };
        let ResolvedTarget::System(id) = usage.target() else {
            return Ok(None);
        };
        let Some(index) = crate::system_nominal_index(id, self.inventory()) else {
            return Ok(None);
        };
        let Some(SystemEntity::Nominal(entry)) = system_entity(id, self.inventory()) else {
            return Ok(None);
        };
        if entry.spelling != nominal {
            return Ok(None);
        }
        let targs = self.tree.first_child_with(ty, Production::Targs)?;
        if entry.world_parameters.is_empty() {
            return Ok(targs.is_none().then(Vec::new));
        }
        let Some(targs) = targs else {
            return Ok(None);
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        if arguments.len() != entry.world_parameters.len() {
            return Ok(None);
        }
        let mut regions = Vec::with_capacity(arguments.len());
        for argument in arguments {
            let usage = self.use_at(argument, LexicalUseRole::TypeArgumentRegion)?;
            let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Region,
            } = usage.target()
            else {
                return Ok(None);
            };
            self.constrain_region_kind_for_rule(
                argument,
                declaration,
                super::super::model::CheckedRegionKind::World,
                RegionKindConflictOwner::Sys2,
            )?;
            regions.push(declaration);
        }
        let _ = index;
        Ok(Some(regions))
    }

    /// Returns the writer-chosen tail IDENT of one fixed-command input label.
    fn label_identifier(&self, label: NodeId) -> Result<String, CheckStop> {
        let [tail] = self.tree.direct_identifiers(label)?[..] else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        self.decoded(tail)
    }

    fn decoded(&self, terminal: usize) -> Result<String, CheckStop> {
        std::str::from_utf8(self.tree.token_bytes(terminal)?)
            .map(str::to_owned)
            .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding.into())
    }

    /// Returns every node of one production in finalized node order.
    fn nodes_with(&self, production: Production) -> Result<Vec<NodeId>, CheckStop> {
        let mut nodes = Vec::new();
        for index in 0..self.tree.topology().nodes.len() {
            let node = NodeId::from_index(index).ok_or(SemanticCompilerFailure::CounterOverflow)?;
            if self.tree.production(node)? == production {
                nodes.push(node);
            }
        }
        Ok(nodes)
    }
}
