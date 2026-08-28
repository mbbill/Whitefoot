//! Actualization of a permitted counted loop [PAR-2 candidate]: what a split
//! loop emits, what each world runs, and what the program observes.
//!
//! The claim this module exists to check is the same one the pair path makes —
//! overlapping changes nothing observable — with one thing added that the pair
//! path never does: the *shape* of the combination tree is chosen by the
//! runtime rather than written in the source. So the byte comparisons here span
//! worker counts that produce genuinely different trees, and the differential
//! against the unsplit lowering is what says the tree does not matter.
//!
//! A run at `WF_WORKERS=1` proves nothing about overlapping — it takes the
//! sequential world — so the cases that need real overlap read the runtime's
//! own grant count and refuse a repeat that never actually handed anything out.

use std::process::Command;

use super::parallel::{
    build_executable_without_parallel_runtime, clone_symbols, function_body, grants_over_runs,
    identical, run_counting_grants,
};
use super::{
    build_executable, emit, emit_with_overlap, module_requires_parallel_runtime, test_directory,
};

/// How many runs a case sums the runtime's grant counter over before deciding
/// that a repeat overlapped nothing.
///
/// One run's count is a sample of the schedule: the offering thread may run
/// every one of its own offers before another lane reaches them, and on a
/// saturated machine it usually does. These fixtures fold a range worth only a
/// few dozen offers, so the sample is thin — measured on this machine at a
/// one-minute load average of 7.1, twelve runs each, [`PERMITTED_FOLD`] was
/// granted 3-8 lanes at `WF_WORKERS=4` and 3-14 at 8, against the pair path's
/// `par_layout`, which is granted 1 610-2 214 at two workers. A total over four
/// runs puts the floor an order of magnitude above zero and still fails
/// outright against a runtime that grants nothing, which is the whole of what
/// these assertions are for; it buys no confidence in a rate, which is not.
const GRANT_RUNS: usize = 4;

/// A counted `for` the judgment permits: one accumulator under `+wrap`, a
/// claim-free pure body doing real arithmetic per iteration, no write to
/// anything the iteration did not introduce, and no edge leaving the loop.
///
/// The body is an iterated integer mix rather than two operations, because the
/// split allowance refuses a range that is not worth splitting and a
/// two-operation body over this span would be refused — correctly, and then
/// this module would be measuring nothing. Its result is written to standard
/// output as eight bytes, so a difference anywhere in the fold is a difference
/// in the bytes.
const PERMITTED_FOLD: &[u8] = br#"fn mix(seed: own u64) -> result: own u64 pure {
  doc "A claim-free pure mix with enough arithmetic that splitting the range around it pays.";
  let state = seed;
  let round = 0_u64;
  loop @rounds {
    let done = ieq(round, 24_u64);
    if done {
      break @rounds;
    }
    let shifted = irotl(state, 27_u32);
    let scaled = state *wrap 6364136223846793005_u64;
    set state = ixor(shifted, scaled);
    set state = state +wrap 1442695040888963407_u64;
    set round = round +wrap 1_u64;
  }
  return state;
}

fn low_byte(v: own u64) -> result: own u8 pure {
  let low = iand(v, 255_u64);
  match cvt<u64, u8>(low) {
    Ok(value: byte) => {
      return byte;
    }
    Err(error: problem) => {
      return 0_u8;
    }
  }
}

fn spell['d](destination: &uniq 'd buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  let cursor = at;
  let rest = value;
  loop @octets {
    let limit = at +wrap 8_u64;
    let done = ige(cursor, limit);
    if done {
      break @octets;
    }
    let room = len(deref(destination));
    let writable = ilt(cursor, room);
    if writable {
      let byte = low_byte(v: rest);
      set deref(destination)[cursor] = byte;
    }
    set rest = irotr(rest, 8_u32);
    set cursor = cursor +wrap 1_u64;
  }
  return at +wrap 8_u64;
}

fn folded(lo: own u64, hi: own u64) -> result: own u64 pure {
  let total = 0_u64;
  for @points i in lo..hi {
    let mixed = mix(seed: i);
    set total = total +wrap mixed;
  }
  return total;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let value = folded(lo: 0_u64, hi: 400000_u64);
  let report = buffer_new(8_u64, 0_u8);
  region 'r {
    let filled = spell<'r>(destination: &uniq 'r report, at: 0_u64, value: value);
  }
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s report, start: 0_u64, end: 8_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
"#;

/// The permitted loop of [`PERMITTED_FOLD`] over ranges the split has to answer
/// for without folding anything: empty, inverted, and one wide.
///
/// The exit status carries the comparison, so a splitter that computed a width
/// before testing the endpoints — which wraps an inverted range to something
/// near 2^64 — fails here rather than hanging somewhere later.
const EDGE_RANGES: &[u8] = br#"fn mix(seed: own u64) -> result: own u64 pure {
  let state = seed;
  let round = 0_u64;
  loop @rounds {
    let done = ieq(round, 24_u64);
    if done {
      break @rounds;
    }
    let shifted = irotl(state, 27_u32);
    let scaled = state *wrap 6364136223846793005_u64;
    set state = ixor(shifted, scaled);
    set state = state +wrap 1442695040888963407_u64;
    set round = round +wrap 1_u64;
  }
  return state;
}

