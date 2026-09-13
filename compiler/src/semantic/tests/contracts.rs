use crate::{
    OverlapLowering, SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule,
    lower_checked,
};

use super::super::model::CheckedSliceOrigin;
use super::{assert_parse_rule, assert_rule, with_semantics};

fn assert_behavior_rule(source: &str, rule: SemanticRule) {
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected {rule:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule, "{issue:?}");
    });
}

fn assert_behavior_site(source: &str, rule: SemanticRule, expected: &str) {
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected {rule:?}");
        };
        assert_eq!(issue.rule(), rule, "{issue:?}");
        let SemanticLocation::SourceNode(_, coordinate) = issue.location();
        let start = usize::try_from(coordinate.start().value()).unwrap();
        let end = usize::try_from(coordinate.end().value()).unwrap();
        assert_eq!(&source[start..end], expected);
    });
}

const BOUNDED_GROUP: &str = r#"formal Key<K: copy> {
  fn hash(value: own K) -> result: own u64 pure contract {
    ensures result <= 99_u64;
  };
}

fn constant_hash(input: own u64) -> output: own u64 pure contract {
  ensures output <= 99_u64;
} {
  return 17_u64;
}

actual ScalarKey : Key<u64> {
  hash = constant_hash;
}

fn apply<Key<K>>(value: own K) -> out: own u64 pure contract {
  ensures out <= 99_u64;
} {
  let result = Key::hash(value: value);
  return result;
}

fn main() -> status: own ExitStatus pure {
  let result = apply::<ScalarKey>(value: 123_u64);
  let bounded = result + 1_u64;
  if bounded == 18_u64 {
    return exit_status(code: 0_u8);
  }
  return exit_status(code: 1_u8);
}
"#;

#[test]
fn group_contracts_publish_only_after_structurally_matched_actual_proofs() {
    for mode in [OverlapLowering::Off, OverlapLowering::On] {
        with_semantics(BOUNDED_GROUP.as_bytes(), |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("{outcome:?}");
            };
            assert!(
                checked
                    .data
                    .functions
                    .iter()
                    .all(|function| !function.formal_hypothesis)
            );
            let lowered = lower_checked(*checked, mode).expect("direct-call lowering");
            assert_eq!(
                lowered
                    .functions()
                    .iter()
                    .filter(|function| !function.blocks().is_empty())
                    .count(),
                3
            );
        });
    }
    // Equivalent integer relations still differ structurally; no theorem
    // implication or algebraic-law engine selects a behavior binding.
    assert_behavior_rule(
        &BOUNDED_GROUP.replace("ensures output <= 99_u64;", "ensures output < 100_u64;"),
        SemanticRule::Fn4,
    );
    assert_behavior_rule(
        &BOUNDED_GROUP.replace("return 17_u64;", "return 100_u64;"),
        SemanticRule::Fn9,
    );
}

#[test]
fn owned_function_formal_results_follow_ordinary_transfer() {
    let source = r#"formal Factory<T: affine> {
  fn make(value: own T) -> result: own T pure;
}

fn retain(value: own FixedVector<u8, 4>) -> result: own FixedVector<u8, 4> pure {
  return move value;
}

actual Identity : Factory<FixedVector<u8, 4>> {
  make = retain;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // v0.58 removes FN-1 state routing and the derived FN-4 freshness
    // ceiling. A same-signature actual can return the value it consumes.
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    with_semantics(
        source
            .replace("return move value;", "return fixed_vector::<u8, 4>();")
            .as_bytes(),
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "{outcome:?}"
            );
        },
    );
}

#[test]
fn raw_function_binders_are_visible_and_emit_no_symbolic_hypotheses() {
    let source = br#"fn zero() -> result: own u64 pure {
  return 0_u64;
}

fn apply<fn get() -> result: own u64 pure>() -> result: own u64 pure {
  return get();
}

fn main() -> status: own ExitStatus pure {
  let value = apply::<fn zero>();
  let other = apply::<fn zero>();
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("{outcome:?}");
        };
        assert!(
            checked
                .data
                .functions
                .iter()
                .all(|function| !function.formal_hypothesis)
        );
        let lowered = lower_checked(*checked, OverlapLowering::Off)
            .expect("raw function substitution lowers directly");
        assert_eq!(
            lowered
                .functions()
                .iter()
                .filter(|function| !function.blocks().is_empty())
                .count(),
            3
        );
    });
}

#[test]
fn a_nominal_only_function_argument_must_match_its_signature() {
    let source = r#"struct Holder<fn pick(value: own u64) -> result: own u64 pure> {
  marker: u64;
}

