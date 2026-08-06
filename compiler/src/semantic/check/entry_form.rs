//! The [FN-7] entry-form admission judgment over one closed compilation unit.
//!
//! [FN-7] fixes exactly two entry shapes. The unlabelled entry carries no
//! `program_kind` child, declares no value parameter, writes `own unit`, and
//! writes one of four exact effect rows. A kind-declaring entry carries one
//! `program_kind` child whose IDENT must equal one row of a closed kind table;
//! that row fixes the entry's written result and the effect categories its row
//! may draw from, and the entry declares its standard inputs as labelled value
//! parameters selected from that kind's closed standard-input table.
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

use super::super::model::CheckedEntryForm;
use super::{CheckStop, Checker};

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

/// The only [FN-7] program-kind row with a defined entry form in v0.18.
///
/// `service` and `embedded` are reserved spellings with no form; [FN-7] makes
/// naming one the same hard error as naming no row at all, so the closed
/// admitted set is exactly this table.
const ADMITTED_KINDS: [&str; 1] = ["command"];

/// [FN-7]'s closed standard-input table for kind `command`, in table-ordinal
/// order. Ordinal identity, never type identity, selects the supplied value:
/// `command.stdout` and `command.stderr` share one type and stay two inputs.
const COMMAND_INPUTS: [StandardInput; 4] = [
    input("args", "own Args", "Args"),
    input("cwd", "own DirectoryRead", "DirectoryRead"),
    input("stdout", "own Output", "Output"),
    input("stderr", "own Output", "Output"),
];

const COMMAND_RESULT: &str = "own ExitStatus";
const COMMAND_RESULT_NOMINAL: &str = "ExitStatus";
const COMMAND_EFFECTS: &str =
    "any subset of `allocates(heap), external, blocks, traps` in EFF-1 canonical order";

