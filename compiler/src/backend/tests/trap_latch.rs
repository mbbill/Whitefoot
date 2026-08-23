//! The first-trap-wins latch: exactly one [DIAG-3] record under any schedule.
//!
//! A `claim` is the writer's lemma, so a *correct* program traps under no
//! schedule and nothing here can be reached by one. These cases are about the
//! other kind. A false executed claim is the sole writer-reachable language
//! runtime contract violation [SCOPE-4], so an execution that reaches one is
//! erroneous and the program is defective — and [PAR-1]'s observable-identity
//! guarantee is conditional on contract compliance exactly as [SCOPE-3]'s
//! freedom from undefined behavior is conditional on its trusted computing
//! base. What the guarantee narrows to for an erroneous execution is what this
//! file pins:
//!
//! - the process writes **exactly one** complete record and aborts, whatever
//!   the interleaving — never two, never a partial or interleaved one;
//! - the record names *a* claim whose predicate evaluated false, and *which*
//!   one may depend on the schedule;
//! - the sequential schedule is deterministic, so a defective program still
//!   has one reproduction path that names one claim every time.
//!
//! No source here states a false claim, because v0.34 admits none: every
//! fixture is an accepted program carrying true local residuals, and the
//! falsehood is injected into the checked IR after acceptance through the same
//! `force_claim_false_for_test` hook the single-threaded record-shape case
//! uses. The defect is a property of the run, which is what a defective
//! execution is.
//!
//! That first property is a mechanism rather than an argument. `wf_trap` takes
//! a process-wide latch before its first byte and a losing thread parks, and
//! [`the_latch_is_what_keeps_the_record_single`] is the control that proves it:
//! the same emitted module with the latch's decision defeated writes two
//! records instead of one. Without that control these cases would pass against
//! a program whose two threads never actually raced.

use std::process::Command;

use super::{build_executable, emit_with_overlap_and_false_claims, test_directory};

/// Two sibling calls the window judgment permits, each of which spins for long
/// enough that the two threads arrive at their own claims together.
///
/// Both claims are true, and this source is accepted: v0.34 admits no written
/// false claim, so the defect is injected into the checked IR by
/// [`super::emit_with_overlap_and_false_claims`] and both predicates evaluate
/// false at run time. An execution that reaches both therefore reaches two
/// traps. The spin is what makes the race real: without it the inline member
/// traps before the lane has picked its task up, and the case would pass
/// against a schedule that never overlapped anything. `left` is the handed-out
/// member and `right` runs inline, so source order names `left`.
const TWO_RACING_CLAIMS: &[u8] = br#"fn spin(seed: own u64) -> result: own u64 pure {
  let total = seed;
  for @acc i in 0_u64..400000_u64 {
    set total = total +wrap i;
  }
  return total;
}

fn left(seed: own u64) -> result: own u64 traps {
  let total = spin(seed: seed);
  let values = array_new<u64, 8>(1_u64);
  let bounded = imin(seed, 7_u64);
  let in_range = ilt(bounded, 8_u64);
  let injected_false = False();
  claim left_index_in_range: in_range because "premises: bounded is the minimum of the parameter seed and seven, and values has length eight\nderivation: a minimum is at most either operand, so bounded is at most seven and therefore below eight\nconclusion: ilt(bounded, 8_u64) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the following length-eight array subscript uses bounded";
  let picked = values[bounded];
  return picked +wrap total;
}

fn right(seed: own u64) -> result: own u64 traps {
  let total = spin(seed: seed);
  let values = array_new<u64, 8>(1_u64);
  let bounded = imin(seed, 7_u64);
  let in_range = ilt(bounded, 8_u64);
  let injected_false = False();
  claim right_index_in_range: in_range because "premises: bounded is the minimum of the parameter seed and seven, and values has length eight\nderivation: a minimum is at most either operand, so bounded is at most seven and therefore below eight\nconclusion: ilt(bounded, 8_u64) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the following length-eight array subscript uses bounded";
  let picked = values[bounded];
  return picked +wrap total;
}

command fn main() -> status: own ExitStatus traps {
  let a = left(seed: 1_u64);
  let b = right(seed: 2_u64);
  let total = imax(a, b);
  return exit_status(code: 0_u8);
}
"#;

/// The same shape with one claim forced false instead of two, so the trap
/// identity is unique and no schedule has anything to select.
const ONE_RACING_CLAIM: &[u8] = br#"fn spin(seed: own u64) -> result: own u64 pure {
  let total = seed;
  for @acc i in 0_u64..400000_u64 {
    set total = total +wrap i;
  }
  return total;
}