fn bad(value: own Bool) -> result: own Bool pure {
  return value;
}

fn inspect(value: &Holder<fn bad>) -> result: own unit pure {
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_behavior_site(source, SemanticRule::Fn4, "fn bad");
    with_semantics(
        source.replace("own Bool", "own u64").as_bytes(),
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "{outcome:?}"
            );
        },
    );
}

#[test]
fn called_raw_function_mismatches_point_at_the_written_argument() {
    let source = r#"fn bad(value: own Bool) -> result: own Bool pure {
  return value;
}

fn apply<fn pick(value: own u64) -> result: own u64 pure>(value: own u64) -> result: own u64 pure {
  return pick(value: value);
}

fn main() -> status: own ExitStatus pure {
  let result = apply::<fn bad>(value: 7_u64);
  return exit_status(code: 0_u8);
}
"#;
    assert_behavior_site(source, SemanticRule::Fn4, "fn bad");
}

#[test]
fn function_arguments_do_not_masquerade_as_store_regions() {
    let base = "fn spare() -> result: own unit pure {\n  return unit;\n}\n\n";
    let main = "fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n";
    for ty in [
        "Holder<fn spare>",
        "Box<fn spare>",
        "FixedVector<fn spare, 4>",
        "Vector<fn spare>",
    ] {
        let source = format!(
            "struct Holder['s] {{\n  data: Box<'s, u64>;\n}}\n\n{base}fn inspect(value: &{ty}) -> result: own unit pure {{\n  return unit;\n}}\n\n{main}"
        );
        assert_behavior_rule(&source, SemanticRule::Type5);
    }
    let source = format!(
        "formal Inspect<T: linear> {{\n  fn inspect(value: &T) -> result: own unit pure;\n}}\n\nfn ignore['s](value: &Box<'s, u64>) -> result: own unit pure {{\n  return unit;\n}}\n\nactual Bound['s] : Inspect<Box<'s, u64>> {{\n  inspect = ignore;\n}}\n\n{base}struct Holder<Inspect<T>> {{\n  value: T;\n}}\n\nfn inspect(value: &Holder<Bound<fn spare>>) -> result: own unit pure {{\n  return unit;\n}}\n\n{main}"
    );
    assert_behavior_rule(&source, SemanticRule::Fn2);
    let source = format!(
        "{base}fn main() -> status: own ExitStatus pure {{\n  let value = arena_new::<fn spare, u64>(1_u64);\n  return exit_status(code: 0_u8);\n}}\n"
    );
    assert_behavior_rule(&source, SemanticRule::Op1);
}

#[test]
fn unused_formal_members_obey_ordinary_signature_formation() {
    for (signature, rule) in [
        (
            "fn inspect(value: &u64) -> result: own unit writes(value);",
            SemanticRule::Eff1,
        ),
        (
            "fn inspect['r](left: &'r u64, right: &'r u64) -> result: &'r u64 pure;",
            SemanticRule::Fn1,
        ),
        (
            "fn inspect(value: own u64) -> result: own u64 pure contract {\n    requires value;\n  };",
            SemanticRule::Op5,
        ),
    ] {
        let source = format!(
            "formal Invalid {{\n  {signature}\n}}\n\nfn main() -> status: own ExitStatus pure {{\n  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_behavior_rule(&source, rule);
    }
}

#[test]
fn member_regions_are_instantiated_per_call_and_never_on_the_formal_header() {
    let source = br#"formal Pass {
  fn pass['r](value: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure;
}

fn identity['s](value: own Slice<'s, u8>) -> result: own Slice<'s, u8> pure {
  return value;
}

actual SharedPass : Pass {
  pass = identity;
}

fn both<Pass>['a, 'b](left: own Slice<'a, u8>, left_peer: own Slice<'a, u8>, right: own Slice<'b, u8>, right_peer: own Slice<'b, u8>) -> result: own unit pure {
  let first = Pass::pass(value: left);
  let first_peer = Pass::pass(value: left_peer);
  let second = Pass::pass(value: right);
  let second_peer = Pass::pass(value: right_peer);
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let first = array_new::<u8, 2>(0_u8);
  let second = array_new::<u8, 3>(0_u8);
  region {
    let left = slice_of(&first);
    let left_peer = slice_of(&first);
    region {
      let right = slice_of(&second);
      let right_peer = slice_of(&second);
      both::<SharedPass>(left: left, left_peer: left_peer, right: right, right_peer: right_peer);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    for mode in [OverlapLowering::Off, OverlapLowering::On] {
        with_semantics(source, |outcome| {
            let SemanticOutcome::Complete(checked) = outcome else {
                panic!("{outcome:?}");
            };
            lower_checked(*checked, mode)
                .expect("each member call uses its own ordinary loan regions");
        });
    }
    assert_parse_rule(b"formal Pass['r] {\n}\n", crate::SyntaxRule::Gram2);
}

#[test]
fn a_bound_call_cannot_drop_part_of_a_vector_on_a_written_cycle() {
    let source = r#"fn stop() -> result: own unit pure {
  return unit;
}

fn first<fn work() -> result: own unit pure>() -> result: own unit pure {
  return work();
}

fn second<fn work() -> result: own unit pure>() -> result: own unit pure {
  return first::<fn work>();
}

fn main() -> status: own ExitStatus pure {
  first::<fn second::<fn stop>>();
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected FN-6 rejection");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn6);
        assert!(format!("{:?}", issue.kind()).contains("first -> second -> first"));
    });
    let acyclic = source.replace("return first::<fn work>();", "return work();");
    with_semantics(acyclic.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "acyclic composed calls must remain admitted"
        );
    });
}

