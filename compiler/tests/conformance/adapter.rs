//! The named native adapter for the compiler-independent conformance corpus.
//!
//! `tests/conformance/runner.py` owns the corpus: its structure, its declared
//! rule coverage, and the schema of one manifest line. This module owns the
//! other half its docstring names — driving each case through a real
//! toolchain and reducing the outcome to one corpus verdict. It re-derives
//! none of the corpus: it reads the same `manifest.jsonl` bytes and applies
//! the same match rule. There is no readiness axis: `runnable` is the only
//! status a manifest line may carry, so every case must reach its declared
//! verdict and a case that does not is a defect rather than a status.
//!
//! The split is deliberate. Python states what the corpus *is*, which must
//! outlive this compiler; Rust states what *this* toolchain does with it,
//! because reproducing compiler behaviour in Python would create a second,
//! divergent implementation of the language.
//!
//! A source `accept` or `reject` reaches the complete semantic-publication
//! boundary of the ordinary compiler path. A `run` or `unsupported` case
//! continues through ordinary lowering and target compilation, and a `run`
//! then reaches a real invocation: the emitted module is linked with the same
//! host arguments every Whitefoot executable uses, the manifest's `arrange`
//! is realized as actual fixture files, actual argument bytes, an actual
//! standard input, and actual redirection. A process that ends without an exit
//! status is a harness stop, never a language verdict.
//! Nothing about a case's identity, name, or family selects a path here.
//!
//! The corpus-wide run is an ordinary test in the shared source-corpus
//! executable. It obtains every case's verdict and executes each run case.
//! `make conformance-run` selects it for focused use; the full gate reaches
//! it once through the normal corpus tests, with no ignored opt-in. The
//! adapter excludes no case, weakens no expectation, and skips nothing;
//! running it prints the complete tally.

use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use whitefoot::{
    CompilationFailureKind, CompilerLimits, HOST_LINK_LIBRARIES, HOST_OPTIMIZATION_ARGUMENTS,
    SourceInput, check, compile,
};

use crate::support::append_runtime_objects;

use super::corpus::{self, Arrangement, Case, Expectation, Verdict};

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
    let inputs = [SourceInput::new(&path, &source)];
    // `accept` and `reject` are source-language verdicts. [STOR-6] begins only
    // after their complete semantic boundary, so target qualification cannot
    // change either one. A runtime or unsupported expectation still needs the
    // complete toolchain: the former executes its module, while the latter may
    // name a capability first encountered during lowering.
    let module = match &case.expect {
        Expectation::Accept | Expectation::Reject(_) => {
            check(&inputs, CompilerLimits::default()).map(|()| None)
        }
        Expectation::Run(_) | Expectation::Unsupported => {
            compile(&inputs, CompilerLimits::default()).map(Some)
        }
    };
    let module = match module {
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
        verdict: execute(
            module
                .as_deref()
                .expect("a run expectation uses the complete compiler path"),
            case.arrange.as_ref(),
        ),
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
            // order observable in the combined bytes. Ordinary EFF-2/PAR-1
            // paths over the shared factory preserve that call order.
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
    command.arg("-pthread");
    let (_sources, objects) = append_runtime_objects(&mut command, directory, None, None);
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
    for object in objects {
        std::fs::remove_file(object).expect("remove materialized native library object");
    }
    std::fs::remove_file(&assembly).expect("remove the case's emitted module");
    executable
}

/// One case's outcome, named as `runner.py` names it.
///
/// x1 retires the readiness axis with the manifest statuses that carried it:
/// `Xfail`, `Xpass` and `Skip` are gone because `pending` and `xfail` are no
/// longer manifest values, so a case either reaches its declared verdict or
/// fails.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Outcome {
    Pass,
    Fail,
}

fn outcome(case: &Case, reached: &Verdict) -> Outcome {
    // A case that stops as unsupported without declaring it is a toolchain gap
    // reported in the wrong place: the corpus has no status for a gap, so it
    // is an ordinary failure.
    if matches!(reached, Verdict::Unsupported(_)) && case.expect != Expectation::Unsupported {
        return Outcome::Fail;
    }
    if case.expect.matched_by(reached) {
        Outcome::Pass
    } else {
        Outcome::Fail
    }
}

#[test]
fn the_corpus_reaches_its_declared_verdict_through_the_ordinary_compiler_path() {
    let cases = corpus::load();
    assert!(
        !cases.is_empty(),
        "the conformance manifest declared no case"
    );
    let mut tally: BTreeMap<Outcome, usize> = BTreeMap::new();
    let mut reports = Vec::new();
    for case in &cases {
        let reached = reach(case);
        let outcome = outcome(case, &reached.verdict);
        *tally.entry(outcome).or_default() += 1;
        if outcome == Outcome::Fail {
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
    let failed = tally.get(&Outcome::Fail).copied().unwrap_or_default();
    assert_eq!(failed, 0, "conformance adapter: {summary}");
}
