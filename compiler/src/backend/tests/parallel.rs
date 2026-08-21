//! Actualization of the permission judgment: what a permitted overlap group
//! emits, what an unpermitted pair still emits, what links, and what the
//! program observes.
//!
//! The load-bearing claim of this whole path is that overlapping changes
//! nothing observable. A test that only ran the overlapped program would pass
//! just as well against a runtime that never granted a lane, so the runs below
//! read the runtime's own grant count and refuse to accept a repeat that never
//! actually overlapped.
//!
//! Actualization is compile-time opt-in, so every case that expects a hand-out
//! emits through [`emit_with_overlap`], which is what `whitefootc --par`
//! compiles. Plain [`emit`] is the default compilation, and
//! `the_default_compilation_hands_nothing_out` is the case that pins what it
//! leaves out.

use std::path::Path;
use std::process::Command;

use super::{
    HOST_OPTIMIZATION_ARGUMENTS, PARALLEL_RUNTIME_SOURCE, build_executable, compile_and_run, emit,
    emit_with_overlap, module_requires_parallel_runtime, test_directory,
};

/// A claim-free recursive fold over a heap tree, the smallest shape that has
/// every eligible form at once: a self-recursive sibling pair inside `fold`,
/// sibling constructor pairs inside `pair`, `quad`, and `oct`, and a run of
/// four sibling calls in `main`. Its whole result is written to standard
/// output, so a difference anywhere in the tree is a difference in the bytes.
const OVERLAPPING_FOLD: &[u8] = br#"enum Node {
  Leaf(w: u64);
  Branch(left: box<Node>, right: box<Node>, w: u64);
}

fn leaf(w: own u64) -> result: own box<Node> allocates(heap) {
  let node = Leaf(w: w);
  return box_new(move node);
}

fn branch(left: own box<Node>, right: own box<Node>) -> result: own box<Node> allocates(heap) {
  let node = Branch(left: move left, right: move right, w: 0_u64);
  return box_new(move node);
}

fn pair(a: own u64, b: own u64) -> result: own box<Node> allocates(heap) {
  let l = leaf(w: a);
  let r = leaf(w: b);
  return branch(left: move l, right: move r);
}

fn quad(a: own u64, b: own u64, c: own u64, d: own u64) -> result: own box<Node> allocates(heap) {
  let l = pair(a: a, b: b);
  let r = pair(a: c, b: d);
  return branch(left: move l, right: move r);
}

fn oct(a: own u64, b: own u64, c: own u64, d: own u64, e: own u64, f: own u64, g: own u64, h: own u64) -> result: own box<Node> allocates(heap) {
  let l = quad(a: a, b: b, c: c, d: d);
  let r = quad(a: e, b: f, c: g, d: h);
  return branch(left: move l, right: move r);
}

fn mix(a: own u64, b: own u64) -> result: own u64 pure {
  let spun = irotl(a, 13_u32);
  let scattered = imulhi(b, 2654435761_u64);
  let blended = ixor(spun, b);
  return ixor(blended, scattered);
}

fn fold['b](node: &uniq 'b box<Node>) -> result: own u64 reads('b), writes('b) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold<'b>(node: move l);
      let b = fold<'b>(node: move r);
      let mixed = mix(a: a, b: b);
      set deref(slot) = mixed;
      return mixed;
    }
  }
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

fn spell['d](destination: &uniq 'd buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads('d), writes('d) {
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

command fn main(command.stdout as out: own Output) -> status: own ExitStatus allocates(heap), external, blocks {
  let t0 = oct(a: 1_u64, b: 2_u64, c: 3_u64, d: 4_u64, e: 5_u64, f: 6_u64, g: 7_u64, h: 8_u64);
  let t1 = oct(a: 9_u64, b: 10_u64, c: 11_u64, d: 12_u64, e: 13_u64, f: 14_u64, g: 15_u64, h: 16_u64);
  let t2 = oct(a: 17_u64, b: 18_u64, c: 19_u64, d: 20_u64, e: 21_u64, f: 22_u64, g: 23_u64, h: 24_u64);
  let t3 = oct(a: 25_u64, b: 26_u64, c: 27_u64, d: 28_u64, e: 29_u64, f: 30_u64, g: 31_u64, h: 32_u64);
  let half0 = branch(left: move t0, right: move t1);
  let half1 = branch(left: move t2, right: move t3);
  let root = branch(left: move half0, right: move half1);
  let report = buffer_new(8_u64, 0_u8);
  region 'tree {
    let value = fold<'tree>(node: &uniq 'tree root);
    region 'r {
      let filled = spell<'r>(destination: &uniq 'r report, at: 0_u64, value: value);
    }
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

