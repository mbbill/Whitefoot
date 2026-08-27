//! The staged loop permission judgment [PAR-3] over `for` and `loop` bodies
//! that perform I/O.
//!
//! Each grant fixture is a shape a real program writes; each denial fixture
//! violates exactly one numbered condition and asserts *that* condition, so a
//! denial arriving for the wrong reason fails the test. The denials are
//! deliberately the bulk of the file, for the reason the counted judgment's
//! tests give: granting is the easy half, and the whole risk of a rule that
//! lets an implementation keep several iterations in flight is a loop that
//! should have been refused and was not.
//!
//! The last section re-attacks the five holes the loan column closed, against
//! this judgment rather than against the counted one. It admits body shapes
//! [PAR-2] refuses outright — an uncounted loop, an early typed exit, a write
//! of enclosing storage that is not an accumulator — so its neighbourhood is
//! wider and every one of those holes has to be shown closed again here.

use crate::SemanticOutcome;

use super::super::permission::PermissionMetadata;
use super::super::staged_permission::{
    Disposition, PlaceDisposition, Segment, StagedDenial, StagedPermission, StagedVerdict,
};
use super::with_semantics;

fn permission_of(source: &[u8]) -> PermissionMetadata {
    with_semantics(source, |outcome| {
        let SemanticOutcome::Complete(program) = outcome else {
            panic!("staged permission fixture must check: {outcome:?}");
        };
        program.data.permission.clone()
    })
}

/// The only staged loop of one function. Every fixture below keeps its
/// interesting function to a single loop that performs I/O, so the assertion
/// cannot drift onto a neighbour.
fn only_staged<'table>(table: &'table PermissionMetadata, name: &str) -> &'table StagedPermission {
    let judged = staged(table, name);
    assert_eq!(
        judged.len(),
        1,
        "{name} must have exactly one staged loop: {judged:?}"
    );
    judged[0]
}

fn staged<'table>(table: &'table PermissionMetadata, name: &str) -> Vec<&'table StagedPermission> {
    table
        .named(name)
        .unwrap_or_else(|| panic!("no permission table for {name}"))
        .staged
        .iter()
        .collect()
}

/// The denial of one staged loop, asserted to cite the expected condition.
fn denied(source: &[u8], function: &str, condition: u8) -> StagedDenial {
    let table = permission_of(source);
    let judged = only_staged(&table, function);
    let StagedVerdict::Denied(denial) = &judged.verdict else {
        panic!("expected a denial, got {:?}", judged.verdict);
    };
    assert_eq!(
        denial.condition(),
        condition,
        "denied by the wrong condition: {denial:?}"
    );
    denial.clone()
}

fn permitted(source: &[u8], function: &str) -> StagedPermission {
    let table = permission_of(source);
    let judged = only_staged(&table, function).clone();
    assert_eq!(
        judged.verdict,
        StagedVerdict::Permitted,
        "expected a permitted staged loop"
    );
    judged
}

/// The dispositions of one staged loop, in the order the ledger prints them.
fn dispositions(judged: &StagedPermission) -> Vec<Disposition> {
    judged
        .dispositions
        .iter()
        .map(|place: &PlaceDisposition| place.disposition)
        .collect()
}

// ----------------------------------------------------------------------
// Grants
// ----------------------------------------------------------------------

/// The shape the whole rule exists for: one file per iteration, with the name
/// and the destination constructed inside the body.
///
/// Every disposition of the table appears here, which is what makes this one
/// fixture the readable statement of the judgment: the directory is read-only,
/// the factory is reached only before the cut, the total is reached only after
/// it, and the two per-iteration buffers are replicated.
#[test]
fn a_loop_with_iteration_own_scratch_is_permitted_and_carries_every_disposition() {
    let judged = permitted(ITERATION_OWN_SCRATCH, "main");
    assert_eq!(
        dispositions(&judged),
        vec![
            Disposition::Serialized(Segment::Prologue),
            Disposition::ReadOnly,
            Disposition::Serialized(Segment::Remainder),
            Disposition::Replicated,
            Disposition::Replicated,
        ]
    );
}

