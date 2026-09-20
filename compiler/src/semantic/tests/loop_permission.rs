//! The loop permission judgment over counted `for` statements [PAR-2].
//!
//! Each grant fixture is a shape a real program writes; each denial fixture
//! violates exactly one numbered condition and asserts *that* condition, so a
//! denial arriving for the wrong reason fails the test. The denials are
//! deliberately the bulk of the file: granting is the easy half, and the whole
//! risk of a rule that lets an implementation choose a combination tree is a
//! loop that should have been refused and was not. Design:
//! `research/investigations/proof-derived-parallelism/loop/DESIGN.md`.
//!
//! v0.60 keeps the accumulator condition and the body's exit refusals and
//! replaces the view-formation family by [REF-4]'s proved range reference
//! `&r[s*i+b..s*i+b+s]`. `LoopDenial::Loan` is gone with [CAP-1]: there is one
//! reference kind, and what a callee reaches through one is its declared row
//! projected onto the actual's path, which condition 2 already judges. Every
//! fixture that used to name a loan therefore names the shared write or the
//! uncovered read the row actually exhibits, and the read-only borrows v0.59
//! refused by their exclusivity are now permitted.

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

/// The compute programs under `tests/programs/compute` are not owned by this
/// module; they carry the whole-program shapes the judgment was designed
/// against, and their port to v0.60 lands with those files.
#[test]
fn bfs_pull_is_permitted_while_sparse_discovery_remains_source_ordered() {
    let source = include_bytes!("../../../../tests/programs/compute/bfs.wf");
    let table = permission_of(source);
    assert_eq!(
        only_loop(&table, "pull_level").verdict,
        LoopVerdict::PermittedEligible
    );
    assert!(
        loops(&table, "bfs_sparse")
            .iter()
            .all(|item| matches!(item.verdict, LoopVerdict::Denied(_)))
    );
}

// ----------------------------------------------------------------------
// Grants
// ----------------------------------------------------------------------

/// The runtime-stride partition: one range reference per iteration over one
/// runtime-capacity origin, with the endpoint proof written as an explicit
/// local invariant. This is the shape [PAR-2]'s proved range family exists
/// for, and the derived fixtures below change exactly one thing about it
/// each.
const RUNTIME_PARTITION_SOURCE: &str = r#"fn paint(output: &[u64]) -> result: own u64 writes(output) {
  let count = deref(output).len;
  for (x in 0_u64..count) {
    set deref(output)[x] = 1_u64;
  }
  return count;
}

fn partition(width: own u64, padding: own u64, base: own u64) -> result: own Box<Array<u64>> pure contract {
  requires width <= 8_u64;
  requires padding <= 8_u64;
  requires base <= 8_u64;
} {
  let stride = width + padding;
  let cells = 6_u64 * stride;
  let total = cells + base;
  let values = box_array_filled::<u64>(count: total, value: 0_u64);
  for (i in 0_u64..4_u64) {
    let offset = i * stride;
    let start = offset + base;
    let end = start + stride;
    invariant bounded: end <= total {
      use stride times (i + 1_u64 <= 6_u64);
    }
    let row = &values.inner[start..end];
    let painted = paint(output: row);
  }
  return move values;
}

fn main() -> status: own ExitStatus pure {
  let values = partition(width: 3_u64, padding: 2_u64, base: 1_u64);
  return exit_status(code: 0_u8);
}
"#;

/// An element write reached through `deref` of a reference parameter whose row
/// declares the write is one of the four places [PAR-2] admits.
///
/// v0.59 refused this: a view element store had no map family of its own and
/// needed the caller to hand down a range assignment. v0.60 names the shape
/// outright — "one direct `Array` or `Slots` subscript rooted in an own
/// binding declared outside L or reached through `deref` of a reference
/// parameter whose row declares the write" — so the helper's own loop is the
/// ordinary single-binder affine element map.
#[test]
fn a_reference_parameter_element_write_is_an_admitted_element_map() {
    let judged = permitted(RUNTIME_PARTITION_SOURCE.as_bytes(), "paint");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

#[test]
fn runtime_stride_sum_base_and_descendant_helper_calls_are_permitted() {
    with_semantics(RUNTIME_PARTITION_SOURCE.as_bytes(), |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("{outcome:?}");
        };
        for function in &program.data.functions {
            super::entailment::validate_derivations(&function.entailment);
        }
    });
    let judged = permitted(RUNTIME_PARTITION_SOURCE.as_bytes(), "partition");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// The same partition with its two endpoints bound through different exact
