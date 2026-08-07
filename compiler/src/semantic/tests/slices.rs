use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{
    CheckedExpression, CheckedSliceOrigin, CheckedSliceSource, CheckedStatement, CheckedType,
};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn slices_retain_type_source_and_access_operations() {
    let source = br#"const bytes: array<u8, 2> =[4_u8, 9_u8];

fn first['r](values: own slice<'r, u8>) -> own u8 reads('r), traps {
  let length: own u64 = len<u8>(values);
  check ieq<u64>(length, 2_u64) else trap "length";
  return values[0_u64];
}

fn main() -> own unit traps {
  region 'view {
    let values: own slice<'view, u8> = slice_of<'view, u8>(&'view bytes);
    let value: own u8 = first<'view>(values: move values);
    check ieq<u8>(value, 4_u8) else trap "value";
  }
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("slice program must check: {outcome:?}");
        };
        let first = &checked.data.functions[0];
        assert!(matches!(first.parameters[0].ty, CheckedType::Slice { .. }));
        assert!(matches!(
            first.body[0],
            CheckedStatement::Let {
                value: CheckedExpression::SliceLength { .. },
                ..
            }
        ));
        assert!(matches!(
            first.body[2],
            CheckedStatement::Return {
                value: CheckedExpression::SliceIndex { .. },
                ..
            }
        ));

        let main = &checked.data.functions[1];
        let CheckedStatement::Region { body, .. } = &main.body[0] else {
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
    let source = br#"fn invalid['r](values: own slice<'r, u8>) -> own u8 pure {
  return values[0_u64];
}

fn main() -> own unit pure {
  return unit;
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
fn a_live_slice_prevents_writes_and_moves_of_its_source() {
    assert_rule(
        br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let window: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
    set values[0_u64] = 1_u8;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
    assert_rule(
        br#"fn main() -> own unit pure {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let window: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
    let taken: own array<u8, 2> = move values;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn slice_loans_live_until_their_named_data_region_ends() {
    assert_rule(
        br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'outer {
    region 'inner {
      let view: own slice<'outer, u8> = slice_of<'outer, u8>(&'outer values);
    }
    set values[0_u64] = 1_u8;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    assert_rule(
        br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  let take_view: own Bool = True();
  region 'outer {
    match take_view {
      True() => {
        let view: own slice<'outer, u8> = slice_of<'outer, u8>(&'outer values);
      }
      False() => {
      }
    }
    set values[0_u64] = 1_u8;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let ended_region = br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
  }
  set values[0_u64] = 1_u8;
  return unit;
}
"#;
    with_semantics(ended_region, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the shared claim must end with its named data region: {outcome:?}"
        );
    });
}

#[test]
fn slice_loans_follow_structured_break_region_exits() {
    assert_rule(
        br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
    loop @once {
      break @once;
    }
    set values[0_u64] = 1_u8;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let ended_on_break = br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  loop @once {
    region 'view {
      let view: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
      break @once;
    }
  }
  set values[0_u64] = 1_u8;
  return unit;
}
"#;
    with_semantics(ended_on_break, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "breaking out of the data region must end its shared claim: {outcome:?}"
        );
    });

    assert_rule(
        br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'outside {
    loop @once {
      let view: own slice<'outside, u8> = slice_of<'outside, u8>(&'outside values);
      break @once;
    }
  }
  return unit;
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
  source: buffer<u8>;
  sibling: buffer<u8>;
}

"#;

    let direct_move = format!(
        r#"{OWNER}fn main() -> own unit allocates(heap), traps {{
  let source: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let sibling: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let owner: own Owner = Owner(source: move source, sibling: move sibling);
  region 'view {{
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view owner.source);
    let taken: own buffer<u8> = move owner.sibling;
  }}
  return unit;
}}
"#
    );
    assert_rule(
        direct_move.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let call = format!(
        r#"{OWNER}fn consume(value: own buffer<u8>) -> own unit pure {{
  return unit;
}}

