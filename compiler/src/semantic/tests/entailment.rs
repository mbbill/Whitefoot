//! Unit tests for the L0 entailment engine, one family per [ENT] rule,
//! including the adversarial stale-fact and fresh-binding shapes the spec
//! text was reviewed against.
//!
//! Derivation tests observe the engine through the retained obligation and
//! claim dispositions via the test-only dark checker, which skips the
//! [OP-4]/[CLM-2] rejection so a function's complete summary stays
//! observable. The rejection behavior itself is tested at the end of this
//! file through the ordinary acceptance path.

use crate::{
    CallRequirementDisposition, SemanticIssueKind, SemanticOutcome, SemanticRule, SourceInput,
};

use super::super::entailment::{
    CallGoalDisposition, CallGoalEvidence, CallGoalOutcome, ClaimDisposition, ClaimOutcome,
    CountedAtomicDerivation, CountedCaptureSide, CountedDerivationSet, CountedProofPoint,
    CountedRootAtom, DerivationId, DerivationNode, DerivationRootKind, FlowEvent, FlowEventId,
    FlowEventKind, FunctionEntailment, GoalId, GoalSign, ImplicitBoundKind, JoinParent,
    LengthBound, ObligationOutcome, PlaceProjection, Relation, TermId, TermKind, ZERO, type_range,
};
use super::super::model::{CheckedExpression, CheckedStatement, IntegerType};
use super::{assert_rule, with_semantics, with_semantics_dark, with_semantics_inputs};

fn obligations(source: &[u8], function: &str) -> Vec<ObligationOutcome> {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("entailment test source must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"));
        function.entailment.obligations.clone()
    })
}

fn claims(source: &[u8], function: &str) -> Vec<ClaimOutcome> {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("entailment test source must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"));
        function.entailment.claims.clone()
    })
}

fn call_goals(source: &[u8], function: &str) -> Vec<CallGoalOutcome> {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("call-goal test source must check completely: {outcome:?}");
        };
        let function = checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"));
        function.entailment.call_goals.clone()
    })
}

fn entailment(source: &[u8], function: &str) -> FunctionEntailment {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("entailment test source must check completely: {outcome:?}");
        };
        checked
            .data
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
            .unwrap_or_else(|| panic!("function {function} must exist"))
            .entailment
            .clone()
    })
}

fn entailments(source: &[u8], function: &str) -> Vec<FunctionEntailment> {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("entailment test source must check completely: {outcome:?}");
        };
        checked
            .data
            .functions
            .iter()
            .filter(|candidate| candidate.name == function)
            .map(|candidate| candidate.entailment.clone())
            .collect()
    })
}

fn discharge_flags(source: &[u8], function: &str) -> Vec<bool> {
    obligations(source, function)
        .iter()
        .map(|outcome| outcome.discharged)
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum DerivationConclusion {
    Relation(Relation),
    Goal { goal: GoalId, sign: GoalSign },
    Contradiction,
}

fn retained_term(summary: &FunctionEntailment, id: TermId) -> &TermKind {
    summary
        .inventory
        .terms
        .get(id.0 as usize)
        .unwrap_or_else(|| panic!("retained term ID {id:?} must resolve"))
}

fn assert_relation_terms_resolve(summary: &FunctionEntailment, relation: &Relation) {
    for term in relation.terms() {
        retained_term(summary, term);
    }
}

fn retained_conclusion(
    conclusions: &[DerivationConclusion],
    id: DerivationId,
) -> &DerivationConclusion {
    conclusions
        .get(id.0 as usize)
        .unwrap_or_else(|| panic!("retained derivation ID {id:?} must resolve"))
}

fn retained_event(summary: &FunctionEntailment, id: FlowEventId) -> &FlowEvent {
    summary
        .derivations
        .events
        .get(id.0 as usize)
        .unwrap_or_else(|| panic!("retained flow event ID {id:?} must resolve"))
}

fn node_event(node: &DerivationNode) -> Option<FlowEventId> {
    match node {
        DerivationNode::SourceBound { event, .. }
        | DerivationNode::SourceDistinct { event, .. }
        | DerivationNode::SourceGoal { event, .. }
        | DerivationNode::JoinBound { event, .. }
        | DerivationNode::JoinDistinct { event, .. }
        | DerivationNode::JoinGoal { event, .. }
        | DerivationNode::JoinContradiction { event, .. }
        | DerivationNode::MaterializedBound { event, .. }
        | DerivationNode::MaterializedDistinct { event, .. }
        | DerivationNode::MaterializedGoal { event, .. }
        | DerivationNode::MaterializedContradiction { event, .. } => Some(*event),
        _ => None,
    }
}

fn root_contains(
    summary: &FunctionEntailment,
    root: DerivationId,
    predicate: impl Fn(&DerivationNode) -> bool,
) -> bool {
    let mut seen = vec![false; summary.derivations.nodes.len()];
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        let index = id.0 as usize;
        if seen[index] {
            continue;
        }
        seen[index] = true;
        let node = &summary.derivations.nodes[index];
        if predicate(node) {
            return true;
        }
        stack.extend(node.parent_ids());
    }
    false
}

fn obligation_root(summary: &FunctionEntailment, ordinal: usize) -> DerivationId {
    summary.obligations[ordinal]
        .derivation
        .unwrap_or_else(|| panic!("obligation {ordinal} must have one exact root"))
}

fn call_root(summary: &FunctionEntailment, ordinal: usize) -> DerivationId {
    summary.call_goals[ordinal]
        .derivation
        .unwrap_or_else(|| panic!("call goal {ordinal} must have one exact root"))
}

fn assert_root_contains(
    summary: &FunctionEntailment,
    root: DerivationId,
    predicate: impl Fn(&DerivationNode) -> bool,
    description: &str,
) {
    assert!(
        root_contains(summary, root, predicate),
        "root {root:?} must contain {description}: {:#?}",
        summary.derivations.nodes
    );
}

fn root_has_event_kind(
    summary: &FunctionEntailment,
    root: DerivationId,
    kind: FlowEventKind,
) -> bool {
    root_contains(summary, root, |node| {
        node_event(node).is_some_and(|event| retained_event(summary, event).kind == kind)
    })
}

fn assert_root_has_event_kind(
    summary: &FunctionEntailment,
    root: DerivationId,
    kind: FlowEventKind,
) {
    assert!(
        root_has_event_kind(summary, root, kind),
        "root {root:?} must descend from {kind:?}: {:#?}",
        summary.derivations.nodes
    );
}

fn normalized_derivation_dump(summary: &FunctionEntailment) -> Vec<u8> {
    format!(
        "{:#?}",
        (
            &summary.derivations.events,
            &summary.derivations.nodes,
            &summary.derivations.roots,
            &summary.counted_derivations,
            &summary.inventory,
        )
    )
    .into_bytes()
}

fn assert_source_event(summary: &FunctionEntailment, id: FlowEventId, used: &mut [bool]) {
    let event = retained_event(summary, id);
    used[id.0 as usize] = true;
    assert!(matches!(
        event.kind,
        FlowEventKind::S1
            | FlowEventKind::S2
            | FlowEventKind::S3
            | FlowEventKind::S4
            | FlowEventKind::S5
            | FlowEventKind::S6
            | FlowEventKind::S7
            | FlowEventKind::S9
            | FlowEventKind::S10
            | FlowEventKind::S11
    ));
}

fn assert_synthetic_event(
    summary: &FunctionEntailment,
    id: FlowEventId,
    kind: FlowEventKind,
    used: &mut [bool],
) {
    let event = retained_event(summary, id);
    used[id.0 as usize] = true;
    assert_eq!(event.kind, kind);
    assert!(event.node_path.is_none());
}

fn assert_join_parents(
    parents: &[JoinParent],
    conclusions: &[DerivationConclusion],
    accepts: impl Fn(&DerivationConclusion) -> bool,
    require_contributor: bool,
) {
    let mut has_noncontradictory_parent = false;
    for (ordinal, parent) in parents.iter().enumerate() {
        assert_eq!(
            parent.ordinal,
            u32::try_from(ordinal).expect("test join ordinal fits u32")
        );
        let conclusion = retained_conclusion(conclusions, parent.parent);
        has_noncontradictory_parent |= !matches!(conclusion, DerivationConclusion::Contradiction);
        assert!(
            matches!(conclusion, DerivationConclusion::Contradiction) || accepts(conclusion),
            "join predecessor {parent:?} has incompatible conclusion {conclusion:?}"
        );
    }
    if require_contributor {
        assert!(
            has_noncontradictory_parent,
            "a noncontradictory joined fact needs a contributing predecessor"
        );
    }
}

fn term_integer_range(kind: &TermKind) -> Option<(i128, i128)> {
    match kind {
        TermKind::Place(_, ty) | TermKind::ProjectedPlace(_, ty) => Some(type_range(*ty)),
        TermKind::Length(_) | TermKind::ProjectedLength(_) | TermKind::CountedCapture { .. } => {
            Some(type_range(IntegerType::U64))
        }
        TermKind::Zero | TermKind::Constant(_) | TermKind::ConstParameter(_) => None,
    }
}

fn assert_array_length_bound(
    summary: &FunctionEntailment,
    left: TermId,
    right: TermId,
    bound: i128,
) {
    let relation_matches = |length: TermId, length_bound: LengthBound| match length_bound {
        LengthBound::Constant(value) => {
            (left == length && right == ZERO && bound == value)
                || (left == ZERO && right == length && bound == -value)
        }
        LengthBound::Equal(parameter) => {
            ((left == length && right == parameter) || (left == parameter && right == length))
                && bound == 0
        }
    };
    let matched = [left, right].into_iter().any(|candidate| {
        matches!(
            retained_term(summary, candidate),
            TermKind::Length(_) | TermKind::ProjectedLength(_)
        ) && summary.inventory.length_bounds[candidate.0 as usize]
            .is_some_and(|length_bound| relation_matches(candidate, length_bound))
    });
    assert!(matched, "array-length implicit bound must resolve exactly");
}

fn counted_atoms(
    counted: &CountedDerivationSet,
) -> [(CountedRootAtom, &CountedAtomicDerivation); 8] {
    [
        (
            CountedRootAtom::LowerCaptureToEndpoint,
            &counted.lower_capture_eq_endpoint.forward,
        ),
        (
            CountedRootAtom::LowerEndpointToCapture,
            &counted.lower_capture_eq_endpoint.reverse,
        ),
        (
            CountedRootAtom::UpperCaptureToEndpoint,
            &counted.upper_capture_eq_endpoint.forward,
        ),
        (
            CountedRootAtom::UpperEndpointToCapture,
            &counted.upper_capture_eq_endpoint.reverse,
        ),
        (
            CountedRootAtom::BinderToLowerCapture,
            &counted.binder_eq_lower_capture.forward,
        ),
        (
            CountedRootAtom::LowerCaptureToBinder,
            &counted.binder_eq_lower_capture.reverse,
        ),
        (
            CountedRootAtom::LowerCaptureLeBinder,
            &counted.lower_capture_le_binder.atomic,
        ),
        (
            CountedRootAtom::BinderLtUpperCapture,
            &counted.binder_lt_upper_capture.atomic,
        ),
    ]
}

const fn counted_atom_index(atom: CountedRootAtom) -> usize {
    match atom {
        CountedRootAtom::LowerCaptureToEndpoint => 0,
        CountedRootAtom::LowerEndpointToCapture => 1,
        CountedRootAtom::UpperCaptureToEndpoint => 2,
        CountedRootAtom::UpperEndpointToCapture => 3,
        CountedRootAtom::BinderToLowerCapture => 4,
        CountedRootAtom::LowerCaptureToBinder => 5,
        CountedRootAtom::LowerCaptureLeBinder => 6,
        CountedRootAtom::BinderLtUpperCapture => 7,
    }
}

fn assert_counted_atomic_parent(
    summary: &FunctionEntailment,
    conclusions: &[DerivationConclusion],
    counted: &CountedDerivationSet,
    atomic: &CountedAtomicDerivation,
) {
    let Relation::Bound { left, right, bound } = atomic.relation else {
        panic!("every counted atomic root must name one normalized bound");
    };
    retained_term(summary, left);
    retained_term(summary, right);
    match retained_conclusion(conclusions, atomic.parent) {
        DerivationConclusion::Relation(Relation::Bound {
            left: parent_left,
            right: parent_right,
            bound: parent_bound,
        }) => {
            assert_eq!((*parent_left, *parent_right), (left, right));
            assert_eq!(
                *parent_bound, bound,
                "the retained parent must prove the exact normalized S11 relation"
            );
        }
        DerivationConclusion::Contradiction => {}
        conclusion => panic!("counted atomic parent is incompatible: {conclusion:?}"),
    }
    let parent = &summary.derivations.nodes[atomic.parent.0 as usize];
    match atomic.proof_point {
        CountedProofPoint::PreheaderSnapshot => assert!(matches!(
            parent,
            DerivationNode::MaterializedBound { .. }
                | DerivationNode::MaterializedContradiction { .. }
        )),
        CountedProofPoint::BodyEntry => {
            let DerivationNode::SourceBound { event, .. } = parent else {
                panic!("a non-reconstructed body-entry root must be its S11 source bound");
            };
            let event = retained_event(summary, *event);
            assert_eq!(event.kind, FlowEventKind::S11);
            assert_eq!(event.node_path.as_ref(), Some(&counted.counted_node_path));
        }
    }
}

fn validate_counted_derivation_set(
    summary: &FunctionEntailment,
    conclusions: &[DerivationConclusion],
    counted: &CountedDerivationSet,
) {
    assert!(!counted.counted_node_path.components().is_empty());
    let Relation::Equal {
        left: lower_capture,
        right: lower_endpoint,
    } = counted.lower_capture_eq_endpoint.relation
    else {
        panic!("the first counted semantic root must be the lower capture equality");
    };
    let Relation::Equal {
        left: upper_capture,
        right: upper_endpoint,
    } = counted.upper_capture_eq_endpoint.relation
    else {
        panic!("the second counted semantic root must be the upper capture equality");
    };
    let Relation::Equal {
        left: binder,
        right: binder_lower_capture,
    } = counted.binder_eq_lower_capture.relation
    else {
        panic!("the third counted semantic root must be binder initialization");
    };
    assert_eq!(binder_lower_capture, lower_capture);
    assert!(matches!(
        retained_term(summary, lower_capture),
        TermKind::CountedCapture {
            range_path,
            side: CountedCaptureSide::Lower,
        } if range_path == counted.counted_node_path.components()
    ));
    assert!(matches!(
        retained_term(summary, upper_capture),
        TermKind::CountedCapture {
            range_path,
            side: CountedCaptureSide::Upper,
        } if range_path == counted.counted_node_path.components()
    ));
    assert!(matches!(
        retained_term(summary, binder),
        TermKind::Place(_, IntegerType::U64)
    ));
    assert!(!matches!(
        retained_term(summary, lower_endpoint),
        TermKind::CountedCapture { .. }
    ));
    assert!(!matches!(
        retained_term(summary, upper_endpoint),
        TermKind::CountedCapture { .. }
    ));

    let equality_atoms = [
        (
            &counted.lower_capture_eq_endpoint,
            lower_capture,
            lower_endpoint,
        ),
        (
            &counted.upper_capture_eq_endpoint,
            upper_capture,
            upper_endpoint,
        ),
        (&counted.binder_eq_lower_capture, binder, lower_capture),
    ];
    for (equality, left, right) in equality_atoms {
        assert_eq!(
            equality.forward.relation,
            Relation::Bound {
                left,
                right,
                bound: 0,
            }
        );
        assert_eq!(
            equality.reverse.relation,
            Relation::Bound {
                left: right,
                right: left,
                bound: 0,
            }
        );
        assert_eq!(
            equality.forward.proof_point,
            CountedProofPoint::PreheaderSnapshot
        );
        assert_eq!(
            equality.reverse.proof_point,
            CountedProofPoint::PreheaderSnapshot
        );
    }
    let lower_bound = Relation::Bound {
        left: lower_capture,
        right: binder,
        bound: 0,
    };
    let upper_bound = Relation::Bound {
        left: binder,
        right: upper_capture,
        bound: -1,
    };
    assert_eq!(counted.lower_capture_le_binder.relation, lower_bound);
    assert_eq!(counted.lower_capture_le_binder.atomic.relation, lower_bound);
    assert_eq!(counted.binder_lt_upper_capture.relation, upper_bound);
    assert_eq!(counted.binder_lt_upper_capture.atomic.relation, upper_bound);
    assert_eq!(
        counted.lower_capture_le_binder.atomic.proof_point,
        CountedProofPoint::BodyEntry
    );
    assert_eq!(
        counted.binder_lt_upper_capture.atomic.proof_point,
        CountedProofPoint::BodyEntry
    );
    for (_, atomic) in counted_atoms(counted) {
        assert_counted_atomic_parent(summary, conclusions, counted, atomic);
    }
}

