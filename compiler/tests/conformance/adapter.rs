//! The named native adapter for the compiler-independent conformance corpus.
//!
//! `tests/conformance/runner.py` owns the corpus: its structure, its declared
//! rule coverage, and the schema of one manifest line. This module owns the
//! other half its docstring names — driving each case through a real
//! toolchain and reducing the outcome to one corpus verdict. It re-derives
//! none of the corpus: it reads the same `manifest.jsonl` bytes, applies the
//! same match rule, and honours the same `runnable`/`pending`/`xfail` axis.
//!
//! The split is deliberate. Python states what the corpus *is*, which must
//! outlive this compiler; Rust states what *this* toolchain does with it,
//! because reproducing compiler behaviour in Python would create a second,
//! divergent implementation of the language.
//!
//! A case reaches its verdict through the ordinary compiler path and, when
//! the expectation is a `run`, through a real invocation: the
//! emitted module is linked with the same host arguments every Whitefoot
//! executable uses, the manifest's `arrange` is realized as actual fixture
//! files, actual argument bytes, an actual standard input, and actual
//! redirection, and the process's own exit status is the verdict. A process
//! that ends without an exit status is a harness stop, never a language
//! verdict.
//! Nothing about a case's identity, name, or family selects a path here.
//!
//! The corpus-wide run is `#[ignore]`d for cost, not for a blocker: it obtains
//! the actual compiler verdict for every non-pending case and links and runs
//! every run case. It is kept out of the default `cargo test` run and
//! invoked by `make conformance-run` with `--ignored`; root `make check`
//! includes that focused target. The wiring and the attribute are one unit.
//! The adapter excludes no case, weakens no expectation, and skips nothing the
//! manifest does not itself mark `pending`; running it prints the complete
//! tally.

use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use whitefoot::{
    COMPLETION_BRIDGE_HEADER, COMPLETION_BRIDGE_SOURCE, COMPLETION_CONTRACT_HEADER,
    COMPLETION_FILE_ADAPTER_HEADER, COMPLETION_FILE_ADAPTER_SOURCE, COMPLETION_FILE_POSIX_HEADER,
    COMPLETION_FILE_POSIX_SOURCE, COMPLETION_LINUX_IO_URING_HEADER,
    COMPLETION_LINUX_IO_URING_SOURCE, COMPLETION_RUNTIME_SOURCE, COMPLETION_WAIT_HOST_SOURCE,
    CompilationFailureKind, CompilerLimits, FLOOR_RUNTIME_SOURCE, HOST_LINK_LIBRARIES,
    HOST_OPTIMIZATION_ARGUMENTS, SCHED_CORE_HEADER, SCHED_CORE_SOURCE, SCHED_ENTRY_HEADER,
    SCHED_ENTRY_SOURCE, SCHED_PRIM_HEADER, SCHED_PRIM_HOST_SOURCE, SCHED_SWITCH_HEADER,
    SourceInput, compile, module_requires_completion_runtime, module_requires_parallel_runtime,
};

use super::corpus::{self, Arrangement, Case, Expectation, Status, Verdict};

static NEXT_INVOCATION: AtomicU64 = AtomicU64::new(0);

/// One case's corpus verdict together with the toolchain detail behind it.
///
/// The verdict is what the corpus compares; the note is the compiler's own
/// diagnostic text, kept so a failure report says which stop produced the
/// verdict instead of leaving a reader to guess.
struct Reached {
    verdict: Verdict,
    note: Option<String>,
}

/// Drives one case to its corpus verdict through the ordinary compiler path.
fn reach(case: &Case) -> Reached {
    let source = case.source();
    let path = case.logical_path();
    let module = match compile(
        &[SourceInput::new(&path, &source)],
        CompilerLimits::default(),
    ) {
        Ok(module) => module,
        Err(failure) => {
            let note = Some(failure.to_string());
            let verdict = match failure.kind() {
                // A numbered source rule was violated. The rule the
                // diagnostic cites is the verdict's whole content; an
                // uncited stop stays `None` rather than being attributed to
                // the rule the case happens to declare.
                CompilationFailureKind::Source => {
                    Verdict::Reject(failure.rule_id().map(ToOwned::to_owned))
                }
                // [QUAL-1] and [QUAL-2] stops are the specification's own
                // non-rejections citing no language rule.
                CompilationFailureKind::TargetQualification => {
                    Verdict::Unsupported(failure.to_string())
                }
                // Valid source needing an unimplemented capability reaches
                // the same verdict kind, so a `runnable` case fails rather
                // than passing as though the language had judged it.
                CompilationFailureKind::Unsupported => Verdict::Unsupported(failure.to_string()),
                // Everything else is a compiler, resource, or target stop the
                // corpus does not model. It must never become a verdict.
                _ => Verdict::Stopped(failure.to_string()),
            };
            return Reached { verdict, note };
        }
    };
    if !case.expect.needs_execution() {
        return Reached {
            verdict: Verdict::Accept,
            note: None,
        };
    }
    Reached {
        verdict: execute(&module, case.arrange.as_ref()),
        note: None,
    }
}