/// Two sibling calls the judgment refuses: the second reads the first's
/// binding, so condition 1 denies the pair and nothing may be handed out.
const DEPENDENT_SIBLINGS: &[u8] = br#"fn twice(v: own u64) -> result: own u64 pure {
  return imax(v, v);
}

command fn main() -> status: own ExitStatus pure {
  let first = twice(v: 3_u64);
  let second = twice(v: first);
  let total = imax(first, second);
  return exit_status(code: 0_u8);
}
"#;

/// A program whose own functions are spelled like the runtime's entry points.
///
/// It overlaps, so the module carries the runtime symbols too, and both sets
/// have to coexist. `par_try_fork` and `par_join` are ordinary IDENTs
/// [FORM-3], so nothing may stop a writer from declaring them.
const RUNTIME_SHAPED_NAMES: &[u8] = br#"fn par_try_fork(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

fn par_join(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

fn par_thunk_0(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

command fn main() -> status: own ExitStatus pure {
  let a = par_try_fork(x: 1_u64);
  let b = par_join(x: 2_u64);
  let c = par_thunk_0(x: 3_u64);
  let ab = imax(a, b);
  let total = imax(ab, c);
  return exit_status(code: 0_u8);
}
"#;

/// The backend's own symbols never collide with a source function's.
///
/// A source function is emitted as `wf_` plus its IDENT, and [FORM-3] spells
/// IDENT `[a-z][a-z0-9_]*`, so the `wf__par_` prefix the runtime uses is
/// unreachable from source. Without that reservation this program is accepted
/// by the checker and then rejected by the host toolchain with a raw
/// `invalid redefinition of function` — an accepted program failing to build,
/// which no source-level diagnostic explains.
#[test]
fn a_program_named_like_the_runtime_still_compiles_and_links() {
    let module = emit_with_overlap(RUNTIME_SHAPED_NAMES);
    assert!(
        module_requires_parallel_runtime(&module),
        "the fixture must actually hand work out:\n{module}"
    );
    assert!(
        module.contains("define internal i64 @wf_par_try_fork(i64 "),
        "the source function keeps its own symbol:\n{module}"
    );
    assert!(
        module.contains("define weak ptr @wf__par_try_fork(ptr %fn, ptr %arg) {"),
        "the runtime keeps its reserved symbol:\n{module}"
    );
    let output = compile_and_run(&module);
    assert_eq!(output.status.code(), Some(0));
}

/// One handed-out call emits its frame, its outlined thunk, the lane offer,
/// and a join whose refusal edge calls the same thunk on the same frame.
#[test]
fn a_permitted_pair_is_outlined_offered_and_joined() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);

    // The thunk is the outlined call: it loads the arguments out of the frame,
    // calls the same monomorphized function the inline edge calls, and stores
    // the result back into the frame. Its number is the module's, so the
    // assertion is on the shape rather than on which group came first.
    assert!(
        module.contains("(ptr %frame) {\nentry:\n  %p0 = getelementptr inbounds "),
        "no outlined thunk over a frame:\n{module}"
    );
    assert!(
        module.contains("%result = call i64 @wf_fold(ptr %a0)"),
        "the thunk must call the same function the inline edge calls:\n{module}"
    );
    assert!(
        module.contains("  store i64 %result, ptr %slot\n  ret void\n"),
        "the thunk must leave its result in the frame:\n{module}"
    );
    // Both runtime entry points are the module's own weak definitions, so a
    // module that hands work out is still a complete program.
    assert!(
        module.contains("define weak ptr @wf__par_try_fork(ptr %fn, ptr %arg) {"),
        "no weak refusal of the lane offer:\n{module}"
    );
    assert!(
        module.contains("define weak void @wf__par_join(ptr %handle) {"),
        "no weak join:\n{module}"
    );

    // `fold`'s own recursive pair: the first call is offered, the second runs
    // inline on this thread, and only then is the offered one joined. The
    // ordering is what makes the overlap window exactly the second call.
    let body = function_body(&module, "@wf_fold");
    let offer = body
        .find("call ptr @wf__par_try_fork(ptr @wf__par_thunk_")
        .expect("fold must offer its first recursive call");
    let inline = body
        .find("call i64 @wf_fold(")
        .expect("fold must run its second recursive call inline");
    let join = body
        .find("call void @wf__par_join(ptr")
        .expect("fold must join what it offered");
    assert!(
        offer < inline,
        "the offer must precede the inline call:\n{body}"
    );
    assert!(
        inline < join,
        "the join must follow the inline call:\n{body}"
    );
    // The refused edge runs the same thunk on the same frame, so the two edges
    // are one lowering of one source call reached two ways.
    let refused = body
        .find("call void @wf__par_thunk_")
        .expect("the refused edge must run the thunk on this thread");
    assert!(
        inline < refused,
        "the refusal edge belongs to the join, not the offer:\n{body}"
    );
    // Nothing between the offer and the join reads the offered value: the
    // value is defined by the load in the block the join branches to.
    let read = body
        .find("\npar.done.")
        .expect("the joined value must be read in the join's own block");
    assert!(
        join < read,
        "the value must be read after the join:\n{body}"
    );
}