#[test]
fn actual_captured_region_bounds_are_checked_at_each_application() {
    let source = r#"formal Inspect<T: linear> {
  fn inspect(value: &T) -> result: own unit pure;
}

fn ignore['s](value: &Box<'s, u64>) -> result: own unit pure {
  return unit;
}

actual ExtentOnly['b: affine] : Inspect<Box<'b, u64>> {
  inspect = ignore;
}

fn apply<Inspect<T>>(value: &T) -> result: own unit pure {
  return Inspect::inspect(value: value);
}

fn accepts['h: linear](value: &Box<'h, u64>) -> result: own unit pure {
  return apply::<ExtentOnly<'h>>(value: value);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_behavior_rule(source, SemanticRule::Prov6);
    let matching = source.replace("'h: linear", "'h: affine");
    with_semantics(matching.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    let unbounded = source.replace("'h: linear", "'h");
    assert_behavior_rule(&unbounded, SemanticRule::Prov6);
}

#[test]
fn member_region_bounds_are_checked_before_an_actual_can_be_called() {
    let source = r#"formal Inspect {
  fn length['s: linear](values: &Vector<'s, u64>) -> result: own u64 reads(values);
}

fn shorter['t: affine](values: &Vector<'t, u64>) -> result: own u64 reads(values) {
  return len_of(deref(values));
}

actual Narrow : Inspect {
  length = shorter;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_behavior_rule(source, SemanticRule::Fn4);
    let matching = source.replace("'t: affine", "'t: linear");
    with_semantics(matching.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    let unbounded = source.replace("'t: affine", "'t");
    assert_behavior_rule(&unbounded, SemanticRule::Fn4);
}

#[test]
fn actual_expansion_cycles_include_member_function_arguments() {
    let source = r#"formal Work {
  fn run() -> result: own unit pure;
}

actual Recursive : Work {
  run = drive::<Recursive>;
}

fn drive<Work>() -> result: own unit pure {
  Work::run();
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Fn3);
        assert!(format!("{:?}", issue.kind()).contains("Recursive -> Recursive"));
    });
}

#[test]
fn captured_store_brands_are_bound_before_loan_region_alpha_matching() {
    let source = r#"struct Item {
  rank: u64;
}

formal Inspect<T: linear> {
  fn inspect(value: &T) -> result: own unit pure;
}

fn ignore['store](value: &Box<'store, Item>) -> result: own unit pure {
  return unit;
}

actual ItemInspect['s] : Inspect<Box<'s, Item>> {
  inspect = ignore;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
}

#[test]
fn qualified_actual_forwarding_remains_an_acyclic_abbreviation() {
    let source = r#"formal Factory {
  fn make() -> result: own u64 pure;
}

fn zero() -> result: own u64 pure {
  return 0_u64;
}

actual First : Factory {
  make = zero;
}

actual Second : Factory {
  make = First::make;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(
            checked
                .data
                .functions
                .iter()
                .filter(|function| function.body.is_some())
                .count(),
            2
        );
        assert!(
            checked
                .data
                .functions
                .iter()
                .all(|function| !function.formal_hypothesis)
        );
    });
    // A self-reference is already visible at the declaration point, so this
    // reaches abbreviation-cycle checking rather than a forward-use error.
    assert_behavior_rule(
        &source.replace("make = zero;", "make = First::make;"),
        SemanticRule::Fn3,
    );
}

#[test]
fn actual_member_aliases_preserve_the_complete_instantiation_cycle() {
    let source = r#"formal Work {
  fn run() -> result: own u64 pure;
}

actual First : Work {
  run = trampoline;
}

actual Alias : Work {
  run = First::run;
}

