#![forbid(unsafe_code)]

//! The driver for `tests/snapshot/`, the recorded-verdict corpus.
//!
//! `tests/snapshot/index.tsv` is the corpus: one row per program, each naming
//! the verdict this compiler reached when the row was written. This file only
//! reaches those verdicts again and reports the ones that moved. It decides
//! nothing about the language — `tests/snapshot/README.md` states what the
//! corpus is and, just as importantly, what it is not: a snapshot of one
//! compiler, outside the conformance boundary, with no specification
//! authority.
//!
//! Two deliberate narrowings keep it honest. It compares `accept` against
//! `reject` and nothing else, because the numbered rule a rejection cites is a
//! diagnostic choice that must stay free to improve; the index carries the
//! rule for a reader, never for this comparison. And every case reaches its
//! verdict through the ordinary compiler path with no link and no execution,
//! so a row is a statement about the checker rather than about a host
//! toolchain.
//!
//! A rejection reached before resolution — a lexical, grammatical, or
//! [FORM-2] canonical-source stop — records nothing about the checker, so it
//! is reported rather than counted as a reject: the corpus admits only cases
//! that get far enough to have a semantic verdict at all.
//!
//! `#[ignore]`d for cost, exactly as the conformance adapter is: it compiles
//! every case in the corpus, so it stays out of the default `cargo test` run
//! and `make snapshot-run` invokes it with `--ignored`. Root `make check`
//! includes that target. The wiring and the attribute are one unit.

use std::path::{Path, PathBuf};

use whitefoot::{CompilationFailureKind, CompilationStage, CompilerLimits, SourceInput, compile};

/// The two verdicts this corpus records, and the only comparison it makes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Verdict {
    Accept,
    Reject,
}

impl Verdict {
    fn parse(text: &str) -> Self {
        match text {
            "accept" => Self::Accept,
            "reject" => Self::Reject,
            other => panic!("index verdict must be accept or reject, found {other:?}"),
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
        }
    }
}

/// One index row, reduced to what the comparison needs.
struct Row {
    id: String,
    family: String,
    verdict: Verdict,
}

/// What the compiler actually did with one case.
enum Reached {
    /// A semantic verdict, with the first diagnostic line behind a rejection.
    Verdict(Verdict, Option<String>),
    /// A stop that is not a verdict about this program's semantics.
    NotAVerdict(String),
}

fn snapshot_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler package must live directly under the repository root")
        .join("tests")
        .join("snapshot")
}

/// Reads the index in file order; that order is the run order.
fn load() -> Vec<Row> {
    let path = snapshot_directory().join("index.tsv");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let mut rows = Vec::new();
    for (number, line) in text.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            7,
            "index line {} has {} fields, expected 7",
            number + 1,
            fields.len()
        );
        rows.push(Row {
            id: fields[0].to_owned(),
            family: fields[1].to_owned(),
            verdict: Verdict::parse(fields[2]),
        });
    }
    rows
}

/// Drives one case through the ordinary compiler path, without linking it.
fn reach(row: &Row) -> Reached {
    let path = snapshot_directory()
        .join("cases")
        .join(&row.family)
        .join(format!("{}.wf", row.id));
    let source = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("cannot read case source {}: {error}", path.display()));
    let logical = format!("{}.wf", row.id);
    match compile(
        &[SourceInput::new(&logical, &source)],
        CompilerLimits::default(),
    ) {
        Ok(_) => Reached::Verdict(Verdict::Accept, None),
        Err(failure) => {
            let detail = failure.to_string();
            let first = detail.lines().next().unwrap_or_default().to_owned();
            if failure.kind() != CompilationFailureKind::Source {
                return Reached::NotAVerdict(first);
            }
            // Before resolution the compiler has judged the bytes, not the
            // program. Such a stop is a corpus defect, not a reject.
            let semantic = !matches!(
                failure.stage(),
                CompilationStage::SourceEnvelope
                    | CompilationStage::Lexing
                    | CompilationStage::TerminalClassification
                    | CompilationStage::Parsing
                    | CompilationStage::Finalization
                    | CompilationStage::CanonicalSource
            );
            if semantic {
                Reached::Verdict(Verdict::Reject, Some(first))
            } else {
                Reached::NotAVerdict(first)
            }
        }
    }
}

#[test]
#[ignore = "Cost, not a blocker: this compiles every recorded case, so it stays out of \
            default `cargo test`. `make snapshot-run` invokes it with `--ignored`, and root \
            `make check` includes that target; removing the attribute without dropping \
            `--ignored` would select no test."]
fn every_recorded_case_still_reaches_its_recorded_verdict() {
    let rows = load();
    assert!(!rows.is_empty(), "the snapshot index declared no case");
    let mut passed = 0usize;
    let mut flips = Vec::new();
    for row in &rows {
        match reach(row) {
            Reached::Verdict(reached, _) if reached == row.verdict => passed += 1,
            Reached::Verdict(reached, note) => flips.push(format!(
                "  FLIP {} expected {} reached {}  {}",
                row.id,
                row.verdict.name(),
                reached.name(),
                note.unwrap_or_default()
            )),
            Reached::NotAVerdict(first) => flips.push(format!(
                "  FLIP {} expected {} reached no semantic verdict  {first}",
                row.id,
                row.verdict.name(),
            )),
        }
    }
    for flip in &flips {
        println!("{flip}");
    }
    let summary = format!("Pass={passed} Flip={}", flips.len());
    println!("snapshot corpus: {summary}");
    assert!(
        flips.is_empty(),
        "snapshot corpus: {summary}; a verdict that moved must be explained in the commit \
         message, and the row updated only with that explanation"
    );
}