/// A pair the judgment denies emits exactly the sequential calls, with no
/// frame, no thunk, no offer, and no join anywhere in the module.
#[test]
fn a_denied_pair_emits_exactly_the_sequential_calls() {
    let module = emit_with_overlap(DEPENDENT_SIBLINGS);
    assert!(
        !module.contains("wf_par"),
        "a denied pair must name no part of the runtime:\n{module}"
    );
    let body = function_body(&module, "@wf_main");
    let calls: Vec<_> = body.match_indices("call i64 @wf_twice(").collect();
    assert_eq!(calls.len(), 2, "both calls stay ordinary calls:\n{body}");
}

/// A permitted pair whose first member is a borrowed binding is not handed
/// out, because promoting that binding reads the call's value at its
/// definition site — between the offer and the join, where the value does not
/// exist yet.
///
/// The judgment still permits the pair; this is the lowering refusing to
/// actualize a permission it cannot carry on one straight-line edge, and it
/// refuses by dropping the group rather than by moving the read.
#[test]
fn a_permitted_pair_whose_first_member_is_borrowed_is_not_handed_out() {
    let borrowed = br#"fn make() -> result: own u64 pure {
  return 7_u64;
}

fn peek['r](v: &'r u64) -> result: own u64 reads('r) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let first = make();
  let second = make();
  region 'r {
    let seen = peek<'r>(v: &'r first);
  }
  return exit_status(code: 0_u8);
}
"#;
    let module = emit_with_overlap(borrowed);
    assert!(
        !module.contains("wf_par"),
        "a borrowed first member must not be handed out:\n{module}"
    );

    // The same pair with only the *second* member borrowed is handed out: the
    // last member always runs on the calling thread, so its own value is read
    // after the join like every other.
    let trailing = br#"fn make() -> result: own u64 pure {
  return 7_u64;
}

fn peek['r](v: &'r u64) -> result: own u64 reads('r) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let first = make();
  let second = make();
  region 'r {
    let seen = peek<'r>(v: &'r second);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert!(
        emit_with_overlap(trailing).contains("call ptr @wf__par_try_fork(ptr @wf__par_thunk_"),
        "a borrowed last member does not stop the group"
    );
}

/// A module that hands nothing out is a complete program with no runtime, and
/// says so: the link path's own predicate is false, so no link anywhere adds
/// the runtime to it.
#[test]
fn a_module_that_hands_nothing_out_needs_no_runtime() {
    let module = emit_with_overlap(DEPENDENT_SIBLINGS);
    assert!(!module_requires_parallel_runtime(&module));
    let output = compile_and_run(&module);
    assert_eq!(output.status.code(), Some(0));

    // The overlapping module is the other half of the same statement: it does
    // hand work out, so linking the runtime is what gives it lanes.
    assert!(module_requires_parallel_runtime(&emit_with_overlap(
        OVERLAPPING_FOLD
    )));
}

