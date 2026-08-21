//! The permission judgment P over sibling call statements.
//!
//! Each grant fixture is a shape a real program writes; each denial fixture
//! violates exactly one numbered condition and asserts *that* condition, so a
//! denial arriving for the wrong reason fails the test. Design:
//! `research/investigations/proof-derived-parallelism/DESIGN.md` section 3.

use crate::SemanticOutcome;

use super::super::permission::{
    Access, ConflictKind, Denial, ExitKind, FunctionPermissions, PairSide, PermissionMetadata,
    PermissionPair, PermissionVerdict,
};
use super::with_semantics;

fn permission_of(source: &[u8]) -> PermissionMetadata {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("permission fixture must check: {outcome:?}");
        };
        program.data.permission.clone()
    })
}

fn function_table<'table>(
    table: &'table PermissionMetadata,
    name: &str,
) -> &'table FunctionPermissions {
    table
        .named(name)
        .unwrap_or_else(|| panic!("no permission table for {name}"))
}

/// The only analyzed pair of one function. Every fixture below keeps its
/// interesting block to a single pair so the assertion cannot drift onto a
/// neighbour.
fn only_pair<'table>(table: &'table PermissionMetadata, name: &str) -> &'table PermissionPair {
    let permissions = function_table(table, name);
    assert_eq!(
        permissions.pairs.len(),
        1,
        "{name} must have exactly one analyzed pair: {:?}",
        permissions.pairs
    );
    &permissions.pairs[0]
}

fn denial(pair: &PermissionPair, condition: u8) -> &Denial {
    let PermissionVerdict::Denied(denial) = &pair.verdict else {
        panic!("expected a denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        denial.condition(),
        condition,
        "denied by the wrong condition: {denial:?}"
    );
    denial
}

// ----------------------------------------------------------------------
// Grants
// ----------------------------------------------------------------------

/// The two-child unique tree fold: each recursive sibling reaches storage
/// only through its own `&uniq` payload binder, and [OWN-13] makes the two
/// binders disjoint. This is the shape a parallel fold is written in.
#[test]
fn two_child_unique_sibling_calls_are_permitted_and_eligible() {
    let source = br#"enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn boxed_leaf(w: own u64) -> result: own box<BoxNode> allocates(heap) {
  let leaf = Leaf(w: w);
  return box_new(move leaf);
}

fn boxed_branch(left: own box<BoxNode>, right: own box<BoxNode>) -> result: own box<BoxNode> allocates(heap) {
  let branch = Branch(left: move left, right: move right, w: 0_u64);
  return box_new(move branch);
}

fn fold['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads('b), writes('b) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold<'b>(node: move l);
      let b = fold<'b>(node: move r);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }
  }
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region 'tree {
    let total = fold<'tree>(node: &uniq 'tree branch0);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "fold");
    assert_eq!(pair.first.callee_name, "fold");
    assert_eq!(pair.second.callee_name, "fold");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
    let runs = &function_table(&table, "fold").runs;
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].sites.len(), 2);
}

/// Read-only sibling recursion. Nothing is written at all, so condition 2 is
/// satisfied by an empty write footprint rather than by disjointness.
#[test]
fn read_only_sibling_recursion_is_permitted_and_eligible() {
    let source = br#"enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn depth['b](node: &'b box<BoxNode>) -> result: own u64 reads('b) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return 1_u64;
    }
    Branch(left: l, right: r, w: slot) => {
      let a = depth<'b>(node: l);
      let b = depth<'b>(node: r);
      return imax(a, b);
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "depth");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
}

/// Three `reads`-only calls over one and the same buffer. Shared reads of
/// overlapping storage conflict with nothing [OWN-5], so all three pairs hold
/// and the adjacent statements form one chain — the bisection shape, where
/// every lane views the same immutable input.
#[test]
fn reads_only_siblings_over_one_place_form_one_eligible_chain() {
    let source = br#"fn width['r](data: &'r buffer<u64>) -> result: own u64 reads('r) {
  return len(deref(data));
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let buf = buffer_new(8_u64, 1_u64);
  region 'r {
    let lo = width<'r>(data: &'r buf);
    let mid = width<'r>(data: &'r buf);
    let hi = width<'r>(data: &'r buf);
    let part = imax(mid, hi);
    let total = imax(lo, part);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let permissions = function_table(&table, "main");
    assert_eq!(permissions.pairs.len(), 2);
    for pair in &permissions.pairs {
        assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
    }
    assert_eq!(permissions.runs.len(), 1);
    assert_eq!(permissions.runs[0].sites.len(), 3);
}