/// [PAR-2]'s unit is an index subrange, so it admits only a `for_stmt`. This
/// judgment's unit is the iteration the statement graph gives, so an uncounted
/// loop with a hand-carried index is admitted on the same terms — and the index
/// it advances before the submission is serialized by the prologue rather than
/// replicated or denied.
#[test]
fn an_uncounted_loop_is_admitted_on_the_same_terms_as_a_counted_one() {
    let judged = permitted(UNCOUNTED_LOOP, "main");
    assert_eq!(judged.form, "loop");
    assert_eq!(
        dispositions(&judged),
        vec![
            Disposition::Serialized(Segment::Prologue),
            Disposition::Serialized(Segment::Prologue),
            Disposition::ReadOnly,
            Disposition::Serialized(Segment::Remainder),
            Disposition::Replicated,
        ]
    );
}

/// An exit edge written before the submission is admitted, and this is the
/// sharpest place the two loop rules part company: [PAR-2] refuses every exit
/// edge of a counted loop outright, while this rule asks only where the edge
/// is. The same counted loop therefore carries a denial from one judgment and a
/// grant from the other.
#[test]
fn an_exit_edge_written_in_the_prologue_is_admitted() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let done = ige(index, 2_u64);
    if done {
      break @scan;
    }
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
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
    permitted(source, "main");
}

/// A loop that performs no I/O has no cut and therefore no staged schedule to
/// permit. It gets no verdict at all rather than a denial, so the ledger's
/// volume is a function of the loops that do I/O and every counted loop in the
/// corpus keeps exactly the lines it had.
#[test]
fn a_loop_with_no_may_suspend_action_gets_no_staged_verdict() {
    let source = b"command fn main() -> status: own ExitStatus pure {
  let total = 0_u64;
  for @sum i in 0_u64..8_u64 {
    set total = total +wrap i;
  }
  return exit_status(code: 0_u8);
}
";
    let table = permission_of(source);
    assert!(
        staged(&table, "main").is_empty(),
        "a loop with no submission is not staged"
    );
}

/// A nested loop that performs I/O is judged on its own terms, and the enclosing
/// loop that also performs I/O is judged separately. No rule joins two iteration
/// spaces into one.
#[test]
fn each_loop_that_performs_io_is_judged_on_its_own() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @outer outer_index in 0_u64..2_u64 {
    let outer_name = buffer_new(16_u64, 97_u8);
    region 'of {
      let outer_permit = reserve_file<'of>(factory: &uniq 'of files);
      region 'on {
        match open_file<'of, 'on>(permit: move outer_permit, root: &'of cwd, name: &'on outer_name, start: 0_u64, end: 4_u64) {
          Ok(value: outer_handle) => {
            set seen = seen +wrap 1_u64;
          }
          Err(error: outer_problem) => {
          }
        }
      }
    }
    for @inner inner_index in 0_u64..2_u64 {
      let inner_name = buffer_new(16_u64, 97_u8);
      region 'if {
        let inner_permit = reserve_file<'if>(factory: &uniq 'if files);
        region 'in {
          match open_file<'if, 'in>(permit: move inner_permit, root: &'if cwd, name: &'in inner_name, start: 0_u64, end: 4_u64) {
            Ok(value: inner_handle) => {
              set seen = seen +wrap 1_u64;
            }
            Err(error: inner_problem) => {
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let judged = staged(&table, "main");
    assert_eq!(judged.len(), 2, "two loops perform I/O: {judged:?}");
    // The outer loop's own submission is its cut, and the inner loop's
    // submission lies after it, so the inner loop is part of the outer
    // remainder. Every place the inner body reaches is therefore judged twice,
    // once per loop, and neither verdict reads the other.
    assert_eq!(judged[1].verdict, StagedVerdict::Permitted);
}

// ----------------------------------------------------------------------
// Condition 1: the cut
// ----------------------------------------------------------------------

/// A submission written inside one arm of a `match` is reached on some paths
/// and not others, so no program point of the body cuts it into a prologue and
/// a remainder. Getting this wrong in the permissive direction would put an
/// exit edge on the wrong side of the cut, which is why the query is a real
/// dominator and post-dominator pair and not a statement index.
#[test]
fn a_submission_reached_on_only_some_paths_has_no_cut() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let wanted = ilt(index, 2_u64);
    if wanted {
      region 'f {
        let permit = reserve_file<'f>(factory: &uniq 'f files);
        region 'n {
          match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
            Ok(value: handle) => {
              set seen = seen +wrap 1_u64;
            }
            Err(error: problem) => {
            }
          }
        }
      }
    }
    set seen = seen +wrap 1_u64;
  }
  return exit_status(code: 0_u8);
}
"#;
    let denial = denied(source, "main", 1);
    let StagedDenial::NoCut { .. } = denial else {
        panic!("expected a cut denial: {denial:?}");
    };
}