/// The module carries a weak sequential answer to both runtime entry points,
/// and the runtime's own definitions replace them at link.
///
/// Without this the whole path could be silently sequential forever: every
/// other test here passes just as well when the weak refusal wins, because
/// refusing every lane is a correct execution.
#[test]
fn the_runtime_replaces_the_modules_weak_refusal() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);
    let directory = test_directory();

    // Linked with nothing: the module's own weak definitions answer, every
    // lane is refused, and the program still produces its whole result.
    let alone = build_executable_without_runtime(&module, &directory);
    let sequential = Command::new(&alone)
        .output()
        .expect("run the linked module");
    assert_eq!(sequential.status.code(), Some(0));
    assert_eq!(
        sequential.stdout.len(),
        8,
        "the fold must report eight bytes"
    );

    // Linked with the runtime: the strong definitions win. The count is the
    // runtime's own, reported at process exit by the observer unit, so a
    // link that kept the weak refusal reports zero here and fails.
    let (granted, parallel) = run_counting_grants(&module, &directory, "4");
    assert_eq!(parallel.status.code(), Some(0));
    assert_eq!(
        parallel.stdout, sequential.stdout,
        "granting lanes must not move one byte of the result"
    );
    assert!(
        granted > 0,
        "the runtime granted no lane, so nothing was overlapped"
    );

    // And the runtime's own default: one lane of execution is the calling
    // thread alone, so the pool never starts and every offer is refused.
    let (unset, quiet) = run_counting_grants(&module, &directory, "1");
    assert_eq!(quiet.stdout, sequential.stdout);
    assert_eq!(unset, 0, "WF_WORKERS=1 must never start the pool");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The repeat: the same program, at every worker count, over and over, is the
/// same bytes. Lanes are granted at every count above one, so each repeat is a
/// real overlapped execution rather than the refusal path run again.
#[test]
fn an_overlapped_program_reports_one_byte_sequence_at_every_worker_count() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    let mut runs = Vec::new();
    for workers in ["1", "2", "4", "8"] {
        for _ in 0..5 {
            let output = Command::new(&executable)
                .env("WF_WORKERS", workers)
                .output()
                .expect("run the overlapped program");
            assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
            runs.push((format!("WF_WORKERS={workers}"), output.stdout));
        }
    }
    // The sequential reference is the same program with no runtime linked at
    // all, so the repeat is compared against today's execution and not only
    // against itself.
    let alone = build_executable_without_runtime(&module, &directory);
    let reference = Command::new(&alone)
        .output()
        .expect("run the linked module");
    runs.push(("no runtime".to_owned(), reference.stdout));

    identical(&runs).expect("overlapping must not move one byte of the result");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// Overlap lowering is compile-time opt-in: the default compilation of a
/// program full of eligible sites names no part of the runtime.
///
/// This is what makes the feature free when it is not asked for. The judgment
/// still ran — the same source with the lowering switched on hands work out,
/// which is the second half of this test — so what the default drops is the
/// outlining and nothing else. The outlining is not free: it passes arguments
/// through a memory frame and is reached through a function pointer, so the
/// call cannot be inlined, and the batch audit measured that alone at about
/// 1.2x on the layout demo and 2.1x on `fib(38)` with no runtime linked and
/// `WF_WORKERS` unset.
#[test]
fn the_default_compilation_hands_nothing_out() {
    let default = emit(OVERLAPPING_FOLD);
    assert!(
        !default.contains("wf__par_"),
        "the default compilation must name no runtime symbol:\n{default}"
    );
    assert!(
        !default.contains("(ptr %frame) {"),
        "the default compilation must outline no thunk:\n{default}"
    );
    assert!(
        !module_requires_parallel_runtime(&default),
        "no link path may add the runtime to a default build"
    );

    // The same source asked for lanes: the sites were there all along, so the
    // assertions above are about the option and not about the program.
    let requested = emit_with_overlap(OVERLAPPING_FOLD);
    assert!(
        requested.contains("call ptr @wf__par_try_fork(ptr @wf__par_thunk_"),
        "the fixture must hand work out when asked, or this test is vacuous"
    );
    assert!(module_requires_parallel_runtime(&requested));
}

