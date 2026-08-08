use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{CheckedExpression, CheckedNominalKind, CheckedStatement, CheckedType};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn box_creation_dereference_and_cleanup_are_explicit() {
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let value = 41_u64;
  let owner = box_new(value);
  let loaded = deref(owner);
  check ieq(loaded, 41_u64) else trap "box value";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("box creation and copy read must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Let {
            value: CheckedExpression::BoxNew { nominal, .. },
            ..
        } = &main.body[1]
        else {
            panic!("box_new must remain explicit in the checked program");
        };
        assert!(matches!(
            checked.data.nominals[nominal.0 as usize].kind,
            CheckedNominalKind::Box {
                referent: CheckedType::Integer(_)
            }
        ));
        assert!(matches!(
            &main.body[2],
            CheckedStatement::Let {
                value: CheckedExpression::BoxDeref {
                    referent: CheckedType::Integer(_),
                    ..
                },
                ..
            }
        ));
        let CheckedStatement::Return { drops, .. } = &main.body[4] else {
            panic!("main must end in return");
        };
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].ty, CheckedType::Nominal(*nominal));
    });
}

#[test]
fn affine_box_referent_move_stays_an_explicit_capability_boundary() {
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let bytes = buffer_new(1_u64, 0_u8);
  let owner = box_new(move bytes);
  let extracted = move deref(owner);
  return unit;
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::BoxReferentMove);
}

#[test]
fn region_bearing_box_and_arena_content_reject_under_stor5() {
    let expected = SemanticIssueKind::RegionBearingStorage {
        mechanical_fix: "keep the slice or arena as a direct local, parameter, or result; do not store it inside another value",
    };
    assert_rule(
        br#"fn invalid['r](value: own box<slice<'r, u8>>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['r](value: own slice<'r, u8>) -> own unit allocates(heap) {
  box_new(move value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['storage, 'data](value: own arena<'storage, slice<'data, u8>>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['data, 'storage](value: own slice<'data, u8>) -> own unit allocates(arena 'storage) {
  arena_new<'storage, slice<'data, u8>>(move value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        expected,
    );
}

/// [STOR-2] the box nominal is derived from the operand, so a purely local
/// box names `box<T>` nowhere for the written-type interning pass to find.
///
/// The control pair differs only in whether some *other* declaration spells
/// `box<u64>`. Before the checker could intern a derived referent, the first
/// program failed with a compiler failure while the second compiled — an
/// implementation limitation deciding what source was acceptable.
#[test]
fn a_derived_box_nominal_is_interned_whether_or_not_the_type_is_spelled_elsewhere() {
    let named_nowhere = br#"fn main() -> own unit allocates(heap) {
  let owner = box_new(41_u64);
  let loaded = deref(owner);
  return unit;
}
"#;
    let named_in_a_signature = br#"fn take(b: own box<u64>) -> own unit pure {
  return unit;
}

fn main() -> own unit allocates(heap) {
  let owner = box_new(41_u64);
  take(b: move owner);
  return unit;
}
"#;
    for source in [named_nowhere.as_slice(), named_in_a_signature.as_slice()] {
        with_semantics(source, |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("a derived box nominal must check: {outcome:?}");
            };
            // The derived nominal sits inside the executable prefix, because
            // executable code allocates and drops it.
            let boxes = checked
                .data
                .nominals
                .iter()
                .take(checked.data.executable_nominal_count)
                .filter(|nominal| {
                    matches!(
                        nominal.kind,
                        CheckedNominalKind::Box {
                            referent: CheckedType::Integer(_)
                        }
                    )
                })
                .count();
            assert_eq!(
                boxes, 1,
                "exactly one box<u64> nominal, and it is executable"
            );
        });
    }
}
