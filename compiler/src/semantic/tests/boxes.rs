use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{CheckedExpression, CheckedNominalKind, CheckedStatement, CheckedType};
use super::{assert_rule, assert_unsupported, with_semantics};

#[test]
fn box_creation_dereference_and_cleanup_are_explicit() {
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let value: own u64 = 41_u64;
  let owner: own box<u64> = box_new<u64>(value);
  let loaded: own u64 = deref(owner);
  check ieq<u64>(loaded, 41_u64) else trap "box value";
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
  let bytes: own buffer<u8> = buffer_new<u8>(1_u64, 0_u8);
  let owner: own box<buffer<u8>> = box_new<buffer<u8>>(move bytes);
  let extracted: own buffer<u8> = move deref(owner);
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
  box_new<slice<'r, u8>>(move value);
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
