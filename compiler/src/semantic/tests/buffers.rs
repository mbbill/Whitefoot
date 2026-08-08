use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::model::{
    CheckedExpression, CheckedFlatElement, CheckedSetTarget, CheckedStatement,
    CheckedTargetDomainObligation, CheckedType, IntegerType, NominalId,
};
use super::{assert_rule, with_semantics};

#[test]
fn primitive_buffers_retain_allocation_checks_accesses_and_cleanup() {
    let source = br#"fn make(n: own u64) -> own buffer<u16> allocates(heap), traps {
  return buffer_new(n, 3_u16);
}

fn main() -> own unit allocates(heap), traps {
  let values = make(n: 4_u64);
  let length = len(values);
  let ok = ilt(2_u64, length);
  claim sized_by_make: ok because "make allocates n slots and main passes four";
  set values[2_u64] = 9_u16;
  let stored = values[2_u64];
  check length == 4_u64 else trap "length drift";
  check stored == 9_u16 else trap "store drift";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("primitive buffer family must check: {outcome:?}");
        };
        let make = &checked.data.functions[0];
        assert!(make.declared_allocates_heap);
        assert!(make.declared_traps);
        assert!(matches!(
            &make.body[0],
            CheckedStatement::Return {
                value: CheckedExpression::BufferFill {
                    element: CheckedFlatElement::Integer(IntegerType::U16),
                    trap,
                    target_domains,
                    ..
                },
                ..
            } if trap.rule_id == "OP-9"
                && target_domains.allocation()
                    == CheckedTargetDomainObligation::RuntimeSizedAllocation
                && target_domains.element_address()
                    == CheckedTargetDomainObligation::ElementAddress
        ));

        let main = &checked.data.functions[1];
        let CheckedStatement::Set { target, .. } = &main.body[4] else {
            panic!("the statement after the discharging claim must be indexed SET-1");
        };
        let CheckedSetTarget::BufferIndex(target) = target else {
            panic!("SET-1 target must retain its buffer root and OP-4 check");
        };
        assert_eq!(
            target.root.element,
            CheckedFlatElement::Integer(IntegerType::U16)
        );
        assert_eq!(target.trap.rule_id, "OP-4");
        assert_eq!(
            target.target_domain,
            CheckedTargetDomainObligation::ElementAddress
        );
        assert!(matches!(
            &main.body[1],
            CheckedStatement::Let {
                value: CheckedExpression::BufferLength { .. },
                ..
            }
        ));
        assert!(matches!(
            &main.body[5],
            CheckedStatement::Let {
                value: CheckedExpression::BufferIndex {
                    trap,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    ..
                },
                ..
            } if trap.rule_id == "OP-4"
        ));
        let CheckedStatement::Return { drops, .. } = &main.body[8] else {
            panic!("main must end in return");
        };
        assert_eq!(drops.len(), 1);
        assert_eq!(
            drops[0].ty,
            CheckedType::Buffer {
                element: CheckedFlatElement::Integer(IntegerType::U16),
            }
        );
    });
}