fn poly<T: affine>() -> result: own u64 pure {
  return invoke::<Alias>();
}

fn trampoline() -> result: own u64 pure {
  return poly::<u64>();
}

fn invoke<Work>() -> result: own u64 pure {
  return Work::run();
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    for program in [
        source.to_owned(),
        source.replace("invoke::<Alias>()", "invoke::<First>()"),
    ] {
        with_semantics(program.as_bytes(), |outcome| {
            let SemanticOutcome::SourceIssue { issue } = outcome else {
                panic!("{outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Fn6);
            let SemanticIssueKind::PolymorphicRecursion { cycle, .. } = issue.kind() else {
                panic!("{issue:?}");
            };
            assert!(
                cycle.contains("poly") && cycle.contains("trampoline"),
                "{cycle}"
            );
        });
    }
}

#[test]
fn qualified_function_formals_cannot_silently_drop_specialization_arguments() {
    let source = r#"formal Work {
  fn run() -> result: own unit pure;
}

fn recurse<Work>() -> result: own unit pure {
  return recurse::<fn Work::run::<u64>>();
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_behavior_rule(source, SemanticRule::Fn2);
    with_semantics(
        source
            .replace("fn Work::run::<u64>", "fn Work::run")
            .as_bytes(),
        |outcome| {
            assert!(
                matches!(outcome, SemanticOutcome::Complete(_)),
                "{outcome:?}"
            );
        },
    );
}

#[test]
fn instantiation_forwards_all_kinds_and_refuses_constructed_cycles_on_both_paths() {
    let forward = br#"fn task() -> result: own unit pure {
  return unit;
}

fn repeat<T: copy, const n: u64, fn work() -> result: own unit pure>(value: own T) -> result: own T pure {
  work();
  return repeat::<T, n, fn work>(value: value);
}

fn main() -> status: own ExitStatus pure {
  let result = repeat::<u64, 1, fn task>(value: 0_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(forward, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("{outcome:?}");
        };
        lower_checked(*checked, OverlapLowering::Off)
            .expect("the unchanged vector has a finite direct-call instance graph");
    });
    let wrapped = r#"fn nested<fn work() -> result: own unit pure>() -> result: own unit pure {
  return nested::<fn nested::<fn work>>();
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_behavior_rule(wrapped, SemanticRule::Fn6);
    let growing = r#"struct Grow<T: affine> {
  next: box<Grow<box<T>>>;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let selector = r#"fn relation(value: own u64) -> result: own u64 pure contract {
  ensures result == value;
} {
  return value;
}

"#;
    // Resolution's FN-9 selector preflight and ordinary complete-unit
    // checking must both refuse before materializing Grow<box<...>>.
    for source in [growing.to_owned(), format!("{selector}{growing}")] {
        with_semantics(source.as_bytes(), |outcome| {
            let SemanticOutcome::SourceIssue { issue } = outcome else {
                panic!("{outcome:?}");
            };
            assert_eq!(issue.rule(), SemanticRule::Fn6);
            let SemanticIssueKind::PolymorphicRecursion { cycle, .. } = issue.kind() else {
                panic!("{issue:?}");
            };
            assert_eq!(cycle, "Grow -> Grow");
        });
    }
}

#[test]
fn the_formal_row_survives_a_narrower_actual_and_the_containing_instance() {
    let source = r#"struct Pair {
  left: u64;
  right: u64;
}

formal Inspect<T: affine> {
  fn read(value: &T) -> result: own u64 reads(value);
}

fn left(input: &Pair) -> result: own u64 reads(input.left) {
  return deref(input).left;
}

actual Left : Inspect<Pair> {
  read = left;
}

fn outer<Inspect<T>>(value: &T) -> result: own u64 reads(value) {
  return Inspect::read(value: value);
}

fn wrapper(value: &Pair) -> result: own u64 reads(value) {
  return outer::<Left>(value: value);
}

fn main() -> status: own ExitStatus pure {
  let pair = Pair(left: 3_u64, right: 5_u64);
  region {
    let observed = wrapper(value: &pair);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("{outcome:?}");
        };
        lower_checked(*checked, OverlapLowering::Off)
            .expect("covered actual lowers as a direct call");
    });
    // v0.58 EFF-2 projects the formal's root onto the resolved actual place,
    // without value-history leaf expansion. The complete formal read remains
    // exhibited; substituting the narrower actual would accept this wrapper
    // incorrectly when it declares only the left field.
    assert_behavior_rule(
        &source.replace(
            "fn wrapper(value: &Pair) -> result: own u64 reads(value)",
            "fn wrapper(value: &Pair) -> result: own u64 reads(value.left)",
        ),
        SemanticRule::Eff2,
    );
}