fn left(seed: own u64) -> result: own u64 traps {
  let total = spin(seed: seed);
  let values = array_new<u64, 8>(1_u64);
  let bounded = imin(seed, 7_u64);
  let in_range = ilt(bounded, 8_u64);
  let injected_false = False();
  claim left_index_in_range: in_range because "premises: bounded is the minimum of the parameter seed and seven, and values has length eight\nderivation: a minimum is at most either operand, so bounded is at most seven and therefore below eight\nconclusion: ilt(bounded, 8_u64) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the following length-eight array subscript uses bounded";
  let picked = values[bounded];
  return picked +wrap total;
}

fn right(seed: own u64) -> result: own u64 pure {
  return spin(seed: seed);
}

command fn main() -> status: own ExitStatus traps {
  let a = left(seed: 1_u64);
  let b = right(seed: 2_u64);
  let total = imax(a, b);
  return exit_status(code: 0_u8);
}
"#;

/// How many times a case that samples schedules runs the program. Each run is
/// a process spawn and about four milliseconds, and the property under test is
/// per-run rather than statistical, so this buys schedule variety and not
/// confidence in a rate.
const SAMPLES: usize = 40;

/// The record channel of one run, split into its lines, with the exit status.
fn trap_run(executable: &std::path::Path, workers: &str) -> (Option<i32>, Vec<u8>) {
    let output = Command::new(executable)
        .env("WF_WORKERS", workers)
        .output()
        .expect("run the trapping program");
    assert!(
        output.stdout.is_empty(),
        "a trapping program publishes nothing: {:?}",
        output.stdout
    );
    (output.status.code(), output.stderr)
}

/// One well-formed [DIAG-3] record, or the reason these bytes are not one.
///
/// The check is the record's own shape rather than a substring search: exactly
/// one line, terminated by exactly one LF, opening the four fields [DIAG-3]
/// fixes in their fixed order. Two records concatenated, a truncated record,
/// and two records interleaved mid-field all fail it.
fn sole_record(bytes: &[u8]) -> Result<String, String> {
    let text = String::from_utf8(bytes.to_vec()).map_err(|_| "not UTF-8".to_owned())?;
    let Some(body) = text.strip_suffix('\n') else {
        return Err(format!("no terminating LF: {text:?}"));
    };
    if body.contains('\n') {
        return Err(format!("more than one record: {text:?}"));
    }
    let opening = "{\"rule_id\":\"CLM-1\",\"message\":\"";
    let Some(rest) = body.strip_prefix(opening) else {
        return Err(format!("not a [DIAG-3] record: {text:?}"));
    };
    let Some((name, tail)) = rest.split_once('"') else {
        return Err(format!("unterminated claim name: {text:?}"));
    };
    if !tail.starts_with(",\"function\":\"")
        || !tail.ends_with("]}")
        || !tail.contains("\"node_path\":[")
    {
        return Err(format!("record fields are not [DIAG-3]'s: {text:?}"));
    }
    Ok(name.to_owned())
}

