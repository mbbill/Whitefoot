use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{
    CheckedExpression, CheckedSliceOrigin, CheckedSliceSource, CheckedStatement, CheckedType,
};
use super::{assert_rule, assert_rule_kind, assert_unsupported, with_semantics};

#[test]
fn region_substitution_does_not_implicitly_shorten_direct_view_values() {
    let prefix = "const data: array<u8, 2> =[7_u8, 9_u8];\n\nfn choose['r](first: own Slice<'r, u8>, second: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {\n  return first;\n}\n\n";
    let distinct = format!(
        "{prefix}fn main() -> status: own ExitStatus pure {{\n  region {{\n    let first = slice_of(&data);\n    region {{\n      let second = slice_of(&data);\n      let result = choose(first: first, second: second);\n    }}\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    );
    assert_rule_kind(distinct.as_bytes(), SemanticRule::Type5, |kind| {
        matches!(kind, SemanticIssueKind::TypeMismatch { .. })
    });
    let same = format!(
        "{prefix}fn main() -> status: own ExitStatus pure {{\n  region {{\n    let first = slice_of(&data);\n    let second = slice_of(&data);\n    let result = choose(first: first, second: second);\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
    );
    with_semantics(same.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
}

#[test]
fn invariant_brands_require_exact_view_types_in_either_parameter_order() {
    for reversed in [false, true] {
        let (parameters, arguments) = if reversed {
            (
                "view: own Slice<'s, u8>, marker: &Mark<'s>",
                "view: view, marker: &marker",
            )
        } else {
            (
                "marker: &Mark<'s>, view: own Slice<'s, u8>",
                "marker: &marker, view: view",
            )
        };
        let source = format!(
            "struct Mark['s] {{\n  value: u64;\n}}\n\nconst data: array<u8, 2> =[7_u8, 9_u8];\n\nfn inspect['s]({parameters}) -> result: own u64 pure {{\n  return 0_u64;\n}}\n\nfn main() -> status: own ExitStatus pure {{\n  region {{\n    let view = slice_of(&data);\n    region 'inner {{\n      let marker = Mark<'inner>(value: 1_u64);\n      region {{\n        let result = inspect({arguments});\n      }}\n    }}\n  }}\n  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_rule_kind(source.as_bytes(), SemanticRule::Type5, |kind| {
            matches!(kind, SemanticIssueKind::TypeMismatch { .. })
        });
    }
}

#[test]
fn array_views_preserve_exclusivity_and_element_domains() {
    let source = r#"fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 2>(0_u8);
  region {
    let view = mut_slice_of(&uniq values);
    set view[0_u64] = 1_u8;
    let seen = view[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    for (source, rule) in [
        (
            source.replace("mut_slice_of(&uniq values)", "slice_of(&values)"),
            SemanticRule::Set1,
        ),
        (
            source.replace("set view[0_u64]", "set view[2_u64]"),
            SemanticRule::Op4,
        ),
        (
            source.replace("set view[0_u64]", "set values[0_u64]"),
            SemanticRule::Own5,
        ),
        (
            source.replace(
                "set view[0_u64] = 1_u8;",
                "let other = mut_slice_of(&uniq values);",
            ),
            SemanticRule::Own5,
        ),
        (
            source.replace("mut_slice_of(&uniq values)", "mut_slice_of(&values)"),
            SemanticRule::Type5,
        ),
    ] {
        assert_rule_kind(source.as_bytes(), rule, |_| true);
    }
    let constant = source
        .replace("  let values = array_new::<u8, 2>(0_u8);\n", "")
        .replace("fn", "const values: array<u8, 2> =[0_u8, 0_u8];\n\nfn");
    assert_rule_kind(constant.as_bytes(), SemanticRule::Const2, |_| true);
}

#[test]
fn borrowed_storage_views_preserve_parent_permissions() {
    let source = r#"fn relay['r](view: own Slice<'r, u64>) -> result: own Slice<'r, u64> pure contract {
  ensures len_of(result) == len_of(view);
} {
  return view;
}

fn first(view: own Slice<u64>) -> result: own u64 reads(view) contract {
  requires len_of(view) == 2_u64;
} {
  return view[0_u64];
}

fn inspect(values: &array<u64, 2>) -> result: own u64 reads(values) {
  region {
    let view = slice_of(&deref(values));
    return first(view: view);
  }
}

fn edit(values: &uniq array<u64, 2>) -> (before: own u64, after: own u64) reads(values), writes(values) {
  region {
    let view = slice_of(&deref(values));
    let before = first(view: view);
    let duplicate = view;
    let observed = duplicate[1_u64];
    set deref(values)[0_u64] = 19_u64;
    region {
      let writer = mut_slice_of(&uniq deref(values));
      set writer[1_u64] = 23_u64;
      let after = writer[1_u64];
      return before, after;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    let relayed = source.replace(
        "let duplicate = view;",
        "let duplicate = relay(view: view);",
    );
    with_semantics(relayed.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    for source in [source.to_owned(), relayed] {
        let early_write = source.replace(
            "let observed = duplicate[1_u64];",
            "set deref(values)[1_u64] = 17_u64;\n    let observed = duplicate[1_u64];",
        );
        assert_rule_kind(early_write.as_bytes(), SemanticRule::Own5, |kind| {
            matches!(kind, SemanticIssueKind::BorrowConflict)
        });
        let second_writer = source.replace(
            "set writer[1_u64] = 23_u64;",
            "let second = mut_slice_of(&uniq deref(values));\n      set writer[1_u64] = 23_u64;",
        );
        assert_rule_kind(second_writer.as_bytes(), SemanticRule::Own5, |kind| {
            matches!(kind, SemanticIssueKind::BorrowConflict)
        });
    }
}

#[test]
fn borrowed_array_views_preserve_exact_brand_parameters() {
    let source = br#"struct Mark['s] {
  value: u64;
}

const data: array<u8, 2> =[7_u8, 9_u8];

fn inspect['s](marker: &Mark<'s>, view: own Slice<'s, u8>) -> result: own u64 reads(view) {
  return len_of(view);
}

fn main() -> status: own ExitStatus pure {
  region {
    let parent = &data;
    region 'inner {
      let view = slice_of(&deref(parent));
      let marker = Mark<'inner>(value: 1_u64);
      region {
        let result = inspect(marker: &marker, view: view);
        if result == 2_u64 {
          return exit_status(code: 0_u8);
        }
      }
    }
  }
  return exit_status(code: 1_u8);
}
"#;
    // The former unsupported probe now reaches VIEW-2 through the ordinary
    // child-borrow path. The source and its exact brand relation are retained.
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    let direct = std::str::from_utf8(source)
        .unwrap()
        .replace("slice_of(&deref(parent))", "slice_of(&data)");
    with_semantics(direct.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        )
    });
}

#[test]
fn selected_view_results_preserve_each_borrowed_storage_parent() {
    let source = r#"fn choose['r](first: own Slice<'r, u64>, second: own Slice<'r, u64>, left: own Bool) -> result: own Slice<'r, u64> pure contract {
  requires len_of(first) == 2_u64;
  requires len_of(second) == 2_u64;
  ensures len_of(result) == 2_u64;
} {
  if left {
    return first;
  } else {
    return second;
  }
}

fn read_first(view: own Slice<u64>) -> result: own u64 reads(view) contract {
  requires len_of(view) == 2_u64;
} {
  return view[0_u64];
}

fn inspect(left: &uniq array<u64, 2>, right: &uniq array<u64, 2>, select_first: own Bool) -> result: own u64 reads(left, right), writes(left, right) {
  region {
    let a = slice_of(&deref(left));
    let b = slice_of(&deref(right));
    let selected = choose(first: a, second: b, left: select_first);
    let observed = read_first(view: selected);
    set deref(left)[0_u64] = 19_u64;
    set deref(right)[0_u64] = 23_u64;
    return observed;
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    for parent in ["left", "right"] {
        let early_write = source.replace(
            "let observed = read_first(view: selected);",
            &format!(
                "set deref({parent})[1_u64] = 31_u64;\n    let observed = read_first(view: selected);"
            ),
        );
        assert_rule_kind(early_write.as_bytes(), SemanticRule::Own5, |kind| {
            matches!(kind, SemanticIssueKind::BorrowConflict)
        });
    }
}

#[test]
fn borrowed_storage_child_views_preserve_live_and_dead_relay_copies() {
    let source = r#"fn child['r](view: &uniq MutSlice<'r, u64>) -> result: own Slice<'r, u64> pure contract {
  requires len_of(deref(view)) == 2_u64;
  ensures len_of(result) == 2_u64;
} {
  let result = slice_of(&'r deref(view));
  return result;
}

fn relay['r](view: own Slice<'r, u64>) -> result: own Slice<'r, u64> pure contract {
  ensures len_of(result) == len_of(view);
} {
  return view;
}

fn inspect(values: &uniq array<u64, 2>) -> result: own u64 reads(values), writes(values) {
  region {
    let writer = mut_slice_of(&uniq deref(values));
    region {
      let parent = &uniq writer;
      region {
        let shared = FORM;
        let copied = relay(view: shared);
        let observed = copied[0_u64];
        set deref(parent)[1_u64] = 23_u64;
        return observed;
      }
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for formation in [
        "slice_of(&deref(parent))",
        "child(view: &uniq deref(parent))",
    ] {
        let source = source.replace("FORM", formation);
        with_semantics(source.as_bytes(), |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "{outcome:?}"
            );
        });
        let early_write = source.replace(
            "let observed = copied[0_u64];",
            "set deref(parent)[1_u64] = 17_u64;\n        let observed = copied[0_u64];",
        );
        assert_rule_kind(early_write.as_bytes(), SemanticRule::Own5, |kind| {
            matches!(kind, SemanticIssueKind::BorrowConflict)
        });
    }
}

#[test]
fn moved_formal_views_keep_their_shared_children_frozen_across_calls() {
    let source = r#"fn child['r](view: &uniq MutSlice<'r, u64>) -> result: own Slice<'r, u64> pure contract {
  requires len_of(deref(view)) == 2_u64;
  ensures len_of(result) == 2_u64;
} {
  let result = slice_of(&'r deref(view));
  return result;
}

fn overwrite(view: own MutSlice<u64>) -> result: own unit writes(view) contract {
  requires len_of(view) == 2_u64;
} {
  set view[0_u64] = 19_u64;
  return unit;
}

fn update(view: &uniq MutSlice<u64>) -> result: own unit writes(view) contract {
  requires len_of(deref(view)) == 2_u64;
} {
  set deref(view)[0_u64] = 19_u64;
  return unit;
}

fn accept_view(view: own MutSlice<u64>) -> result: own unit pure {
  return unit;
}

fn inspect(view: own MutSlice<u64>) -> result: own u64 reads(view), writes(view) contract {
  requires len_of(view) == 2_u64;
} {
  let writer = move view;
  region {
    let shared = child(view: &uniq writer);
    let done = CALL;
    let observed = shared[0_u64];
    return observed;
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for (call, effects) in [
        ("overwrite(view: move writer)", "reads(view), writes(view)"),
        ("update(view: &uniq writer)", "reads(view), writes(view)"),
        ("update(view: move parent)", "reads(view), writes(view)"),
        ("accept_view(view: move writer)", "reads(view)"),
    ] {
        let source = source.replace("CALL", call).replace(
            "result: own u64 reads(view), writes(view) contract",
            &format!("result: own u64 {effects} contract"),
        );
        let source = if call == "update(view: move parent)" {
            source.replace(
                "  let writer = move view;\n  region {\n    let shared = child(view: &uniq writer);\n    let done = update(view: move parent);\n    let observed = shared[0_u64];\n    return observed;\n  }",
                "  let writer = move view;\n  region {\n    let parent = &uniq writer;\n    region {\n      let shared = slice_of(&deref(parent));\n      let done = update(view: move parent);\n      let observed = shared[0_u64];\n      return observed;\n    }\n  }",
            )
        } else {
            source
        };
        assert_rule_kind(source.as_bytes(), SemanticRule::Own5, |kind| {
            matches!(kind, SemanticIssueKind::BorrowConflict)
        });
        let indent = if call == "update(view: move parent)" {
            "      "
        } else {
            "    "
        };
        let last_use = source.replace(
            &format!("let done = {call};\n{indent}let observed = shared[0_u64];"),
            &format!("let observed = shared[0_u64];\n{indent}let done = {call};"),
        );
        with_semantics(last_use.as_bytes(), |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "{call}: {outcome:?}"
            );
        });
        let wrong_effects = if effects == "reads(view)" {
            last_use.replace(
                "result: own u64 reads(view) contract",
                "result: own u64 reads(view), writes(view) contract",
            )
        } else {
            last_use.replace(
                "result: own u64 reads(view), writes(view) contract",
                "result: own u64 reads(view) contract",
            )
        };
        if effects == "reads(view)" {
            assert_rule_kind(wrong_effects.as_bytes(), SemanticRule::Eff2, |kind| {
                matches!(kind, SemanticIssueKind::EffectMismatch { extra, .. }
                    if extra == &["writes(view)".to_owned()])
            });
        } else {
            assert_rule_kind(wrong_effects.as_bytes(), SemanticRule::Eff2, |kind| {
                matches!(kind, SemanticIssueKind::EffectMismatch { missing, .. }
                    if missing == &["writes(view)".to_owned()])
            });
        }
    }
}

#[test]
fn slices_retain_type_source_and_access_operations() {
    let source = br#"const bytes: FixedVector<u8, 2> =[4_u8, 9_u8];

fn first(values: own Slice<u8>) -> result: own u8 reads(values) {
  let length = len_of(values);
  let nonempty = 0_u64 < length;
  if nonempty {
    return values[0_u64];
  } else {
    return 0_u8;
  }
}

fn main() -> status: own ExitStatus pure {
  region {
    let values = slice_of(&bytes);
    let value = first(values: values);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("slice program must check: {outcome:?}");
        };
        let first = &checked.data.functions[0];
        assert!(matches!(first.parameters[0].ty, CheckedType::Slice { .. }));
        assert!(matches!(
            first.body.as_deref().expect("WF body")[0],
            CheckedStatement::Let {
                value: CheckedExpression::SliceMeasure { .. },
                ..
            }
        ));
        let CheckedStatement::Match { arms, .. } = &first.body.as_deref().expect("WF body")[2]
        else {
            panic!("the explicit nonempty guard must remain a checked branch");
        };
        assert!(arms.iter().any(|arm| matches!(
            arm.body.first(),
            Some(CheckedStatement::Return {
                value: CheckedExpression::SliceIndex { .. },
                ..
            })
        )));

        let main = &checked.data.functions[1];
        let CheckedStatement::Region { body, .. } = &main.body.as_deref().expect("WF body")[0]
        else {
            panic!("main must retain the view region");
        };
        assert!(matches!(
            body[0],
            CheckedStatement::Let {
                value: CheckedExpression::SliceOf {
                    source: CheckedSliceSource::Array { .. },
                    ..
                },
                ..
            }
        ));
    });
}

#[test]
fn incoming_slice_reads_require_their_origin_effect() {
    let source = br#"fn invalid(values: own Slice<u8>) -> result: own u8 pure {
  return values[0_u64];
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("missing slice read effect must be rejected: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Eff2);
    });
}

#[test]
fn moved_owner_borrows_and_slices_keep_the_incoming_formal_effect_path() {
    let source = br#"fn touch_after_move(value: own FixedVector<u8, 2>) -> result: own u8 reads(value), writes(value) {
  let moved = move value;
  region {
    let holder = &uniq moved;
    let spare = len_of(deref(holder));
    let nonempty = 0_u64 < spare;
    if nonempty {
      let byte = deref(holder)[0_u64];
      set deref(holder)[0_u64] = byte;
      return byte;
    } else {
      return 0_u8;
    }
  }
}

fn slice_after_move(value: own FixedVector<u8, 2>) -> result: own u8 reads(value) contract {
  requires head_of(value) <= room_of(value);
} {
  let moved = move value;
  region {
    let view = slice_of(&moved);
    let spare = len_of(view);
    let nonempty = 0_u64 < spare;
    if nonempty {
      return view[0_u64];
    } else {
      return 0_u8;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "a local lifetime must not erase the moved formal owner: {outcome:?}"
        );
    });
}

