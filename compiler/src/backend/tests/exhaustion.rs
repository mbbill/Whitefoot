//! The resource-exhaustion floor: what an execution does when it runs out.
//!
//! Exhaustion is the one abnormal end a *correct* program can reach. A false
//! `claim` cannot happen in a reviewed program and yet gets a byte-exact
//! [DIAG-3] record; running out of stack or heap needs no source defect at all
//! and, before this floor, produced zero bytes and a bare host signal. These
//! cases pin the floor that closes that asymmetry.
//!
//! Two separate obligations live here and must not be confused:
//!
//! - *containment* — a frame larger than the guard region walks its pages on
//!   the way down, so it cannot step over the guard into whatever is mapped
//!   below. This is a safety property, not a reporting one: without it an
//!   accepted program can silently overwrite a neighbouring thread's live
//!   stack. [`every_generated_definition_carries_the_stack_probe`] is its
//!   case.
//! - *reporting* — exhaustion ends the process by a defined abort that first
//!   writes one fixed record naming only the resource class. The record
//!   carries no `rule_id`, no function, and no node path, and that absence is
//!   what mechanically distinguishes it from a [DIAG-3] record.
//!
//! The record's bytes are fixed by two independent constraints that happen to
//! agree. A signal handler may only reach async-signal-safe facilities, which
//! admits a constant string written with `write` and essentially nothing else;
//! and [PAR-1] requires observables to be identical under every permitted
//! schedule, which forbids the record from naming a worker, a thread, a depth,
//! or an address. Either one alone would force a fixed constant.

use std::process::Command;

use super::{build_executable, compile, test_directory};

/// The attribute group [`crate::backend::emitter`] gives every definition, and
/// the value it carries on this host.
///
/// The value is the host C compiler's own frame-probing helper, so a generated
/// frame walks its pages exactly the way the runtime's translation unit does.
#[cfg(target_os = "macos")]
const HOST_STACK_PROBE: &str = "\"probe-stack\"=\"__chkstk_darwin\"";
#[cfg(not(target_os = "macos"))]
const HOST_STACK_PROBE: &str = "\"probe-stack\"=\"inline-asm\"";

/// A program that reaches several kinds of generated definition at once: a
/// heap box and its compiler-generated drop glue, a recursive walk, a system
/// transfer, and the entry itself.
const MIXED_DEFINITIONS: &[u8] = br#"enum Chain {
  End();
  More(next: box<Chain>);
}

