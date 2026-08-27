//! The permission judgment P over sibling call statements.
//!
//! Each grant fixture is a shape a real program writes; each denial fixture
//! violates exactly one numbered condition and asserts *that* condition, so a
//! denial arriving for the wrong reason fails the test. Design:
//! `research/investigations/proof-derived-parallelism/DESIGN.md` section 3.
//!
//! The window fixtures at the end put statements *between* the two calls. They
//! are deliberately weighted toward denials: widening the judged set is the
//! easy half, and the whole risk of the widening is a window that should have
//! been refused and was not. Each of those names the clause that refuses it,
//! and one grant fixture pins the single obligation the rule deliberately does
//! *not* carry, so making the rule symmetric would fail a test rather than
//! pass silently.

use crate::SemanticOutcome;

use super::super::permission::{
    ConflictKind, Denial, ExitKind, FootprintHalf, FunctionPermissions, PairSide,
    PermissionMetadata, PermissionPair, PermissionVerdict,
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

/// Direct system operations are ordinary permission candidates. Their
/// compiler-owned execution contract is retained for lowering, but authority
/// overlap is decided solely from the concrete actual places.
#[test]
fn independent_direct_output_operations_are_permitted() {
    let source = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  region 'out {
    region 'err {
      region 'source {
        let first = write_once<'out, 'source>(output: &uniq 'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
        let second = write_once<'err, 'source>(output: &uniq 'err err, source: &'source bytes, start: 1_u64, end: 2_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    assert_eq!(pair.first.callee_name, "write_once");
    assert_eq!(pair.second.callee_name, "write_once");
    assert_eq!(pair.first.target_action, crate::TargetAction::MAY_SUSPEND);
    assert_eq!(pair.second.target_action, crate::TargetAction::MAY_SUSPEND);
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
}

/// One Output is an ordinary mutable state object. Two loans covering the
/// same named region therefore fail before overlap permission is considered.
#[test]
fn direct_output_operations_on_one_state_cannot_hold_two_unique_loans() {
    let source = br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  region 'out {
    region 'source {
      let first = write_once<'out, 'source>(output: &uniq 'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
      let second = write_once<'out, 'source>(output: &uniq 'out out, source: &'source bytes, start: 1_u64, end: 2_u64);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected an ordinary loan conflict, got {:?}", pair.verdict);
    };
    assert_eq!(kind.halves(), ("exclusive loan", "exclusive loan"));
    assert_eq!(*sides, (PairSide::First, PairSide::Second));
}

#[test]
fn completion_waits_for_the_exact_nonadjacent_unique_loan() {
    let source = br#"command fn main(command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let bytes = buffer_new(3_u64, 65_u8);
  region 'out {
    region 'err {
      region 'source {
        let first = write_once<'out, 'source>(output: &uniq 'out out, source: &'source bytes, start: 0_u64, end: 1_u64);
        let middle = write_once<'err, 'source>(output: &uniq 'err err, source: &'source bytes, start: 1_u64, end: 2_u64);
        let last = write_once<'out, 'source>(output: &uniq 'out out, source: &'source bytes, start: 2_u64, end: 3_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let steps = &function_table(&table, "main").completion_steps;
    assert_eq!(
        steps.len(),
        3,
        "the adjacent eligible pairs form one schedule"
    );
    assert!(steps[0].has_later_independent_call);
    assert!(steps[1].has_later_independent_call);
    assert!(!steps[2].has_later_independent_call);
    assert!(steps[0].wait_for.is_empty());
    assert!(steps[1].wait_for.is_empty());
    assert_eq!(steps[2].wait_for, vec![steps[0].site.binding]);
}

/// Two short exclusive factory loans mint independent ordinary owners. Once
/// those calls return, the permits can feed two opens through shared loans of
/// one directory without retaining either factory loan.
#[test]
fn independent_permits_allow_opens_through_one_shared_directory() {
    let source = br#"fn open_two['directory](first_permit: own FilePermit, second_permit: own FilePermit, directory: &'directory DirectoryRead) -> result: own unit reads(first_permit, second_permit, directory), writes(first_permit, second_permit) {
  let first = open_directory_source<'directory>(permit: move first_permit, directory: directory);
  let second = open_directory_source<'directory>(permit: move second_permit, directory: directory);
  return unit;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region 'state {
    let first_permit = reserve_file<'state>(factory: &uniq 'state files);
    let second_permit = reserve_file<'state>(factory: &uniq 'state files);
    open_two<'state>(first_permit: move first_permit, second_permit: move second_permit, directory: &'state cwd);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "open_two");
    assert_eq!(pair.first.callee_name, "open_directory_source");
    assert_eq!(pair.second.callee_name, "open_directory_source");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
}

