//! Runner for the §9.1 initialization-cost row.
//!
//! Two questions, two measurements, because one of them cannot answer the
//! other:
//!
//! 1. **Steady-state throughput.** `drain.wf`, whose reusable buffer the
//!    language initializes at allocation, against the same drain in C over
//!    *uninitialized* storage — the control §9.1 names. The timed region is
//!    the whole process, so the one-time fill is counted, exactly as the row
//!    requires. This is the primary preregistered observable.
//!
//! 2. **The one-time cost itself.** One C binary, one source, one changed
//!    allocation call: `calloc` against `malloc`. This is the same-source
//!    ablation, and it is the measurement that actually decides the stop
//!    condition, because a one-page fill is orders of magnitude below what
//!    whole-process timing can resolve. Reporting only the first would leave
//!    the stop condition unrefuted rather than determinate.
//!
//! Usage: `runner verify` then `runner bench`. The work root comes from
//! `WF_INIT_WORK_ROOT`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// The fixed run identity. Its raw directory must not already exist.
const RUN_ID: &str = "buffer-init-cost-1";
/// Paired rounds. Each of the six execution orders appears five times.
const ROUNDS: usize = 30;
/// Bytes of corpus. Large enough that the drain, not process startup,
/// dominates a whole-process measurement.
const CORPUS_BYTES: usize = 256 * 1024 * 1024;
/// The buffer both sides reuse, matching `wfgrep`'s own.
const PAGE: usize = 4096;
/// Allocations per `fill` invocation, and how many invocations are timed.
const FILL_REPETITIONS: u64 = 2_000_000;
const FILL_INVOCATIONS: usize = 9;
/// Deterministic bootstrap over complete rounds.
const BOOTSTRAPS: usize = 10_000;
const BOOTSTRAP_SEED: u64 = 20_260_806;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    let root = PathBuf::from(
        std::env::var("WF_INIT_WORK_ROOT").expect("WF_INIT_WORK_ROOT names the work root"),
    );
    let corpus = prepare_corpus(&root);
    match mode.as_str() {
        "verify" => verify(&root, &corpus),
        "bench" => {
            verify(&root, &corpus);
            bench(&root, &corpus);
        }
        other => {
            eprintln!("runner: unknown mode {other}; use verify or bench");
            std::process::exit(2);
        }
    }
}

/// The three timed programs, in a fixed order.
///
/// `whitefoot` initializes its buffer because the language does; `uninit` is
/// the §9.1 control; `init` is the same C source with one changed allocation
/// call, which is what isolates the fill from every other difference.
fn programs(root: &Path) -> [(&'static str, Command); 3] {
    let build = root.join("build");
    let mut whitefoot = Command::new(build.join("whitefoot-drain"));
    whitefoot.arg("input.bin");
    let mut uninit = Command::new(build.join("control"));
    uninit.args(["drain", "malloc", "input.bin"]);
    let mut init = Command::new(build.join("control"));
    init.args(["drain", "calloc", "input.bin"]);
    [
        ("whitefoot", whitefoot),
        ("uninit", uninit),
        ("init", init),
    ]
}

/// Writes the corpus once and returns the status a correct drain must report.
///
/// The bytes are a fixed linear congruential stream, so the expected witness
/// is computable here and the comparison has a real oracle instead of merely
/// agreeing programs.
fn prepare_corpus(root: &Path) -> u8 {
    let directory = root.join("corpus");
    std::fs::create_dir_all(&directory).expect("create the corpus directory");
    let path = directory.join("input.bin");

    let mut bytes = Vec::with_capacity(CORPUS_BYTES);
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
    for _ in 0..CORPUS_BYTES {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // The low bits of an LCG are poor; the high bits are what is used.
        bytes.push((state >> 56) as u8);
    }
    if !path.exists() || std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0) as usize != CORPUS_BYTES
    {
        std::fs::write(&path, &bytes).expect("write the corpus");
    }

    // Both drains touch byte zero of every delivered chunk. A regular file
    // delivers full pages until the last, so the witness is the sum of the
    // bytes at every page boundary.
    let mut witness: u8 = 0;
    let mut at = 0;
    while at < CORPUS_BYTES {
        witness = witness.wrapping_add(bytes[at]);
        at += PAGE;
    }
    if witness == 0 { 3 } else { 0 }
}

/// The correctness oracle: every program must report the status the corpus
/// determines, and the Whitefoot program must still have the shape the row is
/// about.
fn verify(root: &Path, expected: &u8) {
    let module = std::fs::read_to_string(root.join("build/drain.opt.ll"))
        .expect("read the optimized Whitefoot module");
    assert_eq!(
        module.matches("@calloc(i64 1, i64 4096)").count(),
        1,
        "the drain must still allocate and initialize exactly one page"
    );
    assert_eq!(
        module.matches("@malloc(").count(),
        0,
        "an uninitialized Whitefoot allocation would invalidate the comparison"
    );
    assert_eq!(
        module.matches("call i64 @read(").count(),
        1,
        "the drain must still be one reused buffer and one host transfer site"
    );

    for (name, mut command) in programs(root) {
        let status = command
            .current_dir(root.join("corpus"))
            .status()
            .unwrap_or_else(|error| panic!("run {name}: {error}"));
        assert_eq!(
            status.code(),
            Some(i32::from(*expected)),
            "{name} disagreed with the corpus oracle"
        );
    }
    println!("oracle: every drain reports {expected}");
}

