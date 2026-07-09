// An expensive, PURE (reads-only, no memory writes, no allocation, no trap)
// scalar function. Models a separately-compiled "medium" function that a real
// program will NOT inline. It computes a "gain" via a fixed-point iteration —
// deterministic, depends only on its argument.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Config {
    pub base: f64,
    pub target: f64,
    pub k: f64,
}

// The honest boundary: #[inline(never)] models a function rustc refuses to
// inline (large / separately compiled). Same crate: LLVM still sees the body
// and MAY infer `readnone` and hoist it anyway.
#[inline(never)]
pub fn compute_gain(cfg: &Config) -> f64 {
    let mut g = cfg.base;
    let mut acc = 0.0f64;
    for i in 0..256 {
        // Newton-ish fixed point on g^3 = target, pure math, no memory.
        g = g - (g * g * g - cfg.target) / (3.0 * g * g + 1e-9);
        acc += (cfg.k * g + i as f64).sin() * (cfg.k - g).cos();
    }
    g + acc * 1e-9
}