fn folded(lo: own u64, hi: own u64) -> result: own u64 pure {
  let total = 7_u64;
  for @points i in lo..hi {
    let mixed = mix(seed: i);
    set total = total +wrap mixed;
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  doc "Every degenerate range folds to the accumulator it arrived with, and one wide range folds to the same value split or not.";
  let empty = folded(lo: 5_u64, hi: 5_u64);
  if ieq(empty, 7_u64) {
  } else {
    return exit_status(code: 1_u8);
  }
  let inverted = folded(lo: 400000_u64, hi: 5_u64);
  if ieq(inverted, 7_u64) {
  } else {
    return exit_status(code: 2_u8);
  }
  let single = folded(lo: 5_u64, hi: 6_u64);
  let one = mix(seed: 5_u64);
  let expected = one +wrap 7_u64;
  if ieq(single, expected) {
  } else {
    return exit_status(code: 3_u8);
  }
  return exit_status(code: 0_u8);
}
"#;

/// The same permitted loop inside a function carrying enough live scalars that
/// the splitter's lane frame cannot hold them.
///
/// The frame is bounded by the runtime, and a split whose frame exceeds it
/// would have every claim refused forever: the program would pay for the
/// splitter and never overlap. So the bound is applied at compile time and the
/// loop declines with a line naming the width.
const WIDE_FRAME: &[u8] = br#"fn mix(seed: own u64) -> result: own u64 pure {
  let state = seed;
  let round = 0_u64;
  loop @rounds {
    let done = ieq(round, 24_u64);
    if done {
      break @rounds;
    }
    let shifted = irotl(state, 27_u32);
    let scaled = state *wrap 6364136223846793005_u64;
    set state = ixor(shifted, scaled);
    set state = state +wrap 1442695040888963407_u64;
    set round = round +wrap 1_u64;
  }
  return state;
}

