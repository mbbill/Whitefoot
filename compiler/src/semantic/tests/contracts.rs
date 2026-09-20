use crate::{
    OverlapLowering, SemanticIssueKind, SemanticLocation, SemanticOutcome, SemanticRule,
    lower_checked,
};

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

const BOUNDED_GROUP: &str = r#"interface Key<K: copy> {
  fn hash(value: own K) -> result: own u64 pure contract {
    ensures result <= 99_u64;
  };
}

fn constant_hash(input: own u64) -> output: own u64 pure contract {
  ensures output <= 99_u64;
} {
  return 17_u64;
}

binding ScalarKey : Key<u64> {
  hash = constant_hash;
}

fn apply<interface Key<K>>(value: own K) -> out: own u64 pure contract {
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

// Retired with the region parameter of [FN-2, FORM-8]: v0.59's
// `member_regions_are_instantiated_per_call_and_never_on_the_formal_header`
// instantiated a formal member's `['r]` at each call and refused a region
// list on the formal header. v0.60 has no region parameter and no `Slice`
// type, so a formal member has no per-call region left to instantiate; the
// surviving FN-4 signature check over reference parameters is
// `formal_row_comparison_uses_parameter_ordinals_not_binder_spellings` and
// `formal_range_reference_parameters_compare_by_ordinal` below.

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
fn actual_expansion_cycles_include_member_function_arguments() {
    let source = r#"interface Work {
  fn run() -> result: own unit pure;
}

binding Recursive : Work {
  run = drive::<Recursive>;
}

fn drive<interface Work>() -> result: own unit pure {
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
fn qualified_actual_forwarding_remains_an_acyclic_abbreviation() {
    let source = r#"interface Factory {
  fn make() -> result: own u64 pure;
}

fn zero() -> result: own u64 pure {
  return 0_u64;
}

binding First : Factory {
  make = zero;
}

binding Second : Factory {
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
    let source = r#"interface Work {
  fn run() -> result: own u64 pure;
}

binding First : Work {
  run = trampoline;
}

binding Alias : Work {
  run = First::run;
}

fn poly<T: drop>() -> result: own u64 pure {
  return invoke::<Alias>();
}

fn trampoline() -> result: own u64 pure {
  return poly::<u64>();
}

fn invoke<interface Work>() -> result: own u64 pure {
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
    let growing = r#"struct Grow<T: drop> {
  next: Box<Grow<Box<T>>>;
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
    // checking must both refuse before materializing Grow<Box<...>>.
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
fn missing_entry_diagnostic_salvage_checks_instantiation_before_discovery() {
    let source = "struct Grow<T: drop> {\n  next: Box<Grow<Box<T>>>;\n}\n";
    assert_behavior_rule(source, SemanticRule::Fn6);
    let source = "fn repeat<T: drop>(value: own T) -> result: own unit pure {\n  let boxed = box_new::<T>(value: move value);\n  repeat::<Box<T>>(value: move boxed);\n  return unit;\n}\n";
    assert_behavior_rule(source, SemanticRule::Fn6);
}

#[test]
fn static_group_bindings_have_no_executable_metadata() {
    let source = br#"interface Zeroed {
  fn zero() -> result: own i32 pure;
}

fn make_zero() -> result: own i32 pure {
  return 0_i32;
}

binding Zero : Zeroed {
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
    let source = br#"interface Marker {
}

binding Empty : Marker {
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
    let source = br#"struct Wrapper<T: drop> {
  value: T;
}

interface Marker<T: drop> {
}

binding Wrapped : Marker<Wrapper<i32>> {
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
    let source = br#"struct Wrapper<T: drop> {
  value: T;
}

interface Factory {
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
    let source = br#"contract InvalidIdentity {
  fn combine() -> result: own Slots<u8, 1> pure;
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
            "a binding group binds every interface member exactly once in declared order",
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
fn repeated_member_points_at_the_later_signature() {
    let source = br#"interface Repeated {
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
            "each interface member name occurs once",
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
    let source = br#"interface Plain {
}

binding Invalid : Plain<i32> {
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
fn incompatible_and_out_of_order_bindings_point_at_the_fn_bind() {
    let source = br#"interface Pair {
  fn first() -> result: own i32 pure;
  fn second() -> result: own i32 pure;
}

fn make_first() -> result: own i32 pure {
  return 1_i32;
}

fn make_second() -> result: own i32 pure {
  return 2_i32;
}

binding Reversed : Pair {
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
            "binding member names follow the interface's declared order",
            "a nonmatching behavior argument",
        ),
        b"second = make_second;",
    );
}

#[test]
fn missing_binding_points_at_the_complete_actual_declaration() {
    let source = br#"interface Pair {
  fn first() -> result: own i32 pure;
  fn second() -> result: own i32 pure;
}

fn make_first() -> result: own i32 pure {
  return 1_i32;
}

binding Incomplete : Pair {
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
            "a binding group binds every interface member exactly once in declared order",
            "a nonmatching behavior argument",
        ),
        b"binding Incomplete : Pair {\n  first = make_first;\n}",
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
fn formal_row_comparison_uses_parameter_ordinals_not_binder_spellings() {
    // v0.59 compared positional regions here. [FN-4] still normalizes the two
    // rows by parameter ordinal before comparing them, and [EFF-1] now writes
    // one path per entry, so the formal's `x, y` and the actual's
    // `first, second` are the same row.
    let source = br#"interface LengthSum {
  fn sum(x: &Slots<u8, 4>, y: &Slots<u8, 4>) -> result: own u64 reads(x), reads(y);
}

fn add_lengths(first: &Slots<u8, 4>, second: &Slots<u8, 4>) -> result: own u64 reads(first), reads(second) {
  let first_length = deref(first).len;
  let second_length = deref(second).len;
  return first_length +wrap second_length;
}

binding Sum : LengthSum {
  sum = add_lengths;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("ordinal-normalized rows must check: {outcome:?}");
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
fn formal_range_reference_parameters_compare_by_ordinal() {
    // v0.59 wrote this operand as `own Slice<u8>`. `&[T]` is a reference kind
    // admitted only in parameter position [TYPE-8, REF-4]; the FN-4 ordinal
    // comparison over it is unchanged.
    let source = br#"interface ByteReader {
  fn first(values: &[u8]) -> result: own u8 reads(values);
}

fn read_first(bytes: &[u8]) -> result: own u8 reads(bytes) {
  let spare = deref(bytes).len;
  let ok = 0_u64 < spare;
  if ok {
    return deref(bytes)[0_u64];
  } else {
    return 0_u8;
  }
}

binding Bytes : ByteReader {
  first = read_first;
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("range-reference parameters must compare by ordinal: {outcome:?}");
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

// Retired with the slice return ceiling of [VIEW-6] and the borrow-mode
// result of [FN-1]: v0.59's `formal_slice_results_share_function_signature_
// formation` read `CheckedFunction::slice_return_ceiling` for an `own
// Slice<'r, u8>` result and refused a `&uniq 'descriptor Slice<..>` result
// with `BorrowedSliceResult`. v0.60 has no slice type and no borrow-mode
// result: `rtype := "own" type` leaves nothing for a signature to return but
// an owned value, and [REF-3]'s `EscapingReference` is the successor refusal
// for a body that tries to return a reference, kept as the conformance case
// `ref3-neg-returned-reference`.