/// [OWN-5, PROV-3] a live loan refuses a write to and a move of its origin,
/// and a shared view's loan is live exactly while that view is still used.
///
/// Every program here uses the view *after* the offending statement, which is
/// what makes the loan live there. The same programs without that later use
/// are the accepts `a_copy_view_loan_ends_at_its_last_use` records: [S27]
/// made the shared view copy, so it is consumed by nothing and its loan ends
/// at its last use rather than at the end of its named data region.
#[test]
fn a_live_slice_prevents_writes_and_moves_of_its_source() {
    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region {
    let window = slice_of(&values);
    set values[0_u64] = 1_u8;
    let seen = window[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region {
    let window = slice_of(&values);
    let taken = move values;
    let seen = window[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

/// VIEW-2 applies to children of formal views as well as local storage.
#[test]
fn formal_view_children_release_at_the_last_use_of_all_descriptors() {
    let source = r#"fn child['r](view: &uniq MutSlice<'r, u8>) -> result: own Slice<'r, u8> pure contract {
  requires len_of(deref(view)) == 1_u64;
  ensures len_of(result) == 1_u64;
} {
  let result = slice_of(&'r deref(view));
  return result;
}

fn reuse(view: &uniq MutSlice<u8>) -> result: own u8 reads(view), writes(view) contract {
  requires len_of(deref(view)) == 1_u64;
} {
  region {
    let shared = FORM;
    COPY
    let previous = shared[0_u64];
    BEFORE
    set deref(view)[0_u64] = 9_u8;
    AFTER
    return previous;
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for formation in ["slice_of(&deref(view))", "child(view: &uniq deref(view))"] {
        for (copy, before, after, live) in [
            ("", "", "", false),
            (
                "let copied = shared;",
                "let last = copied[0_u64];",
                "",
                false,
            ),
            ("", "", "let last = shared[0_u64];", true),
            (
                "let copied = shared;",
                "",
                "let last = copied[0_u64];",
                true,
            ),
        ] {
            let source = source
                .replace("FORM", formation)
                .replace("    COPY\n", &optional_statement(copy))
                .replace("    BEFORE\n", &optional_statement(before))
                .replace("    AFTER\n", &optional_statement(after));
            if live {
                assert_rule(
                    source.as_bytes(),
                    SemanticRule::Own5,
                    SemanticIssueKind::BorrowConflict,
                );
            } else {
                with_semantics(source.as_bytes(), |outcome| {
                    assert!(
                        matches!(outcome, SemanticOutcome::Complete(_)),
                        "all shared descriptors ended before the write: {formation}: {outcome:?}"
                    );
                });
            }
        }
    }

    fn optional_statement(statement: &str) -> String {
        if statement.is_empty() {
            String::new()
        } else {
            format!("    {statement}\n")
        }
    }
}

#[test]
fn local_holders_of_formal_views_keep_every_live_child_copy_frozen() {
    for (before, after, live) in [
        ("      let last = copied[0_u64];\n", "", false),
        ("", "      let last = copied[0_u64];\n", true),
    ] {
        let source = r#"fn reuse(view: own MutSlice<u8>) -> result: own u8 reads(view), writes(view) contract {
  requires len_of(view) == 1_u64;
} {
  region {
    let holder = &uniq view;
    region {
      let shared = slice_of(&deref(holder));
      let copied = shared;
      let previous = shared[0_u64];
BEFORE      set deref(holder)[0_u64] = 9_u8;
AFTER      return previous;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#
        .replace("BEFORE", before)
        .replace("AFTER", after);
        if live {
            assert_rule(
                source.as_bytes(),
                SemanticRule::Own5,
                SemanticIssueKind::BorrowConflict,
            );
        } else {
            with_semantics(source.as_bytes(), |outcome| {
                assert!(
                    matches!(outcome, SemanticOutcome::Complete(_)),
                    "a local holder must use the child's formal backing origin: {outcome:?}"
                );
            });
        }
    }
}

/// [PROV-3, OWN-5] a copy view's loan ends at its last use, and the region
/// that named it is the ceiling rather than the extent.
///
/// Before [S27] every shared loan lived to the end of its named data region,
/// which is what the two rejections below measured. The classification made
/// the shared view copy, so it is consumed by nothing and its loan ends where
/// its own liveness does: a use after the offending statement keeps the loan
/// live — in an enclosing region and out of a branch alike — and a view with
/// no later use leaves the storage writable at the next statement.
#[test]
fn slice_loans_live_until_their_last_use_inside_their_named_data_region() {
    // A view formed in an inner block, naming the outer region, is the
    // program the region extent used to refuse. Its binding cannot be used
    // after that block at all, so its last use is inside it and the loan
    // cannot reach the write [PROV-3].
    let inner_view = br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region 'outer {
    region {
      let view = slice_of(&'outer values);
      let seen = view[0_u64];
    }
    set values[0_u64] = 1_u8;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(inner_view, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "a view whose binding is gone has no later use: {outcome:?}"
        );
    });

    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  let take_view = True();
  region {
    let view = slice_of(&values);
    if take_view {
      let seen = view[0_u64];
    }
    set values[0_u64] = 1_u8;
    let after = view[1_u64];
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    // The loan ends at the view's last use, so the write the region used to
    // refuse is admitted inside that same region [PROV-3].
    let dead_view = br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region {
    let view = slice_of(&values);
    let seen = view[0_u64];
    set values[0_u64] = 1_u8;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(dead_view, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "a copy view's loan must end at its last use: {outcome:?}"
        );
    });

    let ended_region = br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region {
    let view = slice_of(&values);
  }
  set values[0_u64] = 1_u8;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(ended_region, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the shared borrow must end with its named data region: {outcome:?}"
        );
    });
}

#[test]
fn slice_loans_follow_structured_break_region_exits() {
    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region {
    let view = slice_of(&values);
    loop @once {
      break @once;
    }
    set values[0_u64] = 1_u8;
    let seen = view[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let ended_on_break = br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  loop @once {
    let view = slice_of(&values);
    break @once;
  }
  set values[0_u64] = 1_u8;
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(ended_on_break, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "breaking out of the data region must end its shared borrow: {outcome:?}"
        );
    });

    // [OWN-11] the body's own region is what an elided borrow takes, so the
    // outer region has to be named for this fault to be written at all.
    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region 'r {
    loop @once {
      let view = slice_of(&'r values);
      break @once;
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own11,
        SemanticIssueKind::BorrowRegionOutsideLoop {
            mechanical_fix: "introduce the borrow region inside the enclosing loop body",
        },
    );
}

#[test]
fn consuming_a_projection_respects_loans_of_residual_fields() {
    const OWNER: &str = r#"struct Owner {
  source: FixedVector<u8, 1>;
  sibling: FixedVector<u8, 1>;
}

"#;

    let direct_move = format!(
        r#"{OWNER}fn main() -> status: own ExitStatus pure {{
  let source_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq source_empty, value: 0_u8);
  }}
  let source = move source_empty;
  let sibling_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq sibling_empty, value: 0_u8);
  }}
  let sibling = move sibling_empty;
  let owner = Owner(source: move source, sibling: move sibling);
  region {{
    let view = slice_of(&owner.source);
    let taken = move owner.sibling;
    let seen = view[0_u64];
  }}
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_rule(
        direct_move.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let call = format!(
        r#"{OWNER}fn consume(value: own FixedVector<u8, 1>) -> result: own unit pure {{
  return unit;
}}

fn main() -> status: own ExitStatus pure {{
  let source_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq source_empty, value: 0_u8);
  }}
  let source = move source_empty;
  let sibling_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq sibling_empty, value: 0_u8);
  }}
  let sibling = move sibling_empty;
  let owner = Owner(source: move source, sibling: move sibling);
  region {{
    let view = slice_of(&owner.source);
    consume(value: move owner.sibling);
    let seen = view[0_u64];
  }}
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_rule(
        call.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let matched = r#"enum Slot {
  Full(value: FixedVector<u8, 1>);
  Empty();
}

