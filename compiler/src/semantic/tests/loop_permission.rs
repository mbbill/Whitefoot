//! The loop permission judgment over counted `for` statements.
//!
//! Each grant fixture is a shape a real program writes; each denial fixture
//! violates exactly one numbered condition and asserts *that* condition, so a
//! denial arriving for the wrong reason fails the test. The denials are
//! deliberately the bulk of the file: granting is the easy half, and the whole
//! risk of a rule that lets an implementation choose a combination tree is a
//! loop that should have been refused and was not. Design:
//! `research/investigations/proof-derived-parallelism/loop/DESIGN.md`.

use crate::{SemanticIssueKind, SemanticOutcome, SemanticRule};

use super::super::loop_permission::{
    LoopActualization, LoopCombine, LoopDenial, LoopPermission, LoopVerdict,
};
use super::super::permission::PermissionMetadata;
use super::{with_semantics, with_semantics_dark};

fn permission_of(source: &[u8]) -> PermissionMetadata {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("loop permission fixture must check: {outcome:?}");
        };
        program.data.permission.clone()
    })
}

fn dark_permission_of(source: &[u8]) -> PermissionMetadata {
    with_semantics_dark(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("dark loop permission fixture must check: {outcome:?}");
        };
        program.data.permission.clone()
    })
}

/// The only counted loop of one function. Every fixture below keeps its
/// interesting function to a single loop so the assertion cannot drift onto a
/// neighbour; the nesting fixtures name their loops by ordinal instead.
fn only_loop<'table>(table: &'table PermissionMetadata, name: &str) -> &'table LoopPermission {
    let judged = &loops(table, name);
    assert_eq!(
        judged.len(),
        1,
        "{name} must have exactly one counted loop: {judged:?}"
    );
    judged[0]
}

fn loops<'table>(table: &'table PermissionMetadata, name: &str) -> Vec<&'table LoopPermission> {
    table
        .named(name)
        .unwrap_or_else(|| panic!("no permission table for {name}"))
        .loops
        .iter()
        .collect()
}

/// The denial of one loop, asserted to cite the expected condition.
fn denial(judged: &LoopPermission, condition: u8) -> &LoopDenial {
    let LoopVerdict::Denied(denial) = &judged.verdict else {
        panic!("expected a denial, got {:?}", judged.verdict);
    };
    assert_eq!(
        denial.condition(),
        condition,
        "denied by the wrong condition: {denial:?}"
    );
    denial
}

fn denied(source: &[u8], function: &str, condition: u8) -> LoopDenial {
    let table = permission_of(source);
    denial(only_loop(&table, function), condition).clone()
}

fn permitted(source: &[u8], function: &str) -> LoopPermission {
    let table = permission_of(source);
    let judged = only_loop(&table, function).clone();
    assert_eq!(
        judged.verdict,
        LoopVerdict::PermittedEligible,
        "expected an eligible permitted loop"
    );
    judged
}

// ----------------------------------------------------------------------
// Grants
// ----------------------------------------------------------------------

/// The reduction: a counted loop over a pure callee, folding one accumulator
/// under `+wrap`. This is the shape the whole rule exists for — the escape
/// count of the grid family, written as the loop a writer reaches for first.
#[test]
fn a_counted_reduction_over_a_pure_callee_is_permitted_and_eligible() {
    let source = b"fn interesting(index: own u64) -> result: own Bool pure {
  let low = iand(index, 7_u64);
  let seen = 0_u64;
  loop @spin {
    let done = seen == 4_u64;
    if done {
      break @spin;
    }
    set seen = seen +wrap 1_u64;
  }
  return low == 3_u64;
}

command fn main() -> status: own ExitStatus pure {
  let hits = 0_u64;
  for @scan (i in 0_u64..4096_u64) {
    let escaped = interesting(index: i);
    if escaped {
      set hits = hits +wrap 1_u64;
    }
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(judged.combines, vec!["+wrap"]);
    assert!(
        !judged.advises_split,
        "a permitted loop needs no rewrite advice"
    );
}

/// A checked local invariant is erased before execution. It contributes no
/// read, write, loan, accumulator, or exit to the loop permission survey.
#[test]
fn a_local_invariant_in_the_body_has_no_runtime_footprint() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..4_u64) {
    invariant two_steps: 0_u64 <= 2_u64;
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
"#;
    let judged = permitted(source, "main");
    assert_eq!(judged.verdict, LoopVerdict::PermittedEligible);
    assert_eq!(judged.combines, vec!["+wrap"]);
    assert!(matches!(
        judged.actualization,
        Some(LoopActualization::Reduction {
            combine: LoopCombine::AddWrap,
            ..
        })
    ));
}