command fn main() -> status: own ExitStatus pure {
  doc "Thirty-two live scalars stand between the loop and a frame that fits.";
  let a0 = 0_u64;
  let a1 = 1_u64;
  let a2 = 2_u64;
  let a3 = 3_u64;
  let a4 = 4_u64;
  let a5 = 5_u64;
  let a6 = 6_u64;
  let a7 = 7_u64;
  let a8 = 8_u64;
  let a9 = 9_u64;
  let b0 = 10_u64;
  let b1 = 11_u64;
  let b2 = 12_u64;
  let b3 = 13_u64;
  let b4 = 14_u64;
  let b5 = 15_u64;
  let b6 = 16_u64;
  let b7 = 17_u64;
  let b8 = 18_u64;
  let b9 = 19_u64;
  let c0 = 20_u64;
  let c1 = 21_u64;
  let c2 = 22_u64;
  let c3 = 23_u64;
  let c4 = 24_u64;
  let c5 = 25_u64;
  let c6 = 26_u64;
  let c7 = 27_u64;
  let c8 = 28_u64;
  let c9 = 29_u64;
  let d0 = 30_u64;
  let d1 = 31_u64;
  let total = 0_u64;
  for @points i in 0_u64..400000_u64 {
    let mixed = mix(seed: i);
    let biased = mixed +wrap a0;
    set total = total +wrap biased;
  }
  if ieq(total, 0_u64) {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;

/// A permitted loop that captures three enclosing values and folds under an
/// operation that is not `+wrap`.
///
/// Two things nothing else here reaches. **Captures**: the chunk and the
/// splitter declare them, the site passes them, and a lane frame carries them
/// through a thunk — four lists that have to agree on order and type, and a
/// fixture with no capture at all cannot tell whether they do. The three
/// differ in value *and* are folded asymmetrically, so a swapped pair moves the
/// published bytes. **A second combine**: `ixor` has a different identity from
/// `+wrap`, so seeding a chunk with the wrong one, or seeding the left half
/// where the right should be, changes the answer here and not there.
const CAPTURED_XOR_FOLD: &[u8] = br#"fn mix(seed: own u64, salt: own u64, rounds: own u64) -> result: own u64 pure {
  let state = ixor(seed, salt);
  let round = 0_u64;
  loop @rounds {
    let done = ieq(round, rounds);
    if done {
      break @rounds;
    }
    let shifted = irotl(state, 27_u32);
    let scaled = state *wrap 6364136223846793005_u64;
    set state = ixor(shifted, scaled);
    set state = state +wrap 1442695040888963407_u64;
    set round = round +wrap 1_u64;
  }
  return state;
}

fn low_byte(v: own u64) -> result: own u8 pure {
  let low = iand(v, 255_u64);
  match cvt<u64, u8>(low) {
    Ok(value: byte) => {
      return byte;
    }
    Err(error: problem) => {
      return 0_u8;
    }
  }
}

fn spell['d](destination: &uniq 'd buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  let cursor = at;
  let rest = value;
  loop @octets {
    let limit = at +wrap 8_u64;
    let done = ige(cursor, limit);
    if done {
      break @octets;
    }
    let room = len(deref(destination));
    let writable = ilt(cursor, room);
    if writable {
      let byte = low_byte(v: rest);
      set deref(destination)[cursor] = byte;
    }
    set rest = irotr(rest, 8_u32);
    set cursor = cursor +wrap 1_u64;
  }
  return at +wrap 8_u64;
}

fn folded(salt: own u64, rounds: own u64, stride: own u64) -> result: own u64 pure {
  doc "Three captured values, each used differently, folded under ixor.";
  let total = 12345678901234567890_u64;
  for @points i in 0_u64..400000_u64 {
    let stepped = i *wrap stride;
    let mixed = mix(seed: stepped, salt: salt, rounds: rounds);
    set total = ixor(total, mixed);
  }
  return total;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let value = folded(salt: 9876543210_u64, rounds: 24_u64, stride: 7_u64);
  let report = buffer_new(8_u64, 0_u8);
  region 'r {
    let filled = spell<'r>(destination: &uniq 'r report, at: 0_u64, value: value);
  }
  region 'o {
    region 's {
      match write_once<'o, 's>(output: &uniq 'o out, source: &'s report, start: 0_u64, end: 8_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
"#;

/// A permitted loop is emitted as a chunk, a splitter, and one ordinary
/// hand-out of the splitter's left half.
///
/// The whole design is in this shape: the leaf is the *loop*, never one
/// iteration, so the body keeps the vectorization and unrolling the loop gave
/// it; and the splitter's two halves are an ordinary overlap group, so the
/// claim, the frame, the thunk, and the join are the machinery a permitted pair
/// already uses rather than a second one.
#[test]
fn a_permitted_loop_is_outlined_split_and_joined() {
    let module = emit_with_overlap(PERMITTED_FOLD);

    assert!(
        module.contains("define internal i64 @wf__par_chunk_"),
        "a split loop must outline its own body as a chunk:\n{module}"
    );
    assert!(
        module.contains("define internal i64 @wf__par_split_"),
        "a split loop must synthesize a range splitter:\n{module}"
    );
    let splitter_symbol = synthesized(&module, "@wf__par_split_");
    let chunk_symbol = synthesized(&module, "@wf__par_chunk_");
    let splitter = function_body(&module, &splitter_symbol);
    for step in [
        "call ptr @wf__par_claim(",
        "call void @wf__par_publish(",
        "call void @wf__par_join(",
        "call void @wf__par_release(",
    ] {
        assert!(
            splitter.contains(step),
            "the splitter must take the ordinary lane protocol, missing {step}:\n{splitter}"
        );
    }
    assert!(
        splitter.contains(&format!("call i64 {chunk_symbol}(")),
        "the splitter's leaf must be the chunk, not one iteration:\n{splitter}"
    );

    // The leaf is a loop: the chunk carries a back edge and the counted range's
    // own guard, and names no runtime symbol at all.
    let chunk = function_body(&module, &chunk_symbol);
    assert!(
        chunk.contains("icmp ult i64") && chunk.matches("phi ").count() > 0,
        "the chunk must be the counted loop itself:\n{chunk}"
    );
    let (_, chunk_body) = chunk.split_once('\n').expect("a definition has a body");
    assert!(
        !chunk_body.contains("@wf__par_"),
        "the chunk is the loop and must reach no runtime entry point:\n{chunk}"
    );

    // Asked once per loop entry, in the enclosing function, and never inside
    // the chunk or per iteration.
    assert_eq!(
        module.matches("call i64 @wf__par_split_budget(").count(),
        1,
        "the split allowance must be asked exactly once, at loop entry:\n{module}"
    );
    assert!(
        function_body(&module, "@wf_folded").contains("call i64 @wf__par_split_budget("),
        "the allowance must be asked where the loop is entered:\n{module}"
    );
}

/// The default compilation splits nothing.
///
/// A permission is never an obligation. A build that did not ask for
/// actualization emits exactly the module it emitted before this path existed:
/// the loop inline, no synthesized function, and no runtime symbol anywhere.
#[test]
fn the_default_compilation_of_a_permitted_loop_splits_nothing() {
    let module = emit(PERMITTED_FOLD);
    assert!(
        !module.contains("wf__par_"),
        "the default build of a permitted loop must name no part of the split:\n{module}"
    );
    assert!(!module_requires_parallel_runtime(&module));
}

/// The sequential world of a split loop is the loop.
///
/// Two things carry that. The enclosing function's clone calls the chunk's
/// clone and reaches no runtime entry point — no claim, no allowance query, no
/// join — so nothing about a split is executed there. And the chunk's clone is
/// the chunk, byte for byte once its own symbols are restored, so the loop the
/// sequential world runs is not merely similar to the one the overlapped world
/// runs at its leaf: it is the same lowering.
///
/// The clone exists at all so that each copy has exactly one caller and the
/// loop is inlined back into it. Sharing one chunk between the worlds would
/// give it two callers and cost the sequential world that inlining, which is
/// the whole reason the second copy is cheap.
#[test]
fn the_sequential_world_of_a_split_loop_is_the_loop() {
    let module = emit_with_overlap(PERMITTED_FOLD);

    let chunk_symbol = synthesized(&module, "@wf__par_chunk_");
    let splitter_symbol = synthesized(&module, "@wf__par_split_");
    let chunk_clone_symbol = chunk_symbol.replace("@wf_", "@wf__par_seq_");

    let mut cloned = clone_symbols(&module);
    cloned.sort_unstable();
    assert_eq!(
        cloned,
        [
            chunk_clone_symbol.as_str(),
            "@wf__par_seq_folded",
            "@wf__par_seq_main"
        ],
        "the clone set must be the enclosing closure plus the chunk, and never the splitter:\n{module}"
    );

    let clone = function_body(&module, "@wf__par_seq_folded");
    assert!(
        clone.contains(&format!("call i64 {chunk_clone_symbol}(")),
        "the sequential world must call the chunk's own clone:\n{clone}"
    );
    for absent in [
        "@wf__par_claim",
        "@wf__par_publish",
        "@wf__par_join",
        "@wf__par_split_budget",
        splitter_symbol.as_str(),
    ] {
        assert!(
            !clone.contains(absent),
            "the sequential world must not reach {absent}:\n{clone}"
        );
    }

    // The clone renaming is the whole difference between the two copies. After
    // undoing it there must be nothing left, or the sequential world runs a
    // second lowering of the loop that nobody audited.
    let chunk = function_body(&module, &chunk_symbol);
    let chunk_clone = function_body(&module, &chunk_clone_symbol);
    assert_eq!(
        chunk_clone.replace("@wf__par_seq_", "@wf_"),
        chunk,
        "the two worlds' copies of the loop must be one lowering"
    );
}

/// The one synthesized symbol in this module beginning with `prefix`.
///
/// The synthesized halves are numbered after every source function, so their
/// ordinals move whenever a fixture gains one. Reading the symbol out of the
/// module keeps these cases about the shape rather than about the count. The
/// tail has to be a number, because the runtime's own `wf__par_split_budget`
/// shares a prefix with the splitter and is not one of these.
fn synthesized(module: &str, prefix: &str) -> String {
    let mut found: Vec<String> = module
        .lines()
        .filter_map(|line| line.split_once(prefix))
        .filter_map(|(head, tail)| head.starts_with("define ").then_some(tail))
        .filter_map(|tail| tail.split_once('('))
        .filter(|(ordinal, _)| ordinal.parse::<u32>().is_ok())
        .map(|(ordinal, _)| format!("{prefix}{ordinal}"))
        .collect();
    found.sort_unstable();
    found.dedup();
    let [only] = found.as_slice() else {
        panic!("the module must define exactly one {prefix}: {found:?}\n{module}");
    };
    only.clone()
}

/// The split publishes one byte sequence at every worker count, and the counts
/// that matter really did overlap.
///
/// This is the case the runtime-chosen grain has to answer for. The number of
/// chunks depends on the lane count, so these runs cut genuinely different
/// combination trees; every admitted combine is exactly associative, so every
/// tree has to publish the same bytes.
#[test]
fn a_split_loop_publishes_one_byte_sequence_at_every_worker_count() {
    let module = emit_with_overlap(PERMITTED_FOLD);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);

    let mut runs = Vec::new();
    for workers in ["0", "1", "2", "3", "4", "5", "8", "10", "16"] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the split program");
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
        assert_eq!(output.stdout.len(), 8, "WF_WORKERS={workers}");
        runs.push((format!("WF_WORKERS={workers}"), output.stdout));
    }
    let shipped = Command::new(&executable)
        .env_remove("WF_WORKERS")
        .output()
        .expect("run the split program at the shipped default");
    assert_eq!(shipped.status.code(), Some(0));
    runs.push(("WF_WORKERS unset".to_owned(), shipped.stdout));
    identical(&runs).expect("a split range must not move one byte of the fold");

    // A repeat over runs that never handed anything out would pass against a
    // runtime that granted nothing, so the counts that should overlap are
    // asked whether they did — over [`GRANT_RUNS`] runs, because one steal is
    // a race rather than a property of the lowering.
    for workers in ["4", "8"] {
        let lanes: usize = workers.parse().expect("the worker setting is a number");
        if !super::parallel::a_steal_is_observable(lanes) {
            continue;
        }
        let granted = grants_over_runs(&module, &directory, Some(workers), GRANT_RUNS);
        assert!(
            granted > 0,
            "WF_WORKERS={workers} granted no lane in {GRANT_RUNS} runs, so the repeat \
             above overlapped nothing"
        );
    }
    let (opted_out, _) = run_counting_grants(&module, &directory, Some("1"));
    assert_eq!(opted_out, 0, "WF_WORKERS=1 must take the sequential world");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A split that carries captures and folds under a second admitted operation
