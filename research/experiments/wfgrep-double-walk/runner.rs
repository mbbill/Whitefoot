//! WFGREP-DOUBLE-WALK runner: output-identity verification, null-comparison
//! precision demonstration, paired end-to-end measurement, and hardware-
//! counter capture for the candidate source shapes against the fresh
//! current-bytes baseline (and the pinned system `grep -h -F` for the
//! fresh-baseline and confirmation phases).
//!
//! The corpus, digests, statistics, and noise controls are inherited from
//! WFGREP-BASELINE; corpus generation code is identical so the pinned
//! manifest verifies unchanged. Phases are subcommands invoked by the bundle
//! Makefile in protocol order. Every observation appends one JSON line to
//! the raw evidence file named by `WFD_RAW`. Samples are never deleted,
//! extended, or rerun after a result is observed.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const CORPUS_LARGE_TARGET: u64 = 268_435_456;
const CORPUS_DENSE_TARGET: u64 = 134_217_728;
const CORPUS_MANY_FILES: usize = 4096;
const CORPUS_MANY_TARGET: u64 = 16_384;
const NEEDLE: &str = "XQWFNEEDLE";
const ABSENT: &str = "XQWFABSENT";
const ROUNDS: usize = 30;
const COUNTER_REPETITIONS: usize = 5;
const BOOTSTRAP_RESAMPLES: usize = 10_000;
const BOOTSTRAP_SEED: u64 = 20_260_806;

/// Every locally built subject binary, in build order. `base` is the fresh
/// baseline built from the current `tests/programs/wfgrep.wf` bytes.
const SUBJECTS: &[&str] = &["base", "s1", "s2", "s3"];

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.first().map(String::as_str) {
        Some("gen") => generate(),
        Some("verify") => verify(),
        Some("null") => null_phase(&arguments),
        Some("bench-grep") => bench(&"grep".to_owned()),
        Some("bench-shape") => match arguments.get(1) {
            Some(shape) => bench(shape),
            None => Err("bench-shape requires a shape name".to_owned()),
        },
        Some("confirm") => match arguments.get(1) {
            Some(shape) => confirm(shape),
            None => Err("confirm requires a shape name".to_owned()),
        },
        Some("counters") => counters(),
        _ => Err(
            "usage: runner gen | verify | null TAG | bench-grep | bench-shape NAME | confirm NAME | counters"
                .to_owned(),
        ),
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

fn raw_path() -> PathBuf {
    PathBuf::from(required_env("WFD_RAW"))
}

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("environment variable {name} must be set"))
}

/// One frozen measurement case: a pattern and an explicit relative file list.
struct Case {
    name: &'static str,
    pattern: &'static str,
    files: Vec<String>,
    /// Whether the case participates in classification. The floor case is a
    /// process-startup diagnostic and is never classified.
    classified: bool,
}