/// Every admitted combine reaches a grant, one loop each, and the verdict
/// names the operation the accumulator recombines under.
///
/// The set is closed and normative, so a widening that added an operation
/// without adding it here would leave the new one untested; a narrowing would
/// fail one of these outright.
#[test]
fn each_admitted_combine_permits_its_loop_and_is_named() {
    for (combine, initial, step) in [
        ("+wrap", "0_u64", "total +wrap i"),
        ("*wrap", "1_u64", "total *wrap i"),
        ("iand", "18446744073709551615_u64", "iand(total, i)"),
        ("ior", "0_u64", "ior(total, i)"),
        ("ixor", "0_u64", "ixor(total, i)"),
        ("imin", "18446744073709551615_u64", "imin(total, i)"),
        ("imax", "0_u64", "imax(total, i)"),
    ] {
        let source = format!(
            "command fn main() -> status: own ExitStatus pure {{
  let total = {initial};
  for @sum (i in 0_u64..16_u64) {{
    set total = {step};
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let judged = permitted(source.as_bytes(), "main");
        assert_eq!(judged.combines, vec![combine], "for {step}");
    }
    for (combine, initial, step) in [
        ("band", "True()", "band(every, bit)"),
        ("bor", "False()", "bor(every, bit)"),
        ("bxor", "False()", "bxor(every, bit)"),
    ] {
        let source = format!(
            "command fn main() -> status: own ExitStatus pure {{
  let every = {initial};
  for @scan (i in 0_u64..16_u64) {{
    let low = iand(i, 1_u64);
    let bit = low == 0_u64;
    set every = {step};
  }}
  return exit_status(code: 0_u8);
}}
"
        );
        let judged = permitted(source.as_bytes(), "main");
        assert_eq!(judged.combines, vec![combine], "for {step}");
    }
}

/// A loop that carries nothing at all is permitted, and the verdict says so
/// rather than naming an accumulator it does not have.
///
/// Such a loop computes nothing an enclosing scope can observe, which is why
/// disjointness alone is not a capability: it is the accumulator that makes a
/// permitted loop worth permitting.
#[test]
fn a_counted_loop_carrying_nothing_is_permitted_with_no_accumulator() {
    let source = b"fn work(x: own u64) -> result: own u64 pure {
  return x *wrap 3_u64;
}

command fn main() -> status: own ExitStatus pure {
  for @scan (i in 0_u64..16_u64) {
    let seen = work(x: i);
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert!(judged.combines.is_empty());
    assert_eq!(
        judged.actualization, None,
        "permission alone must not turn an unobservable stateless loop into a map"
    );
}

/// A callee that writes through a `&uniq` parameter into storage the
/// *iteration* introduced is permitted.
///
/// The row says the callee writes; the projection says what it writes, and a
/// place rooted in a binding the body opens is created fresh by every
/// iteration. Refusing every writing callee would leave any loop whose
/// iteration builds a scratch structure through a helper out of reach, and the
/// judgment holds the resolved places a diagnostic cannot rebuild.
#[test]
fn a_callee_writing_iteration_own_storage_is_permitted() {
    let source =
        b"fn bump['s](slot: &uniq 's u64, x: own u64) -> result: own u64 reads(slot), writes(slot) {
  set deref(slot) = deref(slot) +wrap x;
  return deref(slot);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let scratch = 0_u64;
    region 'acc {
      let got = bump::<'acc>(slot: &uniq 'acc scratch, x: i);
      set total = total +wrap got;
    }
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(judged.combines, vec!["+wrap"]);
}

/// A `replace` whose target the iteration introduced is permitted: the value
/// it reads out is this iteration's, not the previous one's.
#[test]
fn a_replace_of_iteration_own_storage_is_permitted() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  for @swap (i in 0_u64..8_u64) {
    let held = buffer_new(4_u64, 0_u64);
    let fresh = buffer_new(4_u64, i);
    let previous = replace held = move fresh;
  }
  return exit_status(code: 0_u8);
}
";
    permitted(source, "main");
}

/// Nested counted loops are judged on their own terms, and no rule joins two
/// index ranges into one iteration space.
///
/// The outer loop's accumulator is written inside the inner body; both loops
/// are permitted, because a split of either preserves the leaf order within
/// each part.
#[test]
fn nested_counted_loops_are_each_judged_on_their_own_terms() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @rows (r in 0_u64..8_u64) {
    for @cols (c in 0_u64..8_u64) {
      set total = total +wrap c;
    }
  }
  return exit_status(code: 0_u8);
}
";
    let table = permission_of(source);
    let judged = loops(&table, "main");
    assert_eq!(judged.len(), 2, "one verdict per loop: {judged:?}");
    for level in judged {
        assert_eq!(level.verdict, LoopVerdict::PermittedEligible);
        assert_eq!(level.combines, vec!["+wrap"]);
    }
}

/// One OP-4 outcome inside nested loops retains its affine image separately
/// for each active counted binder. Here the offset depends only on the outer
/// binder: overlapping outer iterations is sound, while overlapping inner
/// iterations would repeatedly write the same element and is denied.
#[test]
fn a_nested_map_is_granted_only_to_the_binder_in_its_retained_image() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(8_u64, 0_u64);
  for @rows (r in 0_u64..8_u64) {
    for @cols (c in 0_u64..4_u64) {
      set out[r] = c;
    }
  }
  return exit_status(code: 0_u8);
}
";
    let table = permission_of(source);
    let judged = loops(&table, "main");
    assert_eq!(judged.len(), 2);
    assert_eq!(judged[0].verdict, LoopVerdict::PermittedEligible);
    assert_eq!(
        judged[0].actualization,
        Some(LoopActualization::IndependentMap)
    );
    assert!(matches!(
        denial(judged[1], 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// An explicit proof cannot name an unavailable premise. PRF-1 rejects the
/// source before loop permission can observe the unproved subscript.
#[test]
fn an_unproved_source_premise_cannot_authorize_a_loop_subscript() {
    let source =
        br#"fn tally['s](src: &'s buffer<u64>, limit: own u64) -> result: own u64 reads(src) {
  let room = len(deref(src));
  invariant scaled_limit_fits: 4_u64 * limit <= 4_u64 * room {
    use 4 * (limit <= room);
  }
  let total = 0_u64;
  for @sum (i in 0_u64..limit) {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally::<'s>(src: &'s data, limit: 64_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unavailable proof premise must reject before permission: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Prf1);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedSourceProof { .. }
        ));
    });
}

