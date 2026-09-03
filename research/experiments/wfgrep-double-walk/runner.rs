//! WFGREP-DOUBLE-WALK replay check: corpus generation identical to
//! WFGREP-BASELINE, then byte-identity verification of the three kept shape
//! sources (and the pinned system `grep -h -F`) against the inherited
//! manifest.
//!
//! The single run of 2026-08-06 (`RESULTS.md`) also ran null, bench,
//! counters, and confirm phases paired against B0, the then-current
//! `tests/programs/wfgrep.wf`. That program has since become a different
//! program (a recursive search printing `PATH:LINE:TEXT` lines) and its
//! frozen bytes predate specification v0.40, so B0 cannot be rebuilt from
//! HEAD and those phases live only in the freeze commit `RESULTS.md` names.
//! What HEAD can still assert is here: the shapes, kept on the active
//! specification, still produce the pinned bytes and exit codes on the
//! pinned corpus.
//!
//! Phases are subcommands invoked by the bundle Makefile: `gen`, then
//! `verify`. Results go to stdout; nothing here appends to the committed
//! raw evidence of the frozen run.

use std::path::{Path, PathBuf};
use std::process::Command;

const CORPUS_LARGE_TARGET: u64 = 268_435_456;
const CORPUS_DENSE_TARGET: u64 = 134_217_728;
const CORPUS_MANY_FILES: usize = 4096;
const CORPUS_MANY_TARGET: u64 = 16_384;
const NEEDLE: &str = "XQWFNEEDLE";
const ABSENT: &str = "XQWFABSENT";

/// Every locally built subject binary, in build order: the three shape
/// sources under `shapes/`.
const SUBJECTS: &[&str] = &["s1", "s2", "s3"];

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.first().map(String::as_str) {
        Some("gen") => generate(),
        Some("verify") => verify(),
        _ => Err("usage: runner gen | verify".to_owned()),
    };
    if let Err(message) = result {
        eprintln!("runner: {message}");
        std::process::exit(1);
    }
}

// --- configuration -------------------------------------------------------

fn work_root() -> PathBuf {
    PathBuf::from(required_env("WFD_WORK_ROOT"))
}

fn corpus_root() -> PathBuf {
    work_root().join("corpus")
}

fn subject_binary(name: &str) -> PathBuf {
    if name == "grep" {
        PathBuf::from("/usr/bin/grep")
    } else {
        work_root().join("build").join(name)
    }
}

fn leading_arguments(name: &str) -> Vec<&'static str> {
    if name == "grep" { vec!["-h", "-F"] } else { Vec::new() }
}

fn manifest_path() -> PathBuf {
    PathBuf::from(required_env("WFD_MANIFEST"))
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("environment variable {name} must be set"))
}

/// One frozen measurement case: a pattern and an explicit relative file list.
struct Case {
    name: &'static str,
    pattern: &'static str,
    files: Vec<String>,
}

fn cases() -> Vec<Case> {
    let many: Vec<String> = (0..CORPUS_MANY_FILES)
        .map(|index| format!("many/f{index:04}.txt"))
        .collect();
    vec![
        Case { name: "large", pattern: NEEDLE, files: vec!["large.txt".into()] },
        Case { name: "nomatch", pattern: ABSENT, files: vec!["large.txt".into()] },
        Case { name: "dense", pattern: NEEDLE, files: vec!["dense.txt".into()] },
        Case { name: "many", pattern: NEEDLE, files: many },
        Case { name: "floor", pattern: NEEDLE, files: vec!["floor/empty.txt".into()] },
    ]
}

// --- corpus generation (identical to WFGREP-BASELINE) --------------------

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// Appends one generated line (60 to 120 lowercase/space bytes plus a
/// newline); when `needle` is set, its bytes overwrite columns 20..30.
fn push_line(output: &mut Vec<u8>, state: &mut XorShift, needle: Option<&str>) {
    let length = 60 + (state.next() % 61) as usize;
    let start = output.len();
    for _ in 0..length {
        let value = (state.next() % 27) as u8;
        output.push(if value == 26 { b' ' } else { b'a' + value });
    }
    if let Some(pattern) = needle {
        output[start + 20..start + 20 + pattern.len()].copy_from_slice(pattern.as_bytes());
    }
    output.push(b'\n');
}

/// Generates one file of terminated lines until `target` bytes are reached,
/// injecting the needle on lines whose index satisfies the case rule.
fn generate_file(path: &Path, seed: u64, target: u64, inject: impl Fn(u64) -> bool) -> (u64, u64) {
    let mut state = XorShift(seed);
    let mut content = Vec::with_capacity(target as usize + 256);
    let mut lines = 0u64;
    let mut matches = 0u64;
    while (content.len() as u64) < target {
        let needle = inject(lines);
        push_line(&mut content, &mut state, needle.then_some(NEEDLE));
        if needle {
            matches += 1;
        }
        lines += 1;
    }
    std::fs::write(path, &content).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
    (lines, matches)
}

