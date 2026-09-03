use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{CheckedExpression, CheckedNominalKind, CheckedStatement, CheckedType};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn box_creation_dereference_and_cleanup_are_explicit() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let value = 41_u64;
  let owner = box_new(value);
  let loaded = deref(owner);
  return exit_status(code: 0_u8);
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
        let CheckedStatement::Return { drops, .. } = &main.body[3] else {
            panic!("main must end in return");
        };
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].ty, CheckedType::Nominal(*nominal));
    });
}

/// Replacing one whole box owner transfers the second allocation into the
/// first binding without changing the first binding's box type. This used to
/// be hidden inside a retired assertion-locality fixture; it is an ordinary ownership and
/// checked-model property.
#[test]
fn whole_box_replacement_preserves_the_owner_shape() {
    let source = br#"struct Pair {
  value: u64;
}

fn replace_owner() -> result: own u64 allocates(heap) {
  let first_value = Pair(value: 0_u64);
  let first = box_new(move first_value);
  let second_value = Pair(value: 1_u64);
  let second = box_new(move second_value);
  let old = replace first = move second;
  return deref(first).value;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("whole-box replacement must preserve the owner type: {outcome:?}");
        };
        let replace_owner = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "replace_owner")
            .expect("replace_owner function");
        assert!(
            replace_owner
                .body
                .iter()
                .any(|statement| matches!(statement, CheckedStatement::Replace { .. }))
        );
    });
}

#[test]
fn affine_box_referent_move_stays_an_explicit_capability_boundary() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let bytes = buffer_new(1_u64, 0_u8);
  let owner = box_new(move bytes);
  let extracted = move deref(owner);
  return exit_status(code: 0_u8);
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::BoxReferentMove);
}

/// [SET-1] admits a `deref` target through either of two roots: a live usable
/// `&uniq` holder, or a live own-mode binding whose storage the `deref`
/// reaches [STOR-1]. A `box_new` result is own mode, so `set deref(b)` over it
/// is the second root and never needs a holder; the target-side dispatch
/// nevertheless resolved a holder for every `deref` target and reported these
/// spec-legal targets as TYPE-7 "deref requires a borrow holder". Box content
/// is copy-typed here, so SET-1 admits the target and it stops explicitly:
/// the target names the root binding, which lowers to the content pointer
/// under the box's own IR type, so no store addresses the content.
#[test]
fn box_content_set_targets_are_own_rooted_rather_than_holder_derefs() {
    assert_unsupported(
        br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let b = box_new(4_i32);
  set deref(b) = 7_i32;
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
    // [SET-2] shares SET-1's writability relation, so an affine, region-free
    // box content is a legal `replace` target and reaches the same stop.
    assert_unsupported(
        br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let bytes = buffer_new(1_u64, 0_u8);
  let owner = box_new(move bytes);
  let other = buffer_new(1_u64, 1_u8);
  let old = replace deref(owner) = move other;
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
}

/// The ordinary own-rooted judgments the routed target now reaches still
/// reject before the capability stop: [STOR-1] for an affine final selected
/// type, which `set` never writes, and [OWN-1] for a dead root, which SET-1
/// never revives.
#[test]
fn box_content_set_targets_keep_their_source_rejections() {
    assert_rule(
        br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let bytes = buffer_new(1_u64, 0_u8);
  let owner = box_new(move bytes);
  let other = buffer_new(1_u64, 1_u8);
  set deref(owner) = move other;
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "buffer<u8>".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
    assert_rule(
        br#"fn eat(b: own box<i32>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let b = box_new(4_i32);
  eat(b: move b);
  set deref(b) = 7_i32;
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );
}

#[test]
fn region_bearing_box_and_arena_content_reject_under_stor5() {
    let expected = SemanticIssueKind::RegionBearingStorage {
        mechanical_fix: "keep the slice or arena as a direct local, parameter, or result; do not store it inside another value",
    };
    assert_rule(
        br#"fn invalid['r](value: own box<slice<'r, u8>>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['r](value: own slice<'r, u8>) -> result: own unit allocates(heap) {
  box_new(move value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['storage, 'data](value: own arena<'storage, slice<'data, u8>>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    assert_rule(
        br#"fn invalid['data, 'storage](value: own slice<'data, u8>) -> result: own unit allocates(arena 'storage) {
  arena_new::<'storage, slice<'data, u8>>(move value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    // `box_new` derives its content type from the operand [STOR-2], and an
    // arena descriptor operand bears a region exactly as a slice operand
    // does, so the derived judgment is STOR-5's relation over that type
    // rather than a slice-shaped operand test.
    assert_rule(
        br#"fn invalid['r](value: own arena<'r, u64>) -> result: own unit allocates(heap) {
  box_new(move value);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
    let named_nowhere = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let owner = box_new(41_u64);
  let loaded = deref(owner);
  return exit_status(code: 0_u8);
}
"#;
    let named_in_a_signature = br#"fn take(b: own box<u64>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let owner = box_new(41_u64);
  take(b: move owner);
  return exit_status(code: 0_u8);
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
