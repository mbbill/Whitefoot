//! Executable evidence for the bounded selective stackless slice.

use std::process::{Command, Stdio};

use super::system::with_mutated_completion_ir;
use super::{build_linked_executable, compile, test_directory};
use crate::backend::emitter::emit_llvm_for_target;
use crate::backend::qualification::SystemTarget;

const WRITER_SCHEDULER_PROBE: &str = include_str!("../completion/writer_scheduler_probe.c");

const STACKLESS_WRAPPER: &[u8] = br#"fn publish(output: &uniq Output, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output) contract {
  define ordered = start <= end;
  define capacity = len_of(deref(source));
  requires ordered;
  requires end <= capacity;
} {
  return write_once(output: move output, source: source, start: start, end: end);
}

fn relay(output: &uniq Output, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output) contract {
  define ordered = start <= end;
  define capacity = len_of(deref(source));
  requires ordered;
  requires end <= capacity;
} {
  return publish(output: move output, source: source, start: start, end: end);
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region {
    let outcome = relay(output: &uniq out, source: &bytes, start: 0_u64, end: 1_u64);
  }
  return exit_status(code: 0_u8);
}
"#;

const STACKLESS_EMPTY_WRAPPER: &[u8] = br#"fn publish(output: &uniq Output, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output) contract {
  define ordered = start <= end;
  define capacity = len_of(deref(source));
  requires ordered;
  requires end <= capacity;
} {
  return write_once(output: move output, source: source, start: start, end: end);
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region {
    let outcome = publish(output: &uniq out, source: &bytes, start: 0_u64, end: 0_u64);
  }
  return exit_status(code: 0_u8);
}
"#;

/// How many runs the migration observation below gets.
///
/// A migration is a rare race, not a property that shows up on demand: the
/// ready frame is claimed by a scheduler lane only if a lane reaches it inside
/// a window one call wide, before the writer thread resumes it itself. On a
/// machine with cores to spare — this one, with ten — it happens within a run
/// or two. On a shared runner it does not: measured across this batch's gate
/// runs, the three-core macOS runner and the four-core Linux runner each
/// reached one migration in some gate runs and none across a whole
/// ninety-six-run sample in others, which puts the rate near one attempt in a
/// hundred. Ninety-six attempts therefore missed it about half the time, and
/// each miss was a red gate reporting a scheduler defect that was not there.
///
/// A thousand attempts miss a one-in-a-hundred event about once in a thousand
/// gate runs. The loop stops at the first migration, so the cost is paid only
/// where the event is rare: measured here at 16 milliseconds a run with the
/// link outside the loop, so a host that never sees one spends about twenty
/// seconds and a host that sees one immediately spends none of it.
const MIGRATION_ATTEMPTS: usize = 1024;

fn compile_windows_stackless(source: &[u8]) -> String {
    let target = SystemTarget::for_triple("x86_64-pc-windows-msvc")
        .expect("the native Windows stackless target");
    with_mutated_completion_ir(source, |program| {
        emit_llvm_for_target(program, target)
            .expect("the Windows stackless probe must emit")
            .into_string()
    })
}

#[test]
fn windows_stackless_submit_waits_for_capacity_and_retries_its_single_registration() {
    let module = compile_windows_stackless(STACKLESS_WRAPPER);
    let tail = module
        .split("\ndefine internal i1 @wf__stackless_start_")
        .skip(1)
        .find(|definition| definition.contains("@wf__completion_file_write_submit_writer"))
        .expect("the stackless write tail");

    for declaration in [
        "declare i32 @wf__completion_file_write_submit_writer(i32, ptr, i64, ptr, ptr)",
        "declare void @wf__completion_wait_core_capacity()",
        "declare void @wf__writer_begin_suspend(ptr, ptr)",
        "declare void @abort() noreturn",
    ] {
        assert!(
            module.contains(declaration),
            "Windows stackless output must name the native ABI `{declaration}`:\n{module}"
        );
    }
    assert_eq!(
        module
            .matches("declare void @wf__completion_wait_core_capacity()")
            .count(),
        1,
        "the completion and stackless ABI sets must not redeclare the shared wait helper"
    );
    assert!(
        !module.contains("define weak i32 @wf__completion_file_write_submit_writer"),
        "a missing Windows stackless runtime must be a link error"
    );
    assert!(
        !module.contains("define weak void @wf__writer_begin_suspend"),
        "Windows must not retain a weak writer-scheduler fallback"
    );

    assert_eq!(
        tail.matches("call i32 @wf__completion_file_write_submit_writer")
            .count(),
        1,
        "capacity retry must jump to the same submit call"
    );
    assert!(tail.contains("%direct_only = icmp eq i32 %status, 0"));
    assert!(tail.contains("%accepted = icmp eq i32 %status, 1"));
    assert!(tail.contains("%capacity = icmp eq i32 %status, 2"));
    assert!(tail.contains("br i1 %capacity, label %capacity_wait, label %invalid_submit"));
    assert!(tail.contains(
        "capacity_wait:\n  call void @wf__completion_wait_core_capacity()\n  br label %submit"
    ));
    assert!(tail.contains("invalid_submit:\n  call void @abort()\n  unreachable"));
    assert!(
        !tail.contains("@wf__writer_begin_suspend"),
        "the tail retry must not register the frame again"
    );
}

