#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const RUN_ID: &str = "wf-literal-line-floor-1";
const PREFIX_LENGTH: u64 = 67_108_864;
const FULL_SHA256: &str = "08c2d7399372afe859238e25cb414e5fadbe5a416a8e69418787305b1e79296f";
const PREFIX_SHA256: &str = "ce55e37ed74f5b34773ce83597e5d61a83d0d0792d9cbb95fe0fc898ed09a1ee";
const NEEDLE_HEX: &str = "d0a8d0b5d180d0bbd0bed0ba20d0a5d0bed0bbd0bcd181";
const NEEDLE_SHA256: &str = "192672866949818d8c8ea7089c9e622801bd763489f0314c004a459c616cc9b1";
const MEMCHR_SHA256: &str = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98";
const BOOTSTRAPS: usize = 10_000;
const BOOTSTRAP_SEED: u64 = 2_026_080_503;
const VARIANTS: [&str; 4] = ["whitefoot", "c", "naive-rust", "memmem-rust"];
const ORDERS: [[usize; 4]; 4] = [[0, 1, 3, 2], [1, 2, 0, 3], [2, 3, 1, 0], [3, 0, 2, 1]];

#[derive(Clone, Debug, Eq, PartialEq)]
struct Sample {
    digest: u64,
    records: u64,
    input_hash: u64,
    needle_hash: u64,
    elapsed_ns: u64,
    repetitions: u64,
    length: u64,
}

#[derive(Clone, Debug)]
struct Round {
    elapsed: [u64; 4],
}

struct EnvironmentIdentity {
    arch: String,
    product_version: String,
    build_version: String,
    clang: String,
    rustc_release: String,
    rustc_commit: String,
    llvm: String,
}

fn work_root() -> Result<PathBuf, String> {
    std::env::var_os("WF_LITERAL_LINE_WORK_ROOT")
        .map(PathBuf::from)
        .ok_or_else(|| "WF_LITERAL_LINE_WORK_ROOT is required".to_owned())
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

fn command_text(command: Command, label: &str) -> Result<String, String> {
    let output = command_output(command, label)?;
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| format!("{label} output is not UTF-8: {error}"))
}

fn require_environment() -> Result<EnvironmentIdentity, String> {
    let mut uname = Command::new("uname");
    uname.arg("-m");
    let arch = command_text(uname, "uname -m")?;
    let mut product = Command::new("sw_vers");
    product.arg("-productVersion");
    let product_version = command_text(product, "sw_vers -productVersion")?;
    let mut build = Command::new("sw_vers");
    build.arg("-buildVersion");
    let build_version = command_text(build, "sw_vers -buildVersion")?;
    let mut clang = Command::new("/usr/bin/clang");
    clang.arg("--version");
    let clang_output = command_text(clang, "clang --version")?;
    let mut rustc = Command::new("rustc");
    rustc.arg("-vV");
    let rustc_output = command_text(rustc, "rustc -vV")?;
    if arch != "arm64"
        || product_version != "26.5.2"
        || build_version != "25F84"
        || !clang_output.contains("Apple clang version 21.0.0 (clang-2100.1.1.101)")
        || !rustc_output.contains("release: 1.97.1")
        || !rustc_output.contains("commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452")
        || !rustc_output.contains("LLVM version: 22.1.6")
    {
        return Err("target or toolchain identity differs from the frozen protocol".to_owned());
    }
    let value = |prefix: &str| {
        rustc_output
            .lines()
            .find_map(|line| line.strip_prefix(prefix))
            .unwrap_or_default()
            .to_owned()
    };
    Ok(EnvironmentIdentity {
        arch,
        product_version,
        build_version,
        clang: clang_output.lines().next().unwrap_or_default().to_owned(),
        rustc_release: value("release: "),
        rustc_commit: value("commit-hash: "),
        llvm: value("LLVM version: "),
    })
}

fn sha256(path: &Path) -> Result<String, String> {
    let mut command = Command::new("shasum");
    command.args(["-a", "256"]).arg(path);
    let output = command_output(command, &format!("shasum {}", path.display()))?;
    String::from_utf8(output.stdout)
        .map_err(|error| format!("shasum output is not UTF-8: {error}"))?
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| "shasum produced no digest".to_owned())
}