/// A chain is not implied by its adjacent pairs. Here s1 and s2 are disjoint
/// and s2 and s3 are disjoint, but s1 and s3 write the same cell, so the
/// chain stops at two members even though both adjacent pairs hold.
#[test]
fn a_chain_stops_where_a_nonadjacent_pair_conflicts() {
    let source = br#"fn bump['r](slot: &uniq 'r u64) -> result: own u64 reads('r), writes('r) {
  let seen = deref(slot);
  set deref(slot) = 7_u64;
  return seen;
}

command fn main() -> status: own ExitStatus pure {
  let first = 1_u64;
  let second = 2_u64;
  region 'r {
    let a = bump<'r>(slot: &uniq 'r first);
    let b = bump<'r>(slot: &uniq 'r second);
    let c = bump<'r>(slot: &uniq 'r first);
    let part = imax(b, c);
    let total = imax(a, part);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let permissions = function_table(&table, "main");
    assert_eq!(permissions.pairs.len(), 2);
    for pair in &permissions.pairs {
        assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
    }
    assert_eq!(permissions.runs.len(), 1);
    assert_eq!(permissions.runs[0].sites.len(), 2);
}

// ----------------------------------------------------------------------
// Denials, each by its own condition
// ----------------------------------------------------------------------

/// Condition 1. The second sibling passes the first's result, so the two
/// cannot overlap however small the dataflow value is.
#[test]
fn a_dataflow_link_between_siblings_is_denied_by_condition_one() {
    let source = br#"enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn fold_shift['b](node: &uniq 'b box<BoxNode>, shift: own u64) -> result: own u64 reads('b), writes('b) {
  let base = fold<'b>(node: move node);
  return imax(base, shift);
}

fn fold['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads('b), writes('b) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold<'b>(node: move l);
      let b = fold_shift<'b>(node: move r, shift: a);
      let total = imax(a, b);
      set deref(slot) = total;
      return total;
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "fold");
    let Denial::Dataflow { binding } = denial(pair, 1) else {
        panic!("expected a dataflow denial, got {:?}", pair.verdict);
    };
    assert_eq!(*binding, pair.first.binding, "cited the wrong binding");
}

/// Condition 2. Two `&uniq` actuals resolve to one and the same place, so the
/// two write footprints overlap under [OWN-7].
#[test]
fn overlapping_unique_arguments_are_denied_by_condition_two() {
    let source = br#"fn bump['r](slot: &uniq 'r u64) -> result: own u64 reads('r), writes('r) {
  let seen = deref(slot);
  set deref(slot) = 7_u64;
  return seen;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region 'r {
    let lo = bump<'r>(slot: &uniq 'r cell);
    let hi = bump<'r>(slot: &uniq 'r cell);
    let total = imax(lo, hi);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Footprint { kind, left, right } = denial(pair, 2) else {
        panic!("expected a footprint denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind::WriteWrite,
        "both actuals are written through"
    );
    let (Access::Place { place: left, .. }, Access::Place { place: right, .. }) = (left, right)
    else {
        panic!("expected two conflicting places, got {left:?} and {right:?}");
    };
    assert!(left.overlaps(right));
    assert_eq!(left, right, "both actuals resolve to the one cell");
}