/// The differential: the same source lowered *without* any overlap group
/// produces the same bytes as the overlapped lowering, at every worker count.
///
/// This is the comparison the rest of this module cannot make. Every other
/// test here links one emitted module two ways, so a defect introduced by the
/// outlining itself — a moved read, a hoisted operand, a join in the wrong
/// place — is present in the reference too and compares equal. The reference
/// here is the default compilation of the same source, whose calls were never
/// handed out, which is the only way an overlap's "changes nothing observable"
/// claim can be checked against something other than itself.
#[test]
fn the_overlapped_lowering_agrees_with_the_lowering_that_hands_nothing_out() {
    let sequential_module = emit(OVERLAPPING_FOLD);
    assert!(
        !module_requires_parallel_runtime(&sequential_module),
        "the reference module must contain no hand-out at all"
    );
    assert!(
        module_requires_parallel_runtime(&emit_with_overlap(OVERLAPPING_FOLD)),
        "the overlapped module must hand work out, or the comparison is vacuous"
    );

    let directory = test_directory();
    let reference = Command::new(build_executable(&sequential_module, &directory))
        .output()
        .expect("run the module that hands nothing out");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(
        reference.stdout.len(),
        8,
        "the fold must report eight bytes"
    );

    let overlapped = build_executable(&emit_with_overlap(OVERLAPPING_FOLD), &directory);
    let mut runs = vec![("no overlap lowering".to_owned(), reference.stdout)];
    for workers in ["1", "2", "4", "8"] {
        let output = Command::new(&overlapped)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the overlapped program");
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
        runs.push((format!("WF_WORKERS={workers}"), output.stdout));
    }
    identical(&runs).expect("outlining a call must not move one byte of the result");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The negative control for the repeat above: its comparison reports a
/// difference when one is present, so a green repeat is a claim about the
/// program rather than about the comparison.
#[test]
fn the_repeat_comparison_reports_an_injected_difference() {
    let same = vec![
        ("WF_WORKERS=1".to_owned(), b"abcdefgh".to_vec()),
        ("WF_WORKERS=2".to_owned(), b"abcdefgh".to_vec()),
    ];
    identical(&same).expect("equal runs must compare equal");

    for injected in [b"abcdefgi".to_vec(), b"abcdefg".to_vec(), Vec::new()] {
        let mut runs = same.clone();
        runs.push(("WF_WORKERS=4".to_owned(), injected));
        let report = identical(&runs).expect_err("a differing run must be reported");
        assert!(report.contains("WF_WORKERS=4"), "{report}");
    }
}

/// The negative control the repeat actually needs: a lowering with its joins
/// removed is caught.
///
/// The comparison control above proves the byte comparison reports a
/// difference it is handed. This one proves the *program* comparison reports a
/// real lowering defect: the branch's own emitted module, with both join calls
/// struck out, linked against the real runtime. A missed join lets the calling
/// thread read a frame slot the worker has not written and lets the frame's
/// activation return underneath the worker, so a run either publishes
/// different bytes or dies.
///
/// Detection is per-run and not certain — a granted lane sometimes finishes
/// before the read anyway — so the control runs the injected build twelve
/// times and requires that at least one run disagree with the reference. The
/// measured per-run detection rate is about four in five, which puts a false
/// green here below one in a hundred million.
#[test]
fn the_repeat_reports_a_lowering_whose_joins_were_removed() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);
    let directory = test_directory();

    let reference = Command::new(build_executable(&module, &directory))
        .env("WF_WORKERS", "1")
        .output()
        .expect("run the intact program");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(reference.stdout.len(), 8);

    let joinless = module.replace(
        "  call void @wf__par_join(ptr ",
        "  call void @wf__par_join_removed(ptr ",
    );
    assert_ne!(joinless, module, "the injection must change the module");
    let joinless = format!(
        "{joinless}\ndefine internal void @wf__par_join_removed(ptr %handle) {{\nentry:\n  ret void\n}}\n"
    );
    let broken = build_executable(&joinless, &directory);

    let mut disagreements = 0;
    for _ in 0..12 {
        let output = Command::new(&broken)
            .env("WF_WORKERS", "4")
            .output()
            .expect("run the join-less program");
        if output.status.code() != Some(0) || output.stdout != reference.stdout {
            disagreements += 1;
        }
    }
    assert!(
        disagreements > 0,
        "removing every join changed nothing observable in twelve runs, \
         so this comparison cannot see a missed join"
    );

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// Every run produced the same bytes, or the first run that did not.
fn identical(runs: &[(String, Vec<u8>)]) -> Result<(), String> {
    let Some((first_name, first)) = runs.first() else {
        return Err("no run to compare".to_owned());
    };
    for (name, bytes) in &runs[1..] {
        if bytes != first {
            return Err(format!(
                "{name} produced {bytes:?}, but {first_name} produced {first:?}"
            ));
        }
    }
    Ok(())
}