/// A dominating branch establishes the same `limit <= len(src)` fact. The
/// loop remains eligible because permission reads the checked body footprint,
/// not the proof route for its subscript.
#[test]
fn a_dominating_bound_outside_the_loop_leaves_it_eligible() {
    let source =
        br#"fn tally['s](src: &'s buffer<u64>, limit: own u64) -> result: own u64 reads(src) {
  let room = len(deref(src));
  let total = 0_u64;
  let fits = limit <= room;
  if fits {
    for @sum (i in 0_u64..limit) {
      let v = deref(src)[i];
      set total = total +wrap v;
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally::<'s>(src: &'s data, limit: 64_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    let judged = permitted(source, "tally");
    assert_eq!(judged.combines, vec!["+wrap"]);
}

// ----------------------------------------------------------------------
// Condition 1: one accumulator, or none
// ----------------------------------------------------------------------

/// The float denial, and the reason the whole rule can exist.
///
/// `fadd.strict` is not associative, so an implementation that chose a
/// different combination tree would publish different bytes. The admitted set
/// is enumerated and contains no float, so this is refused outright rather
/// than hedged — and the denial cites the statement, which names the operation
/// the writer wrote.
#[test]
fn a_float_accumulator_is_denied_by_condition_one() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let step = 0.5_f64;
  for @sum (i in 0_u64..1024_u64) {
    set total = fadd.strict(total, step);
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 1),
        LoopDenial::NotAReduction { .. }
    ));

    // The identical loop over an integer accumulator is permitted, so the
    // refusal above is about the operation and not about the loop.
    let integral = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  let step = 5_u64;
  for @sum (i in 0_u64..1024_u64) {
    set total = total +wrap step;
  }
  return exit_status(code: 0_u8);
}
";
    assert_eq!(permitted(integral, "main").combines, vec!["+wrap"]);
}

/// An integer operation that is associative over the integers is still
/// refused when each application carries an obligation or a clamp that
/// regrouping moves. `+sat` is the pointed one: it is not even associative.
#[test]
fn a_saturating_accumulator_is_denied_by_condition_one() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let step = 1_u64;
    set total = total +sat step;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 1),
        LoopDenial::NotAReduction { .. }
    ));
}

/// A fold hidden behind a pure callee is refused: the combine must be a
/// syntactic operation of the admitted set, never a call result.
///
/// Without this the callee below folds `fadd.strict` one frame away, invisible
/// to a survey that reads the operation written in the body.
#[test]
fn a_fold_through_a_callee_is_denied_by_condition_one() {
    let source = b"fn blend(acc: own f64, x: own f64) -> result: own f64 pure {
  return fadd.strict(acc, x);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  for @sum (i in 0_u64..16_u64) {
    let step = 0.5_f64;
    set total = blend(acc: total, x: step);
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 1),
        LoopDenial::NotAReduction { .. }
    ));
}

/// A scan carries a value no operation combines: the write reads nothing of
/// the previous value, so which iteration wrote last would be observable.
#[test]
fn carried_state_that_is_no_reduction_is_denied_by_condition_one() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let prev = 0_u64;
  for @walk (i in 0_u64..16_u64) {
    set prev = i *wrap 3_u64;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 1),
        LoopDenial::NotAReduction { .. }
    ));
}

/// A `replace` of enclosing storage reads the previous value out [SET-2], so
/// the destination is carried state no operation combines.
#[test]
fn a_replace_of_enclosing_storage_is_denied_by_condition_one() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let held = buffer_new(4_u64, 0_u64);
  for @swap (i in 0_u64..8_u64) {
    let fresh = buffer_new(4_u64, i);
    let previous = replace held = move fresh;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 1),
        LoopDenial::NotAReduction { .. }
    ));
}

/// An accumulator read a second time makes the iterations order-dependent:
/// what the later read sees is the running total, which no split reproduces.
#[test]
fn an_accumulator_read_outside_its_combine_is_denied_by_condition_one() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let doubled = total +wrap i;
    set total = total +wrap doubled;
  }
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::AccumulatorRead { reads, .. } = denied(source, "main", 1) else {
        panic!("expected a read-count denial");
    };
    assert_eq!(reads, 2);
}

/// The read count walks the *subscript* of a write target too.
///
/// A running counter spelled in a subscript is a read of the running value
/// like any other. Today an [OP-4] bound obligation forces a dominating read
/// of the same counter, so this fixture carries three reads rather than two —
/// the walk is defence that the bound obligation happens to duplicate, and it
/// stops depending on that coincidence.
#[test]
fn an_accumulator_read_in_a_write_subscript_is_denied_by_condition_one() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let table = buffer_new(64_u64, 0_u64);
  let cursor = 0_u64;
  for @fill (i in 0_u64..8_u64) {
    let room = cursor < 64_u64;
    if room {
      set table[cursor] = i;
    }
    set cursor = cursor +wrap 1_u64;
  }
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::AccumulatorRead { reads, .. } = denied(source, "main", 1) else {
        panic!("expected a read-count denial");
    };
    assert_eq!(reads, 3, "the guard, the subscript, and the combine");
}

/// A borrow of the accumulator formed in the body: the body refuses every
/// borrow-forming statement outright, so a value read through such a holder
/// can never reach the count in the first place.
#[test]
fn a_borrow_of_the_accumulator_is_refused_as_a_borrow_forming_statement() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    region 'look {
      let view = &'look total;
      let seen = deref(view);
      let bumped = seen +wrap i;
    }
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::BodyForm {
            form: "a statement that forms a borrow of storage the iteration does not introduce"
        }
    ));

    // A borrow taken *after* the loop is outside the body, so the same
    // reduction stays permitted: the refusal is per body, never per function.
    let after = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    set total = total +wrap i;
  }
  region 'after {
    let view = &'after total;
    let seen = deref(view);
    let bumped = seen +wrap 1_u64;
  }
  return exit_status(code: 0_u8);
}
";
    permitted(after, "main");
}

/// Two accumulators are refused, and this is the one refusal the split advice
/// outlives: a hand-written recursion may return an aggregate.
#[test]
fn two_accumulators_are_denied_by_condition_one_and_keep_the_split_advice() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  let mask = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    set total = total +wrap i;
    set mask = ior(mask, i);
  }
  return exit_status(code: 0_u8);
}
";
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    let LoopDenial::ManyAccumulators { accumulators } = denial(judged, 1) else {
        panic!("expected an accumulator-count denial");
    };
    assert_eq!(*accumulators, 2);
    assert!(judged.advises_split, "the rewrite is still available");
    assert_eq!(judged.combines, vec!["+wrap", "ior"]);
}

