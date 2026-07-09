use puremath::{compute_gain, Config};
use std::hint::black_box;
use std::time::Instant;

// The opaque dynamic-library symbol (true separately-compiled boundary).
extern "C" {
    fn compute_gain_dyn(cfg: *const Config) -> f64;
}

const N: usize = 2_000_000; // pixels
const REPS: usize = 9;

// ---- Workload: apply a loop-invariant "gain" to every pixel. -------------
// out[i] = gain(cfg) * input[i]. gain is loop-invariant & PURE.

// (1) BASELINE, same crate: natural code — the invariant call sits in the loop.
//     Tests whether rustc/LLVM auto-hoists an #[inline(never)] same-crate call
//     via readnone attribute inference (LTO on).
#[inline(never)]
fn baseline_samecrate(input: &[f64], out: &mut [f64], cfg: &Config) {
    for i in 0..input.len() {
        out[i] = compute_gain(cfg) * input[i];
    }
}

// (2) BEST-EFFORT SAFE RUST == (3) XLANG CEILING: hoist by hand (one line).
//     This is exactly the code xlang's effect row produces automatically.
#[inline(never)]
fn hoisted(input: &[f64], out: &mut [f64], cfg: &Config) {
    let g = compute_gain(cfg);
    for i in 0..input.len() {
        out[i] = g * input[i];
    }
}

// (4) BASELINE, dylib: same natural code but the call crosses a true dynamic
//     boundary. Opaque `bl` — no purity attrs. LTO cannot cross it.
#[inline(never)]
fn baseline_dylib(input: &[f64], out: &mut [f64], cfg: &Config) {
    for i in 0..input.len() {
        out[i] = unsafe { compute_gain_dyn(cfg as *const Config) } * input[i];
    }
}

// (5) BEST-EFFORT SAFE RUST across dylib: human hoists the opaque call.
#[inline(never)]
fn hoisted_dylib(input: &[f64], out: &mut [f64], cfg: &Config) {
    let g = unsafe { compute_gain_dyn(cfg as *const Config) };
    for i in 0..input.len() {
        out[i] = g * input[i];
    }
}

fn checksum(out: &[f64]) -> f64 {
    out.iter().sum()
}

fn bench<F: FnMut(&[f64], &mut [f64], &Config)>(
    name: &str,
    input: &[f64],
    out: &mut [f64],
    cfg: &Config,
    mut f: F,
) -> (f64, f64) {
    // warmup
    f(input, out, cfg);
    let mut times = Vec::new();
    let mut sum = 0.0;
    for _ in 0..REPS {
        let t = Instant::now();
        f(black_box(input), black_box(out), black_box(cfg));
        let e = t.elapsed().as_secs_f64() * 1e3;
        times.push(e);
        sum = checksum(out);
        black_box(sum);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = times[times.len() / 2];
    println!("{:<26} median = {:>10.4} ms   checksum = {:.10}", name, median, sum);
    (median, sum)
}

fn main() {
    let cfg = Config { base: 1.3, target: 2.0, k: 0.7 };
    let input: Vec<f64> = (0..N).map(|i| ((i % 997) as f64) * 1e-3 + 0.5).collect();
    let mut out = vec![0.0f64; N];

    // sanity: identical gain both paths
    let g1 = compute_gain(&cfg);
    let g2 = unsafe { compute_gain_dyn(&cfg as *const Config) };
    println!("gain samecrate = {:.15}, gain dylib = {:.15}, equal = {}", g1, g2, g1 == g2);
    println!("N = {}, reps = {}\n", N, REPS);

    let (_, c1) = bench("1 baseline_samecrate", &input, &mut out, &cfg, baseline_samecrate);
    let (_, c2) = bench("2 hoisted (==ceiling)", &input, &mut out, &cfg, hoisted);
    let (_, c3) = bench("4 baseline_dylib", &input, &mut out, &cfg, baseline_dylib);
    let (_, c4) = bench("5 hoisted_dylib", &input, &mut out, &cfg, hoisted_dylib);

    println!(
        "\nbit-identical outputs: {}",
        c1 == c2 && c2 == c3 && c3 == c4
    );
}