fn validate_derivations(summary: &FunctionEntailment) {
    assert_eq!(
        summary.inventory.terms.len(),
        summary.inventory.length_bounds.len(),
        "term and length-bound inventories stay densely aligned"
    );
    assert_eq!(retained_term(summary, ZERO), &TermKind::Zero);

    let mut conclusions = Vec::with_capacity(summary.derivations.nodes.len());
    let mut depths = Vec::with_capacity(summary.derivations.nodes.len());
    let mut used_events = vec![false; summary.derivations.events.len()];
    let mut parent_edges = 0usize;

    for (index, node) in summary.derivations.nodes.iter().enumerate() {
        let parents = node.parent_ids();
        parent_edges += parents.len();
        assert!(
            parents.iter().all(|parent| parent.0 < index as u32),
            "every derivation parent must precede its child"
        );
        let depth = parents
            .iter()
            .map(|parent| depths[parent.0 as usize])
            .max()
            .map_or(0, |depth: u32| depth + 1);
        depths.push(depth);

        let conclusion = match node {
            DerivationNode::SourceBound {
                relation,
                left,
                right,
                bound,
                event,
            } => {
                assert_relation_terms_resolve(summary, relation);
                retained_term(summary, *left);
                retained_term(summary, *right);
                assert_source_event(summary, *event, &mut used_events);
                match relation {
                    Relation::Bound {
                        left: source_left,
                        right: source_right,
                        bound: source_bound,
                    } => assert_eq!(
                        (left, right, bound),
                        (source_left, source_right, source_bound)
                    ),
                    Relation::Equal {
                        left: source_left,
                        right: source_right,
                    } => {
                        assert_eq!(*bound, 0);
                        assert!(
                            (*left == *source_left && *right == *source_right)
                                || (*left == *source_right && *right == *source_left)
                        );
                    }
                    Relation::Distinct { .. } => panic!("a distinct source is not a bound"),
                }
                DerivationConclusion::Relation(Relation::Bound {
                    left: *left,
                    right: *right,
                    bound: *bound,
                })
            }
            DerivationNode::SourceDistinct { left, right, event } => {
                assert!(left <= right, "disequality identities are normalized");
                retained_term(summary, *left);
                retained_term(summary, *right);
                assert_source_event(summary, *event, &mut used_events);
                DerivationConclusion::Relation(Relation::Distinct {
                    left: *left,
                    right: *right,
                })
            }
            DerivationNode::SourceGoal { goal, sign, event } => {
                assert!(summary.inventory.goals.get(goal.0 as usize).is_some());
                assert_source_event(summary, *event, &mut used_events);
                DerivationConclusion::Goal {
                    goal: *goal,
                    sign: *sign,
                }
            }
            DerivationNode::ImplicitBound {
                left,
                right,
                bound,
                kind,
            } => {
                let left_kind = retained_term(summary, *left);
                let right_kind = retained_term(summary, *right);
                match kind {
                    ImplicitBoundKind::Reflexive => {
                        assert_eq!(left, right);
                        assert_eq!(*bound, 0);
                    }
                    ImplicitBoundKind::Constant => match (left_kind, right_kind) {
                        (TermKind::Constant(value), TermKind::Zero) => assert_eq!(bound, value),
                        (TermKind::Zero, TermKind::Constant(value)) => assert_eq!(*bound, -value),
                        _ => panic!("constant implicit bound must relate that constant to Z"),
                    },
                    ImplicitBoundKind::TypeMaximum => {
                        assert_eq!(*right, ZERO);
                        let (_, maximum) = term_integer_range(left_kind)
                            .expect("type maximum requires an integer-like term");
                        assert_eq!(*bound, maximum);
                    }
                    ImplicitBoundKind::TypeMinimum => {
                        assert_eq!(*left, ZERO);
                        let (minimum, _) = term_integer_range(right_kind)
                            .expect("type minimum requires an integer-like term");
                        assert_eq!(*bound, -minimum);
                    }
                    ImplicitBoundKind::ArrayLength => {
                        assert_array_length_bound(summary, *left, *right, *bound);
                    }
                }
                DerivationConclusion::Relation(Relation::Bound {
                    left: *left,
                    right: *right,
                    bound: *bound,
                })
            }
            DerivationNode::TransitiveBound {
                left,
                middle,
                right,
                bound,
                first,
                second,
            } => {
                let DerivationConclusion::Relation(Relation::Bound {
                    left: first_left,
                    right: first_right,
                    bound: first_bound,
                }) = retained_conclusion(&conclusions, *first)
                else {
                    panic!("transitivity first parent must be a bound");
                };
                let DerivationConclusion::Relation(Relation::Bound {
                    left: second_left,
                    right: second_right,
                    bound: second_bound,
                }) = retained_conclusion(&conclusions, *second)
                else {
                    panic!("transitivity second parent must be a bound");
                };
                assert_eq!((first_left, first_right), (left, middle));
                assert_eq!((second_left, second_right), (middle, right));
                assert_eq!(*bound, first_bound.saturating_add(*second_bound));
                DerivationConclusion::Relation(Relation::Bound {
                    left: *left,
                    right: *right,
                    bound: *bound,
                })
            }
            DerivationNode::StrengthenedBound {
                left,
                right,
                bound,
                weak,
                distinct,
            } => {
                assert_eq!(*bound, -1);
                assert_eq!(
                    retained_conclusion(&conclusions, *weak),
                    &DerivationConclusion::Relation(Relation::Bound {
                        left: *left,
                        right: *right,
                        bound: 0,
                    })
                );
                let DerivationConclusion::Relation(Relation::Distinct {
                    left: distinct_left,
                    right: distinct_right,
                }) = retained_conclusion(&conclusions, *distinct)
                else {
                    panic!("strengthening's second parent must be a disequality");
                };
                assert!(
                    (*distinct_left == *left && *distinct_right == *right)
                        || (*distinct_left == *right && *distinct_right == *left)
                );
                DerivationConclusion::Relation(Relation::Bound {
                    left: *left,
                    right: *right,
                    bound: *bound,
                })
            }
            DerivationNode::SubsumedBound {
                left,
                right,
                held,
                requested,
                parent,
            } => {
                assert!(*held < *requested);
                assert_eq!(
                    retained_conclusion(&conclusions, *parent),
                    &DerivationConclusion::Relation(Relation::Bound {
                        left: *left,
                        right: *right,
                        bound: *held,
                    })
                );
                DerivationConclusion::Relation(Relation::Bound {
                    left: *left,
                    right: *right,
                    bound: *requested,
                })
            }
            DerivationNode::Equality {
                left,
                right,
                forward,
                reverse,
            } => {
                assert_eq!(
                    retained_conclusion(&conclusions, *forward),
                    &DerivationConclusion::Relation(Relation::Bound {
                        left: *left,
                        right: *right,
                        bound: 0,
                    })
                );
                assert_eq!(
                    retained_conclusion(&conclusions, *reverse),
                    &DerivationConclusion::Relation(Relation::Bound {
                        left: *right,
                        right: *left,
                        bound: 0,
                    })
                );
                DerivationConclusion::Relation(Relation::Equal {
                    left: *left,
                    right: *right,
                })
            }
            DerivationNode::DisequalityFromStrictBound {
                left,
                right,
                parent,
            } => {
                assert!(left < right, "strict-derived disequalities are normalized");
                let DerivationConclusion::Relation(Relation::Bound {
                    left: parent_left,
                    right: parent_right,
                    bound: parent_bound,
                }) = retained_conclusion(&conclusions, *parent)
                else {
                    panic!("strict-bound disequality requires a bound parent");
                };
                assert!(*parent_bound <= -1);
                assert!(
                    (*parent_left == *left && *parent_right == *right)
                        || (*parent_left == *right && *parent_right == *left)
                );
                DerivationConclusion::Relation(Relation::Distinct {
                    left: *left,
                    right: *right,
                })
            }
            DerivationNode::GoalProjection {
                goal,
                sign,
                relation,
                parent,
            } => {
                assert_relation_terms_resolve(summary, relation);
                assert_eq!(
                    retained_conclusion(&conclusions, *parent),
                    &DerivationConclusion::Relation(relation.clone())
                );
                let retained_goal = summary
                    .inventory
                    .goals
                    .get(goal.0 as usize)
                    .expect("projected goal ID must resolve");
                let projection = retained_goal
                    .projection
                    .as_ref()
                    .expect("projection node requires a projected goal");
                let expected = match sign {
                    GoalSign::Positive => projection.clone(),
                    GoalSign::Negative => projection.negated(),
                };
                assert_eq!(*relation, expected);
                DerivationConclusion::Goal {
                    goal: *goal,
                    sign: *sign,
                }
            }
            DerivationNode::L0Contradiction { term, parent } => {
                retained_term(summary, *term);
                let DerivationConclusion::Relation(Relation::Bound { left, right, bound }) =
                    retained_conclusion(&conclusions, *parent)
                else {
                    panic!("L0 contradiction requires a bound parent");
                };
                assert_eq!((left, right), (term, term));
                assert!(*bound < 0);
                DerivationConclusion::Contradiction
            }
            DerivationNode::GoalContradiction {
                goal,
                positive,
                negative,
            } => {
                assert!(summary.inventory.goals.get(goal.0 as usize).is_some());
                assert_eq!(
                    retained_conclusion(&conclusions, *positive),
                    &DerivationConclusion::Goal {
                        goal: *goal,
                        sign: GoalSign::Positive,
                    }
                );
                assert_eq!(
                    retained_conclusion(&conclusions, *negative),
                    &DerivationConclusion::Goal {
                        goal: *goal,
                        sign: GoalSign::Negative,
                    }
                );
                DerivationConclusion::Contradiction
            }
            DerivationNode::JoinBound {
                left,
                right,
                bound,
                event,
                parents,
            } => {
                retained_term(summary, *left);
                retained_term(summary, *right);
                assert_synthetic_event(summary, *event, FlowEventKind::Join, &mut used_events);
                assert_join_parents(
                    parents,
                    &conclusions,
                    |conclusion| {
                        matches!(
                            conclusion,
                            DerivationConclusion::Relation(Relation::Bound {
                                left: parent_left,
                                right: parent_right,
                                bound: parent_bound,
                            }) if parent_left == left && parent_right == right && parent_bound <= bound
                        )
                    },
                    true,
                );
                DerivationConclusion::Relation(Relation::Bound {
                    left: *left,
                    right: *right,
                    bound: *bound,
                })
            }
            DerivationNode::JoinDistinct {
                left,
                right,
                event,
                parents,
            } => {
                assert!(left <= right, "joined disequalities are normalized");
                retained_term(summary, *left);
                retained_term(summary, *right);
                assert_synthetic_event(summary, *event, FlowEventKind::Join, &mut used_events);
                assert_join_parents(
                    parents,
                    &conclusions,
                    |conclusion| {
                        conclusion
                            == &DerivationConclusion::Relation(Relation::Distinct {
                                left: *left,
                                right: *right,
                            })
                    },
                    true,
                );
                DerivationConclusion::Relation(Relation::Distinct {
                    left: *left,
                    right: *right,
                })
            }
            DerivationNode::JoinGoal {
                goal,
                sign,
                event,
                parents,
            } => {
                assert!(summary.inventory.goals.get(goal.0 as usize).is_some());
                assert_synthetic_event(summary, *event, FlowEventKind::Join, &mut used_events);
                assert_join_parents(
                    parents,
                    &conclusions,
                    |conclusion| {
                        conclusion
                            == &DerivationConclusion::Goal {
                                goal: *goal,
                                sign: *sign,
                            }
                    },
                    true,
                );
                DerivationConclusion::Goal {
                    goal: *goal,
                    sign: *sign,
                }
            }
            DerivationNode::JoinContradiction { event, parents } => {
                assert_synthetic_event(summary, *event, FlowEventKind::Join, &mut used_events);
                assert_join_parents(
                    parents,
                    &conclusions,
                    |conclusion| matches!(conclusion, DerivationConclusion::Contradiction),
                    false,
                );
                assert!(parents.iter().all(|parent| matches!(
                    retained_conclusion(&conclusions, parent.parent),
                    DerivationConclusion::Contradiction
                )));
                DerivationConclusion::Contradiction
            }
            DerivationNode::MaterializedBound {
                left,
                right,
                bound,
                event,
                parent,
            } => {
                assert_synthetic_event(summary, *event, FlowEventKind::Snapshot, &mut used_events);
                let conclusion = DerivationConclusion::Relation(Relation::Bound {
                    left: *left,
                    right: *right,
                    bound: *bound,
                });
                assert_eq!(retained_conclusion(&conclusions, *parent), &conclusion);
                conclusion
            }
            DerivationNode::MaterializedDistinct {
                left,
                right,
                event,
                parent,
            } => {
                assert_synthetic_event(summary, *event, FlowEventKind::Snapshot, &mut used_events);
                let conclusion = DerivationConclusion::Relation(Relation::Distinct {
                    left: *left,
                    right: *right,
                });
                assert_eq!(retained_conclusion(&conclusions, *parent), &conclusion);
                conclusion
            }
            DerivationNode::MaterializedGoal {
                goal,
                sign,
                event,
                parent,
            } => {
                assert!(summary.inventory.goals.get(goal.0 as usize).is_some());
                assert_synthetic_event(summary, *event, FlowEventKind::Snapshot, &mut used_events);
                let conclusion = DerivationConclusion::Goal {
                    goal: *goal,
                    sign: *sign,
                };
                assert_eq!(retained_conclusion(&conclusions, *parent), &conclusion);
                conclusion
            }
            DerivationNode::MaterializedContradiction { event, parent } => {
                assert_synthetic_event(summary, *event, FlowEventKind::Snapshot, &mut used_events);
                assert_eq!(
                    retained_conclusion(&conclusions, *parent),
                    &DerivationConclusion::Contradiction
                );
                DerivationConclusion::Contradiction
            }
        };
        conclusions.push(conclusion);
    }

    assert!(
        used_events.iter().all(|used| *used),
        "finish must prune every unreferenced flow event"
    );
    assert_eq!(
        summary.derivations.metrics.unique_nodes as usize,
        summary.derivations.nodes.len()
    );
    assert_eq!(
        summary.derivations.metrics.parent_edges as usize,
        parent_edges
    );
    assert_eq!(
        summary.derivations.metrics.maximum_depth,
        depths.iter().copied().max().unwrap_or(0)
    );
    if !summary.derivations.nodes.is_empty() {
        assert!(summary.derivations.metrics.retained_bytes > 0);
    }

    for counted in &summary.counted_derivations {
        validate_counted_derivation_set(summary, &conclusions, counted);
    }

    let mut seen_obligations = vec![false; summary.obligations.len()];
    let mut seen_calls = vec![false; summary.call_goals.len()];
    let mut seen_counted = vec![[false; 8]; summary.counted_derivations.len()];
    let mut counted_root_order = Vec::new();
    let mut class_counts = [0u32; 4];
    for root in &summary.derivations.roots {
        let conclusion = retained_conclusion(&conclusions, root.node);
        match root.kind {
            DerivationRootKind::BoundsObligation(ordinal) => {
                let ordinal = ordinal as usize;
                let outcome = summary
                    .obligations
                    .get(ordinal)
                    .expect("bounds-root ordinal must resolve");
                assert!(!seen_obligations[ordinal], "one exact root per obligation");
                seen_obligations[ordinal] = true;
                assert!(outcome.discharged);
                assert_eq!(outcome.conjunct, 0);
                assert_eq!(outcome.derivation, Some(root.node));
                assert!(!outcome.node_path.components().is_empty());
                retained_term(summary, outcome.requested.right);
                match conclusion {
                    DerivationConclusion::Relation(relation) => {
                        let left = outcome
                            .requested
                            .left
                            .expect("noncontradictory accepted bound has a tracked offset");
                        retained_term(summary, left);
                        assert_eq!(
                            relation,
                            &Relation::Bound {
                                left,
                                right: outcome.requested.right,
                                bound: outcome.requested.bound,
                            }
                        );
                        assert!(!outcome.contradictory);
                    }
                    DerivationConclusion::Contradiction => assert!(outcome.contradictory),
                    DerivationConclusion::Goal { .. } => {
                        panic!("a bounds root cannot conclude a goal")
                    }
                }
            }
            DerivationRootKind::CallGoal(ordinal) => {
                let ordinal = ordinal as usize;
                let outcome = summary
                    .call_goals
                    .get(ordinal)
                    .expect("call-root ordinal must resolve");
                assert!(!seen_calls[ordinal], "one exact root per discharged call");
                seen_calls[ordinal] = true;
                assert_eq!(outcome.disposition, CallGoalDisposition::Discharged);
                assert_eq!(outcome.derivation, Some(root.node));
                assert!(!outcome.node_path.components().is_empty());
                assert!(!outcome.final_check.components().is_empty());
                match conclusion {
                    DerivationConclusion::Goal {
                        goal,
                        sign: GoalSign::Positive,
                    } => {
                        let retained_goal = summary
                            .inventory
                            .goals
                            .get(goal.0 as usize)
                            .expect("call goal ID must resolve");
                        assert_eq!(retained_goal.expression, outcome.goal.root);
                    }
                    DerivationConclusion::Contradiction => {}
                    DerivationConclusion::Goal {
                        sign: GoalSign::Negative,
                        ..
                    }
                    | DerivationConclusion::Relation(_) => {
                        panic!("a discharged call root must be positive or contradictory")
                    }
                }
            }
            DerivationRootKind::CountedS11 { occurrence, atom } => {
                counted_root_order.push((occurrence, atom));
                let occurrence = occurrence as usize;
                let counted = summary
                    .counted_derivations
                    .get(occurrence)
                    .expect("counted-root occurrence must resolve");
                let index = counted_atom_index(atom);
                assert!(
                    !seen_counted[occurrence][index],
                    "one exact root per counted atomic relation"
                );
                seen_counted[occurrence][index] = true;
                let expected = counted_atoms(counted)
                    .into_iter()
                    .find_map(|(candidate, atomic)| (candidate == atom).then_some(atomic.parent))
                    .expect("the fixed counted atom must exist");
                assert_eq!(root.node, expected);
            }
        }

        match &summary.derivations.nodes[root.node.0 as usize] {
            DerivationNode::SourceGoal { .. }
            | DerivationNode::JoinGoal { .. }
            | DerivationNode::MaterializedGoal { .. } => class_counts[1] += 1,
            DerivationNode::GoalProjection { .. } => class_counts[2] += 1,
            DerivationNode::L0Contradiction { .. }
            | DerivationNode::GoalContradiction { .. }
            | DerivationNode::JoinContradiction { .. }
            | DerivationNode::MaterializedContradiction { .. } => class_counts[3] += 1,
            _ => class_counts[0] += 1,
        }
    }

    let mut reachable = vec![false; summary.derivations.nodes.len()];
    let mut stack: Vec<_> = summary
        .derivations
        .roots
        .iter()
        .map(|root| root.node)
        .collect();
    while let Some(id) = stack.pop() {
        let index = id.0 as usize;
        if reachable[index] {
            continue;
        }
        reachable[index] = true;
        stack.extend(summary.derivations.nodes[index].parent_ids());
    }
    assert!(
        reachable.iter().all(|is_reachable| *is_reachable),
        "finish must prune every node outside the mandatory-root sub-DAG"
    );

    for (ordinal, outcome) in summary.obligations.iter().enumerate() {
        assert_eq!(outcome.derivation.is_some(), outcome.discharged);
        assert_eq!(seen_obligations[ordinal], outcome.discharged);
    }
    for (ordinal, outcome) in summary.call_goals.iter().enumerate() {
        let discharged = outcome.disposition == CallGoalDisposition::Discharged;
        assert_eq!(outcome.derivation.is_some(), discharged);
        assert_eq!(seen_calls[ordinal], discharged);
    }
    assert!(
        seen_counted
            .iter()
            .all(|occurrence| occurrence.iter().all(|seen| *seen)),
        "every counted statement must retain all eight atomic roots"
    );
    let expected_counted_order: Vec<_> = summary
        .counted_derivations
        .iter()
        .enumerate()
        .flat_map(|(occurrence, counted)| {
            counted_atoms(counted).map(|(atom, _)| {
                (
                    u32::try_from(occurrence).expect("counted test occurrence fits u32"),
                    atom,
                )
            })
        })
        .collect();
    assert_eq!(
        counted_root_order, expected_counted_order,
        "counted ledger roots stay grouped in source occurrence and normative atom order"
    );
    assert_eq!(
        class_counts,
        [
            summary.derivations.metrics.bounds_roots,
            summary.derivations.metrics.opaque_goal_roots,
            summary.derivations.metrics.projected_goal_roots,
            summary.derivations.metrics.contradiction_roots,
        ]
    );
}