/// The inner loop's endpoint reads the outer accumulator, so the outer loop
/// reads it twice and is refused while the inner one is not.
///
/// The inner range's endpoints are captured once before its body runs
/// [FN-1], so splitting the inner loop is sound whatever the outer one does.
#[test]
fn a_nested_endpoint_reading_the_accumulator_denies_only_the_outer_loop() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @rows (r in 0_u64..8_u64) {
    for @cols (c in 0_u64..total) {
      set total = total +wrap c;
    }
  }
  return exit_status(code: 0_u8);
}
";
    let table = permission_of(source);
    let judged = loops(&table, "main");
    assert_eq!(judged.len(), 2);
    assert!(matches!(
        denial(judged[0], 1),
        LoopDenial::AccumulatorRead { .. }
    ));
    assert_eq!(judged[1].verdict, LoopVerdict::PermittedEligible);
}

// ----------------------------------------------------------------------
// Condition 2: no shared writable footprint
// ----------------------------------------------------------------------

/// The smallest parallel map: OP-4 has already proved `i` is a valid index,
/// and [FN-1] gives a distinct compiler-owned `i` to every iteration. The loop
/// judgment consumes that successful obligation and treats each write as the
/// disjoint range `[i, i + 1)`.
#[test]
fn a_proven_counted_binder_element_map_is_permitted() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    set out[i] = i *wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert!(judged.combines.is_empty());
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// Permission consumes the offset's exact checked value rather than its
/// spelling. Copying the binder and applying one proved affine transform keeps
/// the nonzero coefficient, so distinct iterations still select distinct
/// elements.
#[test]
fn a_copied_affine_binder_element_map_is_permitted() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(128_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let copy = i;
    let slot = copy * 2_u64;
    set out[slot] = i;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// The permission result above must come from checked OP-4 evidence, not from
/// recognizing the source expression again. The retained outcome records the
/// exact coefficient and constant computed at that program point.
#[test]
fn op4_retains_the_affine_index_map_consumed_by_parallel_permission() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(128_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let copy = i;
    let slot = copy * 2_u64;
    set out[slot] = i;
  }
  return exit_status(code: 0_u8);
}
";
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(checked) = outcome else {
            panic!("the affine map fixture must check: {outcome:?}");
        };
        let main = checked
            .data
            .functions
            .iter()
            .find(|function| function.name == "main")
            .expect("main is checked");
        let maps = main
            .entailment
            .obligations
            .iter()
            .find(|outcome| outcome.family == super::super::entailment::ObligationFamily::Bounds)
            .expect("the mapped subscript has one OP-4 outcome")
            .affine_index_maps
            .as_slice();
        let [map] = maps else {
            panic!("the discharged OP-4 site must retain one active-loop image: {maps:?}");
        };
        assert_eq!(map.coefficient, 2);
        assert_eq!(map.constant, 0);
    });
}
/// A constant image has coefficient zero and is not injective. OP-4 proves
/// the element access itself, but PAR-2 correctly keeps the whole-root write.
#[test]
fn a_zero_coefficient_element_map_is_denied() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let slot = i - i;
    set out[slot] = i;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// Two injective maps do not automatically have disjoint images across
/// iterations. The fixed rule therefore requires every write site on one
/// mapped root to carry the same coefficient and constant.
#[test]
fn two_different_affine_maps_of_one_root_are_denied() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(128_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let even = i * 2_u64;
    let odd = even + 1_u64;
    set out[even] = i;
    set out[odd] = i;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// Repeating one mapped write is harmless: one iteration may update its own
/// element more than once, while the common map keeps every other iteration
/// on a distinct element.
#[test]
fn repeated_writes_with_the_same_affine_map_are_permitted() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(128_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let slot = i * 2_u64;
    set out[slot] = i;
    set out[slot] = i;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// Reading and writing the same proved affine element keeps every iteration
/// inside its own disjoint cell. The read-side OP-4 image is compared with the
/// write image; this is not treated as a whole-buffer dependence.
#[test]
fn a_same_index_read_modify_write_is_permitted() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u8);
  for @update (i in 0_u64..64_u64) {
    let old = out[i];
    let next = old +wrap 1_u8;
    set out[i] = next;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// A proved element read does not hide another occurrence that reaches the
/// whole mapped collection. The latter has no single-element range and keeps
/// condition 2 fail-closed.
#[test]
fn a_whole_collection_read_still_denies_a_same_map_update() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u8);
  for @update (i in 0_u64..64_u64) {
    let room = len(out);
    let old = out[i];
    set out[i] = old +wrap 1_u8;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// A writable unique buffer parameter is one exclusive source place. Once
/// OP-4 proves `i` is in range, the same affine-map rule can divide that place
/// into disjoint per-iteration elements; ownership does not require copying
/// the buffer into the callee.
#[test]
fn a_unique_borrowed_output_accepts_a_proved_element_map() {
    let source = br#"fn fill['r](out: &uniq 'r buffer<u8>, count: own u64) -> result: own unit writes(out) contract {
  define room = len(deref(out));
  requires count <= room;
} {
  for @fill (i in 0_u64..count) {
    set deref(out)[i] = 1_u8;
  }
  return unit;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u8);
  region 'call {
    let filled = fill::<'call>(out: &uniq 'call out, count: 64_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    let judged = permitted(source, "fill");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// The common-map requirement is per resolved collection. Ownership keeps two
/// distinct roots disjoint, so each may use its own injective affine image.
#[test]
fn different_owned_roots_may_use_different_affine_maps() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let evens = buffer_new(128_u64, 0_u64);
  let shifted = buffer_new(65_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let even = i * 2_u64;
    let next = i + 1_u64;
    set evens[even] = i;
    set shifted[next] = i;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// Two collection fields share one source root binding but resolve to
/// disjoint places. Their read-modify-write maps are therefore checked per
/// resolved collection rather than being mixed together by the struct
/// binding that happens to contain them.
#[test]
fn sibling_collection_roots_may_read_and_write_their_own_maps() {
    let source = b"struct Columns {
  left: array<u64, 64>;
  right: array<u64, 64>;
}

command fn main() -> status: own ExitStatus pure {
  let left = array_new::<u64, 64>(0_u64);
  let right = array_new::<u64, 64>(0_u64);
  let columns = Columns(left: move left, right: move right);
  for @update (i in 0_u64..63_u64) {
    let next = i + 1_u64;
    let old_left = columns.left[i];
    set columns.left[i] = old_left +wrap 1_u64;
    let old_right = columns.right[next];
    set columns.right[next] = old_right +wrap 1_u64;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}
/// A map may coexist with one admitted reduction. The map supplies the
/// disjoint writes while the real accumulator supplies the recombination
/// payload; no synthetic map accumulator is introduced.
#[test]
fn an_exact_map_with_a_reduction_uses_reduction_actualization() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  let total = 0_u64;
  for @fill (i in 0_u64..64_u64) {
    set out[i] = i *wrap i;
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(judged.combines, vec!["+wrap"]);
    assert!(matches!(
        judged.actualization,
        Some(LoopActualization::Reduction {
            combine: LoopCombine::AddWrap,
            ..
        })
    ));
}

/// An explicit proof with an unavailable premise cannot make an unproved write
/// available to affine-map permission. PRF-1 rejects it first.
#[test]
fn an_unproved_source_premise_is_rejected_before_affine_map_permission() {
    let source = br#"fn fill(output: own buffer<u64>, limit: own u64) -> result: own buffer<u64> reads(output), writes(output) {
  let room = len(output);
  invariant limit_fits: limit <= room {
    use limit <= room;
  }
  for @fill (i in 0_u64..limit) {
    set output[i] = i;
  }
  return move output;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let output = buffer_new(64_u64, 0_u64);
  let filled = fill(output: move output, limit: 64_u64);
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unavailable proof premise must reject before permission: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Prf1);
        assert!(matches!(
            issue.kind(),
            SemanticIssueKind::UndischargedSourceProof { .. }
        ));
    });
}

/// A shared call loan is harmless beside another shared call, but not beside
/// another iteration's write to the same mapped root. The loan is checked
/// explicitly rather than relying on the argument also appearing as a read.
#[test]
fn a_shared_call_loan_on_the_mapped_root_is_denied() {
    let source = b"fn observe['r](value: &'r buffer<u64>) -> result: own unit pure {
  return unit;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    region 'look {
      let seen = observe::<'look>(value: &'look out);
    }
    set out[i] = i;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(denied(source, "main", 2), LoopDenial::Loan { .. }));
}