struct Owner {
  source: FixedVector<u8, 1>;
  sibling: Slot;
}

fn main() -> status: own ExitStatus pure {
  let source_empty = fixed_vector::<u8, 1>();
  region {
    place_back(vector: &uniq source_empty, value: 0_u8);
  }
  let source = move source_empty;
  let sibling_value_empty = fixed_vector::<u8, 1>();
  region {
    place_back(vector: &uniq sibling_value_empty, value: 0_u8);
  }
  let sibling_value = move sibling_value_empty;
  let sibling = Full(value: move sibling_value);
  let owner = Owner(source: move source, sibling: move sibling);
  region {
    let view = slice_of(&owner.source);
    match owner.sibling {
      Full(value: item) => {
      }
      Empty() => {
      }
    }
    let seen = view[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(
        matched.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let given = format!(
        r#"{OWNER}fn main() -> status: own ExitStatus pure {{
  let source_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq source_empty, value: 0_u8);
  }}
  let source = move source_empty;
  let sibling_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq sibling_empty, value: 0_u8);
  }}
  let sibling = move sibling_empty;
  let owner = Owner(source: move source, sibling: move sibling);
  let spare_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq spare_empty, value: 0_u8);
  }}
  let spare = move spare_empty;
  let choose_owner = True();
  region {{
    let view = slice_of(&owner.source);
    let selected = if choose_owner {{
      give move owner.sibling;
    }} else {{
      give move spare;
    }}
    let seen = view[0_u64];
  }}
  return exit_status(code: 0_u8);
}}
"#
    );
    assert_rule(
        given.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let propagated = r#"struct Owner {
  source: FixedVector<u8, 1>;
  result: Result<u8, Overflow>;
}

