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
//! The last section rechecks the five gaps the loan column closed, against
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
  for @scan (index in 0_u64..4_u64) {
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
  for @sum (i in 0_u64..8_u64) {
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
  for @outer (outer_index in 0_u64..2_u64) {
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
    for @inner (inner_index in 0_u64..2_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
    let StagedDenial::ExitInRemainder {
        edge,
        ref statement,
        selected_by_submission,
    } = denial
    else {
        panic!("expected an exit denial: {denial:?}");
    };
    // A `break_stmt` carries no node path, so the loop it names is the only
    // identity the denial can print, and the edge carries it.
    assert_eq!(edge, "a break naming this loop");
    assert!(
        statement.is_none(),
        "a break carries no node path to cite: {statement:?}"
    );
    // The break is a statement of the remainder in its own right, not the cut
    // statement's own edge, so the judgment does not attribute the submission's
    // outcome selects it.
    assert!(!selected_by_submission);
    // The remedy names the hoist and then says plainly where the hoist is not
    // available. The verification writer of 2026-08-28 met the hoist advice on
    // a read-to-EOF loop whose only break is selected by the read's own
    // `ReadEnd` outcome, and no rewrite of that loop can take it.
    assert_eq!(
        denial.writer_form(),
        "take every early return, break, or propagate in the prologue, before the body's first I/O submission. Where the exit is selected by the may-suspend call's own outcome — a read-to-EOF loop's `ReadEnd` break is — it cannot be taken before the submission and PAR-3 cannot stage that loop as written: the shapes staged today are a fixed-trip bounded loop and a per-file loop over names, and one file's chunk loop stays sequential"
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
    for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
        for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
/// whose accesses to storage rooted outside the loop are taken in index order.
/// That is what admits an ordinary source-order accumulator write with no
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
  for @scan (index in 0_u64..4_u64) {
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

/// A construction whose elements are affine costs its own reuse freedom and
/// nothing else.
///
/// `buffer_vacant` fills the interned `Option<T>` instance its type record
/// names, and that class resolves: a nominal element copies only when it is
/// tag-only [OWN-1], and the prelude's `Option<T>` carries a field in `Some` at
/// every T [PRE-1]. So the answer is "affine", not "unknown" — and [PAR-3]
/// conditions a loop's permission on the disposition of the places the body
/// reaches, never on a construction of the body. Escalating one construction's
/// affine element into a denial of the whole staged permission refused a
/// pipeline the rule grants; the body's own buffer is iteration-own storage
/// every iteration allocates for itself, exactly as the source-order execution
/// does.
///
/// The sibling direction — a copy element earning the reuse freedom — is the
/// `replicated` row of `buffer_new` in the granted table this test also reads.
/// The third, a class the judgment cannot resolve at all, still denies on
/// condition 6; no source program reaches it, because every construction this
/// judgment sees carries its element in its own type record.
#[test]
fn a_construction_whose_elements_are_affine_costs_the_loop_nothing() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan (index in 0_u64..4_u64) {
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
    let judged = permitted(source, "main");
    let citations: Vec<&PlaceDisposition> = judged
        .dispositions
        .iter()
        .filter(|place| place.disposition == Disposition::Replicated)
        .collect();
    assert_eq!(
        citations.len(),
        1,
        "the copy-element buffer earns the reuse freedom and the affine one carries no row: {:?}",
        judged.dispositions
    );
    assert!(
        judged
            .dispositions
            .iter()
            .all(|place| place.disposition != Disposition::Denied),
        "no place of a permitted loop is denied: {:?}",
        judged.dispositions
    );
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
  for @scan (index in 0_u64..4_u64) {
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
    assert!(
        denial
            .writer_form()
            .contains("write the borrow as an argument"),
        "the advice must name the form that carries a stateable loan: {}",
        denial.writer_form()
    );
}

/// The other direction of the same guard, which is the wrong denial the loan
/// column's own boundary review found and repaired: a bare borrow of storage the
/// iteration introduces needs no loan, because each iteration borrows its own
/// instance, so it must not refuse.
#[test]
fn a_body_bound_borrow_of_iteration_own_storage_is_admitted() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan (index in 0_u64..4_u64) {
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

/// Condition 7's other half, which the form refusal above does not reach: a
/// footprint *element* whose caller place the judgment does not resolve.
///
/// A slice reads through an origin this judgment holds no place for, so the
/// projection produces an unresolved element rather than a place with a
/// disposition. It must deny as [`StagedDenial::Unresolved`] rather than as
/// [`StagedDenial::BodyForm`]: the two carry different writer advice and the
/// same condition number, so a test that only checked the number would not
/// tell them apart. This is the fail-closed direction — the element is on
/// storage the body never writes, so a resolving judgment would grant it.
#[test]
fn an_unresolved_footprint_element_denies_as_unresolved_rather_than_as_a_form() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let table = buffer_new(16_u64, 97_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'v {
      let view = slice_of(&'v table);
      let seen = len(view);
      set total = total +wrap seen;
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
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
    let StagedDenial::Unresolved { .. } = denial else {
        panic!("expected an unresolved-element denial: {denial:?}");
    };
    assert!(
        denial.writer_form().contains("slice_of"),
        "the advice must name the binding that stands in front of the storage: {}",
        denial.writer_form()
    );
}

/// The admitted direction of the same variant: the identical length read taken
/// from the buffer itself resolves, so the loop is granted.
///
/// The two programs read the same length of the same enclosing buffer and
/// differ only in whether a slice stands between the read and the storage.
/// That is what makes the denial above a resolution limit of this judgment and
/// not a hazard of the program — and what makes it worth removing later.
#[test]
fn the_same_length_read_taken_without_a_slice_resolves_and_is_admitted() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let table = buffer_new(16_u64, 97_u8);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let seen = len(table);
    set total = total +wrap seen;
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
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

/// The third condition-7 refusal, and the second sanctioned over-denial: an
/// expression statement anywhere in the body.
///
/// Its reach projects onto no actual, so the judgment cannot form the call's
/// footprint and refuses the loop rather than reading it as empty. The denial
/// is unrelated to overlap — the helper here writes only iteration-own storage
/// — so the advice must name the form that carries the same call with a
/// footprint the judgment does read, which is a `let` binding of its result.
#[test]
fn an_expression_statement_refuses_as_a_form_and_names_the_let_binding() {
    let source = br#"fn stamp['b](slot: &uniq 'b buffer<u8>, index: own u64) -> result: own unit reads(slot), writes(slot) {
  let room = len(deref(slot));
  let wide = ilt(0_u64, room);
  if wide {
    set deref(slot)[0_u64] = 7_u8;
  }
  return unit;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    let scratch = buffer_new(8_u64, 0_u8);
    region 's {
      stamp<'s>(slot: &uniq 's scratch, index: index);
    }
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
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
    let StagedDenial::BodyForm { form, .. } = denial else {
        panic!("expected a form denial: {denial:?}");
    };
    assert_eq!(form, "an expression statement");
    assert!(
        denial
            .writer_form()
            .contains("bind the call's result with `let`"),
        "the advice must name the binding form: {}",
        denial.writer_form()
    );
}

/// Condition 7's third refused form, which the two above do not reach: an
/// expression statement whose discarded result is an own-mode affine value, so
/// the checked tree carries the compiler-derived release beside it.
///
/// The release is a [STOR-3] edge with its own footprint, and this judgment
/// classifies neither it nor the call's reach, so it refuses the form. The
/// advice has to differ from the plain expression statement's: binding the
/// value with `let` does not only give the call a footprint the judgment
/// reads, it moves the release to the binding's own scope exit.
#[test]
fn a_discarded_owned_result_refuses_as_its_own_form() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    buffer_new(8_u64, 0_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
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
    let StagedDenial::BodyForm { form, .. } = denial else {
        panic!("expected a form denial: {denial:?}");
    };
    assert_eq!(form, "a discarded expression statement");
    assert!(
        denial
            .writer_form()
            .contains("let the binding's own release"),
        "the advice must name what binding the value moves: {}",
        denial.writer_form()
    );
}

// ----------------------------------------------------------------------
// The loan column's closed gaps, rechecked
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
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
// The [OWN-7] overlap class
// ----------------------------------------------------------------------

/// A recurrence carried through a struct field denies, and the denial names
/// both halves of the overlapping pair.
///
/// This is the widening a boundary review found on 2026-08-27. The body
/// reads `work.seen` before the cut and replaces `work` after it. Keyed by the
/// exact resolved path those are two rows and each is safe alone — no
/// footprint writes *`work.seen`*, and nothing else touches *`work`* — while
/// the storage the two share carries a value from one iteration into the next:
/// sequentially `work.seen` ends at four, and with four prologues in flight it
/// ends at one. Keyed by the [OWN-7] class they are one place the body reaches
/// on both sides of the cut, which is condition 5's denial.
///
/// The denial has to name the pair. One statement alone does not show a reader
/// why a loop mentioning two different paths was refused.
#[test]
fn a_recurrence_carried_through_a_struct_field_denies_and_names_the_pair() {
    let table = permission_of(FIELD_RECURRENCE);
    let judged = only_staged(&table, "main");
    let StagedVerdict::Denied(StagedDenial::NoDisposition {
        argument,
        overlapping,
    }) = &judged.verdict
    else {
        panic!("expected a condition 5 denial: {:?}", judged.verdict);
    };
    let overlapping = overlapping
        .as_ref()
        .expect("a denial the overlap decided names the other half of the pair");
    assert_ne!(
        argument, overlapping,
        "the pair is two statements, not one cited twice"
    );
    // The first row is the field read the exact-path judgment called
    // read-only, and it now carries its class's disposition.
    assert_eq!(
        dispositions(judged)[0],
        Disposition::Denied,
        "the field read carries the class's disposition: {:?}",
        judged.dispositions
    );
}

/// The granted half of the same pair: two *disjoint* field paths of one record
/// stay two places.
///
/// [OWN-7] is a prefix test, not a root test, so widening the judgment to the
/// class must not collapse a record into one indivisible place. The prologue
/// reads `pair.a` and the remainder writes `pair.b`; neither path is a prefix
/// of the other, so the read is read-only and the write is serialized in the
/// remainder, and the loop is granted.
#[test]
fn two_disjoint_fields_of_one_record_are_judged_independently() {
    let source = br#"struct Pair {
  a: u64;
  b: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let pair = Pair(a: 1_u64, b: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let carried = pair.a;
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set pair.b = pair.b +wrap carried;
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
    assert_eq!(
        dispositions(&judged),
        vec![
            Disposition::ReadOnly,
            Disposition::Serialized(Segment::Prologue),
            Disposition::ReadOnly,
            Disposition::Serialized(Segment::Remainder),
            Disposition::Replicated,
        ]
    );
}

/// The mirror of the recurrence denies too, which is why the repair is stated
/// over the class rather than over one disposition.
///
/// Here the prologue replaces the whole record and the remainder reads one of
/// its fields, so the exact-path judgment called the write `serialized-P` and
/// the read `read-only` — two different safe answers, both wrong for the same
/// reason. A repair that only taught `read-only` about writes would have left
/// this open.
#[test]
fn the_mirror_of_the_field_recurrence_denies_as_well() {
    let source = br#"struct Carrier {
  tag: u64;
  spare: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let carrier = Carrier(tag: 0_u64, spare: 0_u64);
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let previous = replace carrier = Carrier(tag: index, spare: 0_u64);
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let seen = carrier.tag;
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
    let denial = denied(source, "main", 5);
    let StagedDenial::NoDisposition { overlapping, .. } = denial else {
        panic!("expected a condition 5 denial: {denial:?}");
    };
    assert!(
        overlapping.is_some(),
        "the prologue write and the remainder read are the pair"
    );
}

/// The severe form: the submission borrows a field of storage the remainder
/// replaces.
///
/// `open_file(name: &'n held.name, …)` retains that borrow to its `terminal`
/// milestone, and the remainder runs `replace held = Holder(…)`, dropping the
/// buffer the borrow names while a later iteration's open is still outstanding
/// on it. That is the precise hazard condition 3 exists to prevent, and the
/// exact-path judgment called the borrowed place read-only because no
/// footprint writes *`held.name`*.
///
/// The replace is written after the match rather than inside one arm, so both
/// arms agree on what they own at the join and the hazard is unconditional:
/// every iteration's remainder installs a new buffer under the borrow the next
/// iteration's prologue is about to take.
#[test]
fn a_borrow_into_storage_the_remainder_replaces_denies_by_the_retained_borrow() {
    let source = br#"struct Holder {
  name: buffer<u8>;
  seen: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let first = buffer_new(16_u64, 97_u8);
  let held = Holder(name: move first, seen: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let fresh = buffer_new(16_u64, 98_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n held.name, start: 0_u64, end: 0_u64) {
          Ok(value: handle) => {
          }
          Err(error: problem) => {
          }
        }
        let previous = replace held = Holder(name: move fresh, seen: 1_u64);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let denial = denied(source, "main", 3);
    let StagedDenial::RetainedBorrow {
        written_at,
        overlapping,
        ..
    } = denial
    else {
        panic!("expected a condition 3 denial: {denial:?}");
    };
    assert!(
        written_at.is_some(),
        "the borrowed field and the write that denies are the pair, and the \
         write is the replaced record rather than the field itself"
    );
    assert!(
        overlapping,
        "the write is on the record and the borrow on its field, which is the \
         [OWN-7] pair this denial reports as one storage"
    );
}

/// The pair a condition-3 denial names is the borrow and the *write*, not the
/// borrow twice.
///
/// Here the retained borrow is on `held.name`, the write is on `held.seen`
/// through a callee's row, and the loan the remainder takes is on the whole
/// record. `held.name` and `held.seen` are disjoint under [OWN-7], so the
/// class that carries both flags is the whole record's, and the write that
/// supplies `written` is not the statement the borrow came from. Naming the
/// first place that widened the class would print the borrow as its own
/// counterpart and tell the reader nothing.
///
/// The loop denies whatever condition it is attributed to: the remainder holds
/// an exclusive loan on enclosing storage, and the record is a place a
/// footprint writes and a retained loan touches, so condition 5 has no
/// disposition for it either.
#[test]
fn a_condition_three_denial_names_the_write_and_not_the_borrow_twice() {
    let source = br#"struct Holder {
  name: buffer<u8>;
  seen: u64;
}

fn bump['b](holder: &uniq 'b Holder) -> result: own unit reads(holder.seen), writes(holder.seen) {
  set deref(holder).seen = deref(holder).seen +wrap 1_u64;
  return unit;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seed = buffer_new(16_u64, 97_u8);
  let held = Holder(name: move seed, seen: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n held.name, start: 0_u64, end: 0_u64) {
          Ok(value: handle) => {
            region 'b {
              let done = bump<'b>(holder: &uniq 'b held);
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
    let denial = denied(source, "main", 3);
    let StagedDenial::RetainedBorrow {
        argument,
        written_at,
        overlapping,
        ..
    } = denial
    else {
        panic!("expected a condition 3 denial: {denial:?}");
    };
    let written_at =
        written_at.expect("the write that denies is on another path and must be named");
    assert_ne!(
        written_at, argument,
        "naming the borrow as its own counterpart says nothing"
    );
    assert!(
        overlapping,
        "the write is on a different path of the same [OWN-7] class, which is \
         what makes the two halves one storage"
    );
}

// ----------------------------------------------------------------------
// Condition 2: the cut statement's own leaving edge
// ----------------------------------------------------------------------

/// A `propagate` whose right-hand side *is* the cut leaves from the remainder.
///
/// The second widening the review of 2026-08-27 found. The statement performs
/// the submission, so its footprint is the prologue's; but the `Err` edge is
/// selected by that submission's own outcome, which only the remainder joins.
/// With K iterations in flight the decision to leave is therefore taken after
/// P(i+1..i+K) already submitted opens the source-order execution never
/// performs. Admitting it would also decide a language capability by source
/// shape: the same exit written as a `match` arm was denied all along.
#[test]
fn a_propagate_whose_right_hand_side_is_the_cut_leaves_from_the_remainder() {
    let source = br#"fn scan_all['c](cwd: &'c DirectoryRead, files: own FileFactory) -> result: own Result<u64, IoError> reads(cwd, files), writes(files), allocates(heap) {
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let name = buffer_new(16_u64, 97_u8);
    region 'p {
      let permit = reserve_file<'p>(factory: &uniq 'p files);
      region 'n {
        let handle = propagate open_file<'c, 'n>(permit: move permit, root: cwd, name: &'n name, start: 0_u64, end: 4_u64);
        set total = total +wrap 1_u64;
      }
    }
  }
  return Ok<u64, IoError>(value: total);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  region 'c {
    match scan_all<'c>(cwd: &'c cwd, files: move files) {
      Ok(value: counted) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let denial = denied(source, "scan_all", 2);
    let StagedDenial::ExitInRemainder {
        edge,
        ref statement,
        selected_by_submission,
    } = denial
    else {
        panic!("expected an exit denial: {denial:?}");
    };
    assert_eq!(edge, "a propagate");
    assert!(
        statement.is_some(),
        "a propagate carries the node path of its own statement"
    );
    // The propagated call *is* the cut, so its Err edge is selected by the
    // submission's own outcome: no rewrite takes it before the submission, and
    // the remedy has to say so rather than repeat the hoist advice.
    assert!(selected_by_submission);
    assert_eq!(
        denial.writer_form(),
        "PAR-3 cannot stage this loop as written: the submission's own outcome selects this edge, so no rewrite takes it before the submission. The shapes staged today are a fixed-trip bounded loop and a per-file loop over names; one file's chunk loop stays sequential"
    );
}

/// The granted half: a `propagate` written *before* the cut leaves from the
/// prologue and is admitted.
///
/// The two programs differ in one thing — whether the propagated call is the
/// body's first `may-suspend` action — and that is exactly the fact the
/// condition turns on. At this edge no prologue of a later iteration has begun,
/// because prologues run in index order and P(i) has not completed, so no
/// operation the source-order execution never performs has been submitted.
#[test]
fn a_propagate_written_before_the_cut_leaves_from_the_prologue_and_is_admitted() {
    let source = br#"fn classify(index: own u64) -> result: own Result<u64, IoError> pure {
  return Ok<u64, IoError>(value: index);
}

fn scan_all['c](cwd: &'c DirectoryRead, files: own FileFactory) -> result: own Result<u64, IoError> reads(cwd, files), writes(files), allocates(heap) {
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
    let kept = propagate classify(index: index);
    let name = buffer_new(16_u64, 97_u8);
    region 'p {
      let permit = reserve_file<'p>(factory: &uniq 'p files);
      region 'n {
        match open_file<'c, 'n>(permit: move permit, root: cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            set total = total +wrap 1_u64;
          }
          Err(error: problem) => {
          }
        }
      }
    }
  }
  return Ok<u64, IoError>(value: total);
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  region 'c {
    match scan_all<'c>(cwd: &'c cwd, files: move files) {
      Ok(value: counted) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    permitted(source, "scan_all");
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
/// are one staged shape whose subscript obligation is discharged by a constant,
/// a dominating branch, and a counted binder carrying an inductive source
/// invariant. Their verdicts and tables must be identical. The former
/// minimum-plus-runtime-assertion route was retired with executable assertions
/// operations; the invariant route is the proof-carrying replacement.
/// A judgment that read the proof state could tell the programs apart; this one
/// may not.
#[test]
fn the_staged_verdict_is_the_same_under_every_route_to_the_same_fact() {
    let constant = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
    let invariant_proved = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan (
    index in 0_u64..4_u64,
    invariant index_bound: ile(index, 4_u64)
  ) {
    let name = buffer_new(16_u64, 97_u8);
    set name[index] = 98_u8;
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
    let verdicts = [
        constant.as_slice(),
        branched.as_slice(),
        invariant_proved.as_slice(),
    ]
    .map(|source| {
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
        "the constant and invariant-proved routes must agree"
    );
    assert_eq!(verdicts[0].0, StagedVerdict::Permitted);
}

/// A proof fact cannot erase a real cross-segment storage dependency. Here the
/// accumulator invariant proves the prologue subscript, but that same `seen`
/// storage is written in the remainder. PAR-3 must therefore keep the ordinary
/// place row and deny the loop instead of consulting how the subscript checked.
#[test]
fn an_invariant_proved_accumulator_index_keeps_its_cross_segment_dependency() {
    let source = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let seen = 0_u64;
  for @scan (
    index in 0_u64..4_u64,
    invariant seen_bound: ile(seen, index)
  ) {
    let name = buffer_new(16_u64, 97_u8);
    set name[seen] = 98_u8;
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
          }
          Err(error: problem) => {
          }
        }
      }
    }
    set seen = index;
  }
  return exit_status(code: 0_u8);
}
"#;
    let denial = denied(source, "main", 5);
    assert!(matches!(denial, StagedDenial::NoDisposition { .. }));
}

