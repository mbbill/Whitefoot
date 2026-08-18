use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::model::{
    CheckedExpression, CheckedFlatElement, CheckedSetTarget, CheckedStatement,
    CheckedTargetDomainObligation, CheckedType, IntegerType, NominalId,
};
use super::{assert_rule, assert_unsupported, with_semantics};

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
  claim length_drift: ieq(length, 4_u64) because "length drift";
  claim store_drift: ieq(stored, 9_u16) because "store drift";
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
fn buffer_vacant_constructs_an_all_none_affine_element_buffer() {
    let source = br#"fn main() -> own unit allocates(heap), traps {
  let slots = buffer_vacant<box<u64>>(3_u64);
  let count = len(slots);
  claim vacant_length: ieq(count, 3_u64) because "vacant length";
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("buffer_vacant must check: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Let { value, .. } = &main.body[0] else {
            panic!("the first statement binds the vacant buffer");
        };
        let CheckedExpression::BufferVacant {
            element,
            trap,
            target_domains,
            ..
        } = value
        else {
            panic!("buffer_vacant retains its own OP-9 allocation record");
        };
        assert_eq!(
            checked.data.nominals[element.0 as usize].name,
            "Option<box<u64>>"
        );
        assert_eq!(trap.rule_id, "OP-9");
        assert_eq!(
            target_domains.allocation(),
            CheckedTargetDomainObligation::RuntimeSizedAllocation
        );
        assert_eq!(
            target_domains.element_address(),
            CheckedTargetDomainObligation::ElementAddress
        );
        assert_eq!(
            value.ty(),
            CheckedType::Buffer {
                element: CheckedFlatElement::Nominal(*element),
            }
        );
        // The [ENT-5] length fact from the allocation discharges the ieq
        // check's operands without a claim, which acceptance already proves.
        assert!(matches!(
            &main.body[1],
            CheckedStatement::Let {
                value: CheckedExpression::BufferLength { .. },
                ..
            }
        ));
    });
}

#[test]
fn buffer_vacant_requires_its_written_payload_and_effect_row() {
    // [TYPE-5]: the element payload type is a retained written argument.
    assert_rule(
        b"fn main() -> own unit allocates(heap), traps {\n  let slots = buffer_vacant(3_u64);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::InvalidOperation,
    );
    // [EFF-2]: the row is allocates(heap), traps, both directions.
    assert_rule(
        b"fn main() -> own unit traps {\n  let slots = buffer_vacant<u32>(3_u64);\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    assert_rule(
        b"fn main() -> own unit allocates(heap) {\n  let slots = buffer_vacant<u32>(3_u64);\n  return unit;\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    // [TYPE-5]: the one operand is the own u64 length.
    assert_rule(
        b"fn main() -> own unit allocates(heap), traps {\n  let slots = buffer_vacant<u32>(3_u32);\n  return unit;\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn buffer_vacant_rejects_a_region_bearing_payload_under_stor5() {
    assert_rule(
        br#"fn invalid['r](value: own slice<'r, u8>) -> own unit allocates(heap), traps {
  let slots = buffer_vacant<slice<'r, u8>>(2_u64);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        SemanticRule::Stor5,
        SemanticIssueKind::RegionBearingStorage {
            mechanical_fix: "keep the slice or arena as a direct local, parameter, or result; do not store it inside another value",
        },
    );
}

#[test]
fn affine_element_views_and_structural_composites_stop_explicitly() {
    // A slice over an affine-element buffer has no implemented in-place
    // read; it stops as capability, not as a source rejection.
    assert_unsupported(
        br#"fn main() -> own unit allocates(heap), traps {
  let slots = buffer_vacant<u32>(4_u64);
  region 'v {
    let view = slice_of(&'v slots);
  }
  return unit;
}
"#,
        UnsupportedSemanticFeature::CompositeValues,
    );
    // A structural affine element (a nested buffer) is spec-formable
    // [TYPE-2] but has no implemented representation.
    assert_unsupported(
        br#"fn keep(value: own buffer<buffer<u8>>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        UnsupportedSemanticFeature::CompositeValues,
    );
}

#[test]
fn array_elements_stay_copy_only_under_type2() {
    with_semantics(
        br#"fn keep(value: own array<Option<u32>, 2>) -> own unit pure {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#,
        |outcome| {
            let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("an affine array element must reject: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Type2);
        },
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
  claim length_drift: ieq(length, 4_u64) because "length drift";
  claim value_drift: ieq(value, 7_u64) because "value drift";
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
                value: CheckedExpression::BufferLength { root, .. },
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