fn main() -> own unit allocates(heap), traps {{
  let source: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let sibling: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let owner: own Owner = Owner(source: move source, sibling: move sibling);
  region 'view {{
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view owner.source);
    consume(value: move owner.sibling);
  }}
  return unit;
}}
"#
    );
    assert_rule(
        call.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let matched = r#"enum Slot {
  Full(value: buffer<u8>);
  Empty();
}

struct Owner {
  source: buffer<u8>;
  sibling: Slot;
}

fn main() -> own unit allocates(heap), traps {
  let source: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let sibling_value: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let sibling: own Slot = Full(value: move sibling_value);
  let owner: own Owner = Owner(source: move source, sibling: move sibling);
  region 'view {
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view owner.source);
    match owner.sibling {
      Full(value: item) => {
      }
      Empty() => {
      }
    }
  }
  return unit;
}
"#;
    assert_rule(
        matched.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let given = format!(
        r#"{OWNER}fn main() -> own unit allocates(heap), traps {{
  let source: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let sibling: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let owner: own Owner = Owner(source: move source, sibling: move sibling);
  let choose_owner: own Bool = True();
  region 'view {{
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view owner.source);
    let selected: own buffer<u8> = match choose_owner {{
      True() => {{
        give move owner.sibling;
      }}
      False() => {{
        give buffer_new<u8>(1_u64, 0_u8);
      }}
    }}
  }}
  return unit;
}}
"#
    );
    assert_rule(
        given.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let propagated = r#"struct Owner {
  source: buffer<u8>;
  result: Result<u8, Overflow>;
}

fn invalid(owner: own Owner) -> own Result<unit, Overflow> pure {
  region 'view {
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view owner.source);
    let value: own u8 = propagate owner.result;
  }
  return Ok(value: unit);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        propagated.as_bytes(),
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    let ended_region = format!(
        r#"{OWNER}fn main() -> own unit allocates(heap), traps {{
  let source: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let sibling: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let owner: own Owner = Owner(source: move source, sibling: move sibling);
  region 'view {{
    let view: own slice<'view, u8> = slice_of<'view, u8>(&'view owner.source);
  }}
  let taken: own buffer<u8> = move owner.sibling;
  return unit;
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

#[test]
fn slice_views_are_not_set_targets() {
    assert_rule(
        br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let window: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
    set window[0_u64] = 1_u8;
  }
  return unit;
}
"#,
        SemanticRule::Set1,
        SemanticIssueKind::InvalidSetTarget {
            root_class: "slice view".to_owned(),
            required_classes: "live own storage or a live usable &uniq referent",
        },
    );
}

#[test]
fn slice_formation_enforces_storage_duration_and_explicit_boundaries() {
    assert_rule(
        br#"fn invalid['caller]() -> own unit pure {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  let window: own slice<'caller, u8> = slice_of<'caller, u8>(&'caller values);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own10,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
    assert_unsupported(
        br#"struct Item {
  value: u8;
}

fn observe['r](values: own slice<'r, Item>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::CompositeValues,
    );
    assert_unsupported(
        br#"fn invalid['source](values: &'source buffer<u8>) -> own unit pure {
  region 'view {
    let window: own slice<'view, u8> = slice_of<'view, u8>(&'view deref(values));
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
    assert_rule(
        br#"fn invalid['r](values: own array<u8, 2>) -> own slice<'r, u8> pure {
  return slice_of<'r, u8>(&'r values);
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own10,
        SemanticIssueKind::InvalidBorrowLifetime,
    );
}

#[test]
fn slice_of_keeps_nonflat_element_arguments_in_the_op1_domain() {
    assert_rule(
        br#"struct Item {
  value: u8;
}

fn main() -> own unit pure {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    slice_of<'view, Item>(&'view values);
  }
  return unit;
}
"#,
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}

#[test]
fn returned_slices_keep_signature_ceilings_and_substituted_call_origins() {
    let source = br#"fn pass['r](value: own slice<'r, u8>) -> own slice<'r, u8> pure {
  return move value;
}