fn assert_derivation_mutation_rejected(
    summary: &FunctionEntailment,
    mutate: impl FnOnce(&mut FunctionEntailment),
) {
    let mut mutant = summary.clone();
    mutate(&mut mutant);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            validate_derivations(&mutant);
        }))
        .is_err(),
        "the hostile counted-root mutation must fail the structural checker"
    );
}

#[derive(Debug, Default, Eq, PartialEq)]
struct DistinctGroundCounts {
    source: usize,
    strict: usize,
    contradiction: usize,
    joins: usize,
    join_edges: usize,
    join_parent_counts: Vec<usize>,
}

fn collect_distinct_grounds(
    summary: &FunctionEntailment,
    id: DerivationId,
    counts: &mut DistinctGroundCounts,
) {
    match &summary.derivations.nodes[id.0 as usize] {
        DerivationNode::SourceDistinct { .. } => counts.source += 1,
        DerivationNode::DisequalityFromStrictBound { .. } => counts.strict += 1,
        DerivationNode::JoinDistinct { parents, .. } => {
            counts.joins += 1;
            counts.join_edges += parents.len();
            counts.join_parent_counts.push(parents.len());
            for parent in parents {
                collect_distinct_grounds(summary, parent.parent, counts);
            }
        }
        DerivationNode::MaterializedDistinct { parent, .. } => {
            collect_distinct_grounds(summary, *parent, counts);
        }
        DerivationNode::L0Contradiction { .. }
        | DerivationNode::GoalContradiction { .. }
        | DerivationNode::JoinContradiction { .. }
        | DerivationNode::MaterializedContradiction { .. } => counts.contradiction += 1,
        node => panic!("distinct proof has incompatible ground {node:?}"),
    }
}

fn projected_call_parent(summary: &FunctionEntailment, ordinal: usize) -> DerivationId {
    let root = summary.call_goals[ordinal]
        .derivation
        .expect("discharged call must have one exact root");
    let DerivationNode::GoalProjection { parent, .. } = &summary.derivations.nodes[root.0 as usize]
    else {
        panic!("call root must be its exact L0 goal projection");
    };
    *parent
}

#[test]
fn accepted_transitive_bounds_and_discharged_calls_retain_exact_parent_roots() {
    let source = br#"const count: u64 = 4_u64;

struct Pair {
  count: u64;
}

fn below(value: own u64) -> own unit traps requires {
  check ilt(value, 4_u64) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn read(values: own array<i32, count>, p: own Pair, i: own u64) -> own i32 traps {
  if ile(i, p.count) {
    if ilt(p.count, 4_u64) {
      let item = values[i];
      below(value: i);
      return item;
    } else {
      return 0_i32;
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(summary.obligations.len(), 1);
    assert_eq!(summary.call_goals.len(), 1);
    assert!(summary.obligations[0].discharged);
    assert_eq!(
        summary.call_goals[0].disposition,
        CallGoalDisposition::Discharged
    );
    let bounds_root = summary.obligations[0]
        .derivation
        .expect("accepted subscript must retain an exact root");
    let call_root = summary.call_goals[0]
        .derivation
        .expect("discharged ordinary call must retain an exact root");
    assert_ne!(bounds_root, call_root);
    assert_eq!(summary.derivations.roots.len(), 2);
    assert!(
        summary
            .derivations
            .nodes
            .iter()
            .any(|node| matches!(node, DerivationNode::TransitiveBound { .. }))
    );
    for (index, node) in summary.derivations.nodes.iter().enumerate() {
        let parents: Vec<_> = match node {
            DerivationNode::TransitiveBound { first, second, .. } => vec![*first, *second],
            _ => Vec::new(),
        };
        assert!(parents.iter().all(|parent| parent.0 < index as u32));
    }
}

#[test]
fn normalized_derivations_are_byte_identical_across_twenty_analyses() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, left: own Bool) -> own i32 pure {
  if left {
    if ilt(i, 4_u64) {
    } else {
      return 0_i32;
    }
  } else if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let first = entailment(source, "read");
    validate_derivations(&first);
    assert!(
        first
            .derivations
            .nodes
            .iter()
            .any(|node| matches!(node, DerivationNode::JoinBound { .. }))
    );
    let expected = normalized_derivation_dump(&first);
    for run in 1..20 {
        let actual = entailment(source, "read");
        validate_derivations(&actual);
        assert_eq!(
            normalized_derivation_dump(&actual),
            expected,
            "normalized function-local ledger changed on run {run}"
        );
    }
}

// ---------------------------------------------------------------------
// [ENT-3] S1 branch facts and their exact negation
// ---------------------------------------------------------------------

#[test]
fn a_dominating_branch_discharges_the_guarded_index_and_not_the_other_arm() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
    return values[i];
  } else {
    return values[i];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    let outcomes = &summary.obligations;
    assert_eq!(outcomes.len(), 2);
    assert!(outcomes[0].discharged, "True arm carries i < 4 = len");
    assert!(!outcomes[1].discharged, "False arm carries only i >= 4");
    assert_eq!(outcomes[1].residual.as_deref(), Some("i < len(values)"));
}

#[test]
fn a_projected_bool_scrutinee_retains_its_exact_s1_carrier() {
    let source = br#"struct Flags {
  ready: Bool;
}

fn need_ready(value: own Bool) -> own unit traps requires {
  check value else trap "ready";
} {
  check True() else trap "body";
  return unit;
}

fn caller(flags: own Flags) -> own unit traps {
  if flags.ready {
    need_ready(value: flags.ready);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("projected Bool S1 fixture must check: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        let CheckedStatement::Match { scrutinee, .. } = &caller.body[0] else {
            panic!("projected Bool branch must retain one checked match");
        };
        assert!(matches!(scrutinee, CheckedExpression::Project { .. }));
        let expected = scrutinee
            .carrier()
            .expect("projected Bool has an exact carrier");
        validate_derivations(&caller.entailment);
        let call = &caller.entailment.call_goals[0];
        assert_eq!(call.disposition, CallGoalDisposition::Discharged);
        let root = call.derivation.expect("S1 whole-goal call root");
        let DerivationNode::SourceGoal { event, .. } =
            &caller.entailment.derivations.nodes[root.0 as usize]
        else {
            panic!("the exact projected whole goal must select its S1 opaque root");
        };
        let retained = retained_event(&caller.entailment, *event);
        assert_eq!(retained.kind, FlowEventKind::S1);
        assert_eq!(retained.node_path.as_ref(), Some(expected));
    });
}

#[test]
fn s1_true_and_false_edges_retain_their_exact_comparison_roots() {
    let source = br#"fn need_below(value: own u64) -> own unit traps requires {
  check ilt(value, 4_u64) else trap "below";
} {
  check True() else trap "body";
  return unit;
}

fn need_at_least(value: own u64) -> own unit traps requires {
  check ige(value, 4_u64) else trap "at least";
} {
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  if ilt(value, 4_u64) {
    need_below(value: value);
  } else {
    need_at_least(value: value);
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "caller");
    validate_derivations(&summary);
    assert_eq!(summary.call_goals.len(), 2);
    for ordinal in 0..2 {
        assert_root_has_event_kind(&summary, call_root(&summary, ordinal), FlowEventKind::S1);
    }
    assert!(matches!(
        summary.derivations.nodes[call_root(&summary, 0).0 as usize],
        DerivationNode::SourceGoal { .. }
    ));
    assert!(matches!(
        summary.derivations.nodes[call_root(&summary, 1).0 as usize],
        DerivationNode::GoalProjection { .. }
    ));
    assert!(matches!(
        summary.derivations.nodes[projected_call_parent(&summary, 1).0 as usize],
        DerivationNode::SourceBound { .. }
    ));
}

#[test]
fn a_constant_offset_discharges_against_a_const_array_and_a_too_large_one_reports() {
    let source = br#"const count: u64 = 4_u64;

const table: array<u8, count> =[10_u8, 20_u8, 30_u8, 40_u8];

fn read() -> own u8 pure {
  let inside = table[2_u64];
  let outside = table[9_u64];
  return inside;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    let outcomes = &summary.obligations;
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes[0].discharged,
        "2 < 4 by the implicit length equality"
    );
    assert!(!outcomes[1].discharged, "9 < 4 is not derivable");
    assert_eq!(outcomes[1].residual.as_deref(), Some("9_u64 < len(table)"));
    assert!(outcomes[1].derivation.is_none());
    let root = obligation_root(&summary, 0);
    assert_root_contains(
        &summary,
        root,
        |node| {
            matches!(
                node,
                DerivationNode::ImplicitBound {
                    kind: ImplicitBoundKind::Constant,
                    ..
                }
            )
        },
        "the literal constant implicit fact",
    );
    assert_root_contains(
        &summary,
        root,
        |node| {
            matches!(
                node,
                DerivationNode::ImplicitBound {
                    kind: ImplicitBoundKind::ArrayLength,
                    ..
                }
            )
        },
        "the named constant array's implicit length",
    );
    assert_root_contains(
        &summary,
        root,
        |node| matches!(node, DerivationNode::SubsumedBound { .. }),
        "requested-bound subsumption",
    );
}

// ---------------------------------------------------------------------
// [ENT-3] comparison origin (b) and its path validity
// ---------------------------------------------------------------------

#[test]
fn a_bool_binding_carries_its_comparison_to_the_match_when_no_kill_intervenes() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let flag = ilt(i, 4_u64);
  if flag {
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "read"), vec![true]);
}

#[test]
fn a_set_between_initializer_and_use_invalidates_the_comparison_origin() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let flag = ilt(i, 4_u64);
  set i = i +wrap 1_u64;
  if flag {
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the origin's operand fact was killed by the assignment"
    );
}

// ---------------------------------------------------------------------
// [ENT-4] closure: transitivity, strengthening, contradiction, and the
// flow/closure boundary
// ---------------------------------------------------------------------

#[test]
fn transitivity_composes_branch_facts_through_a_middle_term() {
    let source = br#"const count: u64 = 4_u64;

struct Pair {
  count: u64;
  other: u64;
}

fn read(values: own array<i32, count>, p: own Pair, i: own u64) -> own i32 pure {
  if ile(i, p.count) {
    if ilt(p.count, 4_u64) {
      return values[i];
    } else {
      return 0_i32;
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "i <= p.count and p.count < 4 compose to i < len(values)"
    );
}

#[test]
fn disequality_strengthens_a_weak_bound_to_a_strict_one() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ile(i, 4_u64) {
    if ieq(i, 4_u64) {
      return 0_i32;
    } else {
      return values[i];
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "i <= 4 with i != 4 strengthens to i <= 3 < len(values)"
    );
}

#[test]
fn equality_retains_both_directed_parents_and_reflexive_implicit_support() {
    let source = br#"fn need_equal(left: own u64, right: own u64) -> own unit traps requires {
  check ieq(left, right) else trap "equal";
} {
  check True() else trap "body";
  return unit;
}

fn directed(left: own u64, right: own u64) -> own unit traps {
  check ile(left, right) else trap "forward";
  check ile(right, left) else trap "reverse";
  need_equal(left: left, right: right);
  return unit;
}

fn reflexive(value: own u64) -> own unit traps {
  need_equal(left: value, right: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let directed = entailment(source, "directed");
    validate_derivations(&directed);
    let equality = projected_call_parent(&directed, 0);
    let DerivationNode::Equality {
        forward, reverse, ..
    } = &directed.derivations.nodes[equality.0 as usize]
    else {
        panic!("the exact equality projection must name both directed bounds");
    };
    for parent in [*forward, *reverse] {
        let DerivationNode::SourceBound { event, .. } =
            &directed.derivations.nodes[parent.0 as usize]
        else {
            panic!("each directed equality parent comes from its passed check");
        };
        assert_eq!(retained_event(&directed, *event).kind, FlowEventKind::S2);
    }

    let reflexive = entailment(source, "reflexive");
    validate_derivations(&reflexive);
    assert_root_contains(
        &reflexive,
        call_root(&reflexive, 0),
        |node| {
            matches!(
                node,
                DerivationNode::ImplicitBound {
                    kind: ImplicitBoundKind::Reflexive,
                    ..
                }
            )
        },
        "the reflexive implicit bound",
    );
}

#[test]
fn a_contradictory_state_discharges_every_obligation() {
    let source = br#"const count: u64 = 4_u64;

fn below_minimum(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 0_u64) {
    return values[9_u64];
  } else {
    return 0_i32;
  }
}

fn above_maximum(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if igt(i, 18446744073709551615_u64) {
    return values[9_u64];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for (function, kind) in [
        ("below_minimum", ImplicitBoundKind::TypeMinimum),
        ("above_maximum", ImplicitBoundKind::TypeMaximum),
    ] {
        let summary = entailment(source, function);
        validate_derivations(&summary);
        assert_eq!(summary.obligations.len(), 1, "{function}");
        assert!(summary.obligations[0].discharged, "{function}");
        assert!(summary.obligations[0].contradictory, "{function}");
        let root = obligation_root(&summary, 0);
        assert!(matches!(
            summary.derivations.nodes[root.0 as usize],
            DerivationNode::L0Contradiction { .. }
        ));
        assert_root_contains(
            &summary,
            root,
            |node| matches!(node, DerivationNode::ImplicitBound { kind: actual, .. } if *actual == kind),
            "the matching integer type range",
        );
    }
}

#[test]
fn a_kill_between_establishment_and_query_breaks_an_underived_chain() {
    // The flow carries established facts and closure happens at the query
    // [ENT-3, ENT-4]: consuming the middle term's root before the query
    // leaves the endpoints unrelated, because i - Z was never established
    // as its own fact on this straight-line path.
    let source = br#"const count: u64 = 4_u64;

struct Pair {
  count: u64;
  other: u64;
}

fn eat(p: own Pair) -> own unit pure {
  return unit;
}

fn read(values: own array<i32, count>, p: own Pair, i: own u64) -> own i32 pure {
  if ile(i, p.count) {
    if ilt(p.count, 4_u64) {
      eat(p: move p);
      return values[i];
    } else {
      return 0_i32;
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![false],
        "consuming p kills both links before any join materializes i <= 3"
    );
    assert!(summary.obligations[0].derivation.is_none());
}

// ---------------------------------------------------------------------
// [ENT-5] kills: assignment overlap and effect-row write projection
// ---------------------------------------------------------------------

#[test]
fn an_assignment_to_a_sibling_field_keeps_facts_and_to_the_fact_field_kills_them() {
    let source = br#"const count: u64 = 4_u64;

struct Pair {
  count: u64;
  other: u64;
}

fn read(values: own array<i32, count>, p: own Pair) -> own i32 pure {
  if ilt(p.count, 4_u64) {
    set p.other = 9_u64;
    let kept = values[p.count];
    set p.count = 9_u64;
    let lost = values[p.count];
    return kept;
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true, false],
        "OWN-7 overlap: p.other is disjoint from p.count; p.count is not"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S1);
    assert!(summary.obligations[1].derivation.is_none());
}

#[test]
fn a_callee_writing_through_a_unique_borrow_kills_facts_on_that_place() {
    let source = br#"const count: u64 = 4_u64;

fn bump['w](p: &uniq 'w u64) -> own unit writes('w) {
  set deref(p) = 9_u64;
  return unit;
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
    region 'w {
      bump<'w>(p: &uniq 'w i);
    }
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![false],
        "the callee's writes row projects onto the unique actual's place"
    );
    assert!(summary.obligations[0].derivation.is_none());
}

#[test]
fn a_callee_with_no_writes_row_kills_nothing() {
    let source = br#"const count: u64 = 4_u64;

fn peek['r](p: &'r u64) -> own u64 reads('r) {
  return deref(p);
}

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
    region 'r {
      let seen = peek<'r>(p: &'r i);
    }
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "a call whose row carries no writes kills nothing"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S1);
}

// ---------------------------------------------------------------------
// [ENT-5] joins and scope-exit ordering
// ---------------------------------------------------------------------

#[test]
fn a_join_keeps_the_weakest_bound_held_on_every_continuing_arm() {
    let source = br#"const two: u64 = 2_u64;

const count: u64 = 4_u64;

fn read(wide: own array<i32, count>, narrow: own array<i32, two>, i: own u64) -> own i32 pure {
  if ilt(i, 2_u64) {
  } else if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  let in_wide = wide[i];
  let in_narrow = narrow[i];
  return in_wide;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true, false],
        "the join keeps i <= 3 (weakest across arms), not the True arm's i <= 1"
    );
    assert_root_contains(
        &summary,
        obligation_root(&summary, 0),
        |node| matches!(node, DerivationNode::JoinBound { parents, .. } if parents.len() == 2),
        "the predecessor-complete joined bound",
    );
}