/// An exact affine value is insufficient when OP-4 cannot prove the subscript
/// is inside the collection. The dark entry lets this test inspect that
/// fail-closed permission verdict without changing ordinary source acceptance.
#[test]
fn an_unproved_counted_binder_element_map_remains_denied() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let slot = i + 1_u64;
    set out[slot] = i;
  }
  return exit_status(code: 0_u8);
}
";
    let table = dark_permission_of(source);
    assert!(matches!(
        denial(only_loop(&table, "main"), 2),
        LoopDenial::SharedWrite { .. }
    ));
    assert_eq!(only_loop(&table, "main").actualization, None);
}

/// A non-injective index map is refused for the same reason and by the same
/// condition, so nothing about the refusal depends on the index expression
/// being distinguishable.
#[test]
fn a_non_injective_element_write_is_denied_by_condition_two() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let slot = iand(i, 7_u64);
    set out[slot] = i;
  }
  return exit_status(code: 0_u8);
}
";
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    assert!(matches!(denial(judged, 2), LoopDenial::SharedWrite { .. }));
    assert_eq!(judged.actualization, None);
}

/// A stencil writes one element and reads another of the same buffer, which is
/// a dependence across iterations. Its read and write maps differ, so the
/// fixed same-map refinement refuses it.
#[test]
fn a_stencil_is_denied_by_condition_two() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 1_u64);
  for @fill (i in 1_u64..64_u64) {
    let prior = i -wrap 1_u64;
    set out[i] = out[prior];
  }
  return exit_status(code: 0_u8);
}
";
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    assert!(matches!(denial(judged, 2), LoopDenial::SharedWrite { .. }));
    assert_eq!(judged.actualization, None);
}

/// A callee that writes *caller* storage carries state across iterations
/// exactly as a `set` in the body does, and the combine is whatever its own
/// body performs — which can be a float fold one frame away. The projection
/// onto the actual is what tells this apart from the iteration-own grant.
#[test]
fn a_callee_writing_enclosing_storage_is_denied_by_condition_two() {
    let source =
        b"fn accum['s](slot: &uniq 's f64, x: own f64) -> result: own u64 reads(slot), writes(slot) {
  set deref(slot) = fadd.strict(deref(slot), x);
  let bits = reinterpret::<f64, u64>(deref(slot));
  return iand(bits, 1_u64);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let count = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    region 'acc {
      let one = accum::<'acc>(slot: &uniq 'acc total, x: 0.5_f64);
      set count = count +wrap one;
    }
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(denied(source, "main", 2), LoopDenial::Loan { .. }));
}

/// An expression statement is a call whose reach no row projects onto an
/// actual, and a discarded one carries its own [STOR-3] release. Neither has a
/// footprint this judgment computes, so both refuse — and the refusal is
/// reported ahead of the numbered conditions, because nothing else about the
/// statement is known.
#[test]
fn an_expression_statement_in_the_body_is_denied_by_condition_two() {
    let source = b"fn work(x: own u64) -> result: own u64 pure {
  return x *wrap 3_u64;
}

command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..4_u64) {
    work(x: i);
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::BodyForm { .. }
    ));
}

// ----------------------------------------------------------------------
// Completion actualization boundary
// ----------------------------------------------------------------------