fn cases() -> Vec<Case> {
    let many: Vec<String> = (0..CORPUS_MANY_FILES)
        .map(|index| format!("many/f{index:04}.txt"))
        .collect();
    vec![
        Case { name: "large", pattern: NEEDLE, files: vec!["large.txt".into()], classified: true },
        Case { name: "nomatch", pattern: ABSENT, files: vec!["large.txt".into()], classified: true },
        Case { name: "dense", pattern: NEEDLE, files: vec!["dense.txt".into()], classified: true },
        Case { name: "many", pattern: NEEDLE, files: many, classified: true },
        Case { name: "floor", pattern: NEEDLE, files: vec!["floor/empty.txt".into()], classified: false },
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
    emit(&format!(
        "{{\"record\":\"gen\",\"time\":{},\"large_lines\":{large_lines},\"large_matches\":{large_matches},\
         \"dense_lines\":{dense_lines},\"dense_matches\":{dense_matches},\
         \"many_lines\":{many_lines},\"many_matches\":{many_matches}}}",
        unix_time()
    ));
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

/// Verifies corpus digests against the inherited pinned manifest, records
/// every subject's binary identity, then per subject and case verifies
/// byte-identical stdout and exit codes against the pinned output digests.
fn verify() -> Result<(), String> {
    let manifest = std::fs::read_to_string(manifest_path()).map_err(|error| error.to_string())?;
    let pinned: Vec<&str> = manifest.lines().filter(|line| !line.is_empty()).collect();
    let computed = compute_corpus_manifest()?;
    for line in &computed {
        if !pinned.contains(&line.as_str()) {
            return Err(format!("corpus digest mismatch: computed `{line}` is not pinned"));
        }
    }
    let mut identities = String::new();
    for name in SUBJECTS.iter().chain(["grep"].iter()) {
        identities.push_str(&format!(
            "\"{name}_sha256\":\"{}\",",
            sha256(&subject_binary(name))?
        ));
    }
    emit(&format!(
        "{{\"record\":\"identity\",\"time\":{},{identities}\"manifest\":\"{}\"}}",
        unix_time(),
        manifest_path().display(),
    ));
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
            emit(&format!(
                "{{\"record\":\"verify\",\"time\":{},\"subject\":\"{name}\",\"case\":\"{}\",\
                 \"bytes\":{},\"exit\":{exit},\"sha256\":\"{digest}\"}}",
                unix_time(),
                case.name,
                output.len(),
            ));
        }
    }
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

// --- timing --------------------------------------------------------------

/// Runs one timed invocation: stdout to /dev/null, stderr captured and
/// required empty, elapsed wall time from spawn to exit.
fn timed_run(binary: &Path, case: &Case, leading: &[&str]) -> Result<(u64, i32), String> {
    let mut command = Command::new(binary);
    command
        .current_dir(corpus_root())
        .env_clear()
        .env("LC_ALL", "C")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command.args(leading).arg(case.pattern).args(&case.files);
    let start = Instant::now();
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let mut stderr = Vec::new();
    child
        .stderr
        .take()
        .ok_or("missing stderr pipe")?
        .read_to_end(&mut stderr)
        .map_err(|error| error.to_string())?;
    let status = child.wait().map_err(|error| error.to_string())?;
    let elapsed = start.elapsed().as_nanos() as u64;
    if !stderr.is_empty() {
        return Err(format!(
            "case {}: unexpected stderr: {}",
            case.name,
            String::from_utf8_lossy(&stderr)
        ));
    }
    let code = status.code().unwrap_or(-1);
    let expected = if case.name == "nomatch" || case.name == "floor" { 1 } else { 0 };
    if code != expected {
        return Err(format!("case {}: exit {code}, expected {expected}", case.name));
    }
    Ok((elapsed, code))
}

/// Reads every case file completely so both timed positions in a round see
/// the same warm page-cache state.
fn warm(case: &Case) -> Result<u64, String> {
    let root = corpus_root();
    let mut buffer = vec![0u8; 1 << 20];
    let mut total = 0u64;
    for name in &case.files {
        let mut file = std::fs::File::open(root.join(name)).map_err(|error| error.to_string())?;
        loop {
            let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            total += read as u64;
        }
    }
    Ok(total)
}

/// One paired phase: the reference subject against one other subject, 30
/// order-alternating warm rounds per case.
///
/// Ratio orientation per phase, fixed by the protocol:
/// - null and bench-grep: ratio = other / reference where the reference is
///   B0 (`base`); in bench-grep the "other" is grep, so below 1.0 means the
///   baseline is slower — the WFGREP-BASELINE orientation.
/// - bench-shape: the reference is the shape, the "other" is B0, so the
///   ratio (B0 / shape) above 1.0 means the shape is faster.
/// - confirm: the reference is the shape, the "other" is grep — the
///   baseline orientation with the shape as subject.
fn paired_phase(phase: &str, reference: &str, other: &str) -> Result<(), String> {
    record_power(phase, "start");
    let reference_binary = subject_binary(reference);
    let other_binary = subject_binary(other);
    let reference_leading = leading_arguments(reference);
    let other_leading = leading_arguments(other);
    for case in cases() {
        for _ in 0..3 {
            timed_run(&reference_binary, &case, &reference_leading)?;
            if other != reference {
                timed_run(&other_binary, &case, &other_leading)?;
            }
        }
        let mut ratios = Vec::with_capacity(ROUNDS);
        let mut elapsed_by_side: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        let mut position_medians: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        for round in 0..ROUNDS {
            warm(&case)?;
            // Side 0 is the reference, side 1 the other; the reference runs
            // first on even rounds.
            let reference_first = round % 2 == 0;
            let sample = |side: usize, position: usize| -> Result<u64, String> {
                let (binary, leading, label) = if side == 0 {
                    (&reference_binary, &reference_leading, reference)
                } else {
                    (&other_binary, &other_leading, other)
                };
                let (elapsed, exit) = timed_run(binary, &case, leading)?;
                emit(&format!(
                    "{{\"record\":\"sample\",\"phase\":\"{phase}\",\"case\":\"{}\",\"round\":{round},\
                     \"position\":{position},\"side\":{side},\"binary\":\"{label}\",\
                     \"elapsed_ns\":{elapsed},\"exit\":{exit}}}",
                    case.name,
                ));
                Ok(elapsed)
            };
            let (first_side, second_side) = if reference_first { (0, 1) } else { (1, 0) };
            let first = sample(first_side, 1)? as f64;
            let second = sample(second_side, 2)? as f64;
            let (reference_elapsed, other_elapsed) =
                if reference_first { (first, second) } else { (second, first) };
            elapsed_by_side[0].push(reference_elapsed);
            elapsed_by_side[1].push(other_elapsed);
            position_medians[usize::from(!reference_first)].push(other_elapsed / reference_elapsed);
            ratios.push(other_elapsed / reference_elapsed);
        }
        let summary = summarize(&ratios);
        let reference_median = median(&mut elapsed_by_side[0].clone());
        let other_median = median(&mut elapsed_by_side[1].clone());
        let reference_first_median = median(&mut position_medians[0].clone());
        let other_first_median = median(&mut position_medians[1].clone());
        emit(&format!(
            "{{\"record\":\"summary\",\"phase\":\"{phase}\",\"case\":\"{}\",\"rounds\":{ROUNDS},\
             \"reference\":\"{reference}\",\"other\":\"{other}\",\
             \"reference_median_ns\":{:.0},\"other_median_ns\":{:.0},\
             \"ratio_median\":{:.6},\"ci_low\":{:.6},\"ci_high\":{:.6},\"relative_half_width\":{:.6},\
             \"reference_first_ratio_median\":{:.6},\"other_first_ratio_median\":{:.6},\
             \"classified\":{}}}",
            case.name,
            reference_median,
            other_median,
            summary.point,
            summary.low,
            summary.high,
            summary.relative_half_width,
            reference_first_median,
            other_first_median,
            case.classified,
        ));
    }
    record_power(phase, "end");
    Ok(())
}

fn null_phase(arguments: &[String]) -> Result<(), String> {
    let tag = arguments.get(1).ok_or("null requires a tag argument")?;
    paired_phase(&format!("null-{tag}"), "base", "base")
}

fn bench(other: &str) -> Result<(), String> {
    if other == "grep" {
        // Fresh same-protocol baseline: reference B0, other grep, so the
        // ratio keeps the WFGREP-BASELINE orientation (grep / B0).
        paired_phase("bench-grep", "base", "grep")
    } else {
        if !SUBJECTS.contains(&other) {
            return Err(format!("unknown shape {other}"));
        }
        // Shape phases: reference is the shape, other is B0, so the ratio
        // is B0 / shape — above 1.0 means the shape is faster.
        paired_phase(&format!("bench-{other}"), other, "base")
    }
}

fn confirm(shape: &str) -> Result<(), String> {
    if !SUBJECTS.contains(&shape) {
        return Err(format!("unknown shape {shape}"));
    }
    paired_phase(&format!("confirm-{shape}"), shape, "grep")
}

struct Summary {
    point: f64,
    low: f64,
    high: f64,
    relative_half_width: f64,
}

fn summarize(ratios: &[f64]) -> Summary {
    let point = median(&mut ratios.to_vec());
    let mut state = XorShift(BOOTSTRAP_SEED);
    let mut resampled = Vec::with_capacity(BOOTSTRAP_RESAMPLES);
    for _ in 0..BOOTSTRAP_RESAMPLES {
        let mut draw: Vec<f64> = (0..ratios.len())
            .map(|_| ratios[(state.next() % ratios.len() as u64) as usize])
            .collect();
        resampled.push(median(&mut draw));
    }
    resampled.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let low = resampled[(0.025 * (BOOTSTRAP_RESAMPLES - 1) as f64).round() as usize];
    let high = resampled[(0.975 * (BOOTSTRAP_RESAMPLES - 1) as f64).round() as usize];
    Summary { point, low, high, relative_half_width: (high - low) / (2.0 * point) }
}

fn median(values: &mut Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = values.len();
    if n % 2 == 1 { values[n / 2] } else { (values[n / 2 - 1] + values[n / 2]) / 2.0 }
}

// --- counters ------------------------------------------------------------

/// Captures `/usr/bin/time -l` accounting (real/user/sys, instructions
/// retired, cycles elapsed, peak footprint) for every subject on every case.
fn counters() -> Result<(), String> {
    record_power("counters", "start");
    for case in cases() {
        warm(&case)?;
        for name in SUBJECTS.iter().chain(["grep"].iter()) {
            let binary = subject_binary(name);
            let leading = leading_arguments(name);
            for repetition in 0..COUNTER_REPETITIONS {
                let mut command = Command::new("/usr/bin/time");
                command
                    .arg("-l")
                    .arg(&binary)
                    .args(&leading)
                    .arg(case.pattern)
                    .args(&case.files)
                    .current_dir(corpus_root())
                    .env_clear()
                    .env("LC_ALL", "C")
                    .stdout(Stdio::null());
                let output = command.output().map_err(|error| error.to_string())?;
                let text = String::from_utf8_lossy(&output.stderr).into_owned();
                emit(&format!(
                    "{{\"record\":\"counters\",\"case\":\"{}\",\"binary\":\"{name}\",\"repetition\":{repetition},\
                     \"real\":{},\"user\":{},\"sys\":{},\"instructions\":{},\"cycles\":{},\"peak_footprint\":{}}}",
                    case.name,
                    field(&text, "real"),
                    field(&text, "user"),
                    field(&text, "sys"),
                    field(&text, "instructions retired"),
                    field(&text, "cycles elapsed"),
                    field(&text, "peak memory footprint"),
                ));
            }
        }
    }
    record_power("counters", "end");
    Ok(())
}

fn field(text: &str, name: &str) -> String {
    for line in text.lines() {
        if let Some(position) = line.find(name) {
            let value = line[..position].split_whitespace().last().unwrap_or("0");
            return value.to_owned();
        }
    }
    "0".to_owned()
}

// --- evidence ------------------------------------------------------------

fn emit(line: &str) {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(raw_path())
        .expect("open raw evidence file");
    writeln!(file, "{line}").expect("append raw evidence");
}

fn record_power(phase: &str, event: &str) {
    let batt = Command::new("/usr/bin/pmset")
        .args(["-g", "batt"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).replace('\n', " ").replace('"', "'"))
        .unwrap_or_default();
    let modes = Command::new("/usr/bin/pmset")
        .args(["-g"])
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .find(|line| line.contains("lowpowermode"))
                .unwrap_or("lowpowermode unknown")
                .trim()
                .to_owned()
        })
        .unwrap_or_default();
    emit(&format!(
        "{{\"record\":\"power\",\"phase\":\"{phase}\",\"event\":\"{event}\",\"time\":{},\"battery\":\"{}\",\"mode\":\"{}\"}}",
        unix_time(),
        batt.trim(),
        modes,
    ));
}

fn unix_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}
