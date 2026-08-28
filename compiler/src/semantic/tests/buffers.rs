use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule, UnsupportedSemanticFeature};

use super::super::entailment::{DerivationNode, GoalSign, ObligationFamily};
use super::super::model::{
    CheckedExpression, CheckedFlatElement, CheckedLayoutMagnitude, CheckedSetTarget,
    CheckedStatement, CheckedTargetDomainObligation, CheckedType, IntegerType, NominalId,
};
use super::{assert_rule, assert_unsupported, with_semantics, with_semantics_dark};

#[test]
fn allocation_fit_is_static_exact_componentized_and_contradiction_closing() {
    let unproved = br#"fn allocate(n: own u64) -> result: own unit allocates(heap) {
  let values = buffer_new(n, 0_u16);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(unproved, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("an unproved allocation fit must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op9);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedAllocationFitObligation { .. }
        ));
    });
    with_semantics_dark(unproved, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the dark hook must retain the OP-9 obligation: {outcome:?}");
        };
        let allocation = &checked.data.functions[0].entailment.obligations;
        assert_eq!(allocation.len(), 1);
        assert_eq!(allocation[0].family, ObligationFamily::AllocationFit);
        assert!(!allocation[0].discharged);
    });

    let exact = br#"fn bounded_count(n: own u64) -> result: own u64 pure contract {
  ensures ile(result, 9223372036854775807_u64);
} {
  let within = ile(n, 9223372036854775807_u64);
  if within {
    return n;
  } else {
    return 9223372036854775807_u64;
  }
}

fn allocate(n: own u64) -> result: own unit allocates(heap) {
  let bounded = bounded_count(n: n);
  let values = buffer_new(bounded, 0_u16);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(exact, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "the exact predicate must discharge OP-9: {outcome:?}"
        );
    });

    let component = br#"fn allocate(n: own u64) -> result: own unit allocates(heap) {
  let within = ile(n, 9223372036854775807_u64);
  if within {
    let values = buffer_new(n, 0_u16);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(component, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the canonical L0 component must discharge OP-9: {outcome:?}");
        };
        let allocation = checked.data.functions[0]
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::AllocationFit)
            .expect("the allocation retains its OP-9 occurrence");
        let proof = allocation
            .derivation
            .expect("the discharged allocation retains one proof");
        assert!(matches!(
            checked.data.functions[0].entailment.derivations.nodes[proof.0 as usize],
            DerivationNode::GoalNormalization {
                sign: GoalSign::Positive,
                ..
            }
        ));
    });

    let refuted = br#"fn allocate(n: own u64) -> result: own unit allocates(heap) {
  let fits = buffer_fits<u8>(n);
  let does_not_fit = bnot(fits);
  if does_not_fit {
    let within = ile(n, 18446744073709551615_u64);
    if within {
      let values = buffer_new(n, 0_u8);
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(refuted, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!(
                "a fixed true component plus the exact negative must close contradiction: {outcome:?}"
            );
        };
        let allocation = checked.data.functions[0]
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == ObligationFamily::AllocationFit)
            .expect("the allocation retains its OP-9 occurrence");
        assert!(allocation.discharged);
        assert!(allocation.contradictory);
        let contradiction = allocation
            .derivation
            .expect("the contradictory allocation keeps its proof");
        assert!(matches!(
            checked.data.functions[0].entailment.derivations.nodes[contradiction.0 as usize],
            DerivationNode::GoalContradiction { .. }
        ));
    });
}

#[test]
fn buffer_fits_admits_direct_region_free_array_and_buffer_types() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let array_fit = buffer_fits<array<u8, 4>>(0_u64);
  let buffer_fit = buffer_fits<buffer<u8>>(0_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("direct region-free composite types belong to buffer_fits: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let expected = [
            CheckedType::Array {
                element: CheckedFlatElement::Integer(IntegerType::U8),
                length: super::super::model::CheckedConst::Value(4),
            },
            CheckedType::Buffer {
                element: CheckedFlatElement::Integer(IntegerType::U8),
            },
        ];
        for (statement, expected) in main.body.iter().take(2).zip(expected) {
            let CheckedStatement::Let {
                value: CheckedExpression::BufferFits { element, .. },
                ..
            } = statement
            else {
                panic!("each binding must retain its typed buffer_fits expression");
            };
            assert_eq!(*element, expected);
        }
    });
}