/// A submission written inside a loop of the body runs several times per
/// iteration, so the body has no single cut. The refusal is structural and
/// stated ahead of the dominator query, because the query's answer on a cyclic
/// region is not the single-entry single-exit shape the condition asks for.
#[test]
fn a_submission_written_inside_a_loop_of_the_body_has_no_cut() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let inner = 0_u64;
    loop @twice {
      let done = ige(inner, 2_u64);
      if done {
        break @twice;
      }
      set inner = inner +wrap 1_u64;
      let name = buffer_new(16_u64, 97_u8);
      region 'f {
        let permit = reserve_file<'f>(factory: &uniq 'f files);
        region 'n {
          match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
            Ok(value: handle) => {
              set seen = seen +wrap 1_u64;
            }
            Err(error: problem) => {
            }
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let table = permission_of(source);
    let judged = staged(&table, "main");
    let outer = judged
        .iter()
        .find(|entry| entry.form == "for")
        .expect("the counted loop is judged");
    let StagedVerdict::Denied(denial) = &outer.verdict else {
        panic!("expected a denial, got {:?}", outer.verdict);
    };
    assert_eq!(denial.condition(), 1, "{denial:?}");
}

// ----------------------------------------------------------------------
// Condition 2: exits
// ----------------------------------------------------------------------

/// The `wide8` shape: the body returns on its first failed open, and that edge
/// lies after the submission. With later iterations already submitted, the
/// decision to leave would be taken after opens the source-order execution
/// never performs.
#[test]
fn a_return_after_the_submission_denies() {
    let denial = denied(EXIT_IN_REMAINDER, "main", 2);
    let StagedDenial::ExitInRemainder { edge, .. } = denial else {
        panic!("expected an exit denial: {denial:?}");
    };
    assert_eq!(edge, "a return");
}

/// A `break` naming the staged loop is the same edge by another spelling, and
/// it is cited without a source node because a `break_stmt` carries none.
#[test]
fn a_break_after_the_submission_denies() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
          }
          Err(error: problem) => {
            break @scan;
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let denial = denied(source, "main", 2);
    let StagedDenial::ExitInRemainder { edge, statement } = denial else {
        panic!("expected an exit denial: {denial:?}");
    };
    assert_eq!(edge, "a break");
    assert!(
        statement.is_none(),
        "a break carries no node path to cite: {statement:?}"
    );
}

/// A `give` delivering to an initializer written *outside* the loop leaves it,
/// and a `give` delivering to one written inside it does not. The two spellings
/// are identical in the body, so telling them apart is the initializer stack's
/// whole job.
#[test]
fn a_give_leaving_the_loop_denies_and_one_delivered_inside_it_does_not() {
    let leaving = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let taken = ieq(1_u64, 1_u64);
  let outcome = if taken {
    for @scan index in 0_u64..4_u64 {
      let name = buffer_new(16_u64, 97_u8);
      region 'f {
        let permit = reserve_file<'f>(factory: &uniq 'f files);
        region 'n {
          match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
            Ok(value: handle) => {
              give 1_u64;
            }
            Err(error: problem) => {
            }
          }
        }
      }
    }
    give 0_u64;
  } else {
    give 2_u64;
  }
  return exit_status(code: 0_u8);
}
"#;
    let denial = denied(leaving, "main", 2);
    let StagedDenial::ExitInRemainder { edge, .. } = denial else {
        panic!("expected an exit denial: {denial:?}");
    };
    assert_eq!(edge, "a give");

    let inside = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let taken = ieq(index, 0_u64);
            let weight = if taken {
              give 1_u64;
            } else {
              give 0_u64;
            }
            set seen = seen +wrap weight;
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
    permitted(inside, "main");
}