/// publishes what the unsplit lowering publishes, at every worker count.
///
/// The identity of `ixor` is not the identity of `+wrap`, and the three
/// captures have to reach the chunk in the order and the types its parameters
/// declare — through a lane frame and a thunk on the granted edge. Both are
/// checked against the default compilation of the same source rather than
/// against another run of the same module, so a defect the split introduces is
/// not present in the reference.
#[test]
fn a_split_loop_carries_its_captures_and_a_second_combine() {
    let unsplit = emit(CAPTURED_XOR_FOLD);
    let split = emit_with_overlap(CAPTURED_XOR_FOLD);
    assert!(
        split.contains("@wf__par_split_"),
        "the fixture's loop must actually split, or this checks nothing:\n{split}"
    );
    // Three captures, so the chunk takes the seed, both endpoints, and them.
    let chunk = function_body(&split, &synthesized(&split, "@wf__par_chunk_"));
    let signature = chunk.lines().next().expect("a definition has a signature");
    assert_eq!(
        signature.matches("i64 %").count(),
        6,
        "the chunk must declare the seed, both endpoints, and three captures: {signature}"
    );

    let directory = test_directory();
    let reference = Command::new(build_executable(&unsplit, &directory))
        .output()
        .expect("run the module that splits nothing");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(reference.stdout.len(), 8);

    let executable = build_executable(&split, &directory);
    let mut runs = vec![("no split lowering".to_owned(), reference.stdout)];
    for workers in ["1", "2", "4", "8"] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the split program");
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
        runs.push((format!("WF_WORKERS={workers}"), output.stdout));
    }
    identical(&runs).expect("a captured, xor-folded split must not move one byte");

    // Eight lanes need eight cores to show a steal, which is the same limit
    // the repeat above already carries; on a smaller host the zero is the
    // host's and this says so instead of reporting the lowering.
    if super::parallel::a_steal_is_observable(8) {
        let granted = grants_over_runs(&split, &directory, Some("8"), GRANT_RUNS);
        assert!(
            granted > 0,
            "the comparison above overlapped nothing in {GRANT_RUNS} runs"
        );
    }

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The differential: the same source lowered with no split at all publishes the
/// same bytes as the split lowering, at every worker count.
///
/// Every other case here links one emitted module several ways, so a defect
/// introduced by the split itself is present in the reference too and compares
/// equal. This reference is the default compilation, whose loop was never
/// outlined and never regrouped, which is the only way "regrouping the fold
/// changes nothing" can be checked against something other than itself.
#[test]
fn a_split_loop_agrees_with_the_lowering_that_splits_nothing() {
    let unsplit = emit(PERMITTED_FOLD);
    assert!(
        !module_requires_parallel_runtime(&unsplit),
        "the reference module must contain no split at all"
    );
    let split = emit_with_overlap(PERMITTED_FOLD);
    assert!(
        module_requires_parallel_runtime(&split),
        "the split module must hand work out, or the comparison is vacuous"
    );

    let directory = test_directory();
    let reference = Command::new(build_executable(&unsplit, &directory))
        .output()
        .expect("run the module that splits nothing");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(reference.stdout.len(), 8);

    let executable = build_executable(&split, &directory);
    let mut runs = vec![("no split lowering".to_owned(), reference.stdout)];
    for workers in ["1", "2", "4", "8"] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the split program");
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
        runs.push((format!("WF_WORKERS={workers}"), output.stdout));
    }
    identical(&runs).expect("splitting a range must not move one byte of the result");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// One admitted combine, at one accumulator type, as a fold a program actually
