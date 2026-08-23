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

/// Deeper than a pool lane's stack and far shallower than the entry's, so a
/// death at this depth is a lane's and a sequential run at it is not close to
/// its own limit.
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

/// A lane that runs out of stack writes the same record.
///
/// Worth its own case because the two thread classes arrive as *different*
/// signals — an entry overflow as SIGSEGV, a lane overflow as SIGBUS — so a
/// disposition that took only SIGSEGV would pass the case above and still miss
/// every worker overflow, which is exactly the class the parallel default
/// introduced.
///
/// The depth is the discriminator: a sequential run at it completes, so a
/// death here is a lane's. Which runs die is a schedule's to choose — whether
/// the deep half of the recursion reaches a lane at all depends on a steal
/// race — so this case samples until it sees the lane path, and holds *every*
/// run to the same standard: a run either completes or writes exactly the
/// record. A bare signal or a partial record fails it on the first run,
/// sampled or not.
#[test]
fn an_exhausted_lane_writes_the_same_resource_record() {
    let directory = test_directory();
    let module = super::emit_with_overlap(&spine_source(LANE_DEPTH));
    let executable = build_executable(&module, &directory);
    let mut exhausted = 0;
    for _ in 0..20 {
        let output = Command::new(&executable)
            .output()
            .expect("run the overlapped recursion");
        match output.status.code() {
            Some(_) => assert!(
                output.stderr.is_empty(),
                "a run that completed wrote to the record channel: {:?}",
                String::from_utf8_lossy(&output.stderr)
            ),
            None => {
                assert_resource_record(&output.stderr, "stack");
                exhausted += 1;
                break;
            }
        }
    }
    assert!(
        exhausted > 0,
        "twenty runs never carried the recursion onto a lane, so this case \
         exercised nothing"
    );
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