fn generate() -> Result<(), String> {
    let root = corpus_root();
    std::fs::create_dir_all(root.join("many")).map_err(|error| error.to_string())?;
    std::fs::create_dir_all(root.join("floor")).map_err(|error| error.to_string())?;
    let (large_lines, large_matches) =
        generate_file(&root.join("large.txt"), 0x1000_0001, CORPUS_LARGE_TARGET, |line| line % 1024 == 512);
    let (dense_lines, dense_matches) =
        generate_file(&root.join("dense.txt"), 0x2000_0002, CORPUS_DENSE_TARGET, |line| line % 2 == 1);
    let mut many_lines = 0u64;
    let mut many_matches = 0u64;
    for index in 0..CORPUS_MANY_FILES {
        let seed = 0x3000_0000u64 + (index as u64).wrapping_mul(0x9E37_79B9);
        let inject_first = index % 8 == 0;
        let (lines, matches) = generate_file(
            &root.join(format!("many/f{index:04}.txt")),
            seed,
            CORPUS_MANY_TARGET,
            |line| inject_first && line == 0,
        );
        many_lines += lines;
        many_matches += matches;
    }
    std::fs::write(root.join("floor/empty.txt"), b"").map_err(|error| error.to_string())?;
    println!(
        "gen: large {large_lines} lines / {large_matches} matches, \
         dense {dense_lines} lines / {dense_matches} matches, \
         many {many_lines} lines / {many_matches} matches"
    );
    Ok(())
}

/// Computes the pinned corpus digest lines exactly as the baseline runner
/// did, so the inherited manifest verifies unchanged.
fn compute_corpus_manifest() -> Result<Vec<String>, String> {
    let root = corpus_root();
    let mut lines = Vec::new();
    for name in ["large.txt", "dense.txt", "floor/empty.txt"] {
        lines.push(format!("corpus {} {name}", sha256(&root.join(name))?));
    }
    let mut combined = String::new();
    for index in 0..CORPUS_MANY_FILES {
        let name = format!("many/f{index:04}.txt");
        combined.push_str(&format!("{} {name}\n", sha256(&root.join(&name))?));
    }
    let listing = work_root().join("build").join("many-digests.txt");
    std::fs::write(&listing, combined).map_err(|error| error.to_string())?;
    lines.push(format!("corpus-many {} many", sha256(&listing)?));
    Ok(lines)
}

fn sha256(path: &Path) -> Result<String, String> {
    let output = Command::new("/usr/bin/shasum")
        .args(["-a", "256"])
        .arg(path)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!("shasum failed for {}", path.display()));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.split_whitespace().next().unwrap_or("").to_owned())
}

fn sha256_bytes(bytes: &[u8]) -> Result<String, String> {
    let path = work_root().join("build").join("digest-scratch");
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
    sha256(&path)
}

// --- verification --------------------------------------------------------

/// Verifies corpus digests against the inherited pinned manifest, reports
/// every subject's binary identity, then per subject and case verifies
/// byte-identical stdout and exit codes against the pinned output digests.
/// A stale or partial corpus fails at the first step; `make gen` rebuilds it.
fn verify() -> Result<(), String> {
    let manifest = std::fs::read_to_string(manifest_path()).map_err(|error| error.to_string())?;
    let pinned: Vec<&str> = manifest.lines().filter(|line| !line.is_empty()).collect();
    let computed = compute_corpus_manifest()?;
    for line in &computed {
        if !pinned.contains(&line.as_str()) {
            return Err(format!("corpus digest mismatch: computed `{line}` is not pinned"));
        }
    }
    println!("corpus: {} digest lines match the inherited manifest", computed.len());
    for name in SUBJECTS.iter().chain(["grep"].iter()) {
        println!("identity: {name} sha256 {}", sha256(&subject_binary(name))?);
    }
    for name in SUBJECTS.iter().chain(["grep"].iter()) {
        for case in cases() {
            let (output, exit) = captured_run(&subject_binary(name), &case, &leading_arguments(name))?;
            let digest = sha256_bytes(&output)?;
            let output_line = format!("output {digest} {}", case.name);
            let exit_line = format!("exit {exit} {}", case.name);
            for required in [&output_line, &exit_line] {
                if !pinned.contains(&required.as_str()) {
                    return Err(format!(
                        "subject {name}, case {}: `{required}` is not pinned",
                        case.name
                    ));
                }
            }
            println!(
                "verify: {name} {} {} bytes, exit {exit}, sha256 {digest}",
                case.name,
                output.len()
            );
        }
    }
    println!("verify: every subject and case matches the pinned outputs and exit codes");
    Ok(())
}

fn captured_run(binary: &Path, case: &Case, leading: &[&str]) -> Result<(Vec<u8>, i32), String> {
    let mut command = Command::new(binary);
    command.current_dir(corpus_root()).env_clear().env("LC_ALL", "C");
    command.args(leading).arg(case.pattern).args(&case.files);
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.stderr.is_empty() {
        return Err(format!(
            "case {}: unexpected stderr from {}: {}",
            case.name,
            binary.display(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok((output.stdout, output.status.code().unwrap_or(-1)))
}
