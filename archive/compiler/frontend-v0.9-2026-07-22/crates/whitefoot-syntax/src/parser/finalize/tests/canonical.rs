use whitefoot_contract::{SourceId, SourceInput};

use super::super::{
    CanonicalCompilerFailure, CanonicalLimit, CanonicalLimits, CanonicalLocation, CanonicalOutcome,
    CanonicalResourceFailure, FinalizeOutcome, audit_canonical_v0_9, finalize_v0_9,
};
use super::support::{CANONICAL_LIMITS, FINALIZE_LIMITS, reaches_canonical_syntax, with_parsed};

fn audit_source(
    source: &[u8],
    audit: impl for<'classified, 'lexed, 'source> FnOnce(CanonicalOutcome<'classified, 'lexed, 'source>),
) {
    let inputs = [SourceInput::new("format.wf", source)];
    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(finalized) = finalize_v0_9(parsed, FINALIZE_LIMITS) else {
            panic!("complete derivation must finalize");
        };
        audit(audit_canonical_v0_9(finalized, CANONICAL_LIMITS));
    });
}

#[test]
fn exact_empty_and_nonempty_source_forests_publish_canonical_syntax() {
    for source in [
        b"\n".as_slice(),
        b"fn main() -> own unit pure {\n}\n".as_slice(),
        b"const first: i32 = 1_i32;\n\nconst second: i32 = 2_i32;\n".as_slice(),
    ] {
        audit_source(source, |outcome| {
            let CanonicalOutcome::Complete(unit) = outcome else {
                panic!("exact FORM-2 source must pass: {outcome:?}");
            };
            assert_eq!(unit.classified_bundle().source_bundle().len(), 1);
            assert_eq!(unit.root_extent().len(), 1);
        });
    }
}

#[test]
fn ordered_canonical_sources_keep_independent_forests() {
    let inputs = [
        SourceInput::new("empty.wf", b"\n"),
        SourceInput::new("first.wf", b"const first: i32 = 1_i32;\n"),
        SourceInput::new("second.wf", b"fn second() -> own unit pure {\n}\n"),
    ];
    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(finalized) = finalize_v0_9(parsed, FINALIZE_LIMITS) else {
            panic!("ordered canonical bundle must finalize");
        };
        let CanonicalOutcome::Complete(unit) = audit_canonical_v0_9(finalized, CANONICAL_LIMITS)
        else {
            panic!("each ordered source forest must pass independently");
        };
        assert_eq!(unit.root_extent().len(), 3);
        assert_eq!(unit.root_extent()[0].source(), SourceId::from_ordinal(0));
        assert_eq!(unit.root_extent()[1].source(), SourceId::from_ordinal(1));
        assert_eq!(unit.root_extent()[2].source(), SourceId::from_ordinal(2));
    });
}

#[test]
fn nested_blocks_arms_and_requires_follow_tree_depth() {
    let source = br#"fn guarded(value: own i32) -> own unit traps requires {
  check ieq<i32>(value, 0_i32) else trap "precondition";
} {
  match value {
    Some(payload: item) => {
      check ieq<i32>(item, payload) else trap "drift";
    }
    None() => {
      return unit;
    }
  }
}
"#;
    audit_source(source, |outcome| {
        assert!(
            matches!(outcome, CanonicalOutcome::Complete(_)),
            "nested canonical fixture must pass: {outcome:?}"
        );
    });
}