fn sha256_bytes(bytes: &[u8]) -> Result<String, String> {
    let mut child = Command::new("shasum")
        .args(["-a", "256"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| format!("cannot run shasum: {error}"))?;
    child
        .stdin
        .take()
        .ok_or_else(|| "missing shasum stdin".to_owned())?
        .write_all(bytes)
        .map_err(|error| format!("cannot write shasum input: {error}"))?;
    let output = child
        .wait_with_output()
        .map_err(|error| format!("shasum failed: {error}"))?;
    if !output.status.success() {
        return Err("shasum failed for byte input".to_owned());
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("shasum output is not UTF-8: {error}"))?
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| "shasum produced no digest".to_owned())
}

fn require_hash(path: &Path, expected: &str, label: &str) -> Result<(), String> {
    let observed = sha256(path)?;
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "{label} SHA-256 mismatch: expected {expected}, observed {observed}"
        ))
    }
}

fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).map_err(|error| error.to_string())?;
            u8::from_str_radix(pair, 16).map_err(|error| error.to_string())
        })
        .collect()
}

fn prepare_input(source: &Path, destination: &Path) -> Result<(), String> {
    require_hash(source, FULL_SHA256, "full ru.txt")?;
    if destination.exists()
        && destination
            .metadata()
            .map_err(|error| error.to_string())?
            .len()
            == PREFIX_LENGTH
        && sha256(destination)? == PREFIX_SHA256
    {
        println!("input-ready={}", destination.display());
        return Ok(());
    }
    std::fs::create_dir_all(
        destination
            .parent()
            .ok_or_else(|| "prefix has no parent".to_owned())?,
    )
    .map_err(|error| format!("cannot create input directory: {error}"))?;
    let mut input = File::open(source).map_err(|error| format!("cannot open source: {error}"))?;
    let mut output =
        File::create(destination).map_err(|error| format!("cannot create prefix: {error}"))?;
    let copied = std::io::copy(
        &mut Read::by_ref(&mut input).take(PREFIX_LENGTH),
        &mut output,
    )
    .map_err(|error| format!("cannot extract prefix: {error}"))?;
    output
        .sync_all()
        .map_err(|error| format!("cannot sync prefix: {error}"))?;
    if copied != PREFIX_LENGTH {
        return Err(format!("source ended after {copied} prefix bytes"));
    }
    require_hash(destination, PREFIX_SHA256, "ru.txt prefix")?;
    println!("input-ready={}", destination.display());
    Ok(())
}

fn executable(root: &Path, variant: &str) -> PathBuf {
    root.join("build").join(variant)
}

fn parse_sample(output: Output, label: &str) -> Result<Sample, String> {
    if !output.stderr.is_empty() {
        return Err(format!("{label} produced stderr"));
    }
    let text =
        String::from_utf8(output.stdout).map_err(|error| format!("{label} output: {error}"))?;
    let mut fields = BTreeMap::new();
    for word in text.split_whitespace() {
        let (name, value) = word
            .split_once('=')
            .ok_or_else(|| format!("malformed field {word:?}"))?;
        if fields.insert(name, value).is_some() {
            return Err(format!("duplicate field {name}"));
        }
    }
    let names = [
        "digest",
        "records",
        "input_hash",
        "needle_hash",
        "elapsed_ns",
        "repetitions",
        "length",
    ];
    if fields.len() != names.len() || names.iter().any(|name| !fields.contains_key(name)) {
        return Err(format!("unexpected sample fields: {fields:?}"));
    }
    let parse = |name: &str| {
        fields[name]
            .parse::<u64>()
            .map_err(|error| format!("invalid {name}: {error}"))
    };
    Ok(Sample {
        digest: parse("digest")?,
        records: parse("records")?,
        input_hash: parse("input_hash")?,
        needle_hash: parse("needle_hash")?,
        elapsed_ns: parse("elapsed_ns")?,
        repetitions: parse("repetitions")?,
        length: parse("length")?,
    })
}