/// products and its domain established by ordinary guards rather than by the
/// written proof. [PAR-2] consumes the retained exact images, not the
/// spelling, so the range family must still recognize it.
#[test]
fn equivalent_product_endpoints_use_checked_images() {
    let source = br#"fn paint(output: &[u64]) -> result: own u64 writes(output) {
  let count = deref(output).len;
  for (x in 0_u64..count) {
    set deref(output)[x] = 1_u64;
  }
  return count;
}

fn partition(width: own u64, padding: own u64, base: own u64) -> result: own Box<Array<u64>> pure contract {
  requires width <= 8_u64;
  requires padding <= 8_u64;
  requires base <= 8_u64;
} {
  let stride = width + padding;
  let cells = 6_u64 * stride;
  let total = cells + base;
  let values = box_array_filled::<u64>(count: total, value: 0_u64);
  for (i in 0_u64..4_u64) {
    let offset = i * stride;
    let start = offset + base;
    let after = i + 1_u64;
    let limit = after * stride;
    let end = limit + base;
    if start <= end {
      if end <= total {
        let row = &values.inner[start..end];
        let painted = paint(output: row);
      }
    }
  }
  return move values;
}

fn main() -> status: own ExitStatus pure {
  let values = partition(width: 3_u64, padding: 2_u64, base: 1_u64);
  return exit_status(code: 0_u8);
}
"#;
    let judged = permitted(source, "partition");
    assert_eq!(
        judged.actualization,
        Some(LoopActualization::IndependentMap)
    );
}