#[test]
fn zero_length_arrays_have_the_empty_sequence_layout_ceiling() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let array_fit = buffer_fits<array<u64, 0>>(18446744073709551615_u64);
  let slots = buffer_vacant<array<u64, 0>>(0_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("zero-length array ceilings must admit the exact empty sequence: {outcome:?}");
        };
        let main = &checked.data.functions[0];
        let CheckedStatement::Let {
            value:
                CheckedExpression::BufferFits {
                    layout_ceiling: array_ceiling,
                    ..
                },
            ..
        } = &main.body[0]
        else {
            panic!("the first binding must retain the array buffer_fits query");
        };
        assert_eq!(array_ceiling.size, CheckedLayoutMagnitude::Finite(0));
        assert_eq!(array_ceiling.align, 1);
        assert_eq!(array_ceiling.stride, CheckedLayoutMagnitude::Finite(1));

        let CheckedStatement::Let {
            value:
                CheckedExpression::BufferVacant {
                    layout_ceiling: option_ceiling,
                    ..
                },
            ..
        } = &main.body[1]
        else {
            panic!("the second binding must retain the vacant Option allocation ceiling");
        };
        assert_eq!(option_ceiling.size, CheckedLayoutMagnitude::Finite(4));
        assert_eq!(option_ceiling.align, 4);
        assert_eq!(option_ceiling.stride, CheckedLayoutMagnitude::Finite(4));
    });
}

#[test]
fn primitive_buffers_retain_allocation_checks_accesses_and_cleanup() {
    let source = br#"fn make() -> result: own buffer<u16> allocates(heap) {
  return buffer_new(4_u64, 3_u16);
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let values = make();
  let length = len(values);
  let ok = ilt(2_u64, length);
  if ok {
  } else {
    return exit_status(code: 1_u8);
  }
  set values[2_u64] = 9_u16;
  let stored = values[2_u64];
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("primitive buffer family must check: {outcome:?}");
        };
        let make = &checked.data.functions[0];
        assert!(make.declared_allocates_heap);
        assert!(!make.declared_traps);
        assert!(matches!(
            &make.body[0],
            CheckedStatement::Return {
                value: CheckedExpression::BufferFill {
                    element: CheckedFlatElement::Integer(IntegerType::U16),
                    target_domains,
                    layout_ceiling,
                    ..
                },
                ..
            } if layout_ceiling.stride == CheckedLayoutMagnitude::Finite(2)
                && target_domains.allocation()
                    == CheckedTargetDomainObligation::RuntimeSizedAllocation
                && target_domains.element_address()
                    == CheckedTargetDomainObligation::ElementAddress
        ));

        let main = &checked.data.functions[1];
        assert!(main.declared_allocates_heap);
        assert!(!main.declared_traps);

        let CheckedStatement::Set { target, .. } = &main.body[4] else {
            panic!("the statement after the dominating length branch must be indexed SET-1");
        };
        let CheckedSetTarget::BufferIndex(target) = target else {
            panic!("SET-1 target must retain its buffer root and OP-4 obligation");
        };
        assert_eq!(
            target.root.element,
            CheckedFlatElement::Integer(IntegerType::U16)
        );
        assert!(!target.obligation.components().is_empty());
        assert_eq!(
            target.target_domain,
            CheckedTargetDomainObligation::ElementAddress
        );
        assert!(matches!(
            &main.body[0],
            CheckedStatement::Let {
                value: CheckedExpression::UserCall { .. },
                ..
            }
        ));
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
                    obligation,
                    target_domain: CheckedTargetDomainObligation::ElementAddress,
                    ..
                },
                ..
            } if !obligation.components().is_empty()
        ));
        let CheckedStatement::Return { drops, .. } = &main.body[6] else {
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
        b"command fn main() -> status: own ExitStatus traps {\n  let values = buffer_new(2_u64, 0_u8);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    with_semantics(
        b"command fn main() -> status: own ExitStatus allocates(heap) {\n  let values = buffer_new(2_u64, 0_u8);\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| assert!(matches!(outcome, SemanticOutcome::Complete(_))),
    );
    assert_rule(
        b"command fn main() -> status: own ExitStatus allocates(heap), traps {\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
}