fn invalid(owner: own Owner) -> result: own Result<unit, Overflow> pure {
  region {
    let view = slice_of(&owner.source);
    let value = propagate owner.result;
    let seen = view[0_u64];
  }
  return Ok<unit, Overflow>(value: unit);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_rule(
        propagated.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let ended_region = format!(
        r#"{OWNER}fn main() -> status: own ExitStatus pure {{
  let source_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq source_empty, value: 0_u8);
  }}
  let source = move source_empty;
  let sibling_empty = fixed_vector::<u8, 1>();
  region {{
    place_back(vector: &uniq sibling_empty, value: 0_u8);
  }}
  let sibling = move sibling_empty;
  let owner = Owner(source: move source, sibling: move sibling);
  region {{
    let view = slice_of(&owner.source);
  }}
  let taken = move owner.sibling;
  return exit_status(code: 0_u8);
}}
"#
    );
    with_semantics(ended_region.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "a residual-field move must be restored after the loan region ends: {outcome:?}"
        );
    });
}

/// [SET-1, VIEW-1] a target path traverses a view exactly at exclusive loan
/// strength: the shared view refuses the element write and the exclusive one
/// performs it.
#[test]
fn a_shared_view_is_no_set_target_and_an_exclusive_view_is() {
    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  region {
    let window = slice_of(&values);
    set window[0_u64] = 1_u8;
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Set1,
        SemanticIssueKind::InvalidSetTarget {
            root_class: "shared view".to_owned(),
            required_classes: "live own storage, a live usable &uniq referent, or an exclusive view",
        },
    );
    // Keep the store-resident case alongside the inline-owned-place tests:
    // both storage classes use the same exclusive view rule.
    with_semantics(
        br#"fn main() -> status: own ExitStatus pure {
  region 'a {
    let workspace = arena_frame::<8, 8, 'a>();
    region {
      let values = arena_vector_proved::<u8>(store: &uniq workspace, count: 2_u64);
      for @fill (
        at in 0_u64..2_u64,
        invariant grown: len_of(values) >= at,
        invariant spare: room_of(values) + at >= 2_u64,
        invariant flat: head_of(values) <= 0_u64
      ) {
        place_back(vector: &uniq values, value: 0_u8);
      }
      region {
        let window = mut_slice_of(&uniq values);
        set window[0_u64] = 1_u8;
        let seen = window[0_u64];
        if seen == 1_u8 {
        } else {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "an element write through an exclusive view is admitted: {outcome:?}"
            );
        },
    );
}

#[test]
fn slice_formation_enforces_storage_duration_and_explicit_boundaries() {
    assert_rule_kind(
        br#"fn invalid['caller](anchor: &'caller u8) -> result: &'caller u8 pure {
  let values_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq values_empty, value: 0_u8);
  }
  let values_0 = move values_empty;
  region {
    place_back(vector: &uniq values_0, value: 0_u8);
  }
  let values = move values_0;
  let window = slice_of(&'caller values);
  return anchor;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
    assert_unsupported(
        br#"struct Item {
  value: u8;
}

fn observe(values: own Slice<Item>) -> result: own unit pure {
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::CompositeValues,
    );
    let borrowed_run = br#"fn invalid(values: &FixedVector<u8, 2>) -> result: own unit pure {
  region {
    let window = slice_of(&deref(values));
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // Formation through a storage holder is implemented. This unchanged
    // source still owes VIEW-2's non-wrap premise, now reached at BLK-0.
    assert_rule_kind(borrowed_run, SemanticRule::Blk0, |kind| {
        matches!(kind, SemanticIssueKind::UndischargedKernelRequirement(_))
    });
    let contiguous = std::str::from_utf8(borrowed_run).unwrap().replace(
        "fn invalid(values: &FixedVector<u8, 2>) -> result: own unit pure {",
        "fn invalid(values: &FixedVector<u8, 2>) -> result: own unit pure contract {\n  requires head_of(deref(values)) <= room_of(deref(values));\n} {",
    );
    with_semantics(contiguous.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    assert_rule_kind(
        br#"fn invalid['r](values: own FixedVector<u8, 2>) -> result: own Slice<'r, u8> pure {
  return slice_of(&'r values);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own10,
        |kind| matches!(kind, SemanticIssueKind::InvalidBorrowLifetime { .. }),
    );
}

/// [TYPE-5] `slice_of` is outside the retained-argument class, so it carries
/// no written argument at all: the region comes from the operand's own borrow
/// and the element from the place it views. Both halves are asserted, because
/// a fix that only stopped demanding the argument would leave the derivation
/// untested, and one that only derived would not reject the deleted form.
#[test]
fn slice_of_derives_its_region_and_rejects_a_written_argument() {
    let source = br#"fn main() -> status: own ExitStatus pure {
  let data = fixed_vector::<u8, 4>();
  for @fill_data (
    at in 0_u64..4_u64,
    invariant grown: len_of(data) >= at,
    invariant spare: room_of(data) + at >= 4_u64,
    invariant flat: head_of(data) <= 0_u64
  ) {
    place_back(vector: &uniq data, value: 0_u8);
  }
  region {
    region {
      let view = slice_of(&data);
      let length = len_of(view);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(_) = outcome else {
            panic!("the argument-free form must check: {outcome:?}");
        };
    });

    // The derived region is the borrow's, not merely *a* region in scope: the
    // same source with the outer region borrowed instead must reject, because
    // `'outer` outlives the binding the view is taken from is not the point —
    // the loan is keyed on the region the borrow writes.
    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let data = fixed_vector::<u8, 4>();
  for @fill_data (
    at in 0_u64..4_u64,
    invariant grown: len_of(data) >= at,
    invariant spare: room_of(data) + at >= 4_u64,
    invariant flat: head_of(data) <= 0_u64
  ) {
    place_back(vector: &uniq data, value: 0_u8);
  }
  region {
    let view = slice_of(&data);
    let taken = move data;
    let seen = view[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    // [OP-1] the deleted form is the rejection, on the same footing as a
    // written argument on any other de-argumented row. So A1's deletion,
    // correct on every legal call, would remove the very violation this
    // asserts — the `derivation.rs:224` class.
    // The written `<'view, u8>` IS the subject and must stay written.
    assert_rule(
        br#"fn main() -> status: own ExitStatus pure {
  let data = fixed_vector::<u8, 4>();
  for @fill_data (
    at in 0_u64..4_u64,
    invariant grown: len_of(data) >= at,
    invariant spare: room_of(data) + at >= 4_u64,
    invariant flat: head_of(data) <= 0_u64
  ) {
    place_back(vector: &uniq data, value: 0_u8);
  }
  region 'view {
    slice_of::<'view, u8>(&data);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}

#[test]
fn returned_slices_keep_signature_ceilings_and_substituted_call_origins() {
    let source = br#"fn pass['r](value: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  return value;
}

fn choose['r](take_left: own Bool, left: own Slice<'r, u8>, right: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  if take_left {
    return left;
  } else {
    return right;
  }
}

fn main() -> status: own ExitStatus pure {
  let left_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq left_empty, value: 11_u8);
  }
  let left_0 = move left_empty;
  region {
    place_back(vector: &uniq left_0, value: 11_u8);
  }
  let left = move left_0;
  let right_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq right_empty, value: 29_u8);
  }
  let right_0 = move right_empty;
  region {
    place_back(vector: &uniq right_0, value: 29_u8);
  }
  let right = move right_0;
  region {
    let pass_source = slice_of(&left);
    let passed = pass(value: pass_source);
    let passed_room = len_of(passed);
    let passed_ok = 0_u64 < passed_room;
    if passed_ok {
    } else {
      return exit_status(code: 1_u8);
    }
    let passed_value = passed[0_u64];
    let left_source = slice_of(&left);
    let right_source = slice_of(&right);
    let take_left = False();
    let selected = choose(take_left: take_left, left: left_source, right: right_source);
    let selected_room = len_of(selected);
    let selected_ok = 0_u64 < selected_room;
    if selected_ok {
    } else {
      return exit_status(code: 2_u8);
    }
    let selected_value = selected[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("returned slices must check: {outcome:?}");
        };
        assert_eq!(checked.data.functions[0].slice_return_ceiling.len(), 2);
        assert_eq!(checked.data.functions[1].slice_return_ceiling.len(), 3);
        assert!(matches!(
            checked.data.functions[0].slice_return_ceiling[0],
            CheckedSliceOrigin::ImmutableConst
        ));

        // The two runs are built by `fixed_vector` plus two `place_back`s
        // each. Each unit call has its own region before the owning alias,
        // so the view region is main's eleventh statement.
        let CheckedStatement::Region { body, .. } =
            &checked.data.functions[2].body.as_deref().expect("WF body")[10]
        else {
            panic!("main must retain the slice region");
        };
        let CheckedStatement::Let {
            value:
                CheckedExpression::UserCall {
                    slice_origins: passed,
                    ..
                },
            ..
        } = &body[1]
        else {
            panic!("pass-through call must retain slice origins");
        };
        assert_eq!(passed.len(), 2);
        assert_eq!(
            passed
                .iter()
                .filter(|origin| matches!(origin, CheckedSliceOrigin::SourcePlace { .. }))
                .count(),
            1
        );

        let CheckedStatement::Let {
            value:
                CheckedExpression::UserCall {
                    slice_origins: selected,
                    ..
                },
            ..
        } = &body[9]
        else {
            panic!("choice call must retain every permitted slice origin");
        };
        assert_eq!(selected.len(), 3);
        assert_eq!(
            selected
                .iter()
                .filter(|origin| matches!(origin, CheckedSliceOrigin::SourcePlace { .. }))
                .count(),
            2
        );
    });
}

#[test]
fn returned_slice_origins_drive_effects_and_alias_conflicts() {
    let wrapper = br#"fn pass['r](value: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  return value;
}

fn first(value: own Slice<u8>) -> result: own u8 reads(value) {
  let returned = pass(value: value);
  let spare = len_of(returned);
  let ok = 0_u64 < spare;
  if ok {
    return returned[0_u64];
  } else {
    return 0_u8;
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(wrapper, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "wrapper reads must retain the incoming slice effect: {outcome:?}"
        );
    });

    assert_rule(
        br#"fn choose['r](take_left: own Bool, left: own Slice<'r, u8>, right: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  if take_left {
    return left;
  } else {
    return right;
  }
}

fn main() -> status: own ExitStatus pure {
  let left_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq left_empty, value: 0_u8);
  }
  let left_0 = move left_empty;
  region {
    place_back(vector: &uniq left_0, value: 0_u8);
  }
  let left = move left_0;
  let right_empty = fixed_vector::<u8, 2>();
  region {
    place_back(vector: &uniq right_empty, value: 0_u8);
  }
  let right_0 = move right_empty;
  region {
    place_back(vector: &uniq right_0, value: 0_u8);
  }
  let right = move right_0;
  region {
    let left_view = slice_of(&left);
    let right_view = slice_of(&right);
    let take_left = True();
    let selected = choose(take_left: take_left, left: left_view, right: right_view);
    set right[0_u64] = 1_u8;
    let seen = selected[0_u64];
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    // [OWN-12] compares the resolved places two argument positions reach. A
    // view parameter is one binding of its declaration, and the region its
    // elided type carries is a region of its own [FORM-8] that no other
    // position of the declaration names, so forwarding it beside a unique
    // borrow of a different binding overlaps nothing.
    with_semantics(
        br#"fn consume(view: own Slice<u8>, output: &uniq MutSlice<u8>) -> result: own unit pure {
  return unit;
}

fn wrapper(view: own Slice<u8>, output: &uniq MutSlice<u8>) -> result: own unit pure {
  return consume(view: view, output: move output);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "forwarding a view beside a unique borrow of another binding is no overlap: {outcome:?}"
            );
        },
    );

    // The overlap is refused where it is written: the caller that forms the
    // view over the very storage it hands to the unique position reaches one
    // place twice, once uniquely.
    //
    // Keep this legacy descriptor control beside the run/view overlap cases:
    // the shared view and unique actual must resolve to the same storage.
    assert_rule(
        br#"fn consume(view: own Slice<u8>, output: &uniq buffer<u8>) -> result: own unit pure {
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let bytes = buffer_new(2_u64, 0_u8);
  region {
    let view = slice_of(&bytes);
    let done = consume(view: view, output: &uniq bytes);
  }
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn slice_value_matches_and_borrowed_slice_results_are_rejected() {
    assert_rule(
        br#"fn choose['r](take_left: own Bool, left: own Slice<'r, u8>, right: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure {
  let selected = if take_left {
    give left;
  } else {
    give right;
  }
  return move selected;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::SliceValueMatch {
            // v0.22 wording said "a match statement whose arms"; v0.23 extends
            // the prohibition to `value_if`, so the fix names both forms. The
            // rule and the kind are unchanged — only the mechanical fix's
            // prose follows the delta, and this assertion never reached it
            // before because the ownership join stopped first.
            mechanical_fix: "use a match or if statement whose branches return the slice directly, or call helpers with direct slice results",
        },
    );
    assert_rule(
        br#"fn invalid['descriptor, 'data](value: &'descriptor Slice<'data, u8>) -> result: &'descriptor Slice<'data, u8> pure {
  return value;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn1,
        SemanticIssueKind::BorrowedSliceResult {
            mechanical_fix: "return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor",
        },
    );

    let borrowed_input = br#"fn first(value: &Slice<u8>) -> result: own u8 reads(value) {
  let spare = len_of(deref(value));
  let ok = 0_u64 < spare;
  if ok {
    return deref(value)[0_u64];
  } else {
    return 0_u8;
  }
}

fn wrapper(value: &Slice<u8>) -> result: own u8 reads(value) {
  return first(value: value);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(borrowed_input, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "descriptor and underlying slice provenance must both survive: {outcome:?}"
        );
    });
}

