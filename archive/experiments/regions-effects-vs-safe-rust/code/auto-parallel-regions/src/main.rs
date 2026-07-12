// auto-parallel-regions: scatter-by-permutation
//
// Workload: reorder-and-transform. `n` particles with scalar field input[i],
// a precomputed permutation `perm` (bijection of 0..n, e.g. a spatial sort).
// Produce out[perm[i]] = transform(input[i]).
//
// The write is a SCATTER by a computed index. `perm` is a true bijection, so
// every out slot is written exactly once => writes are provably DISJOINT and
// the loop is embarrassingly parallel. Safe Rust's borrow checker cannot see
// perm is injective, so it forbids out[perm[i]] = .. across threads. That
// disjointness fact is exactly what an xlang region/permutation effect carries
// into the optimizer, licensing an automatic parallel scatter with no unsafe.

use rayon::prelude::*;
use std::time::Instant;

#[inline(always)]
fn transform(x: f64, work: usize) -> f64 {
    let mut a = x;
    for _ in 0..work {
        a = a * 1.0000001 + 0.5;
        a = a - (a * a) * 1e-19;
    }
    a
}

// Send/Sync raw pointer: the "xlang ceiling" write channel. What xlang emits
// automatically once the permutation-disjointness fact proves scatter writes
// never collide.
#[derive(Copy, Clone)]
struct SendPtr(*mut f64);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}

fn verify_permutation(perm: &[u32], n: usize) -> bool {
    if perm.len() != n {
        return false;
    }
    let mut seen = vec![0u8; n];
    for &p in perm {
        let d = p as usize;
        if d >= n || seen[d] != 0 {
            return false;
        }
        seen[d] = 1;
    }
    true
}

fn verify_permutation_stamp(perm: &[u32], n: usize, seen: &mut [u32], stamp: u32) -> bool {
    if perm.len() != n || seen.len() != n || stamp == 0 {
        return false;
    }
    for &p in perm {
        let d = p as usize;
        if d >= n || seen[d] == stamp {
            return false;
        }
        seen[d] = stamp;
    }
    true
}