/// Positioned reads reserve free fragments on the same file root. Shared
/// file loans coexist, while the two destination loans remain disjoint.
#[test]
fn positioned_reads_on_one_file_with_disjoint_destinations_are_permitted() {
    let source = br#"fn probe(file: own ReadFile) -> result: own unit reads(file), writes(file), allocates(heap) {
  let left = buffer_new(1_u64, 0_u8);
  let right = buffer_new(1_u64, 0_u8);
  region 'file {
    region 'left {
      region 'right {
        let first = read_at<'file, 'left>(file: &'file file, destination: &uniq 'left left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
        let second = read_at<'file, 'right>(file: &'file file, destination: &uniq 'right right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
      }
    }
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "probe");
    assert_eq!(pair.first.callee_name, "read_at");
    assert_eq!(pair.second.callee_name, "read_at");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
}

/// A direct inline system operation is not mistaken for an unknown call and
/// therefore needs neither a wrapper nor a special source marker.
#[test]
fn direct_inline_system_operations_form_an_eligible_pair() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let first = exit_status(code: 0_u8);
  let second = exit_status(code: 1_u8);
  return move second;
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    assert_eq!(pair.first.target_action, crate::TargetAction::INLINE);
    assert_eq!(pair.second.target_action, crate::TargetAction::INLINE);
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
}

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

fn fold['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads(node), writes(node) {
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

#[test]
fn disjoint_effect_fields_do_not_shrink_a_whole_object_unique_loan() {
    let source = br#"struct Pair {
  left: u64;
  right: u64;
}

fn set_left['r](pair: &uniq 'r Pair) -> result: own unit writes(pair.left) {
  set deref(pair).left = 1_u64;
  return unit;
}

fn set_right['r](pair: &uniq 'r Pair) -> result: own unit writes(pair.right) {
  set deref(pair).right = 2_u64;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let pair = Pair(left: 0_u64, right: 0_u64);
  region 'r {
    let first = set_left<'r>(pair: &uniq 'r pair);
    let second = set_right<'r>(pair: &uniq 'r pair);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Loan { kind, .. } = denial(pair, 2) else {
        panic!("the whole-object unique loans must remain independent of effect precision");
    };
    assert_eq!(kind.halves(), ("exclusive loan", "exclusive loan"));
}

/// Read-only sibling recursion. Nothing is written at all, so condition 2 is
/// satisfied by an empty write footprint rather than by disjointness.
#[test]
fn read_only_sibling_recursion_is_permitted_and_eligible() {
    let source = br#"enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn depth['b](node: &'b box<BoxNode>) -> result: own u64 reads(node) {
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
    let source = br#"fn width['r](data: &'r buffer<u64>) -> result: own u64 reads(data) {
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
    let source = br#"fn bump['r](slot: &uniq 'r u64) -> result: own u64 reads(slot), writes(slot) {
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

fn fold_shift['b](node: &uniq 'b box<BoxNode>, shift: own u64) -> result: own u64 reads(node), writes(node) {
  let base = fold<'b>(node: move node);
  return imax(base, shift);
}

fn fold['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads(node), writes(node) {
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
    let Denial::Dataflow {
        binding,
        definer,
        reader,
    } = denial(pair, 1)
    else {
        panic!("expected a dataflow denial, got {:?}", pair.verdict);
    };
    assert_eq!(*binding, pair.first.binding, "cited the wrong binding");
    assert_eq!(
        (*definer, *reader),
        (PairSide::First, PairSide::Second),
        "the link runs from s1 to s2, with nothing between them"
    );
}

/// Condition 2. Two `&uniq` actuals resolve to one and the same place, so the
/// two write footprints overlap under [OWN-7].
#[test]
fn overlapping_unique_arguments_are_denied_by_condition_two() {
    let source = br#"fn bump['r](slot: &uniq 'r u64) -> result: own u64 reads(slot), writes(slot) {
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
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::ExclusiveLoan,
            later: FootprintHalf::ExclusiveLoan
        },
        "each actual is a `&uniq` borrow of the one cell, and the loan is the cause the row write is downstream of"
    );
    assert_eq!(
        *sides,
        (PairSide::First, PairSide::Second),
        "the conflict is between the two members, not with anything between them"
    );
}

