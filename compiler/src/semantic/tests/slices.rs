use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{CheckedExpression, CheckedSliceSource, CheckedStatement, CheckedType};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn slices_retain_type_source_and_access_operations() {
    let source = br#"const bytes: array<u8, 2> = [4_u8, 9_u8];

fn first ['r](values: own slice<'r, u8>) -> own u8 reads('r), traps {
  let length: own u64 = len<u8>(values);
  check ieq<u64>(length, 2_u64) else trap "length";
  return index<u8>(values, 0_u64);
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
    let source = br#"fn invalid ['r](values: own slice<'r, u8>) -> own u8 pure {
  return index<u8>(values, 0_u64);
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
    set index<u8>(values, 0_u64) = 1_u8;
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
fn slice_views_are_not_set_targets() {
    assert_rule(
        br#"fn main() -> own unit traps {
  let values: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let window: own slice<'view, u8> = slice_of<'view, u8>(&'view values);
    set index<u8>(window, 0_u64) = 1_u8;
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
        br#"fn invalid ['caller]() -> own unit pure {
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

fn observe ['r](values: own slice<'r, Item>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::CompositeValues,
    );
    assert_unsupported(
        br#"fn invalid ['source](values: &'source buffer<u8>) -> own unit pure {
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
    assert_unsupported(
        br#"fn invalid ['r](values: own array<u8, 2>) -> own slice<'r, u8> pure {
  return slice_of<'r, u8>(&'r values);
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::RegionsAndBorrows,
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