/// A wrapper which deliberately keeps a unique factory loan across a
/// may-suspend open cannot overlap across loop iterations. This is an ordinary
/// loan consequence of that wrapper signature, not a property of the system
/// open API, whose permit and directory inputs are independent.
#[test]
fn a_may_suspend_directory_wrapper_keeps_its_unique_loan() {
    let source = b"fn probe['f, 'c](factory: &uniq 'f FileFactory, root: &'c DirectoryRead) -> result: own u64 reads(factory, root), writes(factory) {
  region 'reserve {
    match reserve_file::<'f>(factory: move factory) {
      Ok(value: permit) => {
        match open_directory_source::<'c>(permit: move permit, directory: root) {
          SourceOpened(value: listing) => {
            return 1_u64;
          }
          SourceOpenFailed(error: refused, permit: refused_2) => {
            return 0_u64;
          }
        }
      }
      Err(error: spent) => {
        return 0_u64;
      }
    }
  }
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let total = 0_u64;
  for @scan (i in 0_u64..4_u64) {
    region 'probe_call {
      let seen = probe::<'probe_call, 'probe_call>(factory: &uniq 'probe_call files, root: &'probe_call cwd);
      set total = total +wrap seen;
    }
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(denied(source, "main", 2), LoopDenial::Loan { .. }));
}

/// A direct advancing Source operation follows the same boundary. Its unique
/// Source and destination loans, rather than a system-only relation, prevent
/// loop-iteration overlap.
#[test]
fn a_direct_directory_state_transition_keeps_its_unique_loan() {
    let source = b"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let destination = buffer_new(1_u64, 0_u8);
  region 'open {
    match reserve_file::<'open>(factory: &uniq 'open files) {
      Ok(value: permit) => {
        match open_directory_source::<'open>(permit: move permit, directory: &'open cwd) {
          SourceOpened(value: listing) => {
            let total = 0_u64;
            for @scan (i in 0_u64..4_u64) {
              region 'attempt {
                let outcome = directory_next::<'attempt, 'attempt>(source: &uniq 'attempt listing, destination: &uniq 'attempt destination, start: 0_u64, end: 1_u64);
              }
              set total = total +wrap 1_u64;
            }
          }
          SourceOpenFailed(error: refused, permit: refused_2) => {
          }
        }
      }
      Err(error: spent) => {
        return exit_status(code: 8_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(denied(source, "main", 2), LoopDenial::Loan { .. }));
}

// ----------------------------------------------------------------------
// Condition 4: no exit edge
// ----------------------------------------------------------------------

/// A `break` that closes the judged loop skips the rest of the range, so the
/// set of iterations is no longer the whole range.
#[test]
fn a_break_out_of_the_loop_is_denied_by_condition_four() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let stop = i == 9_u64;
    if stop {
      break @sum;
    }
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::Exit { edge } = denied(source, "main", 4) else {
        panic!("expected an exit denial");
    };
    assert_eq!(edge, "a break");
}

/// A `break` naming an *enclosing* loop leaves this one too, while a `break`
/// naming a loop opened inside the body does not — which is the distinction
/// the loop identity carries.
#[test]
fn a_break_to_an_enclosing_loop_is_denied_while_an_inner_break_is_not() {
    let outward = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  loop @outer {
    for @sum (i in 0_u64..16_u64) {
      let stop = i == 9_u64;
      if stop {
        break @outer;
      }
      set total = total +wrap i;
    }
    break @outer;
  }
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::Exit { edge } = denied(outward, "main", 4) else {
        panic!("expected an exit denial");
    };
    assert_eq!(edge, "a break");

    let inward = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let seen = 0_u64;
    loop @inner {
      set seen = seen +wrap 1_u64;
      let done = seen == 4_u64;
      if done {
        break @inner;
      }
    }
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    permitted(inward, "main");
}

/// A `return` in the body leaves the loop and the function.
#[test]
fn a_return_in_the_body_is_denied_by_condition_four() {
    let source = b"fn walk(n: own u64) -> result: own u64 pure {
  let total = 0_u64;
  for @sum (i in 0_u64..n) {
    let hit = i == 3_u64;
    if hit {
      return total;
    }
    set total = total +wrap i;
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  let seen = walk(n: 9_u64);
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::Exit { edge } = denied(source, "walk", 4) else {
        panic!("expected an exit denial");
    };
    assert_eq!(edge, "a return");
}

/// A `give` that delivers into a value initializer the *body* opens leaves
/// nothing, so the loop is permitted.
///
/// `give` reaches the innermost value initializer enclosing it [GIVE-1]. When
/// the writer puts a `value_if` inside the loop body, its arms deliver to a
/// binding of this same iteration; refusing every loop that contains one would
/// cost the shape a writer reaches for whenever an iteration's contribution
/// depends on a test. The next case is the same statement one level out, where
/// it does leave.
#[test]
fn a_give_delivering_inside_the_body_is_permitted() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let low = iand(i, 1_u64);
    let even = low == 0_u64;
    let weight = if even {
      give 3_u64;
    } else {
      give 5_u64;
    }
    set total = total +wrap weight;
  }
  return exit_status(code: 0_u8);
}
";
    assert_eq!(permitted(source, "main").combines, vec!["+wrap"]);
}

/// A `give` leaves the loop *and* the enclosing value initializer, and a
/// combination tree over the whole range has no representation for that edge:
/// it would fold every iteration where the loop stopped at the first hit.
#[test]
fn a_give_in_the_body_is_denied_by_condition_four() {
    let source =
        b"fn scan_until['s](src: &'s buffer<u64>, needle: own u64) -> result: own u64 reads(src) {
  let count = len(deref(src));
  let acc = 0_u64;
  let always = True();
  let answer = if always {
    for @scan (i in 0_u64..count) {
      let v = deref(src)[i];
      set acc = acc +wrap v;
      let hit = v == needle;
      if hit {
        give i;
      }
    }
    give 4096_u64;
  } else {
    give 4096_u64;
  }
  return answer +wrap acc;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  set data[10_u64] = 7_u64;
  region 's {
    let t = scan_until::<'s>(src: &'s data, needle: 7_u64);
    return exit_status(code: 0_u8);
  }
}
";
    let LoopDenial::Exit { edge } = denied(source, "scan_until", 4) else {
        panic!("expected an exit denial");
    };
    assert_eq!(edge, "a give");

    // The same loop with the give removed is permitted, so the refusal is
    // about the edge and not about the shape.
    let contained =
        b"fn scan_until['s](src: &'s buffer<u64>, needle: own u64) -> result: own u64 reads(src) {
  let count = len(deref(src));
  let acc = 0_u64;
  let always = True();
  let answer = if always {
    for @scan (i in 0_u64..count) {
      let v = deref(src)[i];
      set acc = acc +wrap v;
    }
    give 4096_u64;
  } else {
    give 4096_u64;
  }
  return answer +wrap acc;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  set data[10_u64] = 7_u64;
  region 's {
    let t = scan_until::<'s>(src: &'s data, needle: 7_u64);
    return exit_status(code: 0_u8);
  }
}
";
    assert_eq!(permitted(contained, "scan_until").combines, vec!["+wrap"]);
}