/// Condition 2, the caller-side half. `take`'s row is `pure` and reaches no
/// caller storage at all, but its own operand reads the cell `bump` writes,
/// and the overlap moves exactly that read across `bump`'s call. Before this
/// condition existed the pair was permitted, eligible, and produced the
/// pre-write value with no runtime linked.
#[test]
fn an_operand_read_of_written_storage_is_denied_by_condition_two() {
    let source = br#"fn bump['r](slot: &uniq 'r u64) -> result: own u64 writes(slot) {
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
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::ExclusiveLoan,
            later: FootprintHalf::OperandRead
        }
    );
    assert_eq!(*sides, (PairSide::First, PairSide::Second));
}

/// The same hazard through a subscript rather than a whole binding: the
/// element read is rooted at the buffer the first call writes through.
#[test]
fn an_operand_element_read_of_a_written_buffer_is_denied_by_condition_two() {
    let source =
        br#"fn fill['d](dst: &uniq 'd buffer<u64>, mark: own u64) -> result: own u64 reads(dst), writes(dst) {
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
    let Denial::Loan { kind, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::ExclusiveLoan,
            later: FootprintHalf::OperandRead
        }
    );
}

/// The caller-side half in its other direction: s1's own operand reads the
/// cell s2 writes through. Which member takes a lane is the implementation's
/// choice, so permission may not depend on it and both directions are judged.
#[test]
fn an_operand_read_by_the_first_call_of_storage_the_second_writes_is_denied() {
    let source = br#"fn take(v: own u64) -> result: own u64 pure {
  return v;
}

fn bump['r](slot: &uniq 'r u64) -> result: own u64 writes(slot) {
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
    let Denial::Loan { kind, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::OperandRead,
            later: FootprintHalf::ExclusiveLoan
        }
    );
}

/// Condition 2 in its other direction: s1 only reads, s2 writes the same
/// place. The judgment's first conflict loop never sees this pair, so the
/// second one has to.
#[test]
fn a_write_by_the_second_call_over_a_read_by_the_first_is_denied_by_condition_two() {
    let source = br#"fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn bump['r](slot: &uniq 'r u64) -> result: own u64 writes(slot) {
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
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::SharedLoan,
            later: FootprintHalf::ExclusiveLoan
        },
        "s1's shared borrow and s2's `&uniq` of the one cell conflict as loans before either row is consulted"
    );
    assert_eq!(*sides, (PairSide::First, PairSide::Second));
}

/// Target suspension is not a permission denial. Two calls consuming distinct
/// capabilities remain eligible; lowering chooses the completion route.
#[test]
fn may_suspend_release_wrappers_on_distinct_capabilities_are_permitted() {
    let source = br#"fn release_read_file(file: own ReadFile) -> result: own unit writes(file) {
  return unit;
}

fn release_pair(first: own ReadFile, second: own ReadFile) -> result: own unit writes(first, second) {
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
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
    assert_eq!(pair.first.target_action, crate::TargetAction::MAY_SUSPEND);
    assert_eq!(pair.second.target_action, crate::TargetAction::MAY_SUSPEND);
}

/// Condition 4. The first statement's `propagate` right-hand side has an
/// `Err` edge straight to the function-return sink [ERR-3]; overlapping it
/// with the following write would run a write the sequential execution skips.
#[test]
fn a_propagating_first_statement_is_denied_by_condition_four() {
    let source = br#"fn narrow(v: own u32) -> result: own Result<u8, NarrowError> pure {
  return cvt<u32, u8>(v);
}