fn assert_issue_slice(source: &[u8], rule: SemanticRule, kind: SemanticIssueKind, expected: &[u8]) {
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue } = outcome else {
            panic!("expected {rule:?}/{kind:?}, got {outcome:?}");
        };
        assert_eq!(issue.rule(), rule);
        assert_eq!(issue.kind(), &kind);
        let SemanticLocation::SourceNode(_, coordinate) = issue.location();
        let start = usize::try_from(coordinate.start().value()).expect("test offset fits");
        let end = usize::try_from(coordinate.end().value()).expect("test offset fits");
        assert_eq!(&source[start..end], expected);
    });
}

#[test]
fn factored_member_call_grammar_preserves_constructor_boundaries() {
    for (statement, rule) in [
        ("let value = Empty::<u64>();", SemanticRule::Type5),
        ("let value = True::<u64>();", SemanticRule::Type5),
        ("let value = True<u64>();", SemanticRule::Type5),
        ("let value = True(7_u64);", SemanticRule::Gram8),
        ("let value = Empty(7_u64);", SemanticRule::Gram8),
        ("Empty();", SemanticRule::Type5),
        ("let (left, right) = Empty();", SemanticRule::Type5),
    ] {
        let source = format!(
            "struct Empty {{\n}}\n\nfn main() -> status: own ExitStatus pure {{\n  {statement}\n  return exit_status(code: 0_u8);\n}}\n"
        );
        assert_behavior_rule(&source, rule);
    }
}

#[test]
fn missing_entry_diagnostic_salvage_checks_instantiation_before_discovery() {
    let source = "struct Grow<T: affine> {\n  next: box<Grow<box<T>>>;\n}\n";
    assert_behavior_rule(source, SemanticRule::Fn6);
    let source = "fn repeat<T: affine>(value: own T) -> result: own unit pure {\n  let boxed = box_new(move value);\n  repeat::<box<T>>(value: move boxed);\n  return unit;\n}\n";
    assert_behavior_rule(source, SemanticRule::Fn6);
}

#[test]
fn static_group_bindings_have_no_executable_metadata() {
    let source = br#"formal Zeroed {
  fn zero() -> result: own i32 pure;
}

fn make_zero() -> result: own i32 pure {
  return 0_i32;
}

actual Zero : Zeroed {
  zero = make_zero;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("valid static conformance must check: {outcome:?}");
        };
        // D7 replaces standalone conformance metadata with checked group
        // expansion. No symbolic function hypothesis reaches executable IR.
        assert!(
            checked
                .data
                .functions
                .iter()
                .all(|function| !function.formal_hypothesis)
        );
        assert_eq!(
            checked
                .data
                .functions
                .iter()
                .filter(|function| function.name == "make_zero")
                .count(),
            1
        );

        // The bound function is still present exactly once in the ordinary
        // function table. Contract metadata contributes no executable function.
        assert_eq!(
            checked
                .data
                .functions
                .iter()
                .filter(|function| function.body.is_some())
                .count(),
            2
        );
        let lowered = lower_checked(*checked, OverlapLowering::Off)
            .expect("static contract metadata must not alter ordinary lowering");
        assert_eq!(
            lowered
                .functions()
                .iter()
                .filter(|function| !function.blocks().is_empty())
                .count(),
            2
        );
    });
}

#[test]
fn empty_formal_and_actual_groups_are_valid() {
    let source = br#"formal Marker {
}

actual Empty : Marker {
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("empty marker conformance must check: {outcome:?}");
        };
        assert_eq!(
            checked
                .data
                .functions
                .iter()
                .filter(|function| function.body.is_some())
                .count(),
            1
        );
        assert!(!checked.data.functions[0].formal_hypothesis);
    });
}

#[test]
fn actual_header_materializes_its_only_generic_nominal_instance() {
    let source = br#"struct Wrapper<T: affine> {
  value: T;
}

formal Marker<T: affine> {
}

actual Wrapped : Marker<Wrapper<i32>> {
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("contract-only generic subject must check: {outcome:?}");
        };
        assert!(
            checked
                .data
                .nominals
                .iter()
                .any(|nominal| nominal.name.starts_with("Wrapper<"))
        );
        let semantic_nominal_count = checked.data.nominals.len();
        let lowered = lower_checked(*checked, OverlapLowering::Off)
            .expect("conformance-only nominal metadata must not affect ordinary lowering");
        // FN-3 expansion leaves only concrete nominal instances here; the
        // retired conformance-subject placeholder is no longer in this table.
        assert_eq!(lowered.nominals().len(), semantic_nominal_count);
    });
}