#[test]
fn a_join_keeps_a_disequality_derived_in_opposite_strict_orientations() {
    let source = br#"fn need_distinct(left: own u64, right: own u64) -> own unit pure requires {
  check ine(left, right) else trap "distinct";
} {
  return unit;
}

fn need_left_below_right(left: own u64, right: own u64) -> own unit pure requires {
  check ilt(left, right) else trap "left below right";
} {
  return unit;
}

fn need_right_below_left(left: own u64, right: own u64) -> own unit pure requires {
  check ilt(right, left) else trap "right below left";
} {
  return unit;
}

fn caller(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "left below right";
  } else {
    check ilt(right, left) else trap "right below left";
  }
  need_distinct(left: left, right: right);
  need_left_below_right(left: left, right: right);
  need_right_below_left(left: left, right: right);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "caller");
    validate_derivations(&summary);
    let outcomes = &summary.call_goals;
    assert_eq!(outcomes.len(), 3);
    assert_eq!(
        outcomes[0].disposition,
        CallGoalDisposition::Discharged,
        "each strict orientation derives the same disequality, so ENT-5 keeps it"
    );
    assert_eq!(
        outcomes[0].evidence,
        vec![CallGoalEvidence::ExactL0Projection]
    );
    assert_eq!(outcomes[1].disposition, CallGoalDisposition::Unproved);
    assert_eq!(outcomes[2].disposition, CallGoalDisposition::Unproved);
    assert!(outcomes[1].evidence.is_empty());
    assert!(outcomes[2].evidence.is_empty());
    let mut counts = DistinctGroundCounts::default();
    collect_distinct_grounds(&summary, projected_call_parent(&summary, 0), &mut counts);
    assert_eq!(
        counts,
        DistinctGroundCounts {
            strict: 2,
            joins: 1,
            join_edges: 2,
            join_parent_counts: vec![2],
            ..DistinctGroundCounts::default()
        },
        "the normalized joined disequality names both opposite strict parents"
    );
}

#[test]
fn a_joined_derived_disequality_strengthens_a_later_weak_bound() {
    let source =
        br#"fn need_left_below_right(left: own u64, right: own u64) -> own unit pure requires {
  check ilt(left, right) else trap "left below right";
} {
  return unit;
}

fn caller(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "left below right";
  } else {
    check ilt(right, left) else trap "right below left";
  }
  check ile(left, right) else trap "later weak bound";
  need_left_below_right(left: left, right: right);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "caller");
    validate_derivations(&summary);
    let outcomes = &summary.call_goals;
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        outcomes[0].evidence,
        vec![CallGoalEvidence::ExactL0Projection],
        "the joined disequality and later weak bound strengthen to the strict requirement"
    );
    let parent = projected_call_parent(&summary, 0);
    let DerivationNode::StrengthenedBound { distinct, .. } =
        &summary.derivations.nodes[parent.0 as usize]
    else {
        panic!("post-join weak bound must be strengthened by the joined disequality");
    };
    let mut counts = DistinctGroundCounts::default();
    collect_distinct_grounds(&summary, *distinct, &mut counts);
    assert_eq!(counts.strict, 2);
    assert_eq!(counts.joins, 1);
    assert_eq!(counts.join_edges, 2);
}

#[test]
fn a_write_kills_a_disequality_materialized_by_a_join() {
    let source = br#"fn need_distinct(left: own u64, right: own u64) -> own unit pure requires {
  check ine(left, right) else trap "distinct";
} {
  return unit;
}

fn kept(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "left below right";
  } else {
    check ilt(right, left) else trap "right below left";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn killed(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "left below right";
  } else {
    check ilt(right, left) else trap "right below left";
  }
  set left = left +wrap 1_u64;
  need_distinct(left: left, right: right);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let kept_summary = entailment(source, "kept");
    validate_derivations(&kept_summary);
    let kept = &kept_summary.call_goals;
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(kept[0].evidence, vec![CallGoalEvidence::ExactL0Projection]);
    let mut kept_counts = DistinctGroundCounts::default();
    collect_distinct_grounds(
        &kept_summary,
        projected_call_parent(&kept_summary, 0),
        &mut kept_counts,
    );
    assert_eq!(kept_counts.strict, 2);
    assert_eq!(kept_counts.join_edges, 2);

    let killed_summary = entailment(source, "killed");
    validate_derivations(&killed_summary);
    let killed = &killed_summary.call_goals;
    assert_eq!(killed.len(), 1);
    assert_eq!(killed[0].disposition, CallGoalDisposition::Unproved);
    assert!(killed[0].evidence.is_empty());
    assert!(killed_summary.derivations.roots.is_empty());
    assert!(killed_summary.derivations.nodes.is_empty());
    assert!(killed_summary.derivations.events.is_empty());
}

#[test]
fn joins_keep_disequality_across_same_strict_explicit_and_mixed_grounds() {
    let source = br#"fn need_distinct(left: own u64, right: own u64) -> own unit pure requires {
  check ine(left, right) else trap "distinct";
} {
  return unit;
}

fn same_strict(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "first strict";
  } else {
    check ilt(left, right) else trap "second strict";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn both_explicit(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ine(left, right) else trap "first distinct";
  } else {
    check ine(right, left) else trap "second distinct";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn mixed(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ine(left, right) else trap "explicit";
  } else {
    check ilt(right, left) else trap "strict";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in ["same_strict", "both_explicit", "mixed"] {
        let summary = entailment(source, function);
        validate_derivations(&summary);
        let outcomes = &summary.call_goals;
        assert_eq!(outcomes.len(), 1, "{function}");
        assert_eq!(
            outcomes[0].disposition,
            CallGoalDisposition::Discharged,
            "{function} establishes the same normalized disequality on every input"
        );
        assert_eq!(
            outcomes[0].evidence,
            vec![CallGoalEvidence::ExactL0Projection],
            "{function} discharges through the common L0 relation"
        );
        if function == "mixed" {
            let mut counts = DistinctGroundCounts::default();
            collect_distinct_grounds(&summary, projected_call_parent(&summary, 0), &mut counts);
            assert_eq!(
                counts,
                DistinctGroundCounts {
                    source: 1,
                    strict: 1,
                    joins: 1,
                    join_edges: 2,
                    join_parent_counts: vec![2],
                    ..DistinctGroundCounts::default()
                },
                "the mixed join names its explicit and strict-derived predecessor roots"
            );
        }
    }
}

#[test]
fn a_many_way_join_keeps_mixed_disequality_and_ignores_a_contradictory_input() {
    let source = br#"fn need_distinct(left: own u64, right: own u64) -> own unit pure requires {
  check ine(left, right) else trap "distinct";
} {
  return unit;
}

fn caller(left: own u64, right: own u64, first: own Bool, second: own Bool, third: own Bool) -> own unit traps {
  if first {
    check ilt(left, right) else trap "left below right";
  } else if second {
    check ine(left, right) else trap "explicit distinct";
  } else if third {
    check ilt(right, left) else trap "right below left";
  } else {
    check ilt(0_u64, 0_u64) else trap "contradictory input";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "caller");
    validate_derivations(&summary);
    let outcomes = &summary.call_goals;
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        outcomes[0].evidence,
        vec![CallGoalEvidence::ExactL0Projection]
    );
    let mut counts = DistinctGroundCounts::default();
    collect_distinct_grounds(&summary, projected_call_parent(&summary, 0), &mut counts);
    assert_eq!(counts.source, 1);
    assert_eq!(counts.strict, 2);
    assert_eq!(counts.contradiction, 1);
    assert_eq!(counts.joins, 3);
    counts.join_parent_counts.sort_unstable();
    assert_eq!(counts.join_parent_counts, vec![2, 2, 2]);
    assert_eq!(
        counts.join_edges, 6,
        "the three binary joins transitively name all four reaching inputs"
    );
}

#[test]
fn equality_missing_relation_and_a_kill_each_prevent_disequality_survival() {
    let source = br#"fn need_distinct(left: own u64, right: own u64) -> own unit pure requires {
  check ine(left, right) else trap "distinct";
} {
  return unit;
}

fn equality_input(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "strict";
  } else {
    check ieq(left, right) else trap "equal";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn missing_input(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "strict";
  } else {
    check True() else trap "no relation";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn killed_input(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, right) else trap "strict then killed";
    set left = left +wrap 1_u64;
  } else {
    check ilt(right, left) else trap "other strict";
  }
  need_distinct(left: left, right: right);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in ["equality_input", "missing_input", "killed_input"] {
        let summary = entailment(source, function);
        validate_derivations(&summary);
        let outcomes = &summary.call_goals;
        assert_eq!(outcomes.len(), 1, "{function}");
        assert_eq!(
            outcomes[0].disposition,
            CallGoalDisposition::Unproved,
            "{function} has at least one reaching input without the same live disequality"
        );
        assert!(outcomes[0].evidence.is_empty(), "{function}");
    }
}

#[test]
fn derived_disequality_closure_preserves_contradiction_and_no_loop_induction() {
    let source = br#"fn impossible() -> own unit pure requires {
  check ilt(1_u64, 0_u64) else trap "impossible";
} {
  return unit;
}

fn reverse_weak_transitivity_control(left: own u64, right: own u64) -> own unit traps {
  check ile(left, right) else trap "weak";
  check ilt(right, left) else trap "strict reverse";
  impossible();
  return unit;
}

fn both_strict(left: own u64, right: own u64) -> own unit traps {
  check ilt(left, right) else trap "first strict";
  check ilt(right, left) else trap "second strict";
  impossible();
  return unit;
}

fn all_contradictory(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, left) else trap "left contradiction";
  } else {
    check ilt(right, right) else trap "right contradiction";
  }
  impossible();
  return unit;
}

fn no_induction(left: own u64, right: own u64, leave: own Bool) -> own unit traps {
  loop @again {
    need_distinct(left: left, right: right);
    check ilt(left, right) else trap "inside only";
    if leave {
      break @again;
    }
  }
  return unit;
}

fn need_distinct(left: own u64, right: own u64) -> own unit pure requires {
  check ine(left, right) else trap "distinct";
} {
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in [
        "reverse_weak_transitivity_control",
        "both_strict",
        "all_contradictory",
    ] {
        let summary = entailment(source, function);
        validate_derivations(&summary);
        let outcomes = &summary.call_goals;
        assert_eq!(outcomes.len(), 1, "{function}");
        assert_eq!(
            outcomes[0].disposition,
            CallGoalDisposition::Discharged,
            "{function} reaches an all-derivable state; the reverse-weak case is a transitivity contradiction control"
        );
        assert_eq!(
            outcomes[0].evidence,
            vec![CallGoalEvidence::AllDerivable],
            "{function}"
        );
        if function == "all_contradictory" {
            assert_root_contains(
                &summary,
                call_root(&summary, 0),
                |node| {
                    matches!(
                        node,
                        DerivationNode::JoinContradiction { parents, .. }
                            if parents.len() == 2
                    )
                },
                "the binary all-contradictory join with both predecessor roots",
            );
        }
    }
    let loop_summary = entailment(source, "no_induction");
    validate_derivations(&loop_summary);
    let loop_outcomes = &loop_summary.call_goals;
    assert_eq!(loop_outcomes.len(), 1);
    assert_eq!(
        loop_outcomes[0].disposition,
        CallGoalDisposition::Unproved,
        "a relation established inside one ordinary iteration is not induced at its head"
    );
    assert!(loop_outcomes[0].evidence.is_empty());
}

#[test]
fn an_arm_that_leaves_by_return_contributes_nothing_to_the_join() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "only the True arm reaches the continuation, so its fact survives"
    );
}

#[test]
fn a_fresh_binding_reusing_an_expired_spelling_inherits_no_stale_fact() {
    // The stale-fact/fresh-binding attack shape: each arm declares its own
    // `j`; the second is a distinct declaration event [ENT-2] and no fact
    // established for the first may attach to it.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, pick: own Bool) -> own i32 pure {
  if pick {
    let j = 0_u64;
    if ilt(j, 4_u64) {
      return values[j];
    } else {
      return 0_i32;
    }
  } else {
    let j = 9_u64;
    return values[j];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(summary.obligations.len(), 2);
    assert!(
        summary.obligations[0].discharged,
        "the first j is branch-guarded"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S5);
    assert!(
        !summary.obligations[1].discharged,
        "the second j is a fresh declaration event with no facts"
    );
    assert!(summary.obligations[1].derivation.is_none());
}

#[test]
fn a_fact_about_an_outer_binding_survives_a_region_exit() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  region 'a {
    if ilt(i, 4_u64) {
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "scope-exit kills reach only bindings whose scope ends at the edge"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S1);
}

// ---------------------------------------------------------------------
// [ENT-5] break, give, and propagate edges
// ---------------------------------------------------------------------

#[test]
fn a_break_edge_carries_surviving_facts_to_the_loop_continuation() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  loop @l {
    if ilt(i, 4_u64) {
      break @l;
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the break edge exits only loop-local scopes; the fact on i survives"
    );
}

#[test]
fn a_kill_before_the_break_edge_leaves_the_continuation_unproved() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  loop @l {
    if ilt(i, 4_u64) {
      set i = i +wrap 1_u64;
      break @l;
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the assignment killed the branch fact before the break edge"
    );
}

#[test]
fn give_edges_join_at_the_value_match_continuation_with_arm_facts_dead() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let picked = if ilt(i, 4_u64) {
    give values[i];
  } else {
    give 0_i32;
  }
  let after = values[i];
  return picked;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes[0].discharged,
        "inside the True arm the S1 fact discharges the given index"
    );
    assert!(
        !outcomes[1].discharged,
        "after the give join neither arm's exclusive fact survives"
    );
}

#[test]
fn a_propagate_continuation_keeps_prior_facts_when_the_call_writes_nothing() {
    let source = br#"const count: u64 = 4_u64;

enum Fail {
  Bad();
}

fn source(flag: own Bool) -> own Result<u64, Fail> pure {
  if flag {
    return Ok<u64, Fail>(value: 1_u64);
  } else {
    let bad = Bad();
    return Err<u64, Fail>(error: bad);
  }
}

fn read(values: own array<i32, count>, i: own u64, flag: own Bool) -> own Result<i32, Fail> pure {
  if ilt(i, 4_u64) {
    let v = propagate source(flag: flag);
    let a = values[i];
    return Ok<i32, Fail>(value: a);
  } else {
    return Ok<i32, Fail>(value: 0_i32);
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the Err edge leaves the function; the normal continuation keeps i < 4"
    );
}

// ---------------------------------------------------------------------
// [ENT-5] the no-induction loop rule
// ---------------------------------------------------------------------

#[test]
fn a_loop_body_kill_removes_the_fact_from_every_iteration_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  let before = values[i];
  loop @l {
    let inside = values[i];
    set i = i +wrap 1_u64;
    if ilt(i, 4_u64) {
    } else {
      break @l;
    }
  }
  return before;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true, false],
        "the head state subtracts every fact the body's assignment may kill"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S1);
    assert!(summary.obligations[1].derivation.is_none());
}

#[test]
fn a_kill_free_loop_body_keeps_the_entry_fact_at_the_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let inside = values[i];
    break @l;
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, true],
        "no kill event in the body touches i, so the fact holds at the head"
    );
}

#[test]
fn d1h_and_d1i_distinguish_a_return_inside_the_loop_from_one_after_it() {
    let source = br#"const count: u64 = 4_u64;

fn return_inside(values: own array<i32, count>, i: own u64, stop: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let at_head = values[i];
    if stop {
      return at_head;
    }
    break @l;
  }
  return 0_i32;
}

fn return_after(values: own array<i32, count>, i: own u64, stop: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let at_head = values[i];
    break @l;
  }
  if stop {
    return 1_i32;
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "return_inside"),
        vec![true],
        "D1h: a return edge reaches no later head, so it cannot erase the entry fact"
    );
    assert_eq!(
        discharge_flags(source, "return_after"),
        vec![true],
        "D1i control: moving the same return after the loop stays discharged"
    );
}

#[test]
fn a_kill_followed_only_by_the_current_loop_break_does_not_poison_the_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let at_head = values[i];
    set i = i +wrap 1_u64;
    break @l;
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the set's only successor leaves this loop, so no later head observes it"
    );
}

#[test]
fn a_kill_followed_only_by_an_enclosing_break_does_not_poison_the_inner_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, leave_outer: own Bool, leave_inner: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @outer {
    loop @inner {
      let at_head = values[i];
      if leave_outer {
        set i = i +wrap 1_u64;
        break @outer;
      }
      if leave_inner {
        break @inner;
      }
    }
    break @outer;
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "the assignment reaches only the enclosing loop's continuation, not the inner head"
    );
}