fn stamp['o](slot: &uniq 'o u8) -> result: own u64 writes(slot) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe['o](v: own u32, slot: &uniq 'o u8) -> result: own Result<unit, NarrowError> writes(slot) {
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
    let Denial::SkippingExit { side, kind } = denial(pair, 4) else {
        panic!("expected a skipping-exit denial, got {:?}", pair.verdict);
    };
    assert_eq!(*kind, ExitKind::PropagateError);
    assert_eq!(
        *side,
        PairSide::First,
        "the propagating statement is s1 itself, not one between the two"
    );
}

// ----------------------------------------------------------------------
// Claims in the call closure
// ----------------------------------------------------------------------

/// The same two-child fold with one `claim` in the recursive closure. This is
/// the case an earlier judgment permitted but refused to actualize, so that no
/// schedule could choose which claim traps first. It is eligible now: a false
/// executed claim is a contract violation [SCOPE-4], an execution reaching one
/// is erroneous, and a correct program — this one — traps under no schedule at
/// all. The four conditions are the whole judgment.
///
/// The claim sits in `scaled`, one call deeper, because v0.34 admits only a
/// local non-derivable residual and the fold's own `a + b` overflow guard was
/// a statement about the callers rather than a lemma. Depth is what this case
/// needs anyway: the claim is reached through the ordinary call graph from
/// both judged callees, which is exactly the closure the retired gate walked.
#[test]
fn a_claim_bearing_closure_is_eligible() {
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

fn scaled(values: own array<u8, 8>, index: own u64) -> result: own u8 traps {
  let size = len(values);
  let bounded = 0_u64;
  loop @select_bound {
    if ieq(bounded, index) {
      break @select_bound;
    } else if ieq(bounded, 7_u64) {
      break @select_bound;
    } else {
      set bounded = bounded +wrap 1_u64;
    }
  }
  let inside = ilt(bounded, size);
  claim index_in_range: inside because "premises: bounded starts at zero, advances by one only on this function's ordinary-loop backedge, and exits no later than seven; values has length eight\nderivation: induction over reached loop bodies keeps bounded between zero and seven inclusive\nconclusion: ilt(bounded, size) is true\nchecker gap: ENT carries no induction fact across this ordinary-loop backedge\nconsumers: the following length-eight array subscript uses bounded";
  return values[bounded];
}

fn bubble['b](node: &uniq 'b box<BoxNode>) -> result: own u64 reads(node), writes(node), traps {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      let w = deref(leaf_w);
      let values = array_new<u8, 8>(0_u8);
      let touched = scaled(values: move values, index: w);
      return w;
    }
    Branch(left: l, right: r, w: slot) => {
      let a = bubble<'b>(node: move l);
      let b = bubble<'b>(node: move r);
      let total = a +wrap b;
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
    if ieq(total, 7_u64) {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "bubble");
    assert_eq!(
        pair.verdict,
        PermissionVerdict::PermittedEligible,
        "a claim reachable from the callees is not a reason to refuse"
    );
    let runs = &function_table(&table, "bubble").runs;
    assert_eq!(
        runs.len(),
        1,
        "an eligible pair forms its chain like any other: {runs:?}"
    );

    // The claim really is in the closure of the judged pair, so what the case
    // above pins is the redirect and not an accident of this fixture: `bubble`
    // calls itself, and its leaf arm calls `scaled`, which carries
    // `index_in_range`.
    assert_eq!(pair.first.callee_name, "bubble");
    assert_eq!(pair.second.callee_name, "bubble");
    assert!(
        std::str::from_utf8(source)
            .expect("the fixture is UTF-8")
            .contains("claim index_in_range:"),
        "the fixture must keep the claim whose closure this case is about"
    );

    // main's next pair feeds both leaves into one branch and is still denied
    // by condition 1: widening actualization moved no condition.
    let outer = function_table(&table, "main");
    assert_eq!(outer.pairs.len(), 2);
    assert_eq!(outer.pairs[0].verdict, PermissionVerdict::PermittedEligible);
    assert_eq!(outer.pairs[1].verdict.denied_condition(), Some(1));
}

// ----------------------------------------------------------------------
// The window: statements written between the two calls
// ----------------------------------------------------------------------

/// The F3 shape. One pure builtin between the two recursive calls, reading a
/// local the calls do not reach and defining a binding neither of them reads.
/// Before the window rule this ended the enumeration: no pair, no verdict, no
/// ledger line — so the same fold with the operation wrapped in a function
/// kept a parallel chain and this one silently did not.
#[test]
fn a_pure_builtin_between_two_calls_keeps_the_pair() {
    let source = br#"enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn fold['b](node: &uniq 'b box<BoxNode>, seed: own u64) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold<'b>(node: move l, seed: seed);
      let gap = seed +wrap 1_u64;
      let b = fold<'b>(node: move r, seed: seed);
      let kids = imax(a, b);
      let total = imax(kids, gap);
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
    assert_eq!(pair.first.callee_name, "fold");
    assert_eq!(pair.second.callee_name, "fold");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
    let runs = &function_table(&table, "fold").runs;
    assert_eq!(
        runs.len(),
        1,
        "the two calls form one chain across the builtin"
    );
    assert_eq!(runs[0].sites.len(), 2);
}

/// Condition 2, clause 2c. The interposed `set` writes the storage s2's callee
/// reads through its actual. Under the schedule that hands s2 to a lane, that
/// read races the store and takes the pre-`set` value where source order
/// requires the post-`set` one.
#[test]
fn an_interposed_write_into_the_second_callees_read_is_denied_by_condition_two() {
    let source = br#"fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region 'r {
    let a = peek<'r>(v: &'r other);
    set cell = 5_u64;
    let b = peek<'r>(v: &'r cell);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::Write,
            later: FootprintHalf::SharedLoan
        }
    );
    assert_eq!(
        *sides,
        (PairSide::Between(0), PairSide::Second),
        "the interposed write, not s1, is what conflicts with s2"
    );
}