// ----------------------------------------------------------------------
// Shared fixtures
// ----------------------------------------------------------------------

/// The field recurrence: `work.seen` read before the cut and `work` replaced
/// after it, which is one storage under [OWN-7] and two rows without it.
const FIELD_RECURRENCE: &[u8] = br#"struct Work {
  seen: u64;
  code: u64;
}

command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let work = Work(seen: 0_u64, code: 0_u64);
  for @scan (index in 0_u64..4_u64) {
    let carried = work.seen;
    let name = buffer_new(16_u64, 97_u8);
    region 'f {
      let permit = reserve_file<'f>(factory: &uniq 'f files);
      region 'n {
        match open_file<'f, 'n>(permit: move permit, root: &'f cwd, name: &'n name, start: 0_u64, end: 4_u64) {
          Ok(value: handle) => {
            let bumped = carried +wrap 1_u64;
            let previous = replace work = Work(seen: bumped, code: 0_u64);
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

/// The granted shape, named once because four tests read it.
const ITERATION_OWN_SCRATCH: &[u8] = br#"command fn main(command.cwd as cwd: own DirectoryRead, command.files as files: own FileFactory) -> status: own ExitStatus reads(cwd, files), writes(cwd, files), allocates(heap) {
  let total = 0_u64;
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
  for @scan (index in 0_u64..4_u64) {
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