fn choose['r](take_left: own Bool, left: own slice<'r, u8>, right: own slice<'r, u8>) -> own slice<'r, u8> pure {
  match take_left {
    True() => {
      return move left;
    }
    False() => {
      return move right;
    }
  }
}

fn main() -> own unit traps {
  let left: own array<u8, 2> = array_new<u8, 2>(11_u8);
  let right: own array<u8, 2> = array_new<u8, 2>(29_u8);
  region 'view {
    let pass_source: own slice<'view, u8> = slice_of<'view, u8>(&'view left);
    let passed: own slice<'view, u8> = pass<'view>(value: move pass_source);
    let passed_value: own u8 = passed[0_u64];
    check ieq<u8>(passed_value, 11_u8) else trap "returned slice pass through";
    let left_source: own slice<'view, u8> = slice_of<'view, u8>(&'view left);
    let right_source: own slice<'view, u8> = slice_of<'view, u8>(&'view right);
    let take_left: own Bool = False();
    let selected: own slice<'view, u8> = choose<'view>(take_left: take_left, left: move left_source, right: move right_source);
    let selected_value: own u8 = selected[0_u64];
    check ieq<u8>(selected_value, 29_u8) else trap "returned slice choice";
  }
  return unit;
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

        let CheckedStatement::Region { body, .. } = &checked.data.functions[2].body[2] else {
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
        } = &body[7]
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
    let wrapper = br#"fn pass['r](value: own slice<'r, u8>) -> own slice<'r, u8> pure {
  return move value;
}

fn first['r](value: own slice<'r, u8>) -> own u8 reads('r), traps {
  let returned: own slice<'r, u8> = pass<'r>(value: move value);
  return returned[0_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(wrapper, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "wrapper reads must retain the incoming slice effect: {outcome:?}"
        );
    });

    assert_rule(
        br#"fn choose['r](take_left: own Bool, left: own slice<'r, u8>, right: own slice<'r, u8>) -> own slice<'r, u8> pure {
  match take_left {
    True() => {
      return move left;
    }
    False() => {
      return move right;
    }
  }
}

fn main() -> own unit traps {
  let left: own array<u8, 2> = array_new<u8, 2>(0_u8);
  let right: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let left_view: own slice<'view, u8> = slice_of<'view, u8>(&'view left);
    let right_view: own slice<'view, u8> = slice_of<'view, u8>(&'view right);
    let take_left: own Bool = True();
    let selected: own slice<'view, u8> = choose<'view>(take_left: take_left, left: move left_view, right: move right_view);
    set right[0_u64] = 1_u8;
  }
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::BorrowConflict,
    );

    assert_rule(
        br#"fn consume['data, 'write](view: own slice<'data, u8>, output: &uniq 'write buffer<u8>) -> own unit pure {
  return unit;
}

fn wrapper['data, 'write](view: own slice<'data, u8>, output: &uniq 'write buffer<u8>) -> own unit pure {
  return consume<'data, 'write>(view: move view, output: move output);
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own12,
        SemanticIssueKind::BorrowConflict,
    );
}

#[test]
fn slice_value_matches_and_borrowed_slice_results_are_rejected() {
    assert_rule(
        br#"fn choose['r](take_left: own Bool, left: own slice<'r, u8>, right: own slice<'r, u8>) -> own slice<'r, u8> pure {
  let selected: own slice<'r, u8> = match take_left {
    True() => {
      give move left;
    }
    False() => {
      give move right;
    }
  }
  return move selected;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Own5,
        SemanticIssueKind::SliceValueMatch {
            mechanical_fix: "use a match statement whose arms return the slice directly, or call helpers with direct slice results",
        },
    );
    assert_rule(
        br#"fn invalid['descriptor, 'data](value: &'descriptor slice<'data, u8>) -> &'descriptor slice<'data, u8> pure {
  return value;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Fn1,
        SemanticIssueKind::BorrowedSliceResult {
            mechanical_fix: "return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor",
        },
    );

    let borrowed_input = br#"fn first['descriptor, 'data](value: &'descriptor slice<'data, u8>) -> own u8 reads('descriptor 'data), traps {
  return deref(value)[0_u64];
}

fn wrapper['descriptor, 'data](value: &'descriptor slice<'data, u8>) -> own u8 reads('descriptor 'data), traps {
  return first<'descriptor, 'data>(value: value);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(borrowed_input, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "descriptor and underlying slice provenance must both survive: {outcome:?}"
        );
    });
}
