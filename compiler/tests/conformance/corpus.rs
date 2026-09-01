//! The typed reading of `tests/conformance/manifest.jsonl`.
//!
//! The corpus owns its own schema; this module only restates it in Rust types
//! so the adapter can act on it. It deliberately re-derives nothing: an
//! unreadable line, an unknown expectation kind, or an unknown `arrange` key
//! is an error here rather than a skipped case, because a corpus entry the
//! adapter cannot act on must not silently disappear from the run.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::json::{self, Value};

/// The verdict space the corpus fixes:
/// `accept | reject(rule) | run(exit) | unsupported`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Verdict {
    /// The unit is a complete accepted program.
    Accept,
    /// The unit violates the named numbered rule, or an unnamed one.
    Reject(Option<String>),
    /// The program ran to completion with this command status.
    Run(i32),
    /// The specification's own non-rejection stop [QUAL-1, QUAL-2, PROG-3].
    Unsupported(String),
    /// Not one of the corpus's verdicts: the compiler stopped for a reason
    /// the corpus does not model, such as an internal invariant or a resource
    /// ceiling. Kept distinct so such a stop can never be reported as a
    /// language judgment.
    Stopped(String),
}

/// The expectation one manifest line declares.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expectation {
    /// `{"kind":"accept"}`
    Accept,
    /// `{"kind":"reject","rule":R}`
    Reject(String),
    /// `{"kind":"run","exit":N}`
    Run(i32),
    /// `{"kind":"unsupported","why":W}`
    Unsupported,
}

impl Expectation {
    /// Whether satisfying this expectation requires running the program.
    pub const fn needs_execution(&self) -> bool {
        matches!(self, Self::Run(_))
    }

    /// The corpus's own match rule, restated exactly as `runner.py` states it:
    /// a rejection must cite the declared rule, a run must reach the declared
    /// status, and an `unsupported` expectation asserts the verdict kind only,
    /// never a diagnostic's wording.
    pub fn matched_by(&self, verdict: &Verdict) -> bool {
        match (self, verdict) {
            (Self::Accept, Verdict::Accept) => true,
            (Self::Reject(rule), Verdict::Reject(Some(cited))) => rule == cited,
            (Self::Run(exit), Verdict::Run(reached)) => exit == reached,
            (Self::Unsupported, Verdict::Unsupported(_)) => true,
            _ => false,
        }
    }
}

/// The toolchain-readiness axis, separate from the expectation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// Must match the expectation.
    Runnable,
    /// This toolchain cannot reach the case yet; skipped.
    Pending,
    /// The expectation is the correct spec behaviour and this toolchain does
    /// not yet produce it; reported, non-failing, and flagged if it starts
    /// matching.
    Xfail,
}

/// One fixture object placed under the invocation's initial directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fixture {
    /// The path's exact bytes, relative to the initial directory.
    pub path: Vec<u8>,
    /// A regular file's exact contents, or `None` for a directory.
    pub bytes: Option<Vec<u8>>,
}

/// The invocation one `run` case needs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Arrangement {
    /// The complete native argument vector, position 0 included.
    pub argv: Option<Vec<Vec<u8>>>,
    /// The standard input body; absent means an empty standard input.
    pub stdin: Option<Vec<u8>>,
    /// Fixtures created under the initial directory before the invocation.
    pub files: Vec<Fixture>,
    /// Opaque sink labels for the redirected streams.
    pub redirect: BTreeMap<String, String>,
}

/// One corpus case.
#[derive(Clone, Debug)]
pub struct Case {
    /// The case id, which also names its `.wf` source.
    pub id: String,
    /// The verdict the specification requires.
    pub expect: Expectation,
    /// The toolchain-readiness axis.
    pub status: Status,
    /// The reason a `pending` or `xfail` case carries.
    pub reason: Option<String>,
    /// The invocation arrangement, present only on an executed case.
    pub arrange: Option<Arrangement>,
}

impl Case {
    /// Reads this case's canonical source bytes.
    pub fn source(&self) -> Vec<u8> {
        let path = cases_directory().join(format!("{}.wf", self.id));
        std::fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read case source {}: {error}", path.display()))
    }

    /// The logical path the compiler sees, which is the case's own file name.
    pub fn logical_path(&self) -> String {
        format!("{}.wf", self.id)
    }
}

/// The conformance corpus directory, reached from the compiler package.
pub fn corpus_directory() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler package must live directly under the repository root")
        .join("tests")
        .join("conformance")
}