fn depth['r](chain: &'r box<Chain>) -> result: own u64 reads('r) {
  match deref(deref(chain)) {
    End() => {
      return 0_u64;
    }
    More(next: inner) => {
      let below = depth<'r>(chain: inner);
      return below +wrap 1_u64;
    }
  }
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let end = End();
  let bottom = box_new(move end);
  let one = More(next: move bottom);
  let boxed = box_new(move one);
  region 'chain {
    let measured = depth<'chain>(chain: &'chain boxed);
    if ieq(measured, 1_u64) {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

/// Every definition the module emits carries the probe attribute, and the
/// group it names is the host's.
///
/// The count equality is the point of the case. Containment is a completeness
/// property — one unprobed large frame is enough to jump the guard region into
/// a neighbouring thread's live stack — so a case that merely found *some*
/// probed definition would pass against a module that left the drop glue, a
/// clone, or a thunk unprobed. Counting both sides is what makes "every
/// generated function" checkable rather than asserted.
#[test]
fn every_generated_definition_carries_the_stack_probe() {
    for module in [
        compile(MIXED_DEFINITIONS),
        super::emit_with_overlap(MIXED_DEFINITIONS),
    ] {
        let definitions = module
            .lines()
            .filter(|line| line.starts_with("define "))
            .count();
        assert!(
            definitions > 1,
            "the fixture must reach more than one definition:\n{module}"
        );
        let probed = module
            .lines()
            .filter(|line| line.starts_with("define ") && line.ends_with(" #0 {"))
            .count();
        assert_eq!(
            probed,
            definitions,
            "every generated definition must carry the probe group; \
             {} of {definitions} did not:\n{module}",
            definitions - probed
        );
        assert_eq!(
            module.matches("attributes #0 = { ").count(),
            1,
            "the module declares its one attribute group once:\n{module}"
        );
        assert!(
            module.contains(&format!("attributes #0 = {{ {HOST_STACK_PROBE} }}")),
            "the group must name this host's probing helper:\n{module}"
        );
    }
}

/// The probe is emitted only for a frame past the page threshold, so an
/// ordinary program pays nothing for it.
///
/// This is what makes the containment fix free rather than a trade: the
/// attribute changes the machine code of exactly the functions whose frames
/// could jump the guard, and leaves every other function alone.
#[test]
fn an_ordinary_frame_emits_no_probe_call() {
    let directory = test_directory();
    let executable = build_executable(&compile(MIXED_DEFINITIONS), &directory);
    let symbols = Command::new("/usr/bin/nm")
        .arg(&executable)
        .output()
        .expect("read the linked symbol table");
    let listed = String::from_utf8_lossy(&symbols.stdout);
    assert!(
        !listed.contains("chkstk"),
        "no ordinary frame reaches the probing helper:\n{listed}"
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A recursion whose depth is a parameter, in a shape the host optimizer
/// cannot turn back into a loop.
///
/// The recursive result is consumed by an addition after a second call
/// returns, so there is no tail call to eliminate and each level really takes
/// a frame. `spine`'s two callees are also an eligible overlap pair, which is
/// what lets the `--par` build carry the deep half onto a lane.
fn spine_source(depth: u64) -> Vec<u8> {
    format!(
        r#"fn leafval(v: own f64) -> result: own f64 pure {{
  return fmul.strict(v, 0.5_f64);
}}

fn spine(depth: own u64, v: own f64) -> result: own f64 pure {{
  let done = ieq(depth, 0_u64);
  if done {{
    return v;
  }}
  let next = depth -wrap 1_u64;
  let a = spine(depth: next, v: v);
  let b = leafval(v: v);
  return fadd.strict(a, b);
}}

command fn main() -> status: own ExitStatus pure {{
  let total = spine(depth: {depth}_u64, v: 1.0009765625_f64);
  let bits = reinterpret<f64, u64>(total);
  let low = iand(bits, 1_u64);
  match cvt<u64, u8>(low) {{
    Ok(value: byte) => {{
      return exit_status(code: byte);
    }}
    Err(error: wide) => {{
      return exit_status(code: 9_u8);
    }}
  }}
}}
"#
    )
    .into_bytes()
}

/// Deep enough that a lane sized the way lanes used to be sized cannot hold it,
/// and far inside the stack every thread now gets.
const LANE_DEPTH: u64 = 2_000_000;

/// Deeper than the entry's own stack, so the sequential run reaches the guard
/// page the floor exists to report.
const RUNAWAY_DEPTH: u64 = 100_000_000;

/// The record is the resource class and nothing else.
///
/// The absent fields carry the whole distinction. A [DIAG-3] trap record names
/// a `rule_id`, the function, and the node path, because a false claim is
/// something the writer did. Exhaustion is not: no operation in the program
/// has "runs out of stack" in its meaning, and the same source on the same
/// input succeeds or fails depending on the environment. A record that
/// attributed it to source would be claiming something untrue.
fn assert_resource_record(stderr: &[u8], resource: &str) {
    let text = String::from_utf8_lossy(stderr);
    assert_eq!(
        text,
        format!("{{\"resource\":\"{resource}\"}}\n"),
        "an exhausted execution writes exactly its resource record"
    );
    assert_eq!(text.lines().count(), 1, "one record, one line: {text:?}");
    for absent in ["rule_id", "function", "node_path", "message"] {
        assert!(
            !text.contains(absent),
            "a resource record must not carry {absent}, which would make it \
             indistinguishable from a [DIAG-3] trap record: {text:?}"
        );
    }
}

/// The depth a program can reach is the compiler's number, not the shell's.
///
/// The environment's limit is cut to a megabyte here, well under what this
/// recursion needs, and the program still runs to completion — because the
/// entry does not run on the stack the host started the process with. Before
/// the floor the same program at this depth died with a bare signal under an
/// ordinary eight-megabyte limit, and whether it died at all depended on who
/// ran it.
#[test]
fn the_entry_runs_on_a_stack_the_compiler_sized() {
    let directory = test_directory();
    let executable = build_executable(&compile(&spine_source(LANE_DEPTH)), &directory);
    let constrained = Command::new("/bin/sh")
        .arg("-c")
        .arg(format!("ulimit -s 1024; exec {}", executable.display()))
        .output()
        .expect("run the program under a reduced stack limit");
    assert_eq!(
        constrained.status.code(),
        Some(0),
        "a recursion inside the compiler's ceiling must not depend on the \
         environment's: {}",
        String::from_utf8_lossy(&constrained.stderr)
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// An entry that runs out of stack writes one record and aborts.
///
/// This is the case the whole floor exists for: before it, the process died
/// with a bare SIGSEGV and not one byte said why — the only abnormal end a
/// correct program can reach was the only one with no diagnosis.
#[test]
fn an_exhausted_entry_writes_one_resource_record() {
    let directory = test_directory();
    let executable = build_executable(&compile(&spine_source(RUNAWAY_DEPTH)), &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the runaway recursion");
    assert_eq!(
        output.status.code(),
        None,
        "an exhausted execution ends by abort, not by a returned status"
    );
    assert_resource_record(&output.stderr, "stack");
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// An overlapped run that exhausts its stack writes the same record, whichever
/// thread was carrying the recursion.
///
/// Worth its own case because the two thread classes arrive as *different*
/// signals — an entry overflow as SIGSEGV, a lane overflow as SIGBUS — so a
/// disposition that took only SIGSEGV would pass the case above and still miss
/// every worker overflow, which is exactly the class the parallel default
/// introduced.
///
/// It used to be the *depth* that said a death here was a lane's: lanes were
/// sized from `RLIMIT_STACK` and the entry from the compiler's own constant, so
/// a depth between the two ceilings could only die on a lane. That asymmetry is
/// what [`a_deep_recursion_completes_at_every_worker_count`] removes, so no
/// depth discriminates any more. What is left is the standard every run is held
/// to: past the ceiling a run must write exactly the record and must not die
/// bare, and at the default pool the deep descent reaches a lane in the great
/// majority of runs — measured 27 of 30 on this shape while the ceilings still
/// differed. A bare signal or a partial record fails this on the first run.
#[test]
fn an_exhausted_lane_writes_the_same_resource_record() {
    let directory = test_directory();
    let module = super::emit_with_overlap(&spine_source(RUNAWAY_DEPTH));
    let executable = build_executable(&module, &directory);
    for _ in 0..3 {
        let output = Command::new(&executable)
            .output()
            .expect("run the overlapped recursion");
        assert_eq!(
            output.status.code(),
            None,
            "a recursion past every thread's ceiling must not return a status"
        );
        assert_resource_record(&output.stderr, "stack");
    }
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// Whether a recursion completes is not decided by a steal race.
///
/// A stolen call is an ordinary Whitefoot call that starts at the bottom of the
/// stealing lane's own stack. When a lane's stack was smaller than the entry's,
/// that made a steal *lose* headroom, and the same binary on the same input
/// either finished or died depending on which thread got there first: on this
/// recursion, 11 of 30 runs at two workers, 3 of 30 at eight, and 4 of 30 at
/// the default. [PAR-1] survives that — overlap-resource exhaustion is outside
/// its observables — but "does my program run" is not something a schedule may
/// decide.
///
/// A lane sized like the entry makes a steal strictly headroom-positive
/// instead: no thread has less room than the entry, and a stolen subtree gets a
/// whole fresh stack, so the deepest any schedule reaches is at least what the
/// no-steal schedule reaches. The case is the whole worker range rather than
/// one setting because the failure it guards was a distribution, not a
/// threshold — it showed up at every count above one, and worst where the pool
/// was largest.
#[test]
fn a_deep_recursion_completes_at_every_worker_count() {
    let directory = test_directory();
    let module = super::emit_with_overlap(&spine_source(LANE_DEPTH));
    let executable = build_executable(&module, &directory);
    for workers in ["0", "1", "2", "4", "8", "16"] {
        for _ in 0..3 {
            let output = Command::new(&executable)
                .env("WF_WORKERS", workers)
                .output()
                .expect("run the overlapped recursion");
            assert_eq!(
                output.status.code(),
                Some(0),
                "a recursion inside every thread's ceiling died at \
                 WF_WORKERS={workers}, so its liveness depends on the \
                 schedule: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    for _ in 0..3 {
        let output = Command::new(&executable)
            .output()
            .expect("run the overlapped recursion at the shipped default");
        assert_eq!(
            output.status.code(),
            Some(0),
            "the shipped default is the setting a binary handed to somebody \
             runs under: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A fault that is not exhaustion keeps its own signal, status, and core dump.
///
/// This is the property that stops the floor from becoming a diagnostic that
/// misdirects. A handler that reported every SIGSEGV as an exhausted stack
/// would make a genuine memory defect and a deep recursion look identical at
/// the point of death, which is worse than saying nothing: the reader would
/// go looking for a recursion that is not there. So the handler tests the
/// faulting address against the running thread's own stack bounds, and for
/// anything outside them puts the default disposition back and returns, which
/// re-executes the faulting instruction with the floor no longer in the way.
///
/// The faulting body is C because Whitefoot cannot express a wild pointer —
/// that is the language's whole point, and it is why this case cannot be
/// written as a `.wf` fixture. Everything under test is still the shipped
/// mechanism: the same translation unit, installed the same way, entered
/// through the same `wf__floor_run`, with only the program body replaced.
const WILD_FAULT_BODY: &str = r#"extern int wf__floor_run(int argc, char **argv);

int wf__main_body(int argc, char **argv) {
    (void)argc;
    (void)argv;
    *(volatile int *)0xdeadb000 = 1;
    return 0;
}

int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;

#[test]
fn a_fault_that_is_not_exhaustion_keeps_its_own_disposition() {
    let directory = test_directory();
    let body = directory.join("wild_body.c");
    let floor = directory.join("wf_floor.c");
    let executable = directory.join("wild");
    std::fs::write(&body, WILD_FAULT_BODY).expect("write the faulting body");
    std::fs::write(&floor, crate::FLOOR_RUNTIME_SOURCE).expect("write the floor runtime");
    let compiled = Command::new("/usr/bin/clang")
        .arg("-pthread")
        .arg("-x")
        .arg("c")
        .arg(&body)
        .arg(&floor)
        .args(super::HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        compiled.status.success(),
        "the floor and a faulting body must link:\n{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let output = Command::new(&executable).output().expect("run the fault");
    assert!(
        output.stderr.is_empty(),
        "the floor must add nothing to a fault that is not its own: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        output.status.code(),
        None,
        "a wild fault still ends the process by its own signal"
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A heap request no host can serve, read at an index the compiler knows only
/// through a `u8` range.
///
/// The read is what keeps the allocation alive. At the shipped optimization
/// level LLVM deletes an allocation whose contents nothing observes — it
/// forwards the fill value to the loads and removes the `malloc`/`free` pair,
/// taking the refusal edge with it — so a naively written case would ask for
/// sixteen terabytes, return normally, and test nothing at all. Routing the
/// index through a type range leaves the optimizer unable to decide the load.
const REFUSED_ALLOCATION: &[u8] = br#"fn giant(i: own u8) -> result: own u8 allocates(heap) {
  let b = buffer_new(4000000000000000000_u64, 7_u8);
  let wide = cvt<u8, u64>(i);
  let element = b[wide];
  return element;
}

command fn main(command.args as args: own Args) -> status: own ExitStatus allocates(heap) {
  let count = 0_u64;
  region 'invocation {
    set count = args_count<'invocation>(args: &'invocation args);
  }
  match cvt<u64, u8>(count) {
    Ok(value: v) => {
      let r = giant(i: v);
      return exit_status(code: r);
    }
    Err(error: e) => {
      return exit_status(code: 9_u8);
    }
  }
}
"#;

/// A module carrying both a claim and heap storage, so one writer serves both
/// record classes and the case can show they do not bleed into each other.
///
/// The claim is true and the source is accepted; the falsehood is injected
/// into the checked IR after acceptance, the same way the trap-latch cases do
/// it, because the language admits no source that states a false claim. The
/// unused `False()` binding is what the injection redirects the claim's
/// condition to, so the defect is a property of the run rather than of the
/// source.
const CLAIM_AND_HEAP: &[u8] = br#"fn pick(seed: own u64) -> result: own u64 allocates(heap), traps {
  let scratch = buffer_new(4_u64, 0_u8);
  let values = array_new<u64, 8>(1_u64);
  let bounded = imin(seed, 7_u64);
  let in_range = ilt(bounded, 8_u64);
  let injected_false = False();
  claim index_in_range: in_range because "premises: bounded is the minimum of the parameter seed and seven, and values has length eight\nderivation: a minimum is at most either operand, so bounded is at most seven and therefore below eight\nconclusion: ilt(bounded, 8_u64) is true\nchecker gap: ENT does not publish the result range of imin\nconsumers: the following length-eight array subscript uses bounded";
  let picked = values[bounded];
  return picked;
}

command fn main() -> status: own ExitStatus allocates(heap), traps {
  let value = pick(seed: 3_u64);
  match cvt<u64, u8>(value) {
    Ok(value: byte) => {
      return exit_status(code: byte);
    }
    Err(error: wide) => {
      return exit_status(code: 9_u8);
    }
  }
}
"#;

/// One program reaching every allocation form the emitter lowers: a filled
/// buffer, a vacant one, a heap box, and an arena node.
///
/// The lengths are constants so the fit obligation discharges statically and
/// the fixture stays about the refusal edges rather than about proving a
/// dynamic length fits.
const ALL_HEAP_FORMS: &[u8] = br#"fn shapes(n: own u64) -> result: own u64 allocates(heap) {
  let filled = buffer_new(4_u64, 5_u64);
  let vacant = buffer_vacant<u32>(4_u64);
  let boxed = box_new(7_u64);
  let held = deref(boxed);
  let filled_len = len(filled);
  let vacant_len = len(vacant);
  let total = held +wrap filled_len;
  set total = total +wrap vacant_len;
  region 'a {
    let kept = arena_new<'a, u64>(3_u64);
    let seen = deref(kept);
    set total = total +wrap seen;
  }
  return total;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let total = shapes(n: 4_u64);
  match cvt<u64, u8>(total) {
    Ok(value: byte) => {
      return exit_status(code: byte);
    }
    Err(error: wide) => {
      return exit_status(code: 9_u8);
    }
  }
}
"#;

/// An allocation the host refuses ends the process the same way an exhausted
/// stack does: one record naming the resource, then a defined abort.
///
/// Before this, a refused allocation was a bare `abort()` with zero bytes —
/// the same observable event as a false claim and as a corrupted-heap abort
/// inside the allocator itself, with nothing to tell a reader which had
/// happened.
#[test]
fn an_allocation_the_host_refuses_writes_one_resource_record() {
    let directory = test_directory();
    let executable = build_executable(&compile(REFUSED_ALLOCATION), &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the refused allocation");
    assert_eq!(
        output.status.code(),
        None,
        "a refused allocation ends by abort, not by a returned status"
    );
    assert_resource_record(&output.stderr, "heap");
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// Every allocation-refusal edge reaches the resource abort, not a bare one.
///
/// The completeness matters the same way the probe attribute's does: a module
/// that routed three of its four refusal edges and left the fourth calling
/// `@abort` directly would still die silently on exactly the allocation that
/// took the fourth path, and nothing about the program would say which.
#[test]
fn every_allocation_refusal_edge_reaches_the_resource_abort() {
    let module = compile(ALL_HEAP_FORMS);
    let lines: Vec<&str> = module.lines().collect();
    for refusal in [
        "box.new.oom.",
        "arena.new.oom.",
        "buffer.fill.oom.",
        "buffer.vacant.oom.",
    ] {
        let mut found = 0;
        for (index, line) in lines.iter().enumerate() {
            if !line.starts_with(refusal) || !line.ends_with(':') {
                continue;
            }
            found += 1;
            assert_eq!(
                lines.get(index + 1).copied().unwrap_or_default(),
                "  call void @wf_resource_abort()",
                "the {line} edge must reach the resource abort, not a bare one"
            );
        }
        assert!(
            found > 0,
            "the fixture must reach a {refusal} edge:\n{module}"
        );
    }
}

/// A claim trap still writes exactly its own record, and nothing else.
///
/// The two classes share one writer and one latch, which is what makes "no
/// execution produces both records" a mechanism rather than a hope. This case
/// is the other half of that: sharing the writer must not let the resource
/// record leak into a trap's output. The distinction lives in the bytes — a
/// [DIAG-3] record names a rule, a function, and a node path; a resource
/// record names a resource class and nothing else — so the check is on the
/// bytes.
#[test]
fn a_claim_trap_still_writes_only_its_own_record() {
    let module =
        super::emit_with_overlap_and_false_claims(CLAIM_AND_HEAP, &[("pick", "index_in_range")]);
    assert!(
        module.contains("@.wf_resource.heap"),
        "the fixture must carry heap storage, or it shows nothing:\n{module}"
    );
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    let output = Command::new(&executable)
        .env("WF_WORKERS", "1")
        .output()
        .expect("run the defective program");
    let text = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), None, "a trap aborts");
    assert_eq!(text.lines().count(), 1, "exactly one record: {text:?}");
    assert!(
        text.starts_with("{\"rule_id\":\"CLM-1\",\"message\":\"index_in_range\""),
        "the trap must write its own [DIAG-3] record: {text:?}"
    );
    assert!(
        !text.contains("\"resource\""),
        "a claim trap must not carry a resource record: {text:?}"
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A recursion whose every activation carries an array far larger than a
/// guard page, written and read at an index only the run knows so the frame
/// cannot be shrunk away.
///
/// After the host inliner merges several levels together each activation moves
/// the stack pointer by roughly three hundred kilobytes at once.
const LARGE_FRAME_SPINE: &[u8] =
    br#"fn spine(depth: own u64, v: own u64, i: own u8) -> result: own u64 pure {
  let pad = array_new<u64, 7168>(v);
  let wide = cvt<u8, u64>(i);
  set pad[wide] = depth;
  let done = ieq(depth, 0_u64);
  if done {
    return pad[wide];
  }
  let next = depth -wrap 1_u64;
  let a = spine(depth: next, v: v, i: i);
  let b = pad[wide];
  return a +wrap b;
}

command fn main(command.args as args: own Args) -> status: own ExitStatus pure {
  let count = 0_u64;
  region 'invocation {
    set count = args_count<'invocation>(args: &'invocation args);
  }
  match cvt<u64, u8>(count) {
    Ok(value: idx) => {
      let depth = count *wrap 20000_u64;
      let r = spine(depth: depth, v: 3_u64, i: idx);
      let ok = igt(r, 0_u64);
      if ok {
        return exit_status(code: 0_u8);
      }
      return exit_status(code: 1_u8);
    }
    Err(error: e) => {
      return exit_status(code: 9_u8);
    }
  }
}
"#;

/// A frame far larger than the guard region is still reported, not absorbed.
///
/// This is the behaviour the probe attribute buys, as distinct from the
/// attribute being present. A frame that moves the stack pointer three hundred
/// kilobytes in one step can clear the whole guard region without touching it;
/// what happens next depends on what is mapped where it lands. If that memory
/// is mapped — under the pool, the next lane's stack is packed a few pages
/// below — the write succeeds and the program carries on with frames outside
/// its own stack, eventually returning an answer for a computation that never
/// fit. Nothing about that run says anything went wrong.
///
/// Ablating just this attribute from just this function, on this program,
/// turns the run below from a reported death into exactly that: the same frame
/// arithmetic to the instruction, and a normal exit 0. So the case here is
/// that an overflow this shape reaches the guard and says so.
#[test]
fn a_frame_larger_than_the_guard_region_is_still_reported() {
    let directory = test_directory();
    let executable = build_executable(&compile(LARGE_FRAME_SPINE), &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the large-frame recursion");
    assert_eq!(
        output.status.code(),
        None,
        "a recursion this deep cannot fit any stack, so it must not return: \
         {:?}",
        output.status
    );
    assert_resource_record(&output.stderr, "stack");
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

// ------------------------------------------- the compiler's own recursion

/// A boxed spine whose length is a runtime quantity, built by a counted loop.
///
/// The source contains no recursive function at all. The only recursion in the
/// program used to be the one the compiler generated to *destroy* the value at
/// scope exit, which is the point: a writer looking at this program can see no
/// depth to bound, cannot instrument the traversal, and cannot avoid it.
fn boxed_spine_source(depth: u64) -> Vec<u8> {
    format!(
        r#"enum Tree {{
  Leaf();
  Branch(left: box<Tree>, right: box<Tree>);
}}

struct Holder {{
  node: box<Tree>;
}}

fn boxed_leaf() -> result: own box<Tree> allocates(heap) {{
  let leaf = Leaf();
  return box_new(move leaf);
}}

fn boxed_branch(left: own box<Tree>, right: own box<Tree>) -> result: own box<Tree> allocates(heap) {{
  let branch = Branch(left: move left, right: move right);
  return box_new(move branch);
}}

command fn main() -> status: own ExitStatus allocates(heap) {{
  let seed = boxed_leaf();
  let held = Holder(node: move seed);
  for @grow i in 0_u64..{depth}_u64 {{
    let sibling = boxed_leaf();
    let placeholder = boxed_leaf();
    let taken = replace held.node = move placeholder;
    let taller = boxed_branch(left: move taken, right: move sibling);
    let spent = replace held.node = move taller;
  }}
  return exit_status(code: 0_u8);
}}
"#
    )
    .into_bytes()
}

/// A value whose ownership graph is a chain rather than a cycle: deep in
/// nothing, and reached by the same emitter.
const SHALLOW_OWNERSHIP: &[u8] = br#"command fn main() -> status: own ExitStatus allocates(heap) {
  let slots = buffer_vacant<box<u64>>(2_u64);
  let boxed = box_new(7_u64);
  let wrapped = Some<box<u64>>(value: move boxed);
  let vacant = replace slots[0_u64] = move wrapped;
  return exit_status(code: 0_u8);
}
"#;

/// Every compiler-derived drop in one module, with the drops each one calls.
fn drop_glue_calls(module: &str) -> Vec<(String, Vec<String>)> {
    let mut glue: Vec<(String, Vec<String>)> = Vec::new();
    // A drop is a caller only inside its own definition. The program's own
    // functions call these helpers too, and attributing those calls to
    // whichever definition happened to be last would invent an edge.
    let mut inside = false;
    for line in module.lines() {
        if let Some(rest) = line.strip_prefix("define private void @wf.drop.") {
            let name = rest.split('(').next().expect("a definition names a symbol");
            glue.push((format!("wf.drop.{name}"), Vec::new()));
            inside = true;
            continue;
        }
        if line == "}" {
            inside = false;
            continue;
        }
        if inside
            && let Some((_, callee)) = line.split_once("call void @wf.drop.")
            && let Some((current, calls)) = glue.last_mut()
        {
            let callee = format!("wf.drop.{}", callee.split('(').next().unwrap_or_default());
            assert_ne!(
                &callee, current,
                "a compiler-derived drop calls itself, so its depth is the \
                 value's and no writer can see it: {current}"
            );
            calls.push(callee);
        }
    }
    glue
}

/// No compiler-derived drop can reach itself.
///
/// This is the whole property, and it is structural rather than a depth a case
/// happened to survive: a cycle among these definitions is unbounded recursion
/// on the destruction path, reached after the program has already spent its
/// stack, in code the writer never wrote and cannot instrument. Before this
/// traversal existed, `wf.drop.t0` called `wf.drop.t0` and the corpus had three
/// programs that dragged it.
///
/// The check is over the emitted module rather than over a list of names, so a
/// new nominal shape whose glue closes a cycle fails it without anyone
/// remembering to extend a table.
#[test]
fn no_compiler_derived_drop_reaches_itself() {
    let module = compile(&boxed_spine_source(4));
    let glue = drop_glue_calls(&module);
    assert!(
        glue.iter()
            .any(|(name, _)| name.starts_with("wf.drop.step.")),
        "a program with a recursive nominal must lower its drop to a \
         traversal: {module}"
    );
    let index: std::collections::HashMap<&str, usize> = glue
        .iter()
        .enumerate()
        .map(|(position, (name, _))| (name.as_str(), position))
        .collect();
    // Depth-first over the call graph, refusing a back edge. A drop glue that
    // recursed at all would already have failed inside `drop_glue_calls`; this
    // is what catches a cycle through two or more definitions.
    let mut colour = vec![0_u8; glue.len()];
    let mut path: Vec<(usize, usize)> = Vec::new();
    for root in 0..glue.len() {
        if colour[root] != 0 {
            continue;
        }
        colour[root] = 1;
        path.push((root, 0));
        while let Some((node, cursor)) = path.last_mut() {
            let node = *node;
            let Some(callee) = glue[node].1.get(*cursor) else {
                colour[node] = 2;
                path.pop();
                continue;
            };
            *cursor += 1;
            let Some(target) = index.get(callee.as_str()).copied() else {
                continue;
            };
            assert_ne!(
                colour[target], 1,
                "the compiler-derived drops {} and {callee} reach each other, \
                 which is unbounded recursion on the destruction path",
                glue[node].0
            );
            if colour[target] == 0 {
                colour[target] = 1;
                path.push((target, 0));
            }
        }
    }
}

/// A program whose drops cannot reach themselves emits no traversal at all.
///
/// The traversal is not a new default; it is what the emitter does at exactly
/// the edges whose depth the *value* chooses. Every other drop keeps the
/// straight-line expansion it has always had, whose depth the type bounds. A
/// case that only checked the recursive side would pass against an emitter that
/// put every program on a worklist and charged them all for it.
#[test]
fn an_ownership_chain_keeps_its_straight_line_drop() {
    let module = compile(SHALLOW_OWNERSHIP);
    assert!(
        module.contains("@wf.drop."),
        "this program owns heap storage and must derive drops: {module}"
    );
    assert!(
        !module.contains("@wf.drop.push"),
        "a drop whose depth the type bounds must not pay for a worklist: \
         {module}"
    );
}

/// The traversal reclaims a deep value correctly, end to end.
///
/// The depth here is not the claim — the case above is what says the traversal
/// cannot run out of stack at any depth — this one says the traversal is right:
/// it frees the whole structure, in one pass, and the program ends normally
/// with an empty record channel.
#[test]
fn a_deep_boxed_spine_is_reclaimed_without_a_record() {
    let directory = test_directory();
    let executable = build_executable(&compile(&boxed_spine_source(1_000_000)), &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the deep spine");
    assert_eq!(
        output.status.code(),
        Some(0),
        "destroying a deep value must end the program normally: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "a completed run wrote to the record channel: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}