/// Condition 2, clause 2a. The interposed `set` writes the storage s1's callee
/// writes through its actual. Under the schedule that hands s1 out this is a
/// live store/store race between the lane and the calling thread.
#[test]
fn an_interposed_write_over_the_first_callees_write_is_denied_by_condition_two() {
    let source = br#"fn bump['r](slot: &uniq 'r u64) -> result: own u64 writes(slot) {
  set deref(slot) = 7_u64;
  return 1_u64;
}

fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region 'r {
    let a = bump<'r>(slot: &uniq 'r cell);
    set cell = 5_u64;
    let b = peek<'r>(v: &'r other);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::ExclusiveLoan,
            later: FootprintHalf::Write
        }
    );
    assert_eq!(
        *sides,
        (PairSide::First, PairSide::Between(0)),
        "the conflict is s1 against the interposed write"
    );
}

/// Condition 2, clause 2c's operand half — the obligation the window adds that
/// no pair rule has. `take`'s row is `pure` and reaches no caller storage at
/// all, but the schedule that hands s2 to a lane evaluates its operands at the
/// hand-out point, above the interposed `set`, so it reads 1 where source
/// order gives 15. No callee row is involved on either side.
#[test]
fn an_interposed_write_under_the_second_calls_operand_read_is_denied_by_condition_two() {
    let source = br#"fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region 'r {
    let a = peek<'r>(v: &'r other);
    set cell = 15_u64;
    let b = take(v: cell);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Footprint { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a footprint denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::Write,
            later: FootprintHalf::OperandRead
        }
    );
    assert_eq!(*sides, (PairSide::Between(0), PairSide::Second));
}

/// The mirror of the fixture above, and the one obligation the window rule
/// deliberately does **not** carry: an interposed write over *s1's* operand
/// read is permitted.
///
/// Both realizable schedules agree. Where s1 takes the lane, its operands are
/// evaluated before the fork, so the lane already holds the value 1 and the
/// later store cannot reach it. Where s2 takes the lane, s1 has run to
/// completion before the interposed statement executes at all. Neither
/// schedule lets the store move above the read, so the operand half is
/// one-sided for s1 and two-sided for s2.
///
/// This fixture exists to fail if that asymmetry is ever "tidied" into a
/// symmetric rule: the widening would still be sound, but it would silently
/// stop admitting the shape the asymmetry was derived to admit.
#[test]
fn an_interposed_write_over_the_first_calls_operand_read_is_permitted() {
    let source = br#"fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region 'r {
    let a = take(v: cell);
    set cell = 15_u64;
    let b = peek<'r>(v: &'r other);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
}