/// A propagating `let` carries an `Err` edge to the function-return sink
/// [ERR-3], which leaves the loop on the failing iteration.
#[test]
fn a_propagate_in_the_body_is_denied_by_condition_four() {
    let source = b"fn narrow(v: own u64) -> result: own Result<u32, NarrowError> pure {
  return cvt::<u64, u32>(v);
}

fn tally(n: own u64) -> result: own Result<u64, NarrowError> pure {
  let total = 0_u64;
  for @sum (i in 0_u64..n) {
    let small = propagate narrow(v: i);
    set total = total +wrap i;
  }
  return Ok<u64, NarrowError>(value: total);
}

command fn main() -> status: own ExitStatus pure {
  let outcome = tally(n: 8_u64);
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::Exit { edge } = denied(source, "tally", 4) else {
        panic!("expected an exit denial");
    };
    assert_eq!(edge, "a propagate");
}

// ----------------------------------------------------------------------
// Proof-relevant operations in the body and call closure
// ----------------------------------------------------------------------

/// The fixed remainder interval proves the following subscript before loop
/// permission inspects the body. The proof adds no runtime operation and the
/// independent reduction remains eligible.
#[test]
fn an_automatic_remainder_bound_in_the_body_preserves_reduction_permission() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 8>(0_u8);
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let bounded = i % 8_u64;
    let picked = values[bounded];
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
"#;
    let judged = permitted(source, "main");
    assert_eq!(judged.combines, vec!["+wrap"]);
    assert!(matches!(
        judged.actualization,
        Some(LoopActualization::Reduction {
            combine: LoopCombine::AddWrap,
            ..
        })
    ));
}

/// A dominating branch is the control-flow proof route. Its subscript checks,
/// and the loop's independent reduction remains eligible and actualized.
#[test]
fn a_branch_proved_subscript_in_the_body_is_permitted() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 8>(0_u8);
  let size = len(values);
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let bounded = imin(i, 7_u64);
    let inside = bounded < size;
    if inside {
      let picked = values[bounded];
    }
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    assert_eq!(judged.verdict, LoopVerdict::PermittedEligible);
    assert_eq!(judged.combines, vec!["+wrap"]);
    assert!(matches!(
        judged.actualization,
        Some(LoopActualization::Reduction {
            combine: LoopCombine::AddWrap,
            ..
        })
    ));
}

/// An accumulator-indexed subscript with no dominating fact is rejected before
/// permission. This keeps the negative boundary entirely in source semantics.
#[test]
fn an_unproved_accumulator_subscript_is_rejected_before_permission() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 128>(0_u8);
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let picked = values[total];
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unproved accumulator subscript must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
    });
}

/// In the accepted replacement, the ordinary guard and guarded subscript each
/// read the accumulator beside its combine. Condition 1 must count all three
/// reads and refuse the reduction.
#[test]
fn a_guard_reading_the_accumulator_is_still_a_read() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let values = array_new::<u8, 128>(0_u8);
  let size = len(values);
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let inside = total < size;
    if inside {
      let picked = values[total];
    }
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    let LoopVerdict::Denied(LoopDenial::AccumulatorRead { reads, .. }) = &judged.verdict else {
        panic!(
            "a guard and guarded subscript reading the accumulator must refuse, got {:?}",
            judged.verdict
        );
    };
    assert_eq!(
        *reads, 3,
        "the guard read and its guarded subscript both count beside the combine's"
    );
}

/// A callee with an unproved subscript is rejected before loop permission can
/// treat the call closure as complete.
#[test]
fn an_unproved_subscript_in_the_call_closure_is_rejected() {
    let source = br#"fn narrow(v: own u64) -> result: own u64 pure {
  let values = array_new::<u64, 8>(1_u64);
  let bounded = imin(v, 7_u64);
  return values[bounded];
}

command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let got = narrow(v: i);
    set total = total +wrap got;
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("an unproved callee subscript must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
    });
}

/// A callee whose branch proves its own subscript is a normal pure call in the
/// body. It contributes no shared write and does not block reduction
/// actualization.
#[test]
fn a_proof_complete_call_closure_is_permitted() {
    let source = br#"fn narrow(v: own u64) -> result: own u64 pure {
  let values = array_new::<u64, 8>(1_u64);
  let size = len(values);
  if v < size {
    return values[v];
  }
  return 0_u64;
}

command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let got = narrow(v: i);
    set total = total +wrap got;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    assert_eq!(judged.verdict, LoopVerdict::PermittedEligible);
    assert!(
        judged.actualization.is_some(),
        "a permitted loop over a proof-complete callee still carries its fold"
    );
}

// ----------------------------------------------------------------------
// The fact-state invariant
// ----------------------------------------------------------------------