/// Links one module with no runtime at all, whatever its own predicate says.
fn build_executable_without_runtime(module: &str, directory: &Path) -> std::path::PathBuf {
    let assembly = directory.join("alone.ll");
    let executable = directory.join("alone");
    std::fs::write(&assembly, module).expect("write the module");
    let linked = Command::new("/usr/bin/clang")
        .arg("-x")
        .arg("ir")
        .arg(&assembly)
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        linked.status.success(),
        "a module that hands work out must link with no runtime:\n{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    std::fs::remove_file(&assembly).expect("remove the module");
    executable
}

/// Links one module against the runtime plus an observer that reports the
/// runtime's own grant count at process exit, then runs it at `workers`.
///
/// The observer reads `wf__par_grants`, which no Whitefoot construct can name;
/// it exists exactly so a pool that never grants a lane cannot pass for one
/// that does.
fn run_counting_grants(
    module: &str,
    directory: &Path,
    workers: &str,
) -> (u64, std::process::Output) {
    let assembly = directory.join("counted.ll");
    let runtime = directory.join("counted_runtime.c");
    let observer = directory.join("observer.c");
    let executable = directory.join("counted");
    std::fs::write(&assembly, module).expect("write the module");
    std::fs::write(&runtime, PARALLEL_RUNTIME_SOURCE).expect("write the runtime");
    std::fs::write(
        &observer,
        "#include <stdio.h>\nextern unsigned long wf__par_grants;\n__attribute__((destructor)) static void wf__par_report(void) {\n    fprintf(stderr, \"grants=%lu\\n\", wf__par_grants);\n}\n",
    )
    .expect("write the observer");
    let linked = Command::new("/usr/bin/clang")
        .arg("-pthread")
        .arg("-x")
        .arg("ir")
        .arg(&assembly)
        .arg("-x")
        .arg("c")
        .arg(&runtime)
        .arg(&observer)
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        linked.status.success(),
        "the runtime and its observer must link:\n{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let output = Command::new(&executable)
        .env("WF_WORKERS", workers)
        .output()
        .expect("run the counted program");
    let report = String::from_utf8_lossy(&output.stderr).into_owned();
    let granted = report
        .lines()
        .find_map(|line| line.strip_prefix("grants="))
        .and_then(|count| count.trim().parse::<u64>().ok())
        .unwrap_or_else(|| panic!("the observer must report a grant count, got {report:?}"));
    for path in [assembly, runtime, observer, executable] {
        std::fs::remove_file(path).expect("remove a counted-run artifact");
    }
    (granted, output)
}

/// The text of one emitted function definition, from its `define` line to its
/// closing brace.
fn function_body<'module>(module: &'module str, symbol: &str) -> &'module str {
    let opening = format!("{symbol}(");
    let start = module
        .match_indices(&opening)
        .find_map(|(offset, _)| {
            let line = module[..offset]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            module[line..offset].starts_with("define").then_some(line)
        })
        .unwrap_or_else(|| panic!("the module must define {symbol}:\n{module}"));
    let end = module[start..]
        .find("\n}\n")
        .map(|offset| start + offset + 2)
        .expect("a definition must close");
    &module[start..end]
}
