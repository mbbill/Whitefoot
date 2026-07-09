// region-bulk-free: build many short-lived expression ASTs, evaluate, discard.
//
// Workload: a compiler-front-end-ish pass. Per "batch" we build TREES trees,
// each a full binary expression tree of DEPTH (2^DEPTH - 1 nodes). We evaluate
// each tree (folds the whole tree -> forces every node to be touched, defeats
// DCE) and then discard it. This is the canonical "build a large short-lived
// graph of many small owned nodes, use it, drop it, many times" pattern.
//
// The three allocation strategies differ ONLY in how nodes are allocated and
// freed. Evaluation is byte-for-byte identical logic across all three.

use std::hint::black_box;
use std::time::Instant;

const DEPTH: u32 = 12; // 4095 nodes/tree
const TREES: usize = 1000; // ~4.09M nodes per batch
const RUNS: usize = 15;

// deterministic tiny PRNG so all versions build identical trees & identical sums
#[inline(always)]
fn next(state: &mut u64) -> u64 {
    // xorshift64
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

// ---------------------------------------------------------------------------
// VERSION 1: BASELINE — idiomatic safe Rust, Box tree, per-node Drop.
// ---------------------------------------------------------------------------
mod boxed {
    use super::next;
    pub enum Expr {
        Num(i64),
        Add(Box<Expr>, Box<Expr>),
        Mul(Box<Expr>, Box<Expr>),
    }
    fn build(depth: u32, st: &mut u64) -> Box<Expr> {
        if depth == 0 {
            Box::new(Expr::Num((next(st) & 0xff) as i64))
        } else {
            let l = build(depth - 1, st);
            let r = build(depth - 1, st);
            if next(st) & 1 == 0 {
                Box::new(Expr::Add(l, r))
            } else {
                Box::new(Expr::Mul(l, r))
            }
        }
    }
    fn eval(e: &Expr) -> i64 {
        match e {
            Expr::Num(n) => *n,
            Expr::Add(a, b) => eval(a).wrapping_add(eval(b)),
            Expr::Mul(a, b) => eval(a).wrapping_mul(eval(b)),
        }
    }
    // returns (checksum, alloc+build+eval+free elapsed already timed by caller)
    pub fn run_batch(depth: u32, trees: usize, seed: u64) -> i64 {
        let mut st = seed;
        let mut acc: i64 = 0;
        for _ in 0..trees {
            let t = build(depth, &mut st);
            acc = acc.wrapping_add(eval(&t));
            // t dropped here: recursive per-node free
        }
        acc
    }
}

// ---------------------------------------------------------------------------
// VERSION 2: BEST-EFFORT SAFE RUST — bumpalo arena, bulk reset (no per-node
// free, no per-node drop glue). This is the adversary trying to tie xlang.
// Uses `reset()` to reuse the region across trees within a batch, then the
// Bump is dropped once at batch end (single bulk free of the chunks).
// ---------------------------------------------------------------------------
mod bump {
    use super::next;
    use bumpalo::Bump;
    pub enum Expr<'a> {
        Num(i64),
        Add(&'a Expr<'a>, &'a Expr<'a>),
        Mul(&'a Expr<'a>, &'a Expr<'a>),
    }
    fn build<'a>(b: &'a Bump, depth: u32, st: &mut u64) -> &'a Expr<'a> {
        if depth == 0 {
            b.alloc(Expr::Num((next(st) & 0xff) as i64))
        } else {
            let l = build(b, depth - 1, st);
            let r = build(b, depth - 1, st);
            if next(st) & 1 == 0 {
                b.alloc(Expr::Add(l, r))
            } else {
                b.alloc(Expr::Mul(l, r))
            }
        }
    }
    fn eval(e: &Expr) -> i64 {
        match e {
            Expr::Num(n) => *n,
            Expr::Add(a, b) => eval(a).wrapping_add(eval(b)),
            Expr::Mul(a, b) => eval(a).wrapping_mul(eval(b)),
        }
    }
    // Fresh Bump per batch (allocated + fully freed each batch).
    pub fn run_batch(depth: u32, trees: usize, seed: u64) -> i64 {
        let mut st = seed;
        let mut acc: i64 = 0;
        let mut b = Bump::new();
        for _ in 0..trees {
            {
                let t = build(&b, depth, &mut st);
                acc = acc.wrapping_add(eval(t));
            }
            b.reset(); // bulk "free" of this tree's nodes (keeps chunk for reuse)
        }
        acc // b dropped here -> chunk(s) returned to allocator (one bulk free)
    }
}