// ----------------------------------------------------------------------
// Condition 3: retained borrows
// ----------------------------------------------------------------------

/// The hoisted destination: the body writes storage the iteration does not
/// introduce, and the `may-suspend` call that writes it retains its borrow past
/// its own submission. This version replicates only storage the body itself
/// constructs, so the denial names the borrow and points at the per-iteration
/// form.
#[test]
fn a_hoisted_destination_written_through_a_retained_borrow_denies() {
    let denial = denied(HOISTED_DESTINATION, "main", 3);
    let StagedDenial::RetainedBorrow {
        replicable_shape, ..
    } = denial
    else {
        panic!("expected a retained-borrow denial: {denial:?}");
    };
    assert!(
        replicable_shape,
        "a buffer of copy elements could be replicated once the coverage proof exists"
    );
}

/// An enumeration cursor has one position and no copy element type, so no
/// coverage proof and no later analysis can ever replicate it. The denial says
/// so, because telling that writer to allocate one per iteration would be wrong
/// advice.
#[test]
fn an_enclosing_enumeration_cursor_can_never_be_replicated() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let total = 0_u64;
  region 'c {
    let permit = reserve_file<'c>(factory: &uniq 'c files);
    match open_directory_source<'c>(permit: move permit, directory: &'c cwd) {
      Ok(value: list) => {
        for @scan index in 0_u64..4_u64 {
          let entries = buffer_new(1024_u64, 0_u8);
          region 'b {
            match directory_next<'b, 'b>(source: &uniq 'b list, destination: &uniq 'b entries, start: 0_u64, end: 1024_u64) {
              ListBytes(next: bytes, entries: reported) => {
                set total = total +wrap reported;
              }
              ListEnd() => {
              }
              ListFailed(error: problem) => {
              }
            }
          }
        }
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let denial = denied(source, "main", 3);
    let StagedDenial::RetainedBorrow {
        replicable_shape, ..
    } = denial
    else {
        panic!("expected a retained-borrow denial: {denial:?}");
    };
    assert!(
        !replicable_shape,
        "an opaque system nominal has no copy element type"
    );
}

/// A `may-suspend` call retaining a *shared* borrow on enclosing storage no
/// footprint of the body writes is admitted: the directory every open reads is
/// exactly that place, and refusing it would refuse the rule's own target
/// program.
#[test]
fn a_retained_shared_borrow_of_unwritten_enclosing_storage_is_read_only() {
    let judged = permitted(ITERATION_OWN_SCRATCH, "main");
    assert!(
        judged
            .dispositions
            .iter()
            .any(|place| place.disposition == Disposition::ReadOnly),
        "the directory is read-only: {:?}",
        judged.dispositions
    );
}

// ----------------------------------------------------------------------
// Condition 4: exclusive loans in the remainder
// ----------------------------------------------------------------------

/// The loan column's own case, moved to this judgment: a callee that declares
/// nothing about its `&uniq` parameter still holds an exclusive loan on it, and
/// two remainders coexist, so an exclusive loan on enclosing storage denies
/// whatever the row says.
#[test]
fn a_pure_exclusive_borrow_of_enclosing_storage_in_the_remainder_denies() {
    let source = br#"fn touch['c](cell: &uniq 'c u64) -> result: own u64 pure {
  return 1_u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let cell = 0_u64;
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'c {
              set seen = touch<'c>(cell: &uniq 'c cell);
            }
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
    let denial = denied(source, "main", 4);
    let StagedDenial::RemainderExclusiveLoan { .. } = denial else {
        panic!("expected a remainder-loan denial: {denial:?}");
    };
}