/// Condition 2, the caller-side half. `take`'s row is `pure` and reaches no
/// caller storage at all, but its own operand reads the cell `bump` writes,
/// and the overlap moves exactly that read across `bump`'s call. Before this
/// condition existed the pair was permitted, eligible, and produced the
/// pre-write value with no runtime linked.
#[test]
fn an_operand_read_of_written_storage_is_denied_by_condition_two() {
    let source = br#"fn bump['r](slot: &uniq 'r u64) -> result: own u64 writes('r) {
  set deref(slot) = 15_u64;
  return 1_u64;
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region 'r {
    let a = bump<'r>(slot: &uniq 'r cell);
    let b = take(v: cell);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Footprint { kind, left, right } = denial(pair, 2) else {
        panic!("expected a footprint denial, got {:?}", pair.verdict);
    };
    assert_eq!(*kind, ConflictKind::WriteOperandRead);
    let (Access::Place { place: left, .. }, Access::Place { place: right, .. }) = (left, right)
    else {
        panic!("expected two conflicting places, got {left:?} and {right:?}");
    };
    assert_eq!(
        left, right,
        "the borrow and the operand resolve to the one cell"
    );
}

/// The same hazard through a subscript rather than a whole binding: the
/// element read is rooted at the buffer the first call writes through.
#[test]
fn an_operand_element_read_of_a_written_buffer_is_denied_by_condition_two() {
    let source =
        br#"fn fill['d](dst: &uniq 'd buffer<u64>, mark: own u64) -> result: own u64 reads('d), writes('d) {
  let room = len(deref(dst));
  let k = 0_u64;
  loop @go {
    let done = ige(k, room);
    if done {
      break @go;
    }
    set deref(dst)[k] = mark;
    set k = k +wrap 1_u64;
  }
  return mark;
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let buf = buffer_new(4_u64, 1_u64);
  region 'd {
    let a = fill<'d>(dst: &uniq 'd buf, mark: 9_u64);
    let b = take(v: buf[0_u64]);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Footprint { kind, .. } = denial(pair, 2) else {
        panic!("expected a footprint denial, got {:?}", pair.verdict);
    };
    assert_eq!(*kind, ConflictKind::WriteOperandRead);
}

/// The caller-side half in its other direction: s1's own operand reads the
/// cell s2 writes through. Which member takes a lane is the implementation's
/// choice, so permission may not depend on it and both directions are judged.
#[test]
fn an_operand_read_by_the_first_call_of_storage_the_second_writes_is_denied() {
    let source = br#"fn take(v: own u64) -> result: own u64 pure {
  return v;
}

fn bump['r](slot: &uniq 'r u64) -> result: own u64 writes('r) {
  set deref(slot) = 7_u64;
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region 'r {
    let a = take(v: cell);
    let b = bump<'r>(slot: &uniq 'r cell);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Footprint { kind, .. } = denial(pair, 2) else {
        panic!("expected a footprint denial, got {:?}", pair.verdict);
    };
    assert_eq!(*kind, ConflictKind::OperandReadWrite);
}

/// Condition 2 in its other direction: s1 only reads, s2 writes the same
/// place. The judgment's first conflict loop never sees this pair, so the
/// second one has to.
#[test]
fn a_write_by_the_second_call_over_a_read_by_the_first_is_denied_by_condition_two() {
    let source = br#"fn peek['r](v: &'r u64) -> result: own u64 reads('r) {
  return deref(v);
}

fn bump['r](slot: &uniq 'r u64) -> result: own u64 writes('r) {
  set deref(slot) = 7_u64;
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region 'r {
    let a = peek<'r>(v: &'r cell);
    let b = bump<'r>(slot: &uniq 'r cell);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Footprint { kind, left, right } = denial(pair, 2) else {
        panic!("expected a footprint denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind::ReadWrite,
        "s1 reads and s2 writes, which is not two writes"
    );
    let (Access::Place { place: left, .. }, Access::Place { place: right, .. }) = (left, right)
    else {
        panic!("expected two conflicting places, got {left:?} and {right:?}");
    };
    assert_eq!(left, right, "both actuals resolve to the one cell");
}

/// Condition 3. The row gate refuses `external` before any place is
/// consulted, even though the two consumed actuals are disjoint.
#[test]
fn an_external_row_callee_is_denied_by_condition_three() {
    let source =
        br#"fn release_read_file(file: own ReadFile) -> result: own unit external, blocks {
  return unit;
}