#[test]
fn selected_target_validated_root_frame_passes_llvm_verification() {
    let llvm = compile(STACKLESS_WRAPPER);
    assert!(llvm.contains("%frame = alloca { [64 x i8], [2 x i64]"));
    assert!(llvm.contains("}, align 8\n  call void @wf__writer_frame_init"));
    assert!(llvm.contains("call void @wf__completion_file_join"));
    assert!(!llvm.contains("invalid_take:"));

    let directory = test_directory();
    let module = directory.join("root_frame.ll");
    let object = directory.join("root_frame.o");
    std::fs::write(&module, llvm).expect("write validated root-frame module");
    let output = Command::new("/usr/bin/clang")
        .args(["-Wno-override-module", "-x", "ir", "-c"])
        .arg(&module)
        .arg("-o")
        .arg(&object)
        .output()
        .expect("run the LLVM verifier through clang");
    std::fs::remove_dir_all(&directory).expect("remove root-frame verifier directory");
    assert!(
        output.status.success(),
        "validated root-frame module did not verify:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn may_suspend_tail_wrappers_release_the_writer_stack_and_resume_on_a_scheduler_lane() {
    let llvm = compile(STACKLESS_WRAPPER);
    assert!(llvm.contains("@wf__stackless_root_start_"));
    assert!(llvm.contains("@wf__stackless_root_resume_"));
    assert!(llvm.contains("@wf__stackless_start_"));
    assert!(llvm.contains("@wf__completion_file_write_submit_writer"));
    assert!(llvm.contains("call void @wf__writer_run_root"));
    assert!(crate::module_requires_writer_scheduler(&llvm));
    assert!(llvm.contains("alloca { [64 x i8], [2 x i64]"));
    assert!(!llvm.contains("setjmp"));
    assert!(!llvm.contains("fiber"));

    let host = r#"
#include <stdint.h>
#include <stdio.h>
extern uint64_t wf__writer_resume_migrations(void);
extern uint64_t wf__writer_resume_count(void);
__attribute__((destructor)) static void report_writer_resume(void) {
    fprintf(stderr, "writer-resumes=%llu migrations=%llu\n",
        (unsigned long long)wf__writer_resume_count(),
        (unsigned long long)wf__writer_resume_migrations());
}
"#;
    {
        let directory = test_directory();
        let executable = build_linked_executable(&llvm, Some(host), &[], &directory);
        let output = Command::new(&executable)
            .env("WF_WORKERS", "0")
            .env("WF_IO_HELPERS", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("run single-lane stackless wrapper executable");
        std::fs::remove_dir_all(&directory).expect("remove stackless test directory");
        assert!(
            output.status.success(),
            "single-lane stackless executable failed: {output:?}"
        );
        assert_eq!(output.stdout, b"A");
        let report = String::from_utf8(output.stderr).expect("ASCII scheduler report");
        assert!(report.contains("writer-resumes=1"), "{report}");
        assert!(report.contains("migrations=0"), "{report}");
    }
    // A migration is a race the observation has to win: the ready frame is
    // claimed by a scheduler lane only if a lane reaches it before the writer
    // resumes it itself. Nothing about the module or the fixture changes
    // between attempts, so the link is paid once and the sample is as wide as
    // the runs are cheap.
    let mut migrated = false;
    let directory = test_directory();
    let executable = build_linked_executable(&llvm, Some(host), &[], &directory);
    for _ in 0..MIGRATION_ATTEMPTS {
        let output = Command::new(&executable)
            .env("WF_WORKERS", "2")
            .env("WF_IO_HELPERS", "1")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("run stackless wrapper executable");
        assert!(
            output.status.success(),
            "stackless executable failed: {output:?}"
        );
        assert_eq!(output.stdout, b"A");
        let report = String::from_utf8(output.stderr).expect("ASCII scheduler report");
        assert!(report.contains("writer-resumes=1"), "{report}");
        if !report.contains("migrations=0") {
            migrated = true;
            break;
        }
    }
    std::fs::remove_dir_all(&directory).expect("remove stackless test directory");
    assert!(
        migrated,
        "an eligible scheduler worker never claimed the ready frame"
    );
}

#[test]
fn an_empty_stackless_write_stays_inline_and_issues_no_writer_resume() {
    let llvm = compile(STACKLESS_EMPTY_WRAPPER);
    assert!(llvm.contains("%vacant = icmp eq i64 %extent, 0"));
    assert!(llvm.contains("br i1 %vacant, label %inline, label %submit"));
    let host = r#"
#include <stdint.h>
#include <stdio.h>
extern uint64_t wf__writer_resume_count(void);
__attribute__((destructor)) static void report_writer_resume(void) {
    fprintf(stderr, "writer-resumes=%llu\n",
        (unsigned long long)wf__writer_resume_count());
}
"#;
    let directory = test_directory();
    let executable = build_linked_executable(&llvm, Some(host), &[], &directory);
    let output = Command::new(&executable)
        .env("WF_WORKERS", "0")
        .env("WF_IO_HELPERS", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run empty stackless wrapper executable");
    std::fs::remove_dir_all(&directory).expect("remove empty stackless test directory");
    assert!(
        output.status.success(),
        "empty stackless executable failed: {output:?}"
    );
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"writer-resumes=0\n");
}

#[test]
fn unsupported_branching_may_suspend_shape_keeps_the_synchronous_abi() {
    let llvm = compile(
        br#"command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let bytes = buffer_new(1_u64, 65_u8);
  region {
    match write_once(output: &uniq out, source: &bytes, start: 0_u64, end: 1_u64) {
      Ok(value: written) => {
      }
      Err(error: problem) => {
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    assert!(!llvm.contains("wf__stackless"));
    assert!(!llvm.contains("wf__writer_frame_init"));
    assert!(!crate::module_requires_writer_scheduler(&llvm));
    assert!(llvm.contains("@wf_main("));
}

#[test]
fn a_stack_backed_slice_crossing_the_suspend_point_keeps_the_synchronous_abi() {
    let llvm = compile(
        br#"fn publish(output: &uniq Output, source: &buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads(output, source), writes(output) contract {
  define ordered = start <= end;
  define capacity = len_of(deref(source));
  requires ordered;
  requires end <= capacity;
} {
  return write_once(output: move output, source: source, start: start, end: end);
}

fn observe(values: own Slice<u8>) -> result: own unit reads(values) contract {
  define capacity = len_of(values);
  requires 0_u64 < capacity;
} {
  let value = values[0_u64];
  return unit;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let local = array_new::<u8, 1>(65_u8);
  let bytes = buffer_new(1_u64, 65_u8);
  region {
    let view = slice_of(&local);
    region {
      let outcome = publish(output: &uniq out, source: &bytes, start: 0_u64, end: 1_u64);
    }
    let ignored = observe(values: move view);
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    assert!(!llvm.contains("wf__stackless"));
    assert!(!llvm.contains("wf__writer_frame_init"));
    assert!(llvm.contains("@wf_main("));
}

#[test]
fn pure_modules_keep_the_existing_direct_abi_and_link_no_stackless_runtime() {
    let llvm = compile(
        b"command fn main() -> status: own ExitStatus pure {\n  return exit_status(code: 0_u8);\n}\n",
    );
    assert!(!llvm.contains("wf__stackless"));
    assert!(!llvm.contains("wf__writer_"));
    assert!(!llvm.contains("wf__completion_file_write_submit_writer"));

    /* The compute-only C runtime is a separate compiled boundary. Its default
     * source must preprocess the writer-scheduler hooks away, rather than
     * retaining a weak call or an always-false queue probe on every steal
     * pass. */
    let directory = test_directory();
    let source = directory.join("par_runtime.c");
    let object = directory.join("par_runtime.o");
    std::fs::write(&source, crate::PARALLEL_RUNTIME_SOURCE)
        .expect("write pure parallel runtime source");
    let compiled = Command::new("/usr/bin/clang")
        .args(["-std=c11", "-O2", "-pthread", "-c"])
        .arg(&source)
        .arg("-o")
        .arg(&object)
        .output()
        .expect("compile pure parallel runtime");
    assert!(
        compiled.status.success(),
        "pure parallel runtime compile failed: {compiled:?}"
    );
    let symbols = Command::new("/usr/bin/nm")
        .arg(&object)
        .output()
        .expect("inspect pure parallel runtime symbols");
    assert!(symbols.status.success());
    assert!(
        !String::from_utf8_lossy(&symbols.stdout).contains("wf__writer_scheduler"),
        "pure compute runtime retained writer-scheduler code"
    );
    std::fs::remove_dir_all(&directory).expect("remove pure runtime probe directory");
}

#[test]
fn scheduler_races_token_alignment_and_done_wake_pass_under_sanitizers() {
    let directory = test_directory();
    for (name, source) in [
        ("contract.h", crate::COMPLETION_CONTRACT_HEADER),
        ("writer_scheduler.h", crate::WRITER_SCHEDULER_HEADER),
        ("runtime.c", crate::COMPLETION_RUNTIME_SOURCE),
        ("writer_scheduler.c", crate::WRITER_SCHEDULER_SOURCE),
        ("probe.c", WRITER_SCHEDULER_PROBE),
    ] {
        std::fs::write(directory.join(name), source).expect("write writer scheduler probe input");
    }
    let executable = directory.join("writer_scheduler_probe");
    let output = Command::new("/usr/bin/clang")
        .current_dir(&directory)
        .args([
            "-std=c11",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-pthread",
            "-fsanitize=address,undefined",
            "runtime.c",
            "writer_scheduler.c",
            "probe.c",
            "-o",
        ])
        .arg(&executable)
        .output()
        .expect("compile writer scheduler sanitizer probe");
    assert!(output.status.success(), "probe compile failed: {output:?}");
    let status = Command::new(&executable)
        .status()
        .expect("run writer scheduler sanitizer probe");
    assert!(status.success(), "writer scheduler probe exited {status:?}");
    std::fs::remove_dir_all(&directory).expect("remove writer scheduler probe directory");

    assert!(crate::COMPLETION_CONTRACT_HEADER.contains("sizeof(wf_completion_token) == 16u"));
    assert!(crate::COMPLETION_CONTRACT_HEADER.contains("alignof(wf_completion_token)"));
    assert!(!crate::COMPLETION_BRIDGE_SOURCE.contains("wf_bridge_routes"));
    assert!(crate::COMPLETION_RUNTIME_SOURCE.contains("outcome->adapter_tag = slot->adapter_tag"));
    assert!(crate::COMPLETION_RUNTIME_SOURCE.contains("slot->adapter_tag = 0"));
    assert!(!crate::COMPLETION_RUNTIME_SOURCE.contains("ready_frame"));
    assert!(!crate::COMPLETION_BRIDGE_SOURCE.contains("writer_worker"));
    let enqueue = crate::WRITER_SCHEDULER_SOURCE
        .split_once("static void wf_writer_enqueue(void *frame)")
        .expect("writer scheduler has one enqueue transition")
        .1
        .split_once("static void *wf_writer_dequeue")
        .expect("enqueue precedes dequeue")
        .0;
    assert!(enqueue.contains("wf__writer_scheduler_notify();"));
    let final_recheck = crate::PARALLEL_COMPLETION_RUNTIME_SOURCE
        .split_once("__atomic_fetch_or(&wf__par_idle")
        .expect("worker announces idle before its final source checks")
        .1
        .split_once("pthread_mutex_lock(&lane->lock)")
        .expect("final source checks precede the host wait lock")
        .0;
    assert!(final_recheck.contains("WF_PAR_WRITER_HELP_ONCE()"));
}
