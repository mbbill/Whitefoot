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

use crate::{SemanticOutcome, SemanticRule};

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
    let source = br#"command fn main(command.stdout as out: own OutputStream, command.stderr as err: own OutputStream) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  region 'out {
    region 'err {
      region {
        let first = write_once(output: &uniq 'out out, source: &bytes, start: 0_u64, end: 1_u64);
        let second = write_once(output: &uniq 'err err, source: &bytes, start: 1_u64, end: 2_u64);
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

/// One OutputStream is an ordinary mutable state object. Two loans covering the
/// same named region therefore fail before overlap permission is considered.
#[test]
fn direct_output_operations_on_one_state_cannot_hold_two_unique_loans() {
    let source = br#"command fn main(command.stdout as out: own OutputStream) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  region 'out {
    region {
      let first = write_once(output: &uniq 'out out, source: &bytes, start: 0_u64, end: 1_u64);
      let second = write_once(output: &uniq 'out out, source: &bytes, start: 1_u64, end: 2_u64);
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
    let source = br#"command fn main(command.stdout as out: own OutputStream, command.stderr as err: own OutputStream) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let bytes = buffer_new(3_u64, 65_u8);
  region 'out {
    region 'err {
      region {
        let first = write_once(output: &uniq 'out out, source: &bytes, start: 0_u64, end: 1_u64);
        let middle = write_once(output: &uniq 'err err, source: &bytes, start: 1_u64, end: 2_u64);
        let last = write_once(output: &uniq 'out out, source: &bytes, start: 2_u64, end: 3_u64);
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
    assert_eq!(steps[2].wait_for, vec![steps[0].site.call.clone()]);
}

/// Two short exclusive factory loans mint independent ordinary owners. Once
/// those calls return, the permits can feed two opens through shared loans of
/// one directory without retaining either factory loan.
#[test]
fn independent_permits_allow_opens_through_one_shared_directory() {
    let source = br#"fn open_two(first_permit: own HandlePermit, second_permit: own HandlePermit, directory: &DirectoryRead) -> result: own unit reads(first_permit, second_permit, directory), writes(first_permit, second_permit) {
  let first = open_directory_source(permit: move first_permit, directory: directory);
  let second = open_directory_source(permit: move second_permit, directory: directory);
  return unit;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.handles as files: own HandleFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files) {
  region {
    match reserve_handle(factory: &uniq files) {
      Ok(value: first_permit) => {
        match reserve_handle(factory: &uniq files) {
          Ok(value: second_permit) => {
            open_two(first_permit: move first_permit, second_permit: move second_permit, directory: &cwd);
          }
          Err(error: spent) => {
            return exit_status(code: 8_u8);
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
      region {
        let first = read_at(file: &'file file, destination: &uniq 'left left, file_offset: 0_u64, start: 0_u64, end: 1_u64);
        let second = read_at(file: &'file file, destination: &uniq right, file_offset: 1_u64, start: 0_u64, end: 1_u64);
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

fn fold(node: &uniq box<BoxNode>) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold(node: move l);
      let b = fold(node: move r);
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
  region {
    let total = fold(node: &uniq branch0);
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

fn set_left(pair: &uniq Pair) -> result: own unit writes(pair.left) {
  set deref(pair).left = 1_u64;
  return unit;
}

fn set_right(pair: &uniq Pair) -> result: own unit writes(pair.right) {
  set deref(pair).right = 2_u64;
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  let pair = Pair(left: 0_u64, right: 0_u64);
  region {
    let first = set_left(pair: &uniq pair);
    let second = set_right(pair: &uniq pair);
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

fn depth(node: &box<BoxNode>) -> result: own u64 reads(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return 1_u64;
    }
    Branch(left: l, right: r, w: slot) => {
      let a = depth(node: l);
      let b = depth(node: r);
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
    let source = br#"fn width(data: &buffer<u64>) -> result: own u64 reads(data) {
  return len(deref(data));
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let buf = buffer_new(8_u64, 1_u64);
  region {
    let lo = width(data: &buf);
    let mid = width(data: &buf);
    let hi = width(data: &buf);
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
    let source = br#"fn bump(slot: &uniq u64) -> result: own u64 reads(slot), writes(slot) {
  let seen = deref(slot);
  set deref(slot) = 7_u64;
  return seen;
}

command fn main() -> status: own ExitStatus pure {
  let first = 1_u64;
  let second = 2_u64;
  region {
    let a = bump(slot: &uniq first);
    let b = bump(slot: &uniq second);
    let c = bump(slot: &uniq first);
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

fn fold_shift(node: &uniq box<BoxNode>, shift: own u64) -> result: own u64 reads(node), writes(node) {
  let base = fold(node: move node);
  return imax(base, shift);
}

fn fold(node: &uniq box<BoxNode>) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold(node: move l);
      let b = fold_shift(node: move r, shift: a);
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
    assert_eq!(
        Some(*binding),
        pair.first.binding,
        "cited the wrong binding"
    );
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
    let source = br#"fn bump(slot: &uniq u64) -> result: own u64 reads(slot), writes(slot) {
  let seen = deref(slot);
  set deref(slot) = 7_u64;
  return seen;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region {
    let lo = bump(slot: &uniq cell);
    let hi = bump(slot: &uniq cell);
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
    let source = br#"fn bump(slot: &uniq u64) -> result: own u64 writes(slot) {
  set deref(slot) = 15_u64;
  return 1_u64;
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region {
    let a = bump(slot: &uniq cell);
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
        br#"fn fill(dst: &uniq buffer<u64>, mark: own u64) -> result: own u64 reads(dst), writes(dst) {
  let room = len(deref(dst));
  let k = 0_u64;
  loop @go {
    let done = k >= room;
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
  region {
    let a = fill(dst: &uniq buf, mark: 9_u64);
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

fn bump(slot: &uniq u64) -> result: own u64 writes(slot) {
  set deref(slot) = 7_u64;
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region {
    let a = take(v: cell);
    let b = bump(slot: &uniq cell);
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
    let source = br#"fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn bump(slot: &uniq u64) -> result: own u64 writes(slot) {
  set deref(slot) = 7_u64;
  return 1_u64;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region {
    let a = peek(v: &cell);
    let b = bump(slot: &uniq cell);
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
  return cvt::<u32, u8>(v);
}

fn stamp(slot: &uniq u8) -> result: own u64 writes(slot) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe(v: own u32, slot: &uniq u8) -> result: own Result<unit, NarrowError> writes(slot) {
  let narrowed = propagate narrow(v: v);
  let stamped = stamp(slot: move slot);
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
// Proof-complete call closures
// ----------------------------------------------------------------------

/// The first recursive closure contains an unproved helper subscript and is
/// rejected before permission. The second source keeps the same recursive
/// sibling pair and makes only that helper total with a dominating branch.
#[test]
fn a_recursive_closure_requires_source_proof_and_then_is_eligible() {
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

fn scaled(values: own array<u8, 8>, index: own u64) -> result: own u8 pure {
  return values[index];
}

fn bubble(node: &uniq box<BoxNode>) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      let w = deref(leaf_w);
      let values = array_new::<u8, 8>(0_u8);
      let touched = scaled(values: move values, index: w);
      return w;
    }
    Branch(left: l, right: r, w: slot) => {
      let a = bubble(node: move l);
      let b = bubble(node: move r);
      let total = a +wrap b;
      set deref(slot) = total;
      return total;
    }
  }
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region {
    let total = bubble(node: &uniq branch0);
    if total == 7_u64 {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the unproved closure must reject before permission: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
    });

    let proved_source = br#"enum BoxNode {
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

fn scaled(values: own array<u8, 8>, index: own u64) -> result: own u8 pure {
  let size = len(values);
  if index < size {
    return values[index];
  }
  return 0_u8;
}

fn bubble(node: &uniq box<BoxNode>) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      let w = deref(leaf_w);
      let values = array_new::<u8, 8>(0_u8);
      let touched = scaled(values: move values, index: w);
      return w;
    }
    Branch(left: l, right: r, w: slot) => {
      let a = bubble(node: move l);
      let b = bubble(node: move r);
      let total = a +wrap b;
      set deref(slot) = total;
      return total;
    }
  }
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let leaf0 = boxed_leaf(w: 3_u64);
  let leaf1 = boxed_leaf(w: 4_u64);
  let branch0 = boxed_branch(left: move leaf0, right: move leaf1);
  region {
    let total = bubble(node: &uniq branch0);
    if total == 7_u64 {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(proved_source);
    let pair = only_pair(&table, "bubble");
    assert_eq!(
        pair.verdict,
        PermissionVerdict::PermittedEligible,
        "the proof-complete recursive closure must remain eligible"
    );
    let runs = &function_table(&table, "bubble").runs;
    assert_eq!(
        runs.len(),
        1,
        "an eligible pair forms its chain like any other: {runs:?}"
    );

    // The branch-proved helper really is in the closure of the judged pair:
    // `bubble` calls itself and its leaf arm calls `scaled`.
    assert_eq!(pair.first.callee_name, "bubble");
    assert_eq!(pair.second.callee_name, "bubble");
    assert!(
        std::str::from_utf8(proved_source)
            .expect("the fixture is UTF-8")
            .contains("if index < size"),
        "the fixture must keep the dominating source proof in its closure"
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

fn fold(node: &uniq box<BoxNode>, seed: own u64) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold(node: move l, seed: seed);
      let gap = seed +wrap 1_u64;
      let b = fold(node: move r, seed: seed);
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

/// A local invariant between two calls is a compile-time statement, not a
/// runtime window member. It neither splits the pair nor contributes a
/// footprint or exit edge.
#[test]
fn a_local_invariant_between_two_calls_keeps_the_pair() {
    let source = br#"fn peek(value: &u64) -> result: own u64 reads(value) {
  return deref(value);
}

command fn main() -> status: own ExitStatus pure {
  let left = 1_u64;
  let right = 2_u64;
  region {
    let a = peek(value: &left);
    invariant two_steps: 0_u64 <= 2_u64;
    let b = peek(value: &right);
    let total = a +wrap b;
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
    assert_eq!(function_table(&table, "main").runs.len(), 1);
}

/// Condition 2, clause 2c. The interposed `set` writes the storage s2's callee
/// reads through its actual. Under the schedule that hands s2 to a lane, that
/// read races the store and takes the pre-`set` value where source order
/// requires the post-`set` one.
#[test]
fn an_interposed_write_into_the_second_callees_read_is_denied_by_condition_two() {
    let source = br#"fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region {
    let a = peek(v: &other);
    set cell = 5_u64;
    let b = peek(v: &cell);
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
    let source = br#"fn bump(slot: &uniq u64) -> result: own u64 writes(slot) {
  set deref(slot) = 7_u64;
  return 1_u64;
}

fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region {
    let a = bump(slot: &uniq cell);
    set cell = 5_u64;
    let b = peek(v: &other);
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
    let source = br#"fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region {
    let a = peek(v: &other);
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
    let source = br#"fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  region {
    let a = take(v: cell);
    set cell = 15_u64;
    let b = peek(v: &other);
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

fn fold(node: &uniq box<BoxNode>, seed: own u64) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold(node: move l, seed: seed);
      let gap = a +wrap 1_u64;
      let b = fold(node: move r, seed: seed);
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
    assert_eq!(
        Some(*binding),
        pair.first.binding,
        "s1's own result is the link"
    );
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
    let source = br#"fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn take(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let other = 2_u64;
  region {
    let a = peek(v: &other);
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
    let source = br#"fn peek(slot: &u8) -> result: own u64 reads(slot) {
  return cvt::<u8, u64>(deref(slot));
}

fn stamp(slot: &uniq u8) -> result: own u64 writes(slot) {
  set deref(slot) = 9_u8;
  return 1_u64;
}

fn probe['o](outcome: own Result<u8, NarrowError>, a: &uniq 'o u8, b: &'o u8) -> result: own Result<unit, NarrowError> reads(b), writes(a) {
  let seen = peek(slot: b);
  let narrowed = propagate outcome;
  let stamped = stamp(slot: move a);
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

/// An unproved interposed subscript is rejected before the window judgment.
/// Condition 4 remains covered by the interposed propagate case above; the
/// accepted replacement below checks that a safe subscript creates no exit.
#[test]
fn an_unproved_interposed_subscript_is_rejected_before_permission() {
    let source = br#"fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn probe['r](values: own array<u8, 8>, index: own u64, cell: &'r u64, other: &'r u64) -> result: own u64 reads(cell, other) {
  let a = peek(v: other);
  let picked = values[index];
  let b = peek(v: cell);
  return imax(a, b);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  let table = array_new::<u8, 8>(0_u8);
  region {
    let total = probe(values: move table, index: 3_u64, cell: &cell, other: &other);
  }
  return exit_status(code: 0_u8);
}
"#;
    with_semantics(source, |outcome| {
        let SemanticOutcome::SourceIssue { issue, .. } = outcome else {
            panic!("the unproved interposed subscript must reject: {outcome:?}");
        };
        assert_eq!(issue.rule(), SemanticRule::Op4);
    });
}

#[test]
fn a_proved_interposed_subscript_creates_no_exit() {
    let source = br#"fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

fn probe['r](values: own array<u8, 8>, cell: &'r u64, other: &'r u64) -> result: own u64 reads(cell, other) {
  let a = peek(v: other);
  let picked = values[3_u64];
  let b = peek(v: cell);
  return imax(a, b);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  let table = array_new::<u8, 8>(0_u8);
  region {
    let total = probe(values: move table, cell: &cell, other: &other);
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "probe");
    assert_eq!(pair.verdict, PermissionVerdict::PermittedEligible);
    assert_eq!(
        function_table(&table, "probe").runs.len(),
        1,
        "the proof-complete interposed operation must retain the eligible run"
    );
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

fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  let other = 2_u64;
  let which = Low(w: 3_u64);
  region {
    let a = peek(v: &other);
    match which {
      Low(w: lw) => {
        let seen = lw;
      }
      High(w: hw) => {
        let seen = hw;
      }
    }
    let b = peek(v: &cell);
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
    let source = br#"fn peek_uniq(cell: &uniq u64) -> result: own u64 reads(cell) {
  return deref(cell);
}

command fn main() -> status: own ExitStatus pure {
  let cell = 21_u64;
  region {
    let a = peek_uniq(cell: &uniq cell);
    let b = peek_uniq(cell: &uniq cell);
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
    let source = br#"fn ignore_box(node: &box<u64>) -> result: own u64 pure {
  return 7_u64;
}

fn eat_box(node: own box<u64>) -> result: own u64 pure {
  return 9_u64;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let node = box_new(41_u64);
  region {
    let a = ignore_box(node: &node);
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
    let source = br#"fn bump(cell: &uniq u64) -> result: own u64 reads(cell), writes(cell) {
  let was = deref(cell);
  set deref(cell) = was +wrap 1_u64;
  return was;
}

fn takeval(v: own u64) -> result: own u64 pure {
  return v;
}

command fn main() -> status: own ExitStatus pure {
  let cell = 1_u64;
  region {
    let a = bump(cell: &uniq cell);
    let g = &cell;
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
    let source = br#"fn touch_uniqslice(v: &uniq slice<u8>) -> result: own u64 pure {
  return 3_u64;
}

fn a_pure_uniqslice() -> result: own u64 allocates(heap) {
  let buf = buffer_new(8_u64, 1_u8);
  region {
    let v = slice_of(&buf);
    region {
      let a = touch_uniqslice(v: &uniq v);
      let b = touch_uniqslice(v: &uniq v);
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
    let source = br#"fn quiet(cell: &uniq u64) -> result: own u64 pure {
  return 3_u64;
}

fn interposed_pure_syscall(x: own u64, name: own HostString) -> result: own u64 pure {
  let p = x;
  let r = x;
  region {
    let a = quiet(cell: &uniq p);
    let path = relative_path(value: move name);
    let b = quiet(cell: &uniq r);
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

// ----------------------------------------------------------------------
// Call position
// ----------------------------------------------------------------------

/// A call is a candidate wherever it is written in call position.
///
/// [PAR-1] judges calls, and a `match` scrutinee is a call. Reaching it only
/// through a `let` made one written spelling of the same two operations
/// invisible to the judgment: with one candidate there is no window, so no
/// pair was judged, no chain was formed, and the ledger reported nothing at
/// all about a program that plainly performs two independent operations.
#[test]
fn a_call_in_scrutinee_position_is_judged_as_the_bound_form_is() {
    let source = br#"command fn main(command.stdout as out: own OutputStream, command.stderr as err: own OutputStream) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  region 'out {
    region 'err {
      region {
        let first = write_once(output: &uniq 'out out, source: &bytes, start: 0_u64, end: 1_u64);
        match write_once(output: &uniq 'err err, source: &bytes, start: 1_u64, end: 2_u64) {
          Ok(value: written) => {
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    assert_eq!(
        pair.verdict,
        PermissionVerdict::PermittedEligible,
        "two independent outputs are eligible in either written position"
    );
    assert!(
        pair.second.binding.is_none(),
        "a scrutinee call defines no binding; the call occurrence is its identity"
    );
    let steps = &function_table(&table, "main").completion_steps;
    assert_eq!(steps.len(), 2, "the eligible pair forms one schedule");
    assert!(
        steps[0].has_later_independent_call,
        "the bound call runs while the scrutinee call is still outstanding"
    );
    assert!(!steps[1].has_later_independent_call);
    assert_eq!(steps[1].site.call, pair.second.call);
}

/// The same two calls with the scrutinee written first, which must deny.
///
/// The match's dispatch and the arm it selects read the scrutinee's result, so
/// the rest of that statement runs between the call and everything after it.
/// The window therefore contains the match statement itself, and a match
/// statement is a form this judgment does not project — the same refusal a
/// match written *between* two bound calls already gets.
#[test]
fn a_scrutinee_call_denies_against_a_later_call_it_is_read_before() {
    let source = br#"command fn main(command.stdout as out: own OutputStream, command.stderr as err: own OutputStream) -> status: own ExitStatus reads(out, err), writes(out, err), allocates(heap) {
  let bytes = buffer_new(2_u64, 65_u8);
  region 'out {
    region 'err {
      region {
        match write_once(output: &uniq 'out out, source: &bytes, start: 0_u64, end: 1_u64) {
          Ok(value: written) => {
          }
          Err(error: problem) => {
          }
        }
        let second = write_once(output: &uniq 'err err, source: &bytes, start: 1_u64, end: 2_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let pair = only_pair(&table, "main");
    let Denial::InterposedForm { side, form } = denial(pair, 2) else {
        panic!("expected an interposed-form denial, got {:?}", pair.verdict);
    };
    assert_eq!(
        *side,
        PairSide::Between(0),
        "s1's own statement stands there"
    );
    assert_eq!(*form, "a match statement");
    assert!(
        function_table(&table, "main").completion_steps.is_empty(),
        "a denied pair forms no schedule"
    );
}