/// runs.
///
/// The fixtures above reach two of the ten combines at one width. The rest of
/// the set was covered only by `the_identity_of_every_admitted_combine_is_two_sided`
/// in `lowering::builder::split`, which re-implements the operations and so
/// cannot see a lowering that is wrong for one particular combine. These rows
/// close that: every combine, at `u64` and at `u8` where the combine has a
/// width, folded by a real program whose bytes are compared against the
/// lowering that splits nothing.
struct Combine {
    /// Names the fold's function and the row a failure reports.
    name: &'static str,
    /// The [OP-1] spelling the ledger prints, which is what says this row
    /// reached the combine it meant to.
    spelling: &'static str,
    /// The accumulator's written type.
    ty: &'static str,
    /// The accumulator's initial value.
    seed: &'static str,
    /// Statements binding `element` from `mixed`, in order. They are separate
    /// `let`s because a call is not an operand of another operation here.
    element: &'static [&'static str],
    /// The right-hand side of the body's `set total = ...`.
    fold: &'static str,
    /// How the accumulator becomes the `u64` the row publishes.
    publish: &'static str,
}

/// The `u64` accumulator publishes itself.
const PUBLISH_WIDE: &str = "  return total;\n";
/// The `u8` accumulator widens, which is a total pair and so binds directly.
const PUBLISH_NARROW: &str = "  let wide = cvt<u8, u64>(total);\n  return wide;\n";
/// `Bool` has no numeric conversion, so the branch is written out.
const PUBLISH_BOOL: &str =
    "  let wide = 0_u64;\n  if total {\n    set wide = 1_u64;\n  }\n  return wide;\n";