/// Condition 1, clause 1b. The interposed statement reads the binding s1
/// defines. Under the schedule that hands s1 out that value does not exist
/// until the join, which is after every interposed statement has run.
#[test]
fn an_interposed_read_of_the_first_calls_result_is_denied_by_condition_one() {
    let source = br#"enum BoxNode {
  Leaf(w: u64);
  Branch(left: box<BoxNode>, right: box<BoxNode>, w: u64);
}

fn fold['b](node: &uniq 'b box<BoxNode>, seed: own u64) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold<'b>(node: move l, seed: seed);
      let gap = a +wrap 1_u64;
      let b = fold<'b>(node: move r, seed: seed);
      let kids = imax(a, b);
      let total = imax(kids, gap);
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
    let Denial::Dataflow {
        binding,
        definer,
        reader,
    } = denial(pair, 1)
    else {
        panic!("expected a dataflow denial, got {:?}", pair.verdict);
    };
    assert_eq!(*binding, pair.first.binding, "s1's own result is the link");
    assert_eq!((*definer, *reader), (PairSide::First, PairSide::Between(0)));
}

/// Condition 1, clause 1c, and the one real cost of taking the
/// schedule-independent rule: s2 reads a binding an interposed statement
/// defines. Under the schedule that hands s2 out, its operands are evaluated
/// before that statement runs, so the value it would read does not exist yet.
/// The backend's current schedule would survive this window; the rule may not
/// be stated in terms of a schedule, so it denies.
#[test]
fn a_second_call_reading_an_interposed_binding_is_denied_by_condition_one() {
    let source = br#"fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let other = 2_u64;
  region 'r {
    let a = peek<'r>(v: &'r other);
    let seed = 7_u64;
    let b = take(v: seed);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Dataflow {
        definer, reader, ..
    } = denial(pair, 1)
    else {
        panic!("expected a dataflow denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        (*definer, *reader),
        (PairSide::Between(0), PairSide::Second)
    );
}

/// Condition 4 through the window. The interposed `propagate` has an `Err`
/// edge to the function-return sink [ERR-3], so on that edge s2 never runs and
/// the function returns while s1's lane is still executing against a frame the
/// return is about to destroy. That is a use-after-return, not a value
/// difference, and before the window rule it was not even reported.
#[test]
fn an_interposed_propagate_is_denied_by_condition_four() {
    let source = br#"fn peek['o](slot: &'o u8) -> result: own u64 reads(slot) {
  return cvt<u8, u64>(deref(slot));
}

fn stamp['o](slot: &uniq 'o u8) -> result: own u64 writes(slot) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe['o](outcome: own Result<u8, NarrowError>, a: &uniq 'o u8, b: &'o u8) -> result: own Result<unit, NarrowError> reads(b), writes(a) {
  let seen = peek<'o>(slot: b);
  let narrowed = propagate outcome;
  let stamped = stamp<'o>(slot: move a);
  return Ok<unit, NarrowError>(value: unit);
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "probe");
    let Denial::SkippingExit { side, kind } = denial(pair, 4) else {
        panic!("expected a skipping-exit denial, got {:?}", pair.verdict);
    };
    assert_eq!(*kind, ExitKind::PropagateError);
    assert_eq!(*side, PairSide::Between(0));
}