fn run_sample(path: &Path, input: &Path, timed: bool) -> Result<Sample, String> {
    let mode = if timed {
        "--bench-input"
    } else {
        "--verify-input"
    };
    let mut command = Command::new(path);
    command.arg(mode).arg(input).arg(NEEDLE_HEX);
    let label = format!("{} {mode}", path.display());
    parse_sample(command_output(command, &label)?, &label)
}

fn require_code_shape(root: &Path) -> Result<(), String> {
    let build = root.join("build");
    let raw =
        std::fs::read_to_string(build.join("wf.raw.ll")).map_err(|error| error.to_string())?;
    let optimized =
        std::fs::read_to_string(build.join("wf.opt.ll")).map_err(|error| error.to_string())?;
    let c = std::fs::read_to_string(build.join("c.ll")).map_err(|error| error.to_string())?;
    let naive =
        std::fs::read_to_string(build.join("naive-rust.ll")).map_err(|error| error.to_string())?;
    let ceiling =
        std::fs::read_to_string(build.join("memmem-rust.ll")).map_err(|error| error.to_string())?;
    let ceiling_disasm = std::fs::read_to_string(build.join("memmem-rust.disasm"))
        .map_err(|error| error.to_string())?;
    if !raw.contains("define internal i64 @wf_literal_line(")
        || !raw.contains("buffer.index.trap")
        || !optimized.contains("define i64 @wf_literal_line(")
    {
        return Err("Whitefoot raw/optimized code-shape identity missing".to_owned());
    }
    if c.contains("memmem") || naive.contains("memmem") {
        return Err("same-algorithm control unexpectedly references memmem".to_owned());
    }
    if !ceiling.contains("%finder") || !ceiling.contains("call { i64, i64 } %") {
        return Err("Finder dispatch is absent from ceiling LLVM".to_owned());
    }
    if !ceiling_disasm.contains("memmem8searcher18searcher_kind_neon")
        || !ceiling_disasm.contains("cmeq.16b")
    {
        return Err(
            "expected memmem NEON packed-pair mechanism is absent from final disassembly"
                .to_owned(),
        );
    }
    for name in ["wf.s", "c.s", "naive-rust.s", "memmem-rust.s"] {
        if build
            .join(name)
            .metadata()
            .map_err(|error| error.to_string())?
            .len()
            == 0
        {
            return Err(format!("empty assembly artifact {name}"));
        }
    }
    Ok(())
}

fn verify() -> Result<BTreeMap<&'static str, Sample>, String> {
    let root = work_root()?;
    let input = root.join("input/ru-prefix-67108864.bin");
    if input
        .metadata()
        .map_err(|error| format!("cannot stat prefix: {error}"))?
        .len()
        != PREFIX_LENGTH
    {
        return Err("prefix length mismatch".to_owned());
    }
    require_hash(&input, PREFIX_SHA256, "ru.txt prefix")?;
    if sha256_bytes(&decode_hex(NEEDLE_HEX)?)? != NEEDLE_SHA256 {
        return Err("needle SHA-256 mismatch".to_owned());
    }
    require_hash(
        Path::new(
            "/Users/bytedance/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/memchr-2.8.3.crate",
        ),
        MEMCHR_SHA256,
        "memchr 2.8.3 crate",
    )?;
    require_code_shape(&root)?;
    for variant in VARIANTS {
        let mut command = Command::new(executable(&root, variant));
        command.arg("--check");
        let output = command_output(command, &format!("{variant} --check"))?;
        if !output.stdout.is_empty() || !output.stderr.is_empty() {
            return Err(format!("{variant} --check produced output"));
        }
    }
    let mut identities = BTreeMap::new();
    let mut expected: Option<Sample> = None;
    for variant in VARIANTS {
        let sample = run_sample(&executable(&root, variant), &input, false)?;
        if sample.records != 74
            || sample.length != PREFIX_LENGTH
            || sample.elapsed_ns != 0
            || sample.repetitions != 0
        {
            return Err(format!(
                "{variant} verification identity mismatch: {sample:?}"
            ));
        }
        if let Some(reference) = &expected {
            if &sample != reference {
                return Err(format!(
                    "{variant} differs from Whitefoot: {sample:?} vs {reference:?}"
                ));
            }
        } else {
            expected = Some(sample.clone());
        }
        identities.insert(variant, sample);
    }
    println!("WF-LITERAL-LINE correctness and code-shape gate green");
    Ok(identities)
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

