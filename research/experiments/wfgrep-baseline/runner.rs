//! WFGREP-BASELINE runner: corpus generation, digest pinning, output-identity
//! verification, null-comparison precision demonstration, paired end-to-end
//! measurement, and hardware-counter capture for the frozen sequential wfgrep
//! against the pinned system `grep -h -F`.
//!
//! Phases are subcommands invoked by the bundle Makefile in protocol order.
//! Every observation appends one JSON line to the raw evidence file named by
//! the `WFB_RAW` environment variable. Samples are never deleted, extended,
//! or rerun after a result is observed.

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

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let result = match arguments.first().map(String::as_str) {
        Some("gen") => generate(arguments.contains(&"--print-manifest".to_owned())),
        Some("verify") => verify(),
        Some("null") => timed_phase(&arguments, true),
        Some("bench") => timed_phase(&arguments, false),
        Some("counters") => counters(),
        Some("profile") => profile(&arguments),
        _ => Err("usage: runner gen [--print-manifest] | verify | null TAG | bench | counters | profile CASE REPS SECONDS".to_owned()),
    };
    if let Err(message) = result {
        eprintln!("runner: {message}");
        std::process::exit(1);
    }
}

// --- configuration -------------------------------------------------------

fn work_root() -> PathBuf {
    PathBuf::from(required_env("WFB_WORK_ROOT"))
}

fn corpus_root() -> PathBuf {
    work_root().join("corpus")
}

fn wfgrep_binary() -> PathBuf {
    work_root().join("build").join("wfgrep")
}

fn grep_binary() -> PathBuf {
    PathBuf::from("/usr/bin/grep")
}

fn manifest_path() -> PathBuf {
    PathBuf::from(required_env("WFB_MANIFEST"))
}