/// Every admitted combine, at every accumulator type it carries.
///
/// Each element expression is chosen so the fold's own value is not the value a
/// wrong identity element would produce: `iand` keeps a mask alive rather than
/// collapsing to zero, `imin` folds a range whose minimum is not zero, `imax`
/// one whose maximum is not the type's, and `band` fold to `True` where `bor`
/// folds to `False`. The `ixor` and `bxor` rows are the exception and cannot be
/// discriminating that way: the split always cuts a power-of-two number of
/// chunks, so a wrong xor identity is seeded an even number of times and
/// cancels. Their identity is what the two-sidedness table covers; what these
/// rows add for them is that the chunk boundaries and the join are right.
const ADMITTED_COMBINES: &[Combine] = &[
    Combine {
        name: "add_wide",
        spelling: "+wrap",
        ty: "u64",
        seed: "0_u64",
        element: &["let element = mixed;"],
        fold: "total +wrap element",
        publish: PUBLISH_WIDE,
    },
    Combine {
        name: "mul_wide",
        spelling: "*wrap",
        ty: "u64",
        seed: "1_u64",
        element: &["let element = ior(mixed, 1_u64);"],
        fold: "total *wrap element",
        publish: PUBLISH_WIDE,
    },
    Combine {
        name: "and_wide",
        spelling: "iand",
        ty: "u64",
        seed: "18446744073709551615_u64",
        element: &["let element = ior(mixed, 12297829382473034410_u64);"],
        fold: "iand(total, element)",
        publish: PUBLISH_WIDE,
    },
    Combine {
        name: "or_wide",
        spelling: "ior",
        ty: "u64",
        seed: "0_u64",
        element: &["let element = iand(mixed, 4886718345_u64);"],
        fold: "ior(total, element)",
        publish: PUBLISH_WIDE,
    },
    Combine {
        name: "xor_wide",
        spelling: "ixor",
        ty: "u64",
        seed: "0_u64",
        element: &["let element = mixed;"],
        fold: "ixor(total, element)",
        publish: PUBLISH_WIDE,
    },
    Combine {
        name: "min_wide",
        spelling: "imin",
        ty: "u64",
        seed: "18446744073709551615_u64",
        element: &["let element = mixed;"],
        fold: "imin(total, element)",
        publish: PUBLISH_WIDE,
    },
    Combine {
        name: "max_wide",
        spelling: "imax",
        ty: "u64",
        seed: "0_u64",
        element: &["let element = mixed;"],
        fold: "imax(total, element)",
        publish: PUBLISH_WIDE,
    },
    Combine {
        name: "add_narrow",
        spelling: "+wrap",
        ty: "u8",
        seed: "0_u8",
        element: &["let element = low_byte(v: mixed);"],
        fold: "total +wrap element",
        publish: PUBLISH_NARROW,
    },
    Combine {
        name: "mul_narrow",
        spelling: "*wrap",
        ty: "u8",
        seed: "1_u8",
        element: &[
            "let low = low_byte(v: mixed);",
            "let element = ior(low, 1_u8);",
        ],
        fold: "total *wrap element",
        publish: PUBLISH_NARROW,
    },
    Combine {
        name: "and_narrow",
        spelling: "iand",
        ty: "u8",
        seed: "255_u8",
        element: &[
            "let low = low_byte(v: mixed);",
            "let element = ior(low, 170_u8);",
        ],
        fold: "iand(total, element)",
        publish: PUBLISH_NARROW,
    },
    Combine {
        name: "or_narrow",
        spelling: "ior",
        ty: "u8",
        seed: "0_u8",
        element: &[
            "let low = low_byte(v: mixed);",
            "let element = iand(low, 5_u8);",
        ],
        fold: "ior(total, element)",
        publish: PUBLISH_NARROW,
    },
    Combine {
        name: "xor_narrow",
        spelling: "ixor",
        ty: "u8",
        seed: "0_u8",
        element: &["let element = low_byte(v: mixed);"],
        fold: "ixor(total, element)",
        publish: PUBLISH_NARROW,
    },
    Combine {
        name: "min_narrow",
        spelling: "imin",
        ty: "u8",
        seed: "255_u8",
        element: &[
            "let low = low_byte(v: mixed);",
            "let element = ior(low, 3_u8);",
        ],
        fold: "imin(total, element)",
        publish: PUBLISH_NARROW,
    },
    Combine {
        name: "max_narrow",
        spelling: "imax",
        ty: "u8",
        seed: "0_u8",
        element: &[
            "let low = low_byte(v: mixed);",
            "let element = iand(low, 63_u8);",
        ],
        fold: "imax(total, element)",
        publish: PUBLISH_NARROW,
    },
    Combine {
        name: "and_bool",
        spelling: "band",
        ty: "Bool",
        seed: "True()",
        element: &["let element = ige(mixed, 0_u64);"],
        fold: "band(total, element)",
        publish: PUBLISH_BOOL,
    },
    Combine {
        name: "or_bool",
        spelling: "bor",
        ty: "Bool",
        seed: "False()",
        element: &["let element = ilt(mixed, 0_u64);"],
        fold: "bor(total, element)",
        publish: PUBLISH_BOOL,
    },
    Combine {
        name: "xor_bool",
        spelling: "bxor",
        ty: "Bool",
        seed: "False()",
        element: &[
            "let bit = iand(mixed, 1_u64);",
            "let element = ieq(bit, 1_u64);",
        ],
        fold: "bxor(total, element)",
        publish: PUBLISH_BOOL,
    },
];

/// How wide a range every row of [`ADMITTED_COMBINES`] folds. Large enough that
/// the split allowance is worth taking — every row's loop is checked to have
/// actually split — and no larger, because seventeen of them run in one
/// program.
const COMBINE_SPAN: u64 = 200_000;

/// The helpers every row's fold shares: the per-iteration mix that gives the
/// body enough weight to be worth splitting, the narrowing to a byte, and the
/// eight-byte spelling each row publishes through.
const COMBINE_PRELUDE: &str = r#"fn mix(seed: own u64) -> result: own u64 pure {
  doc "A claim-free pure mix with enough arithmetic that splitting the range around it pays.";
  let state = seed;
  let round = 0_u64;
  loop @rounds {
    let done = ieq(round, 24_u64);
    if done {
      break @rounds;
    }
    let shifted = irotl(state, 27_u32);
    let scaled = state *wrap 6364136223846793005_u64;
    set state = ixor(shifted, scaled);
    set state = state +wrap 1442695040888963407_u64;
    set round = round +wrap 1_u64;
  }
  return state;
}

fn low_byte(v: own u64) -> result: own u8 pure {
  let low = iand(v, 255_u64);
  match cvt<u64, u8>(low) {
    Ok(value: byte) => {
      return byte;
    }
    Err(error: problem) => {
      return 0_u8;
    }
  }
}

fn spell['d](destination: &uniq 'd buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  let cursor = at;
  let rest = value;
  loop @octets {
    let limit = at +wrap 8_u64;
    let done = ige(cursor, limit);
    if done {
      break @octets;
    }
    let room = len(deref(destination));
    let writable = ilt(cursor, room);
    if writable {
      let byte = low_byte(v: rest);
      set deref(destination)[cursor] = byte;
    }
    set rest = irotr(rest, 8_u32);
    set cursor = cursor +wrap 1_u64;
  }
  return at +wrap 8_u64;
}
"#;