#[test]
fn formal_member_materializes_its_only_generic_nominal_instance() {
    let source = br#"struct Wrapper<T: affine> {
  value: T;
}

formal Factory {
  fn make() -> result: own Wrapper<i32> pure;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("contract-only generic member type must check: {outcome:?}");
        };
        assert!(
            checked
                .data
                .nominals
                .iter()
                .any(|nominal| nominal.name.starts_with("Wrapper<"))
        );
        let semantic_nominal_count = checked.data.nominals.len();
        let lowered = lower_checked(*checked, OverlapLowering::Off)
            .expect("contract-only nominal metadata must not affect ordinary lowering");
        // A formal signature contributes its concrete type, not a separate
        // contract-subject placeholder that lowering would have to discard.
        assert_eq!(lowered.nominals().len(), semantic_nominal_count);
    });
}

#[test]
fn retired_owned_law_identity_syntax_is_not_admitted() {
    let source = br#"const zero: FixedVector<u8, 1> =[0_u8];

const x: FixedVector<u8, 1> =[0_u8];

const y: FixedVector<u8, 1> =[0_u8];

contract InvalidIdentity {
  fn combine() -> result: own FixedVector<u8, 1> pure;
  law identity(combine, zero);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // D7 removes law syntax, including the formerly invalid owned identity.
    assert_parse_rule(source, crate::SyntaxRule::Gram2);
}

#[test]
fn migrated_group_rejections_keep_their_selected_rules() {
    assert_rule(
        include_bytes!("../../../../tests/conformance/cases/fn3-neg-missing-binding.wf"),
        SemanticRule::Fn3,
        SemanticIssueKind::type_mismatch(
            "an actual binds every formal member exactly once in declared order",
            "a nonmatching behavior argument",
        ),
    );
    // D7 assigns interface compatibility to FN-4. The prior implicit
    // type/conformance uniqueness test is now the two-explicit-actuals
    // positive; it cannot remain a negative under the selected design.
    for source in [
        include_str!("../../../../tests/conformance/cases/fn4-neg-requires-mismatch.wf"),
        include_str!("../../../../tests/conformance/cases/fn4-neg-formal-row-coverage.wf"),
    ] {
        assert_behavior_rule(source, SemanticRule::Fn4);
    }
}

#[test]
fn retired_closed_law_table_has_no_remaining_acceptance_path() {
    // The owner's D7 removes law/law_arg entirely. Retain every old source:
    // formerly discharged and undischarged rows must both stop at grammar,
    // rather than leave a hidden theorem/metadata acceptance path behind.
    for source in [
        br#"contract Semigroup {
  doc "A law uses a semantic name from the closed {associative, commutative, identity} table [FN-4].";
  fn combine(x: own i32, y: own i32) -> result: own i32 pure;
  law associative(combine);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.as_slice(),
        br#"contract BadLaw {
  fn combine(x: own i32, y: own i32) -> result: own i32 pure;
  law distributive(combine, combine);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.as_slice(),
        br#"contract BadMonoid {
  doc "FN-4: signed saturating add is NOT associative ((MAX sat+ 1) sat+ -1 != MAX sat+ (1 sat+ -1)); the stated law must be refuted, not trusted.";
  fn combine(x: own i64, y: own i64) -> result: own i64 pure;
  law associative(combine);
}

fn satadd_signed(x: own i64, y: own i64) -> result: own i64 pure {
  return x +sat y;
}

conform i64: BadMonoid {
  combine = satadd_signed;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.as_slice(),
        br#"contract OpaqueMonoid {
  doc "FN-4: a stated law on a fn whose body is not a single table op has no static proof; stated-but-unchecked is a hard reject, never a trusted fact.";
  fn combine(x: own u64, y: own u64) -> result: own u64 pure;
  law associative(combine);
}

fn twostep(x: own u64, y: own u64) -> result: own u64 pure {
  let t = x +wrap y;
  return t +wrap 1_u64;
}

conform u64: OpaqueMonoid {
  combine = twostep;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.as_slice(),
        br#"contract SatMonoid {
  doc "FN-4 stated-and-checked: the exact direct +sat body and closed unsigned table cells discharge these laws.";
  fn combine(x: own u64, y: own u64) -> result: own u64 pure;
  law associative(combine);
  law commutative(combine);
  law identity(combine, 0_u64);
}

fn satadd(x: own u64, y: own u64) -> result: own u64 pure {
  return x +sat y;
}

conform u64: SatMonoid {
  combine = satadd;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#.as_slice(),
    ] {
        assert_parse_rule(source, crate::SyntaxRule::Gram2);
    }
}

#[test]
fn retired_law_identity_with_wrong_literal_type_is_a_grammar_error() {
    let source = br#"contract BadIdentity {
  fn combine(x: own u64, y: own u64) -> result: own u64 pure;
  law identity(combine, unit);
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_parse_rule(source, crate::SyntaxRule::Gram2);
}

#[test]
fn retired_law_identity_with_an_earlier_constant_is_a_grammar_error() {
    let source = br#"const zero: u64 = 0_u64;

contract AddIdentity {
  fn combine(x: own u64, y: own u64) -> result: own u64 pure;
  law identity(combine, zero);
}

fn saturating_add(x: own u64, y: own u64) -> result: own u64 pure {
  return x +sat y;
}

conform u64: AddIdentity {
  combine = saturating_add;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_parse_rule(source, crate::SyntaxRule::Gram2);
}

#[test]
fn formal_headers_are_generic_but_cannot_construct_other_groups() {
    let source = br#"formal Generic<T: affine> {
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // D7 deliberately admits this formerly rejected declaration: a formal
    // header is the flat type/const part of its expansion.
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
    let nested = String::from_utf8(source.to_vec())
        .expect("ASCII fixture")
        .replace("T: affine", "fn make() -> result: own u64 pure");
    assert_behavior_rule(&nested, SemanticRule::Fn3);
}

#[test]
fn repeated_member_points_at_the_later_signature() {
    let source = br#"formal Repeated {
  fn value() -> result: own i32 pure;
  fn value() -> result: own i32 pure;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::type_mismatch(
            "each formal member name occurs once",
            "a nonmatching behavior argument",
        ),
        b"fn value() -> result: own i32 pure",
    );
}

#[test]
fn retired_numeric_conformance_spelling_is_a_grammar_error() {
    let source = br#"conform i32: Int {
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // Int is a built-in numeric bound; D7 has no conformance declaration.
    // The corresponding formal-domain mismatch is covered in resolution.
    assert_parse_rule(source, crate::SyntaxRule::Form1);
}

#[test]
fn actual_header_arguments_match_the_formal_header_arity() {
    let source = br#"formal Plain {
}

actual Invalid : Plain<i32> {
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::type_mismatch(
            "0 written expanded generic arguments",
            "1 written expanded generic argument",
        ),
        b"Plain<i32>",
    );
}