/// Links and runs one emitted module under the case's own arrangement.
fn execute(module: &str, arrange: Option<&Arrangement>) -> Verdict {
    let default = Arrangement::default();
    let arrange = arrange.unwrap_or(&default);
    let directory = invocation_directory();
    let executable = link(module, &directory);
    for fixture in &arrange.files {
        let path = directory.join(Path::new(std::ffi::OsStr::from_bytes(&fixture.path)));
        match &fixture.bytes {
            Some(bytes) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).expect("fixture parent directory");
                }
                std::fs::write(&path, bytes).expect("write fixture file");
            }
            None => std::fs::create_dir_all(&path).expect("create fixture directory"),
        }
    }

    let mut command = Command::new(&executable);
    command.current_dir(&directory);
    // `arrange.argv` is the complete native vector, so position 0 is the
    // invoked name and the rest are the arguments after it. An absent `argv`
    // leaves the invocation's own default vector, which is exactly the one
    // element the link step produced.
    if let Some(argv) = &arrange.argv {
        let (invoked, rest) = argv.split_first().expect("argv is nonempty");
        command.arg0(std::ffi::OsStr::from_bytes(invoked));
        command.args(rest.iter().map(|bytes| std::ffi::OsStr::from_bytes(bytes)));
    }
    // Absent means an empty standard input, not an inherited one: a case
    // never reads the harness's own input.
    match &arrange.stdin {
        Some(_) => command.stdin(Stdio::piped()),
        None => command.stdin(Stdio::null()),
    };
    let mut sinks: BTreeMap<&str, std::fs::File> = BTreeMap::new();
    for (stream, label) in &arrange.redirect {
        let file = match sinks.get(label.as_str()) {
            // Two streams naming one sink are one destination sharing one
            // open file description, which is what makes cross-owner call
            // order observable in the combined bytes [EFF-5, SYS-12].
            Some(open) => open.try_clone().expect("duplicate the shared sink"),
            None => {
                let file = std::fs::File::create(directory.join(label)).expect("create the sink");
                sinks.insert(
                    label.as_str(),
                    file.try_clone().expect("retain the shared sink"),
                );
                file
            }
        };
        match stream.as_str() {
            "stdout" => command.stdout(Stdio::from(file)),
            "stderr" => command.stderr(Stdio::from(file)),
            other => panic!("unknown redirect stream {other}"),
        };
    }
    // An unredirected stream is its own separate sink. Capturing it keeps two
    // separate destinations without publishing a case's bytes into the gate's
    // own output.
    if !arrange.redirect.contains_key("stdout") {
        command.stdout(Stdio::piped());
    }
    if !arrange.redirect.contains_key("stderr") {
        command.stderr(Stdio::piped());
    }

    let mut child = command.spawn().expect("run conformance case executable");
    if let Some(bytes) = &arrange.stdin {
        child
            .stdin
            .take()
            .expect("standard input was piped")
            .write_all(bytes)
            .expect("supply the case's standard input");
    }
    let status = child
        .wait_with_output()
        .expect("wait for conformance case executable")
        .status;
    std::fs::remove_dir_all(&directory).expect("remove conformance invocation directory");

    status.code().map_or_else(
        || Verdict::Stopped("program terminated without an exit status".to_owned()),
        Verdict::Run,
    )
}

fn invocation_directory() -> PathBuf {
    let sequence = NEXT_INVOCATION.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "whitefoot-conformance-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("unique conformance invocation directory");
    directory
}