/// One program whose every row of [`ADMITTED_COMBINES`] is a permitted counted
/// loop, publishing each row's fold as eight bytes in table order.
fn admitted_combine_source() -> Vec<u8> {
    let mut source = String::from(COMBINE_PRELUDE);
    for combine in ADMITTED_COMBINES {
        let Combine {
            name,
            ty,
            seed,
            element,
            fold,
            publish,
            ..
        } = combine;
        source.push_str(&format!(
            "\nfn fold_{name}(lo: own u64, hi: own u64) -> result: own {ty} pure {{\n  \
             let total = {seed};\n  for @points i in lo..hi {{\n    \
             let mixed = mix(seed: i);\n"
        ));
        for statement in *element {
            source.push_str(&format!("    {statement}\n"));
        }
        // `after` is the previous row's write cursor, and `imin(after, 0)` is
        // zero for every unsigned value it can hold — so the fold's range is
        // the same for every row, and no two of `main`'s calls are independent.
        // That is what keeps every lane this program is granted a range split
        // rather than an overlapped window pair, which is what the grant
        // assertion below has to be about.
        source.push_str(&format!(
            "    set total = {fold};\n  }}\n  return total;\n}}\n\n\
             fn value_{name}(after: own u64) -> result: own u64 pure {{\n  \
             let lo = imin(after, 0_u64);\n  \
             let total = fold_{name}(lo: lo, hi: {COMBINE_SPAN}_u64);\n{publish}}}\n"
        ));
    }
    let width = 8 * ADMITTED_COMBINES.len();
    source.push_str(&format!(
        "\ncommand fn main(command.stdout as out: own Output) -> status: own ExitStatus \
         reads(out), writes(out), allocates(heap) {{\n  \
         let report = buffer_new({width}_u64, 0_u8);\n  region 'r {{\n"
    ));
    let mut at = "0_u64".to_owned();
    for (index, combine) in ADMITTED_COMBINES.iter().enumerate() {
        let name = combine.name;
        source.push_str(&format!(
            "    let v{index} = value_{name}(after: {at});\n    \
             let a{index} = spell<'r>(destination: &uniq 'r report, at: {at}, value: v{index});\n"
        ));
        at = format!("a{index}");
    }
    source.push_str(&format!(
        "  }}\n  region 'o {{\n    region 's {{\n      \
         match write_once<'o, 's>(output: &uniq 'o out, source: &'s report, start: 0_u64, \
         end: {width}_u64) {{\n        Ok(value: next) => {{\n          \
         return exit_status(code: 0_u8);\n        }}\n        Err(error: problem) => {{\n          \
         return exit_status(code: 1_u8);\n        }}\n      }}\n    }}\n  }}\n}}\n"
    ));
    source.into_bytes()
}

/// Every admitted combine folds a real split range and publishes what the
/// lowering that splits nothing publishes.
///
/// Until this case the only combines a running program ever folded through the
/// split were `+wrap` and `ixor`, both at `u64`. Everything else rested on
/// `the_identity_of_every_admitted_combine_is_two_sided`, which takes the
/// identity and the operation from production but evaluates them through its
/// own re-implementation — so it catches a wrong identity constant and cannot
/// catch a wrong lowering for one particular combine.
///
/// The rows are one program rather than seventeen because the expensive half is
/// the link, and one program keeps the whole table to two of them. Each row's
/// loop is confirmed to have split, by the ledger's own line naming that row's
/// combine, so a row that quietly stopped splitting fails here instead of
/// passing as a comparison of two sequential folds.
#[test]
fn every_admitted_combine_splits_and_publishes_the_unsplit_bytes() {
    let source = admitted_combine_source();
    let ledger = super::compile_permission_ledger(&source);
    for combine in ADMITTED_COMBINES {
        let expected = format!("fold_{}  loop at ", combine.name);
        let line = ledger
            .iter()
            .find(|line| line.starts_with("PAR split") && line.contains(&expected))
            .unwrap_or_else(|| {
                panic!(
                    "the {} row's loop must split, or the row checks nothing:\n{}",
                    combine.name,
                    ledger.join("\n")
                )
            });
        assert!(
            line.contains(&format!("split under {} over", combine.spelling)),
            "the {} row must split under {}: {line}",
            combine.name,
            combine.spelling
        );
    }

    let unsplit = emit(&source);
    assert!(
        !module_requires_parallel_runtime(&unsplit),
        "the reference module must contain no split at all"
    );
    let split = emit_with_overlap(&source);

    let directory = test_directory();
    let reference = Command::new(build_executable(&unsplit, &directory))
        .output()
        .expect("run the module that splits nothing");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(
        reference.stdout.len(),
        8 * ADMITTED_COMBINES.len(),
        "every row publishes eight bytes"
    );

    let executable = build_executable(&split, &directory);
    for workers in ["0", "1", "2", "3", "4", "5", "8", "16"] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the split program");
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
        assert_combine_rows(
            &reference.stdout,
            &output.stdout,
            &format!("WF_WORKERS={workers}"),
        );
    }
    let shipped = Command::new(&executable)
        .env_remove("WF_WORKERS")
        .output()
        .expect("run the split program at the shipped default");
    assert_eq!(shipped.status.code(), Some(0));
    assert_combine_rows(&reference.stdout, &shipped.stdout, "WF_WORKERS unset");

    // A splitter in the module is not a range the runtime actually cut: the
    // allowance is asked at each loop entry, and a range too small to be worth
    // splitting gets zero and descends straight to its leaf. Without this the
    // whole table would still pass against seventeen sequential folds — the
    // control is direct, since narrowing [`COMBINE_SPAN`] to a hundred takes
    // this program's grant count to zero in every run.
    if super::parallel::a_steal_is_observable(8) {
        let granted = grants_over_runs(&split, &directory, Some("8"), GRANT_RUNS);
        assert!(
            granted > 0,
            "no row's range was cut in {GRANT_RUNS} runs, so the comparisons above \
             are between two sequential folds"
        );
    }

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// Compares two runs of the combine program row by row, so a failure names the
/// combine that moved rather than an offset into a hundred and thirty-six
/// bytes.
fn assert_combine_rows(reference: &[u8], published: &[u8], setting: &str) {
    assert_eq!(
        published.len(),
        reference.len(),
        "{setting} published {} bytes, not {}",
        published.len(),
        reference.len()
    );
    for (index, combine) in ADMITTED_COMBINES.iter().enumerate() {
        let row = index * 8..index * 8 + 8;
        assert_eq!(
            published[row.clone()],
            reference[row],
            "{setting} moved the {} row, folded under {}",
            combine.name,
            combine.spelling
        );
    }
}

