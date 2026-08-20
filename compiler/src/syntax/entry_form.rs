//! The [FN-7] kind-declaring judgment, read from finalized syntax alone.
//!
//! [FN-7] fixes that judgment as syntactic and total: a compilation unit is
//! kind-declaring exactly when at least one of its top-level `fn_decl` nodes
//! carries a `program_kind` child. It is taken after grammar derivation and
//! before declaration inventory, and it consults no resolved name, type, or
//! effect row -- not even whether the kind IDENT names an admitted [FN-7]
//! table row, whether the declaring function is the entry, or what standard
//! inputs it declares. Entry-form checking reads the judgment from here
//! instead of rederiving it. System inventory is independent of this judgment:
//! [SYS-3] admits that domain to every compilation unit.

use crate::syntax::grammar::Production;
use crate::syntax::parser::{FinalizedTopology, NodeId};

/// Returns the `program_kind` node making one compilation unit kind-declaring.
///
/// `Some` is the [FN-7] kind-declaring judgment for the unit and `None` is its
/// negation, so a caller needing only the flag reads `.is_some()`. The node is
/// the lowest-numbered such node, which is the first in finalized preorder and
/// therefore in top-level item order; it doubles as the [DIAG-1] `SourceNode`
/// location for a rejection owned by that judgment.
///
/// The scan covers every node because the grammar admits `program_kind` in
/// exactly one place: as the optional second child of `fn_decl`, which itself
/// derives only from a top-level `item` [GRAM-2]. A `program_kind` node
/// existing anywhere is therefore the same fact as some top-level `fn_decl`
/// carrying one, and a total scan cannot miss a nesting the grammar does not
/// admit.
#[must_use]
pub(crate) fn unit_program_kind(topology: &FinalizedTopology) -> Option<NodeId> {
    topology
        .nodes
        .iter()
        .position(|record| record.production == Production::ProgramKind)
        .and_then(NodeId::from_index)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic)]

    use super::unit_program_kind;
    use crate::lexer::{LexOutcome, lex};
    use crate::syntax::grammar::Production;
    use crate::syntax::parser::{
        CanonicalOutcome, FinalizeOutcome, FinalizedTopology, NodeId, ParseOutcome,
        audit_canonical, finalize, parse,
    };
    use crate::{
        ACTIVE_KERNEL_SPEC_HASH, CompilerLimits, SourceBundle, SourceInput, TerminalOutcome,
        classify_terminals,
    };

    const UNLABELLED_ENTRY: &[u8] = b"fn main() -> result: own unit pure {\n  return unit;\n}\n";

    const COMMAND_ENTRY: &[u8] = b"command fn main(command.args as args: own Args) -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";

    const NON_ENTRY_KIND: &[u8] = b"command fn helper() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n\nfn main() -> result: own unit pure {\n  return unit;\n}\n";

    fn with_topology<ResultValue>(
        source: &[u8],
        run: impl FnOnce(&FinalizedTopology) -> ResultValue,
    ) -> ResultValue {
        let limits = CompilerLimits::default();
        let inputs = [SourceInput::new("entry.wf", source)];
        let Ok(bundle) = SourceBundle::with_limits(&inputs, limits.source) else {
            panic!("entry-form fixture must form one source bundle");
        };
        let LexOutcome::Complete(lexed) = lex(&bundle, limits.lexer) else {
            panic!("entry-form fixture must lex");
        };
        let TerminalOutcome::Complete(classified) =
            classify_terminals(&lexed, ACTIVE_KERNEL_SPEC_HASH, limits.terminals)
        else {
            panic!("entry-form fixture must classify");
        };
        let ParseOutcome::Complete(parsed) = parse(&classified, limits.parser) else {
            panic!("entry-form fixture must derive");
        };
        let FinalizeOutcome::Complete(finalized) = finalize(parsed, limits.finalizer) else {
            panic!("entry-form fixture must finalize");
        };
        let outcome = audit_canonical(finalized, limits.canonical);
        let CanonicalOutcome::Complete(unit) = outcome else {
            panic!("entry-form fixture must use exact FORM-2 bytes: {outcome:?}");
        };
        run(&unit.finalized.topology)
    }

    /// The [FN-7] wording, evaluated directly over top-level declarations.
    fn declaring_top_level_items(topology: &FinalizedTopology) -> Vec<NodeId> {
        let Some(items) = topology.node_children(topology.root) else {
            panic!("the finalized root must publish its item children");
        };
        items
            .iter()
            .filter_map(|item| {
                let Some([declaration]) = topology.node_children(*item) else {
                    panic!("every item must wrap exactly one declaration");
                };
                topology
                    .node_children(*declaration)?
                    .iter()
                    .copied()
                    .find(|child| {
                        topology
                            .node(*child)
                            .is_some_and(|record| record.production == Production::ProgramKind)
                    })
            })
            .collect()
    }

    #[test]
    fn the_unlabelled_entry_leaves_the_unit_not_kind_declaring() {
        with_topology(UNLABELLED_ENTRY, |topology| {
            assert_eq!(unit_program_kind(topology), None);
            assert!(declaring_top_level_items(topology).is_empty());
        });
    }

    #[test]
    fn a_program_kind_child_makes_the_unit_kind_declaring() {
        with_topology(COMMAND_ENTRY, |topology| {
            let Some(node) = unit_program_kind(topology) else {
                panic!("a command entry must publish its program_kind node");
            };
            assert_eq!(
                topology.node(node).map(|record| record.production),
                Some(Production::ProgramKind)
            );
            // The total scan agrees with FN-7's own wording, which the grammar
            // makes equivalent by admitting `program_kind` only under a
            // top-level `fn_decl`.
            assert_eq!(declaring_top_level_items(topology), vec![node]);
        });
    }

    #[test]
    fn the_judgment_reads_the_kind_child_alone_not_the_declared_name() {
        // The declaring function is `helper`, not the entry; its kind IDENT
        // names no admitted FN-7 table row; and the unit's entry is the
        // ordinary unlabelled `main`. The judgment is still true, because it
        // depends on the `program_kind` child and on nothing resolution or
        // checking would decide later.
        with_topology(NON_ENTRY_KIND, |topology| {
            let Some(node) = unit_program_kind(topology) else {
                panic!("a non-entry program_kind must still make the unit kind-declaring");
            };
            assert_eq!(declaring_top_level_items(topology), vec![node]);
        });
    }
}