fn link(module: &str, directory: &Path) -> PathBuf {
    let assembly = directory.join("case.ll");
    let executable = directory.join("case");
    std::fs::write(&assembly, module).expect("write the case's emitted module");
    let mut command = Command::new("/usr/bin/clang");
    command.arg("-x").arg("ir").arg(&assembly);
    let floor = directory.join("wf_floor.c");
    std::fs::write(&floor, FLOOR_RUNTIME_SOURCE).expect("write the floor runtime");
    command.arg("-pthread").arg("-x").arg("c").arg(&floor);
    // The scheduler core joins under the union of the two predicates
    // (`research/investigations/io-model/PARK-ON-MISS.md` §7, "Where the core
    // is linked"): one scheduler for compute hand-outs and I/O completions.
    // The completion units join only under the second.
    let completion = module_requires_completion_runtime(module);
    if completion || module_requires_parallel_runtime(module) {
        for (name, source) in [
            ("sched/core.h", SCHED_CORE_HEADER),
            ("sched/prim.h", SCHED_PRIM_HEADER),
            ("sched/switch.h", SCHED_SWITCH_HEADER),
            ("sched/entry.h", SCHED_ENTRY_HEADER),
            ("sched/core.c", SCHED_CORE_SOURCE),
            ("sched/prim_host.c", SCHED_PRIM_HOST_SOURCE),
            ("sched/entry.c", SCHED_ENTRY_SOURCE),
        ] {
            std::fs::create_dir_all(directory.join("sched")).expect("stage scheduler directory");
            std::fs::write(directory.join(name), source).expect("write scheduler core unit");
        }
        command
            .arg("-x")
            .arg("c")
            .arg(directory.join("sched/core.c"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("sched/prim_host.c"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("sched/entry.c"));
    }
    if completion {
        for (name, source) in [
            ("completion/contract.h", COMPLETION_CONTRACT_HEADER),
            ("completion/file_adapter.h", COMPLETION_FILE_ADAPTER_HEADER),
            ("completion/bridge.h", COMPLETION_BRIDGE_HEADER),
            ("completion/file_posix.h", COMPLETION_FILE_POSIX_HEADER),
            (
                "completion/linux_io_uring.h",
                COMPLETION_LINUX_IO_URING_HEADER,
            ),
            ("completion/completion_runtime.c", COMPLETION_RUNTIME_SOURCE),
            ("completion/wait_host.c", COMPLETION_WAIT_HOST_SOURCE),
            ("completion/file_adapter.c", COMPLETION_FILE_ADAPTER_SOURCE),
            ("completion/file_posix.c", COMPLETION_FILE_POSIX_SOURCE),
            ("completion/completion_bridge.c", COMPLETION_BRIDGE_SOURCE),
            (
                "completion/linux_io_uring.c",
                COMPLETION_LINUX_IO_URING_SOURCE,
            ),
        ] {
            std::fs::create_dir_all(directory.join("completion"))
                .expect("stage completion directory");
            std::fs::write(directory.join(name), source).expect("write completion runtime unit");
        }
        command
            .arg("-I")
            .arg(directory.join("completion"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("completion/completion_runtime.c"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("completion/wait_host.c"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("completion/file_adapter.c"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("completion/file_posix.c"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("completion/completion_bridge.c"))
            .arg("-x")
            .arg("c")
            .arg(directory.join("completion/linux_io_uring.c"));
    }
    let linked = command
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        // One link recipe: a case that reaches a libm entry point links here
        // exactly as it links under the shipped driver.
        .args(HOST_LINK_LIBRARIES)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        linked.status.success(),
        "clang rejected the emitted module:\n{}\n{module}",
        String::from_utf8_lossy(&linked.stderr)
    );
    std::fs::remove_file(&assembly).expect("remove the case's emitted module");
    executable
}

/// One case's outcome on the readiness axis, named as `runner.py` names it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Outcome {
    Pass,
    Fail,
    Xfail,
    Xpass,
    Skip,
}

fn outcome(case: &Case, reached: &Verdict) -> Outcome {
    let matched = case.expect.matched_by(reached);
    match case.status {
        Status::Pending => Outcome::Skip,
        Status::Xfail if matched => Outcome::Xpass,
        Status::Xfail => Outcome::Xfail,
        // A runnable case that stops as unsupported is a toolchain gap
        // reported in the wrong place: runnable means supported, and a gap
        // belongs in `status`, never in a verdict comparison.
        Status::Runnable
            if matches!(reached, Verdict::Unsupported(_))
                && case.expect != Expectation::Unsupported =>
        {
            Outcome::Fail
        }
        Status::Runnable if matched => Outcome::Pass,
        Status::Runnable => Outcome::Fail,
    }
}

#[test]
#[ignore = "Cost, not a blocker: this obtains every non-pending case's compiler verdict \
            and links and runs every run case, so it stays out of default `cargo test`. \
            `make conformance-run` invokes it with `--ignored`, and root `make check` includes \
            that target; removing the attribute without dropping `--ignored` would select no test."]
fn the_corpus_reaches_its_declared_verdict_through_the_ordinary_compiler_path() {
    let cases = corpus::load();
    assert!(
        !cases.is_empty(),
        "the conformance manifest declared no case"
    );
    let mut tally: BTreeMap<Outcome, usize> = BTreeMap::new();
    let mut reports = Vec::new();
    for case in &cases {
        if case.status == Status::Pending {
            *tally.entry(Outcome::Skip).or_default() += 1;
            continue;
        }
        let reached = reach(case);
        let outcome = outcome(case, &reached.verdict);
        *tally.entry(outcome).or_default() += 1;
        let interesting = match outcome {
            Outcome::Fail | Outcome::Xpass => true,
            Outcome::Xfail => true,
            Outcome::Pass | Outcome::Skip => false,
        };
        if interesting {
            let note = reached
                .note
                .or_else(|| case.reason.clone())
                .unwrap_or_default();
            reports.push(format!(
                "  {outcome:?} {} want {:?} reached {:?}  {note}",
                case.id, case.expect, reached.verdict
            ));
        }
    }
    let summary = tally
        .iter()
        .map(|(outcome, count)| format!("{outcome:?}={count}"))
        .collect::<Vec<_>>()
        .join("  ");
    for report in &reports {
        println!("{report}");
    }
    println!("conformance adapter: {summary}");
    let failed = tally.get(&Outcome::Fail).copied().unwrap_or_default()
        + tally.get(&Outcome::Xpass).copied().unwrap_or_default();
    assert_eq!(failed, 0, "conformance adapter: {summary}");
}
