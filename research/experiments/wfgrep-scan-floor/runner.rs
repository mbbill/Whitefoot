#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const RUN_ID: &str = "wf-scan-floor-1";
const ROUNDS: usize = 30;
const BOOTSTRAPS: usize = 10_000;
const BOOTSTRAP_SEED: u64 = 20_260_805;
const VARIANTS: [&str; 3] = ["whitefoot", "c", "rust"];
const SHAPES: [&str; 2] = ["full", "early"];
const PERMUTATIONS: [[usize; 3]; 6] = [
    [0, 1, 2],
    [0, 2, 1],
    [1, 0, 2],
    [1, 2, 0],
    [2, 0, 1],
    [2, 1, 0],
];

#[derive(Clone, Debug)]
struct Sample {
    shape: String,
    checksum: u64,
    data_hash: u64,
    elapsed_ns: u64,
    repetitions: u64,
    length: u64,
}

#[derive(Clone, Debug)]
struct Round {
    elapsed: BTreeMap<&'static str, u64>,
}

fn work_root() -> Result<PathBuf, String> {
    std::env::var_os("WF_SCAN_WORK_ROOT")
        .map(PathBuf::from)
        .ok_or_else(|| "WF_SCAN_WORK_ROOT is required".to_owned())
}

fn executable(root: &Path, variant: &str, shape: &str) -> PathBuf {
    root.join("build").join(format!("{variant}-{shape}"))
}