/// Two threads racing to trap produce exactly one record, every run.
///
/// The record names one of the two claims, and which one is the schedule's to
/// choose — that freedom is the whole of what a permitted overlap may change
/// about a trap, and it is why the judgment no longer refuses to overlap the
/// claim-bearing calls of correct programs.
#[test]
fn a_racing_pair_of_false_claims_writes_exactly_one_record() {
    let module = emit_with_overlap_and_false_claims(
        TWO_RACING_CLAIMS,
        &[
            ("left", "left_index_in_range"),
            ("right", "right_index_in_range"),
        ],
    );
    let directory = test_directory();
    let executable = build_executable(&module, &directory);

    for run in 0..SAMPLES {
        let (status, stderr) = trap_run(&executable, "4");
        let name = sole_record(&stderr)
            .unwrap_or_else(|reason| panic!("run {run} at WF_WORKERS=4: {reason}"));
        assert!(
            name == "left_index_in_range" || name == "right_index_in_range",
            "run {run} named a claim neither callee carries: {name}"
        );
        assert_eq!(
            status, None,
            "a trap aborts without a status [PROG-3], so the process dies by signal"
        );
    }

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The sequential schedule is the reproduction path: it names one claim, the
/// source-order first one, and its record bytes do not move.
///
/// `WF_WORKERS=0` and `WF_WORKERS=1` both take the module's own sequential
/// world, where nothing is handed out, so a writer handed a defective program
/// has one setting that turns a schedule-selected report back into a fixed one.
#[test]
fn the_sequential_schedule_names_one_claim_every_run() {
    let module = emit_with_overlap_and_false_claims(
        TWO_RACING_CLAIMS,
        &[
            ("left", "left_index_in_range"),
            ("right", "right_index_in_range"),
        ],
    );
    let directory = test_directory();
    let executable = build_executable(&module, &directory);

    let mut reference: Option<Vec<u8>> = None;
    for workers in ["0", "1"] {
        for run in 0..8 {
            let (_, stderr) = trap_run(&executable, workers);
            let name = sole_record(&stderr)
                .unwrap_or_else(|reason| panic!("run {run} at WF_WORKERS={workers}: {reason}"));
            assert_eq!(
                name, "left_index_in_range",
                "the sequential schedule traps at the source-order first claim"
            );
            match &reference {
                None => reference = Some(stderr),
                Some(first) => assert_eq!(
                    &stderr, first,
                    "WF_WORKERS={workers} run {run} moved a byte of the record"
                ),
            }
        }
    }

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// One false claim has one identity, so its record is byte-identical at every
/// worker count — the overlap selects nothing when there is nothing to select.
#[test]
fn a_single_false_claim_reports_the_same_bytes_at_every_worker_count() {
    let module =
        emit_with_overlap_and_false_claims(ONE_RACING_CLAIM, &[("left", "left_index_in_range")]);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);

    let (_, reference) = trap_run(&executable, "1");
    assert_eq!(
        sole_record(&reference).expect("the sequential run traps once"),
        "left_index_in_range"
    );
    for workers in ["0", "2", "4", "8"] {
        for run in 0..4 {
            let (_, stderr) = trap_run(&executable, workers);
            assert_eq!(
                stderr, reference,
                "WF_WORKERS={workers} run {run} moved a byte of the only record this program has"
            );
        }
    }

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The control: with the latch's decision defeated, the same module writes two
/// records. So the cases above are a claim about the latch and not about a
/// program whose threads never raced.
///
/// The injection is one token — the latch is still taken, and its answer is
/// still computed, but every thread is sent down the writing edge — so what it
/// removes is exactly the mechanism and nothing else. Detection was measured
/// at 200 of 200 runs on this machine, both threads reaching the writer
/// because `abort` on the winner is not instantaneous; eight runs here put a
/// false green far below anything else that could go wrong in this file.
///
/// A caught run must carry **two** well-formed records rather than merely not
/// one, so an empty record channel cannot pass for a defeated latch.
#[test]
fn the_latch_is_what_keeps_the_record_single() {
    let module = emit_with_overlap_and_false_claims(
        TWO_RACING_CLAIMS,
        &[
            ("left", "left_index_in_range"),
            ("right", "right_index_in_range"),
        ],
    );
    let directory = test_directory();

    let defeated = module.replace(
        "br i1 %won, label %write.loop, label %park",
        "br i1 true, label %write.loop, label %park",
    );
    assert_ne!(
        defeated, module,
        "the injection must find the latch's own branch"
    );
    let executable = build_executable(&defeated, &directory);

    let mut caught = 0;
    for run in 0..8 {
        let (_, stderr) = trap_run(&executable, "4");
        if sole_record(&stderr).is_ok() {
            continue;
        }
        // Not-one is not the property. An empty channel also fails
        // `sole_record`, so a future change that made the defeated build die
        // before writing anything would keep this control green while proving
        // nothing. What the doc-comment above claims, and what is asserted
        // here, is that the defeated module writes exactly two records.
        let text = String::from_utf8(stderr).expect("the record channel is UTF-8");
        let records: Vec<_> = text.lines().collect();
        assert_eq!(
            records.len(),
            2,
            "run {run}: defeating the latch must produce two records, not {}: {text:?}",
            records.len()
        );
        for record in records {
            let name = sole_record(format!("{record}\n").as_bytes()).unwrap_or_else(|reason| {
                panic!("run {run}: a defeated-latch record is still one [DIAG-3] line: {reason}")
            });
            assert!(
                name == "left_total_is_zero" || name == "right_total_is_zero",
                "run {run} named a claim neither callee carries: {name}"
            );
        }
        caught += 1;
    }
    assert!(
        caught > 0,
        "defeating the latch changed nothing observable in eight runs, so \
         these cases cannot see a second record"
    );

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}
