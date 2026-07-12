// Isolate the FREE cost directly: build the whole forest, hold it alive,
// then time ONLY the teardown. Build phase is untimed. Memory bounded to one
// live forest per run (freed before next run), so no leak accumulation.

use std::hint::black_box;
use std::time::Instant;

const DEPTH: u32 = 12;   // 8191 nodes/tree
const TREES: usize = 1000; // ~8.19M nodes total
const RUNS: usize = 15;

#[inline(always)]
fn next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    *state = x; x
}

mod boxed {
    use super::next;
    pub enum Expr { Num(i64), Add(Box<Expr>, Box<Expr>), Mul(Box<Expr>, Box<Expr>) }
    pub fn build(d: u32, st: &mut u64) -> Box<Expr> {
        if d == 0 { Box::new(Expr::Num((next(st) & 0xff) as i64)) }
        else { let l = build(d-1, st); let r = build(d-1, st);
            if next(st) & 1 == 0 { Box::new(Expr::Add(l, r)) } else { Box::new(Expr::Mul(l, r)) } }
    }
}

mod region {
    use super::next;
    #[derive(Clone, Copy)]
    pub enum Expr { Num(i64), Add(u32, u32), Mul(u32, u32) }
    pub fn build(p: &mut Vec<Expr>, d: u32, st: &mut u64) -> u32 {
        if d == 0 { let id = p.len() as u32; p.push(Expr::Num((next(st)&0xff) as i64)); id }
        else { let l = build(p, d-1, st); let r = build(p, d-1, st); let id = p.len() as u32;
            let n = if next(st)&1==0 { Expr::Add(l,r) } else { Expr::Mul(l,r) }; p.push(n); id }
    }
}

fn time_free_box(seed: u64) -> f64 {
    let mut st = seed;
    let mut forest: Vec<Box<boxed::Expr>> = Vec::with_capacity(TREES);
    for _ in 0..TREES { forest.push(boxed::build(DEPTH, &mut st)); }
    let t0 = Instant::now();
    drop(black_box(forest)); // per-node recursive Drop, 8.19M frees
    t0.elapsed().as_secs_f64() * 1e3
}

fn time_free_region(seed: u64) -> f64 {
    let mut st = seed;
    let mut pool: Vec<region::Expr> = Vec::new();
    let mut roots = Vec::with_capacity(TREES);
    for _ in 0..TREES { roots.push(region::build(&mut pool, DEPTH, &mut st)); }
    black_box(&roots);
    let t0 = Instant::now();
    drop(black_box(pool)); // ONE free of the backing buffer, no destructors
    t0.elapsed().as_secs_f64() * 1e3
}

fn med(name: &str, mut f: impl FnMut() -> f64) -> f64 {
    for _ in 0..3 { black_box(f()); }
    let mut ts: Vec<f64> = (0..RUNS).map(|_| f()).collect();
    ts.sort_by(|a,b| a.partial_cmp(b).unwrap());
    let m = ts[ts.len()/2];
    println!("{name:<32} free = {m:8.4} ms", );
    m
}

fn main() {
    let seed = 0x9E3779B97F4A7C15u64;
    let nodes = ((1u64 << (DEPTH+1)) - 1) as usize * TREES;
    println!("Pure teardown of {nodes} live nodes (median of {RUNS}, ms)\n");
    let b = med("Box/Drop per-node free", || time_free_box(seed));
    let r = med("region bulk free (Vec drop)", || time_free_region(seed));
    println!("\nfree-cost ratio  Box/region = {:.0}x", b / r.max(1e-9));
    println!("region free is effectively O(1): {:.4} ms for {} nodes", r, nodes);
}
