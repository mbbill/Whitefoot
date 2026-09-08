//! Store-cell tests [S39]: `heap_box` and `arena_box` take one value into a
//! cell of a named store, `deref` reads the referent, and the cell's release
//! is the store's own.
//!
//! B7c4b moved this module off the retiring ambient `box<T>` and `box_new`.
//! The checked model is shared: a cell is `CheckedNominalKind::Box`, whose
//! `region` is `Some(store)` for a store-branded cell where it was `None` for
//! the ambient one, and a referent read is the same `BoxDeref`. What changes
//! in every program is the allocation itself, which is now a fallible kernel
//! row over a provider the scope holds, so each case names a store and matches
//! the outcome that hands the value back.

use crate::{
    KernelRow, SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::{CheckedExpression, CheckedNominalKind, CheckedStatement, CheckedType};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn cell_creation_dereference_and_cleanup_are_explicit() {
    let source = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let value = 41_u64;
  region {
    match heap_box(store: &uniq heap, value: value) {
      Ok(value: owner) => {
        let loaded = deref(owner);
        return exit_status(code: 0_u8);
      }
      Err(error: back) => {
        return exit_status(code: 1_u8);
      }
    }
  }
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("cell creation and copy read must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Region { body, .. } = &main.body[1] else {
            panic!("the allocation's region must remain explicit");
        };
        let CheckedStatement::Match {
            scrutinee, arms, ..
        } = &body[0]
        else {
            panic!("the fallible allocation must remain a match");
        };
        assert!(matches!(
            scrutinee,
            CheckedExpression::KernelCall {
                row: KernelRow::HeapBox,
                ..
            }
        ));
        let taken = &arms[0].body;
        let CheckedStatement::Let {
            value:
                CheckedExpression::BoxDeref {
                    nominal,
                    referent: CheckedType::Integer(_),
                    ..
                },
            ..
        } = &taken[0]
        else {
            panic!("the referent read must remain an explicit deref");
        };
        assert!(matches!(
            checked.data.nominals[nominal.0 as usize].kind,
            CheckedNominalKind::Box {
                referent: CheckedType::Integer(_),
                region: Some(_),
                ..
            }
        ));
        let CheckedStatement::Return { drops, .. } = &taken[1] else {
            panic!("the taken arm must end in return");
        };
        assert_eq!(drops.len(), 1);
        assert_eq!(drops[0].ty, CheckedType::Nominal(*nominal));
    });
}

/// Replacing one whole cell owner transfers the second allocation into the
/// first binding without changing the first binding's cell type. This used to
/// be hidden inside a retired assertion-locality fixture; it is an ordinary
/// ownership and checked-model property.
#[test]
fn whole_cell_replacement_preserves_the_owner_shape() {
    let source = br#"struct Pair {
  value: u64;
}

fn replace_owner(store: &uniq Heap) -> result: own Option<u64> reads(store), writes(store), allocates(store) {
  let first_value = Pair(value: 0_u64);
  let second_value = Pair(value: 1_u64);
  region {
    match heap_box(store: &uniq deref(store), value: move first_value) {
      Ok(value: first) => {
        region {
          match heap_box(store: &uniq deref(store), value: move second_value) {
            Ok(value: second) => {
              let old = replace first = move second;
              return Some<u64>(value: deref(first).value);
            }
            Err(error: back) => {
              return None<u64>();
            }
          }
        }
      }
      Err(error: back) => {
        return None<u64>();
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("whole-cell replacement must preserve the owner type: {outcome:?}");
        };
        let replace_owner = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "replace_owner")
            .expect("replace_owner function");
        assert!(
            statements_contain_replace(&replace_owner.body),
            "the whole-owner replacement must remain a checked Replace"
        );
    });
}