#[test]
fn copied_and_returned_own_slice_descriptors_keep_referent_read_effects() {
    let source =
        br#"fn relay['r](view: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure contract {
  ensures len_of(result) == len_of(view);
} {
  return view;
}

fn copied(view: own Slice<u8>) -> result: own u8 reads(view) contract {
  requires 1_u64 <= len_of(view);
} {
  let alias = view;
  return alias[0_u64];
}

fn returned(view: own Slice<u8>) -> result: own u8 reads(view) contract {
  requires 1_u64 <= len_of(view);
} {
  let alias = relay(view: view);
  return alias[0_u64];
}
"#;
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    // Ownership of a copyable descriptor does not own the addressed bytes.
    // Omitting reads(view) rejects through ordinary EFF-2 in either path.
    let text = std::str::from_utf8(source).expect("test source");
    for name in ["copied", "returned"] {
        let row = format!("fn {name}(view: own Slice<u8>) -> result: own u8 reads(view)");
        let pure = format!("fn {name}(view: own Slice<u8>) -> result: own u8 pure");
        let negative = text.replace(&row, &pure);
        assert_rule_kind(negative.as_bytes(), SemanticRule::Eff2, |kind| {
            matches!(kind, SemanticIssueKind::EffectMismatch { .. })
        });
    }
}