#[test]
fn a_propagate_error_edge_does_not_poison_the_loop_head() {
    let source = br#"const count: u64 = 4_u64;

enum Fail {
  Bad();
}

fn source(fail: own Bool) -> own Result<u64, Fail> pure {
  if fail {
    let bad = Bad();
    return Err<u64, Fail>(error: bad);
  }
  return Ok<u64, Fail>(value: 1_u64);
}

fn read(values: own array<i32, count>, i: own u64, fail: own Bool, leave: own Bool) -> own Result<i32, Fail> pure {
  if ilt(i, 4_u64) {
  } else {
    return Ok<i32, Fail>(value: 0_i32);
  }
  loop @l {
    let value = propagate source(fail: fail);
    let at_head = values[i];
    if leave {
      break @l;
    }
  }
  return Ok<i32, Fail>(value: 0_i32);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "only propagate's Ok edge can return to the head; its Err edge leaves the function"
    );
}

#[test]
fn an_else_free_continuing_kill_still_poisons_the_loop_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, mutate: own Bool, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    if mutate {
      set i = i +wrap 1_u64;
    }
    let at_head = values[i];
    if leave {
      break @l;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the mutating arm and the else-free false edge both remain inside the body"
    );
}

#[test]
fn a_give_to_an_initializer_inside_the_loop_carries_its_kill_to_the_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, mutate: own Bool, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    let picked = if mutate {
      set i = i +wrap 1_u64;
      give 1_i32;
    } else {
      give 0_i32;
    }
    let at_head = values[i];
    if leave {
      break @l;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the give reaches an initializer continuation inside the body and then the backedge"
    );
}

#[test]
fn a_mixed_branch_ignores_the_return_only_kill_but_keeps_the_continuing_one() {
    let source = br#"const count: u64 = 4_u64;

fn read(left: own array<i32, count>, right: own array<i32, count>, i: own u64, j: own u64, stop: own Bool, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  if ilt(j, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @l {
    if stop {
      set i = i +wrap 1_u64;
      return 0_i32;
    } else {
      set j = j +wrap 1_u64;
    }
    let left_value = left[i];
    let right_value = right[j];
    if leave {
      break @l;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, false],
        "the returning arm's i kill is non-continuing, while the other arm's j kill reaches the backedge"
    );
}

#[test]
fn a_nested_loop_own_break_carries_kills_to_the_outer_loop_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, leave_outer: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @outer {
    let at_head = values[i];
    loop @inner {
      set i = i +wrap 1_u64;
      break @inner;
    }
    if leave_outer {
      break @outer;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the inner break reaches a continuation inside the outer body and then its backedge"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S11 counted-range structural facts
// ---------------------------------------------------------------------

#[test]
fn a_counted_range_discharges_its_binder_and_safe_predecessor_indices() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  let total = 0_i32;
  for @items i in 1_u64..4_u64 {
    let previous = i -wrap 1_u64;
    let current_value = values[i];
    let previous_value = values[previous];
    set total = total +wrap current_value;
    set total = total +wrap previous_value;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, true],
        "S11 proves i < upper and lower <= i; S7 derives the safe predecessor"
    );
}

#[test]
fn a_counted_range_does_not_prove_the_next_index_or_an_unrelated_carried_index() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, j: own u64) -> own i32 pure {
  let total = 0_i32;
  for @items i in 0_u64..4_u64 {
    let next = i +wrap 1_u64;
    let current_value = values[i];
    let next_value = values[next];
    let unrelated = values[j];
    set total = total +wrap current_value;
    set total = total +wrap next_value;
    set total = total +wrap unrelated;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true, false, false],
        "S11 is not general induction: next may equal upper and j is unrelated"
    );
}

#[test]
fn a_counted_upper_needs_an_independent_relation_to_the_storage_length() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, upper: own u64) -> own i32 pure {
  let total = 0_i32;
  for @items i in 0_u64..upper {
    let value = values[i];
    set total = total +wrap value;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "i < captured upper does not imply upper <= len(values)"
    );
}

#[test]
fn a_counted_preheader_closes_snapshot_consequences_before_body_kills() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  let upper = 4_u64;
  let total = 0_i32;
  for @items i in 0_u64..upper {
    set upper = 0_u64;
    let value = values[i];
    set total = total +wrap value;
  }
  return total;
}

fn ordinary(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let upper = 4_u64;
  if ilt(i, upper) {
    set upper = 0_u64;
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "the capture equals four before the mutable source changes; that closed snapshot consequence remains true"
    );
    let root = obligation_root(&summary, 0);
    assert_root_has_event_kind(&summary, root, FlowEventKind::S11);
    assert_root_contains(
        &summary,
        root,
        |node| matches!(node, DerivationNode::MaterializedBound { .. }),
        "the counted preheader materialization marker",
    );
    assert_eq!(
        discharge_flags(source, "ordinary"),
        vec![false],
        "without S11's preheader materialization, the same write kills the ordinary query-derived relation"
    );
}

#[test]
fn counted_roots_cover_hostile_control_edges_and_unused_s11_facts() {
    let source = br#"enum Stop {
  Failed();
}

fn maybe(fail: own Bool) -> own Result<unit, Stop> pure {
  if fail {
    let stopped = Failed();
    return Err<unit, Stop>(error: stopped);
  }
  return Ok<unit, Stop>(value: unit);
}

fn hostile(lower: own u64, upper: own u64, leave: own Bool, fail: own Bool) -> own Result<unit, Stop> pure {
  for @zero zero in 0_u64..0_u64 {
  }
  for @reversed reversed in 2_u64..1_u64 {
  }
  for @singleton singleton in 0_u64..1_u64 {
  }
  for @maximum maximum in 18446744073709551614_u64..18446744073709551615_u64 {
  }
  let mutable_lower = lower;
  let mutable_upper = upper;
  for @mutated at in mutable_lower..mutable_upper {
    set mutable_lower = 0_u64;
    set mutable_upper = 0_u64;
    if leave {
      break @mutated;
    }
  }
  for @returning at in 0_u64..1_u64 {
    if leave {
      return Ok<unit, Stop>(value: unit);
    }
  }
  for @propagating at in 0_u64..1_u64 {
    let ignored = propagate maybe(fail: fail);
  }
  for @outer_counted outer in 0_u64..1_u64 {
    for @inner_counted inner in 0_u64..1_u64 {
      if leave {
        break @inner_counted;
      }
    }
  }
  loop @ordinary {
    for @breaking at in 0_u64..1_u64 {
      if leave {
        break @ordinary;
      } else {
        break @breaking;
      }
    }
    break @ordinary;
  }
  return Ok<unit, Stop>(value: unit);
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "hostile");
    validate_derivations(&summary);
    assert_eq!(summary.counted_derivations.len(), 10);
    assert_eq!(
        summary
            .derivations
            .roots
            .iter()
            .filter(|root| matches!(root.kind, DerivationRootKind::CountedS11 { .. }))
            .count(),
        80
    );
    let paths: std::collections::HashSet<_> = summary
        .counted_derivations
        .iter()
        .map(|counted| counted.counted_node_path.clone())
        .collect();
    assert_eq!(paths.len(), 10, "every counted occurrence is retained once");
    let mutated = &summary.counted_derivations[4];
    for equality in [
        &mutated.lower_capture_eq_endpoint,
        &mutated.upper_capture_eq_endpoint,
    ] {
        assert!(matches!(
            retained_term(
                &summary,
                match equality.relation {
                    Relation::Equal { right, .. } => right,
                    _ => unreachable!(),
                }
            ),
            TermKind::Place(_, IntegerType::U64)
        ));
        assert_eq!(
            equality.forward.proof_point,
            CountedProofPoint::PreheaderSnapshot,
            "both mutable endpoint identities remain rooted at the once-only snapshot"
        );
    }
}

#[test]
fn counted_roots_cover_contradictory_preheaders_and_neutral_join_predecessors() {
    let source = br#"const count: u64 = 1_u64;

fn contradictory(left: own u64, right: own u64, choose: own Bool) -> own unit traps {
  if choose {
    check ilt(left, left) else trap "left contradiction";
  } else {
    check ilt(right, right) else trap "right contradiction";
  }
  for @impossible i in 0_u64..1_u64 {
  }
  return unit;
}

fn joined(values: own array<i32, count>, x: own u64) -> own i32 pure {
  let upper = 1_u64;
  if ilt(x, 0_u64) {
    let impossible = x;
  }
  let total = 0_i32;
  for @items i in 0_u64..upper {
    let item = values[i];
    set total = total +wrap item;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let contradictory = entailment(source, "contradictory");
    validate_derivations(&contradictory);
    assert_eq!(contradictory.counted_derivations.len(), 1);
    for (_, atomic) in counted_atoms(&contradictory.counted_derivations[0])
        .into_iter()
        .take(6)
    {
        assert!(matches!(
            contradictory.derivations.nodes[atomic.parent.0 as usize],
            DerivationNode::MaterializedContradiction { .. }
        ));
        assert_eq!(atomic.proof_point, CountedProofPoint::PreheaderSnapshot);
    }
    for (_, atomic) in counted_atoms(&contradictory.counted_derivations[0])
        .into_iter()
        .skip(6)
    {
        assert!(matches!(
            contradictory.derivations.nodes[atomic.parent.0 as usize],
            DerivationNode::SourceBound { event, .. }
                if contradictory.derivations.events[event.0 as usize].kind == FlowEventKind::S11
        ));
        assert_eq!(atomic.proof_point, CountedProofPoint::BodyEntry);
    }
    assert_root_contains(
        &contradictory,
        contradictory.counted_derivations[0]
            .lower_capture_eq_endpoint
            .forward
            .parent,
        |node| matches!(node, DerivationNode::JoinContradiction { parents, .. } if parents.len() == 2),
        "the counted statement follows the binary all-contradictory join",
    );

    let joined = entailment(source, "joined");
    validate_derivations(&joined);
    assert_eq!(joined.counted_derivations.len(), 1);
    assert_eq!(joined.obligations.len(), 1);
    assert!(joined.obligations[0].discharged);
    assert_root_contains(
        &joined,
        obligation_root(&joined, 0),
        |node| {
            let DerivationNode::JoinBound { parents, .. } = node else {
                return false;
            };
            let contradictory = parents
                .iter()
                .filter(|parent| {
                    matches!(
                        joined.derivations.nodes[parent.parent.0 as usize],
                        DerivationNode::L0Contradiction { .. }
                            | DerivationNode::GoalContradiction { .. }
                            | DerivationNode::JoinContradiction { .. }
                            | DerivationNode::MaterializedContradiction { .. }
                    )
                })
                .count();
            parents.len() == 2 && contradictory == 1
        },
        "the counted snapshot consequence's join with its contradictory-neutral predecessor",
    );
}

#[test]
fn counted_root_mutations_fail_the_structural_checker() {
    let source = br#"fn probe(upper: own u64) -> own unit pure {
  for @items i in 0_u64..upper {
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "probe");
    validate_derivations(&summary);
    assert_eq!(summary.counted_derivations.len(), 1);

    assert_derivation_mutation_rejected(&summary, |mutant| {
        let index = mutant
            .derivations
            .roots
            .iter()
            .position(|root| matches!(root.kind, DerivationRootKind::CountedS11 { .. }))
            .expect("counted root");
        mutant.derivations.roots.remove(index);
    });
    assert_derivation_mutation_rejected(&summary, |mutant| {
        let root = *mutant
            .derivations
            .roots
            .iter()
            .find(|root| matches!(root.kind, DerivationRootKind::CountedS11 { .. }))
            .expect("counted root");
        mutant.derivations.roots.push(root);
    });
    assert_derivation_mutation_rejected(&summary, |mutant| {
        mutant.counted_derivations[0]
            .counted_node_path
            .components
            .push(999);
    });
    assert_derivation_mutation_rejected(&summary, |mutant| {
        let relation = &mut mutant.counted_derivations[0]
            .lower_capture_eq_endpoint
            .forward
            .relation;
        let Relation::Bound { bound, .. } = relation else {
            unreachable!();
        };
        *bound = 1;
    });
    assert_derivation_mutation_rejected(&summary, |mutant| {
        mutant.counted_derivations[0]
            .upper_capture_eq_endpoint
            .reverse
            .parent = DerivationId(u32::MAX);
    });
    assert_derivation_mutation_rejected(&summary, |mutant| {
        let parent = mutant.counted_derivations[0]
            .lower_capture_eq_endpoint
            .forward
            .parent;
        let event = match mutant.derivations.nodes[parent.0 as usize] {
            DerivationNode::MaterializedBound { event, .. }
            | DerivationNode::MaterializedContradiction { event, .. } => event,
            ref node => panic!("snapshot root has the wrong node: {node:?}"),
        };
        mutant.derivations.events[event.0 as usize].kind = FlowEventKind::Join;
    });
    assert_derivation_mutation_rejected(&summary, |mutant| {
        let killed_preheader_parent = mutant.counted_derivations[0]
            .binder_eq_lower_capture
            .reverse
            .parent;
        mutant.counted_derivations[0]
            .lower_capture_le_binder
            .atomic
            .parent = killed_preheader_parent;
    });
}

#[test]
fn generic_counted_roots_are_deterministic_across_twenty_analyses() {
    let source = br#"fn ranges<const n: u64>(values: own array<u8, n>) -> own unit pure {
  let upper = len(values);
  for @first i in 0_u64..upper {
  }
  for @second j in 1_u64..upper {
  }
  return unit;
}

fn main() -> own unit pure {
  let small = array_new<u8, 2>(0_u8);
  ranges<2>(values: move small);
  let large = array_new<u8, 5>(0_u8);
  ranges<5>(values: move large);
  return unit;
}
"#;
    let normalized_instances = || {
        let instances = entailments(source, "ranges");
        assert_eq!(instances.len(), 2);
        let mut normalized = Vec::new();
        for summary in instances {
            validate_derivations(&summary);
            assert_eq!(summary.counted_derivations.len(), 2);
            assert!(
                summary
                    .inventory
                    .terms
                    .iter()
                    .all(|term| !matches!(term, TermKind::ConstParameter(_))),
                "concrete instances retain no symbolic const term"
            );
            let mut lengths: Vec<_> = summary
                .inventory
                .length_bounds
                .iter()
                .filter_map(|bound| match bound {
                    Some(LengthBound::Constant(value)) => Some(*value),
                    Some(LengthBound::Equal(_)) | None => None,
                })
                .collect();
            lengths.sort_unstable();
            lengths.dedup();
            assert_eq!(lengths.len(), 1);
            normalized.push((lengths[0], normalized_derivation_dump(&summary)));
        }
        normalized.sort_by_key(|(length, _)| *length);
        normalized
    };
    let expected = normalized_instances();
    assert_eq!(
        expected
            .iter()
            .map(|(length, _)| *length)
            .collect::<Vec<_>>(),
        vec![2, 5]
    );
    for run in 1..20 {
        assert_eq!(
            normalized_instances(),
            expected,
            "normalized concrete counted S11 ledgers changed on run {run}"
        );
    }
}

#[test]
fn a_break_free_zero_trip_counted_continuation_is_reachable_not_contradictory() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  for @empty i in 4_u64..4_u64 {
    let ignored = i;
  }
  return values[9_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the real false-header edge prevents the ordinary break-free-loop contradictory join"
    );
}

#[test]
fn a_counted_body_fact_does_not_escape_through_the_zero_trip_edge() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure {
  for @maybe n in 0_u64..1_u64 {
    if ilt(i, 4_u64) {
      let ignored = n;
    } else {
      return 0_i32;
    }
  }
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "the structural false edge contributes the pre-body state to the continuation join"
    );
}

#[test]
fn a_nested_counted_loop_kill_can_reach_an_outer_ordinary_loop_head() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64, leave: own Bool) -> own i32 pure {
  if ilt(i, 4_u64) {
  } else {
    return 0_i32;
  }
  loop @outer {
    for @inner n in 0_u64..1_u64 {
      set i = i +wrap 1_u64;
      let ignored = n;
    }
    let at_head = values[i];
    if leave {
      break @outer;
    }
  }
  return 0_i32;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "nested counted exhaustion reaches the outer continuation and then its backedge"
    );
}

// ---------------------------------------------------------------------
// [ENT-6] obligations and residual rendering
// ---------------------------------------------------------------------