// ---------------------------------------------------------------------------
// VERSION 3: XLANG-EQUIVALENT CEILING — what xlang's region+effect optimizer
// would emit automatically. A monomorphic bump arena over a Vec<Expr> pool:
// nodes are POD (no Drop glue, region-local per the effect system), the whole
// region is a contiguous Vec, and "free" is truncating the Vec (bulk, O(1),
// no destructors). Children referenced by u32 index (region-relative ptr).
// This is 100% SAFE Rust and models the exact codegen the region facts license.
// ---------------------------------------------------------------------------
mod region {
    use super::next;
    #[derive(Clone, Copy)]
    pub enum Expr {
        Num(i64),
        Add(u32, u32),
        Mul(u32, u32),
    }
    // The region is just a growable pool; alloc = push (bump), free = clear.
    fn build(pool: &mut Vec<Expr>, depth: u32, st: &mut u64) -> u32 {
        if depth == 0 {
            let id = pool.len() as u32;
            pool.push(Expr::Num((next(st) & 0xff) as i64));
            id
        } else {
            let l = build(pool, depth - 1, st);
            let r = build(pool, depth - 1, st);
            let id = pool.len() as u32;
            let node = if next(st) & 1 == 0 {
                Expr::Add(l, r)
            } else {
                Expr::Mul(l, r)
            };
            pool.push(node);
            id
        }
    }
    fn eval(pool: &[Expr], id: u32) -> i64 {
        match pool[id as usize] {
            Expr::Num(n) => n,
            Expr::Add(a, b) => eval(pool, a).wrapping_add(eval(pool, b)),
            Expr::Mul(a, b) => eval(pool, a).wrapping_mul(eval(pool, b)),
        }
    }
    pub fn run_batch(depth: u32, trees: usize, seed: u64) -> i64 {
        let mut st = seed;
        let mut acc: i64 = 0;
        let mut pool: Vec<Expr> = Vec::new();
        for _ in 0..trees {
            let root = build(&mut pool, depth, &mut st);
            acc = acc.wrapping_add(eval(&pool, root));
            pool.clear(); // bulk free: O(1), no per-node destructor
        }
        acc // pool dropped: one free of the backing buffer
    }
}

fn bench<F: FnMut() -> i64>(name: &str, expect: i64, mut f: F) -> f64 {
    // warmup
    for _ in 0..3 {
        black_box(f());
    }
    let mut times = Vec::with_capacity(RUNS);
    for _ in 0..RUNS {
        let t0 = Instant::now();
        let r = black_box(f());
        let dt = t0.elapsed().as_secs_f64() * 1e3; // ms
        assert_eq!(r, expect, "{name} checksum mismatch");
        times.push(dt);
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = times[times.len() / 2];
    let min = times[0];
    println!("{name:<28} median {med:8.3} ms   min {min:8.3} ms");
    med
}

fn main() {
    let seed = 0x9E3779B97F4A7C15u64;
    // establish the shared checksum from the baseline
    let expect = boxed::run_batch(DEPTH, TREES, seed);
    let bump_chk = bump::run_batch(DEPTH, TREES, seed);
    let region_chk = region::run_batch(DEPTH, TREES, seed);
    assert_eq!(expect, bump_chk, "bump != baseline");
    assert_eq!(expect, region_chk, "region != baseline");
    let nodes = ((1u64 << (DEPTH + 1)) - 1) as usize * TREES;
    println!(
        "workload: {TREES} trees x depth {DEPTH} = {nodes} nodes/batch; checksum {expect:#x}; identical across all versions\n"
    );

    let b = bench("1 BASELINE Box/Drop", expect, || {
        boxed::run_batch(DEPTH, TREES, seed)
    });
    let a = bench("2 SAFE bumpalo (reset)", expect, || {
        bump::run_batch(DEPTH, TREES, seed)
    });
    let r = bench("3 XLANG region (Vec pool)", expect, || {
        region::run_batch(DEPTH, TREES, seed)
    });

    println!("\n--- ratios (higher = slower vs region ceiling) ---");
    println!("baseline / region : {:.2}x", b / r);
    println!("bumpalo  / region : {:.2}x", a / r);
    println!("baseline / bumpalo: {:.2}x", b / a);
}