#[test]
fn buffer_effect_rows_are_checked_both_ways() {
    assert_rule(
        b"fn main() -> own unit traps {\n  let values = buffer_new(2_u64, 0_u8);\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn main() -> own unit allocates(heap) {\n  let values = buffer_new(2_u64, 0_u8);\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn main() -> own unit allocates(heap), traps {\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn buffer_new_keeps_its_primitive_only_operation_domain() {
    assert_rule(
        b"fn main() -> own unit allocates(heap), traps {\n  let initial = False();\n  let values = buffer_new(2_u64, initial);\n  return unit;\n}\n",
        SemanticRule::Op1,
        SemanticIssueKind::InvalidOperation,
    );
}

#[test]
fn struct_buffer_paths_and_reverse_cleanup_are_explicit() {
    let source = br#"struct Columns {
  left: buffer<u64>;
  right: buffer<u64>;
}

fn main() -> own unit allocates(heap), traps {
  let left = buffer_new(4_u64, 0_u64);
  let right = buffer_new(4_u64, 0_u64);
  let columns = Columns(left: move left, right: move right);
  let left_room = len(columns.left);
  let ok = ilt(2_u64, left_room);
  claim left_sized: ok because "columns.left was allocated with four slots";
  set columns.left[2_u64] = 7_u64;
  let length = len(columns.right);
  let value = columns.left[2_u64];
  check length == 4_u64 else trap "length drift";
  check value == 7_u64 else trap "value drift";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("struct-of-buffers must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Set { target, .. } = &main.body[6] else {
            panic!("the statement after the discharging claim must be projected indexed SET-1");
        };
        let CheckedSetTarget::BufferIndex(target) = target else {
            panic!("SET-1 must retain a projected buffer root");
        };
        assert_eq!(target.root.fields, [0]);
        assert!(matches!(
            &main.body[7],
            CheckedStatement::Let {
                value: CheckedExpression::BufferLength { root },
                ..
            } if root.fields == [1]
        ));
        assert!(matches!(
            &main.body[8],
            CheckedStatement::Let {
                value: CheckedExpression::BufferIndex { root, .. },
                ..
            } if root.fields == [0]
        ));
        let CheckedStatement::Return { drops, .. } = &main.body[11] else {
            panic!("main must end in return");
        };
        assert_eq!(drops.len(), 3);
        assert_eq!(drops[0].fields, [1]);
        assert_eq!(
            drops[0].ty,
            CheckedType::Buffer {
                element: CheckedFlatElement::Integer(IntegerType::U64),
            }
        );
        assert_eq!(drops[1].fields, [0]);
        assert_eq!(drops[1].ty, drops[0].ty);
        assert!(drops[2].fields.is_empty());
        assert_eq!(drops[2].ty, CheckedType::Nominal(NominalId(0)));
    });
}

#[test]
fn resource_bearing_enum_owners_have_one_variant_dependent_drop() {
    with_semantics(
        b"enum MaybeBuffer {\n  Empty();\n  Full(value: buffer<u8>);\n}\n\nfn abandon(value: own MaybeBuffer) -> own unit pure {\n  return unit;\n}\n\nfn main() -> own unit pure {\n  return unit;\n}\n",
        |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("resource-bearing enum payload must check: {outcome:?}");
            };
            let CheckedStatement::Return { drops, .. } = &checked.data.functions[0].body[0]
            else {
                panic!("abandon must end in return");
            };
            assert_eq!(drops.len(), 1);
            assert!(drops[0].fields.is_empty());
            assert_eq!(drops[0].ty, CheckedType::Nominal(NominalId(0)));
        },
    );
}

#[test]
fn nested_partial_move_skips_the_moved_subtree_in_structural_drop_order() {
    let source = br#"struct Pair {
  first: buffer<u8>;
  second: buffer<u8>;
}

struct Owner {
  prefix: buffer<u8>;
  pair: Pair;
  suffix: buffer<u8>;
}

fn take(owner: own Owner) -> own buffer<u8> pure {
  return move owner.pair.first;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("projected buffer move must check: {outcome:?}");
        };
        let CheckedStatement::Return {
            value: CheckedExpression::Project { residual_drops, .. },
            ..
        } = &checked.data.functions[0].body[0]
        else {
            panic!("take must return one ownership-consuming projection");
        };
        assert_eq!(residual_drops.len(), 3);
        assert_eq!(residual_drops[0].fields, [2]);
        assert_eq!(residual_drops[1].fields, [1, 1]);
        assert_eq!(residual_drops[2].fields, [0]);
    });
}

#[test]
fn region_bearing_buffer_content_rejects_under_stor5() {
    let expected = SemanticIssueKind::RegionBearingStorage {
        mechanical_fix: "keep the slice or arena as a direct local, parameter, or result; do not store it inside another value",
    };
    assert_rule(
        br#"fn invalid['r](value: own buffer<slice<'r, u8>>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        expected.clone(),
    );
    // The operation half. In v0.22 this was `buffer_new<slice<'r, u8>>(…)`,
    // whose *written* element carried the violation and was cited at the
    // `targ`; A1 deletes that argument [OP-9], and a region-bearing fill is
    // then caught by the flat-element requirement citing OP-1 before STOR-5 is
    // reached. [STOR-5] names `box_new` and `arena_new` — not `buffer_new` —
    // as the derived-content path it owns, and `box_new`'s content type is
    // derived from its operand [STOR-2, OP-2], so that is where the recorded
    // rule and kind still fire, at the operand atom the rule names.
    assert_rule(
        br#"fn invalid['r](value: own slice<'r, u8>) -> own unit allocates(heap), traps {
  box_new(move value);
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