#[test]
fn a_struct_field_base_renders_its_canonical_place_in_the_residual() {
    let source = br#"const count: u64 = 4_u64;

struct Holder {
  data: array<u8, count>;
}

fn read(h: own Holder, i: own u64) -> own u8 pure {
  return h.data[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 1);
    assert!(!outcomes[0].discharged);
    assert_eq!(outcomes[0].residual.as_deref(), Some("i < len(h.data)"));
}

#[test]
fn a_nested_index_offset_is_no_term_and_renders_its_canonical_bytes() {
    let source = br#"const count: u64 = 4_u64;

fn read(lens: own array<u8, count>, order: own array<u64, count>, j: own u64) -> own u8 pure {
  if ilt(j, 4_u64) {
    return lens[order[j]];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 2, "inner offset first, then the outer site");
    assert!(
        outcomes[0].discharged,
        "the inner index over order discharges"
    );
    assert!(
        !outcomes[1].discharged,
        "an index-bearing offset is no term [ENT-2], so the outer obligation is underivable"
    );
    assert_eq!(
        outcomes[1].residual.as_deref(),
        Some("order[j] < len(lens)")
    );
}

#[test]
fn a_buffer_or_slice_offset_renders_the_same_subscript_spelling() {
    let source = br#"const count: u64 = 4_u64;

fn from_buffer(values: own array<u8, count>) -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u64);
  return values[b[0_u64]];
}

fn from_slice['r](values: own array<u8, count>, order: own slice<'r, u64>) -> own u8 reads('r) {
  return values[order[0_u64]];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let buffer = obligations(source, "from_buffer");
    assert_eq!(buffer.len(), 2, "inner offset first, then the outer site");
    assert!(
        buffer[0].discharged,
        "the S6 allocation equality proves the inner offset"
    );
    assert_eq!(
        buffer[1].residual.as_deref(),
        Some("b[0_u64] < len(values)")
    );

    let slice = obligations(source, "from_slice");
    assert_eq!(slice.len(), 2, "inner offset first, then the outer site");
    assert_eq!(slice[0].residual.as_deref(), Some("0_u64 < len(order)"));
    assert_eq!(
        slice[1].residual.as_deref(),
        Some("order[0_u64] < len(values)")
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S6 length facts
// ---------------------------------------------------------------------

#[test]
fn an_allocation_length_equality_proves_a_constant_offset_and_a_runtime_length_does_not() {
    let source = br#"fn sized() -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u8);
  return b[3_u64];
}

fn runtime(n: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  return b[3_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let sized = entailment(source, "sized");
    validate_derivations(&sized);
    assert_eq!(
        sized
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "len(b) = 4 makes 3 < len(b) derivable [ENT-3] S6"
    );
    assert_root_has_event_kind(&sized, obligation_root(&sized, 0), FlowEventKind::S6);
    let runtime = obligations(source, "runtime");
    assert!(
        !runtime[0].discharged,
        "len(b) = n bounds nothing without a fact about n"
    );
    assert_eq!(runtime[0].residual.as_deref(), Some("3_u64 < len(b)"));
}

#[test]
fn an_allocation_length_binding_carries_the_length_into_a_branch() {
    // `let m = len<T>(P)` establishes m = len(P), so a branch over m is a
    // branch over the length itself [ENT-3] S6.
    let source = br#"fn read(n: own u64, i: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  let m = len(b);
  if ilt(i, m) {
    return b[i];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "read"), vec![true]);
}

#[test]
fn a_slice_of_carries_its_source_length() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<u8, count>) -> own u8 pure {
  region 'view {
    let window = slice_of(&'view values);
    return window[3_u64];
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "len(window) = len(values) = 4 [ENT-3] S6"
    );
}

#[test]
fn an_element_write_keeps_the_allocation_equality_that_a_write_to_its_length_kills() {
    // [ENT-5]: a buffer's length is fixed at allocation, so an element write
    // never kills its length fact; a write to the term the equality is held
    // against does. A buffer place is affine [STOR-1], so writing the root
    // binding itself is not a source shape the engine can be shown.
    let source = br#"fn kept(n: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  if ilt(3_u64, n) {
    set b[0_u64] = 1_u8;
    return b[3_u64];
  } else {
    return 0_u8;
  }
}

fn killed(n: own u64) -> own u8 allocates(heap), traps {
  let b = buffer_new(n, 0_u8);
  if ilt(3_u64, n) {
    set n = 0_u64;
    return b[3_u64];
  } else {
    return 0_u8;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let kept = entailment(source, "kept");
    validate_derivations(&kept);
    assert!(
        kept.obligations
            .last()
            .is_some_and(|outcome| outcome.discharged),
        "an element write leaves len(b) = n alive"
    );
    let kept_root = obligation_root(&kept, kept.obligations.len() - 1);
    assert_root_has_event_kind(&kept, kept_root, FlowEventKind::S6);

    let killed = entailment(source, "killed");
    validate_derivations(&killed);
    assert!(
        !killed
            .obligations
            .last()
            .is_some_and(|outcome| outcome.discharged),
        "writing n kills the allocation equality held against it"
    );
    assert!(killed.obligations.last().unwrap().derivation.is_none());
}

#[test]
fn consuming_the_buffer_kills_a_length_binding_that_survives_otherwise() {
    // The support of len(b) is b's root binding, so a consuming use kills
    // every fact holding it, including the equality a length binding carries
    // away from it [ENT-5](c).
    let source = br#"const wide: u64 = 8_u64;

fn eat(b: own buffer<u8>) -> own unit pure {
  return unit;
}

fn kept(other: own array<u8, wide>) -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u8);
  let m = len(b);
  let sample = other[m];
  eat(b: move b);
  return sample;
}

fn killed(other: own array<u8, wide>) -> own u8 allocates(heap), traps {
  let b = buffer_new(4_u64, 0_u8);
  let m = len(b);
  eat(b: move b);
  let sample = other[m];
  return sample;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let kept = entailment(source, "kept");
    validate_derivations(&kept);
    assert_eq!(
        kept.obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "m = len(b) = 4 < 8 while b is live"
    );
    assert_root_has_event_kind(&kept, obligation_root(&kept, 0), FlowEventKind::S6);

    let killed = entailment(source, "killed");
    validate_derivations(&killed);
    assert_eq!(
        killed
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![false],
        "the consuming use kills m's tie to the allocation length"
    );
    assert!(killed.obligations[0].derivation.is_none());
}

#[test]
fn set_targets_carry_the_same_obligation_in_target_position() {
    let source = br#"const count: u64 = 4_u64;

fn write(values: own array<u16, count>, i: own u64) -> own u16 pure {
  if ilt(i, 4_u64) {
    set values[i] = 9_u16;
    return 1_u16;
  } else {
    set values[i] = 9_u16;
    return 0_u16;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "write");
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes[0].discharged,
        "target-position discharge is identical"
    );
    assert!(!outcomes[1].discharged);
    assert_eq!(outcomes[1].residual.as_deref(), Some("i < len(values)"));
}

// ---------------------------------------------------------------------
// [ENT-3] S2 check facts
// ---------------------------------------------------------------------

#[test]
fn a_passed_check_establishes_its_comparison_on_the_continuation() {
    let source = br#"const count: u64 = 4_u64;

fn direct(values: own array<i32, count>, i: own u64) -> own i32 traps {
  check ilt(i, 4_u64) else trap "i must be in range";
  return values[i];
}

fn through_origin(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let ok = ilt(i, 4_u64);
  check ok else trap "i must be in range";
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "direct"), vec![true]);
    assert_eq!(
        discharge_flags(source, "through_origin"),
        vec![true],
        "the check reads comparison-origin shape (b) exactly as a match does"
    );
}

#[test]
fn a_check_without_comparison_origin_establishes_no_l0_fact() {
    // `band` has no comparison projection [ENT-3], so its passed check
    // establishes the exact whole goal but no child comparison relation.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  let low = ilt(i, 4_u64);
  let high = ige(i, 0_u64);
  check band(low, high) else trap "i must be in range";
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "read"), vec![false]);
}

// ---------------------------------------------------------------------
// [ENT-3] S5 copy and conversion equalities
// ---------------------------------------------------------------------

#[test]
fn a_literal_a_copy_and_a_total_conversion_carry_the_value_forward() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  let k = 2_u64;
  let j = k;
  let narrow = 3_u16;
  let widened = cvt<u16, u64>(narrow);
  let first = values[j];
  let second = values[widened];
  return first +wrap second;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true, true],
        "j = k = 2 and widened = narrow = 3, both below len(values)"
    );
    for ordinal in 0..2 {
        assert_root_has_event_kind(
            &summary,
            obligation_root(&summary, ordinal),
            FlowEventKind::S5,
        );
    }
}

