//! The resource-exhaustion floor: what an execution does when it runs out.
//!
//! Running out of stack or heap needs no source defect at all and, before this
//! floor, produced zero bytes and a bare host signal. These cases pin the
//! deliberately deferred resource-availability behavior without turning it
//! into a source proof obligation.
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
//!   carries no `rule_id`, no function, and no node path.
//!
//! The record's bytes are fixed by two independent constraints that happen to
//! agree. A signal handler may only reach async-signal-safe facilities, which
//! admits a constant string written with `write` and essentially nothing else;
//! and [PAR-1] requires observables to be identical under every permitted
//! schedule, which forbids the record from naming a worker, a thread, a depth,
//! or an address. Either one alone would force a fixed constant.

use std::process::Command;

use super::{build_executable, compile, emitted_function, test_directory};

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

fn depth['r](chain: &'r box<Chain>) -> result: own u64 reads(chain) {
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
pub(super) fn spine_source(depth: u64) -> Vec<u8> {
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
/// Exhaustion is external to source proof: no operation in the program has
/// "runs out of stack" in its meaning, and the same source on the same input
/// succeeds or fails depending on the environment. The record therefore names
/// only the unavailable resource.
pub(super) fn assert_resource_record(stderr: &[u8], resource: &str) {
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
             a source location rather than the external resource: {text:?}"
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

/// Links one C body against the shipped floor, exactly as a program links it.
///
/// The bodies below are C because Whitefoot cannot express what they do — a
/// wild pointer, a fault at a chosen address, a signal raised at the process.
/// Everything else is the shipped mechanism: the same translation unit, the
/// same optimization arguments, entered through the same `wf__floor_run`.
fn build_floor_fixture(body: &str, directory: &std::path::Path) -> std::path::PathBuf {
    let source = directory.join("floor_body.c");
    let floor = directory.join("wf_floor.c");
    let executable = directory.join("floor_fixture");
    std::fs::write(&source, body).expect("write the fixture body");
    std::fs::write(&floor, crate::FLOOR_RUNTIME_SOURCE).expect("write the floor runtime");
    let compiled = Command::new("/usr/bin/clang")
        .arg("-pthread")
        .arg("-x")
        .arg("c")
        .arg(&source)
        .arg(&floor)
        .args(super::HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        compiled.status.success(),
        "the floor and a fixture body must link:\n{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    executable
}

#[test]
fn a_fault_that_is_not_exhaustion_keeps_its_own_disposition() {
    let directory = test_directory();
    let executable = build_floor_fixture(WILD_FAULT_BODY, &directory);
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
    assert_eq!(
        signal_of(&output),
        Some(libc_sigsegv()),
        "a wild fault must keep SIGSEGV rather than becoming the floor's \
         abort: {:?}",
        output.status
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The signal that ended a process, or `None` if it exited normally.
fn signal_of(output: &std::process::Output) -> Option<i32> {
    std::os::unix::process::ExitStatusExt::signal(&output.status)
}

const fn libc_sigsegv() -> i32 {
    11
}

const fn libc_sigabrt() -> i32 {
    6
}

/// A body that faults a chosen distance below its own thread's stack.
///
/// The distance is the whole experiment, so it is read from the environment
/// rather than compiled in: one binary, one link, one row per offset.
///
/// The thread is the fixture's own, not the floor's entry thread, and its
/// stack sits at the top of one reservation whose lower 16 MiB the thread
/// unmaps before it writes. That is what makes every row's premise true by
/// construction rather than by luck: a write below the stack is a pointer
/// into nothing because the fixture made it so, not because nothing happened
/// to be there. The entry thread cannot promise that. The floor maps its
/// 64 KiB alternate signal stack after the entry stack, the kernel's top-down
/// placement drops it into the first gap below the stack block, and
/// `/proc/self/maps` shows it there: a `rw-p` mapping directly under the
/// `---p` guard. On a host whose guard is one page, a write four pages below
/// the stack lands in that mapping and completes, which is what the Linux
/// gate did once in ten runs — exit 0, no signal, no record, and nothing for
/// the floor to classify.
///
/// The pad is unmapped rather than left `PROT_NONE` because the rows assert
/// the host's own signal for a pointer into nothing, and Darwin reports a
/// protected page as `SIGBUS` where an unmapped one is `SIGSEGV` on both
/// hosts. It is unmapped only after the thread has attached to the floor and
/// read its bounds, because both of those map memory and the same top-down
/// search would put either into a freshly freed hole under the stack. The
/// thread attaches exactly as a pool lane does, so the band under test is the
/// one every lane runs under.
const OFFSET_FAULT_BODY: &str = r#"/* `pthread_getattr_np` is a GNU extension, so the Linux arm below needs the
   feature-test macro the floor runtime already sets for the same call. */
#define _GNU_SOURCE
#include <pthread.h>
#include <stdlib.h>
#include <sys/mman.h>

extern int wf__floor_run(int argc, char **argv);
extern void wf__floor_attach_thread(void);

/* Below the stack: the largest distance any row asks for, plus slack over the
   stride the in-band rows use, so every address a row names is inside the
   hole this reservation leaves. */
#define PAD_BYTES ((size_t)16 * 1024 * 1024 + (size_t)64 * 1024)
#define STACK_BYTES ((size_t)1024 * 1024)

static char *reservation;

static void *fault_below_own_stack(void *opaque) {
    char *low = NULL;
    unsigned long below;
    (void)opaque;
    /* Both of these map memory, so both happen while the pad is still held. */
    wf__floor_attach_thread();
#if defined(__APPLE__)
    low = (char *)pthread_get_stackaddr_np(pthread_self())
          - pthread_get_stacksize_np(pthread_self());
#else
    {
        pthread_attr_t attributes;
        void *base = NULL;
        size_t size = 0;
        pthread_getattr_np(pthread_self(), &attributes);
        pthread_attr_getstack(&attributes, &base, &size);
        low = (char *)base;
        pthread_attr_destroy(&attributes);
    }
#endif
    below = strtoul(getenv("WF_FAULT_BELOW"), NULL, 10);
    /* From here to the write, nothing in this process maps anything: the
       entry thread is parked in pthread_join and this thread only faults. */
    if (munmap(reservation, PAD_BYTES) != 0) {
        return NULL;
    }
    *(volatile int *)(low - below) = 1;
    return NULL;
}

int wf__main_body(int argc, char **argv) {
    pthread_attr_t attributes;
    pthread_t thread;
    (void)argc;
    (void)argv;
    reservation = mmap(NULL, PAD_BYTES + STACK_BYTES, PROT_NONE,
                       MAP_PRIVATE | MAP_ANON, -1, 0);
    if (reservation == MAP_FAILED) {
        return 2;
    }
    if (mprotect(reservation + PAD_BYTES, STACK_BYTES, PROT_READ | PROT_WRITE) != 0) {
        return 3;
    }
    if (pthread_attr_init(&attributes) != 0
        || pthread_attr_setstack(&attributes, reservation + PAD_BYTES, STACK_BYTES) != 0
        || pthread_create(&thread, &attributes, fault_below_own_stack, NULL) != 0) {
        return 4;
    }
    pthread_join(thread, NULL);
    return 0;
}

int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;

/// The band that separates "this thread ran out" from "this pointer is wild"
/// is the probe's geometry, and nothing wider.
///
/// The classification has exactly one input — the faulting address — so its
/// only checkable property is where the boundary sits. The previous band was
/// 1 MiB, which reported every corruption fault within a megabyte below any
/// thread's stack as an exhausted stack: exit 134 and a `{"resource":"stack"}`
/// record in place of exit 139 and a core, for a defect that has nothing to do
/// with depth. That is the misdirection [`wf_floor.c`]'s own header says a
/// diagnostic must not commit, and no case sampled the band, so the boundary
/// could sit anywhere.
///
/// One page below the stack is where a probed frame's page walk lands, so it
/// must be reported. Four pages below it and beyond is past anything a descent
/// can reach, so it must not be — and must keep SIGSEGV rather than the
/// floor's abort, which is the difference between a core dump of the
/// corruption and a core dump of `abort`.
#[test]
fn only_a_fault_within_the_probe_stride_is_read_as_an_exhausted_stack() {
    let page = page_size();
    let directory = test_directory();
    let executable = build_floor_fixture(OFFSET_FAULT_BODY, &directory);
    // Inside the stride, so a descent really can land here.
    for below in [page / 2, page] {
        let output = Command::new(&executable)
            .env("WF_FAULT_BELOW", below.to_string())
            .output()
            .expect("run the offset fault");
        assert_resource_record(&output.stderr, "stack");
        assert_eq!(
            signal_of(&output),
            Some(libc_sigabrt()),
            "a reported exhaustion ends in the floor's abort, {below} bytes \
             below the stack: {:?}",
            output.status
        );
    }
    // Past it, so nothing that walks its pages on the way down can reach here.
    for below in [4 * page, 64 * 1024, 16 * 1024 * 1024] {
        let output = Command::new(&executable)
            .env("WF_FAULT_BELOW", below.to_string())
            .output()
            .expect("run the offset fault");
        assert!(
            output.stderr.is_empty(),
            "the floor claimed a wild fault {below} bytes below the stack: \
             {:?}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            signal_of(&output),
            Some(libc_sigsegv()),
            "a wild fault {below} bytes below the stack must keep its own \
             signal: {:?}",
            output.status
        );
    }
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A body that takes a signal from outside and then descends deep.
///
/// `kill` at the process is the cheapest thing that reaches the handler with
/// no faulting instruction behind it, which is the whole class: a supervisor,
/// a test harness, a job controller. SIGBUS is the one the floor's own entry
/// thread uses for exhaustion on this host, so it is the signal whose loss
/// costs the most.
const EXTERNAL_SIGNAL_BODY: &str = r#"#include <signal.h>
#include <stdio.h>
#include <unistd.h>

extern int wf__floor_run(int argc, char **argv);

static long descend(long n) {
    volatile char frame[512];
    frame[0] = (char)n;
    if (n == 0) {
        return frame[0];
    }
    return descend(n - 1) + 1;
}

int wf__main_body(int argc, char **argv) {
    (void)argc;
    (void)argv;
    kill(getpid(), SIGBUS);
    printf("SURVIVED ");
    fflush(stdout);
    descend(1000);
    printf("COMPLETED\n");
    return 0;
}

int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;

/// A signal the floor does not own kills the process instead of disarming it.
///
/// The handler's non-guard path restores `SIG_DFL`, and `sigaction` is
/// per-signal and process-wide while the classification above it is
/// per-thread. So the restore is only sound if the process cannot outlive it:
/// otherwise one externally delivered signal — which arrives here with a null
/// `si_addr`, indistinguishable from a null dereference — is swallowed, and
/// every later overflow on any thread arrives as a bare host signal with zero
/// bytes, silently reverting the floor for the rest of the run.
///
/// That was the behaviour: this fixture printed `SURVIVED COMPLETED` and
/// exited 0. Re-raising after the restore is what makes "put it back exactly
/// as it was" true for a signal with no instruction to re-execute.
#[test]
fn an_externally_delivered_signal_does_not_disarm_the_floor() {
    let directory = test_directory();
    let executable = build_floor_fixture(EXTERNAL_SIGNAL_BODY, &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the externally signalled program");
    assert_eq!(
        output.status.code(),
        None,
        "a signal the floor does not own must still end the process: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("COMPLETED"),
        "the process ran on past a signal that should have ended it, with \
         the floor no longer installed: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        output.stderr.is_empty(),
        "the floor must add nothing to a signal that is not its own: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A body that claims the record latch and then runs out of stack.
///
/// The store stands in for the emitted module's writer having already won:
/// what matters is that the floor's handler consults the same word, not how
/// the word was taken. The alarm is what keeps the case terminating — a loser
/// parks until the winner's abort takes the process down, and here there is no
/// winner to do it, so the test's own clock ends the run.
const PREACQUIRED_LATCH_BODY: &str = r#"#include <signal.h>
#include <unistd.h>

extern volatile int *wf__floor_record_latch(void);
extern int wf__floor_run(int argc, char **argv);

static long descend(long n) {
    volatile char frame[512];
    frame[0] = (char)n;
    if (n == 0) {
        return frame[0];
    }
    return descend(n - 1) + 1;
}

int wf__main_body(int argc, char **argv) {
    (void)argc;
    (void)argv;
    *wf__floor_record_latch() = 1;
    alarm(2);
    descend(400000000);
    return 0;
}

int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;

/// The floor and the emitted module take the same latch, so no execution
/// writes two records.
///
/// This is the mechanism behind "exactly one record", and it needs saying with
/// a case because the two writers live in different languages. The floor's
/// signal handler writes the stack record; the module writes the heap record.
/// Separate latches would serialize each writer only against itself, allowing
/// two threads exhausting different resources to interleave records on one
/// channel.
///
/// Here the latch is already taken when the stack runs out. Shared, the
/// handler finds it taken and writes nothing. Separate, it writes the stack
/// record and aborts, which is what this fixture did before the two were one.
#[test]
fn the_floor_and_the_module_share_one_record_latch() {
    let directory = test_directory();
    let executable = build_floor_fixture(PREACQUIRED_LATCH_BODY, &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the pre-acquired latch fixture");
    assert!(
        output.stderr.is_empty(),
        "a record was already claimed, so the floor must write none: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(
        signal_of(&output),
        Some(libc_sigabrt()),
        "the floor took a latch of its own and aborted on it: {:?}",
        output.status
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A `--par` module that can write a heap-resource record.
///
/// Handing a call out makes concurrent allocation refusal possible, so this
/// module must use the shared first-record latch.
const HEAP_RECORD_LANE: &[u8] = br#"fn leafwork(v: own u64) -> result: own u64 pure {
  return v *wrap 3_u64;
}

fn build(n: own u64) -> result: own u64 allocates(heap) {
  let b = buffer_new(4000000000000000000_u64, 7_u8);
  let e = b[0_u64];
  return 0_u64 +wrap n;
}

fn both(n: own u64) -> result: own u64 allocates(heap) {
  let a = build(n: n);
  let c = leafwork(v: n);
  return a +wrap c;
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let r = both(n: 5_u64);
  let ok = igt(r, 0_u64);
  if ok {
    return exit_status(code: 0_u8);
  }
  return exit_status(code: 1_u8);
}
"#;

#[test]
fn a_module_that_writes_a_resource_record_and_hands_a_call_out_is_latched() {
    let module = super::emit_with_overlap(HEAP_RECORD_LANE);
    assert!(
        module.contains("@wf__par_thunk_"),
        "the fixture must hand a call out, or the latch is not the question: \
         {module}"
    );
    assert!(module.contains("@.wf_resource_record.latch"));
    assert!(
        module.contains("call void @wf_resource_abort()"),
        "the fixture must reach a resource record: {module}"
    );
    assert!(
        module.contains("%latch = call ptr @wf__floor_record_latch()"),
        "a module that writes a record on more than one thread must take the \
         shared latch: {module}"
    );
}

fn page_size() -> usize {
    let output = Command::new("/usr/bin/getconf")
        .arg("PAGESIZE")
        .output()
        .expect("read the host page size");
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("the host reports a page size")
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

command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args), allocates(heap) {
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
/// indistinguishable from an internal allocator abort, with nothing to tell a
/// reader which resource was unavailable.
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

/// Filled and vacant buffers whose proved byte ceilings fit the selected
/// target carry no runtime target-domain path. The allocator can still return
/// null, so each operation keeps its ordinary heap-resource failure edge.
#[test]
fn target_qualified_buffers_keep_only_the_heap_refusal_path() {
    let module = compile(ALL_HEAP_FORMS);
    for absent in [
        "buffer.fill.target.",
        "buffer.vacant.target.",
        "@wf_target_domain_abort",
        "@.wf_resource.target_domain",
    ] {
        assert!(
            !module.contains(absent),
            "a target-qualified buffer must not emit {absent}:\n{module}"
        );
    }

    let shapes = emitted_function(&module, "shapes");
    let lines: Vec<&str> = shapes.lines().collect();
    for operation in ["buffer.fill", "buffer.vacant"] {
        let allocation = lines
            .iter()
            .position(|line| line.starts_with(&format!("{operation}.allocate.")))
            .expect("the fixture must reach the buffer allocation block");
        let refusal = lines
            .iter()
            .position(|line| line.starts_with(&format!("{operation}.oom.")))
            .expect("the allocator's null result must retain a refusal block");
        assert!(allocation < refusal);
        let allocation_path = lines[allocation + 1..refusal].join("\n");
        assert!(allocation_path.contains("call ptr @malloc"));
        assert!(allocation_path.contains("icmp ne ptr"));
        assert_eq!(
            lines.get(refusal + 1).copied(),
            Some("  call void @wf_resource_abort()")
        );
    }
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

command fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
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
/// So the case runs the ablation rather than describing it. It strips the
/// attribute group from this one definition, in this one module, and requires
/// the two runs to differ: probed, the descent walks its pages, faults inside
/// the probe stride, and is reported; ablated, it moves the stack pointer
/// 291,600 bytes in one step, faults far outside anything a descent can reach,
/// and the floor correctly refuses to call that exhaustion.
///
/// Both halves are needed and neither alone is the property. Checking only the
/// probed run passes against an emitter that stopped emitting the attribute,
/// as long as something else still reported the death — which is exactly what
/// happened while the discrimination band was a megabyte wide: the ablated
/// skip landed inside the band and was reported too, and no case could tell
/// the difference.
#[test]
fn a_frame_larger_than_the_guard_region_is_still_reported() {
    let module = compile(LARGE_FRAME_SPINE);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
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

    let ablated = ablate_probe(&module, "@wf_spine(");
    assert_eq!(
        module.matches(" #0 {").count() - ablated.matches(" #0 {").count(),
        1,
        "the ablation must remove the group from exactly one definition"
    );
    let elsewhere = test_directory();
    let unprobed = build_executable(&ablated, &elsewhere);
    let output = Command::new(&unprobed)
        .output()
        .expect("run the unprobed large-frame recursion");
    assert!(
        output.stderr.is_empty(),
        "an unprobed frame steps over the guard region, so the fault it \
         eventually takes is not this thread running out and must not be \
         reported as one: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
    std::fs::remove_dir_all(&elsewhere).expect("remove the second test directory");
}

/// The same module with the probe attribute group taken off the one definition
/// whose `define` line contains `signature`.
fn ablate_probe(module: &str, signature: &str) -> String {
    module
        .lines()
        .map(|line| {
            if line.starts_with("define ") && line.contains(signature) {
                line.strip_suffix(" #0 {")
                    .map(|head| format!("{head} {{"))
                    .unwrap_or_else(|| line.to_owned())
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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

/// A cleanup cycle that closes through a `buffer` instead of through a `box`.
///
/// `box` supplies the indirection the target layout needs while the buffer
/// stays inside the cycle: `Chain` -> `box<buffer<Option<Chain>>>` ->
/// `buffer<Option<Chain>>` -> `Option<Chain>` -> `Chain`. Nothing about the
/// program says "recursive"; as with the boxed spine, the only recursion is
/// the one the compiler would generate to destroy the value.
///
/// The shape matters because the two indirections need different traversal
/// arms. A `box` names one content, so one worklist entry carries the whole
/// edge. A buffer names many elements whose reclamation order [STOR-3] fixes,
/// so it takes one entry per element plus one for the block.
fn buffer_chain_source(depth: u64) -> Vec<u8> {
    format!(
        r#"enum Chain {{
  Nil();
  Cons(kids: box<buffer<Option<Chain>>>);
}}

fn nest(inner: own Chain) -> result: own Chain allocates(heap) {{
  let slots = buffer_vacant<Chain>(1_u64);
  let filled = Some<Chain>(value: move inner);
  let vacant = replace slots[0_u64] = move filled;
  match vacant {{
    None() => {{
    }}
    Some(value: stray) => {{
    }}
  }}
  let held = box_new(move slots);
  return Cons(kids: move held);
}}

command fn main() -> status: own ExitStatus allocates(heap) {{
  let holder = buffer_vacant<Chain>(1_u64);
  let seed = Nil();
  let seeded = Some<Chain>(value: move seed);
  let empty = replace holder[0_u64] = move seeded;
  match empty {{
    None() => {{
    }}
    Some(value: stray) => {{
    }}
  }}
  for @build i in 0_u64..{depth}_u64 {{
    let taken = replace holder[0_u64] = None<Chain>();
    match taken {{
      None() => {{
        return exit_status(code: 1_u8);
      }}
      Some(value: inner) => {{
        let grown = nest(inner: move inner);
        let refilled = Some<Chain>(value: move grown);
        let hole = replace holder[0_u64] = move refilled;
        match hole {{
          None() => {{
          }}
          Some(value: leftover) => {{
          }}
        }}
      }}
    }}
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
    // Both owning indirections a cleanup cycle can close through, because the
    // traversal has a separate arm for each and a cycle in either one is the
    // same defect.
    assert_no_drop_glue_cycle(&compile(&boxed_spine_source(4)));
    assert_no_drop_glue_cycle(&compile(&buffer_chain_source(4)));
}

fn assert_no_drop_glue_cycle(module: &str) {
    let glue = drop_glue_calls(module);
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

/// Every address formed by the recursive-drop worklist is dominated by a
/// finite selected-target capacity check. The three `nuw` operations are
/// justified by that check: doubling stays below the maximum entry count,
/// byte scaling stays below the allocator/address ceiling, and incrementing a
/// non-full count stays within the allocated capacity.
#[test]
fn recursive_drop_worklist_growth_proves_each_address_domain() {
    let module = compile(&boxed_spine_source(1));
    let push = definition_body(&module, "define private void @wf.drop.push");
    assert!(
        push.contains("%count.in.range = icmp ule i64 %count, %capacity")
            && push.contains("%maximum.entries = udiv i64 ")
            && push.contains("%growth.fits = select i1 %fresh")
            && push.contains("br i1 %growth.fits, label %grow, label %exhausted"),
        "worklist growth must establish count and selected-target capacity before addressing: \
         {push}"
    );
    assert!(
        push.contains("%doubled = shl nuw i64 %capacity, 1")
            && push.contains("%bytes = mul nuw i64 %wanted, %entry.bytes")
            && push.contains("%after = add nuw i64 %count, 1"),
        "only arithmetic dominated by the finite range checks may carry no-wrap facts: {push}"
    );
    assert!(
        !push.contains("%doubled = shl i64")
            && !push.contains("%bytes = mul i64")
            && !push.contains("%after = add i64"),
        "the worklist must not retain unchecked growth arithmetic: {push}"
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

// ------------------------------------ the cycle that closes through a buffer

/// The shape a reviewer reached for when the traversal shipped with only its
/// `box` arm, written the way they wrote it.
///
/// It is here as a program rather than as a type, because the claim it settles
/// is about acceptance: this compiled and ran before the traversal existed, so
/// nothing the traversal does may take it away. When only the `box` arm was
/// implemented the emitter refused it with a bare compiler-invariant failure —
/// no rule, no coordinate, and no statement of what was unsupported.
const BUFFER_CYCLE: &[u8] = br#"enum Chain {
  Nil();
  Cons(kids: box<buffer<Option<Chain>>>);
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let inner = buffer_vacant<Chain>(2_u64);
  let b = box_new(move inner);
  let node = Cons(kids: move b);
  return exit_status(code: 0_u8);
}
"#;

/// A cleanup cycle through a buffer is a program the compiler accepts.
#[test]
fn a_cleanup_cycle_through_a_buffer_is_accepted_and_runs() {
    let directory = test_directory();
    let executable = build_executable(&compile(BUFFER_CYCLE), &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the buffer cycle");
    assert_eq!(
        output.status.code(),
        Some(0),
        "this program compiled and ran before the traversal existed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "a completed run wrote to the record channel: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The buffer arm carries depth the same way the `box` arm does.
///
/// Accepting the program is not the property; reclaiming it without descending
/// is. This depth cannot fit a machine stack at the frame the recursive glue
/// used to need, and the compiler that generated that glue dies here with a
/// bare signal.
#[test]
fn a_deep_cleanup_cycle_through_a_buffer_is_reclaimed_without_a_record() {
    let directory = test_directory();
    let executable = build_executable(&compile(&buffer_chain_source(1_000_000)), &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the deep buffer chain");
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

/// A buffer in a cleanup cycle whose elements each own further storage.
const WIDE_BUFFER_CYCLE: &[u8] = br#"enum Chain {
  Nil();
  Cons(kids: box<buffer<Option<Chain>>>);
}

fn leafy() -> result: own Chain allocates(heap) {
  let slots = buffer_vacant<Chain>(1_u64);
  let held = box_new(move slots);
  return Cons(kids: move held);
}

command fn main() -> status: own ExitStatus allocates(heap) {
  let slots = buffer_vacant<Chain>(4_u64);
  let child0 = leafy();
  let first = Some<Chain>(value: move child0);
  let hole0 = replace slots[0_u64] = move first;
  match hole0 {
    None() => {
    }
    Some(value: stray0) => {
    }
  }
  let child1 = leafy();
  let second = Some<Chain>(value: move child1);
  let hole1 = replace slots[1_u64] = move second;
  match hole1 {
    None() => {
    }
    Some(value: stray1) => {
    }
  }
  let child2 = leafy();
  let third = Some<Chain>(value: move child2);
  let hole2 = replace slots[2_u64] = move third;
  match hole2 {
    None() => {
    }
    Some(value: stray2) => {
    }
  }
  let child3 = leafy();
  let fourth = Some<Chain>(value: move child3);
  let hole3 = replace slots[3_u64] = move fourth;
  match hole3 {
    None() => {
    }
    Some(value: stray3) => {
    }
  }
  let held = box_new(move slots);
  let root = Cons(kids: move held);
  return exit_status(code: 0_u8);
}
"#;

/// The body of one definition, from its `define` line to its closing brace.
fn definition_body<'a>(module: &'a str, signature: &str) -> &'a str {
    let start = module
        .find(signature)
        .unwrap_or_else(|| panic!("the module defines {signature}: {module}"));
    let body = &module[start..];
    let end = body.find("\n}\n").expect("a definition closes");
    &body[..end]
}

/// [STOR-3] fixes a buffer drop as each element's drop in ascending index
/// order followed by that same one heap free, and the traversal has to produce
/// that order out of a last-in first-out worklist.
///
/// It does it by pushing in the reverse: the block's own entry first, then the
/// elements from the last index down. Walking the indices upward instead, or
/// pushing the block last, both emit a traversal that looks right and reclaims
/// in the wrong order — and pushing the block last would additionally have the
/// elements read out of storage the traversal had already released. Nothing
/// downstream can see the difference, because [STOR-3] gives memory
/// reclamation the empty effect row, so the order is pinned where it is
/// chosen.
#[test]
fn a_buffer_in_a_cleanup_cycle_is_walked_in_the_order_the_rule_fixes() {
    let module = compile(&buffer_chain_source(4));
    // The one definition that takes a buffer descriptor and the worklist: the
    // per-node drop of the buffer inside the cycle.
    let buffer_step = definition_body(&module, "({ ptr, i64 } %value, ptr %work)");
    let block = buffer_step
        .find(", ptr %pointer)")
        .expect("the buffer step pushes the block's own entry");
    let element = buffer_step
        .find(", ptr %slot)")
        .expect("the buffer step pushes one entry per element");
    assert!(
        block < element,
        "the block's entry must be pushed before any element's, so the \
         last-in first-out traversal takes it last: {buffer_step}"
    );
    assert!(
        buffer_step.contains("%index = phi i64 [ %length, %entry ], [ %next, %body ]")
            && buffer_step.contains("%next = sub i64 %index, 1"),
        "the element entries must be pushed from the last index down, so the \
         traversal takes index 0 first: {buffer_step}"
    );
}

/// The block outlives every element that lives in it.
///
/// This is the half of [STOR-3]'s order that a running program can be made to
/// notice. The traversal releases a `box` block as it takes that block's entry,
/// which is what keeps the pending list off the depth; a buffer cannot do that,
/// because its elements are still in the block. Under a host allocator that
/// scribbles freed storage, taking that shortcut turns every element load into
/// a scribbled tag and the enum's own invalid-tag abort fires.
#[test]
fn a_buffer_block_outlives_the_elements_the_traversal_takes_from_it() {
    let directory = test_directory();
    let executable = build_executable(&compile(WIDE_BUFFER_CYCLE), &directory);
    let output = Command::new(&executable)
        .env("MallocScribble", "1")
        .env("MallocPreScribble", "1")
        .output()
        .expect("run the wide buffer cycle");
    assert_eq!(
        output.status.code(),
        Some(0),
        "an element was read out of a block the traversal had released: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}