/// Condition 4, and the one place a `claim` still refuses a window. A claim
/// inside a callee is no longer a reason to refuse anything; this one is in
/// the caller's own block, *between* the two calls, and carries a trap edge to
/// the [DIAG-3] sink. It is an exit out of the window like a `return` or a
/// `propagate` `Err` edge, and an exit taken there abandons an unjoined lane
/// still reading the caller's frame. Nothing in the redirect touches it.
///
/// The window sits in `probe` rather than in `main` because the claim has to
/// be one v0.34 admits: a local, non-derivable, load-bearing residual. The
/// clamp bound is a parameter and the length is the parameter array's, so
/// neither endpoint is a constant the checker can fold, and the subscript that
/// follows is what consumes it. The ordinary subscript between the calls is
/// the control: it interposes too, and only the claim's trap edge denies.
#[test]
fn an_interposed_claim_is_denied_by_condition_four() {
    let source = br#"fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn probe['r](values: own array<u8, 8>, index: own u64, cell: &'r u64, other: &'r u64) -> result: own u64 reads(cell, other), traps {
  let size = len(values);
  let bounded = 0_u64;
  loop @select_bound {
    if ieq(bounded, index) {
      break @select_bound;
    } else if ieq(bounded, 7_u64) {
      break @select_bound;
    } else {
      set bounded = bounded +wrap 1_u64;
    }
  }
  let inside = ilt(bounded, size);
  let a = peek<'r>(v: other);
  claim index_in_range: inside because "premises: bounded starts at zero, advances by one only on this function's ordinary-loop backedge, and exits no later than seven; values has length eight\nderivation: induction over reached loop bodies keeps bounded between zero and seven inclusive\nconclusion: ilt(bounded, size) is true\nchecker gap: ENT carries no induction fact across this ordinary-loop backedge\nconsumers: the following length-eight array subscript uses bounded";
  let picked = values[bounded];
  let b = peek<'r>(v: cell);
  return imax(a, b);
}

command fn main() -> status: own ExitStatus traps {
  let cell = 1_u64;
  let other = 2_u64;
  let table = array_new<u8, 8>(0_u8);
  region 'r {
    let total = probe<'r>(values: move table, index: 3_u64, cell: &'r cell, other: &'r other);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "probe");
    let Denial::SkippingExit { side, kind } = denial(pair, 4) else {
        panic!(
            "a claim between the calls must deny, not fall through to eligibility: {:?}",
            pair.verdict
        );
    };
    assert_eq!(*kind, ExitKind::ClaimTrap);
    assert_eq!(*side, PairSide::Between(0));
}

/// A form the window rule does not account for is refused **with a report**.
/// This is the half of F3 that a verdict-only suite is blind to: before the
/// window rule this program produced no pair at all, so nothing was wrong with
/// any verdict — there was no verdict. Fail-closed and silent are different
/// defects, and only the second one is fixed by denying.
#[test]
fn an_interposed_match_is_denied_by_condition_two() {
    let source = br#"enum Choice {
  Low(w: u64);
  High(w: u64);
}

fn peek['r](v: &'r u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  let which = Low(w: 3_u64);
  region 'r {
    let a = peek<'r>(v: &'r other);
    match which {
      Low(w: lw) => {
        let seen = lw;
      }
      High(w: hw) => {
        let seen = hw;
      }
    }
    let b = peek<'r>(v: &'r cell);
    let total = imax(a, b);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::InterposedForm { side, form } = denial(pair, 2) else {
        panic!(
            "a match between the calls must be reported, not silently unjudged: {:?}",
            pair.verdict
        );
    };
    assert_eq!(*side, PairSide::Between(0));
    assert_eq!(*form, "a match statement");
}

// ----------------------------------------------------------------------
// Condition 2, the loans half [OWN-5, OWN-12]
// ----------------------------------------------------------------------

/// The pointed case of the loans half: both callees declare `reads` only, so
/// the row projection alone sees read against read and would permit — but each
/// actual is a `&uniq` borrow of the one cell, and an overlap would hold two
/// usable exclusive loans on one place, which [OWN-5] never admits at one
/// program point. Before the loans half existed this pair was permitted.
#[test]
fn read_only_unique_borrows_of_one_place_are_denied_by_their_loans() {
    let source = br#"fn peek_uniq['c](cell: &uniq 'c u64) -> result: own u64 reads(cell) {
  return deref(cell);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 21_u64;
  region 'c {
    let a = peek_uniq<'c>(cell: &uniq 'c cell);
    let b = peek_uniq<'c>(cell: &uniq 'c cell);
    let both = a +wrap b;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::ExclusiveLoan,
            later: FootprintHalf::ExclusiveLoan
        }
    );
    assert_eq!(*sides, (PairSide::First, PairSide::Second));
}

/// A shared loan against a consuming `move`: s1 holds `&'c` of the box while
/// its row is `pure`, and s2 consumes the box, whose drop frees the heap
/// block. The row projection sees nothing on s1's side at all; the loan is
/// the only thing standing between the overlap and a read of freed storage.
#[test]
fn a_pure_shared_borrow_against_a_consuming_move_is_denied_by_its_loan() {
    let source = br#"fn ignore_box['c](node: &'c box<u64>) -> result: own u64 pure {
  return 7_u64;
}