fn bench(root: &Path, expected: &u8) {
    let raw = root.join("raw");
    std::fs::create_dir_all(&raw).expect("create the raw directory");
    let log = raw.join(format!("{RUN_ID}.jsonl"));
    assert!(
        !log.exists(),
        "the fixed run identity {RUN_ID} already has raw output at {}; a rerun \
         must use a fresh work root rather than overwrite a recorded sample",
        log.display()
    );
    let mut sink = std::fs::File::create(&log).expect("create the raw log");
    writeln!(
        sink,
        "{{\"type\":\"header\",\"run_id\":\"{RUN_ID}\",\"rounds\":{ROUNDS},\
         \"corpus_bytes\":{CORPUS_BYTES},\"page\":{PAGE},\
         \"bootstraps\":{BOOTSTRAPS},\"bootstrap_seed\":{BOOTSTRAP_SEED}}}"
    )
    .expect("write the header");

    // Six execution orders over three programs, each appearing five times.
    const ORDERS: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];

    let mut elapsed = [Vec::new(), Vec::new(), Vec::new()];
    for round in 0..ROUNDS {
        let order = ORDERS[round % ORDERS.len()];
        let mut seen = [0.0_f64; 3];
        for slot in order {
            let (name, mut command) = programs(root).into_iter().nth(slot).expect("a program");
            let start = Instant::now();
            let status = command
                .current_dir(root.join("corpus"))
                .status()
                .unwrap_or_else(|error| panic!("run {name}: {error}"));
            let seconds = start.elapsed().as_secs_f64();
            assert_eq!(status.code(), Some(i32::from(*expected)), "{name}");
            seen[slot] = seconds;
            writeln!(
                sink,
                "{{\"type\":\"run\",\"round\":{round},\"program\":\"{name}\",\
                 \"elapsed_s\":{seconds:.9}}}"
            )
            .expect("write a run");
        }
        for slot in 0..3 {
            elapsed[slot].push(seen[slot]);
        }
    }

    // Ratio above 1.0 means Whitefoot is faster than the control.
    let against_uninit: Vec<f64> = (0..ROUNDS)
        .map(|round| elapsed[1][round] / elapsed[0][round])
        .collect();
    let against_init: Vec<f64> = (0..ROUNDS)
        .map(|round| elapsed[2][round] / elapsed[0][round])
        .collect();
    // The same-source ablation: initialized C over uninitialized C. Below 1.0
    // would mean the fill costs steady-state throughput.
    let fill_ratio: Vec<f64> = (0..ROUNDS)
        .map(|round| elapsed[1][round] / elapsed[2][round])
        .collect();

    report(&mut sink, "whitefoot_vs_uninitialized_c", &against_uninit, 0);
    report(&mut sink, "whitefoot_vs_initialized_c", &against_init, 1);
    report(&mut sink, "uninitialized_c_vs_initialized_c", &fill_ratio, 2);

    measure_fill(root, &mut sink);
    println!("raw: {}", log.display());
}

/// The isolated one-time cost of the fill, in nanoseconds per allocation.
fn measure_fill(root: &Path, sink: &mut std::fs::File) {
    let mut medians = Vec::new();
    for initialized in ["malloc", "calloc"] {
        let mut samples = Vec::new();
        for _ in 0..FILL_INVOCATIONS {
            let output = Command::new(root.join("build/control"))
                .args(["fill", initialized, &FILL_REPETITIONS.to_string()])
                .output()
                .expect("run the fill measurement");
            assert!(output.status.success(), "the fill measurement failed");
            let line = String::from_utf8(output.stdout).expect("fill output is UTF-8");
            writeln!(sink, "{}", line.trim_end()).expect("write a fill sample");
            let per = line
                .split("\"per_allocation_ns\":")
                .nth(1)
                .and_then(|rest| rest.split(',').next())
                .expect("a fill sample reports its per-allocation cost")
                .parse::<f64>()
                .expect("a per-allocation cost is a number");
            samples.push(per);
        }
        let point = median(&samples);
        println!("fill {initialized}: {point:.4} ns per allocation");
        medians.push(point);
    }
    let cost = medians[1] - medians[0];
    writeln!(
        sink,
        "{{\"type\":\"fill_summary\",\"malloc_ns\":{:.4},\"calloc_ns\":{:.4},\
         \"initialization_ns\":{cost:.4},\"page\":{PAGE}}}",
        medians[0], medians[1]
    )
    .expect("write the fill summary");
    println!("one-time initialization of one {PAGE}-byte page: {cost:.4} ns");
}

fn report(sink: &mut std::fs::File, name: &str, ratios: &[f64], salt: u64) {
    let point = median(ratios);
    let (low, high) = bootstrap_interval(ratios, salt);
    let half_width = (high - low) / (2.0 * point);
    writeln!(
        sink,
        "{{\"type\":\"summary\",\"comparison\":\"{name}\",\
         \"ratio_definition\":\"control_elapsed/subject_elapsed\",\
         \"median\":{point:.9},\"low\":{low:.9},\"high\":{high:.9},\
         \"relative_half_width\":{half_width:.9}}}"
    )
    .expect("write a summary");
    println!("{name}: {point:.4} [{low:.4}, {high:.4}] (half-width {half_width:.4})");
}

fn median(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).expect("no NaN samples"));
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

/// A deterministic percentile interval over complete rounds.
fn bootstrap_interval(values: &[f64], salt: u64) -> (f64, f64) {
    let mut state = BOOTSTRAP_SEED.wrapping_add(salt.wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut estimates = Vec::with_capacity(BOOTSTRAPS);
    for _ in 0..BOOTSTRAPS {
        let sample: Vec<f64> = (0..values.len())
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                values[((state >> 33) as usize) % values.len()]
            })
            .collect();
        estimates.push(median(&sample));
    }
    estimates.sort_by(|left, right| left.partial_cmp(right).expect("no NaN estimates"));
    let low = estimates[BOOTSTRAPS * 25 / 1000];
    let high = estimates[BOOTSTRAPS * 975 / 1000];
    (low, high)
}