#[test]
fn first_gap_mismatch_uses_exact_source_or_deepest_node_location() {
    audit_source(b"fn main() -> own unit pure {}", |outcome| {
        let CanonicalOutcome::SourceIssue(issue) = outcome else {
            panic!("one-line block must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), crate::parser::SyntaxRuleV0_9::Form2);
        let CanonicalLocation::SourceNode(path, coordinate) = issue.location() else {
            panic!("inside-item gap must use SourceNode");
        };
        assert_eq!(path.components(), &[0, 0]);
        assert_eq!(coordinate.source(), SourceId::from_ordinal(0));
        assert_eq!(coordinate.start(), coordinate.end());
    });

    audit_source(b" fn main() -> own unit pure {\n}\n", |outcome| {
        let CanonicalOutcome::SourceIssue(issue) = outcome else {
            panic!("leading trivia must reject: {outcome:?}");
        };
        let CanonicalLocation::SourceBytes(coordinate) = issue.location() else {
            panic!("source-leading gap must use SourceBytes");
        };
        assert_eq!(coordinate.start().value(), 0);
        assert_eq!(coordinate.end().value(), 1);
    });

    audit_source(b"fn main() -> own unit pure {\n}", |outcome| {
        let CanonicalOutcome::SourceIssue(issue) = outcome else {
            panic!("missing final LF must reject: {outcome:?}");
        };
        let CanonicalLocation::SourceBytes(coordinate) = issue.location() else {
            panic!("source-final gap must use SourceBytes");
        };
        assert_eq!(coordinate.start(), coordinate.end());
    });

    audit_source(
        b"const first: i32 = 1_i32;\nconst second: i32 = 2_i32;\n",
        |outcome| {
            let CanonicalOutcome::SourceIssue(issue) = outcome else {
                panic!("missing inter-item blank line must reject: {outcome:?}");
            };
            assert!(matches!(
                issue.location(),
                CanonicalLocation::SourceBytes(_)
            ));
        },
    );
}

#[test]
fn zero_item_source_has_one_exact_lf_form() {
    audit_source(b"", |outcome| {
        let CanonicalOutcome::SourceIssue(issue) = outcome else {
            panic!("zero-byte source must fail FORM-2: {outcome:?}");
        };
        let CanonicalLocation::SourceBytes(coordinate) = issue.location() else {
            panic!("zero-item source mismatch must use SourceBytes");
        };
        assert_eq!(coordinate.start().value(), 0);
        assert_eq!(coordinate.end().value(), 0);
    });
}

#[test]
fn ordered_sources_stop_at_the_first_form2_mismatch() {
    let inputs = [
        SourceInput::new("first.wf", b"fn first() -> own unit pure {\n}\n"),
        SourceInput::new("second.wf", b"fn second() -> own unit pure {}"),
        SourceInput::new("third.wf", b"fn third() -> own unit pure {}"),
    ];
    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(finalized) = finalize_v0_9(parsed, FINALIZE_LIMITS) else {
            panic!("ordered bundle must finalize");
        };
        let CanonicalOutcome::SourceIssue(issue) =
            audit_canonical_v0_9(finalized, CANONICAL_LIMITS)
        else {
            panic!("second source must provide first FORM-2 mismatch");
        };
        let coordinate = match issue.location() {
            CanonicalLocation::SourceBytes(coordinate)
            | CanonicalLocation::SourceNode(_, coordinate) => coordinate,
        };
        assert_eq!(coordinate.source(), SourceId::from_ordinal(1));
    });
}

#[test]
fn tree_mutation_with_the_original_tape_cannot_publish_canonical_syntax() {
    let source = b"fn main() -> own unit pure {\n}\n";
    let inputs = [SourceInput::new("mutated.wf", source)];
    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(mut finalized) = finalize_v0_9(parsed, FINALIZE_LIMITS)
        else {
            panic!("fixture must finalize before hostile mutation");
        };
        let Some(node) = finalized
            .topology
            .nodes
            .iter_mut()
            .find(|node| node.production == whitefoot_syntax_data::ProductionV0_9::FnDecl)
        else {
            panic!("fixture must contain fn_decl");
        };
        node.production = whitefoot_syntax_data::ProductionV0_9::Item;
        assert!(matches!(
            audit_canonical_v0_9(finalized, CANONICAL_LIMITS),
            CanonicalOutcome::CompilerFailure(CanonicalCompilerFailure::InvalidFinalizedTree)
        ));
    });

    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(mut finalized) = finalize_v0_9(parsed, FINALIZE_LIMITS)
        else {
            panic!("fixture must finalize before hostile mutation");
        };
        finalized.topology.terminals[0].local_ordinal = 1;
        assert!(matches!(
            audit_canonical_v0_9(finalized, CANONICAL_LIMITS),
            CanonicalOutcome::CompilerFailure(
                CanonicalCompilerFailure::TerminalBindingDisagreement
            )
        ));
    });
}