/// Two range references of one origin carrying different bases. Their
/// per-iteration writes interleave across iterations, so the two written
/// partitions on one origin cannot have different maps.
#[test]
fn disjoint_siblings_with_shifted_partitions_can_cross_between_iterations() {
    let source = RUNTIME_PARTITION_SOURCE.replace(
        "    let painted = paint(output: row);",
        "    let painted = paint(output: row);\n    let shifted = end + stride;\n    invariant room: shifted <= total {\n      use stride times (i + 2_u64 <= 6_u64);\n    }\n    let other = &values.inner[end..shifted];\n    let second = paint(output: other);",
    );
    assert!(matches!(
        denied(source.as_bytes(), "partition", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

#[test]
fn a_shifted_read_of_a_written_origin_can_cross_between_iterations() {
    let source = RUNTIME_PARTITION_SOURCE.replace(
        "    let painted = paint(output: row);",
        "    let painted = paint(output: row);\n    let shifted = end + stride;\n    invariant room: shifted <= total {\n      use stride times (i + 2_u64 <= 6_u64);\n    }\n    let other = &values.inner[end..shifted];\n    let size = deref(other).len;\n    if 0_u64 < size {\n      let observed = deref(other)[0_u64];\n    }",
    );
    assert!(matches!(
        denied(source.as_bytes(), "partition", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

#[test]
fn forming_an_unused_shifted_reference_reads_no_written_elements() {
    let source = RUNTIME_PARTITION_SOURCE.replace(
        "    let painted = paint(output: row);",
        "    let painted = paint(output: row);\n    let shifted = end + stride;\n    invariant room: shifted <= total {\n      use stride times (i + 2_u64 <= 6_u64);\n    }\n    let other = &values.inner[end..shifted];",
    );
    assert_eq!(
        permitted(source.as_bytes(), "partition").actualization,
        Some(LoopActualization::IndependentMap)
    );
}

#[test]
fn read_only_range_work_does_not_fabricate_an_independent_write_map() {
    let source = RUNTIME_PARTITION_SOURCE
        .replace("writes(output)", "reads(output)")
        .replace(
            "    set deref(output)[x] = 1_u64;",
            "    let observed = deref(output)[x];",
        );
    assert_eq!(
        permitted(source.as_bytes(), "partition").actualization,
        None
    );
}

/// "A whole-origin access ... denies." The guarded element read reaches the
/// origin outside every iteration's own tile.
#[test]
fn a_whole_origin_read_outside_the_tile_denies_overlap() {
    let source = RUNTIME_PARTITION_SOURCE.replace(
        "    let painted = paint(output: row);\n  }",
        "    let painted = paint(output: row);\n    if 0_u64 < total {\n      let outside = values.inner[0_u64];\n    }\n  }",
    );
    assert!(matches!(
        denied(source.as_bytes(), "partition", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// A constant offset gives every iteration the same extent, so the retained
/// endpoint images are not `[s*i+b, s*i+b+s)` for any fixed s and b and no
/// proved range reference exists. The helper's declared write of the origin
/// is then an ordinary shared write.
#[test]
fn a_constant_nonempty_range_is_not_an_iteration_partition() {
    let source = RUNTIME_PARTITION_SOURCE
        .replace("    let offset = i * stride;", "    let offset = 0_u64 * stride;")
        .replace(
            "    invariant bounded: end <= total {\n      use stride times (i + 1_u64 <= 6_u64);\n    }",
            "    invariant bounded: end <= total;",
        );
    assert!(matches!(
        denied(source.as_bytes(), "partition", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// A stride recomputed from the current index is not fixed throughout L, so
/// the endpoint images are not affine in the binder and the family refuses
/// rather than starting a search.
#[test]
fn a_stride_computed_from_the_current_index_is_not_loop_invariant() {
    let source = RUNTIME_PARTITION_SOURCE
        .replace(
            "  let stride = width + padding;\n  let cells = 6_u64 * stride;\n  let total = cells + base;",
            "  let cells = 96_u64;\n  let total = cells + base;",
        )
        .replace(
            "    let offset = i * stride;",
            "    let stride = 4_u64 - i;\n    let offset = i * stride;",
        )
        .replace(
            "    invariant bounded: end <= total {\n      use stride times (i + 1_u64 <= 6_u64);\n    }",
            "    invariant bounded: end <= total;",
        );
    assert!(matches!(
        denied(source.as_bytes(), "partition", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// The compute programs under `tests/programs/compute` are not owned by this
/// module; they carry the whole-program shapes the judgment was designed
/// against, and their port to v0.60 lands with those files.
#[test]
fn runtime_stencil_rows_retain_adjacent_range_permission() {
    let source = include_bytes!("../../../../tests/programs/compute/stencil.wf");
    let table = permission_of(source);
    let rows = loops(&table, "stencil");
    assert_eq!(rows.len(), 4);
    for row in [&rows[0], &rows[2], &rows[3]] {
        assert_eq!(row.verdict, LoopVerdict::PermittedEligible, "{row:?}");
        assert_eq!(row.actualization, Some(LoopActualization::IndependentMap));
    }
    assert!(matches!(rows[1].verdict, LoopVerdict::Denied(_)));
}

/// As above: the blocked compute helpers are whole-program evidence owned by
/// `tests/programs/compute`.
#[test]
fn blocked_compute_helpers_retain_partition_permission_around_local_recurrences() {
    let prefix = permission_of(include_bytes!(
        "../../../../tests/programs/compute/prefix.wf"
    ));
    let stages = loops(&prefix, "prefix");
    assert_eq!(stages.len(), 3);
    for stage in [&stages[0], &stages[2]] {
        assert_eq!(stage.verdict, LoopVerdict::PermittedEligible, "{stage:?}");
        assert_eq!(stage.actualization, Some(LoopActualization::IndependentMap));
    }
    assert!(matches!(stages[1].verdict, LoopVerdict::Denied(_)));
    assert!(matches!(
        loops(&prefix, "scan_block")[0].verdict,
        LoopVerdict::Denied(_)
    ));

    let histogram = permission_of(include_bytes!(
        "../../../../tests/programs/compute/histogram.wf"
    ));
    let stages = loops(&histogram, "histogram");
    assert_eq!(stages.len(), 2);
    for stage in stages {
        assert_eq!(stage.verdict, LoopVerdict::PermittedEligible, "{stage:?}");
        assert_eq!(stage.actualization, Some(LoopActualization::IndependentMap));
    }
    assert!(matches!(
        loops(&histogram, "count_block")[0].verdict,
        LoopVerdict::Denied(_)
    ));
}

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

fn main() -> status: own ExitStatus pure {
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
/// read, write, accumulator, or exit to the loop permission survey.
#[test]
fn a_local_invariant_in_the_body_has_no_runtime_footprint() {
    let source = br#"fn main() -> status: own ExitStatus pure {
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
            "fn main() -> status: own ExitStatus pure {{
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
            "fn main() -> status: own ExitStatus pure {{
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

fn main() -> status: own ExitStatus pure {
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

/// A callee that writes through a reference parameter into storage the
/// *iteration* introduced is permitted.
///
/// The row says the callee writes; the projection says what it writes, and a
/// place rooted in a binding the body opens is created fresh by every
/// iteration. Refusing every writing callee would leave any loop whose
/// iteration builds a scratch structure through a helper out of reach, and the
/// judgment holds the resolved places a diagnostic cannot rebuild.
#[test]
fn a_callee_writing_iteration_own_storage_is_permitted() {
    let source = b"struct Cell {
  value: u64;
}

fn bump(slot: &Cell, x: own u64) -> result: own u64 writes(slot.value) {
  set deref(slot).value = deref(slot).value +wrap x;
  return deref(slot).value;
}

fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let scratch = Cell(value: 0_u64);
    let got = bump(slot: &scratch, x: i);
    set total = total +wrap got;
  }
  return exit_status(code: 0_u8);
}
";
    let judged = permitted(source, "main");
    assert_eq!(judged.combines, vec!["+wrap"]);
}

/// A whole-binding `set` whose target the iteration introduced is permitted:
/// the value [WIN-3] releases is this iteration's, not the previous one's.
///
/// v0.59 spelled this `let previous = replace held = move fresh;`; the
/// `replace` statement is gone and `set p = e;` is its successor.
#[test]
fn a_set_of_iteration_own_storage_is_permitted() {
    let source = b"fn main() -> status: own ExitStatus pure {
  for @swap (i in 0_u64..8_u64) {
    let held = array_filled::<u64, 4>(value: 0_u64);
    let fresh = array_filled::<u64, 4>(value: i);
    set held = fresh;
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
    let source = b"fn main() -> status: own ExitStatus pure {
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 8>(value: 0_u64);
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
    let source = br#"fn tally(src: &Slots<u64, 64>, limit: own u64) -> result: own u64 reads(src) {
  let spare = deref(src).len;
  invariant scaled_limit_fits: 4_u64 * limit <= 4_u64 * spare {
    use 4 times (limit <= spare);
  }
  let total = 0_u64;
  for @sum (i in 0_u64..limit) {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  let t = tally(src: &data, limit: 64_u64);
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

/// A dominating branch establishes the same `limit <= src.len` fact. The loop
/// remains eligible because permission reads the checked body footprint, not
/// the proof route for its subscript.
#[test]
fn a_dominating_bound_outside_the_loop_leaves_it_eligible() {
    let source = br#"fn tally(src: &Slots<u64, 64>, limit: own u64) -> result: own u64 reads(src) {
  let spare = deref(src).len;
  let total = 0_u64;
  let fits = limit <= spare;
  if fits {
    for @sum (i in 0_u64..limit) {
      let v = deref(src)[i];
      set total = total +wrap v;
    }
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  let t = tally(src: &data, limit: 64_u64);
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
    let source = b"fn main() -> status: own ExitStatus pure {
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
    let integral = b"fn main() -> status: own ExitStatus pure {
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
    let source = b"fn main() -> status: own ExitStatus pure {
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

fn main() -> status: own ExitStatus pure {
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
    let source = b"fn main() -> status: own ExitStatus pure {
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

/// A whole-binding `set` of enclosing storage carries state across iterations
/// that no operation combines, so which iteration wrote last is observable.
///
/// v0.59 spelled the same hazard `let previous = replace held = move fresh;`
/// and cited the value `replace` read out. `set p = e;` is its successor and
/// releases the old affine value in place [WIN-3].
#[test]
fn a_set_of_enclosing_storage_is_denied_by_condition_one() {
    let source = b"fn main() -> status: own ExitStatus pure {
  let held = array_filled::<u64, 4>(value: 0_u64);
  for @swap (i in 0_u64..8_u64) {
    let fresh = array_filled::<u64, 4>(value: i);
    set held = fresh;
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
    let source = b"fn main() -> status: own ExitStatus pure {
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let table = array_filled::<u64, 64>(value: 0_u64);
  let cursor = 0_u64;
  for @fill (i in 0_u64..8_u64) {
    let spare = cursor < 64_u64;
    if spare {
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

/// A reference to the accumulator formed in the body is an occurrence of that
/// accumulator like any other, so the read count refuses the reduction.
///
/// v0.59 refused every borrow-forming body statement as a form, because the
/// checked tree erased the borrow's shared-or-uniq mode and an unloaned borrow
/// would widen permission. v0.60 has no mode to erase: forming the reference
/// names the accumulator's path, and the ordinary count is what denies.
#[test]
fn a_reference_to_the_accumulator_is_a_read_of_it() {
    let source = b"fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    let view = &total;
    let seen = deref(view);
    let bumped = seen +wrap i;
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::AccumulatorRead { reads, .. } = denied(source, "main", 1) else {
        panic!("expected a read-count denial");
    };
    assert_eq!(reads, 2, "the formation and the combine");

    // A reference taken *after* the loop is outside the body, so the same
    // reduction stays permitted: the count is per body, never per function.
    let after = b"fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum (i in 0_u64..16_u64) {
    set total = total +wrap i;
  }
  let view = &total;
  let seen = deref(view);
  let bumped = seen +wrap 1_u64;
  return exit_status(code: 0_u8);
}
";
    permitted(after, "main");
}

/// Two accumulators are refused, and this is the one refusal the split advice
/// outlives: a hand-written recursion may return an aggregate.
#[test]
fn two_accumulators_are_denied_by_condition_one_and_keep_the_split_advice() {
    let source = b"fn main() -> status: own ExitStatus pure {
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
    let source = b"fn main() -> status: own ExitStatus pure {
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 64>(value: 0_u64);
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

/// Slots use direct logical-to-physical offsets, while a Ring adds its
/// runtime head and wraps modulo capacity [WIN-1]. The same proved counted
/// index is therefore an independent element map only for Slots [PAR-2].
#[test]
fn a_ring_subscript_is_not_an_affine_element_map() {
    let source = b"fn slots_map() -> result: own u64 pure {
  let values = array_filled::<u64, 4>(value: 0_u64);
  let slots = slots_from_array::<u64, 4>(values: values);
  for @fill (i in 0_u64..4_u64) {
    set slots[i] = i;
  }
  return slots[0_u64];
}

fn ring_map() -> result: own u64 pure {
  let ring = ring_new::<u64, 4>();
  place_back(window: &ring, value: 0_u64);
  place_back(window: &ring, value: 0_u64);
  place_back(window: &ring, value: 0_u64);
  place_back(window: &ring, value: 0_u64);
  for @fill (i in 0_u64..4_u64) {
    set ring[i] = i;
  }
  return ring[0_u64];
}
";
    let table = permission_of(source);
    assert_eq!(
        only_loop(&table, "slots_map").verdict,
        LoopVerdict::PermittedEligible
    );
    let ring = only_loop(&table, "ring_map");
    assert!(matches!(denial(ring, 2), LoopDenial::SharedWrite { .. }));
    assert_eq!(ring.actualization, None);
}

/// Permission consumes the offset's exact checked value rather than its
/// spelling. Copying the binder and applying one proved affine transform keeps
/// the nonzero coefficient, so distinct iterations still select distinct
/// elements.
#[test]
fn a_copied_affine_binder_element_map_is_permitted() {
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 128>(value: 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let step = i;
    let slot = step * 2_u64;
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 128>(value: 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let step = i;
    let slot = step * 2_u64;
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 64>(value: 0_u64);
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 128>(value: 0_u64);
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 128>(value: 0_u64);
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
/// write image; this is not treated as a whole-run dependence.
#[test]
fn a_same_index_read_modify_write_is_permitted() {
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u8, 64>(value: 0_u8);
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u8, 64>(value: 0_u8);
  for @update (i in 0_u64..64_u64) {
    let spare = out.len;
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

/// A writable reference parameter is one source place. Once OP-4 proves `i` is
/// in range, the same affine-map rule divides that place into disjoint
/// per-iteration elements; ownership does not require copying the run into the
/// callee.
#[test]
fn a_reference_output_accepts_a_proved_element_map() {
    let source =
        br#"fn fill(out: &Slots<u8, 64>, count: own u64) -> result: own unit writes(out) contract {
  define spare = deref(out).len;
  requires count <= spare;
} {
  for @fill (i in 0_u64..count) {
    set deref(out)[i] = 1_u8;
  }
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u8, 64>(value: 0_u8);
  let out = slots_from_array::<u8, 64>(values: values);
  let filled = fill(out: &out, count: 64_u64);
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let evens = array_filled::<u64, 128>(value: 0_u64);
  let shifted = array_filled::<u64, 65>(value: 0_u64);
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
  left: Array<u64, 64>;
  right: Array<u64, 64>;
}

fn main() -> status: own ExitStatus pure {
  let left = array_filled::<u64, 64>(value: 0_u64);
  let right = array_filled::<u64, 64>(value: 0_u64);
  let columns = Columns(left: left, right: right);
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 64>(value: 0_u64);
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
    let source = br#"fn fill(output: own Array<u64, 64>, limit: own u64) -> result: own Array<u64, 64> pure {
  let spare = output.len;
  invariant limit_fits: limit <= spare {
    use (limit <= spare);
  }
  for @fill (i in 0_u64..limit) {
    set output[i] = i;
  }
  return output;
}

fn main() -> status: own ExitStatus pure {
  let output = array_filled::<u64, 64>(value: 0_u64);
  let filled = fill(output: output, limit: 64_u64);
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

/// A read row on the mapped root is an access overlapping that root, and it
/// descends from no proved element map, so the read counts do not balance and
/// condition 2 denies.
///
/// v0.59 cited a shared call loan here. [CAP-1] leaves no loan: "the [EFF-2]
/// projection of a helper's declared row counts as an access on its actual
/// range", and this row's access is the whole run.
#[test]
fn a_read_row_on_the_mapped_root_is_denied() {
    let source = b"fn observe(value: &Array<u64, 64>) -> result: own u64 reads(value) {
  return deref(value).len;
}

fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 64>(value: 0_u64);
  for @fill (i in 0_u64..64_u64) {
    let seen = observe(value: &out);
    set out[i] = i;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// An exact affine value is insufficient when OP-4 cannot prove the subscript
/// is inside the collection. The dark entry lets this test inspect that
/// fail-closed permission verdict without changing ordinary source acceptance.
#[test]
fn an_unproved_counted_binder_element_map_remains_denied() {
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 64>(value: 0_u64);
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
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 64>(value: 0_u64);
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

/// A stencil writes one element and reads another of the same run, which is a
/// dependence across iterations. Its read and write maps differ, so the fixed
/// same-map refinement refuses it.
#[test]
fn a_stencil_is_denied_by_condition_two() {
    let source = b"fn main() -> status: own ExitStatus pure {
  let out = array_filled::<u64, 64>(value: 1_u64);
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
    let source = b"struct Holder {
  value: f64;
}

fn accum(slot: &Holder, x: own f64) -> result: own u64 writes(slot.value) {
  set deref(slot).value = fadd.strict(deref(slot).value, x);
  let bits = reinterpret::<f64, u64>(deref(slot).value);
  return iand(bits, 1_u64);
}

fn main() -> status: own ExitStatus pure {
  let total = Holder(value: 0.0_f64);
  let count = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    let one = accum(slot: &total, x: 0.5_f64);
    set count = count +wrap one;
  }
  return exit_status(code: 0_u8);
}
";
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
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

fn main() -> status: own ExitStatus pure {
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
// The ordinary callable boundary [CAP-1]
// ----------------------------------------------------------------------

/// An ordinary helper writes the factory its caller handed it, and that
/// storage outlives the iteration. PRE-1 open/close declarations and a WF
/// wrapper obey the same PAR-2 condition.
///
/// v0.59 cited the unique factory loan the helper held to return. v0.60 has
/// no loan: the helper's declared `writes(factory)` projects onto the caller's
/// path, and that place is neither iteration-own nor a proved range.
#[test]
fn an_ordinary_directory_wrapper_writes_enclosing_storage() {
    let source = br#"fn probe(factory: &HandleFactory, root: &DirectoryRead) -> result: own u64 reads(root), writes(factory) {
  match open_directory_source(factory: factory, directory: root) {
    Ok(value: listing) => {
      let closed = close_directory_source(factory: factory, source: move listing);
      return 1_u64;
    }
    Err(error: refused) => {
      return 0_u64;
    }
  }
}

fn main(factory: &HandleFactory, root: &DirectoryRead) -> result: own unit reads(root), writes(factory) {
  let total = 0_u64;
  for @scan (i in 0_u64..4_u64) {
    let seen = probe(factory: factory, root: root);
    set total = total +wrap seen;
  }
  return unit;
}
"#;
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

/// The direct PRE-1 declaration's ordinary factory, input and destination
/// writes prevent loop-iteration overlap under the same condition. v0.58
/// directory_next returns multiple results, outside PAR-2's direct-let shape;
/// read_next preserves this test's single-result trigger.
#[test]
fn a_direct_read_state_transition_writes_enclosing_storage() {
    let source = br#"fn main(factory: &HandleFactory, input: &InputStream, destination: &[u8]) -> result: own unit writes(factory), writes(input), writes(destination) contract {
  requires 1_u64 <= deref(destination).len;
} {
  let total = 0_u64;
  for @scan (i in 0_u64..4_u64) {
    let outcome = read_next(factory: factory, input: input, destination: destination, start: 0_u64, end: 1_u64);
    set total = total +wrap 1_u64;
  }
  return unit;
}
"#;
    assert!(matches!(
        denied(source, "main", 2),
        LoopDenial::SharedWrite { .. }
    ));
}

// ----------------------------------------------------------------------
// Condition 4: no exit edge
// ----------------------------------------------------------------------

/// A `break` that closes the judged loop skips the rest of the range, so the
/// set of iterations is no longer the whole range.
#[test]
fn a_break_out_of_the_loop_is_denied_by_condition_four() {
    let source = b"fn main() -> status: own ExitStatus pure {
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
    let outward = b"fn main() -> status: own ExitStatus pure {
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

    let inward = b"fn main() -> status: own ExitStatus pure {
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

fn main() -> status: own ExitStatus pure {
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
    let source = b"fn main() -> status: own ExitStatus pure {
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
        b"fn scan_until(src: &Array<u64, 64>, needle: own u64) -> result: own u64 reads(src) {
  let count = deref(src).len;
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

fn main() -> status: own ExitStatus pure {
  let data = array_filled::<u64, 64>(value: 1_u64);
  set data[10_u64] = 7_u64;
  let t = scan_until(src: &data, needle: 7_u64);
  return exit_status(code: 0_u8);
}
";
    let LoopDenial::Exit { edge } = denied(source, "scan_until", 4) else {
        panic!("expected an exit denial");
    };
    assert_eq!(edge, "a give");

    // The same loop with the give removed is permitted, so the refusal is
    // about the edge and not about the shape.
    let contained =
        b"fn scan_until(src: &Array<u64, 64>, needle: own u64) -> result: own u64 reads(src) {
  let count = deref(src).len;
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

fn main() -> status: own ExitStatus pure {
  let data = array_filled::<u64, 64>(value: 1_u64);
  set data[10_u64] = 7_u64;
  let t = scan_until(src: &data, needle: 7_u64);
  return exit_status(code: 0_u8);
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

fn main() -> status: own ExitStatus pure {
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
    let source = br#"const values: Array<u8, 8> =[0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8];

fn main() -> status: own ExitStatus pure {
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
    let source = br#"const values: Array<u8, 8> =[0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8];

fn main() -> status: own ExitStatus pure {
  let size = values.len;
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

/// In the accepted replacement, the ordinary guard and guarded subscript each
/// read the accumulator beside its combine. Condition 1 must count all three
/// reads and refuse the reduction.
#[test]
fn a_guard_reading_the_accumulator_is_still_a_read() {
    let source = br#"const values: Array<u8, 128> =[0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8];

fn main() -> status: own ExitStatus pure {
  let size = values.len;
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

/// A callee whose branch proves its own subscript is a normal pure call in the
/// body. It contributes no shared write and does not block reduction
/// actualization.
#[test]
fn a_proof_complete_call_closure_is_permitted() {
    let source =
        br#"const values: Array<u64, 8> =[1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64, 1_u64];

fn narrow(v: own u64) -> result: own u64 pure {
  let size = values.len;
  if v < size {
    return values[v];
  }
  return 0_u64;
}

fn main() -> status: own ExitStatus pure {
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
    let structural = b"fn tally(src: &Slots<u64, 64>) -> result: own u64 reads(src) {
  let count = deref(src).len;
  let total = 0_u64;
  for @sum (i in 0_u64..count) {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  let t = tally(src: &data);
  return exit_status(code: 0_u8);
}
";
    let invariant_source =
        br#"fn tally(src: &Slots<u64, 64>, bounded_limit: own u64, limit: own u64) -> result: own u64 reads(src) contract {
  define capacity = deref(src).len;
  requires bounded_limit <= limit;
  requires limit <= capacity;
} {
  let spare = deref(src).len;
  invariant limit_fits: bounded_limit <= spare;
  let total = 0_u64;
  for @sum (i in 0_u64..bounded_limit) {
    let v = deref(src)[i];
    set total = total +wrap v;
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  let t = tally(src: &data, bounded_limit: 64_u64, limit: 64_u64);
  return exit_status(code: 0_u8);
}
"#;
    let dominating =
        b"fn tally(src: &Slots<u64, 64>, limit: own u64) -> result: own u64 reads(src) {
  let spare = deref(src).len;
  let total = 0_u64;
  if limit <= spare {
    for @sum (i in 0_u64..limit) {
      let v = deref(src)[i];
      set total = total +wrap v;
    }
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  let t = tally(src: &data, limit: 64_u64);
  return exit_status(code: 0_u8);
}
";
    let branched = b"fn tally(src: &Slots<u64, 64>, limit: own u64) -> result: own u64 reads(src) {
  let spare = deref(src).len;
  let total = 0_u64;
  for @sum (i in 0_u64..limit) {
    let inside = i < spare;
    if inside {
      let v = deref(src)[i];
      set total = total +wrap v;
    }
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  let values = array_filled::<u64, 64>(value: 1_u64);
  let data = slots_from_array::<u64, 64>(values: values);
  let t = tally(src: &data, limit: 64_u64);
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

// Retired with the exclusive/shared distinction of v0.59's loans half:
// `a_read_only_unique_borrow_of_outer_storage_is_denied_by_its_loan` wrote
// `&uniq cell` where v0.60 writes `&cell`, so it is now the same program as
// the grant below, whose successor assertion covers it.

/// Read-only sharing across iterations is the point of a reduction: a
/// reference to one outer cell carries no write, and every iteration reads it.
///
/// This is also what became of v0.59's exclusive borrow of the same cell.
/// [CAP-1] leaves one reference kind, so `&cell` at a `reads` row is the only
/// spelling and the loop is permitted whichever marker v0.59 would have used.
#[test]
fn a_reference_to_outer_storage_at_a_read_row_stays_permitted() {
    let source = br#"fn peek(cell: &u64) -> result: own u64 reads(cell) {
  return deref(cell);
}

fn main() -> status: own ExitStatus pure {
  let cell = 21_u64;
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    let v = peek(cell: &cell);
    set acc = acc +wrap v;
  }
  return exit_status(code: 0_u8);
}
"#;
    permitted(source, "main");
}

/// A reference bound in the body is an ordinary member of the survey.
///
/// v0.59 refused every borrow-forming body statement as a form: no call
/// carried the borrow, so no parameter mode stated its loan, and the checked
/// tree erased whether it was shared or exclusive. v0.60 has one reference
/// kind and no loan, so the statement is judged like any other and a body that
/// only reads outer storage through it is permitted.
#[test]
fn a_body_statement_forming_a_reference_is_an_ordinary_member() {
    let source = br#"fn main() -> status: own ExitStatus pure {
  let cell = 21_u64;
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    let g = &cell;
    let v = deref(g);
    set acc = acc +wrap v;
  }
  return exit_status(code: 0_u8);
}
"#;
    assert_eq!(permitted(source, "main").combines, vec!["+wrap"]);
}

// Retired with the same distinction: `a_body_shared_borrow_of_outer_storage_
// is_knowingly_denied` was the shared spelling of the fixture above, and the
// mode it was knowingly denied for no longer exists, so the two programs are
// one and the grant above is its successor.

/// The admissible side of the body reference guard remains admissible: a
/// `let`-bound reference to storage the iteration itself creates. Each
/// iteration names its own instance.
#[test]
fn a_body_reference_to_iteration_own_storage_stays_permitted() {
    let source = br#"fn main() -> status: own ExitStatus pure {
  let acc = 0_u64;
  for @sum (i in 0_u64..8_u64) {
    let local = array_filled::<u8, 4>(value: 7_u8);
    let h = &local;
    let v = deref(h).len;
    set acc = acc +wrap v;
  }
  return exit_status(code: 0_u8);
}
"#;
    permitted(source, "main");
}