fn raw_path() -> PathBuf {
    PathBuf::from(required_env("WFB_RAW"))
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

// --- corpus generation ---------------------------------------------------

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

fn generate(print_manifest: bool) -> Result<(), String> {
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
    let manifest = compute_corpus_manifest()?;
    emit(&format!(
        "{{\"record\":\"gen\",\"time\":{},\"large_lines\":{large_lines},\"large_matches\":{large_matches},\
         \"dense_lines\":{dense_lines},\"dense_matches\":{dense_matches},\
         \"many_lines\":{many_lines},\"many_matches\":{many_matches}}}",
        unix_time()
    ));
    if print_manifest {
        for line in &manifest {
            println!("{line}");
        }
    }
    Ok(())
}

/// Computes the pinned corpus digest lines: one SHA-256 per single-file
/// corpus and one combined digest over the ordered per-file digest list of
/// the many-files corpus.
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

/// Verifies corpus digests against the committed manifest, then per case
/// verifies byte-identical stdout and equal exit codes between wfgrep and
/// the comparator, and pins the output digest and exit code.
fn verify() -> Result<(), String> {
    let manifest = std::fs::read_to_string(manifest_path()).map_err(|error| error.to_string())?;
    let pinned: Vec<&str> = manifest.lines().filter(|line| !line.is_empty()).collect();
    let computed = compute_corpus_manifest()?;
    for line in &computed {
        if !pinned.contains(&line.as_str()) {
            return Err(format!("corpus digest mismatch: computed `{line}` is not pinned"));
        }
    }
    emit(&format!(
        "{{\"record\":\"identity\",\"time\":{},\"wfgrep_sha256\":\"{}\",\"grep_sha256\":\"{}\"}}",
        unix_time(),
        sha256(&wfgrep_binary())?,
        sha256(&grep_binary())?,
    ));
    for case in cases() {
        let (wf_output, wf_exit) = captured_run(&wfgrep_binary(), &case, &[])?;
        let (grep_output, grep_exit) = captured_run(&grep_binary(), &case, &["-h", "-F"])?;
        if wf_output != grep_output {
            return Err(format!("case {}: stdout differs between wfgrep and grep", case.name));
        }
        if wf_exit != grep_exit {
            return Err(format!("case {}: exit {} vs {}", case.name, wf_exit, grep_exit));
        }
        let digest = sha256_bytes(&wf_output)?;
        let output_line = format!("output {digest} {}", case.name);
        let exit_line = format!("exit {wf_exit} {}", case.name);
        for required in [&output_line, &exit_line] {
            if !pinned.contains(&required.as_str()) {
                return Err(format!("case {}: `{required}` is not pinned", case.name));
            }
        }
        emit(&format!(
            "{{\"record\":\"verify\",\"time\":{},\"case\":\"{}\",\"bytes\":{},\"exit\":{wf_exit},\"sha256\":\"{digest}\"}}",
            unix_time(),
            case.name,
            wf_output.len(),
        ));
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

/// The null phase runs the wfgrep binary in both positions; the bench phase
/// runs wfgrep and the comparator with position alternating by round.
fn timed_phase(arguments: &[String], null: bool) -> Result<(), String> {
    let phase = if null {
        let tag = arguments.get(1).ok_or("null requires a tag argument")?;
        format!("null-{tag}")
    } else {
        "bench".to_owned()
    };
    record_power(&phase, "start");
    let wfgrep = wfgrep_binary();
    let grep = grep_binary();
    for case in cases() {
        // Warm the binaries themselves once per case before round 1.
        for _ in 0..3 {
            timed_run(&wfgrep, &case, &[])?;
            if !null {
                timed_run(&grep, &case, &["-h", "-F"])?;
            }
        }
        let mut ratios = Vec::with_capacity(ROUNDS);
        let mut first_by_binary: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        let mut second_by_binary: [Vec<f64>; 2] = [Vec::new(), Vec::new()];
        for round in 0..ROUNDS {
            warm(&case)?;
            // Position order: in null both are wfgrep; in bench, odd rounds
            // run wfgrep first, even rounds run the comparator first.
            let wf_first = null || round % 2 == 0;
            let sample = |binary_index: usize, position: usize| -> Result<u64, String> {
                let (binary, leading, label): (&Path, &[&str], &str) = if null || binary_index == 0 {
                    (&wfgrep, &[], "wfgrep")
                } else {
                    (&grep, &["-h", "-F"], "grep")
                };
                let (elapsed, exit) = timed_run(binary, &case, leading)?;
                emit(&format!(
                    "{{\"record\":\"sample\",\"phase\":\"{phase}\",\"case\":\"{}\",\"round\":{round},\
                     \"position\":{position},\"binary\":\"{label}\",\"binary_index\":{binary_index},\
                     \"elapsed_ns\":{elapsed},\"exit\":{exit}}}",
                    case.name,
                ));
                Ok(elapsed)
            };
            let (first_index, second_index) = if wf_first { (0, 1) } else { (1, 0) };
            let first = sample(first_index, 1)? as f64;
            let second = sample(second_index, 2)? as f64;
            first_by_binary[first_index].push(first);
            second_by_binary[second_index].push(second);
            let (wf, other) = if wf_first { (first, second) } else { (second, first) };
            // Ratio orientation: comparator / wfgrep. Below 1.0 means the
            // wfgrep side is slower. In the null phase the "comparator" is
            // the second wfgrep invocation, so the ratio tests the harness.
            ratios.push(other / wf);
        }
        let summary = summarize(&ratios);
        let wf_median = median(&mut first_by_binary[0].iter().chain(second_by_binary[0].iter()).copied().collect::<Vec<_>>());
        let other_median = median(&mut first_by_binary[1].iter().chain(second_by_binary[1].iter()).copied().collect::<Vec<_>>());
        emit(&format!(
            "{{\"record\":\"summary\",\"phase\":\"{phase}\",\"case\":\"{}\",\"rounds\":{ROUNDS},\
             \"wfgrep_median_ns\":{:.0},\"other_median_ns\":{:.0},\
             \"ratio_median\":{:.6},\"ci_low\":{:.6},\"ci_high\":{:.6},\"relative_half_width\":{:.6},\
             \"classified\":{}}}",
            case.name,
            wf_median,
            other_median,
            summary.point,
            summary.low,
            summary.high,
            summary.relative_half_width,
            case.classified,
        ));
    }
    record_power(&phase, "end");
    Ok(())
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
/// retired, cycles elapsed, peak footprint) for both binaries on every case.
fn counters() -> Result<(), String> {
    record_power("counters", "start");
    for case in cases() {
        warm(&case)?;
        for (label, binary, leading) in [
            ("wfgrep", wfgrep_binary(), Vec::new()),
            ("grep", grep_binary(), vec!["-h", "-F"]),
        ] {
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
                    "{{\"record\":\"counters\",\"case\":\"{}\",\"binary\":\"{label}\",\"repetition\":{repetition},\
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

// --- profiling support ---------------------------------------------------

/// Launches wfgrep on one case with the file list repeated `reps` times and
/// attaches `/usr/bin/sample` for `seconds`; the profile-only workload is
/// never timed comparatively.
fn profile(arguments: &[String]) -> Result<(), String> {
    let case_name = arguments.get(1).ok_or("profile requires CASE")?;
    let reps: usize = arguments.get(2).ok_or("profile requires REPS")?.parse().map_err(|_| "bad REPS")?;
    let seconds: &str = arguments.get(3).map(String::as_str).unwrap_or("10");
    let binary_label = arguments.get(4).map(String::as_str).unwrap_or("wfgrep");
    let case = cases().into_iter().find(|case| case.name == *case_name).ok_or("unknown case")?;
    let (binary, leading): (PathBuf, Vec<&str>) = if binary_label == "grep" {
        (grep_binary(), vec!["-h", "-F"])
    } else {
        (wfgrep_binary(), Vec::new())
    };
    let mut command = Command::new(&binary);
    command
        .current_dir(corpus_root())
        .env_clear()
        .env("LC_ALL", "C")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.args(&leading).arg(case.pattern);
    for _ in 0..reps {
        command.args(&case.files);
    }
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    let out = work_root().join("build").join(format!("profile-{binary_label}-{case_name}.txt"));
    let status = Command::new("/usr/bin/sample")
        .arg(child.id().to_string())
        .arg(seconds)
        .arg("-f")
        .arg(&out)
        .status()
        .map_err(|error| error.to_string())?;
    let _ = child.kill();
    let _ = child.wait();
    if !status.success() {
        return Err("sample failed".to_owned());
    }
    emit(&format!(
        "{{\"record\":\"profile\",\"time\":{},\"case\":\"{}\",\"binary\":\"{binary_label}\",\"reps\":{reps},\"seconds\":\"{seconds}\",\"file\":\"{}\"}}",
        unix_time(),
        case.name,
        out.display(),
    ));
    println!("profile written to {}", out.display());
    Ok(())
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