/// The same borrow taken in the prologue is admitted, because prologues run in
/// index order and never overlap one another. That is what admits
/// `reserve_file`'s short unique factory loan, and it is a restriction on the
/// schedule rather than an exemption from [OWN-5].
#[test]
fn the_same_exclusive_borrow_taken_in_the_prologue_is_serialized() {
    let source = br#"fn touch['c](cell: &uniq 'c u64) -> result: own u64 pure {
  return 1_u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let cell = 0_u64;
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let touched = 0_u64;
    region 'c {
      set touched = touch<'c>(cell: &uniq 'c cell);
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap touched;
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
    let judged = permitted(source, "main");
    assert!(
        judged
            .dispositions
            .iter()
            .any(|place| place.disposition == Disposition::Serialized(Segment::Prologue)),
        "the cell is serialized by the prologue: {:?}",
        judged.dispositions
    );
}

// ----------------------------------------------------------------------
// Condition 5: dispositions
// ----------------------------------------------------------------------

/// The hidden loop-carried byte: a callee writes the hoisted scratch only on
/// odd indices, so its written extent is not a fact of its signature. Executing
/// the prologue in index order would preserve that carried byte, but the
/// remainder reads the same storage, so no single segment serializes it. This
/// is the case a whole-place write-before-read rule would admit and silently
/// miscompile.
#[test]
fn storage_reached_on_both_sides_of_the_cut_has_no_disposition() {
    let denial = denied(BOTH_SIDES_OF_THE_CUT, "main", 5);
    let StagedDenial::NoDisposition { .. } = denial else {
        panic!("expected a disposition denial: {denial:?}");
    };
}

/// Storage the body reaches only after the cut is serialized by the remainder,
/// whose writes to storage rooted outside the loop commit in index order. That
/// is what admits an ordinary source-order accumulator write with no
/// associativity, no identity element, and no combination tree — a fold
/// [PAR-2]'s admitted operation set can never reach.
#[test]
fn an_accumulator_written_only_in_the_remainder_is_serialized_there() {
    let source = br#"fn weigh(left: own u64, right: own u64) -> result: own u64 pure {
  let scaled = left *wrap 31_u64;
  return scaled -wrap right;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let total = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = weigh(left: total, right: index);
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
    let judged = permitted(source, "main");
    assert!(
        judged
            .dispositions
            .iter()
            .any(|place| place.disposition == Disposition::Serialized(Segment::Remainder)),
        "the non-associative fold is serialized by the remainder: {:?}",
        judged.dispositions
    );
}

// ----------------------------------------------------------------------
// Condition 6: replicable elements
// ----------------------------------------------------------------------

/// `buffer_vacant` fills an interned `Option<T>` nominal whose [OWN-1] class
/// this judgment does not resolve, so a body constructing one fails closed.
/// Condition 6 is the one place a construction of the body itself can deny.
#[test]
fn a_construction_whose_element_class_is_unresolved_denies() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let slots = buffer_vacant<u64>(4_u64);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
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
    let denial = denied(source, "main", 6);
    let StagedDenial::NotReplicable { .. } = denial else {
        panic!("expected a replication denial: {denial:?}");
    };
}

// ----------------------------------------------------------------------
// Condition 7: fail closed
// ----------------------------------------------------------------------

/// A body statement that binds a bare borrow of enclosing storage refuses as a
/// form. The checked tree erases a written borrow's shared-or-uniq mode, so the
/// [OWN-5] loan it would hold cannot be stated, and admitting an unstated loan
/// would widen permission — which is the one direction this judgment must never
/// fail in.
#[test]
fn a_body_bound_borrow_of_enclosing_storage_refuses_as_a_form() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let shared = buffer_new(8_u64, 0_u8);
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    region 'b {
      let holder = &uniq 'b shared;
      let room = len(deref(holder));
      let fits = ilt(0_u64, room);
      if fits {
        set deref(holder)[0_u64] = 1_u8;
      }
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
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
    let denial = denied(source, "main", 7);
    let StagedDenial::BodyForm { .. } = denial else {
        panic!("expected a form denial: {denial:?}");
    };
}