#[test]
fn a_narrowing_conversion_carries_no_equality_into_its_ok_arm() {
    // [OP-6] narrowing is not a total pair, so [ENT-3] S5 does not apply and
    // the `Ok` binder inherits only its own type range.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, n: own u64) -> own i32 pure {
  if ilt(n, 4_u64) {
    match cvt<u64, u8>(n) {
      Ok(value: small) => {
        let widened = cvt<u8, u64>(small);
        return values[widened];
      }
      Err(error: narrowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "small keeps no tie to n, so widened is bounded only by u8's range"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S7 constant-offset arithmetic
// ---------------------------------------------------------------------

#[test]
fn a_trapping_offset_establishes_its_equality_unconditionally() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ilt(i, 3_u64) {
    let next = i + 1_u64;
    return values[next];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "the executed contract check is the proof [ENT-3] S7"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S7);
}

#[test]
fn a_wrapping_offset_establishes_only_where_the_range_is_already_proved() {
    // The wrap has no runtime check, so the equality holds only where the
    // closed state already proves the unwrapped result stays in range.
    let source = br#"const count: u64 = 4_u64;

fn guarded(values: own array<i32, count>, p: own u64) -> own i32 pure {
  if ilt(p, 4_u64) {
    if ige(p, 1_u64) {
      let s = p -wrap 1_u64;
      return values[s];
    } else {
      return 0_i32;
    }
  } else {
    return 0_i32;
  }
}

fn unguarded(values: own array<i32, count>, p: own u64) -> own i32 pure {
  if ilt(p, 4_u64) {
    let s = p -wrap 1_u64;
    return values[s];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "guarded"),
        vec![true],
        "p >= 1 proves p - 1 does not underflow, so s = p - 1 <= 2"
    );
    assert_eq!(
        discharge_flags(source, "unguarded"),
        vec![false],
        "p may be 0, where the wrap reaches u64::MAX"
    );
}

#[test]
fn a_checked_offset_establishes_in_the_ok_arm_only_and_dies_with_its_base() {
    let source = br#"const count: u64 = 4_u64;

fn direct(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 3_u64) {
    match i +checked 1_u64 {
      Ok(value: next) => {
        return values[next];
      }
      Err(error: overflowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn through_binding(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 3_u64) {
    let outcome = i +checked 1_u64;
    match outcome {
      Ok(value: next) => {
        return values[next];
      }
      Err(error: overflowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn killed(values: own array<i32, count>, i: own u64) -> own i32 pure {
  if ilt(i, 3_u64) {
    let outcome = i +checked 1_u64;
    set i = 9_u64;
    match outcome {
      Ok(value: next) => {
        return values[next];
      }
      Err(error: overflowed) => {
        return 0_i32;
      }
    }
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(discharge_flags(source, "direct"), vec![true]);
    assert_eq!(
        discharge_flags(source, "through_binding"),
        vec![true],
        "a bare IDENT naming the outcome carries the same fact"
    );
    assert_eq!(
        discharge_flags(source, "killed"),
        vec![false],
        "writing the base between the initializer and the match ends the origin"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S9 const-array element ranges
// ---------------------------------------------------------------------

#[test]
fn a_const_array_element_carries_its_declared_value_range() {
    let source = br#"const count: u64 = 4_u64;

const inside: array<u64, count> =[0_u64, 1_u64, 3_u64, 2_u64];

const outside: array<u64, count> =[0_u64, 1_u64, 4_u64, 2_u64];

fn low(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let bound = inside[i];
  return values[bound];
}

fn high(values: own array<i32, count>, i: own u64) -> own i32 pure {
  let bound = outside[i];
  return values[bound];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let low = entailment(source, "low");
    validate_derivations(&low);
    assert_eq!(
        low.obligations.len(),
        2,
        "the element read carries its own obligation"
    );
    assert!(
        !low.obligations[0].discharged,
        "the index into the const table is judged separately and unaffected"
    );
    assert!(
        low.obligations[1].discharged,
        "every declared element is at most 3 < len(values)"
    );
    assert_root_has_event_kind(&low, obligation_root(&low, 1), FlowEventKind::S9);
    let high = obligations(source, "high");
    assert!(
        !high[1].discharged,
        "a declared element of 4 reaches len(values)"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S4 requires facts
// ---------------------------------------------------------------------

#[test]
fn a_requires_check_establishes_its_substituted_relation_at_body_entry() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure requires {
  let ok = ilt(i, 4_u64);
  check ok else trap "i must be in range";
} {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    assert_eq!(
        summary
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "the requirement relation is available at body entry"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S4);
}

#[test]
fn a_requires_chain_substitutes_repeatedly_and_reads_a_length_call() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure requires {
  let n = len(values);
  let ok = ilt(i, n);
  check ok else trap "i must be in range";
} {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![true],
        "ok substitutes to the comparison, then n to the length term itself"
    );
}

#[test]
fn every_occurrence_of_a_requires_local_substitutes() {
    // Both operands name the same clause local. Expanding only one would
    // leave a non-term operand and establish nothing; expanding both derives
    // len(values) < len(values), a contradictory entry state [ENT-4].
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure requires {
  let n = len(values);
  let ok = ilt(n, n);
  check ok else trap "unsatisfiable by construction";
} {
  return values[9_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = obligations(source, "read");
    assert_eq!(outcomes.len(), 1);
    assert!(
        outcomes[0].contradictory,
        "both occurrences expanded to the same length term"
    );
    assert!(outcomes[0].discharged);
}

#[test]
fn a_noncomparison_s4_goal_establishes_no_child_l0_fact() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 pure requires {
  let low = ilt(i, 4_u64);
  let high = ige(i, 0_u64);
  let ok = band(low, high);
  check ok else trap "i must be in range";
} {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_eq!(
        discharge_flags(source, "read"),
        vec![false],
        "S4 establishes the atomic band, not either comparison child"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S10 boundary count facts
// ---------------------------------------------------------------------

#[test]
fn a_transfer_count_is_bounded_by_its_bounding_actual_and_not_beyond_it() {
    // The bound is `w <= k` against the actual bound to the operation's own
    // bounding parameter, so a count equal to the length proves nothing.
    let source = br#"const count: u64 = 4_u64;

fn under['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, table: own array<u8, count>) -> own unit reads('o 's), writes('o), external, blocks, traps {
  region 'attempt {
    match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, offset: 0_u64, count: 3_u64) {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

fn exact['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, table: own array<u8, count>) -> own unit reads('o 's), writes('o), external, blocks, traps {
  region 'attempt {
    match write_once<'attempt, 's>(output: &uniq 'attempt deref(output), source: source, offset: 0_u64, count: 4_u64) {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch = buffer_new(1_u64, 0_u8);
  let table = array_new<u8, count>(0_u8);
  region 'publication {
    under<'publication, 'publication>(output: &uniq 'publication out, source: &'publication batch, table: move table);
  }
  return exit_status(code: 0_u8);
}
"#;
    let under = entailment(source, "under");
    validate_derivations(&under);
    assert_eq!(
        under
            .obligations
            .iter()
            .map(|outcome| outcome.discharged)
            .collect::<Vec<_>>(),
        vec![true],
        "written <= 3 < len(table)"
    );
    assert_root_has_event_kind(&under, obligation_root(&under, 0), FlowEventKind::S10);
    assert_eq!(
        discharge_flags(source, "exact"),
        vec![false],
        "written <= 4 admits written = len(table)"
    );
}

#[test]
fn a_transfer_count_bound_enters_the_observing_arm_only() {
    // `Ok(value: w)` observes the count bound; the error arm's own u64 payload
    // is an unrelated required size and gains nothing [ENT-3] S10.
    let source = br#"const count: u64 = 4_u64;

command fn main(command.args as args: own Args) -> own ExitStatus allocates(heap), traps {
  let table = array_new<u8, count>(0_u8);
  let sink = buffer_new(8_u64, 0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
      Ok(value: text) => {
        region 'v {
          region 'd {
            match host_copy_bytes<'v, 'd>(value: &'v text, destination: &uniq 'd sink, offset: 0_u64, capacity: 3_u64) {
              Ok(value: copied) => {
                let good = table[copied];
              }
              Err(error: problem) => {
                match problem {
                  CopyTooSmall(required: needed) => {
                    let bad = table[needed];
                  }
                }
              }
            }
          }
        }
      }
      Err(error: missing) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(
        discharge_flags(source, "main"),
        vec![true, false],
        "only the success arm's binder carries the capacity bound"
    );
}

#[test]
fn a_host_copy_utf8_success_count_is_bounded_by_capacity() {
    // The UTF-8 copy producer carries the same S10 success-count bound as the
    // byte-preserving copy producer: copied <= 3 < len(table).
    let source = br#"const count: u64 = 4_u64;

command fn main(command.args as args: own Args) -> own ExitStatus allocates(heap), traps {
  let table = array_new<u8, count>(0_u8);
  let sink = buffer_new(8_u64, 0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 0_u64) {
      Ok(value: text) => {
        region 'v {
          region 'd {
            match host_copy_utf8<'v, 'd>(value: &'v text, destination: &uniq 'd sink, offset: 0_u64, capacity: 3_u64) {
              Ok(value: copied) => {
                let good = table[copied];
              }
              Err(error: problem) => {
              }
            }
          }
        }
      }
      Err(error: missing) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(
        discharge_flags(source, "main"),
        vec![true],
        "host_copy_utf8's Ok payload is at most the capacity actual"
    );
}

#[test]
fn a_let_bound_transfer_outcome_carries_the_same_count_bound() {
    // The bare IDENT form of [ENT-3] S10, under the same no-kill, no-`set`
    // path discipline as S7's checked-arithmetic origin.
    let source = br#"const count: u64 = 4_u64;

fn deferred['s](output: own Output, source: &'s buffer<u8>, table: own array<u8, count>, limit: own u64) -> own unit reads('s), external, blocks, traps {
  region 'attempt {
    let outcome = write_once<'attempt, 's>(output: &uniq 'attempt output, source: source, offset: 0_u64, count: 3_u64);
    match outcome {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

fn killed['s](output: own Output, source: &'s buffer<u8>, table: own array<u8, count>, limit: own u64) -> own unit reads('s), external, blocks, traps {
  region 'attempt {
    let outcome = write_once<'attempt, 's>(output: &uniq 'attempt output, source: source, offset: 0_u64, count: limit);
    set limit = 9_u64;
    match outcome {
      Ok(value: written) => {
        let sample = table[written];
      }
      Err(error: problem) => {
      }
    }
  }
  return unit;
}

command fn main(command.stdout as out: own Output) -> own ExitStatus allocates(heap), external, blocks, traps {
  let batch = buffer_new(1_u64, 0_u8);
  let table = array_new<u8, count>(0_u8);
  region 'publication {
    deferred<'publication>(output: move out, source: &'publication batch, table: move table, limit: 3_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(discharge_flags(source, "deferred"), vec![true]);
    assert_eq!(
        discharge_flags(source, "killed"),
        vec![false],
        "writing the bounding actual before the match ends the origin"
    );
}

#[test]
fn a_read_once_count_is_observed_on_its_own_outcome_variant() {
    // `read_once` reports through `ReadBytes(count: w)` rather than a
    // `Result`, so the observing arm is named per operation [ENT-3] S10.
    let source = br#"const count: u64 = 4_u64;

command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead) -> own ExitStatus allocates(heap), external, blocks, traps {
  let table = array_new<u8, count>(0_u8);
  region 'a {
    match arg_get<'a>(args: &'a args, position: 1_u64) {
      Ok(value: text) => {
        match relative_path(value: move text) {
          Ok(value: path) => {
            region 'c {
              region 'p {
                match open_read<'c, 'p>(root: &'c cwd, path: &'p path) {
                  Ok(value: file) => {
                    let bytes = buffer_new(64_u64, 0_u8);
                    region 'f {
                      region 'd {
                        match read_once<'f, 'd>(file: &uniq 'f file, destination: &uniq 'd bytes, offset: 0_u64, capacity: 3_u64) {
                          ReadBytes(count: n) => {
                            let sample = table[n];
                          }
                          ReadEnd() => {
                          }
                          ReadFailed(error: problem) => {
                          }
                        }
                      }
                    }
                  }
                  Err(error: unopened) => {
                  }
                }
              }
            }
          }
          Err(error: unresolved) => {
          }
        }
      }
      Err(error: missing) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(
        discharge_flags(source, "main"),
        vec![true],
        "the ReadBytes count is at most the capacity actual"
    );
}

// ---------------------------------------------------------------------
// [ENT-3] S3 claim facts
// ---------------------------------------------------------------------

#[test]
fn a_passed_claim_establishes_its_fact_on_the_continuation() {
    let source = br#"fn read(values: own buffer<i32>, i: own u64) -> own i32 traps {
  let n = len(values);
  let inside = ilt(i, n);
  claim in_range: inside because "the caller walks 0..len";
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "read");
    validate_derivations(&summary);
    let outcomes = &summary.obligations;
    assert_eq!(outcomes.len(), 1);
    assert!(
        outcomes[0].discharged,
        "S3: the passed claim predicate discharges the following subscript"
    );
    assert_root_has_event_kind(&summary, obligation_root(&summary, 0), FlowEventKind::S3);
    let claim_outcomes = claims(source, "read");
    assert_eq!(claim_outcomes.len(), 1);
    assert_eq!(claim_outcomes[0].name, "in_range");
    assert_eq!(claim_outcomes[0].disposition, ClaimDisposition::Retained);
}

#[test]
fn a_claim_without_comparison_origin_is_retained_and_never_judged() {
    let source = br#"fn main() -> own unit traps {
  let flag = True();
  claim held: flag because "constructed";
  return unit;
}
"#;
    let claim_outcomes = claims(source, "main");
    assert_eq!(claim_outcomes.len(), 1);
    assert_eq!(claim_outcomes[0].disposition, ClaimDisposition::Retained);
}

// ---------------------------------------------------------------------
// [CLM-2] redundancy and refutation
// ---------------------------------------------------------------------

#[test]
fn a_derivable_claim_is_redundant_and_reports_the_advisory_without_rejecting() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ilt(i, 4_u64) {
    claim proven: ilt(i, 4_u64) because "already branched";
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("a redundant claim must not reject, got {outcome:?}");
        };
        assert_eq!(program.data.claim_advisories.len(), 1);
        assert_eq!(program.data.claim_advisories[0].function, "read");
        assert_eq!(program.data.claim_advisories[0].name, "proven");
    });
}

#[test]
fn a_refuted_claim_is_a_clm2_rejection_with_predicate_and_negation() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ige(i, 4_u64) {
    claim in_range: ilt(i, 4_u64) because "refuted by the branch";
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("a refuted claim must reject, got {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Clm2);
        let SemanticIssueKind::RefutedClaim(detail) = issue.kind() else {
            panic!("expected the refutation payload, got {:?}", issue.kind());
        };
        assert_eq!(detail.name, "in_range");
    });
}

#[test]
fn a_contradictory_state_never_refutes_a_claim() {
    // [ENT-4]: after a loop with no break the continuation state is
    // contradictory; every relation is derivable there, so the claim is
    // redundant, never rejected. The loop must terminate for the checker's
    // reachability rules, so it returns from inside.
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>, i: own u64) -> own i32 traps {
  if ilt(i, 0_u64) {
    claim absurd: ilt(i, 4_u64) because "under a false branch";
    return values[i];
  } else {
    return 0_i32;
  }
}

fn main() -> own unit pure {
  return unit;
}
"#;
    // i < 0 is unsatisfiable for u64: the True arm's state is contradictory,
    // so the claim is redundant there and the subscript discharges.
    let claim_outcomes = claims(source, "read");
    assert_eq!(claim_outcomes.len(), 1);
    assert_eq!(claim_outcomes[0].disposition, ClaimDisposition::Redundant);
    assert_eq!(discharge_flags(source, "read"), vec![true]);
}

// ---------------------------------------------------------------------
// [OP-4] discharge-or-reject through the ordinary acceptance path
// ---------------------------------------------------------------------

#[test]
fn an_undischarged_subscript_is_an_op4_rejection_with_the_exact_residual() {
    let source = br#"fn read(values: own buffer<i32>, i: own u64) -> own i32 pure {
  return values[i];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    assert_rule(
        source,
        SemanticRule::Op4,
        SemanticIssueKind::UndischargedBoundsObligation {
            residual: "i < len(values)".to_owned(),
            mechanical_fix: "add a dominating `claim` of the residual or a dominating branch establishing it",
        },
    );
}

#[test]
fn a_discharged_program_accepts_and_retains_its_derivations() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  return values[2_u64];
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("a discharged subscript must accept, got {outcome:?}");
        };
        let function = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("read must exist");
        assert_eq!(function.entailment.obligations.len(), 1);
        assert!(function.entailment.obligations[0].discharged);
        validate_derivations(&function.entailment);
        for rewalks in [
            &program.data.provenance.unasserted,
            &program.data.provenance.s4_blinded,
        ] {
            let obligations: Vec<_> = rewalks
                .iter()
                .flat_map(|rewalk| &rewalk.obligations)
                .collect();
            assert_eq!(obligations.len(), 1);
            assert!(
                obligations
                    .iter()
                    .all(|obligation| !obligation.node_path.components().is_empty()
                        && obligation.discharged == obligation.residual.is_none()),
                "counterfactual rewalks retain only provenance's exact path and disposition"
            );
        }
    });
}

#[test]
fn counted_counterfactual_rewalks_publish_only_exact_outcome_shape() {
    let source = br#"const count: u64 = 4_u64;

fn read(values: own array<i32, count>) -> own i32 pure {
  let total = 0_i32;
  for @items i in 0_u64..4_u64 {
    let value = values[i];
    set total = total +wrap value;
  }
  return total;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("the counted counterfactual fixture must accept: {outcome:?}");
        };
        let function = program
            .data
            .functions
            .iter()
            .find(|function| function.name == "read")
            .expect("read must exist");
        validate_derivations(&function.entailment);
        assert_eq!(function.entailment.counted_derivations.len(), 1);
        assert_eq!(
            function
                .entailment
                .derivations
                .roots
                .iter()
                .filter(|root| matches!(root.kind, DerivationRootKind::CountedS11 { .. }))
                .count(),
            8
        );
        let expected_path = &function.entailment.obligations[0].node_path;
        let function_index = function.id.0 as usize;
        for (name, rewalks) in [
            ("unasserted", &program.data.provenance.unasserted),
            ("S4-blinded", &program.data.provenance.s4_blinded),
        ] {
            let rewalk = &rewalks[function_index];
            assert_eq!(rewalk.obligations.len(), 1, "{name}");
            assert_eq!(&rewalk.obligations[0].node_path, expected_path, "{name}");
            assert!(rewalk.obligations[0].discharged, "{name}");
            assert_eq!(rewalk.obligations[0].residual, None, "{name}");
            assert!(rewalk.call_goals.is_empty(), "{name}");
            let dump = format!("{rewalk:?}");
            assert!(!dump.contains("DerivationId"), "{name}: {dump}");
            assert!(!dump.contains("TermId"), "{name}: {dump}");
            assert!(!dump.contains("CountedDerivation"), "{name}: {dump}");
        }
    });
}

#[test]
fn counted_sha256_discharges_all_nine_indices_without_claims() {
    let source = include_bytes!("../../../../tests/programs/sha256_abc.wf");
    let summary = entailment(source, "sha256_abc_word_zero");
    validate_derivations(&summary);
    assert_eq!(summary.obligations.len(), 9);
    assert!(summary.obligations.iter().all(|outcome| outcome.discharged));
    assert!(summary.claims.is_empty());
    assert_eq!(summary.counted_derivations.len(), 3);
    assert_eq!(
        summary.counted_derivations.len() * 5,
        15,
        "the three SHA-256 ranges retain all five semantic S11 relations"
    );
    assert_eq!(
        summary
            .derivations
            .roots
            .iter()
            .filter(|root| matches!(root.kind, DerivationRootKind::CountedS11 { .. }))
            .count(),
        24,
        "the three SHA-256 ranges retain all eight directed atomic roots"
    );
    assert_eq!(
        summary
            .derivations
            .roots
            .iter()
            .filter(|root| matches!(root.kind, DerivationRootKind::BoundsObligation(_)))
            .count(),
        9,
        "all nine existing accepted bounds obligations retain exact roots"
    );
}

#[test]
fn frozen_real_sources_retain_complete_entailment_roots_without_counted_false_positives() {
    let bundles: [&[SourceInput<'_>]; 3] = [
        &[SourceInput::new(
            "utf8parse.wf",
            include_bytes!("../../../../tests/programs/utf8parse.wf"),
        )],
        &[
            SourceInput::new(
                "raw_deflate.wf",
                include_bytes!("../../../../tests/programs/raw_deflate.wf"),
            ),
            SourceInput::new(
                "raw_deflate_dynamic.wf",
                include_bytes!("../../../../tests/programs/raw_deflate_dynamic.wf"),
            ),
            SourceInput::new(
                "raw_deflate_dynamic_decode.wf",
                include_bytes!("../../../../tests/programs/raw_deflate_dynamic_decode.wf"),
            ),
            SourceInput::new(
                "raw_deflate_boundary.wf",
                include_bytes!("../../../../tests/programs/raw_deflate_boundary.wf"),
            ),
        ],
        &[SourceInput::new(
            "wfgrep.wf",
            include_bytes!("../../../../tests/programs/wfgrep.wf"),
        )],
    ];
    for inputs in bundles {
        with_semantics_inputs(inputs, |outcome| {
            let SemanticOutcome::Complete(program) = outcome else {
                panic!("frozen real source bundle must remain accepted: {outcome:?}");
            };
            for function in &program.data.functions {
                validate_derivations(&function.entailment);
                assert!(
                    function.entailment.counted_derivations.is_empty(),
                    "ordinary loops in {} must not acquire counted induction roots",
                    function.name
                );
            }
        });
    }
}

#[test]
fn counted_range_reads_a_dereferenced_projected_endpoint_as_an_s11_term() {
    let source = br#"struct Holder {
  value: box<u64>;
}

fn probe(holder: own Holder) -> own unit traps {
  for @items i in deref(holder.value)..1_u64 {
    claim impossible: ine(i, 0_u64) because "the true edge fixes i to zero";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "probe");
    validate_derivations(&summary);
    assert_eq!(summary.counted_derivations.len(), 1);
    let Relation::Equal { right, .. } = summary.counted_derivations[0]
        .lower_capture_eq_endpoint
        .relation
    else {
        panic!("the lower capture identity must be an equality");
    };
    let TermKind::ProjectedPlace(endpoint, IntegerType::U64) = retained_term(&summary, right)
    else {
        panic!("deref(holder.value) must retain its exact projected-place endpoint identity");
    };
    assert_eq!(
        endpoint.projections,
        vec![PlaceProjection::Field(0), PlaceProjection::Deref]
    );
    let outcomes = &summary.claims;
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn counted_range_kills_a_borrowed_projected_endpoint_after_a_write() {
    let source = br#"struct Limit {
  upper: u64;
}

fn probe(limit: own Limit) -> own unit traps {
  region 'r {
    let holder = &uniq 'r limit;
    for @items i in 0_u64..deref(holder).upper {
      set deref(holder).upper = 0_u64;
      claim safe: ige(i, deref(holder).upper) because "the reread is not the captured endpoint";
    }
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "probe");
    validate_derivations(&summary);
    assert_eq!(summary.counted_derivations.len(), 1);
    let outcomes = &summary.claims;
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, ClaimDisposition::Retained);
}

#[test]
fn counted_range_preserves_multiple_deref_projections_in_one_endpoint_term() {
    let source = br#"fn probe(holder: own box<box<u64>>) -> own unit traps {
  for @items i in deref(deref(holder))..1_u64 {
    claim impossible: ine(i, 0_u64) because "the true edge fixes i to zero";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "probe");
    validate_derivations(&summary);
    assert_eq!(summary.counted_derivations.len(), 1);
    let outcomes = &summary.claims;
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn counted_range_restores_a_borrow_holder_deref_before_nested_box_derefs() {
    let source = br#"fn probe['r](holder: &'r box<box<u64>>) -> own unit reads('r), traps {
  for @items i in deref(deref(deref(holder)))..1_u64 {
    claim impossible: igt(deref(deref(deref(holder))), i) because "the header proves the opposite";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    let ClaimDisposition::Refuted {
        predicate,
        negation,
    } = &outcomes[0].disposition
    else {
        panic!(
            "the opposite of the counted guard must be refuted: {:?}",
            outcomes[0].disposition
        );
    };
    assert!(predicate.contains("deref(deref(deref(holder)))"));
    assert!(negation.contains("deref(deref(deref(holder)))"));
}

#[test]
fn counted_range_does_not_treat_a_read_only_box_deref_as_a_consume() {
    let source = br#"fn probe(holder: own box<u64>) -> own unit traps {
  for @items i in deref(holder)..1_u64 {
    claim impossible: igt(deref(holder), i) because "the captured lower endpoint cannot exceed the binder";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn counted_range_does_not_duplicate_the_deref_of_a_let_bound_owning_box() {
    let source = br#"fn probe() -> own unit allocates(heap), traps {
  let holder = box_new(0_u64);
  for @items i in deref(holder)..1_u64 {
    claim impossible: igt(deref(holder), i) because "the captured lower endpoint cannot exceed the binder";
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = claims(source, "probe");
    assert_eq!(outcomes.len(), 1);
    let ClaimDisposition::Refuted { predicate, .. } = &outcomes[0].disposition else {
        panic!("the opposite of the counted lower bound must be refuted");
    };
    assert!(predicate.contains("deref(holder)"));
    assert!(!predicate.contains("deref(deref(holder))"));
}

// ---------------------------------------------------------------------
// [ENT-2..ENT-5, FN-8] exact signed goals and ordinary calls
// ---------------------------------------------------------------------

#[test]
fn whole_goal_sources_discharge_atomically_while_children_do_not() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn from_branch(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if complete {
    guarded(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn from_check(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
  guarded(value: value);
  return unit;
}

fn from_claim(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  claim established: complete because "complete";
  guarded(value: value);
  return unit;
}

fn from_children(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  if positive {
    if small {
      guarded(value: value);
    } else {
      return unit;
    }
  } else {
    return unit;
  }
  return unit;
}

fn from_false(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if complete {
    return unit;
  } else {
    guarded(value: value);
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;

    for function in ["from_branch", "from_check", "from_claim"] {
        let outcomes = call_goals(source, function);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
        assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
    }
    assert_eq!(
        claims(source, "from_claim")[0].disposition,
        ClaimDisposition::Retained,
        "CLM-2 remains comparison-origin-only even though S3 establishes +G"
    );

    let children = call_goals(source, "from_children");
    assert_eq!(children[0].disposition, CallGoalDisposition::Unproved);
    assert!(children[0].evidence.is_empty());

    let negative = call_goals(source, "from_false");
    assert_eq!(negative[0].disposition, CallGoalDisposition::Refuted);
    assert_eq!(negative[0].evidence, vec![CallGoalEvidence::OpaqueNegative]);
}

#[test]
fn an_exact_comparison_call_retains_every_positive_derivation_ground() {
    let source = br#"fn below(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn exact(value: own u64) -> own unit traps {
  let small = ilt(value, 10_u64);
  if small {
    below(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn projected(value: own u64) -> own unit traps {
  let at_most_nine = ile(value, 9_u64);
  if at_most_nine {
    below(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let exact = entailment(source, "exact");
    validate_derivations(&exact);
    assert_eq!(
        exact.call_goals[0].disposition,
        CallGoalDisposition::Discharged
    );
    assert_eq!(
        exact.call_goals[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ]
    );
    assert!(matches!(
        exact.derivations.nodes[call_root(&exact, 0).0 as usize],
        DerivationNode::SourceGoal { .. }
    ));
    let projected = entailment(source, "projected");
    validate_derivations(&projected);
    assert_eq!(
        projected.call_goals[0].disposition,
        CallGoalDisposition::Discharged
    );
    assert_eq!(
        projected.call_goals[0].evidence,
        vec![CallGoalEvidence::ExactL0Projection]
    );
    assert!(matches!(
        projected.derivations.nodes[call_root(&projected, 0).0 as usize],
        DerivationNode::GoalProjection { .. }
    ));
}

#[test]
fn joined_whole_goals_require_the_same_sign_on_every_reachable_input() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn both(value: own u64, choose: own Bool) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if choose {
    check complete else trap "left";
  } else {
    check complete else trap "right";
  }
  guarded(value: value);
  return unit;
}

fn one(value: own u64, choose: own Bool) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if choose {
    check complete else trap "left";
  } else {
    check True() else trap "other";
  }
  guarded(value: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let both = entailment(source, "both");
    validate_derivations(&both);
    assert_eq!(
        both.call_goals[0].disposition,
        CallGoalDisposition::Discharged
    );
    assert_eq!(
        both.call_goals[0].evidence,
        vec![CallGoalEvidence::OpaquePositive]
    );
    assert!(matches!(
        both.derivations.nodes[call_root(&both, 0).0 as usize],
        DerivationNode::JoinGoal { ref parents, .. } if parents.len() == 2
    ));
    let one = entailment(source, "one");
    validate_derivations(&one);
    assert_eq!(one.call_goals[0].disposition, CallGoalDisposition::Unproved);
}

#[test]
fn a_computed_bool_truth_survives_an_origin_write_but_its_expansion_does_not() {
    let source = br#"fn need_true(value: own Bool) -> own unit traps requires {
  check value else trap "true";
} {
  check True() else trap "body";
  return unit;
}

fn below(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn probe(value: own u64) -> own unit traps {
  let small = ilt(value, 10_u64);
  if small {
    set value = 20_u64;
    need_true(value: small);
    below(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "probe");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
    assert_eq!(outcomes[1].disposition, CallGoalDisposition::Unproved);
}

#[test]
fn a_copy_referent_read_through_an_affine_box_is_an_exact_goal_origin() {
    let source = br#"fn observe['r](value: &'r box<i32>) -> own unit reads('r), traps requires {
  let positive = igt(deref(deref(value)), 0_i32);
  let small = ilt(deref(deref(value)), 10_i32);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  let seen = deref(deref(value));
  check True() else trap "body";
  return unit;
}

fn caller() -> own unit allocates(heap), traps {
  let owner = box_new(5_i32);
  let positive = igt(deref(owner), 0_i32);
  let small = ilt(deref(owner), 10_i32);
  let complete = band(positive, small);
  if complete {
    region 'r {
      observe<'r>(value: &'r owner);
    }
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "caller");
    validate_derivations(&summary);
    assert_eq!(summary.call_goals.len(), 1);
    assert_eq!(
        summary.call_goals[0].disposition,
        CallGoalDisposition::Discharged
    );
    assert_eq!(
        summary.call_goals[0].evidence,
        vec![CallGoalEvidence::OpaquePositive]
    );
    assert_root_has_event_kind(&summary, call_root(&summary, 0), FlowEventKind::S1);
}

#[test]
fn setting_an_intermediate_bool_binding_stops_later_origin_expansion() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  check igt(value, 0_u64) else trap "positive";
} {
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let alias = positive;
  set positive = False();
  if alias {
    guarded(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "caller");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Unproved);
    assert!(outcomes[0].evidence.is_empty());
}

#[test]
fn resolved_writes_stop_future_expansion_of_the_written_origin_binding() {
    let source = br#"fn need() -> own unit traps requires {
  let first = ilt(0_u64, 1_u64);
  let second = ilt(1_u64, 2_u64);
  let complete = band(first, second);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn mutate['r](value: &uniq 'r Bool) -> own unit writes('r) {
  set deref(value) = False();
  return unit;
}

fn through_holder() -> own unit traps {
  let first = ilt(0_u64, 1_u64);
  let second = ilt(1_u64, 2_u64);
  let source = band(first, second);
  region 'r {
    let holder = &uniq 'r source;
    set deref(holder) = False();
  }
  let alias = source;
  if alias {
    need();
  } else {
    return unit;
  }
  return unit;
}

fn through_call() -> own unit traps {
  let first = ilt(0_u64, 1_u64);
  let second = ilt(1_u64, 2_u64);
  let source = band(first, second);
  region 'r {
    mutate<'r>(value: &uniq 'r source);
  }
  let alias = source;
  if alias {
    need();
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in ["through_holder", "through_call"] {
        let outcomes = call_goals(source, function);
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].disposition, CallGoalDisposition::Unproved);
        assert!(outcomes[0].evidence.is_empty());
    }
}

#[test]
fn combined_contradiction_is_absorbing_before_goal_and_l0_support_kills() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn signed(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "first";
  if complete {
    return unit;
  } else {
    set value = 20_u64;
    guarded(value: value);
    claim unreachable: ilt(value, 1_u64) because "combined contradiction";
  }
  return unit;
}

fn l0(value: own u64) -> own unit traps {
  check ilt(value, 5_u64) else trap "low";
  check ige(value, 5_u64) else trap "high";
  set value = 20_u64;
  guarded(value: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for (function, is_goal) in [("signed", true), ("l0", false)] {
        let summary = entailment(source, function);
        validate_derivations(&summary);
        assert_eq!(summary.call_goals.len(), 1);
        assert_eq!(
            summary.call_goals[0].disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            summary.call_goals[0].evidence,
            vec![CallGoalEvidence::AllDerivable]
        );
        let root = &summary.derivations.nodes[call_root(&summary, 0).0 as usize];
        assert!(
            if is_goal {
                matches!(root, DerivationNode::GoalContradiction { .. })
            } else {
                matches!(root, DerivationNode::L0Contradiction { .. })
            },
            "{function} must retain its exact contradiction class: {root:#?}"
        );
    }
    assert!(!matches!(
        claims(source, "signed")[0].disposition,
        ClaimDisposition::Refuted { .. }
    ));
}

#[test]
fn a_discharged_whole_goal_is_accepted_on_the_ordinary_path() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  if complete {
    guarded(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the established whole goal must accept its call: {outcome:?}");
        };
        let caller = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "caller")
            .expect("caller function");
        assert_eq!(caller.entailment.call_goals.len(), 1);
        assert_eq!(
            caller.entailment.call_goals[0].disposition,
            CallGoalDisposition::Discharged
        );
    });
}

#[test]
fn fn8_call_rejection_carries_the_complete_deterministic_payload() {
    let source = br#"fn guarded(value: own u64) -> own unit traps requires {
  let positive = igt(value, 0_u64);
  let small = ilt(value, 10_u64);
  let complete = band(positive, small);
  check complete else trap "complete";
} {
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  guarded(value: value);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unproved complete goal must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::UndischargedCallRequirement(detail) = issue.kind() else {
            panic!(
                "FN-8 must carry the ordinary-call payload: {:?}",
                issue.kind()
            );
        };
        assert_eq!(detail.concrete_callee, "guarded");
        assert!(!detail.final_check.components().is_empty());
        assert!(detail.instantiated_goal.contains("Boolean(And)"));
        assert!(detail.instantiated_goal.contains("Integer(U64)"));
        assert_eq!(detail.disposition, CallRequirementDisposition::Unproved);
        assert_eq!(
            detail.mechanical_fix,
            "establish the complete callee requirement with one dominating branch, check, or claim before the call"
        );
        let crate::SemanticLocation::SourceNode(_, coordinate) = issue.location() else {
            panic!("FN-8 must cite the source call");
        };
        let start = usize::try_from(coordinate.start().value()).expect("offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("offset fits");
        assert_eq!(&source[start..end], b"guarded(value: value)");
    });
}

#[test]
fn actual_obligations_precede_fn8_and_ephemeral_goals_use_the_stronger_fix() {
    let admitted_actual = br#"fn positive(value: own u8) -> own unit traps requires {
  check ilt(value, 10_u8) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn caller() -> own unit traps {
  let values = array_new<u8, 2>(3_u8);
  positive(value: values[0_u64]);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(admitted_actual, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the ephemeral goal is not source-establishable: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn8);
        let SemanticIssueKind::UndischargedCallRequirement(detail) = issue.kind() else {
            panic!("expected FN-8 payload, got {:?}", issue.kind());
        };
        assert_eq!(detail.disposition, CallRequirementDisposition::Unproved);
        assert!(
            detail
                .instantiated_goal
                .contains("argument #0 pre-transfer value")
        );
        assert_eq!(
            detail.mechanical_fix,
            "bind that argument or referent value with one preceding ordinary let, establish the complete requirement over that binding, and pass the binding, borrowing it when the parameter mode requires a borrow"
        );
    });
    let admitted = entailment(admitted_actual, "caller");
    validate_derivations(&admitted);
    assert_eq!(admitted.obligations.len(), 1);
    assert!(admitted.obligations[0].discharged);
    let actual_root = obligation_root(&admitted, 0);
    for kind in [ImplicitBoundKind::Constant, ImplicitBoundKind::ArrayLength] {
        assert_root_contains(
            &admitted,
            actual_root,
            |node| matches!(node, DerivationNode::ImplicitBound { kind: actual, .. } if *actual == kind),
            "the concrete array actual's implicit bounds",
        );
    }
    assert_eq!(admitted.call_goals.len(), 1);
    assert_eq!(
        admitted.call_goals[0].disposition,
        CallGoalDisposition::Unproved
    );
    assert!(admitted.call_goals[0].derivation.is_none());

    let failed_actual = br#"fn positive(value: own u8) -> own unit traps requires {
  check ilt(value, 10_u8) else trap "small";
} {
  check True() else trap "body";
  return unit;
}

fn caller() -> own unit traps {
  let values = array_new<u8, 2>(3_u8);
  positive(value: values[9_u64]);
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    with_semantics(failed_actual, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the actual's own OP-4 failure must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedBoundsObligation { .. }
        ));
    });
    let failed = entailment(failed_actual, "caller");
    validate_derivations(&failed);
    assert_eq!(failed.obligations.len(), 1);
    assert!(!failed.obligations[0].discharged);
    assert!(failed.obligations[0].derivation.is_none());
    assert!(
        failed.call_goals.is_empty(),
        "FN-8 judgment begins only after every actual obligation succeeds"
    );
}

#[test]
fn a_call_is_judged_before_its_callee_write_and_that_write_kills_the_second_call() {
    let source =
        br#"fn update['r](value: &uniq 'r u64) -> own unit reads('r), writes('r), traps requires {
  check ilt(deref(value), 10_u64) else trap "small";
} {
  let old = deref(value);
  set deref(value) = old;
  check True() else trap "body";
  return unit;
}

fn caller(value: own u64) -> own unit traps {
  let small = ilt(value, 10_u64);
  if small {
    region 'first {
      update<'first>(value: &uniq 'first value);
    }
    region 'second {
      update<'second>(value: &uniq 'second value);
    }
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let summary = entailment(source, "caller");
    validate_derivations(&summary);
    assert_eq!(summary.call_goals.len(), 2);
    assert_eq!(
        summary.call_goals[0].disposition,
        CallGoalDisposition::Discharged
    );
    assert_eq!(
        summary.call_goals[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ]
    );
    assert_root_has_event_kind(&summary, call_root(&summary, 0), FlowEventKind::S1);
    assert_eq!(
        summary.call_goals[1].disposition,
        CallGoalDisposition::Unproved
    );
    assert!(summary.call_goals[1].derivation.is_none());
}

#[test]
fn s4_discharges_the_body_call_until_a_body_write_kills_it() {
    let source = br#"fn observe['r](value: &'r u64) -> own unit reads('r), traps requires {
  check ilt(deref(value), 10_u64) else trap "small";
} {
  let seen = deref(value);
  check True() else trap "body";
  return unit;
}

fn update['r](value: &uniq 'r u64) -> own unit reads('r), writes('r), traps requires {
  check ilt(deref(value), 10_u64) else trap "small";
} {
  region 'first {
    observe<'first>(value: &'first deref(value));
  }
  let old = deref(value);
  set deref(value) = old;
  region 'second {
    observe<'second>(value: &'second deref(value));
  }
  check True() else trap "body";
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "update");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(
        outcomes[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ]
    );
    assert_eq!(outcomes[1].disposition, CallGoalDisposition::Unproved);
}

#[test]
fn an_element_write_keeps_a_whole_goal_supported_only_by_length() {
    let source = br#"fn sized(values: own array<u8, 2>) -> own unit traps requires {
  let size = len(values);
  let exact = ieq(size, 2_u64);
  let complete = band(exact, exact);
  check complete else trap "sized";
} {
  check True() else trap "body";
  return unit;
}

fn caller(values: own array<u8, 2>) -> own unit traps {
  let size = len(values);
  let exact = ieq(size, 2_u64);
  let complete = band(exact, exact);
  if complete {
    set values[0_u64] = 9_u8;
    sized(values: move values);
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "caller");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::OpaquePositive]);
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "length-only support survives the element commit: {outcome:?}"
        );
    });
}