fn ratio_values(rounds: &[Round], numerator: usize, denominator: usize) -> Vec<f64> {
    rounds
        .iter()
        .map(|round| round.elapsed[numerator] as f64 / round.elapsed[denominator] as f64)
        .collect()
}

fn ratio_summary(name: &str, values: &[f64], salt: u64) -> String {
    let point = median(values);
    let mut state = BOOTSTRAP_SEED ^ salt;
    let mut estimates = Vec::with_capacity(BOOTSTRAPS);
    for _ in 0..BOOTSTRAPS {
        let mut resample = Vec::with_capacity(32);
        for _ in 0..8 {
            let block = (next_random(&mut state) as usize) % 8;
            resample.extend_from_slice(&values[block * 4..block * 4 + 4]);
        }
        estimates.push(median(&resample));
    }
    estimates.sort_by(f64::total_cmp);
    let low = estimates[249];
    let high = estimates[9_749];
    let relative_half_width = (high - low) / (2.0 * point);
    format!(
        "{{\"name\":\"{name}\",\"median\":{point:.9},\"low\":{low:.9},\"high\":{high:.9},\"relative_half_width\":{relative_half_width:.9}}}"
    )
}

fn summarize(rounds: &[Round]) -> String {
    let comparisons = [
        ("compiler_floor", 1, 0),
        ("cross_toolchain", 2, 0),
        ("algorithm_ceiling", 3, 2),
    ];
    let ratio_json = comparisons
        .into_iter()
        .enumerate()
        .map(|(index, (name, numerator, denominator))| {
            ratio_summary(
                name,
                &ratio_values(rounds, numerator, denominator),
                index as u64 + 1,
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let mut positions = Vec::new();
    for variant in 0..4 {
        for position in 0..4 {
            let values = rounds
                .iter()
                .enumerate()
                .filter(|(round, _)| ORDERS[round % 4][position] == variant)
                .map(|(_, sample)| sample.elapsed[variant] as f64)
                .collect::<Vec<_>>();
            positions.push(format!(
                "{{\"variant\":\"{}\",\"position\":{position},\"median_elapsed_ns\":{:.3}}}",
                VARIANTS[variant],
                median(&values)
            ));
        }
    }

    let mut leave_one_out = Vec::new();
    for (name, numerator, denominator) in comparisons {
        for omitted in 0..4 {
            let values = rounds
                .iter()
                .enumerate()
                .filter(|(round, _)| round % 4 != omitted)
                .map(|(_, round)| {
                    round.elapsed[numerator] as f64 / round.elapsed[denominator] as f64
                })
                .collect::<Vec<_>>();
            leave_one_out.push(format!(
                "{{\"name\":\"{name}\",\"omitted_order_class\":{omitted},\"median\":{:.9}}}",
                median(&values)
            ));
        }
    }
    format!(
        "{{\"run_id\":\"{RUN_ID}\",\"bootstrap_seed\":{BOOTSTRAP_SEED},\"bootstraps\":{BOOTSTRAPS},\"ratios\":[{ratio_json}],\"position_medians\":[{}],\"leave_one_order_class_out\":[{}]}}\n",
        positions.join(","),
        leave_one_out.join(",")
    )
}

fn git(arguments: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    command.args(arguments);
    let output = command_output(command, &format!("git {}", arguments.join(" ")))?;
    String::from_utf8(output.stdout)
        .map(|text| text.trim().to_owned())
        .map_err(|error| format!("git output is not UTF-8: {error}"))
}

fn require_freeze(commit: &str) -> Result<String, String> {
    if commit.is_empty() {
        return Err("bench requires --freeze-commit <exact HEAD>".to_owned());
    }
    let head = git(&["rev-parse", "HEAD"])?;
    if git(&["rev-parse", commit])? != head {
        return Err("--freeze-commit must resolve to current HEAD".to_owned());
    }
    let status = git(&["status", "--porcelain", "--untracked-files=all"])?;
    if !status.is_empty() {
        return Err(format!("bench requires a clean frozen worktree:\n{status}"));
    }
    let repository = PathBuf::from(git(&["rev-parse", "--show-toplevel"])?);
    let protocol_path = repository.join("research/experiments/literal-line-floor/PROTOCOL.md");
    let protocol = std::fs::read_to_string(&protocol_path)
        .map_err(|error| format!("cannot read protocol: {error}"))?;
    if !protocol
        .starts_with("# WF-LITERAL-LINE protocol\n\nStatus: FROZEN BEFORE COMPARATIVE TIMING\n")
    {
        return Err("PROTOCOL.md is not frozen".to_owned());
    }
    git(&[
        "ls-files",
        "--error-unmatch",
        ":(top)research/experiments/literal-line-floor/PROTOCOL.md",
        ":(top)research/experiments/literal-line-floor/CODE_SHAPE.md",
        ":(top)research/experiments/literal-line-floor/runner.rs",
        ":(top)research/experiments/literal-line-floor/literal_line.wf",
    ])?;
    Ok(head)
}

fn require_power() -> Result<String, String> {
    let mut command = Command::new("pmset");
    command.args(["-g", "batt"]);
    let output = command_output(command, "pmset -g batt")?;
    let text = String::from_utf8_lossy(&output.stdout);
    if !text.contains("AC Power") {
        return Err(format!("measurement requires AC Power: {text}"));
    }
    let mut command = Command::new("pmset");
    command.args(["-g", "custom"]);
    let output = command_output(command, "pmset -g custom")?;
    let custom = String::from_utf8_lossy(&output.stdout);
    if !custom
        .lines()
        .any(|line| line.split_whitespace().collect::<Vec<_>>().as_slice() == ["lowpowermode", "0"])
    {
        return Err("measurement requires Low Power Mode off".to_owned());
    }
    Ok(text.lines().next().unwrap_or_default().trim().to_owned())
}

fn write_line(file: &mut File, line: &str) -> Result<(), String> {
    writeln!(file, "{line}").map_err(|error| format!("cannot write evidence: {error}"))?;
    file.flush()
        .map_err(|error| format!("cannot flush evidence: {error}"))
}

fn bench(commit: &str) -> Result<(), String> {
    let identities = verify()?;
    let head = require_freeze(commit)?;
    let root = work_root()?;
    let input = root.join("input/ru-prefix-67108864.bin");
    let repository = PathBuf::from(git(&["rev-parse", "--show-toplevel"])?);
    let experiment = repository.join("research/experiments/literal-line-floor");
    let protocol_sha = sha256(&experiment.join("PROTOCOL.md"))?;
    let code_shape_sha = sha256(&experiment.join("CODE_SHAPE.md"))?;
    let lock_sha = sha256(&experiment.join("ceiling/Cargo.lock"))?;
    let binary_shas = VARIANTS
        .into_iter()
        .map(|variant| sha256(&executable(&root, variant)).map(|sha| (variant, sha)))
        .collect::<Result<Vec<_>, _>>()?;
    let environment = require_environment()?;
    let power_before = require_power()?;
    let run = root.join("runs").join(RUN_ID);
    if run.exists() {
        return Err(format!("create-once run target exists: {}", run.display()));
    }
    std::fs::create_dir_all(run.parent().ok_or_else(|| "run has no parent".to_owned())?)
        .map_err(|error| format!("cannot create runs directory: {error}"))?;
    std::fs::create_dir(&run).map_err(|error| format!("cannot create run: {error}"))?;
    let mut evidence = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(run.join("evidence.jsonl"))
        .map_err(|error| format!("cannot create evidence: {error}"))?;
    write_line(
        &mut evidence,
        &format!(
            "{{\"type\":\"header\",\"run_id\":\"{RUN_ID}\",\"freeze_commit\":\"{head}\",\"rounds\":32,\"bootstrap_seed\":{BOOTSTRAP_SEED},\"bootstraps\":{BOOTSTRAPS},\"power_before\":\"{power_before}\",\"arch\":\"{}\",\"macos_product\":\"{}\",\"macos_build\":\"{}\",\"clang\":\"{}\",\"rustc_release\":\"{}\",\"rustc_commit\":\"{}\",\"rustc_llvm\":\"{}\",\"protocol_sha256\":\"{protocol_sha}\",\"code_shape_sha256\":\"{code_shape_sha}\",\"lock_sha256\":\"{lock_sha}\",\"input_sha256\":\"{PREFIX_SHA256}\",\"needle_sha256\":\"{NEEDLE_SHA256}\",\"memchr_crate_sha256\":\"{MEMCHR_SHA256}\",\"whitefoot_sha256\":\"{}\",\"c_sha256\":\"{}\",\"naive_rust_sha256\":\"{}\",\"memmem_rust_sha256\":\"{}\"}}",
            environment.arch,
            environment.product_version,
            environment.build_version,
            environment.clang,
            environment.rustc_release,
            environment.rustc_commit,
            environment.llvm,
            binary_shas[0].1,
            binary_shas[1].1,
            binary_shas[2].1,
            binary_shas[3].1
        ),
    )?;
    let expected = identities
        .get("whitefoot")
        .ok_or_else(|| "missing identity".to_owned())?;
    let mut rounds = Vec::with_capacity(32);
    for round in 0..32 {
        let mut elapsed = [0_u64; 4];
        for (position, variant_index) in ORDERS[round % 4].into_iter().enumerate() {
            let variant = VARIANTS[variant_index];
            let sample = run_sample(&executable(&root, variant), &input, true)?;
            if sample.digest != expected.digest
                || sample.records != expected.records
                || sample.input_hash != expected.input_hash
                || sample.needle_hash != expected.needle_hash
                || sample.length != expected.length
                || sample.repetitions != 16
                || sample.elapsed_ns == 0
            {
                return Err(format!("{variant} timed identity mismatch: {sample:?}"));
            }
            elapsed[variant_index] = sample.elapsed_ns;
            write_line(
                &mut evidence,
                &format!(
                    "{{\"type\":\"sample\",\"round\":{round},\"block\":{},\"order_class\":{},\"position\":{position},\"variant\":\"{variant}\",\"elapsed_ns\":{},\"digest\":{},\"records\":{},\"repetitions\":16,\"length\":{}}}",
                    round / 4,
                    round % 4,
                    sample.elapsed_ns,
                    sample.digest,
                    sample.records,
                    sample.length
                ),
            )?;
        }
        rounds.push(Round { elapsed });
    }
    let power_after = require_power()?;
    let summary = summarize(&rounds);
    let mut summary_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(run.join("summary.json"))
        .map_err(|error| format!("cannot create summary: {error}"))?;
    summary_file
        .write_all(summary.as_bytes())
        .map_err(|error| format!("cannot write summary: {error}"))?;
    summary_file
        .sync_all()
        .map_err(|error| format!("cannot sync summary: {error}"))?;
    println!("{}", summary.trim());
    write_line(
        &mut evidence,
        &format!("{{\"type\":\"complete\",\"power_after\":\"{power_after}\"}}"),
    )?;
    println!("evidence={}", run.join("evidence.jsonl").display());
    Ok(())
}

fn main() {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match arguments.as_slice() {
        [command, source, destination] if command == "prepare-input" => {
            prepare_input(Path::new(source), Path::new(destination))
        }
        [command] if command == "verify" => verify().map(|_| ()),
        [command, flag, commit] if command == "bench" && flag == "--freeze-commit" => bench(commit),
        _ => Err(
            "usage: runner prepare-input SOURCE PREFIX | verify | bench --freeze-commit HEAD"
                .to_owned(),
        ),
    };
    if let Err(message) = result {
        eprintln!("runner: {message}");
        std::process::exit(1);
    }
}