fn cases_directory() -> PathBuf {
    corpus_directory().join("cases")
}

/// Reads every case line of the manifest, ignoring coverage annotations.
///
/// Annotations carry `covered_by` instead of `id` and describe corpus
/// coverage rather than a program, so they have no verdict to check.
pub fn load() -> Vec<Case> {
    let path = corpus_directory().join("manifest.jsonl");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let mut cases = Vec::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let value = json::parse(line)
            .unwrap_or_else(|error| panic!("manifest line {}: {error}", number + 1));
        if value.get("id").is_none() {
            continue;
        }
        cases.push(
            case(&value).unwrap_or_else(|error| panic!("manifest line {}: {error}", number + 1)),
        );
    }
    cases
}

fn case(value: &Value) -> Result<Case, String> {
    let id = value
        .get("id")
        .and_then(Value::string)
        .ok_or("id must be a string")?
        .to_owned();
    let expect = expectation(value.get("expect").ok_or("missing expect")?)?;
    let status = match value.get("status").and_then(Value::string) {
        None | Some("runnable") => Status::Runnable,
        Some("pending") => Status::Pending,
        Some("xfail") => Status::Xfail,
        Some(other) => return Err(format!("{id}: invalid status {other:?}")),
    };
    let reason = value
        .get("reason")
        .and_then(Value::string)
        .map(ToOwned::to_owned);
    let arrange = value.get("arrange").map(arrangement).transpose()?;
    Ok(Case {
        id,
        expect,
        status,
        reason,
        arrange,
    })
}

fn expectation(value: &Value) -> Result<Expectation, String> {
    match value.get("kind").and_then(Value::string) {
        Some("accept") => Ok(Expectation::Accept),
        Some("reject") => value
            .get("rule")
            .and_then(Value::string)
            .map(|rule| Expectation::Reject(rule.to_owned()))
            .ok_or_else(|| "reject expectation needs a rule".to_owned()),
        Some("run") => value
            .get("exit")
            .and_then(Value::integer)
            .and_then(|exit| i32::try_from(exit).ok())
            .map(Expectation::Run)
            .ok_or_else(|| "run expectation needs an integer exit".to_owned()),
        Some("unsupported") => Ok(Expectation::Unsupported),
        other => Err(format!("invalid expectation kind {other:?}")),
    }
}

fn arrangement(value: &Value) -> Result<Arrangement, String> {
    let argv = value
        .get("argv")
        .map(|argv| {
            argv.array()
                .ok_or_else(|| "argv must be an array".to_owned())?
                .iter()
                .map(hex_bytes)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    let stdin = value.get("stdin").map(hex_bytes).transpose()?;
    let files = value
        .get("files")
        .map(|files| {
            files
                .array()
                .ok_or_else(|| "files must be an array".to_owned())?
                .iter()
                .map(fixture)
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let mut redirect = BTreeMap::new();
    if let Some(Value::Object(streams)) = value.get("redirect") {
        for (stream, sink) in streams {
            let sink = sink
                .string()
                .ok_or_else(|| format!("redirect {stream} needs a sink name"))?;
            redirect.insert(stream.clone(), sink.to_owned());
        }
    }
    Ok(Arrangement {
        argv,
        stdin,
        files,
        redirect,
    })
}

fn fixture(value: &Value) -> Result<Fixture, String> {
    let path = hex_bytes(value.get("path").ok_or("fixture needs a path")?)?;
    let bytes = match (value.get("bytes"), value.get("directory")) {
        (Some(content), None) => Some(hex_bytes(content)?),
        (None, Some(Value::True)) => None,
        _ => return Err("fixture needs exactly one of bytes, directory".to_owned()),
    };
    Ok(Fixture { path, bytes })
}

/// Decodes one manifest byte string.
///
/// Every byte string in the corpus is lowercase hex so a non-UTF-8 argument
/// or path is stated exactly rather than through an encoding the reader has
/// to guess.
fn hex_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let text = value.string().ok_or("byte string must be a string")?;
    if text.len() % 2 != 0 {
        return Err(format!("byte string {text:?} has an odd length"));
    }
    text.as_bytes()
        .chunks(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).map_err(|error| error.to_string())?;
            if !digits
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(format!("byte string {text:?} is not lowercase hex"));
            }
            u8::from_str_radix(digits, 16).map_err(|error| error.to_string())
        })
        .collect()
}