#[test]
fn array_fill_participates_only_in_body_origin_expansion() {
    let source = br#"fn need_true(value: own Bool) -> own unit traps requires {
  check value else trap "true";
} {
  check True() else trap "body";
  return unit;
}

fn probe() -> own unit traps {
  let values = array_new<u8, 4>(0_u8);
  let first_size = len(values);
  let first_exact = ieq(first_size, 4_u64);
  let first = band(first_exact, first_exact);
  if first {
    let second_size = len(values);
    let second_exact = ieq(second_size, 4_u64);
    let second = band(second_exact, second_exact);
    if second {
      return unit;
    } else {
      let impossible = False();
      need_true(value: impossible);
    }
  } else {
    return unit;
  }
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    let outcomes = call_goals(source, "probe");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].disposition, CallGoalDisposition::Discharged);
    assert_eq!(outcomes[0].evidence, vec![CallGoalEvidence::AllDerivable]);
}

#[test]
fn s4_is_independent_of_forward_and_mutually_recursive_traversal_order() {
    let source = br#"fn first(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  second(value: value);
  check True() else trap "body";
  return unit;
}

fn second(value: own u64) -> own unit traps requires {
  check ilt(value, 10_u64) else trap "small";
} {
  first(value: value);
  check True() else trap "body";
  return unit;
}

fn main() -> own unit pure {
  return unit;
}
"#;
    for function in ["first", "second"] {
        let summary = entailment(source, function);
        validate_derivations(&summary);
        assert_eq!(summary.call_goals.len(), 1);
        assert_eq!(
            summary.call_goals[0].disposition,
            CallGoalDisposition::Discharged
        );
        assert_eq!(
            summary.call_goals[0].evidence,
            vec![
                CallGoalEvidence::OpaquePositive,
                CallGoalEvidence::ExactL0Projection,
            ]
        );
        assert_root_has_event_kind(&summary, call_root(&summary, 0), FlowEventKind::S4);
    }
}

#[test]
fn a_forward_concrete_generic_call_uses_its_substituted_goal() {
    let source = br#"fn main() -> own unit pure {
  return unit;
}

fn caller(value: own i32) -> own unit traps {
  let positive = igt(value, 0_i32);
  if positive {
    let result = guarded<i32>(value: value);
  } else {
    return unit;
  }
  return unit;
}

fn guarded<T: Int>(value: own T) -> own T traps requires {
  check igt(value, 0_T) else trap "positive";
} {
  check True() else trap "body";
  return value;
}
"#;
    let summary = entailment(source, "caller");
    validate_derivations(&summary);
    assert_eq!(summary.call_goals.len(), 1);
    assert_eq!(
        summary.call_goals[0].disposition,
        CallGoalDisposition::Discharged
    );
    assert_eq!(
        summary.call_goals[0].evidence,
        vec![
            CallGoalEvidence::OpaquePositive,
            CallGoalEvidence::ExactL0Projection,
        ],
        "concrete generic goal: {:#?}",
        summary.call_goals[0].goal
    );
    assert_root_has_event_kind(&summary, call_root(&summary, 0), FlowEventKind::S1);
    let concrete_goal = format!("{:#?}", summary.call_goals[0].goal);
    assert!(concrete_goal.contains("I32"));
    assert!(!concrete_goal.contains("GenericInt"));
}

#[test]
fn concrete_const_instances_keep_function_local_derivation_inventories() {
    let source = br#"fn first<const n: u64>(values: own array<u8, n>) -> own u8 pure {
  return values[0_u64];
}

fn main() -> own unit pure {
  let small = array_new<u8, 2>(7_u8);
  let small_first = first<2>(values: move small);
  let large = array_new<u8, 5>(9_u8);
  let large_first = first<5>(values: move large);
  return unit;
}
"#;
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("both concrete const instances must check: {outcome:?}");
        };
        let instances: Vec<_> = checked
            .data
            .functions
            .iter()
            .filter(|function| function.name == "first")
            .collect();
        assert_eq!(instances.len(), 2);
        let mut concrete_lengths = Vec::new();
        for instance in instances {
            let summary = &instance.entailment;
            validate_derivations(summary);
            assert_eq!(summary.obligations.len(), 1);
            assert!(summary.obligations[0].discharged);
            let root = obligation_root(summary, 0);
            for kind in [ImplicitBoundKind::Constant, ImplicitBoundKind::ArrayLength] {
                assert_root_contains(
                    summary,
                    root,
                    |node| matches!(node, DerivationNode::ImplicitBound { kind: actual, .. } if *actual == kind),
                    "the concrete const instance's own implicit array proof",
                );
            }
            let lengths: Vec<_> = summary
                .inventory
                .length_bounds
                .iter()
                .filter_map(|bound| match bound {
                    Some(LengthBound::Constant(value)) => Some(*value),
                    Some(LengthBound::Equal(_)) | None => None,
                })
                .collect();
            assert_eq!(lengths.len(), 1);
            concrete_lengths.push(lengths[0]);
        }
        concrete_lengths.sort_unstable();
        assert_eq!(concrete_lengths, vec![2, 5]);
    });
}
