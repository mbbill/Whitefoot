use crate::{SourceId, SourceInput};

use super::super::{
    CanonicalCompilerFailure, CanonicalLimit, CanonicalLimits, CanonicalLocation, CanonicalOutcome,
    CanonicalResourceFailure, FinalizeOutcome, audit_canonical, finalize,
};
use super::support::{
    CANONICAL_LIMITS, FINALIZE_LIMITS, reaches_canonical_syntax, rendered_bytes, with_parsed,
};

fn audit_source(
    source: &[u8],
    audit: impl for<'classified, 'lexed, 'source> FnOnce(CanonicalOutcome<'classified, 'lexed, 'source>),
) {
    let inputs = [SourceInput::new("format.wf", source)];
    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
            panic!("complete derivation must finalize");
        };
        audit(audit_canonical(finalized, CANONICAL_LIMITS));
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
        let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
            panic!("ordered canonical bundle must finalize");
        };
        let CanonicalOutcome::Complete(unit) = audit_canonical(finalized, CANONICAL_LIMITS) else {
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
  check ieq(value, 0_i32) else trap "precondition";
} {
  match value {
    Some(payload: item) => {
      check ieq(item, payload) else trap "drift";
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
fn plain_and_variant_ensures_round_trip_with_clause_joins() {
    let source = br#"fn plain(value: own i32) -> own i32 pure ensures result {
  check ieq(result, value) else trap "post";
} {
  return value;
}

fn selected(value: own i32) -> own Result<i32, i32> pure requires {
  check ieq(value, value) else trap "pre";
} ensures Ok(value: result) {
  let same = ieq(result, value);
  check same else trap "post";
} {
  return Ok<i32, i32>(value: value);
}
"#;
    only_these_trivia_bytes_render(source);
}

#[test]
fn first_gap_mismatch_uses_exact_source_or_deepest_node_location() {
    audit_source(b"fn main() -> own unit pure {}", |outcome| {
        let CanonicalOutcome::SourceIssue(issue) = outcome else {
            panic!("one-line block must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), crate::syntax::parser::SyntaxRule::Form2);
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
        let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
            panic!("ordered bundle must finalize");
        };
        let CanonicalOutcome::SourceIssue(issue) = audit_canonical(finalized, CANONICAL_LIMITS)
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
        let FinalizeOutcome::Complete(mut finalized) = finalize(parsed, FINALIZE_LIMITS) else {
            panic!("fixture must finalize before hostile mutation");
        };
        let Some(node) = finalized
            .topology
            .nodes
            .iter_mut()
            .find(|node| node.production == crate::syntax::grammar::Production::FnDecl)
        else {
            panic!("fixture must contain fn_decl");
        };
        node.production = crate::syntax::grammar::Production::Item;
        assert!(matches!(
            audit_canonical(finalized, CANONICAL_LIMITS),
            CanonicalOutcome::CompilerFailure(CanonicalCompilerFailure::InvalidFinalizedTree)
        ));
    });

    with_parsed(&inputs, |parsed| {
        let FinalizeOutcome::Complete(mut finalized) = finalize(parsed, FINALIZE_LIMITS) else {
            panic!("fixture must finalize before hostile mutation");
        };
        finalized.topology.terminals[0].local_ordinal = 1;
        assert!(matches!(
            audit_canonical(finalized, CANONICAL_LIMITS),
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
            let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
                panic!("resource fixture must finalize");
            };
            let outcome = audit_canonical(finalized, limits);
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
        let FinalizeOutcome::Complete(finalized) = finalize(parsed, FINALIZE_LIMITS) else {
            panic!("path fixture must finalize");
        };
        let outcome = audit_canonical(
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

/// Asserts that a source is canonical and that no single-byte trivia edit is.
///
/// The renderer is held to the same fixtures: canonical bytes must render to
/// themselves, so every layout rule pinned here is pinned in both directions.
fn only_these_trivia_bytes_render(canonical: &[u8]) {
    assert!(reaches_canonical_syntax(canonical));
    assert_eq!(rendered_bytes(canonical).as_deref(), Some(canonical));
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

        // Whatever a mutation derives, rendering it lands on canonical bytes
        // and stays there. A mutation that keeps the token stream renders back
        // to `canonical`; one that changes it renders that other program. The
        // fixed point is what holds for both, so it is what is asserted.
        for mutation in [&removed, &duplicated, &replaced] {
            let Some(rendered) = rendered_bytes(mutation) else {
                continue;
            };
            assert!(reaches_canonical_syntax(&rendered));
            assert_eq!(rendered_bytes(&rendered).as_ref(), Some(&rendered));
        }
    }
}

#[test]
fn generated_trivia_mutations_never_bypass_the_exact_forest_renderer() {
    only_these_trivia_bytes_render(b"const first: i32 = 1_i32;\n\nfn main() -> own unit pure {\n  let value = 2_i32;\n  return unit;\n}\n");
}

/// The one canonical byte sequence [FN-7] states for a complete four-input
/// command entry header.
const COMMAND_ENTRY_HEADER: &[u8] = b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {";

const COMMAND_ENTRY: &[u8] = b"command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {\n  return unit;\n}\n";

#[test]
fn the_command_entry_header_renders_from_form2_without_amendment() {
    // FN-7 states that FORM-2 renders the entry header unamended: neither
    // `program_kind` nor `input_label` is line-bearing or block-bearing, and
    // the existing attachment sets join `command`, `.`, and the label tail
    // with no bytes while separating `as` from its binder by one space. The
    // mutation sweep is the reject side: no other trivia spelling renders.
    assert!(COMMAND_ENTRY.starts_with(COMMAND_ENTRY_HEADER));
    only_these_trivia_bytes_render(COMMAND_ENTRY);
}

/// The property the corpus migration rests on: a transform may produce any
/// layout it likes as long as the bytes parse, because rendering makes them
/// canonical. Each pair below holds the token stream fixed and varies only
/// trivia, so the rendered bytes must be the canonical spelling of that same
/// program rather than some other one.
#[test]
fn rendering_normalizes_any_parseable_layout_onto_canonical_bytes() {
    for (sloppy, canonical) in [
        // No trivia at all where FORM-2 requires a break.
        (
            b"fn main() -> own unit pure {}".as_slice(),
            b"fn main() -> own unit pure {\n}\n".as_slice(),
        ),
        // Leading trivia, which no canonical source carries.
        (
            b" fn main() -> own unit pure {\n}\n".as_slice(),
            b"fn main() -> own unit pure {\n}\n".as_slice(),
        ),
        // A missing final newline.
        (
            b"fn main() -> own unit pure {\n}".as_slice(),
            b"fn main() -> own unit pure {\n}\n".as_slice(),
        ),
        // Wrong indentation and a run of blank lines inside a body.
        (
            b"fn main() -> own unit pure {\n\n\n        let value = 2_i32;\n   return unit;\n}\n"
                .as_slice(),
            b"fn main() -> own unit pure {\n  let value = 2_i32;\n  return unit;\n}\n".as_slice(),
        ),
        // Two top-level items run together; FORM-2 separates them by a blank
        // line, which no amount of local spacing repair would supply.
        (
            b"const first: i32 = 1_i32;\nconst second: i32 = 2_i32;\n".as_slice(),
            b"const first: i32 = 1_i32;\n\nconst second: i32 = 2_i32;\n".as_slice(),
        ),
        // The join line, emitted rather than recognized. A migration writing
        // `if`/`else` from a `match` produces the close and the `else` with no
        // idea they share a line; the renderer is what puts them there.
        (
            b"fn main() -> own unit traps {\nlet flag = True();\nif flag {\ncheck flag else trap \"then\";\n}\nelse\n{\ncheck flag else trap \"else\";\n}\nreturn unit;\n}\n".as_slice(),
            b"fn main() -> own unit traps {\n  let flag = True();\n  if flag {\n    check flag else trap \"then\";\n  } else {\n    check flag else trap \"else\";\n  }\n  return unit;\n}\n".as_slice(),
        ),
        // A flattened `else if` chain, likewise joined by the renderer.
        (
            b"fn main() -> own unit traps {\nlet flag = True();\nif flag {\ncheck flag else trap \"a\";\n} else if flag {\ncheck flag else trap \"b\";\n} else {\ncheck flag else trap \"c\";\n}\nreturn unit;\n}\n".as_slice(),
            b"fn main() -> own unit traps {\n  let flag = True();\n  if flag {\n    check flag else trap \"a\";\n  } else if flag {\n    check flag else trap \"b\";\n  } else {\n    check flag else trap \"c\";\n  }\n  return unit;\n}\n".as_slice(),
        ),
    ] {
        assert!(!reaches_canonical_syntax(sloppy));
        assert_eq!(rendered_bytes(sloppy).as_deref(), Some(canonical));
        assert!(reaches_canonical_syntax(canonical));
    }
}

/// A source holding no item is one newline, from the emitter side too.
#[test]
fn an_item_free_source_renders_as_one_newline() {
    assert_eq!(rendered_bytes(b"").as_deref(), Some(b"\n".as_slice()));
    assert_eq!(rendered_bytes(b"\n\n\n").as_deref(), Some(b"\n".as_slice()));
}

/// FORM-2's join line. `if_stmt` and `value_if` are the only productions
/// owning two brace blocks, and the close-and-open line `} else {` is a
/// rendering no v0.22 production produced. The `requires` block's `} {` is
/// the precedent both follow.
#[test]
fn if_else_renders_its_join_line_and_indents_both_blocks() {
    // An else-free `if`: one block, ordinary break after the close.
    only_these_trivia_bytes_render(
        b"fn main() -> own unit traps {\n  let flag = True();\n  if flag {\n    check flag else trap \"then\";\n  }\n  return unit;\n}\n",
    );
    // A braced `else`: two blocks joined by `} else {` on one line.
    only_these_trivia_bytes_render(
        b"fn main() -> own unit traps {\n  let flag = True();\n  if flag {\n    check flag else trap \"then\";\n  } else {\n    check flag else trap \"else\";\n  }\n  return unit;\n}\n",
    );
    // An `else if` chain: the nested `if_stmt` owns the second block, so the
    // outer node has one pair plus an `else`, and still suppresses its break.
    only_these_trivia_bytes_render(
        b"fn main() -> own unit traps {\n  let flag = True();\n  if flag {\n    check flag else trap \"then\";\n  } else if flag {\n    check flag else trap \"chain\";\n  } else {\n    check flag else trap \"else\";\n  }\n  return unit;\n}\n",
    );
    // A `value_if` initializer delivers from both branches.
    only_these_trivia_bytes_render(
        b"fn main() -> own unit pure {\n  let flag = True();\n  let picked = if flag {\n    give 1_i32;\n  } else {\n    give 2_i32;\n  }\n  return unit;\n}\n",
    );
    // A three-deep chain renders flat: every arm sits at one indent level.
    // This is structural, not a special case. An else-position `if_stmt`
    // begins after the then-block's closing brace, so it is inside no brace
    // pair of its parent and `inside_body` leaves it at the outer format
    // depth. Nothing in the depth computation mentions `if_stmt`, and
    // `has_else` is read only when suppressing the break after a close
    // brace. Do not add a special case here: depth would then accumulate and
    // this fixture would indent each arm one level deeper.
    only_these_trivia_bytes_render(
        b"fn main() -> own unit traps {\n  let flag = True();\n  if flag {\n    check flag else trap \"a\";\n  } else if flag {\n    check flag else trap \"b\";\n  } else if flag {\n    check flag else trap \"c\";\n  } else {\n    check flag else trap \"d\";\n  }\n  return unit;\n}\n",
    );
}

#[test]
fn counted_range_attaches_its_endpoints_and_round_trips_canonically() {
    let canonical = b"fn probe(lower: own u64, upper: own u64) -> own unit pure {\n  for @range index in lower..upper {\n    break @range;\n  }\n  return unit;\n}\n";
    only_these_trivia_bytes_render(canonical);
    let sloppy = b"fn probe(lower:own u64,upper:own u64)->own unit pure{\nfor @range index in lower .. upper{\nbreak @range;\n}\nreturn unit;\n}\n";
    assert_eq!(
        rendered_bytes(sloppy).as_deref(),
        Some(canonical.as_slice())
    );
    assert_eq!(
        rendered_bytes(canonical).as_deref(),
        Some(canonical.as_slice())
    );
}