/// The other direction of the same guard, which is the wrong denial the loan
/// column's own attack found and repaired: a bare borrow of storage the
/// iteration introduces needs no loan, because each iteration borrows its own
/// instance, so it must not refuse.
#[test]
fn a_body_bound_borrow_of_iteration_own_storage_is_admitted() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let scratch = buffer_new(8_u64, 0_u8);
    region 'b {
      let holder = &uniq 'b scratch;
      let room = len(deref(holder));
      let fits = ilt(0_u64, room);
      if fits {
        set deref(holder)[0_u64] = 1_u8;
      }
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
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
    permitted(source, "main");
}

// ----------------------------------------------------------------------
// The loan column's closed holes, re-attacked
// ----------------------------------------------------------------------

/// The owner's original example, moved to this judgment: two iterations each
/// taking `&uniq` of one enclosing cell through a callee whose row declares
/// only a read. The row projects a read; the loan is exclusive regardless, and
/// it is the loan that denies.
#[test]
fn two_uniq_borrows_of_one_cell_with_reads_only_rows_still_deny() {
    let source = br#"fn peek['c](cell: &uniq 'c u64) -> result: own u64 reads(cell) {
  return deref(cell);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let cell = 7_u64;
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'c {
              set seen = peek<'c>(cell: &uniq 'c cell);
            }
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
    let denial = denied(source, "main", 4);
    let StagedDenial::RemainderExclusiveLoan { .. } = denial else {
        panic!("expected a remainder-loan denial: {denial:?}");
    };
}

/// The interposed statement, which is where the loan column's fourth hole was:
/// a statement written between the submission and the rest of the remainder is
/// not exempt from any condition. Here it writes enclosing storage the prologue
/// also reads, and the disposition test sees both touches.
#[test]
fn a_statement_interposed_after_the_submission_is_judged_like_any_other() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let carried = 0_u64;
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let bound = ilt(carried, 4_u64);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set carried = carried +wrap 1_u64;
            set seen = seen +wrap 1_u64;
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
    let denial = denied(source, "main", 5);
    let StagedDenial::NoDisposition { .. } = denial else {
        panic!("expected a disposition denial: {denial:?}");
    };
}

/// Two overlapping *shared* loans deny nothing, exactly as they deny nothing in
/// a [PAR-1] window. Read-only sharing across iterations stays permitted, which
/// is the half of the loan column that must not over-refuse.
#[test]
fn two_shared_borrows_of_one_enclosing_buffer_deny_nothing() {
    let source = br#"fn total['s](source: &'s buffer<u8>) -> result: own u64 reads(source) {
  return len(deref(source));
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let table = buffer_new(8_u64, 3_u8);
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let counted = 0_u64;
    region 't {
      set counted = total<'t>(source: &'t table);
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'u {
              set seen = total<'u>(source: &'u table);
            }
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
    let judged = permitted(source, "main");
    assert!(
        judged
            .dispositions
            .iter()
            .any(|place| place.disposition == Disposition::ReadOnly),
        "the shared table is read-only on both sides of the cut: {:?}",
        judged.dispositions
    );
}

// ----------------------------------------------------------------------
// The fact-state invariant
// ----------------------------------------------------------------------