#[test]
fn canonical_audit_resource_edges_are_explicit_and_deterministic() {
    let source = b"fn main() -> own unit pure {\n}\n";
    let cases = [
        (
            CanonicalLimit::Work,
            CanonicalLimits {
                max_work: 0,
                ..CANONICAL_LIMITS
            },
        ),
        (
            CanonicalLimit::SourceBytes,
            CanonicalLimits {
                max_source_bytes: 0,
                ..CANONICAL_LIMITS
            },
        ),
        (
            CanonicalLimit::TotalSourceBytes,
            CanonicalLimits {
                max_total_source_bytes: 0,
                ..CANONICAL_LIMITS
            },
        ),
        (
            CanonicalLimit::Gaps,
            CanonicalLimits {
                max_gaps: 0,
                ..CANONICAL_LIMITS
            },
        ),
    ];
    for (expected, limits) in cases {
        let inputs = [SourceInput::new("resource.wf", source)];
        with_parsed(&inputs, |parsed| {
            let FinalizeOutcome::Complete(finalized) = finalize_v0_9(parsed, FINALIZE_LIMITS)
            else {
                panic!("resource fixture must finalize");
            };
            let outcome = audit_canonical_v0_9(finalized, limits);
            assert!(
                matches!(
                    outcome,
                    CanonicalOutcome::ResourceFailure(
                        CanonicalResourceFailure::LimitExceeded { limit, .. }
                    ) if limit == expected
                ),
                "unexpected {expected:?} result: {outcome:?}"
            );
        });
    }

    let noncanonical = b"fn main() -> own unit pure {}";
    let inputs = [SourceInput::new("path.wf", noncanonical)];
    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(finalized) = finalize_v0_9(parsed, FINALIZE_LIMITS) else {
            panic!("path fixture must finalize");
        };
        let outcome = audit_canonical_v0_9(
            finalized,
            CanonicalLimits {
                max_path_components: 0,
                ..CANONICAL_LIMITS
            },
        );
        assert!(matches!(
            outcome,
            CanonicalOutcome::ResourceFailure(CanonicalResourceFailure::LimitExceeded {
                limit: CanonicalLimit::PathComponents,
                ..
            })
        ));
    });
}

#[test]
fn generated_trivia_mutations_never_bypass_the_exact_forest_renderer() {
    let canonical = b"const first: i32 = 1_i32;\n\nfn main() -> own unit pure {\n  let value: own i32 = 2_i32;\n  return unit;\n}\n";
    assert!(reaches_canonical_syntax(canonical));
    let trivia_positions: Vec<_> = canonical
        .iter()
        .enumerate()
        .filter_map(|(index, byte)| matches!(byte, b' ' | b'\n').then_some(index))
        .collect();
    assert!(!trivia_positions.is_empty());
    for position in trivia_positions {
        let mut removed = canonical.to_vec();
        removed.remove(position);
        assert!(!reaches_canonical_syntax(&removed));

        let mut duplicated = canonical.to_vec();
        duplicated.insert(position, canonical[position]);
        assert!(!reaches_canonical_syntax(&duplicated));

        let mut replaced = canonical.to_vec();
        replaced[position] = if canonical[position] == b' ' {
            b'\n'
        } else {
            b' '
        };
        assert!(!reaches_canonical_syntax(&replaced));
    }
}
