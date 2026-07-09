// pure-fusion kernel: pipeline of pure element-wise transforms over a large array.
// Compares:
//   A BASELINE      : each stage a separate Vec-returning fn (materializes intermediates)
//   A2 BASELINE-inl : same but not inline(never) (let rustc try to fuse/elide)
//   B BEST-EFFORT   : single fused safe iterator chain (no unsafe)
//   C XLANG-CEILING : hand fused single-pass loop into preallocated buffer
//   D DYN-PIPELINE  : Vec<Box<dyn Fn>> plugin-style boundary (iterators can't fuse)
// All produce bit-identical f64 output.

use std::hint::black_box;
use std::time::Instant;

const N: usize = 32 * 1024 * 1024;

// Cheap element-wise ops => pipeline is MEMORY-BANDWIDTH bound, so the
// number of passes / intermediate arrays dominates (fusion's real lever).
#[inline(always)]
fn s1(x: f64) -> f64 { x * 1.5 + 0.3 }
#[inline(always)]
fn s2(x: f64) -> f64 { x.mul_add(0.5, 1.0) }
#[inline(always)]
fn s3(x: f64) -> f64 { x * 0.998 - 0.5 }
#[inline(always)]
fn s4(x: f64) -> f64 { x.mul_add(1.001, 0.25) }
#[inline(always)]
fn s5(x: f64) -> f64 { x.mul_add(0.75, -0.125) }
#[inline(always)]
fn s6(x: f64) -> f64 { x.mul_add(1.03, 0.01) }

// ---- A: baseline, separate non-inlinable Vec-returning stages ----
#[inline(never)]
fn a1(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s1(x)).collect() }
#[inline(never)]
fn a2(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s2(x)).collect() }
#[inline(never)]
fn a3(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s3(x)).collect() }
#[inline(never)]
fn a4(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s4(x)).collect() }
#[inline(never)]
fn a5(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s5(x)).collect() }
#[inline(never)]
fn a6(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s6(x)).collect() }

fn baseline(input: &[f64]) -> Vec<f64> {
    let v = a1(input);
    let v = a2(&v);
    let v = a3(&v);
    let v = a4(&v);
    let v = a5(&v);
    a6(&v)
}

// ---- A2: same pipeline but let inliner do what it wants ----
#[inline]
fn b1(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s1(x)).collect() }
#[inline]
fn b2(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s2(x)).collect() }
#[inline]
fn b3(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s3(x)).collect() }
#[inline]
fn b4(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s4(x)).collect() }
#[inline]
fn b5(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s5(x)).collect() }
#[inline]
fn b6(input: &[f64]) -> Vec<f64> { input.iter().map(|&x| s6(x)).collect() }

fn baseline_inl(input: &[f64]) -> Vec<f64> {
    let v = b1(input);
    let v = b2(&v);
    let v = b3(&v);
    let v = b4(&v);
    let v = b5(&v);
    b6(&v)
}

// ---- B: best-effort safe fused iterator chain ----
fn fused_iter(input: &[f64]) -> Vec<f64> {
    input.iter()
        .map(|&x| s1(x))
        .map(s2).map(s3).map(s4).map(s5).map(s6)
        .collect()
}

// ---- C: xlang ceiling: manual single-pass loop into preallocated buffer ----
fn fused_loop(input: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0f64; input.len()];
    for i in 0..input.len() {
        out[i] = s6(s5(s4(s3(s2(s1(input[i]))))));
    }
    out
}

// ---- D: dynamic plugin pipeline (opaque boundary; iterators cannot fuse) ----
fn dyn_pipeline(input: &[f64]) -> Vec<f64> {
    let stages: Vec<Box<dyn Fn(f64) -> f64>> = vec![
        Box::new(s1), Box::new(s2), Box::new(s3),
        Box::new(s4), Box::new(s5), Box::new(s6),
    ];
    // natural plugin style: each stage produces a Vec
    let mut cur: Vec<f64> = input.to_vec();
    for st in &stages {
        cur = cur.iter().map(|&x| st(x)).collect();
    }
    cur
}

fn checksum(v: &[f64]) -> u64 {
    // bit-exact reduction independent of order (xor of bits)
    let mut acc: u64 = 0;
    for &x in v { acc ^= x.to_bits().wrapping_mul(0x9E3779B97F4A7C15); }
    acc
}

fn bench<F: Fn(&[f64]) -> Vec<f64>>(name: &str, input: &[f64], f: F, want: u64) {
    // warmup
    for _ in 0..2 { let r = f(black_box(input)); black_box(checksum(&r)); }
    let mut times = Vec::new();
    for _ in 0..9 {
        let t = Instant::now();
        let r = f(black_box(input));
        let cs = checksum(black_box(&r));
        let dt = t.elapsed();
        assert_eq!(cs, want, "MISMATCH in {name}");
        black_box(cs);
        times.push(dt.as_secs_f64() * 1e3);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("{:<18} median {:8.2} ms   min {:8.2} ms", name, times[times.len()/2], times[0]);
}

fn main() {
    let input: Vec<f64> = (0..N).map(|i| ((i as f64) * 0.001).sin() * 100.0).collect();

    let want = checksum(&fused_loop(&input));
    // verify all agree bit-exactly
    assert_eq!(checksum(&baseline(&input)), want, "baseline mismatch");
    assert_eq!(checksum(&baseline_inl(&input)), want, "baseline_inl mismatch");
    assert_eq!(checksum(&fused_iter(&input)), want, "fused_iter mismatch");
    assert_eq!(checksum(&dyn_pipeline(&input)), want, "dyn mismatch");
    println!("verify OK, N={} ({} MB per array), checksum={:#x}\n", N, N*8/1024/1024, want);

    bench("A baseline",     &input, baseline, want);
    bench("A2 baseline-inl",&input, baseline_inl, want);
    bench("D dyn-pipeline", &input, dyn_pipeline, want);
    bench("B fused-iter",   &input, fused_iter, want);
    bench("C fused-loop",   &input, fused_loop, want);
}