#[test]
fn buffer_vacant_constructs_an_all_none_affine_element_buffer() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let slots = buffer_vacant<box<u64>>(3_u64);
  let count = len(slots);
  return exit_status(code: 0_u8);
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
            layout_ceiling,
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
        assert!(layout_ceiling.stride.allocation_limit() >= 1);
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
        b"command fn main() -> status: own ExitStatus allocates(heap), traps {\n  let slots = buffer_vacant(3_u64);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::InvalidOperation,
    );
    // [EFF-2]: allocation is the only effect; OP-9 is statically discharged.
    assert_rule(
        b"command fn main() -> status: own ExitStatus traps {\n  let slots = buffer_vacant<u32>(3_u64);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Eff2,
        SemanticIssueKind::EffectMismatch,
    );
    with_semantics(
        b"command fn main() -> status: own ExitStatus allocates(heap) {\n  let slots = buffer_vacant<u32>(3_u64);\n  return exit_status(code: 0_u8);\n}\n",
        |outcome| assert!(matches!(outcome, SemanticOutcome::Complete(_))),
    );
    // [TYPE-5]: the one operand is the own u64 length.
    assert_rule(
        b"command fn main() -> status: own ExitStatus allocates(heap), traps {\n  let slots = buffer_vacant<u32>(3_u32);\n  return exit_status(code: 0_u8);\n}\n",
        SemanticRule::Type5,
        SemanticIssueKind::TypeMismatch,
    );
}

#[test]
fn buffer_vacant_rejects_a_region_bearing_payload_under_stor5() {
    assert_rule(
        br#"fn invalid['r](value: own slice<'r, u8>) -> result: own unit allocates(heap), traps {
  let slots = buffer_vacant<slice<'r, u8>>(2_u64);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        br#"command fn main() -> status: own ExitStatus allocates(heap), traps {
  let slots = buffer_vacant<u32>(4_u64);
  region 'v {
    let view = slice_of(&'v slots);
  }
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::CompositeValues,
    );
    // A structural affine element (a nested buffer) is spec-formable
    // [TYPE-2] but has no implemented representation.
    assert_unsupported(
        br#"fn keep(value: own buffer<buffer<u8>>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        UnsupportedSemanticFeature::CompositeValues,
    );
}