#[test]
fn different_actual_names_can_bind_the_same_formal_and_type() {
    let source = br#"formal Marker<T: copy> {
}

actual First : Marker<i32> {
}

actual Second : Marker<i32> {
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // D7 replaces the implicit (type, contract) uniqueness key with explicit
    // argument-group names. Choosing between them is a call-site obligation.
    with_semantics(source, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "{outcome:?}"
        );
    });
}

#[test]
fn incompatible_and_out_of_order_bindings_point_at_the_fn_bind() {
    let source = br#"formal Pair {
  fn first() -> result: own i32 pure;
  fn second() -> result: own i32 pure;
}

fn make_first() -> result: own i32 pure {
  return 1_i32;
}

fn make_second() -> result: own i32 pure {
  return 2_i32;
}

actual Reversed : Pair {
  second = make_second;
  first = make_first;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::type_mismatch(
            "actual member names follow the formal's declared order",
            "a nonmatching behavior argument",
        ),
        b"second = make_second;",
    );
}

#[test]
fn missing_binding_points_at_the_complete_actual_declaration() {
    let source = br#"formal Pair {
  fn first() -> result: own i32 pure;
  fn second() -> result: own i32 pure;
}

fn make_first() -> result: own i32 pure {
  return 1_i32;
}

actual Incomplete : Pair {
  first = make_first;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_issue_slice(
        source,
        SemanticRule::Fn3,
        SemanticIssueKind::type_mismatch(
            "an actual binds every formal member exactly once in declared order",
            "a nonmatching behavior argument",
        ),
        b"actual Incomplete : Pair {\n  first = make_first;\n}",
    );
}

#[test]
fn retired_source_contract_bound_is_a_grammar_error() {
    let source = br#"contract Marker {
}

fn generic<T: Marker>() -> result: own unit pure {
  return unit;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    // D7 removes this type-attached contract mechanism. A formal group in
    // the numeric-bound position is separately refused by name resolution.
    assert_parse_rule(source, crate::SyntaxRule::Gram2);
}

#[test]
fn positional_region_alpha_equality_covers_modes_and_normalized_effect_sets() {
    let source = br#"formal LengthSum {
  fn sum(x: &FixedVector<u8, 4>, y: &FixedVector<u8, 4>) -> result: own u64 reads(x, y);
}

