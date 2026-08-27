//! The loop permission judgment over counted `for` statements.
//!
//! Each grant fixture is a shape a real program writes; each denial fixture
//! violates exactly one numbered condition and asserts *that* condition, so a
//! denial arriving for the wrong reason fails the test. The denials are
//! deliberately the bulk of the file: granting is the easy half, and the whole
//! risk of a rule that lets an implementation choose a combination tree is a
//! loop that should have been refused and was not. Design:
//! `research/investigations/proof-derived-parallelism/loop/DESIGN.md`.

use crate::SemanticOutcome;

use super::super::loop_permission::{LoopCombine, LoopDenial, LoopPermission, LoopVerdict};
use super::super::permission::PermissionMetadata;
use super::with_semantics;

fn permission_of(source: &[u8]) -> PermissionMetadata {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("loop permission fixture must check: {outcome:?}");
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
    let done = ieq(seen, 4_u64);
    if done {
      break @spin;
    }
    set seen = seen +wrap 1_u64;
  }
  return ieq(low, 3_u64);
}

command fn main() -> status: own ExitStatus pure {
  let hits = 0_u64;
  for @scan i in 0_u64..4096_u64 {
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
  for @sum i in 0_u64..16_u64 {{
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
  for @scan i in 0_u64..16_u64 {{
    let low = iand(i, 1_u64);
    let bit = ieq(low, 0_u64);
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
  for @scan i in 0_u64..16_u64 {
    let seen = work(x: i);
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert!(judged.combines.is_empty());
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
  for @sum i in 0_u64..16_u64 {
    let scratch = 0_u64;
    region 'acc {
      let got = bump<'acc>(slot: &uniq 'acc scratch, x: i);
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
  for @swap i in 0_u64..8_u64 {
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
  for @rows r in 0_u64..8_u64 {
    for @cols c in 0_u64..8_u64 {
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

/// A `claim` written in the enclosing function but outside the loop leaves the
/// loop eligible.
///
/// Eligibility asks about the body and what the body calls. A claim the writer
/// put before the loop has already been executed when the first iteration
/// runs, so it is no trap site an overlapped schedule could select between.
#[test]
fn a_claim_outside_the_loop_leaves_it_eligible() {
    let source = br#"fn tally['s](src: &'s buffer<u64>, limit: own u64) -> result: own u64 reads(src), traps {
  let room = len(deref(src));
  let bounded_limit = imin(limit, room);
  let fits = ile(bounded_limit, room);
  claim limit_fits: fits because "premises: bounded_limit is the minimum of the requested limit and room, and room is the input buffer's length\nderivation: a minimum is at most either operand, so bounded_limit is at most room\nconclusion: ile(bounded_limit, room) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the counted range below runs to bounded_limit and subscripts the input at its binder";
  let total = 0_u64;
  for @sum i in 0_u64..bounded_limit {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally<'s>(src: &'s data, limit: 64_u64);
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
  for @sum i in 0_u64..1024_u64 {
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
  for @sum i in 0_u64..1024_u64 {
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
  for @sum i in 0_u64..16_u64 {
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
  for @sum i in 0_u64..16_u64 {
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
  for @walk i in 0_u64..16_u64 {
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
  for @swap i in 0_u64..8_u64 {
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
  for @sum i in 0_u64..16_u64 {
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
  for @fill i in 0_u64..8_u64 {
    let room = ilt(cursor, 64_u64);
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
  for @sum i in 0_u64..16_u64 {
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
  for @sum i in 0_u64..16_u64 {
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
  for @sum i in 0_u64..16_u64 {
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
  for @rows r in 0_u64..8_u64 {
    for @cols c in 0_u64..total {
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

/// The parallel map. Two iterations write two elements of one buffer, and a
/// resolved place carries no index segment [ENT-2], so the judgment reads them
/// as one place and fails closed. This is the deferred capability, refused
/// rather than granted.
#[test]
fn an_element_write_into_enclosing_storage_is_denied_by_condition_two() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill i in 0_u64..64_u64 {
    set out[i] = i *wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// A non-injective index map is refused for the same reason and by the same
/// condition, so nothing about the refusal depends on the index expression
/// being distinguishable.
#[test]
fn a_non_injective_element_write_is_denied_by_condition_two() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 0_u64);
  for @fill i in 0_u64..64_u64 {
    let slot = iand(i, 7_u64);
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

/// A stencil writes one element and reads another of the same buffer, which is
/// a dependence across iterations. The write alone already refuses it.
#[test]
fn a_stencil_is_denied_by_condition_two() {
    let source = b"command fn main() -> status: own ExitStatus allocates(heap) {
  let out = buffer_new(64_u64, 1_u64);
  for @fill i in 1_u64..64_u64 {
    let prior = i -wrap 1_u64;
    set out[i] = out[prior];
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
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
  let bits = reinterpret<f64, u64>(deref(slot));
  return iand(bits, 1_u64);
}

command fn main() -> status: own ExitStatus pure {
  let total = 0.0_f64;
  let count = 0_u64;
  for @sum i in 0_u64..8_u64 {
    region 'acc {
      let one = accum<'acc>(slot: &uniq 'acc total, x: 0.5_f64);
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
  for @sum i in 0_u64..4_u64 {
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
    let permit = reserve_file<'f>(factory: move factory);
    match open_directory_source<'c>(permit: move permit, directory: root) {
      Ok(value: listing) => {
        return 1_u64;
      }
      Err(error: refused) => {
        return 0_u64;
      }
    }
  }
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  let total = 0_u64;
  for @scan i in 0_u64..4_u64 {
    region 'probe_call {
      let seen = probe<'probe_call, 'probe_call>(factory: &uniq 'probe_call files, root: &'probe_call cwd);
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
    let permit = reserve_file<'open>(factory: &uniq 'open files);
    match open_directory_source<'open>(permit: move permit, directory: &'open cwd) {
      Ok(value: listing) => {
        let total = 0_u64;
        for @scan i in 0_u64..4_u64 {
          region 'attempt {
            let outcome = directory_next<'attempt, 'attempt>(source: &uniq 'attempt listing, destination: &uniq 'attempt destination, start: 0_u64, end: 1_u64);
          }
          set total = total +wrap 1_u64;
        }
      }
      Err(error: refused) => {
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
  for @sum i in 0_u64..16_u64 {
    let stop = ieq(i, 9_u64);
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
    for @sum i in 0_u64..16_u64 {
      let stop = ieq(i, 9_u64);
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
  for @sum i in 0_u64..16_u64 {
    let seen = 0_u64;
    loop @inner {
      set seen = seen +wrap 1_u64;
      let done = ieq(seen, 4_u64);
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
  for @sum i in 0_u64..n {
    let hit = ieq(i, 3_u64);
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
  for @sum i in 0_u64..16_u64 {
    let low = iand(i, 1_u64);
    let even = ieq(low, 0_u64);
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
    for @scan i in 0_u64..count {
      let v = deref(src)[i];
      set acc = acc +wrap v;
      let hit = ieq(v, needle);
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
    let t = scan_until<'s>(src: &'s data, needle: 7_u64);
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
    for @scan i in 0_u64..count {
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
    let t = scan_until<'s>(src: &'s data, needle: 7_u64);
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
  return cvt<u64, u32>(v);
}

fn tally(n: own u64) -> result: own Result<u64, NarrowError> pure {
  let total = 0_u64;
  for @sum i in 0_u64..n {
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
// Claims in the body and in its call closure
// ----------------------------------------------------------------------

/// A `claim` written in the loop body is an ordinary statement here. It writes
/// nothing, and it leaves the loop only when it is false — an erroneous
/// execution, whose narrowed guarantee the trap latch meets. The loop is
/// permitted, and its accumulator still carries the fold.
#[test]
fn a_claim_in_the_body_is_permitted() {
    let source = br#"command fn main() -> status: own ExitStatus traps {
  let values = array_new<u8, 8>(0_u8);
  let size = len(values);
  let total = 0_u64;
  for @sum i in 0_u64..16_u64 {
    let bounded = imin(i, 7_u64);
    let inside = ilt(bounded, size);
    claim index_small: inside because "premises: bounded is the minimum of the counted binder i and seven, and values has length eight\nderivation: a minimum is at most either operand, so bounded is at most seven and therefore below eight\nconclusion: ilt(bounded, size) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the following length-eight array subscript uses bounded";
    let picked = values[bounded];
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    assert_eq!(judged.verdict, LoopVerdict::PermittedEligible);
    assert_eq!(judged.combines, vec!["+wrap"]);
    let actualization = judged
        .actualization
        .as_ref()
        .expect("a permitted reduction carries its accumulator");
    assert_eq!(actualization.combine, LoopCombine::AddWrap);
}

/// The claim's predicate is still read like any other expression, so a claim
/// that reads the accumulator is still a second read of it and condition 1
/// still refuses. Admitting the statement did not stop counting what it reads.
#[test]
fn a_claim_reading_the_accumulator_is_still_a_read() {
    let source = br#"command fn main() -> status: own ExitStatus traps {
  let values = array_new<u8, 128>(0_u8);
  let size = len(values);
  let total = 0_u64;
  for @sum i in 0_u64..16_u64 {
    let inside = ilt(total, size);
    claim running_small: inside because "premises: total starts at zero and each iteration adds the counted binder, which the range keeps below sixteen, over at most sixteen iterations; values has length one hundred twenty-eight\nderivation: induction over the reached iterations keeps total at the sum of distinct values below sixteen, which is at most one hundred twenty\nconclusion: ilt(total, size) is true\nchecker gap: ENT carries no induction over the accumulator across the counted-range backedge\nconsumers: the following length-128 array subscript uses total";
    let picked = values[total];
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let judged = only_loop(&table, "main");
    let LoopVerdict::Denied(LoopDenial::AccumulatorRead { reads, .. }) = &judged.verdict else {
        panic!(
            "a claim reading the accumulator must still refuse, got {:?}",
            judged.verdict
        );
    };
    assert_eq!(
        *reads, 3,
        "the claim's predicate read and its consumer's subscript both count beside the combine's"
    );
}

/// A `claim` in the body's call closure is likewise no reason to refuse. The
/// judgment no longer walks the call graph for claims at all.
#[test]
fn a_claim_in_the_call_closure_is_permitted() {
    let source = br#"fn narrow(v: own u64) -> result: own u64 traps {
  let values = array_new<u64, 8>(1_u64);
  let size = len(values);
  let bounded = imin(v, 7_u64);
  let inside = ilt(bounded, size);
  claim value_small: inside because "premises: bounded is the minimum of the parameter v and seven, and values has length eight\nderivation: a minimum is at most either operand, so bounded is at most seven and therefore below eight\nconclusion: ilt(bounded, size) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the following length-eight array subscript uses bounded";
  return values[bounded];
}

command fn main() -> status: own ExitStatus traps {
  let total = 0_u64;
  for @sum i in 0_u64..16_u64 {
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
        "a permitted loop over a claim-bearing callee still carries its fold"
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
/// over *programs*: the three loops below are one shape whose subscript
/// obligation is discharged three different ways — from the counted binder's
/// own S11 bounds against a length term, from a claimed fact about a
/// caller-supplied limit, and from a dominating branch — and their verdicts
/// must be identical. A judgment that read the fact state could tell them
/// apart; this one may not.
#[test]
fn the_loop_verdict_is_the_same_under_every_route_to_the_same_fact() {
    let structural = b"fn tally['s](src: &'s buffer<u64>) -> result: own u64 reads(src) {
  let count = len(deref(src));
  let total = 0_u64;
  for @sum i in 0_u64..count {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally<'s>(src: &'s data);
  }
  return exit_status(code: 0_u8);
}
";
    let claimed = br#"fn tally['s](src: &'s buffer<u64>, limit: own u64) -> result: own u64 reads(src), traps {
  let room = len(deref(src));
  let bounded_limit = imin(limit, room);
  let fits = ile(bounded_limit, room);
  claim limit_fits: fits because "premises: bounded_limit is the minimum of the requested limit and room, and room is the input buffer's length\nderivation: a minimum is at most either operand, so bounded_limit is at most room\nconclusion: ile(bounded_limit, room) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the counted range below runs to bounded_limit and subscripts the input at its binder";
  let total = 0_u64;
  for @sum i in 0_u64..bounded_limit {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let data = buffer_new(64_u64, 1_u64);
  region 's {
    let t = tally<'s>(src: &'s data, limit: 64_u64);
  }
  return exit_status(code: 0_u8);
}
"#;
    let branched =
        b"fn tally['s](src: &'s buffer<u64>, limit: own u64) -> result: own u64 reads(src) {
  let room = len(deref(src));
  let total = 0_u64;
  for @sum i in 0_u64..limit {
    let inside = ilt(i, room);
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
    let t = tally<'s>(src: &'s data, limit: 64_u64);
  }
  return exit_status(code: 0_u8);
}
";
    let verdicts = [
        structural.as_slice(),
        claimed.as_slice(),
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
    assert_eq!(verdicts[0], verdicts[1], "a claimed bound moves no verdict");
    assert_eq!(
        verdicts[0], verdicts[2],
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
  for @sum i in 0_u64..8_u64 {
    region 'i {
      let v = peek_uniq<'i>(cell: &uniq 'i cell);
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
  for @sum i in 0_u64..8_u64 {
    region 'i {
      let v = peek<'i>(cell: &'i cell);
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
  for @sum i in 0_u64..8_u64 {
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
  for @sum i in 0_u64..8_u64 {
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
  for @sum i in 0_u64..8_u64 {
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