/// Whether any statement of this body, or of a nested block below it, is a
/// `replace`. The migrated program nests the second allocation inside the
/// first one's taken arm, so the statement is no longer a direct child.
fn statements_contain_replace(body: &[CheckedStatement]) -> bool {
    body.iter().any(|statement| match statement {
        CheckedStatement::Replace { .. } => true,
        CheckedStatement::Region { body, .. } => statements_contain_replace(body),
        CheckedStatement::Match { arms, .. } => {
            arms.iter().any(|arm| statements_contain_replace(&arm.body))
        }
        _ => false,
    })
}

#[test]
fn affine_cell_referent_move_stays_an_explicit_capability_boundary() {
    let source = br#"fn hold(store: &uniq Heap) -> result: own unit reads(store), writes(store), allocates(store) {
  let bytes = fixed_vector::<u8, 1>();
  region {
    match heap_box(store: &uniq deref(store), value: move bytes) {
      Ok(value: owner) => {
        let extracted = move deref(owner);
        return unit;
      }
      Err(error: back) => {
        return unit;
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_unsupported(source, UnsupportedSemanticFeature::BoxReferentMove);
}

/// [SET-1] admits a `deref` target through either of two roots: a live usable
/// `&uniq` holder, or a live own-mode binding whose storage the `deref`
/// reaches [STOR-1]. A cell taken from a store is own mode, so `set deref(b)`
/// over it is the second root and never needs a holder; the target-side
/// dispatch nevertheless resolved a holder for every `deref` target and
/// reported these spec-legal targets as TYPE-7 "deref requires a borrow
/// holder". Cell content is copy-typed here, so SET-1 admits the target and it
/// stops explicitly: the target names the root binding, which lowers to the
/// content pointer under the cell's own IR type, so no store addresses the
/// content.
#[test]
fn cell_content_set_targets_are_own_rooted_rather_than_holder_derefs() {
    assert_unsupported(
        br#"fn hold(store: &uniq Heap) -> result: own unit reads(store), writes(store), allocates(store) {
  region {
    match heap_box(store: &uniq deref(store), value: 4_i32) {
      Ok(value: b) => {
        set deref(b) = 7_i32;
        return unit;
      }
      Err(error: back) => {
        return unit;
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::RegionsAndBorrows,
    );
    // [SET-2] shares SET-1's writability relation, so an affine, region-free
    // cell content is a legal `replace` target and reaches the same stop.
    assert_unsupported(
        br#"fn hold(store: &uniq Heap) -> result: own unit reads(store), writes(store), allocates(store) {
  let bytes = fixed_vector::<u8, 1>();
  let other = fixed_vector::<u8, 1>();
  region {
    match heap_box(store: &uniq deref(store), value: move bytes) {
      Ok(value: owner) => {
        let old = replace deref(owner) = move other;
        return unit;
      }
      Err(error: back) => {
        return unit;
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
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
fn cell_content_set_targets_keep_their_source_rejections() {
    assert_rule(
        br#"fn hold(store: &uniq Heap) -> result: own unit reads(store), writes(store), allocates(store) {
  let bytes = fixed_vector::<u8, 1>();
  let other = fixed_vector::<u8, 1>();
  region {
    match heap_box(store: &uniq deref(store), value: move bytes) {
      Ok(value: owner) => {
        set deref(owner) = move other;
        return unit;
      }
      Err(error: back) => {
        return unit;
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor1,
        SemanticIssueKind::AffineSetTarget {
            target_type: "FixedVector<u8, 1>".to_owned(),
            mechanical_fix: "use replace: let old = replace p = e; binds the previous owner",
        },
    );
    assert_rule(
        br#"fn eat['s](b: own Box<'s, i32>, store: &uniq Heap<'s>) -> result: own unit writes(store) {
  return unit;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 4_i32) {
      Ok(value: b) => {
        eat(b: move b, store: &uniq heap);
        set deref(b) = 7_i32;
        return exit_status(code: 0_u8);
      }
      Err(error: back) => {
        return exit_status(code: 1_u8);
      }
    }
  }
}
"#,
        SemanticRule::Own1,
        SemanticIssueKind::UseAfterMove {
            mechanical_fix: "introduce a new `let` binding before reuse",
        },
    );
}

#[test]
fn region_bearing_cell_content_rejects_under_stor5_at_both_stores() {
    let expected = SemanticIssueKind::RegionBearingStorage {
        mechanical_fix: "keep the slice, arena, or provider as a direct local, parameter, or result; do not store it inside another value",
    };
    assert_rule(
        br#"fn invalid(value: own Box<Slice<u8>>) -> result: own unit pure {
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
        br#"fn invalid(store: &uniq Heap, value: own Slice<u8>) -> result: own unit reads(store), writes(store), allocates(store) {
  region {
    heap_box(store: &uniq deref(store), value: value);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    // The same relation over the other store: a bump extent's cell carries the
    // referent judgment its region-bearing operand supplies, exactly as the
    // general store's does.
    assert_rule(
        br#"fn invalid(value: own Slice<u8>) -> result: own unit pure {
  region 'a {
    let workspace = arena_frame::<64, 8, 'a>();
    region {
      arena_box(store: &uniq workspace, value: value);
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    // A cell derives its content type from the operand [S39], and a store
    // provider bears a region exactly as a view does, so the derived judgment
    // is STOR-5's relation over that type rather than a view-shaped operand
    // test.
    assert_rule(
        br#"fn hold(store: &uniq Heap) -> result: own unit reads(store), writes(store), allocates(store) {
  region 'a {
    let workspace = arena_frame::<64, 8, 'a>();
    region {
      heap_box(store: &uniq deref(store), value: move workspace);
    }
  }
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

/// [S39] the cell nominal is derived from the operand, so a purely local cell
/// names `Box<'s, T>` nowhere for the written-type interning pass to find.
///
/// The control pair differs only in whether some *other* declaration spells
/// `Box<'s, u64>`. Before the checker could intern a derived referent, the
/// first program failed with a compiler failure while the second compiled — an
/// implementation limitation deciding what source was acceptable.
///
/// The counts are re-derived for the store-branded cell rather than kept: a
/// cell's store region is a component of its type [PROV-1, S39], so the
/// helper's written `Box<'s, u64>` is its own region's instance and is
/// interned beside the entry store's derived one. What the case pins is
/// unchanged — the derived nominal is interned once, sits in the executable
/// prefix, and no two instances share a region.
#[test]
fn a_derived_cell_nominal_is_interned_whether_or_not_the_type_is_spelled_elsewhere() {
    let named_nowhere = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 41_u64) {
      Ok(value: owner) => {
        let loaded = deref(owner);
        return exit_status(code: 0_u8);
      }
      Err(error: back) => {
        return exit_status(code: 1_u8);
      }
    }
  }
}
"#;
    let named_in_a_signature = br#"fn take['s](b: own Box<'s, u64>, store: &uniq Heap<'s>) -> result: own unit writes(store) {
  return unit;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_box(store: &uniq heap, value: 41_u64) {
      Ok(value: owner) => {
        take(b: move owner, store: &uniq heap);
        return exit_status(code: 0_u8);
      }
      Err(error: back) => {
        return exit_status(code: 1_u8);
      }
    }
  }
}
"#;
    for (source, expected) in [
        (named_nowhere.as_slice(), 1),
        (named_in_a_signature.as_slice(), 2),
    ] {
        with_semantics(source, |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("a derived cell nominal must check: {outcome:?}");
            };
            // The derived nominal sits inside the executable prefix, because
            // executable code allocates and drops it.
            let regions: Vec<_> = checked
                .data
                .nominals
                .iter()
                .take(checked.data.executable_nominal_count)
                .filter_map(|nominal| match nominal.kind {
                    CheckedNominalKind::Box {
                        referent: CheckedType::Integer(_),
                        region,
                        ..
                    } => Some(region),
                    _ => None,
                })
                .collect();
            assert_eq!(
                regions.len(),
                expected,
                "one executable cell nominal per named store"
            );
            let mut distinct = regions.clone();
            distinct.sort();
            distinct.dedup();
            assert_eq!(
                distinct.len(),
                regions.len(),
                "a store region is interned once, not twice"
            );
        });
    }
}