fn add_lengths(first: &FixedVector<u8, 4>, second: &FixedVector<u8, 4>) -> result: own u64 reads(second, first) {
  let first_length = len_of(deref(first));
  let second_length = len_of(deref(second));
  return first_length +wrap second_length;
}

actual Sum : LengthSum {
  sum = add_lengths;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("positional region alpha equality must check: {outcome:?}");
        };
        assert!(
            checked
                .data
                .functions
                .iter()
                .all(|function| !function.formal_hypothesis)
        );
    });
}

#[test]
fn positional_region_alpha_equality_includes_slice_type_regions() {
    let source = br#"formal ByteReader {
  fn first(values: own Slice<u8>) -> result: own u8 reads(values);
}

fn read_first(bytes: own Slice<u8>) -> result: own u8 reads(bytes) {
  let spare = len_of(bytes);
  let ok = 0_u64 < spare;
  if ok {
    return bytes[0_u64];
  } else {
    return 0_u8;
  }
}

actual Bytes : ByteReader {
  first = read_first;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("slice regions must compare by parameter ordinal: {outcome:?}");
        };
        assert!(
            checked
                .data
                .functions
                .iter()
                .all(|function| !function.formal_hypothesis)
        );
    });
}

#[test]
fn positional_region_ordinal_swap_is_not_alpha_equal() {
    let source = br#"formal FirstLength {
  fn length(x: &FixedVector<u8, 4>, y: &FixedVector<u8, 4>) -> result: own u64 reads(x);
}

fn second_length(first: &FixedVector<u8, 4>, second: &FixedVector<u8, 4>) -> result: own u64 reads(second) {
  return len_of(deref(second));
}

actual Wrong : FirstLength {
  length = second_length;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    assert_behavior_rule(
        std::str::from_utf8(source).expect("ASCII fixture"),
        SemanticRule::Fn4,
    );
}

#[test]
fn formal_slice_results_share_function_signature_formation() {
    let source = br#"formal SlicePass {
  fn pass['r](value: own Slice<'r, u8>) -> result: own Slice<'r, u8> pure;
}

fn preserve['s](bytes: own Slice<'s, u8>) -> result: own Slice<'s, u8> pure {
  return bytes;
}

actual Bytes : SlicePass {
  pass = preserve;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("own direct-slice contract result must check: {outcome:?}");
        };
        let ceiling = &checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "preserve")
            .expect("bound implementation")
            .slice_return_ceiling;
        assert_eq!(ceiling.len(), 2);
        assert!(matches!(ceiling[0], CheckedSliceOrigin::ImmutableConst));
        assert!(matches!(ceiling[1], CheckedSliceOrigin::FormalSlice { .. }));
    });

    assert_rule(
        br#"formal Invalid {
  fn borrowed['descriptor, 'data](value: &uniq 'descriptor Slice<'data, u8>) -> result: &uniq 'descriptor Slice<'data, u8> pure;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#,
        SemanticRule::Fn1,
        SemanticIssueKind::BorrowedSliceResult {
            mechanical_fix: "return the direct own slice descriptor under its data region; do not return a borrow of a slice descriptor",
        },
    );
}

#[test]
fn contract_effect_paths_compare_parameter_and_field_ordinals() {
    let accepted = br#"struct Pair {
  left: u64;
  right: u64;
}

formal Touch {
  fn touch(value: &uniq Pair) -> result: own unit reads(value.left), writes(value.right);
}

fn apply(input: &uniq Pair) -> result: own unit reads(input.left), writes(input.right) {
  let observed = deref(input).left;
  set deref(input).right = observed;
  return unit;
}

actual PairTouch : Touch {
  touch = apply;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(accepted, |outcome| {
        assert!(
            matches!(outcome, SemanticOutcome::Complete(_)),
            "contract paths must alpha-normalize parameter and field ordinals: {outcome:?}"
        );
    });

    let mut wrong = accepted.to_vec();
    let row = b"reads(input.left), writes(input.right)";
    let at = wrong
        .windows(row.len())
        .position(|window| window == row)
        .expect("fixture contains the implementation row");
    wrong.splice(
        at..at + row.len(),
        b"reads(input.left), writes(input.left)".iter().copied(),
    );
    let assignment = b"set deref(input).right = observed;";
    let at = wrong
        .windows(assignment.len())
        .position(|window| window == assignment)
        .expect("fixture contains the implementation assignment");
    wrong.splice(
        at..at + assignment.len(),
        b"set deref(input).left = observed;".iter().copied(),
    );
    assert_behavior_rule(
        std::str::from_utf8(&wrong).expect("ASCII fixture"),
        SemanticRule::Fn4,
    );
}