fn release_pair(first: own ReadFile, second: own ReadFile) -> result: own unit external, blocks {
  let done_first = release_read_file(file: move first);
  let done_second = release_read_file(file: move second);
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "release_pair");
    let Denial::Row {
        side,
        external,
        blocks,
    } = denial(pair, 3)
    else {
        panic!("expected a row denial, got {:?}", pair.verdict);
    };
    assert_eq!(*side, PairSide::First);
    assert!(*external);
    assert!(*blocks);
}

/// Condition 4. The first statement's `propagate` right-hand side has an
/// `Err` edge straight to the function-return sink [ERR-3]; overlapping it
/// with the following write would run a write the sequential execution skips.
#[test]
fn a_propagating_first_statement_is_denied_by_condition_four() {
    let source = br#"fn narrow(v: own u32) -> result: own Result<u8, NarrowError> pure {
  return cvt<u32, u8>(v);
}

fn stamp['o](slot: &uniq 'o u8) -> result: own u64 writes('o) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe['o](v: own u32, slot: &uniq 'o u8) -> result: own Result<unit, NarrowError> writes('o) {
  let narrowed = propagate narrow(v: v);
  let stamped = stamp<'o>(slot: move slot);
  return Ok<unit, NarrowError>(value: unit);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "probe");
    let Denial::SkippingExit { kind } = denial(pair, 4) else {
        panic!("expected a skipping-exit denial, got {:?}", pair.verdict);
    };
    assert_eq!(*kind, ExitKind::PropagateError);
}

// ----------------------------------------------------------------------
// Eligibility
// ----------------------------------------------------------------------

/// The same two-child fold with one `claim` in the recursive closure. P still
/// holds — the sequentialization is a reachable-trap-site consequence, not an
/// illegality — and the table says so with the claim count and one witness,
/// so the denial is visible rather than silent.
#[test]
fn a_claim_bearing_closure_is_permitted_but_not_actualizable() {
    let source = br#"enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn boxed_leaf(w: own u64) -> result: own box<BoxNode> allocates(heap) {
  let leaf = Leaf(w: w);
  return box_new(move leaf);
}

fn boxed_branch(left: own box<BoxNode>, right: own box<BoxNode>) -> result: own box<BoxNode> allocates(heap) {
  let branch = Branch(left: move left, right: move right, w: 0_u64);
  return box_new(move branch);
}

fn bubble['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads('b), writes('b), traps {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = bubble<'b>(node: move l);
      let b = bubble<'b>(node: move r);
      let sum_defined = a +defined b;
      claim bubble_sum_fits: sum_defined because "an in-memory tree has representable widths";
      let total = a + b;
      set deref(slot) = total;
      return total;
    }
  }
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region 'tree {
    let total = bubble<'tree>(node: &uniq 'tree branch0);
    claim bubble_total: ieq(total, 7_u64) because "bubble total";
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "bubble");
    let PermissionVerdict::PermittedNotActualizable {
        claim_sites,
        witness,
    } = &pair.verdict
    else {
        panic!(
            "expected a not-actualizable verdict, got {:?}",
            pair.verdict
        );
    };
    assert_eq!(*claim_sites, 1, "one claim site in the closure of bubble");
    assert_eq!(witness.function, "bubble");
    assert_eq!(witness.claim, "bubble_sum_fits");
    assert_eq!(witness.path, vec![pair.first.callee]);
    assert!(
        function_table(&table, "bubble").runs.is_empty(),
        "a permitted pair that is not eligible never forms a chain"
    );

    // main's own two leaf allocations reach no claim, so the same program
    // still carries an eligible pair: eligibility is per closure, not per
    // program. Its next pair feeds both leaves into one branch and is denied
    // by condition 1.
    let outer = function_table(&table, "main");
    assert_eq!(outer.pairs.len(), 2);
    assert_eq!(outer.pairs[0].verdict, PermissionVerdict::PermittedEligible);
    assert_eq!(outer.pairs[1].verdict.denied_condition(), Some(1));
}