fn command_output(mut command: Command, label: &str) -> Result<Output, String> {
    let output = command
        .output()
        .map_err(|error| format!("cannot run {label}: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "{label} failed with {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn run_check(path: &Path) -> Result<(), String> {
    let mut command = Command::new(path);
    command.arg("--check");
    let label = format!("{} --check", path.display());
    let output = command_output(command, &label)?;
    if output.stdout.is_empty() && output.stderr.is_empty() {
        Ok(())
    } else {
        Err(format!("{label} produced unexpected output"))
    }
}

fn parse_sample(path: &Path, expected_shape: &str) -> Result<Sample, String> {
    let command = Command::new(path);
    let output = command_output(command, &path.display().to_string())?;
    if !output.stderr.is_empty() {
        return Err(format!("{} produced stderr", path.display()));
    }
    let text = std::str::from_utf8(&output.stdout)
        .map_err(|error| format!("{} stdout is not UTF-8: {error}", path.display()))?;
    let mut fields = BTreeMap::new();
    for word in text.split_whitespace() {
        let (name, value) = word
            .split_once('=')
            .ok_or_else(|| format!("malformed sample field {word:?}"))?;
        if fields.insert(name, value).is_some() {
            return Err(format!("duplicate sample field {name}"));
        }
    }
    let expected_names = [
        "shape",
        "checksum",
        "data_hash",
        "elapsed_ns",
        "repetitions",
        "length",
    ];
    if fields.len() != expected_names.len()
        || expected_names.iter().any(|name| !fields.contains_key(name))
    {
        return Err(format!("unexpected sample fields: {fields:?}"));
    }
    let shape = fields["shape"].to_owned();
    if shape != expected_shape {
        return Err(format!("expected shape {expected_shape}, observed {shape}"));
    }
    let parse = |name: &str| {
        fields[name]
            .parse::<u64>()
            .map_err(|error| format!("invalid {name}: {error}"))
    };
    Ok(Sample {
        shape,
        checksum: parse("checksum")?,
        data_hash: parse("data_hash")?,
        elapsed_ns: parse("elapsed_ns")?,
        repetitions: parse("repetitions")?,
        length: parse("length")?,
    })
}

fn verify() -> Result<(), String> {
    let root = work_root()?;
    for shape in SHAPES {
        for variant in VARIANTS {
            run_check(&executable(&root, variant, shape))?;
        }
    }
    println!("WF-SCAN-FLOOR correctness gate green");
    Ok(())
}

fn git_head() -> Result<String, String> {
    let mut command = Command::new("git");
    command.args(["rev-parse", "HEAD"]);
    let output = command_output(command, "git rev-parse HEAD")?;
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("git output is not UTF-8: {error}"))
}

fn require_power() -> Result<String, String> {
    let mut battery = Command::new("pmset");
    battery.args(["-g", "batt"]);
    let battery = command_output(battery, "pmset -g batt")?;
    let battery = String::from_utf8_lossy(&battery.stdout);
    if !battery.contains("AC Power") {
        return Err(format!("measurement requires AC Power: {battery}"));
    }

    let mut custom = Command::new("pmset");
    custom.args(["-g", "custom"]);
    let custom = command_output(custom, "pmset -g custom")?;
    let custom = String::from_utf8_lossy(&custom.stdout);
    let low_power_off = custom.lines().any(|line| {
        let mut words = line.split_whitespace();
        matches!(
            (words.next(), words.next()),
            (Some("lowpowermode"), Some("0"))
        )
    });
    if !low_power_off {
        return Err("measurement requires Low Power Mode off".to_owned());
    }
    Ok(battery.lines().next().unwrap_or_default().trim().to_owned())
}

fn write_line(file: &mut File, line: &str) -> Result<(), String> {
    writeln!(file, "{line}").map_err(|error| format!("cannot write evidence: {error}"))?;
    file.flush()
        .map_err(|error| format!("cannot flush evidence: {error}"))
}

fn median(values: &[f64]) -> f64 {
    let mut ordered = values.to_vec();
    ordered.sort_by(f64::total_cmp);
    let middle = ordered.len() / 2;
    if ordered.len() % 2 == 0 {
        (ordered[middle - 1] + ordered[middle]) / 2.0
    } else {
        ordered[middle]
    }
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn bootstrap_interval(values: &[f64], salt: u64) -> (f64, f64) {
    let mut state = BOOTSTRAP_SEED ^ salt;
    let mut estimates = Vec::with_capacity(BOOTSTRAPS);
    let mut sample = Vec::with_capacity(values.len());
    for _ in 0..BOOTSTRAPS {
        sample.clear();
        for _ in 0..values.len() {
            let index = (next_random(&mut state) as usize) % values.len();
            sample.push(values[index]);
        }
        estimates.push(median(&sample));
    }
    estimates.sort_by(f64::total_cmp);
    (estimates[249], estimates[9_749])
}

fn summarize(shape: &str, rounds: &[Round]) -> Result<String, String> {
    let ratios = |control: &str| {
        rounds
            .iter()
            .map(|round| round.elapsed[control] as f64 / round.elapsed["whitefoot"] as f64)
            .collect::<Vec<_>>()
    };
    let c_ratios = ratios("c");
    let rust_ratios = ratios("rust");
    let c_median = median(&c_ratios);
    let rust_median = median(&rust_ratios);
    let salt = if shape == "full" { 1 } else { 2 };
    let (c_low, c_high) = bootstrap_interval(&c_ratios, salt);
    let (rust_low, rust_high) = bootstrap_interval(&rust_ratios, salt + 10);
    let c_half_width = (c_high - c_low) / (2.0 * c_median);
    let rust_half_width = (rust_high - rust_low) / (2.0 * rust_median);
    Ok(format!(
        "{{\"type\":\"summary\",\"shape\":\"{shape}\",\"ratio_definition\":\"control_elapsed/whitefoot_elapsed\",\"c_median\":{c_median:.9},\"c_low\":{c_low:.9},\"c_high\":{c_high:.9},\"c_relative_half_width\":{c_half_width:.9},\"rust_median\":{rust_median:.9},\"rust_low\":{rust_low:.9},\"rust_high\":{rust_high:.9},\"rust_relative_half_width\":{rust_half_width:.9}}}"
    ))
}

fn bench() -> Result<(), String> {
    verify()?;
    let root = work_root()?;
    let power_before = require_power()?;
    let runs = root.join("runs");
    std::fs::create_dir_all(&runs)
        .map_err(|error| format!("cannot create {}: {error}", runs.display()))?;
    let run = runs.join(RUN_ID);
    std::fs::create_dir(&run)
        .map_err(|error| format!("run directory {} must not exist: {error}", run.display()))?;
    let evidence_path = run.join("evidence.jsonl");
    let mut evidence = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&evidence_path)
        .map_err(|error| format!("cannot create {}: {error}", evidence_path.display()))?;
    let head = git_head()?;
    write_line(
        &mut evidence,
        &format!(
            "{{\"type\":\"header\",\"run_id\":\"{RUN_ID}\",\"commit\":\"{head}\",\"rounds\":{ROUNDS},\"bootstraps\":{BOOTSTRAPS},\"bootstrap_seed\":{BOOTSTRAP_SEED},\"power_before\":\"{power_before}\"}}"
        ),
    )?;

    let mut results: BTreeMap<&str, Vec<Round>> = SHAPES
        .into_iter()
        .map(|shape| (shape, Vec::new()))
        .collect();
    let mut identities: BTreeMap<&str, (u64, u64, u64, u64)> = BTreeMap::new();
    for round_index in 0..ROUNDS {
        let permutation = PERMUTATIONS[round_index % PERMUTATIONS.len()];
        let shape_order = if round_index % 2 == 0 {
            ["full", "early"]
        } else {
            ["early", "full"]
        };
        for shape in shape_order {
            let mut round = Round {
                elapsed: BTreeMap::new(),
            };
            for (position, variant_index) in permutation.into_iter().enumerate() {
                let variant = VARIANTS[variant_index];
                let sample = parse_sample(&executable(&root, variant, shape), shape)?;
                let identity = (
                    sample.checksum,
                    sample.data_hash,
                    sample.repetitions,
                    sample.length,
                );
                match identities.get(shape) {
                    Some(expected) if *expected != identity => {
                        return Err(format!(
                            "{shape} work identity changed: expected {expected:?}, observed {identity:?}"
                        ));
                    }
                    None => {
                        identities.insert(shape, identity);
                    }
                    _ => {}
                }
                if round.elapsed.insert(variant, sample.elapsed_ns).is_some() {
                    return Err(format!("duplicate {variant} sample in round {round_index}"));
                }
                write_line(
                    &mut evidence,
                    &format!(
                        "{{\"type\":\"sample\",\"round\":{round_index},\"position\":{position},\"variant\":\"{variant}\",\"shape\":\"{}\",\"checksum\":{},\"data_hash\":{},\"elapsed_ns\":{},\"repetitions\":{},\"length\":{}}}",
                        sample.shape,
                        sample.checksum,
                        sample.data_hash,
                        sample.elapsed_ns,
                        sample.repetitions,
                        sample.length
                    ),
                )?;
            }
            results
                .get_mut(shape)
                .ok_or_else(|| format!("missing result vector for {shape}"))?
                .push(round);
        }
    }
    let power_after = require_power()?;
    for shape in SHAPES {
        let summary = summarize(shape, &results[shape])?;
        println!("{summary}");
        write_line(&mut evidence, &summary)?;
    }
    write_line(
        &mut evidence,
        &format!("{{\"type\":\"complete\",\"power_after\":\"{power_after}\"}}"),
    )?;
    println!("evidence={}", evidence_path.display());
    Ok(())
}

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments.as_slice() {
        [command] if command == "verify" => verify(),
        [command] if command == "bench" => bench(),
        _ => Err("usage: runner verify|bench".to_owned()),
    };
    if let Err(message) = result {
        eprintln!("runner: {message}");
        std::process::exit(1);
    }
}