/// One permission table whatever the entailment fact state derives.
///
/// The judgment consults typing, rows, resolved places, exit edges, and the
/// call graph, and never a derived fact — so facts-on and facts-off
/// compilation produce the same verdicts by construction. The compiler has no
/// facts-off switch to run a program through twice, so the differential is
/// over *programs*: accepted loops discharge the same subscript from the
/// counted binder's own bounds, a checked local invariant, a branch dominating
/// the whole loop, or a branch local to the iteration. Their verdicts must be
/// identical.
#[test]
fn the_loop_verdict_is_the_same_under_every_route_to_the_same_fact() {
    let structural = b"fn tally['s](src: &'s buffer<u64>) -> result: own u64 reads(src) {
  let count = len(deref(src));
  let total = 0_u64;
  for @sum (i in 0_u64..count) {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally::<'s>(src: &'s data);
  }
  return exit_status(code: 0_u8);
}
";
    let invariant_source =
        br#"fn tally['s](src: &'s buffer<u64>, bounded_limit: own u64, limit: own u64) -> result: own u64 reads(src) contract {
  define capacity = len(deref(src));
  requires bounded_limit <= limit;
  requires limit <= capacity;
} {
  let room = len(deref(src));
  invariant limit_fits: bounded_limit <= room;
  let total = 0_u64;
  for @sum (i in 0_u64..bounded_limit) {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally::<'s>(src: &'s data, bounded_limit: 64_u64, limit: 64_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    let dominating =
        b"fn tally['s](src: &'s buffer<u64>, limit: own u64) -> result: own u64 reads(src) {
  let room = len(deref(src));
  let total = 0_u64;
  if limit <= room {
    for @sum (i in 0_u64..limit) {
      let v = deref(src)[i];
      set total = total +wrap v;
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally::<'s>(src: &'s data, limit: 64_u64);
  }
  return exit_status(code: 0_u8);
}
";
    let branched =
        b"fn tally['s](src: &'s buffer<u64>, limit: own u64) -> result: own u64 reads(src) {
  let room = len(deref(src));
  let total = 0_u64;
  for @sum (i in 0_u64..limit) {
    let inside = i < room;
    if inside {
      let v = deref(src)[i];
      set total = total +wrap v;
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally::<'s>(src: &'s data, limit: 64_u64);
  }
  return exit_status(code: 0_u8);
}
";
    let verdicts = [
        structural.as_slice(),
        invariant_source,
        dominating.as_slice(),
        branched.as_slice(),
    ]
    .map(|source| {
        let table = permission_of(source);
        let judged = only_loop(&table, "tally");
        (judged.verdict.clone(), judged.combines.clone())
    });
    assert_eq!(
        verdicts[0],
        (LoopVerdict::PermittedEligible, vec!["+wrap"]),
        "the structurally bounded loop is permitted"
    );
    assert_eq!(
        verdicts[0], verdicts[1],
        "a checked local invariant moves no verdict"
    );
    assert_eq!(
        verdicts[0], verdicts[2],
        "a loop-dominating branch moves no verdict"
    );
    assert_eq!(
        verdicts[0], verdicts[3],
        "a branch-established bound moves no verdict"
    );
}

/// The loans half of condition 2: every iteration takes `&uniq` of one outer
/// cell while the callee's row declares `reads` only, so the written half is
/// empty and the old judgment permitted — and actually split — a loop whose
/// iterations each hold an exclusive loan on the one place. The loan, not a
/// write, is what denies it.
#[test]
fn a_read_only_unique_borrow_of_outer_storage_is_denied_by_its_loan() {
    let source = br#"fn peek_uniq['c](cell: &uniq 'c u64) -> result: own u64 reads(cell) {
  return deref(cell);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 21_u64;
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    region 'i {
      let v = peek_uniq::<'i>(cell: &uniq 'i cell);
      set acc = acc +wrap v;
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert!(matches!(denied(source, "main", 2), LoopDenial::Loan { .. }));
}

/// The shared control for the loan above: a `&'i` borrow of the same outer
/// cell holds a shared loan, which coexists with itself and conflicts only
/// with writes, so the loop stays permitted. This is the boundary the loans
/// half must not cross — read-only sharing across iterations is the point of
/// a reduction.
#[test]
fn a_shared_borrow_of_outer_storage_stays_permitted() {
    let source = br#"fn peek['c](cell: &'c u64) -> result: own u64 reads(cell) {
  return deref(cell);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 21_u64;
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    region 'i {
      let v = peek::<'i>(cell: &'i cell);
      set acc = acc +wrap v;
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    permitted(source, "main");
}

/// A bare borrow bound in the body: no call carries it, so no parameter mode
/// states its loan, and the checked tree erases whether it is shared or
/// exclusive. The body refuses as a form — before this refusal existed, every
/// iteration of this loop held what the source spells as an exclusive borrow
/// of the one outer cell, and the loop was permitted and split.
#[test]
fn a_body_statement_forming_a_borrow_is_refused() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let cell = 21_u64;
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    region 'i {
      let g = &uniq 'i cell;
      let v = deref(g);
      set acc = acc +wrap v;
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::BodyForm {
            form: "a statement that forms a borrow of storage the iteration does not introduce"
        }
    ));
}

/// The admissible side of the body borrow guard: a `let`-bound borrow of
/// storage the iteration itself creates. Each iteration borrows its own
/// instance, so no loan is needed and the loop stays permitted — the guard
/// refuses borrows of enclosing storage, not borrowing as such.
#[test]
fn a_body_borrow_of_iteration_own_storage_stays_permitted() {
    let source = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    let local = buffer_new(4_u64, 7_u8);
    region 'r {
      let h = &'r local;
      let v = len(deref(h));
      set acc = acc +wrap v;
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    permitted(source, "main");
}

/// The knowingly-denied shape: a shared borrow of enclosing storage bound in
/// the body. Sequentially sound and read-only, but the checked tree erases
/// the borrow's shared-or-uniq mode, so the guard cannot tell it from an
/// exclusive one and fails closed. Restoring it needs the mode carried into
/// the checked borrow forms, recorded as future work in the batch record.
#[test]
fn a_body_shared_borrow_of_outer_storage_is_knowingly_denied() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let shared = 21_u64;
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    region 'r {
      let h = &'r shared;
      let v = deref(h);
      set acc = acc +wrap v;
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::BodyForm {
            form: "a statement that forms a borrow of storage the iteration does not introduce"
        }
    ));
}