/// An empty range folds nothing, an inverted one folds nothing, and a one-wide
/// one folds exactly once.
///
/// The inverted case is the one with teeth. A splitter that computed `hi - lo`
/// before testing the endpoints wraps an inverted range to something near 2^64
/// and then descends into a range the loop never had; the loop it stands for
/// runs zero iterations. The program carries the comparison in its own claims,
/// so it fails here rather than somewhere downstream.
#[test]
fn a_degenerate_range_folds_to_the_accumulator_it_arrived_with() {
    let module = emit_with_overlap(EDGE_RANGES);
    assert!(
        module.contains("@wf__par_split_"),
        "the fixture's loop must actually split, or this checks nothing:\n{module}"
    );
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    for workers in ["0", "1", "2", "4", "8"] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the degenerate-range program");
        assert_eq!(
            output.status.code(),
            Some(0),
            "WF_WORKERS={workers} failed a range claim: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A loop whose lane frame would not fit declines at compile time, and says so.
///
/// The runtime refuses a claim whose frame is over the bound, so a split
/// emitted anyway would descend, be refused every lane, and run the whole range
/// on one thread having paid for the splitter — a sequentialization with no
/// report. Declining here converts that into a line naming the width.
#[test]
fn a_loop_whose_frame_is_too_wide_declines_and_says_so() {
    let ledger = super::compile_permission_ledger(WIDE_FRAME);
    let declined = ledger
        .iter()
        .find(|line| line.starts_with("PAR split"))
        .unwrap_or_else(|| panic!("the lowering must report what it did:\n{ledger:#?}"));
    assert!(
        declined.contains("declined:") && declined.contains("lane frame"),
        "the decline must name the frame it could not fit: {declined}"
    );
    assert!(
        ledger
            .iter()
            .any(|line| line.starts_with("PAR loop") && line.contains("permitted")),
        "the judgment must still permit the loop the lowering declined:\n{ledger:#?}"
    );

    let module = emit_with_overlap(WIDE_FRAME);
    assert!(
        !module.contains("@wf__par_split_"),
        "a declined loop must emit no splitter:\n{module}"
    );
}

/// The compile-time frame bound is the runtime's own.
///
/// Two numbers that have to agree and live in two languages: the lowering
/// decides whether to emit a split from one, and the runtime refuses a claim
/// from the other. If they drifted apart the decline above would fire on loops
/// that fit, or — the direction that matters — not fire on loops that do not.
#[test]
fn the_compile_time_frame_bound_is_the_runtimes() {
    let runtime = super::PARALLEL_RUNTIME_SOURCE;
    let declared = runtime
        .lines()
        .find_map(|line| line.strip_prefix("#define WF_PAR_FRAME_BYTES "))
        .expect("the runtime must state its frame bound");
    assert_eq!(
        declared.trim().parse::<u64>().expect("a decimal bound"),
        crate::LANE_FRAME_BYTES,
        "the lowering's frame bound has drifted from the runtime's"
    );
}

/// A module with a split loop and no runtime linked is a complete program.
///
/// The module carries its own weak answer to every entry point the split names,
/// including the allowance: with no runtime there are no lanes, so the honest
/// allowance is zero, the splitter descends straight to its leaf, and the whole
/// range runs as the loop it was.
#[test]
fn a_module_with_a_split_loop_and_no_runtime_still_runs() {
    let module = emit_with_overlap(PERMITTED_FOLD);
    assert!(
        module.contains("define weak i64 @wf__par_split_budget("),
        "a module that splits must carry its own answer to the allowance:\n{module}"
    );
    let directory = test_directory();
    let alone = build_executable_without_parallel_runtime(&module, &directory);
    let output = Command::new(&alone)
        .output()
        .expect("run the split module with no runtime linked");
    assert_eq!(output.status.code(), Some(0));

    let linked = Command::new(build_executable(&module, &directory))
        .env("WF_WORKERS", "8")
        .output()
        .expect("run the same module against the runtime");
    assert_eq!(linked.status.code(), Some(0));
    assert_eq!(
        output.stdout, linked.stdout,
        "the runtime must change the schedule and not the bytes"
    );

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A split costs the recursion depth of the splitter, and that depth is the
/// allowance rather than the range.
///
/// The bound is a property of the emitted shape and not a policy: the splitter
/// descends exactly `budget` levels, the allowance caps the chunk count at the
/// lane ceiling times the oversubscription, and the base-two logarithm of that
/// is ten. A range of 2^64 iterations therefore costs ten frames, not
/// sixty-four and not a per-iteration one. The program below runs a split loop
/// under a stack limit far below what a range-deep descent would need.
#[test]
fn a_split_loop_costs_a_bounded_stack() {
    let module = emit_with_overlap(PERMITTED_FOLD);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    let status = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!(
            "ulimit -s 512 && WF_WORKERS=8 {}",
            executable.display()
        ))
        .output()
        .expect("run the split program under a stack limit");
    assert_eq!(
        status.status.code(),
        Some(0),
        "a split must not need more stack than its allowance: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}
