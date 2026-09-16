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
//! Compiler-generated probes, allocation edges and cleanup stay here. Pure C
//! floor setup, signal classification, stack provisioning and record-latch
//! observations live in `floor_probe.c`, under the common native test target.
//! Record bytes are a runtime implementation contract; SCOPE-3 keeps resource
//! availability outside the source outcome model.

use std::process::Command;

use super::{build_linked_executable, compile, emitted_function, test_directory};

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

fn depth(chain: &box<Chain>) -> result: own u64 reads(chain) {
  match deref(deref(chain)) {
    End() => {
      return 0_u64;
    }
    More(next: inner) => {
      let below = depth(chain: inner);
      return below +wrap 1_u64;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  let end = End();
  let bottom = box_new(move end);
  let one = More(next: move bottom);
  let boxed = box_new(move one);
  region {
    let measured = depth(chain: &boxed);
    if measured == 1_u64 {
    } else {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#;

fn mixed_module(parallel: bool) -> String {
    use std::sync::OnceLock;
    static PLAIN: OnceLock<String> = OnceLock::new();
    static PARALLEL: OnceLock<String> = OnceLock::new();
    let cell = if parallel { &PARALLEL } else { &PLAIN };
    cell.get_or_init(|| {
        if parallel {
            super::emit_with_overlap(MIXED_DEFINITIONS)
        } else {
            compile(MIXED_DEFINITIONS)
        }
    })
    .clone()
}

fn heap_module() -> String {
    static MODULE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    MODULE.get_or_init(|| compile(ALL_HEAP_FORMS)).clone()
}

fn boxed_module() -> String {
    static MODULE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    MODULE
        .get_or_init(|| compile(&boxed_spine_source(4)))
        .clone()
}

fn buffer_module() -> String {
    static MODULE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    MODULE
        .get_or_init(|| compile(&buffer_chain_source(4)))
        .clone()
}

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
    assert_stack_probes(&mixed_module(false));
    assert_stack_probes(&mixed_module(true));
}

/// Called also by retained loop and pair fixtures, which actually emit chunks,
/// thunks and sequential clones in addition to the mixed fixture's drop glue.
pub(super) fn assert_stack_probes(module: &str) {
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

/// Compare the small function's actual assembly with and without its probe
/// attribute. This observes target code, including inline Linux probes; a
/// whole-executable symbol-name search could not establish ordinary cost.
#[test]
fn an_ordinary_frame_has_the_same_machine_body_without_probe_instrumentation() {
    let directory = test_directory();
    let module = mixed_module(false);
    let (_, probed) = super::stack_ledger::machine_report(&module, &directory);
    let (_, ablated) =
        super::stack_ledger::machine_report(&ablate_probe(&module, "@wf_depth("), &directory);
    let body = |assembly: &str| {
        let label = if cfg!(target_os = "macos") {
            "_wf_depth:"
        } else {
            "wf_depth:"
        };
        let start = assembly
            .find(label)
            .expect("the small function survives native codegen");
        let tail = &assembly[start..];
        let end = tail
            .find(".cfi_endproc")
            .expect("the machine function ends its unwind region");
        tail[..end].to_owned()
    };
    assert_eq!(
        body(&probed),
        body(&ablated),
        "a small frame must pay no added probe instructions"
    );
    std::fs::remove_dir_all(directory).expect("remove assembly comparison");
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
  let done = depth == 0_u64;
  if done {{
    return v;
  }}
  let next = depth -wrap 1_u64;
  let a = spine(depth: next, v: v);
  let b = leafval(v: v);
  return fadd.strict(a, b);
}}

fn main() -> status: own ExitStatus pure {{
  let total = spine(depth: {depth}_u64, v: 1.0009765625_f64);
  let bits = reinterpret::<f64, u64>(total);
  let low = iand(bits, 1_u64);
  match cvt::<u64, u8>(low) {{
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
}

/// The signal that ended a process, or `None` if it exited normally.
fn signal_of(output: &std::process::Output) -> Option<i32> {
    std::os::unix::process::ExitStatusExt::signal(&output.status)
}

#[cfg(not(target_os = "macos"))]
const fn libc_sigsegv() -> i32 {
    11
}

const fn libc_sigabrt() -> i32 {
    6
}

/// A `--par` module that can write a heap-resource record.
///
/// Handing a call out makes concurrent allocation refusal possible, so this
/// module must use the shared first-record latch.
const HEAP_RECORD_LANE: &[u8] = br#"fn leafwork(v: own u64) -> result: own u64 pure {
  return v *wrap 3_u64;
}

fn build(n: own u64) -> result: own u64 pure {
  let b = buffer_new(4000000000000000000_u64, 7_u8);
  let e = b[0_u64];
  return 0_u64 +wrap n;
}

fn both(n: own u64) -> result: own u64 pure {
  let a = build(n: n);
  let c = leafwork(v: n);
  return a +wrap c;
}

fn main() -> status: own ExitStatus pure {
  let r = both(n: 5_u64);
  let ok = r > 0_u64;
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

/// One program reaching every allocation form the emitter lowers: a filled
/// buffer, a vacant one, a heap box, and an arena node.
///
/// The lengths are constants so the fit obligation discharges statically and
/// the fixture stays about the refusal edges rather than about proving a
/// dynamic length fits.
const ALL_HEAP_FORMS: &[u8] = br#"fn shapes(n: own u64) -> result: own u64 pure {
  let filled = buffer_new(4_u64, 5_u64);
  let vacant = buffer_vacant::<u32>(4_u64);
  let boxed = box_new(7_u64);
  let held = deref(boxed);
  let filled_len = len_of(filled);
  let vacant_len = len_of(vacant);
  let total = held +wrap filled_len;
  set total = total +wrap vacant_len;
  region 'a {
    let kept = arena_new::<'a, u64>(3_u64);
    let seen = deref(kept);
    set total = total +wrap seen;
  }
  return total;
}

fn main() -> status: own ExitStatus pure {
  let total = shapes(n: 4_u64);
  match cvt::<u64, u8>(total) {
    Ok(value: byte) => {
      return exit_status(code: byte);
    }
    Err(error: wide) => {
      return exit_status(code: 9_u8);
    }
  }
}
"#;

/// Generated allocation calls alone are interposed; runtime startup allocation
/// remains real. One small four-form image covers success and every refusal.
#[test]
fn each_generated_allocation_form_reaches_its_refusal_record() {
    let directory = test_directory();
    let observed = heap_module()
        .replace("@malloc(", "@wf_test_allocate(")
        .replace("@free(", "@wf_test_release(");
    let host = format!(
        "{}\n__attribute__((constructor)) static void unbuffer(void) {{ setvbuf(stdout, NULL, _IONBF, 0); }}\n",
        super::owned_places::allocation_observer_by_process(4)
    );
    let executable = build_linked_executable(&observed, Some(&host), &[], &directory);
    for refused in 0..=4 {
        let output = Command::new(&executable)
            .env("WF_TEST_REFUSE_ALLOCATION", refused.to_string())
            .output()
            .expect("run scoped allocation refusal");
        let trace = std::str::from_utf8(&output.stdout).expect("ASCII allocation trace");
        if refused == 0 {
            assert_eq!(output.status.code(), Some(18), "{output:?}");
            assert!(output.stderr.is_empty());
            assert!(trace.starts_with("A1;A2;A3;A4;"), "{trace}");
            let mut freed = trace
                .split(';')
                .filter_map(|event| event.strip_prefix('F'))
                .collect::<Vec<_>>();
            freed.sort_unstable();
            assert_eq!(freed, ["1", "2", "3", "4"], "{trace}");
        } else {
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(
                output.status.signal(),
                Some(libc_sigabrt()),
                "refused={refused}: {output:?}"
            );
            let expected = (1..refused).map(|id| format!("A{id};")).collect::<String>()
                + &format!("X{refused};");
            assert_eq!(trace, expected);
            assert_resource_record(&output.stderr, "heap");
        }
    }
    std::fs::remove_dir_all(directory).expect("remove allocation refusal image");
}

/// Filled and vacant buffers whose proved byte ceilings fit the selected
/// target carry no runtime target-domain path. The allocator can still return
/// null, so each operation keeps its ordinary heap-resource failure edge.
#[test]
fn target_qualified_buffers_keep_only_the_heap_refusal_path() {
    let module = heap_module();
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
    let module = heap_module();
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

/// A recursion whose every activation carries an array far larger than a guard
/// page, handed to a retained reader so the frame cannot be shrunk away. A
/// same-index local store/load can forward its scalar value and erase the
/// array entirely; it does not establish a large physical frame. The
/// controlled harness below enters only its base case; the
/// recursive edge keeps the generated function representative of an ordinary
/// source recursion without making the fault depend on a sequence of frames.
const LARGE_FRAME_SPINE: &[u8] =
    br#"fn read_pad(values: &array<u64, 7168>, index: own u64) -> result: own u64 reads(values) contract {
  requires index < 7168_u64;
} {
  return deref(values)[index];
}

fn spine(depth: own u64, v: own u64, i: own u8) -> result: own u64 pure {
  let pad = array_new::<u64, 7168>(v);
  let wide = cvt::<u8, u64>(i);
  set pad[wide] = depth;
  let done = depth == 0_u64;
  if done {
    region {
      return read_pad(values: &pad, index: wide);
    }
  }
  let next = depth -wrap 1_u64;
  let a = spine(depth: next, v: v, i: i);
  region {
    let b = read_pad(values: &pad, index: wide);
    return a +wrap b;
  }
}

fn main(inputs: own Inputs) -> status: own ExitStatus pure {
  let Inputs(args: args, cwd: unused_cwd, stdout: unused_stdout, stderr: unused_stderr, handles: entry_factory, stdin: unused_stdin) = move inputs;
  region {
    close_directory(factory: &uniq entry_factory, directory: move unused_cwd);
  }
  let count = 0_u64;
  region {
    set count = args_count(args: &args);
  }
  match cvt::<u64, u8>(count) {
    Ok(value: idx) => {
      let depth = count *wrap 20000_u64;
      let r = spine(depth: depth, v: 3_u64, i: idx);
      let ok = r > 0_u64;
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

/// Runs the generated large-frame function once on a stack whose surrounding
/// address space belongs to this fixture.
///
/// The stack is smaller than the array payload in one source activation. The
/// reservation beneath it is much larger than that payload and remains
/// `PROT_NONE`, so neither the floor's alternate stack nor another incidental
/// mapping can absorb the first access after an unprobed frame steps over the
/// stack. The thread attaches before making the call, exactly as a runtime lane
/// does, and its signal is classified against these known bounds.
const LARGE_FRAME_BODY: &str = r#"#define _GNU_SOURCE
#include <pthread.h>
#include <stdint.h>
#include <sys/mman.h>

extern int wf__floor_run(int argc, char **argv);
extern void wf__floor_attach_thread(void);
extern uint64_t wf_spine(uint64_t depth, uint64_t value, uint8_t index);

#define PAD_BYTES ((size_t)16 * 1024 * 1024)
#define STACK_BYTES ((size_t)32 * 1024)

static char *reservation;

static void *call_large_frame(void *opaque) {
    (void)opaque;
    wf__floor_attach_thread();
    (void)wf_spine(0, 3, 0);
    return NULL;
}

int wf__main_body(int argc, char **argv) {
    pthread_attr_t attributes;
    pthread_t thread;
    void *returned = NULL;
    (void)argc;
    (void)argv;
    reservation = mmap(NULL, PAD_BYTES + STACK_BYTES, PROT_NONE,
                       MAP_PRIVATE | MAP_ANON, -1, 0);
    if (reservation == MAP_FAILED) {
        return 2;
    }
    if (mprotect(reservation + PAD_BYTES, STACK_BYTES,
                 PROT_READ | PROT_WRITE) != 0) {
        return 3;
    }
    if (pthread_attr_init(&attributes) != 0
        || pthread_attr_setstack(&attributes, reservation + PAD_BYTES,
                                 STACK_BYTES) != 0
        || pthread_create(&thread, &attributes, call_large_frame, NULL) != 0) {
        return 4;
    }
    pthread_attr_destroy(&attributes);
    if (pthread_join(thread, &returned) != 0) {
        return 5;
    }
    return 6;
}

int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
"#;

/// A frame far larger than the guard region is still contained and reported.
///
/// This is the behaviour the probe attribute buys, as distinct from the
/// attribute being present. Both binaries call the same generated function
/// once from the top of the fixture's small stack. Probed, the frame walks its
/// pages and first touches the protected memory within one stride of the low
/// bound, which the floor reports. Ablated, its first access beyond the stack
/// is farther into the fixture's protected reservation, which the floor leaves
/// to the host signal.
///
/// Owning both the stack and the memory beneath it is part of the assertion. A
/// recursive descent on a host-created stack leaves the final pre-fault stack
/// position and neighbouring mappings to target code generation and address
/// placement; either can make an unprobed run fault inside the reporting band
/// even though its large frame did not walk the guard.
#[test]
fn a_frame_larger_than_the_guard_region_is_still_reported() {
    let module = expose_large_frame_spine(&compile(LARGE_FRAME_SPINE));
    let directory = test_directory();
    let executable = build_linked_executable(&module, Some(LARGE_FRAME_BODY), &[], &directory);
    let output = Command::new(&executable)
        .output()
        .expect("run the probed large frame");
    assert_eq!(
        signal_of(&output),
        Some(libc_sigabrt()),
        "a probed frame that exhausts its stack ends in the floor's abort: {:?}",
        output.status,
    );
    assert_resource_record(&output.stderr, "stack");

    let ablated = ablate_probe(&module, "@wf_spine(");
    assert_eq!(
        module.matches(" #0 {").count() - ablated.matches(" #0 {").count(),
        1,
        "the ablation must remove the group from exactly one definition"
    );
    let elsewhere = test_directory();
    let unprobed = build_linked_executable(&ablated, Some(LARGE_FRAME_BODY), &[], &elsewhere);
    let output = Command::new(&unprobed)
        .output()
        .expect("run the unprobed large frame");
    assert!(
        output.stderr.is_empty(),
        "an unprobed frame steps over the guard region, so the fault it \
         eventually takes is not this thread running out and must not be \
         reported as one: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        signal_of(&output),
        Some(protected_page_signal()),
        "an unprobed frame keeps the host's protection-fault signal rather than \
         becoming the floor's abort: {:?}",
        output.status,
    );
    std::fs::remove_dir_all(&directory).expect("remove the test directory");
    std::fs::remove_dir_all(&elsewhere).expect("remove the second test directory");
}

/// Makes the generated function callable by the C fixture and leaves the
/// fixture to supply the program entry. The generated entry remains in the
/// module under an unused name so this changes no part of `wf_spine` itself.
fn expose_large_frame_spine(module: &str) -> String {
    let mut exposed = module
        .replacen("define internal i64 @wf_spine(", "define i64 @wf_spine(", 1)
        .replacen(
            "define i32 @wf__main_body(",
            "define i32 @wf__unused_main_body(",
            1,
        )
        .replacen("define i32 @main(", "define i32 @wf__unused_main(", 1);
    // Keep the ordinary source borrow crossing an opaque call boundary. The
    // reader's body and result stay unchanged; external visibility prevents
    // whole-program argument specialization and noinline preserves its call.
    exposed = exposed
        .lines()
        .map(|line| {
            if line.starts_with("define i64 @wf_read_pad(") {
                line.replace(" #0 {", " noinline #0 {")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    exposed.push_str("\ndeclare i32 @wf__main_body(i32, ptr)\n");
    assert_eq!(
        module.matches("define i64 @wf_spine(").count(),
        1,
        "the fixture must expose exactly one generated spine"
    );
    assert_eq!(
        exposed.matches("define i64 @wf_read_pad(").count(),
        1,
        "the fixture retains exactly one source array reader"
    );
    assert_eq!(
        exposed.matches("define i32 @wf__unused_main_body(").count(),
        1,
        "the fixture must rename exactly one generated entry body"
    );
    assert_eq!(
        exposed.matches("define i32 @wf__unused_main(").count(),
        1,
        "the fixture must rename exactly one generated host entry"
    );
    exposed
}

#[cfg(target_os = "macos")]
const fn protected_page_signal() -> i32 {
    10
}

#[cfg(not(target_os = "macos"))]
const fn protected_page_signal() -> i32 {
    libc_sigsegv()
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

fn boxed_leaf() -> result: own box<Tree> pure {{
  let leaf = Leaf();
  return box_new(move leaf);
}}

fn boxed_branch(left: own box<Tree>, right: own box<Tree>) -> result: own box<Tree> pure {{
  let branch = Branch(left: move left, right: move right);
  return box_new(move branch);
}}

fn main() -> status: own ExitStatus pure {{
  let seed = boxed_leaf();
  let held = Holder(node: move seed);
  for @grow (i in 0_u64..{depth}_u64) {{
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

fn nest(inner: own Chain) -> result: own Chain pure {{
  let slots = buffer_vacant::<Chain>(1_u64);
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

fn main() -> status: own ExitStatus pure {{
  let holder = buffer_vacant::<Chain>(1_u64);
  let seed = Nil();
  let seeded = Some<Chain>(value: move seed);
  let empty = replace holder[0_u64] = move seeded;
  match empty {{
    None() => {{
    }}
    Some(value: stray) => {{
    }}
  }}
  for @build (i in 0_u64..{depth}_u64) {{
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
const SHALLOW_OWNERSHIP: &[u8] = br#"fn main() -> status: own ExitStatus pure {
  let slots = buffer_vacant::<box<u64>>(2_u64);
  let boxed = box_new(7_u64);
  let wrapped = Some<box<u64>>(value: move boxed);
  let vacant = replace slots[0_u64] = move wrapped;
  return exit_status(code: 0_u8);
}
"#;

/// Every compiler-derived drop in one module, with the drops each one calls.
pub(super) fn drop_glue_calls(module: &str) -> Vec<(String, Vec<String>)> {
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
            && let Some((_, calls)) = glue.last_mut()
        {
            calls.push(format!(
                "wf.drop.{}",
                callee.split('(').next().unwrap_or_default()
            ));
        }
    }
    glue
}

/// The derived release of a type whose release graph has a cycle is one
/// release action per node type, entering itself where the graph closes
/// [PROV-6].
///
/// This assertion was its own opposite until 2026-09-04. The compiler ran such
/// a walk on an explicit heap worklist, and the property under test was that no
/// compiler-derived drop reached itself, because the recursion has no name in
/// the source. The owner deleted [PROV-6]'s release-graph cycle refusal that
/// day and ruled that the walk may recurse: a cycle can arise only where a heap
/// is allowed, and a heap-allowed program's resource behaviour is a runtime
/// quantity already, while the worklist bought its bounded depth with a
/// `realloc` on the release path — an allocation, and on refusal an abort, that
/// the writer never wrote. The recursion is not invisible: it is a `STACK
/// cycle` row of the stack ledger, beside the recursions the writer did write.
///
/// The check is over the emitted module rather than over a list of names, so a
/// new nominal shape whose glue closes a cycle is read here without anyone
/// remembering to extend a table.
#[test]
fn a_cyclic_release_graph_lowers_to_one_release_action_that_enters_itself() {
    // Both owning indirections a cleanup cycle can close through.
    assert_recursive_drop_glue(&boxed_module());
    assert_recursive_drop_glue(&buffer_module());
}

fn assert_recursive_drop_glue(module: &str) {
    let glue = drop_glue_calls(module);
    assert!(
        !glue
            .iter()
            .any(|(name, _)| name.starts_with("wf.drop.step.")),
        "the release walk is the type's own release actions and no traversal \
         driver: {module}"
    );
    assert!(
        !module.contains("@wf.drop.push"),
        "a release walk allocates nothing: {module}"
    );
    let mut inside = false;
    for line in module.lines() {
        if line.starts_with("define private void @wf.drop.") {
            inside = true;
        } else if line == "}" {
            inside = false;
        } else if inside {
            assert!(
                !line.contains("@wf_resource_abort")
                    && !line.contains("@realloc")
                    && !line.contains("@malloc"),
                "a release action allocates nothing and reaches no abort: \
                 {line}"
            );
        }
    }
    assert!(
        release_graph_has_cycle(&glue),
        "a recursive nominal's release graph must close: {module}"
    );
}

fn release_graph_has_cycle(glue: &[(String, Vec<String>)]) -> bool {
    let index: std::collections::HashMap<&str, usize> = glue
        .iter()
        .enumerate()
        .map(|(position, (name, _))| (name.as_str(), position))
        .collect();
    // The release graph closes, so exactly one of these definitions reaches
    // itself through the module's own call graph. Depth-first, looking for the
    // back edge rather than refusing it.
    let mut colour = vec![0_u8; glue.len()];
    let mut path: Vec<(usize, usize)> = Vec::new();
    let mut closed = false;
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
            if colour[target] == 1 {
                closed = true;
                continue;
            }
            if colour[target] == 0 {
                colour[target] = 1;
                path.push((target, 0));
            }
        }
    }
    closed
}

/// An acyclic ownership graph must have no direct or mutual release cycle.
#[test]
fn an_ownership_chain_keeps_its_straight_line_drop() {
    let module = compile(SHALLOW_OWNERSHIP);
    let glue = drop_glue_calls(&module);
    assert!(!glue.is_empty(), "the fixture must derive release actions");
    assert!(!release_graph_has_cycle(&glue), "{module}");
}

/// A buffer in a cleanup cycle whose elements each own further storage.
const WIDE_BUFFER_CYCLE: &[u8] = br#"enum Chain {
  Nil();
  Cons(kids: box<buffer<Option<Chain>>>);
}

fn leafy() -> result: own Chain pure {
  let slots = buffer_vacant::<Chain>(1_u64);
  let held = box_new(move slots);
  return Cons(kids: move held);
}

fn main() -> status: own ExitStatus pure {
  let slots = buffer_vacant::<Chain>(4_u64);
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

/// [STOR-3] fixes a buffer's release as each element's release in ascending
/// index order followed by that one heap free, and the release of a buffer
/// inside a cycle is the same action as the release of any other buffer.
///
/// The order is pinned where it is chosen because nothing downstream can see
/// it: [STOR-3] gives memory reclamation the empty effect row. Walking the
/// indices downward, or freeing the block first, both emit a release that
/// looks right and reclaims in the wrong order — and freeing first would read
/// every element out of storage the release had already given back.
///
/// The case read a worklist step here until 2026-09-04. The order it pins is
/// the same one; what changed is that the elements are released by an ordinary
/// ascending loop rather than pushed in reverse onto a last-in first-out list.
#[test]
fn a_buffer_in_a_cleanup_cycle_is_walked_in_the_order_the_rule_fixes() {
    let module = buffer_module();
    // The buffer's own release action: the element loop, then the block free.
    let buffer_drop = definition_body(&module, "define private void @wf.drop.buffer.t");
    assert!(
        buffer_drop.contains("%index = phi i64 [ 0, %entry ], [ %next, %body ]")
            && buffer_drop.contains("%next = add i64 %index, 1"),
        "the elements must be released from index zero upward: {buffer_drop}"
    );
    let element = buffer_drop
        .find("%element = load")
        .expect("the buffer release reads each live element");
    let body = buffer_drop
        .split("body:")
        .nth(1)
        .expect("element loop body")
        .split("done:")
        .next()
        .unwrap();
    assert!(
        body.contains("call void @wf.drop."),
        "each live element invokes its release action: {body}"
    );
    assert!(
        body.contains("br label %head") && !body.contains("call void @free(ptr %pointer)"),
        "backing may only be released after leaving the element loop: {body}"
    );
    let block = buffer_drop
        .find("call void @free(ptr %pointer)")
        .expect("the buffer release frees its own block");
    assert!(
        element < block,
        "every element must be released before the block that holds it: \
         {buffer_drop}"
    );
}

/// Allocation-instance identities expose omitted/double releases, field/element
/// order and backing lifetime directly. Large depth and unverified allocator
/// scribbling did not establish those properties.
#[test]
fn branching_and_nested_release_walks_reclaim_each_instance_in_order() {
    let mut boxed_trace = String::from("A1;");
    for round in 0..4 {
        let first = 2 + 3 * round;
        boxed_trace.push_str(&format!(
            "A{first};A{};A{};F{};",
            first + 1,
            first + 2,
            first + 1
        ));
    }
    boxed_trace.push_str("F1;");
    for round in 0..4 {
        let first = 2 + 3 * round;
        boxed_trace.push_str(&format!("F{first};F{};", first + 2));
    }
    let nested_trace = (1..=9).map(|id| format!("A{id};")).collect::<String>()
        + &(2..=9).map(|id| format!("F{id};")).collect::<String>()
        + "F1;";
    let wide_trace = (1..=10).map(|id| format!("A{id};")).collect::<String>()
        + &(2..=9).map(|id| format!("F{id};")).collect::<String>()
        + "F1;F10;";
    for (name, module, limit, expected) in [
        ("boxed branches", boxed_module(), 13, boxed_trace),
        ("nested buffer", buffer_module(), 9, nested_trace),
        ("wide buffer", compile(WIDE_BUFFER_CYCLE), 10, wide_trace),
    ] {
        let observed = module
            .replace("@malloc(", "@wf_test_allocate(")
            .replace("@free(", "@wf_test_release(");
        let directory = test_directory();
        let observer = super::owned_places::allocation_observer(limit, 0);
        let executable = build_linked_executable(&observed, Some(&observer), &[], &directory);
        let output = Command::new(executable)
            .output()
            .expect("run observed release walk");
        assert_eq!(output.status.code(), Some(0), "{name}: {output:?}");
        assert_eq!(output.stdout, expected.as_bytes(), "{name}");
        assert!(output.stderr.is_empty(), "{name}: {output:?}");
        std::fs::remove_dir_all(directory).expect("remove release observation");
    }
}