fn bench<F: FnMut()>(name: &str, reps: usize, mut f: F) -> f64 {
    f(); // warmup
    let mut ts = Vec::with_capacity(reps);
    for _ in 0..reps {
        let t = Instant::now();
        f();
        ts.push(t.elapsed().as_secs_f64() * 1e3);
    }
    ts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = ts[ts.len() / 2];
    println!("{:<30} median {:8.3} ms  (min {:8.3})", name, med, ts[0]);
    med
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let n: usize = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(1 << 24);
    let reps: usize = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(9);
    let work: usize = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(8);
    let p = rayon::current_num_threads();

    let input: Vec<f64> = (0..n).map(|i| (i as f64) * 1e-6 + 1.0).collect();
    let mut perm: Vec<u32> = (0..n as u32).collect();
    let mut st: u64 = 0x243F6A8885A308D3;
    for i in (1..n).rev() {
        st ^= st << 13;
        st ^= st >> 7;
        st ^= st << 17;
        perm.swap(i, (st % (i as u64 + 1)) as usize);
    }

    // reference (sequential scatter)
    let mut reference = vec![0.0f64; n];
    for i in 0..n {
        reference[perm[i] as usize] = transform(input[i], work);
    }
    let checksum: f64 = reference.iter().sum();
    println!(
        "n={} ({:.1}M) work={} threads={} checksum={:.6e}\n",
        n,
        n as f64 / 1e6,
        work,
        p,
        checksum
    );

    // reusable output buffers
    let mut out = vec![0.0f64; n];
    let mut inv = vec![0u32; n];

    let verify = |o: &[f64], name: &str| {
        if o != reference.as_slice() {
            eprintln!("MISMATCH in {}", name);
            std::process::exit(1);
        }
    };

    // ---------------- V1 BASELINE: safe sequential scatter ----------------
    let t_base = bench("V1 baseline seq-scatter", reps, || {
        for i in 0..n {
            out[perm[i] as usize] = transform(input[i], work);
        }
        std::hint::black_box(&out);
    });
    verify(&out, "baseline");

    // ---------- V2 SAFE adversary A: parallel GATHER via inverse perm ------
    // Safe Rust can't parallelize the scatter, so reformulate: build inverse
    // (inv[perm[i]]=i, itself a SERIAL scatter), then gather out[j]=f(in[inv[j]])
    // which writes each out slot once in order => par_iter_mut legal.
    let t_gather = bench("V2 safe gather+inv (A)", reps, || {
        for i in 0..n {
            inv[perm[i] as usize] = i as u32;
        }
        out.par_iter_mut()
            .zip(inv.par_iter())
            .for_each(|(o, &j)| *o = transform(input[j as usize], work));
        std::hint::black_box(&out);
    });
    verify(&out, "gather+inv");

    // isolate the parallel gather alone (inverse assumed precomputed)
    for i in 0..n {
        inv[perm[i] as usize] = i as u32;
    }
    let inv_ready = inv.clone();
    let t_gather_only = bench("V2b gather only (inv given)", reps, || {
        out.par_iter_mut()
            .zip(inv_ready.par_iter())
            .for_each(|(o, &j)| *o = transform(input[j as usize], work));
        std::hint::black_box(&out);
    });

    // ---------- V2 SAFE adversary B: output-range ownership scatter --------
    // No inverse. Split OUTPUT into P disjoint contiguous ranges via
    // par_chunks_mut; each thread scans ALL input and writes only the elements
    // whose dest falls in its range. Fully safe + parallel, but does O(n*P)
    // redundant streaming reads of perm/input.
    let nchunks = p;
    let chunk = (n + nchunks - 1) / nchunks;
    let t_own = bench("V2c safe range-own scatter (B)", reps, || {
        out.par_chunks_mut(chunk)
            .enumerate()
            .for_each(|(c, oc)| {
                let lo = c * chunk;
                let hi = lo + oc.len();
                for i in 0..n {
                    let d = perm[i] as usize;
                    if d >= lo && d < hi {
                        oc[d - lo] = transform(input[i], work);
                    }
                }
            });
        std::hint::black_box(&out);
    });
    verify(&out, "range-own");

    // ---------- V2 SAFE adversary D: two-phase bucketed partition ----------
    // The sophisticated expert form (safe radix-style scatter). Phase 1: each
    // thread scans a contiguous INPUT chunk once and splits (dest,value) into P
    // thread-local buckets keyed by dest-range. Phase 2: each thread OWNS one
    // output range (par_chunks_mut) and drains everyone's bucket-c into it.
    // Fully safe + parallel, each element read O(1) times -- but writes the
    // payload twice (into bucket, then into out) and allocates P*P vecs.
    let span = chunk; // output range width per bucket (== nchunks buckets)
    let in_chunk = (n + p - 1) / p;
    let t_bucket = bench("V2d safe bucketed 2-phase (D)", reps, || {
        // phase 1: parallel partition into per-thread buckets
        let buckets: Vec<Vec<Vec<(u32, f64)>>> = (0..p)
            .into_par_iter()
            .map(|t| {
                let lo = t * in_chunk;
                let hi = (lo + in_chunk).min(n);
                let mut b: Vec<Vec<(u32, f64)>> = (0..nchunks)
                    .map(|_| Vec::with_capacity(in_chunk / nchunks + 16))
                    .collect();
                for i in lo..hi {
                    let d = perm[i] as usize;
                    b[d / span].push((d as u32, transform(input[i], work)));
                }
                b
            })
            .collect();
        // phase 2: each thread owns a disjoint output range, drains bucket-c
        out.par_chunks_mut(chunk).enumerate().for_each(|(c, oc)| {
            let lo = c * chunk;
            for tb in &buckets {
                for &(d, v) in &tb[c] {
                    oc[d as usize - lo] = v;
                }
            }
        });
        std::hint::black_box(&out);
    });
    verify(&out, "bucketed");

    // ---------------- V3 XLANG CEILING: parallel scatter -------------------
    // What xlang emits automatically from the permutation fact: split the input
    // range across threads, each scatters out[perm[i]]. Disjoint => no races.
    let t_scatter = bench("V3 xlang par-scatter", reps, || {
        let base = SendPtr(out.as_mut_ptr());
        (0..n).into_par_iter().for_each(|i| {
            let base = base;
            let d = perm[i] as usize;
            let v = transform(input[i], work);
            // SAFETY: perm is a bijection of 0..n => unique d per i, no collisions.
            unsafe {
                *base.0.add(d) = v;
            }
        });
        std::hint::black_box(&out);
    });
    verify(&out, "par-scatter");

    // -------- V4 AI-native guarded scatter: verify fact, then dispatch -----
    // This is the source-bundle version of V3: the AI proposes the direct
    // scatter plan, but the compiler/runtime must first prove or guard the
    // bijection fact. This conservative measurement pays the O(n) guard on
    // every dispatch. A real immutable permutation could cache the proof.
    let t_guard = bench("V4 guard verify perm", reps, || {
        let ok = verify_permutation(&perm, n);
        std::hint::black_box(ok);
        if !ok {
            eprintln!("invalid permutation");
            std::process::exit(1);
        }
    });

    let t_guarded_scatter = bench("V4 guarded par-scatter", reps, || {
        if verify_permutation(&perm, n) {
            let base = SendPtr(out.as_mut_ptr());
            (0..n).into_par_iter().for_each(|i| {
                let base = base;
                let d = perm[i] as usize;
                let v = transform(input[i], work);
                // SAFETY: verify_permutation proved perm is a bijection of 0..n.
                unsafe {
                    *base.0.add(d) = v;
                }
            });
        } else {
            for i in 0..n {
                out[perm[i] as usize] = transform(input[i], work);
            }
        }
        std::hint::black_box(&out);
    });
    verify(&out, "guarded-par-scatter");

    // Reusable verifier state: avoids per-dispatch allocation and memset by
    // using a monotonically-increasing stamp in a u32 side table. This models a
    // runtime guard template with scratch storage attached to the proof site.
    let mut seen_stamps = vec![0u32; n];
    let mut stamp = 1u32;
    let t_guard_reuse = bench("V4b guard stamp reuse", reps, || {
        let ok = verify_permutation_stamp(&perm, n, &mut seen_stamps, stamp);
        std::hint::black_box(ok);
        if !ok {
            eprintln!("invalid permutation");
            std::process::exit(1);
        }
        stamp = stamp.wrapping_add(1);
        if stamp == 0 {
            seen_stamps.fill(0);
            stamp = 1;
        }
    });

    let mut seen_stamps = vec![0u32; n];
    let mut stamp = 1u32;
    let t_guarded_scatter_reuse = bench("V4b guarded stamp scatter", reps, || {
        if verify_permutation_stamp(&perm, n, &mut seen_stamps, stamp) {
            stamp = stamp.wrapping_add(1);
            if stamp == 0 {
                seen_stamps.fill(0);
                stamp = 1;
            }
            let base = SendPtr(out.as_mut_ptr());
            (0..n).into_par_iter().for_each(|i| {
                let base = base;
                let d = perm[i] as usize;
                let v = transform(input[i], work);
                // SAFETY: verify_permutation_stamp proved perm is a bijection of 0..n.
                unsafe {
                    *base.0.add(d) = v;
                }
            });
        } else {
            for i in 0..n {
                out[perm[i] as usize] = transform(input[i], work);
            }
        }
        std::hint::black_box(&out);
    });
    verify(&out, "guarded-stamp-par-scatter");

    let best_safe = t_gather.min(t_own).min(t_base).min(t_bucket);
    println!("\n--- summary (median ms), n={}, work={}, p={} ---", n, work, p);
    println!("V1  baseline seq-scatter   : {:8.3}", t_base);
    println!("V2  safe gather+inv        : {:8.3}", t_gather);
    println!("V2b   gather only (no inv) : {:8.3}", t_gather_only);
    println!("V2c safe range-own scatter : {:8.3}", t_own);
    println!("V2d safe bucketed 2-phase  : {:8.3}", t_bucket);
    println!("V3  xlang par-scatter      : {:8.3}", t_scatter);
    println!("V4  guard verify perm      : {:8.3}", t_guard);
    println!("V4  guarded par-scatter    : {:8.3}", t_guarded_scatter);
    println!("V4b guard stamp reuse      : {:8.3}", t_guard_reuse);
    println!("V4b guarded stamp scatter  : {:8.3}", t_guarded_scatter_reuse);
    println!("best safe adversary        : {:8.3}", best_safe);
    println!("\nbest-safe / xlang-ceiling  : {:.2}x", best_safe / t_scatter);
    println!("best-safe / guarded-scatter: {:.2}x", best_safe / t_guarded_scatter);
    println!("best-safe / guarded-stamp  : {:.2}x", best_safe / t_guarded_scatter_reuse);
    println!("guarded / xlang-ceiling    : {:.2}x", t_guarded_scatter / t_scatter);
    println!("guarded-stamp / ceiling    : {:.2}x", t_guarded_scatter_reuse / t_scatter);
    println!("baseline  / xlang-ceiling  : {:.2}x", t_base / t_scatter);
}