#[test]
fn array_elements_stay_copy_only_under_type2() {
    with_semantics(
        br#"fn keep(value: own array<Option<u32>, 2>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        b"command fn main() -> status: own ExitStatus allocates(heap), traps {\n  let initial = False();\n  let values = buffer_new(2_u64, initial);\n  return exit_status(code: 0_u8);\n}\n",
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

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let left = buffer_new(4_u64, 0_u64);
  let right = buffer_new(4_u64, 0_u64);
  let columns = Columns(left: move left, right: move right);
  let left_room = len(columns.left);
  let ok = ilt(2_u64, left_room);
  claim left_sized: ok because "premises: columns.left is the buffer created by buffer_new(4_u64, 0_u64), so left_room is 4_u64\nderivation: 2_u64 is strictly less than 4_u64\nconclusion: 2_u64 is strictly less than left_room\nchecker gap: ENT does not retain the allocation length through nominal construction and projected buffer length\nconsumers: the following projected set and read operations require this exact index bound";
  set columns.left[2_u64] = 7_u64;
  let length = len(columns.right);
  let value = columns.left[2_u64];
  return exit_status(code: 0_u8);
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
        let CheckedStatement::Return { drops, .. } = &main.body[9] else {
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
        b"enum MaybeBuffer {\n  Empty();\n  Full(value: buffer<u8>);\n}\n\nfn abandon(value: own MaybeBuffer) -> result: own unit pure {\n  return unit;\n}\n\ncommand fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
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

fn take(owner: own Owner) -> result: own buffer<u8> pure {
  return move owner.pair.first;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        br#"fn invalid['r](value: own buffer<slice<'r, u8>>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
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
        br#"fn invalid['r](value: own slice<'r, u8>) -> result: own unit allocates(heap), traps {
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

/// [ENT-5]'s element-storage exception holds across a callee boundary: one
/// `len` bound above every write still discharges a later requirement.
///
/// The rule says "for each length term `len(P)`, the root binding of `P` but
/// not `P`'s element storage", so an element write never kills a length fact.
/// A writer reading the rule's kill clause concluded the opposite and re-bound
/// `len` after every call that wrote through a `&uniq` parameter: 34 of the 41
/// length bindings in five programs existed only to re-establish a fact that
/// had never died. Nothing in the tree showed a length fact surviving a write,
/// so the belief cost them nothing the compiler could tell them about.
///
/// The second half is what makes the first half worth stating: the guard is
/// load-bearing. Without any live length fact the call is undischarged, so the
/// hoisted fact is doing real work rather than being redundant ceremony.
#[test]
fn a_hoisted_length_fact_survives_a_callee_element_write() {
    with_semantics(
        br#"fn fill['d](destination: &uniq 'd buffer<u8>, at: own u64) -> result: own u64 reads(destination), writes(destination) {
  doc "Writes one byte through the borrow and returns the next position.";
  let room = len(deref(destination));
  let inside = ilt(at, room);
  if inside {
    set deref(destination)[at] = 65_u8;
  }
  let next = at +wrap 1_u64;
  return next;
}

fn emit['s](source: &'s buffer<u8>, length: own u64) -> result: own u64 reads(source) contract {
  define capacity = len(deref(source));
  requires ile(length, capacity);
} {
  doc "Consumes a prefix whose length the caller has proved.";
  let first = 0_u8;
  let present = ilt(0_u64, length);
  if present {
    set first = deref(source)[0_u64];
  }
  return length;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  doc "Emits a prefix without a live length fact.";
  let line = buffer_new(64_u64, 0_u8);
  let room = len(line);
  let end = 0_u64;
  region 'x {
    set end = fill<'x>(destination: &uniq 'x line, at: end);
  }
  let fits = ile(end, room);
  if fits {
    region 'e {
      let sent = emit<'e>(source: &'e line, length: end);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::Complete(_) = outcome else {
                panic!("a length fact bound above the write must survive it: {outcome:?}");
            };
        },
    );
    with_semantics(
        br#"fn fill['d](destination: &uniq 'd buffer<u8>, at: own u64) -> result: own u64 reads(destination), writes(destination) {
  doc "Writes one byte through the borrow and returns the next position.";
  let room = len(deref(destination));
  let inside = ilt(at, room);
  if inside {
    set deref(destination)[at] = 65_u8;
  }
  let next = at +wrap 1_u64;
  return next;
}

fn emit['s](source: &'s buffer<u8>, length: own u64) -> result: own u64 reads(source) contract {
  define capacity = len(deref(source));
  requires ile(length, capacity);
} {
  doc "Consumes a prefix whose length the caller has proved.";
  let first = 0_u8;
  let present = ilt(0_u64, length);
  if present {
    set first = deref(source)[0_u64];
  }
  return length;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  doc "Emits a prefix without a live length fact.";
  let line = buffer_new(64_u64, 0_u8);
  let end = 0_u64;
  region 'x {
    set end = fill<'x>(destination: &uniq 'x line, at: end);
  }
  region 'e {
    let sent = emit<'e>(source: &'e line, length: end);
  }
  return exit_status(code: 0_u8);
}
"#,
        |outcome| {
            let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
                panic!("with no length fact at all the call is undischarged: {outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Fn8);
        },
    );
}