fn eat_box(node: own box<u64>) -> result: own u64 pure {
  return 9_u64;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let node = box_new(41_u64);
  region 'c {
    let a = ignore_box<'c>(node: &'c node);
    let b = eat_box(node: move node);
    let both = a +wrap b;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::Loan { kind, sides, .. } = denial(pair, 2) else {
        panic!("expected a loan denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *kind,
        ConflictKind {
            earlier: FootprintHalf::SharedLoan,
            later: FootprintHalf::Write
        }
    );
    assert_eq!(*sides, (PairSide::First, PairSide::Second));
}

/// An interposed statement that forms a borrow is refused as a form: the
/// checked tree erases a written borrow's shared-or-uniq mode, so the loan it
/// would hold across the window cannot be stated, and an unloaned borrow
/// would widen permission. The refusal, not an empty footprint, is what the
/// window reports.
#[test]
fn an_interposed_borrow_binding_refuses_the_window() {
    let source = br#"fn bump['r](cell: &uniq 'r u64) -> result: own u64 reads(cell), writes(cell) {
  let was = deref(cell);
  set deref(cell) = was +wrap 1_u64;
  return was;
}

fn takeval(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region 'c {
    let a = bump<'c>(cell: &uniq 'c cell);
    let g = &'c cell;
    let b = takeval(v: 5_u64);
    let seen = deref(g);
    let sum = a +wrap b;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::InterposedForm { form, .. } = denial(pair, 2) else {
        panic!("expected an interposed-form denial, got {:?}", pair.verdict);
    };
    assert_eq!(*form, "a statement that forms a borrow");
}

/// A borrow-moded actual whose place the judgment cannot resolve: a `&uniq`
/// of an own slice binding anchors nowhere `argument_place` reaches, and the
/// loans half fails closed on it even though both rows are `pure` and project
/// nothing.
#[test]
fn an_unresolvable_loan_actual_denies_rather_than_dropping_the_loan() {
    let source =
        br#"fn touch_uniqslice['d, 'r](v: &uniq 'd slice<'r, u8>) -> result: own u64 pure {
  return 3_u64;
}

fn a_pure_uniqslice() -> result: own u64 allocates(heap) {
  let buf = buffer_new(8_u64, 1_u8);
  region 'r {
    let v = slice_of(&'r buf);
    region 'd {
      let a = touch_uniqslice<'d, 'r>(v: &uniq 'd v);
      let b = touch_uniqslice<'d, 'r>(v: &uniq 'd v);
      let s = a +wrap b;
      return s;
    }
  }
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let p = a_pure_uniqslice();
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "a_pure_uniqslice");
    let Denial::UnresolvedFootprint { .. } = denial(pair, 2) else {
        panic!("expected an unresolved denial, got {:?}", pair.verdict);
    };
}

/// Direct system operations are candidates under the same ordinary call
/// permission judgment. An inline, authority-free operation therefore forms
/// the two adjacent eligible pairs rather than becoming opaque interposition.
#[test]
fn an_inline_system_operation_forms_ordinary_adjacent_windows() {
    let source = br#"fn quiet['c](cell: &uniq 'c u64) -> result: own u64 pure {
  return 3_u64;
}

fn interposed_pure_syscall(x: own u64, name: own HostString) -> result: own u64 pure {
  let p = x;
  let r = x;
  region 'c {
    let a = quiet<'c>(cell: &uniq 'c p);
    let path = relative_path(value: move name);
    let b = quiet<'c>(cell: &uniq 'c r);
    let s = a +wrap b;
    match path {
      Ok(value: good) => {
        return s;
      }
      Err(error: bad) => {
        return s +wrap 1_u64;
      }
    }
  }
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let permissions = function_table(&table, "interposed_pure_syscall");
    assert_eq!(permissions.pairs.len(), 2);
    assert_eq!(permissions.pairs[0].second.callee_name, "relative_path");
    assert_eq!(permissions.pairs[1].first.callee_name, "relative_path");
    assert!(
        permissions
            .pairs
            .iter()
            .all(|pair| pair.verdict == PermissionVerdict::PermittedEligible)
    );
}