/// One staged table whatever the entailment fact state derives.
///
/// The judgment consults typing, rows, resolved places, and the statement
/// graph's edges, and never a derived fact — so facts-on and facts-off
/// compilation produce the same verdicts and the same disposition table by
/// construction. The compiler has no facts-off switch to run one program
/// through twice, so the differential is over *programs*: the three loops below
/// are one staged shape whose subscript obligation is discharged three
/// different ways — from a constant against a constant length, from a
/// dominating branch, and from a claimed fact — and their verdicts and tables
/// must be identical. A judgment that read the fact state could tell them
/// apart; this one may not.
#[test]
fn the_staged_verdict_is_the_same_under_every_route_to_the_same_fact() {
    let constant = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    set name[0_u64] = 98_u8;
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
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
    let branched = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let room = len(name);
    let fits = ilt(index, room);
    if fits {
      set name[index] = 98_u8;
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
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
    let claimed = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap), traps {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let room = len(name);
    let bounded = imin(index, room);
    let fits = ilt(bounded, room);
    claim bounded_fits: fits because "premises: bounded is the minimum of the loop binder and room, and room is the buffer's own length, which buffer_new fixed at sixteen\nderivation: a minimum is at most either operand, and room is positive, so bounded is strictly below room\nconclusion: ilt(bounded, room) is true\nchecker gap: ENT does not publish the result range of imin against its own second operand\nconsumers: the element write immediately below subscripts the buffer at bounded";
    set name[bounded] = 98_u8;
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
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
    let verdicts = [constant.as_slice(), branched.as_slice(), claimed.as_slice()].map(|source| {
        let table = permission_of(source);
        let judged = only_staged(&table, "main");
        (judged.verdict.clone(), dispositions(judged))
    });
    assert_eq!(
        verdicts[0], verdicts[1],
        "the constant and branched routes must agree"
    );
    assert_eq!(
        verdicts[0], verdicts[2],
        "the constant and claimed routes must agree"
    );
    assert_eq!(verdicts[0].0, StagedVerdict::Permitted);
}

// ----------------------------------------------------------------------
// Shared fixtures
// ----------------------------------------------------------------------

/// The granted shape, named once because four tests read it.
const ITERATION_OWN_SCRATCH: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let total = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    let data = buffer_new(64_u64, 0_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'h {
              region 'd {
                match read_at<'h, 'd>(file: &'h handle, destination: &uniq 'd data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                  ReadBytes(next: produced) => {
                    set total = total +wrap produced;
                  }
                  ReadEnd() => {
                  }
                  ReadFailed(error: problem) => {
                  }
                }
              }
            }
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

const UNCOUNTED_LOOP: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let opened = 0_u64;
  let index = 0_u64;
  loop @scan {
    let done = ige(index, 4_u64);
    if done {
      break @scan;
    }
    set index = index +wrap 1_u64;
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set opened = opened +wrap 1_u64;
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

const EXIT_IN_REMAINDER: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan index in 0_u64..4_u64 {
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set seen = seen +wrap 1_u64;
          }
          Err(error: problem) => {
            return exit_status(code: 4_u8);
          }
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;

const HOISTED_DESTINATION: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let name = buffer_new(16_u64, 97_u8);
  let data = buffer_new(64_u64, 0_u8);
  let total = 0_u64;
  for @scan index in 0_u64..4_u64 {
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            region 'h {
              region 'd {
                match read_at<'h, 'd>(file: &'h handle, destination: &uniq 'd data, file_offset: 0_u64, start: 0_u64, end: 64_u64) {
                  ReadBytes(next: produced) => {
                    set total = total +wrap produced;
                  }
                  ReadEnd() => {
                  }
                  ReadFailed(error: problem) => {
                  }
                }
              }
            }
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

const BOTH_SIDES_OF_THE_CUT: &[u8] = br#"fn stamp['b](slot: &uniq 'b buffer<u8>, index: own u64) -> result: own unit reads(slot), writes(slot) {
  let room = len(deref(slot));
  let sized = ilt(0_u64, room);
  if sized {
    let parity = index % 2_u64;
    let odd = ieq(parity, 1_u64);
    if odd {
      set deref(slot)[0_u64] = 7_u8;
    }
  }
  return unit;
}

fn first_byte['b](source: &'b buffer<u8>) -> result: own u64 reads(source) {
  let room = len(deref(source));
  let sized = ilt(0_u64, room);
  if sized {
  } else {
    return 0_u64;
  }
  let byte = deref(source)[0_u64];
  return cvt<u8, u64>(byte);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let scratch = buffer_new(1_u64, 0_u8);
  let name = buffer_new(16_u64, 97_u8);
  let total = 0_u64;
  for @scan index in 0_u64..4_u64 {
    region 's {
      let stamped = stamp<'s>(slot: &uniq 's scratch, index: index);
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let seen = 0_u64;
            region 'r {
              set seen = first_byte<'r>(source: &'r scratch);
            }
            set total = total +wrap seen;
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