const UNLABELLED_RESULT: &str = "own unit";
const UNLABELLED_EFFECTS: &str =
    "exactly one of `pure`, `allocates(heap)`, `traps`, `allocates(heap), traps`";

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// Admits the unit's [FN-7] entry and returns the form it admitted.
    pub(super) fn check_entry_form(&self, items: &[NodeId]) -> Result<CheckedEntryForm, CheckStop> {
        let entry = self.entry_declaration(items)?;
        let entry_kind = self.tree.first_child_with(entry, Production::ProgramKind)?;
        self.reject_non_entry_program_kind(entry_kind)?;
        // [SYS-3] admission already decided the unit-level kind-declaring
        // judgment from `syntax::unit_program_kind`. FN-7 needs the finer fact
        // -- whether the *entry* carries a `program_kind` -- plus the scan
        // above, which admits no other declaration carrying one. Once that
        // scan succeeds the two judgments are the same node, so name lookup
        // and entry-form checking cannot disagree about a unit.
        if crate::syntax::unit_program_kind(self.tree.topology()) != entry_kind {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        self.reject_entry_polymorphism(entry)?;

        let form = match entry_kind {
            None => {
                self.check_unlabelled_entry(entry)?;
                CheckedEntryForm::Unlabelled
            }
            Some(kind) => CheckedEntryForm::Command {
                inputs: self.check_kind_declaring_entry(entry, kind)?,
            },
        };
        self.reject_foreign_input_labels(entry, entry_kind)?;
        if entry_kind.is_some() {
            self.reject_calls_to_entry(entry)?;
        }
        Ok(form)
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
        Err(CheckStop::Issue(SemanticIssue {
            rule: SemanticRule::Fn7,
            location: SemanticLocation::BundleRoot(self.resolved.syntax().root_extent().to_vec()),
            kind: SemanticIssueKind::MissingMain,
        }))
    }

    /// Rejects a `program_kind` on any declaration that is not the entry.
    ///
    /// The grammar admits `program_kind` only as the optional first child of a
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

    /// Rejects a generic or region-parameter-bearing entry at its own child.
    ///
    /// [FN-7] states this before the form split, so one judgment serves both
    /// entry shapes.
    fn reject_entry_polymorphism(&self, entry: NodeId) -> Result<(), CheckStop> {
        for production in [Production::Generics, Production::RegionParams] {
            if let Some(child) = self.tree.first_child_with(entry, production)? {
                return self.issue_node(SemanticRule::Fn7, child, SemanticIssueKind::InvalidMain);
            }
        }
        Ok(())
    }

    /// Checks the complete unlabelled entry form: no value parameter, written
    /// result `own unit`, and exactly one of four written effect rows.
    fn check_unlabelled_entry(&self, entry: NodeId) -> Result<(), CheckStop> {
        if self
            .tree
            .first_child_with(entry, Production::ParamList)?
            .is_some()
        {
            // [FN-7] names no more specific node for this violation, so it
            // takes the stated `fn_decl` fallback.
            return self.issue_node(SemanticRule::Fn7, entry, SemanticIssueKind::InvalidMain);
        }
        let (rtype, mode, ty) = self.entry_result(entry)?;
        if !self.has_fixed(mode, FixedTerminal::Own)? || !self.has_fixed(ty, FixedTerminal::Unit)? {
            return self.issue_node(
                SemanticRule::Fn7,
                rtype,
                SemanticIssueKind::InvalidEntryResult {
                    required: UNLABELLED_RESULT,
                },
            );
        }
        let effects = self.entry_effects(entry)?;
        if !self.unlabelled_effects_admitted(effects)? {
            return self.issue_node(
                SemanticRule::Fn7,
                effects,
                SemanticIssueKind::InvalidEntryEffects {
                    admitted: UNLABELLED_EFFECTS,
                },
            );
        }
        Ok(())
    }

    /// Checks a kind-declaring entry and returns its selected input ordinals.
    fn check_kind_declaring_entry(
        &self,
        entry: NodeId,
        kind_node: NodeId,
    ) -> Result<Vec<u8>, CheckStop> {
        let kind = self.identifier(kind_node)?;
        if !ADMITTED_KINDS.contains(&kind.as_str()) {
            return self.issue_node(
                SemanticRule::Fn7,
                kind_node,
                SemanticIssueKind::InvalidProgramKind {
                    kind,
                    admitted_kinds: ADMITTED_KINDS.iter().map(|row| (*row).to_owned()).collect(),
                },
            );
        }
        let inputs = self.check_standard_inputs(entry, &kind)?;

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
    /// Every parameter carries an `input_label`; the label's first IDENT
    /// equals the entry's kind IDENT and its second equals a row's label tail;
    /// each row is selected at most once and selected rows appear in strictly
    /// increasing table-ordinal order; and the written mode and type equal the
    /// row exactly, with no conversion, default, or inferred mode.
    fn check_standard_inputs(&self, entry: NodeId, kind: &str) -> Result<Vec<u8>, CheckStop> {
        let Some(list) = self.tree.first_child_with(entry, Production::ParamList)? else {
            // [FN-7] admits a `command` entry that selects no row; it simply
            // receives no standard input.
            return Ok(Vec::new());
        };
        let mut selected: Vec<u8> = Vec::new();
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
            let (prefix, tail) = self.label_identifiers(label)?;
            let ordinal = COMMAND_INPUTS
                .iter()
                .position(|row| row.tail == tail)
                .and_then(|ordinal| u8::try_from(ordinal).ok());
            let Some(ordinal) = ordinal.filter(|_| prefix == kind) else {
                return self.invalid_label(label, &prefix, &tail);
            };
            if selected.last().is_some_and(|last| *last >= ordinal) {
                // Repeated and out-of-order selections are the same rejection:
                // ordinal identity selects the supplied value, and declared
                // order is the one legal byte sequence [FORM-1, GRAM-8].
                return self.invalid_label(label, &prefix, &tail);
            }
            self.check_standard_input_binding(parameter, &prefix, &tail, ordinal)?;
            selected.push(ordinal);
        }
        Ok(selected)
    }

    /// Requires one selected parameter's written mode and type to equal its
    /// table row exactly.
    fn check_standard_input_binding(
        &self,
        parameter: NodeId,
        prefix: &str,
        tail: &str,
        ordinal: u8,
    ) -> Result<(), CheckStop> {
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
            || !self.resolves_to_system_nominal(ty, row.nominal)?
        {
            return self.issue_node(
                SemanticRule::Fn7,
                parameter,
                SemanticIssueKind::InvalidStandardInput {
                    label: format!("{prefix}.{tail}"),
                    declared: row.written,
                },
            );
        }
        Ok(())
    }

    /// Rejects an `input_label` anywhere outside an admitted kind-declaring
    /// entry's own parameters, including one written in a `fn_sig`.
    fn reject_foreign_input_labels(
        &self,
        entry: NodeId,
        entry_kind: Option<NodeId>,
    ) -> Result<(), CheckStop> {
        let admitted = match entry_kind {
            None => Vec::new(),
            Some(_) => match self.tree.first_child_with(entry, Production::ParamList)? {
                None => Vec::new(),
                Some(list) => self.tree.descendants_with(list, Production::InputLabel)?,
            },
        };
        for label in self.nodes_with(Production::InputLabel)? {
            if admitted.contains(&label) {
                continue;
            }
            let (prefix, tail) = self.label_identifiers(label)?;
            return self.issue_node(
                SemanticRule::Fn7,
                label,
                SemanticIssueKind::StandardInputLabelOutsideEntry {
                    label: format!("{prefix}.{tail}"),
                },
            );
        }
        Ok(())
    }

    /// Rejects a source `call` whose callee resolves to a kind-declaring
    /// entry.
    ///
    /// That entry is invoked exactly once, by program start [PROG-3]: its
    /// standard inputs are supplied there and are neither constructible nor
    /// forgeable by source. The unlabelled entry keeps its ordinary callee
    /// status [TYPE-6, OP-1].
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
        prefix: &str,
        tail: &str,
    ) -> Result<ResultValue, CheckStop> {
        self.issue_node(
            SemanticRule::Fn7,
            label,
            SemanticIssueKind::InvalidStandardInputLabel {
                label: format!("{prefix}.{tail}"),
                declared_labels: COMMAND_INPUTS
                    .iter()
                    .map(|row| format!("command.{}", row.tail))
                    .collect(),
            },
        )
    }

    /// Returns the entry's `rtype` node with its written mode and type.
    fn entry_result(&self, entry: NodeId) -> Result<(NodeId, NodeId, NodeId), CheckStop> {
        let rtype = self
            .tree
            .first_child_with(entry, Production::Rtype)?
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

    /// Reports whether an unlabelled entry's row is one of [FN-7]'s four.
    fn unlabelled_effects_admitted(&self, effects: NodeId) -> Result<bool, CheckStop> {
        if self.has_fixed(effects, FixedTerminal::Pure)? {
            return Ok(true);
        }
        let written = self.tree.children_with(effects, Production::Effect)?;
        let spellings = written
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

    /// Reports whether every category written in a `command` entry's row is
    /// admitted by that kind row.
    ///
    /// The admitted set is `allocates(heap)`, `external`, `blocks`, `traps`;
    /// `pure` is the empty subset. No region-bearing effect is admitted, so a
    /// `reads`, `writes`, or arena allocation entry fails here. Canonical
    /// order and repetition stay [EFF-1]'s judgment.
    fn command_effects_admitted(&self, effects: NodeId) -> Result<bool, CheckStop> {
        if self.has_fixed(effects, FixedTerminal::Pure)? {
            return Ok(true);
        }
        for effect in self.tree.children_with(effects, Production::Effect)? {
            let admitted = if self.has_fixed(effect, FixedTerminal::Allocates)? {
                self.has_fixed(effect, FixedTerminal::Heap)?
                    && !self.has_fixed(effect, FixedTerminal::Arena)?
            } else {
                self.has_fixed(effect, FixedTerminal::External)?
                    || self.has_fixed(effect, FixedTerminal::Blocks)?
                    || self.has_fixed(effect, FixedTerminal::Traps)?
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
        if self.tree.first_child_with(ty, Production::Targs)?.is_some() {
            return Ok(false);
        }
        let Ok(usage) = self.use_at(ty, LexicalUseRole::Type) else {
            return Ok(false);
        };
        let ResolvedTarget::System(id) = usage.target() else {
            return Ok(false);
        };
        Ok(matches!(
            system_entity(id),
            Some(SystemEntity::Nominal(entry)) if entry.spelling == nominal
        ))
    }

    /// Returns the two IDENT spellings of one `input_label`.
    fn label_identifiers(&self, label: NodeId) -> Result<(String, String), CheckStop> {
        let [prefix, tail] = self.tree.direct_identifiers(label)?[..] else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        Ok((self.decoded(prefix)?, self.decoded(tail)?))
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
