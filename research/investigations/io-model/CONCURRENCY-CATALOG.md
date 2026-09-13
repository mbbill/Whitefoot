<!-- Serves the I/O API design: before the API is chosen, this catalog prices
     what Whitefoot's concurrency model (sequential source, checker-derived
     fork-join overlap, state as data, [PAR-3] set aside) can and cannot
     express, against the architectures real C/C++/Rust systems programs use.
     Analysis, not a record: measured claims cite this repository's runs;
     estimates are marked. Superseded by the API design that consumes it. -->

# Concurrency architecture catalog for Whitefoot

What this is: twenty-one concurrency architectures that real C/C++/Rust systems
programs are built out of, each written once in its native idiom and once in
Whitefoot's, with the structural cost and the performance expectation stated
separately. It exists to be argued with before the I/O API is designed, so
every claim that can be grounded in a measured number in this repository is,
and every claim that cannot is labelled an estimate with its reasoning.

Nothing here proposes changing the model. Where a rule costs something, the
cost is priced, not appealed.

---

## 0. The model, restated operationally

### 0.1 The three overlap rules are the entire toolbox

The specification gives three permissions and no others. Read them as three
*shapes a program can be written in* rather than as three features.

**[PAR-1] — the statement window.** Two `let` statements in one block, each
one call, may overlap when their footprints and loans do not interfere, and
every statement written between them is judged by the same conditions. The
arity of the parallelism is the number of calls the writer wrote. The writer
chooses the decomposition; the compiler only grants or denies it.

**[PAR-2] — the counted loop.** Iterations of one `for` may overlap when every
place the body writes is either a binding the body itself introduces, one
proved single-binder affine element write `a[c*i + b]` with `a` and `b`
*mathematical integer constants*, or one accumulator combined with exactly one
of `+wrap *wrap iand ior ixor imin imax band bor bxor`. The arity is the
compiler's: the measured programs emit 16 to 64 chunks over a 98k-to-524k
iteration space regardless of worker count.

**[PAR-3] is set aside and is not used anywhere below.** The specification
carries a staged-loop permission that cuts a body at its first `may-suspend`
submission and keeps K stackful sessions in flight. The owner has set it aside,
so this catalog treats it as nonexistent: architectures 17 through 20 are
derived from the brief's explicit I/O model instead, stated next. Where a
measured number in this repository was produced by the staged-loop runtime, it
is labelled as belonging to a retired design and is not used as this model's
expectation.

**The I/O model, as the brief fixes it.** An operation is issued without
blocking by `op_start`, which returns an owned `Pending` that owns or
exclusively borrows its buffer. `op_finish(p)` blocks until that one completes.
`wait_batch(&uniq slots)` blocks until at least one of a set of pendings
completes and returns which. Blocking blocks the whole program, because there is
one control flow and nothing else needs it. Per-connection state is a row in an
array, never a thread and never a stack. The shape every I/O architecture below
takes is therefore: **`wait_batch` → a [PAR-2] loop over the completed batch,
each iteration writing its own row and its own buffer → a sequential pass that
issues the next operations.** A deep function either does blocking I/O in place
(start plus finish) or returns the operation it wants as data for an outer loop
to issue.

Four consequences do most of the work in this document:

1. **No two overlapping executions ever write one place.** Every architecture
   whose native form depends on concurrent mutation of one structure
   (concurrent hash map, lock-free ring, RCU-protected pointer swap, actor
   mailbox) must become *private state plus a deterministic merge*. That is
   never a translation; it is a different algorithm with a different cost
   model.
2. **No control flow outlives its fork-join.** Every architecture whose native
   form is "a thread that lives across many work items and holds state"
   (pipeline stage, actor, background timer, per-core reactor) must become *a
   phase inside an outer sequential loop, with the thread's state as a row in
   an array*.
3. **PAR-2 hands out elements, PAR-1 hands out ranges.** A per-lane
   *contiguous range* of an output buffer is not reachable from PAR-2 at all:
   an iteration that holds an exclusive loan on a place rooted outside the loop
   denies permission, and `mut_slice_of(&uniq out, lo, hi)` forms exactly such
   a loan. So any pattern needing per-lane scratch or a per-lane output window
   — histogram, scan writeback, sample sort, per-shard insert, 2-D band
   decomposition — drops to PAR-1, **and PAR-1's arity is written in the
   source**. The writer hard-codes the lane count. This is the single most
   frequently recurring cost below — **and the one cost in this document that a
   single extension would remove.** The owner has accepted range loans
   (`mut_slice_of(&uniq buf, start, end)`, disjointness a linear-arithmetic
   obligation). If [PAR-2] additionally admitted an *iteration-exclusive loan on
   an affine range* `[c*i + b, c*(i+1) + b)`, proved disjoint by the same
   arithmetic it already uses for the element case, the per-lane window would come
   from the loop index and W would stop being a source constant. Every place this
   matters carries a marked `[new]` note saying exactly what the verdict becomes:
   architectures **7, 13″, 14′ and 15**, and transformations **T2 and T7**. The
   summary table marks those rows with **`[R]`**.
4. **Permission is not obligation and denial is silent.** A denied loop is not
   a rejected program; it is a program that runs sequentially. The writer's
   only feedback channel is `whitefootc --par-ledger`, which prints
   `PAR permitted` / `PAR denied  <condition>` per site. A program can lose 4x
   to a one-character change with no diagnostic. Treat the ledger as part of
   the language surface for every architecture below.

### 0.2 One rule that will come up repeatedly: the affine constant

PAR-2 admits `set out[a*i + b] = …` only when `a` and `b` are *mathematical
integer constants*. It does not admit a loop-invariant `b`. Concretely:

```whitefoot
for (x in 0_u64..width) {
  set next[y *wrap width +wrap x] = …   // DENIED: y*width is symbolic
}
```

Every 2-D kernel written over a flat row-major buffer is denied on both the
inner and the outer loop by this sentence. All four measured compute kernels
(`mandelbrot`, `fir`, `records`, `quadrature`) write `out[i]` with `a=1, b=0`,
so none of them exercises this edge, and the scoreboard therefore says nothing
about it. I flag it at architectures 13, 14 and 15, where it is load-bearing.

### 0.3 The measured baseline I reason from

From `research/investigations/compute-runtime/RESULTS.md` on `origin/main`, the
sections dated **2026-09-12** — i.e. after the recursive-grain default, the idle
window and the split work unit landed (PRs #43 / #49). `ratio` is a median of
within-pass matched pairs against the best parallel reference; lower is better
for Whitefoot. The earlier tables at `11d1e4a2`, which read `quadrature` at
1.581 / 1.565, are superseded and are not used anywhere in this document.

Hosted `ubuntu-24.04`, Linux x86_64, 4 logical CPUs (2 cores × 2 SMT), at
`fd00c10c`:

| run | kernel | W | wf median | best reference | ratio |
|---|---|---:|---:|---:|---:|
| 34667725821 (2 ms gap) | `quadrature` | 2 | 5,551.1 µs | 5,733.5 µs (`rayon-join`) | **0.969 — FASTEST = wf** |
| 34667725821 (2 ms gap) | `quadrature` | 4 | 5,757.1 µs | 5,874.9 µs (`rayon-join`) | **0.980 — FASTEST = wf** |
| 34668156035 (500 µs gap) | `quadrature` | 2 | 5,802.8 µs | 5,775.7 µs (`rayon-join`) | 1.005 |
| 34668156035 (500 µs gap) | `quadrature` | 4 | 6,034.4 µs | 5,860.2 µs (`rayon-join`) | 1.029 |
| 34668796717 (window covers the gap) | `quadrature` | 2 | 7,480.0 µs | 7,297.6 µs (`rayon-join`) | 1.026 |
| 34668796717 (window covers the gap) | `quadrature` | 4 | 7,568.6 µs | 7,376.2 µs (`rayon-join`) | 1.024 |

Local **Apple M1 Pro**, Darwin arm64, 8 logical CPUs, the two idle-window
tables of 2026-09-12:

| table | kernel | W | wf median | best reference | ratio |
|---|---|---:|---:|---:|---:|
| 2 ms call gap | `quadrature` | 2 | 4,788.5 µs | 4,692.0 µs (`parlay-left`) | 1.032 |
| 2 ms call gap | `quadrature` | 4 | 3,182.8 µs | 3,126.0 µs (`rayon-join`) | 1.063 |
| 500 µs call gap | `quadrature` | 2 | 4,487.7 µs | 4,465.7 µs (`parlay-left`) | 1.000 |
| 500 µs call gap | `quadrature` | 4 | 3,068.4 µs | 2,871.2 µs (`rayon-join`) | 1.073 |
| 2 ms call gap | `quadrature` | 1 | 8,360.2 µs | 8,247.3 µs (`serial`) | **1.014** |

The map kernels on the same M1 Pro table read `mandelbrot` 1.012 at W=2 and
1.023 at W=4, and the section's own summary records mandelbrot 0.943,
quadrature **0.771**, records 0.961 and fir 0.925 for the A/B arm at W≤4.

**The brief's framing now holds for the recursive kernel too, at W≤4.**
`quadrature` reads **0.969–1.029 on hosted ubuntu** (fastest in the 2 ms-gap
run at both W=2 and W=4) and **1.000–1.073 on the M1 Pro**. The W=1 `--par`
tax that the superseded tables showed is gone as well: 8,360.2 µs against
`serial`'s 8,247.3 µs on the M1 Pro is **1.4%**.

**What is still measured as bad for the recursive kernel is high W on the
asymmetric host.** On the M1 Pro's asymmetric-core section, with the idle window
withheld, `quadrature` reads **1.352 against `parlay` at W=8** (the
window-keeping arm reads 1.752 there), and 4,793.2 µs at eight lanes against its
own 3,196.3 µs at four — i.e. **eight lanes are slower than four** on a host with
six performance and two efficiency cores. At W=16, oversubscribed, the same
kernel reads 2.222 / 2.647 and 2.802 across the two idle-window tables. Those
blocks carry no candidate-record verdict (oversubscribed), but they are the
honest shape of the remaining weakness: **not the fork-join edge at modest W, but
lane counts at or above the CPU count on an asymmetric machine.**

I/O numbers, from `research/investigations/io-model/RESULTS.md`, batch 0108,
Linux 6.18, four cores, 64-byte round trips. **The Whitefoot line here was
produced by the staged-loop runtime, which is the retired design; it is recorded
for the references' sake and as evidence about that runtime, and it is *not* the
expectation for the batch model of §0.1.**

| line | 1 conn | 64 conns | 1024 conns | 64 KiB @ 64 conns |
|---|---|---:|---:|---:|
| `uring` reference | 28.1k rt/s | **314.9k** | **349.1k** | 60.5k |
| `epoll` reference | 29.6k | 307.5k | 323.6k | 75.7k |
| Whitefoot, *retired staged runtime* | 35.5k (1.26x) | 232.3k (0.74) | 201.5k (0.58) | 52.2k (0.86) |
| same, p50 | 6 µs | 245 µs | 4,870 µs | 1,064 µs |

Mechanism costs:

- completion core round trip (claim, publish, drain, consume): **34.9–35.6 ns**;
  inline terminal 31.7–32.9 ns. (`io-model/RESULTS.md`.)
- **park-and-wake on a Linux host: 10–16 µs.** Two independent measurements,
  both on four-CPU Linux development hosts of the same class:
  `compiler/src/backend/sched/core.c`'s evidence comment, re-measured at that
  revision on 2026-09-12 for PR #43, reads **10.3 µs** — the median of seven
  runs of 2,000 alternating parks through the scheduler's own
  `wf__par_signal` / `wf_prim_wait_sleep` — and notes that the previous reading
  on another machine of the same class was 16.3 µs.
  `research/experiments/park-on-miss-measurements/README.md` reads
  **16,217.9 ns (16.2 µs)** for a two-thread condition-variable park-and-wake on
  that host, beside **872 ns** for the same ping-pong on **Darwin arm64**, and
  4.40 µs best / 6.23 µs median for the retired design's park-and-publish.
  So the 872–934 ns figure quoted in earlier drafts is the *Darwin arm64*
  switch-cost bundle's number and does not describe a Linux host.
  **Every estimate below uses 10–16 µs on Linux and ~0.9 µs on Darwin arm64, and
  names the host.**
- the same evidence comment gives the other half of the model: a spin-round
  floor of **11.6 ns**, so the scheduler's 1,024-round spin bound is about
  **11.9 µs** — deliberately sized to the park-and-wake it replaces — inside a
  **1,000 µs idle window**. The operational consequence used repeatedly below:
  **a fork-join round costs tens of nanoseconds when the lanes are still hot
  inside that window, and 10–16 µs when a lane has parked.** Back-to-back rounds
  in a tight loop are cheap; rounds separated by more than the idle window pay
  the full wake each time.
- user-level stack switch, hand-written, arm64: **10.4 ns** (9.8–10.4) —
  Darwin arm64, `park-on-miss-switch-cost`. Retained only as the measurement of
  the retired stackful design; the batch model parks no stacks.
- a helper-thread + park path costs 4.2–4.3 µs for a read, ~8.25 µs for a
  write, against 366–439 ns for a direct cached `pread`. A thread handoff needs
  real waiting to repay itself.
- historical grain hazard, `proof-derived-parallelism/RESULTS.md` §8: asking
  for `--par` on `fib(38)` cost 2.6x before the sequential clone landed and
  1.00x after; switching lanes on for `wfgrep` cost **1.40x** because the
  program offered a lane per byte comparison. Profitability is not permission,
  and the shipped profitability policy is the scalar-leaf limit, the recursive
  grain default, the lane budget and the idle window.

### 0.4 Which verdicts moved when the park-and-wake figure was corrected

Earlier drafts used 1–2 µs for a pool-level park-and-wake, taken from the Darwin
arm64 ping-pong. On Linux the measured figure is **10–16 µs**, five to sixteen
times larger. Re-deriving every estimate that touched it:

- **Moved worse — architecture 9 (lock-free ring).** The minimum profitable batch
  is the one whose work exceeds a round. At 10–16 µs per *cold* round and ~100 ns
  per event, that is ~100–160 events to break even rather than ~20, and the
  achievable handoff-latency floor rises from "2–20 µs" to **10–30 µs**. Against
  a Disruptor's 40–100 ns the gap widens from 50–200x to **100–750x**. The
  verdict (not expressible) does not change; the number does.
- **Moved worse — architecture 20 (responsive loop).** Buying p99 back by
  shrinking the chunk now costs 10–16 µs per round rather than ~2 µs: a 100 µs
  chunk carries **10–16% overhead**, not 2%. The verdict stays "bounded loss"
  but the recovery is no longer nearly free, and the document now says so.
- **Moved worse — architecture 2 at high W**, already stated in §2(d): the W=8
  and W=16 regressions on the M1 Pro are the steal path falling out of the spin
  bound onto the park path.
- **Did not move — architectures 3, 4, 6, 14′ (back-to-back rounds).** These run
  rounds in a tight loop, so lanes stay hot inside the 1,000 µs idle window and a
  round costs spin rounds at 11.6 ns, not a wake. The original "low µs"
  estimates were, if anything, pessimistic for these. Each now says "hot round"
  explicitly.
- **Did not move — architecture 16 (background periodic phase).** Nothing there
  waits on a lane wake; the cost is a clock read.
- **Architectures 17–20 are re-derived from scratch under §0.1's batch model**
  (points 3 below), so their old latency floors are replaced rather than
  adjusted. The one figure that carries over is the batch-wait blocking call
  itself, which is a syscall, not a pool wake.

---

## 1. Data-parallel loop / map-reduce

OpenMP `parallel for`, Rayon `par_iter`, TBB `parallel_for` + `parallel_reduce`.

### (a) Native shape

```c
#pragma omp parallel for schedule(dynamic, 64)
for (size_t i = 0; i < n; ++i)
    out[i] = kernel(in[i]);

double total = 0.0;
#pragma omp parallel for reduction(+:total)
for (size_t i = 0; i < n; ++i)
    total += weight(in[i]);
```

```rust
out.par_iter_mut().zip(&inp).for_each(|(o, x)| *o = kernel(*x));
let total: u64 = inp.par_iter().map(weight).sum();
```

### (b) Whitefoot shape

Exactly the shape of `research/experiments/compute-bench/programs/fir.wf` and
`mandelbrot.wf`. No restructuring at all.

```whitefoot
fn render(in: &buffer<f64>, first: own u64, end: own u64)
    -> result: own buffer<u64> reads(in) contract {
  define n = len_of(deref(in));
  requires first <= end;
  requires end <= n;
  requires end <= 1048576_u64;
} {
  let count = end - first;
  let out = buffer_new(count, 0_u64);
  for (target in 0_u64..count) {
    let point = first + target;
    set out[target] = kernel(x: deref(in)[point]);     // a=1, b=0: permitted
  }
  return move out;
}

fn weigh(in: &buffer<u64>, end: own u64) -> result: own u64 reads(in) contract {
  requires end <= len_of(deref(in));
} {
  let total = 0_u64;
  for (i in 0_u64..end) {
    set total = total +wrap weight(x: deref(in)[i]);   // one accumulator, +wrap
  }
  return total;
}
```

### (c) What changes structurally

For the map: nothing. The output buffer must be the one the loop owns and the
write must be `out[i]`, which is how anyone would write it anyway. The proof
obligations are the ordinary ones the program already needs — `first <= end`,
`end <= len_of(…)` — carried in the `contract` block and discharged once at the
call boundary, not per element.

For the reduction, three real restrictions:

- **One accumulator.** Mean-and-variance, min-and-argmin, sum-and-count in one
  pass: denied, `LoopDenial::ManyAccumulators`. The writer runs two loops and
  reads the input twice, or packs two values into one integer and uses `+wrap`
  with a field layout that cannot carry between fields — a real technique, and
  an ugly one.
- **Fixed operation set, integers and bits only.** `fadd.strict` is not
  associative, so **no floating-point reduction is admitted at all**. A
  float sum over 500k elements runs sequentially. For `fir` this does not bite
  because the fold is per-output and iteration-private; for a genuine
  `parallel_reduce` over doubles it is total.
- **No exits.** A body containing `break`, `return`, `give` out of the loop, or
  a `?`-style `propagate_let_rhs` denies. Any reduction that can fail early —
  a checksum that stops on a bad record, a parse that propagates an error — is
  sequential unless rewritten to encode failure in the accumulator's value
  domain (`imin` over positions, `ior` over flags).

Granularity is *not* a writer decision here, which is the good news: the
compiler emitted 16 chunks for 98,304 mandelbrot points and 64 for 524,288 FIR
outputs, and the runtime steals across them (6 and 11 steals at W=4).

### (d) Performance expectation

Throughput: measured, not estimated. 0.906–0.938 of the best reference on FIR
(Whitefoot fastest), 1.034 on mandelbrot, 1.138 on records at W=4 on four
cores; 0.866–0.984 on all three at W=2 on three Apple cores. Call it **parity
to 15% off, and sometimes ahead**.

Why records is the worst of the three: 64 chunks over 131,072 records is
~2,048 records per chunk, and the per-record cost varies with record length, so
the tail chunk dominates; `rayon-join`'s adaptive bisection keeps splitting.
The ratio moved 1.113 → 1.138 → 1.176 as W went 2 → 4 → 8, which is the
signature of a fixed chunk count meeting a rising worker count.

Latency: irrelevant for a batch map. Memory: identical — one output buffer, no
per-lane copies, no task objects. Whitefoot is strictly cheaper than Rayon here
on allocation.

**Strictly worse case:** a float reduction, where Whitefoot is sequential and
the native form is ~Wx faster. On four cores that is a 3–4x loss on that
phase. Second worst: a fine-grained map whose body is a handful of instructions
— the `wfgrep` 1.40x regression is exactly this shape, a lane offered per byte
comparison.

### (e) Verdict

**Direct** for the map and for integer/bit reductions. **Bounded loss** for
reductions: no float folds, one accumulator, no early exit — each a sequential
phase where the native form is W-way.

---

## 2. Recursive divide-and-conquer fork-join

Cilk `spawn`/`sync`, Rayon `join`, TBB `parallel_invoke`.

### (a) Native shape

```c
double adaptive(double a, double b, double whole, double tol, int depth) {
    double m = 0.5 * (a + b);
    double left  = simpson(a, m), right = simpson(m, b);
    if (depth == 0 || fabs(left + right - whole) <= 15.0 * tol)
        return left + right + (left + right - whole) / 15.0;
    double l, r;
    cilk_scope {
        l = cilk_spawn adaptive(a, m, left,  tol * 0.5, depth - 1);
        r =            adaptive(m, b, right, tol * 0.5, depth - 1);
    }
    return l + r;
}
```

### (b) Whitefoot shape

Verbatim from `compute-bench/programs/quadrature.wf`. There is no `spawn`; the
two sibling calls are two `let` statements in one block and PAR-1 judges them.

```whitefoot
fn adaptive(a: own f64, b: own f64, center: own f64, width: own f64,
            fa: own f64, fm: own f64, fb: own f64, whole: own f64,
            tolerance: own f64, depth: own u64) -> result: own f64 pure contract {
  requires depth <= 24_u64;                       // the structural ceiling
} {
  let middle = fmul.strict(fadd.strict(a, b), 0.5_f64);
  /* … estimator … */
  if depth == 0_u64 { /* … */ }
  let next = depth - 1_u64;
  let half_tolerance = fmul.strict(tolerance, 0.5_f64);
  let l = adaptive(a: a, b: middle, /* … */ tolerance: half_tolerance, depth: next);
  let r = adaptive(a: middle, b: b, /* … */ tolerance: half_tolerance, depth: next);
  return fadd.strict(l, r);
}
```

Note `requires depth <= 24_u64`: recursion needs a **fixed structural source
ceiling**, and every caller discharges it. That is the analogue of "prove this
terminates", paid in the signature.

Unbalanced recursion needs no extra source at all. The runtime steals:
`quadrature` recorded 679 / 2,125 / 2,259 steals at W=2/4/8, on a kernel whose
subdivision depth varies by an order of magnitude across the interval.

### (c) What changes structurally

Almost nothing in shape, and two things in substance:

- **The depth ceiling is a contract, and it propagates.** A caller that cannot
  bound its own depth cannot call. For quadtrees, tries and parsers this is
  usually fine (depth is bounded by key width or nesting limit); for a
  divide-and-conquer over a runtime-sized array, the writer must write the
  `log2(n) <= K` argument once, in a `requires`, and thread the bound down.
- **Grain is not the writer's.** Cilk lets you write `if (n < 4096) return
  serial(...)`. In Whitefoot you can write that too — it is ordinary source —
  but you cannot tell the compiler which fork to actualize; PR #33's
  scalar-leaf policy (limit 16) is a compiler-fixed heuristic. The measured
  consequence is in (d).

Proof obligations: the depth bound, plus the ordinary footprint disjointness of
the two sibling calls. In `par_layout.wf`'s ledger the sibling pair is
`PAR permitted … pair(layout, layout)  eligible` and the *only* denials are
condition 1, "the operands of s2 read what s1 defines" — i.e. hand the two
children independent data and the window opens; thread a value from left to
right and it closes. That is the whole discipline.

### (d) Performance expectation

**At W≤4 this is now parity, on both hosts.** The 2026-09-12 tables read
`quadrature` at **0.969 and 0.980** against `rayon-join` on hosted ubuntu at
W=2 and W=4 in run 34667725821 — Whitefoot is the *fastest* form in both blocks
— **1.005 and 1.029** in run 34668156035, **1.026 and 1.024** in run
34668796717, and **1.000–1.073** across the two Apple M1 Pro idle-window tables.
The 1.581 / 1.565 figures that earlier drafts of this catalog carried were taken
before the recursive-grain default, the idle window and the split work unit
landed, and are superseded.

The opt-in tax on the sequential path is also gone: on the M1 Pro, `quadrature`
at W=1 reads 8,360.2 µs against `serial`'s 8,247.3 µs — **1.4%**, inside the
run-to-run spread. The earlier "1.69x on macOS" reading is superseded by the same
change.

What the numbers now say about the mechanism: at W=4 the M1 Pro table records
1,012 steals for a ~3.2 ms call, one steal per ~3 µs. With a Linux park-and-wake
at **10–16 µs** (`sched/core.c`, 10.3 µs; `park-on-miss-measurements`, 16.2 µs)
a steal at that rate could not possibly be paying a full wake — it is being
served inside the 1,024-round spin bound (~11.9 µs of spinning at an 11.6 ns
round) and inside the 1,000 µs idle window. **That is exactly what the idle
window bought, and it is why the ratio moved from 1.58 to ~1.0**: the recursive
kernel's steals stopped hitting the park path. It also predicts where the
remaining loss lives, and the tables agree — see below.

Throughput: **parity at W≤4** (0.97–1.07 across six hosted/local blocks).
Latency: the same ratio, since this is a latency-shaped kernel (one call, one
answer). Memory: lower than a task-based runtime — no task objects.

**Where it still loses: lanes at or above the CPU count on an asymmetric host.**
On the M1 Pro (6 performance + 2 efficiency cores), with the idle window
withheld, `quadrature` reads **1.352 against `parlay` at W=8**, and 4,793.2 µs
at eight lanes against its own 3,196.3 µs at four — eight lanes are *slower than
four*. At W=16 the two idle-window tables read 2.222, 2.647 and 2.802. Those
blocks carry no candidate-record verdict, but the mechanism is legible: once
lanes outnumber usable fast cores, a lane that loses its CPU parks, and a park
on this class of host costs 10–16 µs against a ~3 µs inter-steal interval —
the steal path stops being served by the spin bound and starts paying wakes.

**Strictly worse case:** oversubscription on an asymmetric host (above:
1.35 at W=8, 2.2–2.8 at W=16 on an 8-CPU M1 Pro), and shallow, cheap recursion.
`fib(38)` measured 12.6x slower at four workers before work-stealing landed and
1.33x after, and the `--par` opt-in tax on it was 2.6x before the sequential
clone. The grain hazard is structural — a fork is offered at every node
regardless of subtree size — and the writer has no grain knob; the recursive-grain
default is what closed it for `quadrature`, and it is a compiler-fixed policy.

### (e) Verdict

**Restructure, no loss** — structurally (the depth ceiling is the only addition)
*and now in performance at W≤4*: 0.97–1.07 of the best native work-stealing
runtime across six blocks on two hosts, fastest in two of them. **Bounded loss**
only at lane counts at or above the host's CPU count on an asymmetric machine
(1.35 at W=8, 2.2–2.8 at W=16), where the steal path starts paying the 10–16 µs
Linux-class park-and-wake instead of being served inside the spin bound.

---

## 3. Pipeline with stages and queues, with a stateful stage

TBB `parallel_pipeline`, GStreamer, Rust threads plus crossbeam channels.

### (a) Native shape

```rust
let (tx1, rx1) = bounded(64);
let (tx2, rx2) = bounded(64);
scope(|s| {
    s.spawn(|_| for blk in source() { tx1.send(parse(blk)).unwrap(); });
    s.spawn(|_| {                                   // stateful stage
        let mut dict = Dictionary::new();
        for rec in rx1 { tx2.send(dict.encode(rec)).unwrap(); }
    });
    s.spawn(|_| for enc in rx2 { sink.write_all(&enc).unwrap(); });
});
```

Three threads, three stages, two elastic 64-slot queues absorbing per-item
jitter, stage 2 carrying a dictionary across every item.

### (b) Whitefoot shape

Not expressible as three concurrent stages over one item stream. Expressible as
a **software-pipelined double buffer**: two batches in flight, the stages
applied to *different* batches in one PAR-1 window.

```whitefoot
// [new] mut_slice_of(&uniq buf, start, end) — accepted range loan.
// Restructuring note: queues become two batch buffers; stage state becomes an
// ordinary binding carried across the outer loop; per-item elasticity is gone.

fn run_pipeline(src: &uniq Source, sink: &uniq OutputStream, dict: &uniq Dictionary,
                a: &uniq buffer<Rec>, b: &uniq buffer<Enc>, rounds: own u64)
  -> result: own u64
  reads(src, sink, dict, a, b), writes(src, sink, dict, a, b) {
  doc "Two batches in flight: parse batch k into a while encode+emit drains
       batch k-1 out of b. The dictionary is one place, so encode is the only
       statement that may touch it.";
  let done = 0_u64;
  for @rounds (k in 0_u64..rounds) {
    // window member 1 and member 2 write disjoint places and read disjoint
    // places, so [PAR-1] admits the pair; `dict` is reached only by member 2.
    let filled  = parse_batch(src: &uniq deref(src),  out: &uniq deref(a));
    let emitted = encode_and_emit(dict: &uniq deref(dict), in: &deref(b),
                                  sink: &uniq deref(sink));
    set done = done +wrap emitted;
    swap_batches(a: &uniq deref(a), b: &uniq deref(b));   // [new] or a parity branch
  }
  return done;
}
```

The three-stage case needs three buffers and a three-deep rotation, and the
window is then `parse(→a)`, `encode(b→c)`, `emit(c→sink)` — three members, all
footprint-disjoint, one round per item batch.

`swap_batches` is `[new]` only in the sense that a `replace`-based swap of two
owned buffers is awkward today; a parity branch with two spelled-out arms
avoids it entirely at the cost of duplicating the window.

### (c) What changes structurally

- **The queue becomes a batch size, chosen by the writer, in the source.**
  There is no elastic buffer. A 64-slot channel absorbs a stage that
  occasionally takes 10x its median; a fork-join round does not — the round
  costs `max(stage_i)` every time, not `mean(stage_i)` amortized over 64 slots.
- **The stateful stage is now the serialization point, explicitly.** `dict` is
  one place; only one window member may reach it. That is a *feature* for
  reasoning (the data race is gone by construction) and a cost for throughput:
  the dictionary stage can never be replicated, so pipeline depth is capped by
  the number of stateless stages plus one.
- **The writer chooses the number of stages and the number of buffers in the
  source.** PAR-1's arity is written out. A 6-stage GStreamer graph is 6
  statements and 6 buffers, written by hand.
- **Per-item latency becomes per-batch latency.** An item entering at round k
  leaves at round k+S for an S-stage pipeline, and a round is a batch.

Proof obligations: that the batch buffers are distinct bindings (trivial), that
each stage's `reads`/`writes` row names only its own places (the real work —
one over-broad effect path collapses the whole window), and the range-loan
disjointness if batches are windows into one arena.

### (d) Performance expectation

Throughput: within ~10–20% of the native pipeline when stage costs are stable
and batches are large enough to amortize the round barrier. Estimate, reasoning:
the added work is one fork-join per round instead of per item; from the compute
scoreboard a **hot** fork-join round — one whose lanes are still inside the
1,000 µs idle window, which back-to-back pipeline rounds always are — costs spin
rounds at 11.6 ns rather than a 10–16 µs Linux park-and-wake, so a batch of 1,000
items at 1 µs each pays well under 0.1% barrier overhead. A pipeline whose rounds
are *further apart than the idle window* pays 10–16 µs per round instead, which
is the case to watch. The added serialization is the `max` vs `mean`
effect above, which is the real cost and depends entirely on stage-time
variance. With a stage whose p99 is 10x its p50 and a 3-stage pipeline, a
64-slot queue hides essentially all of it and a round barrier hides none:
expect **1.3–2x worse throughput** in that regime.

Latency: worse by construction — per-item latency becomes (batch size × per-item
cost × stages) instead of (per-item cost × stages). For a 1,000-item batch that
is a 1,000x per-item latency increase. For a streaming media pipeline with a
frame deadline this alone can disqualify the shape unless the batch is one frame.

Memory: better. No channel allocation, no per-item boxing, S+1 fixed buffers
instead of S queues of 64 items each.

**Strictly worse case:** a pipeline with one high-variance stage and a per-item
latency requirement. Throughput loss up to the variance ratio (2x is easy to
construct); latency loss equal to the batch size.

### (e) Verdict

**Restructure, bounded loss.** The loss is per-item latency (becomes per-batch)
and jitter absorption (becomes zero). Throughput is close when stages are
regular.

---

## 4. Producer–consumer with a bounded queue and backpressure

### (a) Native shape

```c
// producer thread
while (read_chunk(&c)) {
    sem_wait(&empty); pthread_mutex_lock(&m);
    ring[head++ & MASK] = c;
    pthread_mutex_unlock(&m); sem_post(&full);
}
// consumer thread
for (;;) {
    sem_wait(&full); pthread_mutex_lock(&m);
    chunk c = ring[tail++ & MASK];
    pthread_mutex_unlock(&m); sem_post(&empty);
    consume(c);
}
```

Backpressure is `sem_wait(&empty)`: the producer blocks when the ring is full,
and the *rate* is adapted continuously and automatically.

### (b) Whitefoot shape

```whitefoot
// Restructuring note: the ring becomes two fixed batch buffers; backpressure
// becomes the batch size, a constant in the source; the rate adapts once per
// round instead of once per item.
fn pump(src: &uniq Source, sink: &uniq Sink,
        a: &uniq buffer<u8>, b: &uniq buffer<u8>, rounds: own u64)
  -> result: own u64
  reads(src, sink, a, b), writes(src, sink, a, b) contract {
  requires len_of(deref(a)) == len_of(deref(b));
} {
  let moved = 0_u64;
  let carried = 0_u64;
  for @pump (k in 0_u64..rounds, invariant progress: carried <= len_of(deref(a))) {
    // one window: fill a from the source while b is drained into the sink.
    let produced = fill(src: &uniq deref(src), out: &uniq deref(a));
    let taken    = drain(in: &deref(b), count: carried, sink: &uniq deref(sink));
    set moved = moved +wrap taken;
    set carried = produced;
    swap(a: &uniq deref(a), b: &uniq deref(b));    // [new], or a parity branch
  }
  return moved;
}
```

### (c) What changes structurally

- **Backpressure stops being a mechanism and becomes a constant.** There is no
  blocking-when-full; there is a buffer whose size the writer chose. If the
  consumer is slower, the producer idles at the round barrier — which is
  backpressure, but quantized to one round. If the consumer is *much* slower,
  the producer wastes a whole round's fill and then waits, where the native
  producer simply blocks on the first full slot.
- **No continuous rate adaptation.** A native bounded queue self-balances at
  the granularity of one item. The round barrier balances at the granularity of
  one batch. For a producer whose rate varies within a round, nothing adapts.
- **No decoupled shutdown.** Native: producer closes the channel, consumer
  drains and exits. Here the round count or a `break` in the producer's result
  ends both at once, and the writer must write the drain-the-last-batch
  epilogue explicitly after the loop.

Proof obligations: the two buffers are the same length (a `requires`), the
carried count is within the buffer (a header `invariant`), and the fill/drain
effect rows do not overlap.

### (d) Performance expectation

Throughput: comparable, ±10%, when the two sides are rate-matched and the batch
is ≥ a few hundred items. Estimate: the added work per round is one fork-join
(a hot round: spin rounds at 11.6 ns, not a 10–16 µs wake, because the pump loop
runs its rounds back to back inside the idle window) and one swap; the removed
work is a mutex/semaphore pair per item,
which at ~20–40 ns per uncontended pair is *saved*, not spent. For small items
Whitefoot can be **faster** here.

Latency: worse by one batch period, as in architecture 3.

Memory: two buffers instead of a ring — same order, often less.

**Strictly worse case:** bursty producer, tight consumer, small items, latency
target. A 64-slot ring absorbs the burst; a two-batch rotation does not, so the
consumer sees an idle round then a full one. Estimate 1.5–2x worse effective
throughput under a bursty arrival process; no measurement in this repository
covers it.

### (e) Verdict

**Restructure, bounded loss.** Cost: backpressure granularity becomes the batch
size, and per-item latency becomes per-batch latency. Throughput is a wash or
better.

---

## 5. Task DAG with dependencies, static and dynamic

TBB flow graph, Taskflow.

### (a) Native shape

```cpp
tf::Taskflow f;
auto A = f.emplace([&]{ load(); });
auto B = f.emplace([&]{ transform_left(); });
auto C = f.emplace([&]{ transform_right(); });
auto D = f.emplace([&]{ merge(); });
A.precede(B, C);  B.precede(D);  C.precede(D);
tf::Executor().run(f).wait();     // B and C overlap; D starts the instant both end
```

### (b) Whitefoot shape, static DAG

A static DAG *is* a PAR-1 window per level. The dependency edges are the data
dependencies the rule already reads.

```whitefoot
fn run_graph(src: &Input, l: &uniq buffer<f64>, r: &uniq buffer<f64>,
             out: &uniq buffer<f64>) -> result: own u64
  reads(src, l, r, out), writes(l, r, out) {
  let loaded = load(src: src);                       // A
  // B and C: disjoint written footprints (l vs r), neither reads what the other
  // defines, so [PAR-1] admits the pair.
  let lb = transform_left(in: src, out: &uniq deref(l));
  let rb = transform_right(in: src, out: &uniq deref(r));
  let merged = merge(a: &deref(l), b: &deref(r), out: &uniq deref(out));   // D
  return merged;
}
```

### (b′) Whitefoot shape, dynamic DAG

Dependencies discovered at run time force the level-synchronous form: the graph
becomes arrays, and each level is one PAR-2 loop.

```whitefoot
// Restructuring note: the executor's ready queue becomes a per-level ready
// array recomputed each level; "start when your last dependency finishes"
// becomes "start at the next level barrier".
fn run_dynamic(dep_count: &uniq buffer<u32>, edges: &buffer<u32>, edge_at: &buffer<u32>,
               ready: &uniq buffer<u32>, next: &uniq buffer<u32>,
               state: &uniq buffer<TaskState>, levels: own u64) -> result: own u64
  reads(dep_count, edges, edge_at, ready, next, state),
  writes(dep_count, ready, next, state) {
  let ready_count = seed_ready(dep: &deref(dep_count), out: &uniq deref(ready));
  let done = 0_u64;
  for @levels (level in 0_u64..levels, invariant fits: ready_count <= len_of(deref(ready))) {
    // level body: one element written per iteration, state[i] is affine a=1,b=0
    for (i in 0_u64..ready_count) {
      set deref(state)[i] = run_task(id: deref(ready)[i], /* … */);
    }
    // sequential: decrement successors' counts, compact the next frontier
    set ready_count = retire_level(state: &deref(state), count: ready_count,
                                   edges: edges, edge_at: edge_at,
                                   dep: &uniq deref(dep_count),
                                   out: &uniq deref(next));
    swap_frontiers(ready: &uniq deref(ready), next: &uniq deref(next));   // [new]
    set done = done +wrap ready_count;
  }
  return done;
}
```

Two things are worth naming in that body. `set deref(state)[i] = …` is the
affine element write and it is permitted; `retire_level` — which decrements an
arbitrary successor's counter and appends to a frontier — is a *scatter* and is
denied by PAR-2 under any arrangement, so it stays sequential.

### (c) What changes structurally

Static DAG: nothing. The writer already writes the topological order; PAR-1
finds the concurrency in it. Effect rows must be precise — one function whose
`writes` row names a whole struct instead of the field it touches closes every
window it appears in. This is the main new discipline: **effect-path precision
is now a performance property, not just a proof property.**

Dynamic DAG, three changes:

- **Barrier per level instead of per edge.** Taskflow starts D the instant B
  and C both finish. Here D starts at the next level barrier, so the makespan
  is `sum over levels of max(task in level)` rather than the true critical path.
  For a DAG with many levels of few tasks, or with one long task per level, the
  gap is large.
- **The frontier compaction is sequential** and is O(edges out of the level).
  For a DAG with high fan-out, this pass can rival the tasks themselves.
- **Task identity becomes an index into arrays**, and each task's result must
  fit a fixed `TaskState` row. Heterogeneous tasks with different result types
  need an enum and a `match`, which is fine but costs a branch and the union's
  size per row.

### (d) Performance expectation

Static: parity. The window is exactly the native fork-join and the measured
sibling-pair path reaches 2.98x on 4 cores in `par_layout.wf`'s eligible phase
(75% of ideal, the shortfall being the tree's own critical path and the lane
budget).

Dynamic: throughput within ~15% for wide, shallow DAGs — most of the work is in
the level loops, which are ordinary PAR-2 maps at the measured 1.03–1.14 ratio,
plus a sequential compaction pass. For deep, narrow, skewed DAGs the
level-barrier makespan can be **2–5x** the critical path. Reasoning, with an
estimate: for L levels where each level has one task of cost T and k tasks of
cost T/10, the barrier makespan is L·T and the critical path may be as low as
L·T/… — no, the critical path is also ≥ the longest chain, so the honest
statement is: the loss equals `sum_level max(level) / critical_path`, which is
1.0 for a balanced DAG and grows without bound for a DAG whose long tasks are
spread across different levels and could have run concurrently with short
chains. I have no measurement; I would want one before believing a number.

Memory: better — no task objects, no executor queues, four index arrays.

**Strictly worse case:** a DAG with a long task in level 3 and a chain of short
tasks in levels 3–20 that Taskflow would have run underneath it. Here the chain
waits for level 3 to end. Loss = the long task's duration, repeated.

### (e) Verdict

Static: **direct.** Dynamic: **restructure, bounded loss** — the loss is
level-barrier makespan versus true critical path, plus a sequential frontier
compaction per level.

---

## 6. Game-engine job system

A frame of many small jobs, jobs spawning jobs, fibers and work stealing, a
frame boundary.

### (a) Native shape

```cpp
void Frame() {
    Counter c;
    for (Entity& e : entities) Sched::Run(&c, [&]{ UpdateTransform(e); });
    Sched::WaitFor(&c);                       // fiber-switch, not a block
    Counter c2;
    Sched::Run(&c2, [&]{ Cull(); });
    Sched::Run(&c2, [&]{ Animate(); });       // Animate spawns per-skeleton jobs
    Sched::WaitFor(&c2);
    Render();                                  // frame boundary
}
```

### (b) Whitefoot shape

```whitefoot
// Restructuring note: a job counter becomes a phase; a job that spawns jobs
// becomes an append to the next phase's array plus one more phase; fibers are
// runtime, not source.
fn frame(world: &uniq World, xf: &uniq buffer<Transform>,
         anim_jobs: &uniq buffer<AnimJob>, n: own u64) -> result: own u64
  reads(world, xf, anim_jobs), writes(world, xf, anim_jobs) contract {
  requires n <= len_of(deref(xf));
} {
  // phase 1: per-entity transform. One element written per iteration.
  for (e in 0_u64..n) {
    set deref(xf)[e] = update_transform(entity: read_entity(world: &deref(world), id: e));
  }
  // phase 2: two independent whole-system jobs — one [PAR-1] window.
  let culled  = cull(xf: &deref(xf), out: &uniq deref(world).visible);
  let spawned = plan_anim(world: &deref(world), out: &uniq deref(anim_jobs));
  // phase 3: the jobs phase 2 produced. A new phase, not a nested spawn.
  for (j in 0_u64..spawned) {
    set deref(anim_jobs)[j] = run_anim(job: deref(anim_jobs)[j], world: &deref(world));
  }
  return apply_anim(jobs: &deref(anim_jobs), count: spawned, world: &uniq deref(world));
}
```

### (c) What changes structurally

- **`WaitFor` becomes a statement boundary.** Every counter wait is a fork-join,
  which it already effectively was. No loss.
- **A job that spawns jobs becomes a phase that fills an array plus a following
  phase that runs it.** The native system runs the spawned job the instant it is
  created, on the same fiber, often before the parent's siblings finish. Here it
  waits for the whole producing phase to end. For a two-deep spawn tree that is
  one extra barrier; for a job graph that spawns at arbitrary depth it is one
  barrier per depth level — architecture 5's dynamic case.
- **`run_anim` writing back into the job row is the only shape PAR-2 permits.**
  A job that writes into a *shared* scene structure at an arbitrary index is a
  scatter and denies. So the per-job output must be a per-job row, and a
  sequential `apply` pass merges. That is one extra pass over the jobs per
  phase, and it is where the frame's serial fraction accumulates.
- **Fibers exist but are not in the language.** The runtime already switches
  user stacks at 10.4 ns and the I/O bench runs with `WF_STACKS=1100`. The
  writer neither sees nor controls them.

### (d) Performance expectation

Per phase: the measured map ratio, 1.03–1.14 of a tuned native scheduler.

Per frame: worse by the extra barriers and the sequential merge passes. Estimate
for a typical frame with 6 phases at 4 cores: if the native system's total
barrier cost is ~6 × (a fiber-switch counter wait, ~100 ns) and Whitefoot's is
6 × (a pool fork-join — a *hot* round at 11.6 ns spin rounds, since a frame's
phases follow one another well inside the 1,000 µs idle window, not a 10–16 µs
park) plus 3 sequential merge passes over the job
arrays, then on a 16.6 ms frame the barrier difference is ~10 µs (0.06%,
negligible) and the merge passes are the real cost: a merge over 10,000 job rows
at ~5 ns/row is 50 µs per pass, 150 µs per frame, **~1% of a 60 Hz frame**. That
is acceptable. The danger is not the barrier; it is the merge.

**Strictly worse case:** thousands of *tiny* jobs. The grain hazard is measured:
`wfgrep` lost 1.40x when a lane was offered per byte comparison, and pre-stealing
`fib(38)` lost 12.6x. A game frame with 50,000 jobs of 200 ns each is exactly
that shape, and Whitefoot has no writer-side grain control — the scalar-leaf
limit of 16 is a compiler heuristic. Expect **1.2–1.5x worse** on a
fine-grained frame, and no knob.

Latency: the frame boundary is natural and unchanged. Memory: much better — no
job objects, no per-job closures, arrays of rows.

### (e) Verdict

**Restructure, bounded loss.** Cost: one extra phase per spawn depth, one
sequential merge pass per phase whose jobs write shared state, and no
writer-side grain control on fine-grained frames.

---

## 7. Shared mutable structure with fine-grained locking or lock-free updates

Concurrent hash map — Java `ConcurrentHashMap`, `DashMap` — parallel inserts and
lookups.

### (a) Native shape

```rust
let map: DashMap<u64, u64> = DashMap::with_capacity(1 << 20);
keys.par_iter().for_each(|&k| {
    *map.entry(k).or_insert(0) += 1;            // per-shard lock, taken per op
});
let hits: usize = probes.par_iter().filter(|&&k| map.contains_key(&k)).count();
// inserts and lookups may be concurrent
```

### (b) Whitefoot shape

**Concurrent insert with concurrent lookup: not expressible.** The reason is
exact: both a lookup and an insert reach the same root, the insert writes it,
and PAR-1/PAR-2 deny any overlap where one statement's written footprint meets
the other's read footprint. There is no sharing classification that could say
"shared with internal synchronization" — [CAP-1] states the kernel defines no
such category, and `own`/`&`/`&uniq` are the complete vocabulary.

**Parallel build, then frozen lookups** is expressible, in two forms.

Form A, PAR-2 over shards, with the insert done by a called function per shard
— *denied*, and the denial is worth writing out because it is the recurring wall:

```whitefoot
for (s in 0_u64..shards) {
  let win = mut_slice_of(&uniq table, s *wrap span, s *wrap span +wrap span);  // [new]
  //  ^ forms an exclusive loan on `table`, which is rooted OUTSIDE the loop.
  //    [PAR-2]: "Every place a footprint of B holds an exclusive loan on is
  //    rooted in a binding B itself introduces" — DENIED. Runs sequentially.
  set filled = insert_range(keys: &keys, shard: s, out: &uniq win);
}
```

Form B, PAR-1 with the lane count written in the source — permitted:

```whitefoot
// Restructuring note: W is a source constant. Four lanes is four statements.
// [new] mut_slice_of range loans; disjointness discharged by linear arithmetic.
fn build_table(keys: &buffer<u64>, n: own u64, table: &uniq buffer<Entry>)
  -> result: own u64 reads(keys, table), writes(table) contract {
  define cap = len_of(deref(table));
  requires cap == 4_u64 *wrap span;
} {
  region {
    let w0 = mut_slice_of(&uniq deref(table), 0_u64,          span);
    let w1 = mut_slice_of(&uniq deref(table), span,           2_u64 *wrap span);
    let w2 = mut_slice_of(&uniq deref(table), 2_u64 *wrap span, 3_u64 *wrap span);
    let w3 = mut_slice_of(&uniq deref(table), 3_u64 *wrap span, 4_u64 *wrap span);
    // Four disjoint exclusive loans on one root coexist [OWN-5]; the four calls
    // form one [PAR-1] chain because every ordered pair is footprint-disjoint.
    let a = insert_shard(keys: keys, n: n, lane: 0_u64, out: &uniq w0);
    let b = insert_shard(keys: keys, n: n, lane: 1_u64, out: &uniq w1);
    let c = insert_shard(keys: keys, n: n, lane: 2_u64, out: &uniq w2);
    let d = insert_shard(keys: keys, n: n, lane: 3_u64, out: &uniq w3);
    return a +wrap b +wrap c +wrap d;
  }
}
```

Each `insert_shard` scans the whole key array and inserts only the keys whose
hash lands in its own shard. Lookups against the finished table are shared
reads and parallelize under PAR-2 with no ceremony at all.

### (c) What changes structurally

- **W is a source constant.** Four lanes is four `let` statements and four range
  loans. Eight lanes is a different program. This is the sharpest single cost in
  the catalog: the program's parallel width is a compile-time literal the writer
  types, on a machine whose core count he does not know.

  **`[new]` — the affine-range extension removes this.** Range loans
  (`mut_slice_of(&uniq buf, start, end)`, disjointness a linear-arithmetic
  obligation) are accepted. If **[PAR-2] also admitted an iteration-exclusive
  loan on an affine range `[c*i + b, c*(i+1) + b)`** — the same constants and the
  same arithmetic it already uses for the single-element case `a*i + b`, proved
  disjoint across iterations for the same reason — then the per-shard window
  comes from the loop index:

  ```whitefoot
  for (s in 0_u64..shards) {                       // [new] affine range loan
    let win = mut_slice_of(&uniq table, s *wrap span, s *wrap span +wrap span);
    set filled[s] = insert_range(keys: &keys, shard: s, out: &uniq win);
  }
  ```

  `shards` is then an ordinary runtime value, the compiler chunks the range as it
  does for every other [PAR-2] loop, and **W stops being a source constant.**
  *What changes in the verdict:* the build-then-freeze half goes from
  "restructure, bounded loss (W in source)" to **restructure, no loss** — the
  remaining costs (no cross-shard probing, insert latency = batch period) are
  properties of the algorithm, not of the rule. *What does not change:*
  concurrent insert with concurrent lookup stays **not expressible**; that is
  [CAP-1], not the affine rule.
- **W passes over the key array instead of one.** Each shard's lane must find
  its own keys. Either every lane scans all n keys (W× the read traffic, but
  perfectly parallel and cache-friendly per lane) or a sequential partition pass
  precedes the build (one extra pass plus W output arrays).
- **Open addressing with probing cannot cross a shard boundary**, so the table
  must be per-shard chained or per-shard open-addressed with its own overflow —
  a real change to the data structure, not just to the loop.
- **Insert and lookup become phases.** A workload that genuinely interleaves
  them — a cache, a session table, a symbol table under concurrent compilation —
  must be rewritten as: batch the lookups, batch the misses, insert the batch,
  repeat. That is a different program with different semantics (a lookup can
  return "not yet inserted, but pending in this batch").

Proof obligations: the range-loan disjointness (linear arithmetic over four
constants and one `span` — easy); that each shard's writes stay inside its
window (the `insert_shard` contract's `requires`/`ensures`); and, for the
batched-interleave form, whatever the program needs to be correct with a
one-batch-stale view.

### (d) Performance expectation

Build throughput: with W lanes each scanning all keys, the work is W× the reads
and 1× the writes, so on 4 cores the build is roughly `max(W × scan, insert) / W`
≈ the sequential scan cost. That is **about 1.0x sequential** — i.e. the
parallelism buys nothing for a cheap hash. With a sequential partition pass
first, the build is one sequential pass plus a parallel insert phase: expect
**2–3x on 4 cores** against DashMap's ~3.2x (Rayon's measured self-scaling
shape), i.e. **0.6–0.9 of native**.

Lookup throughput on a frozen table: parity or better. No atomic, no shard lock,
no `Arc` refcount traffic. Estimate: DashMap's `contains_key` pays one shard
`RwLock` read acquisition (~15–25 ns uncontended) per probe; Whitefoot pays
none. On a lookup-dominated workload Whitefoot should be **1.2–1.5x faster**.

Latency: an insert's latency becomes the batch period, not the operation. For a
session table with a 10 µs insert budget and a 1 ms batch, that is a 100x
regression on the insert path.

Memory: better — no per-shard lock, no entry boxing; worse if the sequential
partition pass needs W staging arrays.

**Strictly worse case:** a genuinely concurrent read-write map with a per-op
latency target — a router's flow table, a JIT's inline-cache map. Whitefoot
cannot express it, and the batched substitute changes observable semantics.

### (e) Verdict

**Not expressible** for concurrent insert with concurrent lookup.
**Restructure, bounded loss** for build-then-freeze: W is a source constant,
the structure must be shardable without cross-shard probing, and insert latency
becomes the batch period. Frozen lookups are **direct** and faster than native.
**`[new]`: with the affine-range extension of (c) the build becomes
restructure, no loss** — W comes from the loop index and the remaining costs are
the algorithm's.

---

## 8. Read-mostly shared data: RwLock, RCU, epoch-based reclamation

### (a) Native shape

```c
// readers, hot path
rcu_read_lock();
const config_t *c = rcu_dereference(g_config);
use(c->table[i]);
rcu_read_unlock();

// writer, rare
config_t *new_c = build_new();
config_t *old = rcu_assign_pointer(g_config, new_c);
synchronize_rcu();                 // wait for all readers to leave
free(old);
```

### (b) Whitefoot shape

```whitefoot
// Restructuring note: the grace period becomes the phase boundary; the epoch
// counter, the hazard pointers and the deferred-free list all disappear.
fn serve_epoch(cfg: &Config, reqs: &buffer<Req>, out: &uniq buffer<Resp>, n: own u64)
  -> result: own u64 reads(cfg, reqs, out), writes(out) contract {
  requires n <= len_of(deref(reqs));
  requires n <= len_of(deref(out));
} {
  for (i in 0_u64..n) {
    set deref(out)[i] = answer(cfg: cfg, req: deref(reqs)[i]);   // shared loan on cfg
  }
  return n;
}

fn main_loop(cfg: own Config, reqs: &uniq buffer<Req>, out: &uniq buffer<Resp>,
             epochs: own u64) -> result: own u64 /* … */ {
  let live = move cfg;
  for @epochs (e in 0_u64..epochs) {
    let n = gather(reqs: &uniq deref(reqs));
    region {
      let answered = serve_epoch(cfg: &live, reqs: &deref(reqs),
                                 out: &uniq deref(out), n: n);   // the read phase
    }
    // the shared loan ends with the region; only now may the writer run.
    if reload_due(epoch: e) {
      set live = rebuild(old: move live);        // exclusive: no reader exists
    }
  }
  return 0_u64;
}
```

### (c) What changes structurally

Very little, and most of what changes is deleted rather than added:

- **The grace period is the region boundary.** `synchronize_rcu()` is exactly
  "wait until no reader holds the old pointer", and the lexical region is a
  static proof of that. No epoch counter, no hazard pointer array, no
  quiescent-state reporting, no deferred-free list.
- **Reclamation is immediate and exact.** `move live` consumes the old
  configuration at the moment no loan reaches it. RCU's memory overshoot — the
  classic "a reader that stalls holds a generation alive" — cannot occur.
- **The writer must fit between phases.** The reload cannot happen *during* a
  read phase. If the read phase is 10 ms, the configuration is up to 10 ms
  stale, and there is no way to shorten that except to shorten the phase. RCU
  has the same property in principle (readers see the old version until they
  re-dereference) but the writer's *publish* is immediate.

Proof obligations: none beyond the ordinary loan rules. This is the one
architecture where Whitefoot's model is strictly simpler than the native one.

### (d) Performance expectation

Reader throughput: **better**. An RCU read-side critical section on a
preemptible kernel costs a per-CPU counter increment and a compiler barrier
(~2–5 ns); an `RwLock` read costs an atomic RMW (~15–25 ns uncontended, far
worse under contention). Whitefoot's reader pays **zero** — `&cfg` is a
compile-time fact, and the loop is the measured PAR-2 map at 1.03–1.14 of the
best reference. On an `RwLock`-based design with 4 cores hammering the read
lock, Whitefoot should be **1.5–3x faster** on the read path; against
well-tuned RCU, parity.

Writer latency: the writer waits for the current read phase to drain, same as
`synchronize_rcu()`, and the wait is deterministic rather than
"until every CPU reports quiescent".

Memory: strictly better. One live version plus one under construction, never
more; no epoch machinery, no deferred list.

**Strictly worse case:** a writer that must publish *now* while a long read
phase is running — a configuration change with a 1 ms SLA against a 50 ms read
phase. Native RCU publishes immediately and readers pick it up at their next
dereference; Whitefoot cannot publish until the phase ends. Loss = the phase
length. Mitigation is the writer's: shorter phases, which costs barrier
overhead.

### (e) Verdict

**Restructure, no loss** — and on the read path a structural win. The only cost
is publish latency bounded by the read-phase length.

---

## 9. Lock-free SPSC/MPSC rings between threads (LMAX Disruptor)

### (a) Native shape

```c
// producer
uint64_t seq = claim(&ring);                  // CAS or plain store for SPSC
ring.slot[seq & MASK] = ev;
store_release(&ring.cursor, seq);             // publish

// consumer, spinning
for (;;) {
    uint64_t avail = load_acquire(&ring.cursor);
    while (next <= avail) handle(&ring.slot[next++ & MASK]);
    cpu_relax();
}
```

The point of this architecture is *per-item handoff latency*: 40–100 ns
producer-to-consumer, with no syscall, no park, and no batching.

### (b) Whitefoot shape

**Not expressible as a handoff.** The refusing axiom is the first one: source is
one sequential control flow, and every construct has a total sequential order.
A Disruptor's value comes from two control flows making progress independently
and meeting at a shared cell; the producer publishes and *keeps going* without
any join. Whitefoot's only overlap is fork-join — the loop or call group
completes before the next statement — so "publish and keep going" has no
expression. It is not a matter of finding the right proof; there is no
construct whose meaning is "start this and do not wait".

The throughput-equivalent restructure is architecture 4's double-buffered batch,
already priced there.

The one partially-analogous thing in the I/O model of §0.1 is `wait_batch`: a
set of pendings can be outstanding at once and the loop advances whichever
completes. But those are *target* operations, not a handoff between two agents
of the program, and nothing in that model lets one part of the program publish
to another and keep going without a join.

### (c) What changes structurally

There is no translation; there is a replacement. If a program's design rests on
a Disruptor, the Whitefoot version is a different design:

- events are accumulated into a batch array by the producing phase;
- the consuming phase runs over the batch;
- the two phases either alternate (one buffer) or overlap on two buffers in one
  PAR-1 window (architecture 3/4);
- sequence numbers, cursors, memory barriers and the `cpu_relax` spin all
  disappear, because there is nothing to synchronize.

### (d) Performance expectation

Throughput: comparable or better. A Disruptor's throughput comes from batching
on the consumer side anyway (`while (next <= avail)`), and Whitefoot's batch
loop is the same inner loop without the `load_acquire` per batch. No measurement
here, but the removed work is real and the added work is one fork-join per
batch.

Latency: **this is the loss, and it is total.** Native per-item handoff is
40–100 ns. Whitefoot's equivalent is one batch period. For a 1,024-event batch
at 100 ns/event that is ~100 µs, i.e. **1,000x worse p50 handoff latency**. If
the batch is shrunk to 8 events to chase latency, the fork-join cost becomes
larger than the work, and the design is worse than sequential.

The floor is worth stating precisely, and the corrected park-and-wake figure
makes it worse than earlier drafts said. A Disruptor's consumer *spins*, so it
never parks; a Whitefoot batch loop that is fed intermittently — which is the
regime a handoff exists for — lets its lanes fall out of the 1,000 µs idle
window and pays the **10–16 µs Linux park-and-wake** per round
(`sched/core.c` 10.3 µs; `park-on-miss-measurements` 16.2 µs; the ~0.9 µs
figure is Darwin arm64 and does not apply). At ~100 ns per event that is
**~100–160 events minimum to break even** and ~1,000–1,600 to amortize to 1%,
so the achievable handoff-latency floor is roughly **10–30 µs**, against the
Disruptor's 40–100 ns — a **100–750x** gap that no restructuring closes. A
continuously-fed pipeline keeps its lanes hot and pays 11.6 ns rounds instead,
but a continuously-fed pipeline is architecture 3, not a low-latency handoff.

Memory: better; no ring, no padding-to-cache-line, no sequence barriers.

### (e) Verdict

**Not expressible** as a low-latency handoff. The batch substitute matches
throughput and loses **100–750x** on handoff latency (floor ~10–30 µs on a Linux
host against 40–100 ns), because an intermittently-fed loop pays the measured
10–16 µs park-and-wake per round.

---

## 10. Actor model — many stateful actors, mailboxes, arbitrary message graph

Erlang, Akka, actix.

### (a) Native shape

```scala
class Node(id: Int) extends Actor {
  var state: Long = 0
  def receive = {
    case Msg(v) =>
      state += v
      if (state > threshold) neighbors.foreach(_ ! Msg(state / neighbors.size))
  }
}
// thousands of actors, each scheduled independently; a message chain of length
// N completes in N hops with no global barrier.
```

### (b) Whitefoot shape

```whitefoot
// Restructuring note: actor -> a row in `state`; mailbox -> a per-actor inbox
// slice; `send` -> an append to the sender's own outbox; delivery -> one
// sequential scatter pass per round. This is BSP, not actors.
fn round(state: &uniq buffer<NodeState>,
         inbox: &buffer<Msg>, inbox_at: &buffer<u32>,
         outbox: &uniq buffer<Msg>, outbox_n: &uniq buffer<u32>,
         n: own u64) -> result: own u64
  reads(state, inbox, inbox_at, outbox, outbox_n),
  writes(state, outbox, outbox_n) contract {
  requires n <= len_of(deref(state));
} {
  // Permitted: each iteration writes state[i] and outbox_n[i], both affine
  // a=1,b=0 on two distinct roots; `inbox` and `inbox_at` are read-only roots.
  //  *** but the outbox: writing outbox[i*FANOUT + j] for j in 0..FANOUT is
  //  *** DENIED — the offset i*FANOUT+j carries two symbolic terms. So the
  //  *** per-actor outbox must be a fixed-arity struct row, not a slice.
  for (i in 0_u64..n) {
    let produced = step(node: deref(state)[i],
                        inbox: inbox, at: deref(inbox_at)[i]);
    set deref(state)[i]    = produced.state;
    set deref(outbox_n)[i] = produced.sent;        // a fixed-arity OutRow
  }
  return n;
}

fn deliver(outbox: &buffer<Msg>, outbox_n: &buffer<u32>,
           inbox: &uniq buffer<Msg>, inbox_at: &uniq buffer<u32>, n: own u64)
  -> result: own u64 reads(outbox, outbox_n, inbox, inbox_at),
                     writes(inbox, inbox_at) {
  doc "Sequential: a message's destination is data, so this is a scatter and no
       overlap rule admits it. Two passes: count per destination, then place.";
  /* … */
}
```

And the driver:

```whitefoot
for @rounds (r in 0_u64..rounds) {
  let stepped   = round(state: &uniq deref(state), /* … */);
  let delivered = deliver(/* … */);                 // sequential scatter
  if delivered == 0_u64 { break @rounds; }          // quiescence
}
```

### (c) What changes structurally

This is the largest restructuring in the catalog after the concurrent map.

- **Actors become rows; the scheduler becomes a round loop.** Every actor
  advances exactly once per round. An actor with nothing to do still costs an
  iteration (or the writer maintains an active-set array and pays a compaction
  pass, which is a scatter, which is sequential).
- **`send` cannot write the destination's mailbox.** The destination index is
  data; a data-dependent write is not affine; PAR-2 denies. So sends go to the
  sender's own outbox row and a **sequential delivery pass** per round moves
  them. That pass is O(messages) and is the serial fraction.
- **A per-actor outbox must be fixed-arity.** `outbox[i*FANOUT + j]` is denied
  (two symbolic terms). Either the outbox row is a struct with a compile-time
  fan-out bound — which caps how many messages an actor may send per round — or
  the send phase is sequential too. This is a genuine expressiveness cap, not
  just a cost.
- **Message-chain latency becomes rounds.** A request that in Akka travels
  A→B→C→D in 4 hops of ~1 µs takes 4 rounds here, and a round costs a full pass
  over all actors plus a delivery pass. With 10,000 actors at 100 ns each, a
  round is ~1 ms and the chain takes 4 ms instead of 4 µs — **1,000x**.
- **Supervision, timeouts, mailbox overflow, selective receive:** none of them
  have an analogue. Supervision becomes an error enum in the row; a timeout
  becomes a round counter in the row; mailbox overflow becomes a `requires` on
  the inbox capacity that the writer must discharge.

Proof obligations: inbox capacity bounds (the scatter must prove it fits), the
per-actor fan-out bound, and the ordinary affine writes.

### (d) Performance expectation

Throughput, message-dominated workload: the step phase is a PAR-2 map at
1.03–1.14 of native, and the delivery pass is sequential. If messages per round
are M and actors are N, the round is `(N × step)/W + M × scatter`. For a graph
where M ≈ N and scatter is ~10 ns while step is ~100 ns, the round is
`N×100/4 + N×10` = `35N` ns against a perfectly parallel `25N` — so **~1.4x
worse throughput** from the serial scatter alone on 4 cores, worsening linearly
with W (at W=16 it is 10+6.25 = 16.25 against 6.25, **2.6x**). The serial
scatter is an Amdahl term and it dominates as cores grow.

Latency: as above, chain latency = hops × round, which is a 100–1,000x
regression on any request/response pattern implemented as actors.

Memory: much better. No actor objects, no per-actor mailbox allocation, no
scheduler queues. Two message arrays and two state arrays.

**Strictly worse case:** a sparse, bursty actor system — 100,000 actors of which
20 are active per round. BSP costs a pass over 100,000 rows to find 20;
Akka costs 20 scheduler dequeues. That is a **5,000x** work amplification, and
the fix (an active-set array) reintroduces a sequential compaction pass. This is
the shape where the model is not merely slower but qualitatively wrong.

### (e) Verdict

**Restructure, bounded loss — and the bound is large.** Costs: a sequential
delivery scatter per round (an Amdahl term that grows with W), a compile-time
fan-out cap per actor, chain latency measured in rounds rather than hops, and
work proportional to the actor count rather than the active count.

---

## 11. Per-core shared-nothing (Seastar / DPDK)

N cores, each with its own loop and its own data, occasional cross-core messages.

### (a) Native shape

```cpp
// one of these per core, pinned, never sharing a cache line with another
for (;;) {
    n = rte_eth_rx_burst(port, qid, bufs, BURST);
    for (i = 0; i < n; i++) handle(&local_state, bufs[i]);
    drain_cross_core_ring(&inbox[qid]);         // rare
    rte_eth_tx_burst(port, qid, out, m);
}
```

### (b) Whitefoot shape

```whitefoot
// Restructuring note: N independent loops become one outer round loop with an
// N-wide inner phase; per-core state becomes state[c]; the cross-core ring
// becomes the same BSP scatter as architecture 10.
// [new] mut_slice_of range loans give each lane its own scratch window.
fn serve_round(state: &uniq buffer<CoreState>,
               rx: &buffer<Frame>, rx_at: &buffer<u32>,
               tx: &uniq buffer<Frame>, tx_n: &uniq buffer<u32>,
               cores: own u64) -> result: own u64
  reads(state, rx, rx_at, tx, tx_n), writes(state, tx, tx_n) {
  for (c in 0_u64..cores) {
    let done = handle_burst(core: c, st: deref(state)[c],
                            rx: rx, at: deref(rx_at)[c]);
    set deref(state)[c] = done.state;
    set deref(tx_n)[c]  = done.produced;
  }
  return cores;
}
```

with the same `deliver`-style sequential pass for cross-core messages, and the
whole thing wrapped in a `loop @rounds` that is §0.1's batch shape when the
burst receive is I/O: one `wait_batch` over the per-core receive pendings, the
[PAR-2] handler loop above, then a sequential pass that re-issues one receive
per core.

Whether the per-core *packet buffers* can be per-lane windows of one pool
depends on the same range-loan question as architecture 7: PAR-2 denies
(exclusive loan rooted outside the loop), PAR-1 permits with the core count
written in the source.

### (c) What changes structurally

- **The N loops become one loop with N iterations, and they synchronize every
  round.** Seastar's cores never wait for each other. Here, a core that receives
  a 64-packet burst and a core that receives none both reach the round barrier
  together, and the round costs the maximum. With the observed packet-arrival
  skew across RSS queues this is a real tax.
- **"Occasional cross-core message" becomes a per-round scatter pass** whether or
  not any message was sent — unless the writer guards it with a cheap
  "any sent?" reduction (`ior` over the per-core sent flags, which *is* an
  admitted accumulator). That guard is worth writing; it turns the common case
  into one parallel reduction and a predicted-not-taken branch.
- **Core count is the iteration count**, so it can be a runtime value — unlike
  the PAR-1 form. That is the argument for keeping per-core state in rows rather
  than in range-loan windows wherever the work per core fits the affine-element
  shape.
- **Pinning, NUMA placement and cache-line padding are gone from the source.**
  The compiler and runtime own them. That removes a class of bugs and removes a
  class of tuning.

### (d) Performance expectation

Throughput at low skew: near parity *while the loop is saturated*. A DPDK-style
round processes a 32-packet burst per core at ~100 ns/packet = ~3 µs of work. A
saturated poll loop runs its rounds back to back, so the lanes stay hot inside
the 1,000 µs idle window and the barrier costs spin rounds at 11.6 ns —
**well under 1% at burst=32**. An *unsaturated* loop is the bad case, and it is
the one the corrected park figure changes: once a lane parks, the next round
costs the Linux **10–16 µs** park-and-wake against ~3 µs of work, i.e.
**300–500% overhead at burst=32**, falling to ~40–65% at burst=256. Whitefoot
therefore wants either a saturated loop or larger bursts than Seastar does —
and the second costs latency.

Throughput at realistic skew: worse. The round is `max` over cores, not `mean`.
For RSS-hashed traffic with a 2:1 queue imbalance, expect **~1.5x** the
per-round cost of the balanced case, against Seastar's zero (each core just
keeps going).

Latency: a packet's service latency becomes (its position in the burst) +
(the round's max). At burst=256 and 100 ns/packet, that is ~25 µs p50 against
DPDK's ~1–3 µs, plus one `wait_batch` blocking call per round. No measurement in
this repository covers the batch model at this shape; the echo-server p50 of
245 µs at 64 connections belongs to the **retired staged runtime** and is not
quoted here as this model's anchor. Estimate: **~10–25x p50 latency against
DPDK at burst=256**, from the burst period alone.

Memory: comparable; better if the per-core pools are windows of one arena.

**Strictly worse case:** heavy, persistent queue imbalance with a tight latency
budget — the case per-core shared-nothing exists for. Cores idle at the barrier
and the p99 is set by the hottest queue.

### (e) Verdict

**Restructure, bounded loss.** Costs: a round barrier where native has none
(throughput loss ≈ the skew ratio, latency loss ≈ the burst period), and
cross-core messages become a sequential per-round scatter.

---

## 12. Structured concurrency / scoped threads

`std::thread::scope`, join handles, a few heterogeneous tasks, join all.

### (a) Native shape

```rust
let mut hist = vec![0u32; 256];
let mut sum = 0u64;
std::thread::scope(|s| {
    let h1 = s.spawn(|| checksum(&data));
    let h2 = s.spawn(|| build_index(&data, &mut idx));
    let h3 = s.spawn(|| compress(&data, &mut out));
    sum = h1.join().unwrap();
    h2.join().unwrap();
    h3.join().unwrap();
});
```

### (b) Whitefoot shape

This is PAR-1, unchanged and unrestructured. `scope` is the block; the join is
the first use of the result; the borrow checker's scoped-borrow guarantee is
the loan rule.

```whitefoot
fn analyze(data: &buffer<u8>, idx: &uniq buffer<u32>, out: &uniq buffer<u8>)
  -> result: own u64 reads(data, idx, out), writes(idx, out) {
  region {
    // Three statements, one block. Every ordered pair is footprint-disjoint:
    // `checksum` writes nothing, `build_index` writes only idx, `compress`
    // writes only out, and no argument of a later one reads a binding an
    // earlier one defines. [PAR-1] admits the chain.
    let sum     = checksum(data: data);
    let indexed = build_index(data: data, out: &uniq deref(idx));
    let packed  = compress(data: data, out: &uniq deref(out));
    return sum +wrap indexed +wrap packed;
  }
}
```

### (c) What changes structurally

Nothing. Three notes rather than costs:

- The join is implicit — it is the first statement that reads a result, or the
  end of the window. There is no handle and nothing to forget to join.
- **`&data` shared by all three is fine** (two overlapping shared loans deny
  nothing), which is the common case and the one `thread::scope` was invented
  for.
- The one discipline: `checksum` must not declare `writes(data)` or anything
  reaching `idx`/`out`, and each callee's effect row must be tight. A function
  that takes `&uniq World` and declares `writes(world)` when it touches one
  field will close every window it is in.

### (d) Performance expectation

Parity. This is the measured `par_layout.wf` path: the eligible sibling-pair
phase reached **2.98x on 4 performance cores (75% of ideal)**, with the
shortfall attributed to the critical path, the lane-budget inline fallback and
the fork/join edges. For three heterogeneous tasks of similar size on 4 cores,
expect the same 70–80% of ideal, which is what `thread::scope` delivers too
(thread spawn is ~10–30 µs, so for short tasks Whitefoot's pool is *faster*).

Latency: better for short tasks — no `pthread_create`. Memory: better — no
per-task stack allocated on demand; the pool's stacks are preallocated.

**Strictly worse case:** tasks that are individually tiny (< ~10 µs). Then the
lane-budget policy refuses the offer and everything runs inline — which is
correct but means no speedup, where `thread::scope` with a pre-warmed pool would
also not help. Genuinely no loss here.

### (e) Verdict

**Direct.** This is the architecture Whitefoot's model was designed around.

---

## 13. Level-synchronous graph algorithms and irregular reductions

Parallel BFS frontier expansion; histogram; counting sort.

### (a) Native shape

```c
// top-down BFS level
#pragma omp parallel for schedule(dynamic, 64)
for (size_t f = 0; f < frontier_n; ++f) {
    uint32_t u = frontier[f];
    for (uint32_t e = off[u]; e < off[u+1]; ++e) {
        uint32_t v = adj[e];
        if (__sync_bool_compare_and_swap(&parent[v], UINT32_MAX, u))
            next[__sync_fetch_and_add(&next_n, 1)] = v;   // atomic append
    }
}

// histogram
#pragma omp parallel
{ uint32_t priv[256] = {0};
  #pragma omp for
  for (i = 0; i < n; ++i) priv[key[i]]++;
  #pragma omp critical
  for (b = 0; b < 256; ++b) hist[b] += priv[b]; }
```

### (b) Whitefoot shape — BFS

The top-down sparse frontier is **not expressible in parallel**: `parent[v]` and
`next[next_n++]` are both data-dependent (scattered) writes, denied by PAR-2 on
the affine condition, and there is no atomic.

The **dense / bottom-up** form is expressible and permitted, because every write
is `x[i]` with the loop's own binder:

```whitefoot
// Restructuring note: sparse frontier -> dense bitmaps over all vertices;
// atomic append -> a per-vertex flag plus a sequential compaction (or none, if
// the next level is also dense). This is direction-optimizing BFS's bottom-up
// step, used unconditionally.
fn bfs_level(off: &buffer<u32>, adj: &buffer<u32>,
             visited: &buffer<u8>, frontier: &buffer<u8>,
             next: &uniq buffer<u8>, parent: &uniq buffer<u32>, n: own u64)
  -> result: own u64
  reads(off, adj, visited, frontier, next, parent), writes(next, parent) contract {
  requires n <= len_of(deref(next));
  requires n <= len_of(deref(parent));
} {
  let found = 0_u64;
  for (v in 0_u64..n) {
    // reads: off, adj, visited, frontier — whole roots, none written by B.
    // writes: next[v] and parent[v] — two roots, each affine a=1,b=0.
    let hit = scan_in_neighbours(off: off, adj: adj, frontier: frontier, v: v);
    set deref(next)[v]   = hit.flag;
    set deref(parent)[v] = hit.from;
    set found = found +wrap hit.count;     // one accumulator, +wrap: admitted
  }
  return found;
}
```

Two accumulators would be denied; here `found` is the only one.

### (b′) Whitefoot shape — histogram

**Not expressible under PAR-2 in any form.** The privatized histogram needs a
per-lane *array* of counters, and PAR-2's accumulator is one whole place
combined with one scalar operation. `hist[key[i]] +wrap= 1` is a scattered
write. A per-lane window `mut_slice_of(&uniq hist, lane*256, lane*256+256)`
forms an exclusive loan rooted outside the loop, which PAR-2 denies.

The expressible form is PAR-1 with the lane count in the source:

```whitefoot
// Restructuring note: W is a source constant. `hist` is W*256 wide; a
// sequential fold collapses it afterwards.
fn histogram(key: &buffer<u8>, n: own u64, hist: &uniq buffer<u32>)
  -> result: own u64 reads(key, hist), writes(hist) contract {
  requires len_of(deref(hist)) == 1024_u64;          // 4 lanes * 256 buckets
  requires n <= len_of(deref(key));
} {
  let quarter = n / 4_u64;
  region {
    let h0 = mut_slice_of(&uniq deref(hist),   0_u64,  256_u64);   // [new]
    let h1 = mut_slice_of(&uniq deref(hist), 256_u64,  512_u64);   // [new]
    let h2 = mut_slice_of(&uniq deref(hist), 512_u64,  768_u64);   // [new]
    let h3 = mut_slice_of(&uniq deref(hist), 768_u64, 1024_u64);   // [new]
    let a = count_into(key: key, first: 0_u64,              end: quarter,          out: &uniq h0);
    let b = count_into(key: key, first: quarter,            end: 2_u64 *wrap quarter, out: &uniq h1);
    let c = count_into(key: key, first: 2_u64 *wrap quarter, end: 3_u64 *wrap quarter, out: &uniq h2);
    let d = count_into(key: key, first: 3_u64 *wrap quarter, end: n,                out: &uniq h3);
    let total = a +wrap b +wrap c +wrap d;
    // sequential fold of 4 x 256 counters into the first 256
    return fold_lanes(hist: &uniq deref(hist), lanes: 4_u64, buckets: 256_u64) +wrap total;
  }
}
```

### (c) What changes structurally

BFS:
- **The frontier stops being sparse.** Every level costs O(V) plus the in-edges
  scanned, instead of O(frontier + its out-edges). For a small-world graph whose
  middle levels hold most of the vertices, this is nearly free and is what
  direction-optimizing BFS does anyway. For a road network or a long path graph
  — diameter in the thousands, frontier of a few hundred — the cost is
  **O(V · D)** against **O(V + E)**. On a 1M-vertex graph with diameter 1,000
  that is a 1,000x work amplification.
- **No atomic `parent` claim**, so a vertex reached from two parents in one level
  must resolve deterministically: `scan_in_neighbours` picks (say) the lowest
  in-neighbour in the frontier, which is *more* deterministic than the native
  CAS race and costs a full in-neighbour scan rather than an early exit.
- The compaction back to a sparse frontier (if wanted) is a scatter: sequential.

Histogram / counting sort:
- **W is a source constant.** Again.

  **`[new]` — the affine-range extension removes this for the count pass.** With
  an iteration-exclusive loan on `[c*i + b, c*(i+1) + b)` admitted by [PAR-2],
  the privatized histogram is written as an ordinary counted loop over lanes:

  ```whitefoot
  for (lane in 0_u64..lanes) {                     // [new] affine range loan
    let win = mut_slice_of(&uniq hist, lane *wrap 256_u64, lane *wrap 256_u64 +wrap 256_u64);
    set counted[lane] = count_into(key: &key, first: lane *wrap chunk,
                                   end: lane *wrap chunk +wrap chunk, out: &uniq win);
  }
  ```

  `lanes` is then a runtime value. *What changes in the verdict:* the histogram
  goes from "restructure, bounded loss (W in source, ~0.9–1.0 of native *if W
  matches the host*)" to **restructure, no loss** — the per-lane privatization is
  the native algorithm, and the lane count now matches the machine. *What does
  not change:* counting sort stays **bounded loss**, because its scatter phase is
  a data-dependent write and no range rule reaches it; the verdict there is set
  by the scatter, not by W.
- The counting-sort *scatter* phase (`out[pos[key[i]]++] = i`) is a data-dependent
  write and stays sequential under every rule. In counting sort the scatter is
  the expensive pass, so the parallelizable fraction is the count pass and the
  prefix sum only.

Proof obligations: the range-loan disjointness (four constants — trivial); the
histogram's `requires len_of(hist) == W*B`; for BFS, the in-neighbour scan's
bounds against `off`/`adj`, which is the ordinary `[OP-4]` work.

### (d) Performance expectation

BFS on a small-world graph (social, web): the level loop is a PAR-2 map at the
measured 1.03–1.14 ratio, so **~0.9x of an OpenMP direction-optimizing BFS** —
close, because the native implementation is doing the same dense pass. Whitefoot
loses only the top-down levels it cannot use.

BFS on a high-diameter graph: **catastrophic**, and the number is the diameter.
This is not a constant-factor loss; it is an asymptotic one, O(V·D) versus
O(V+E).

Histogram: the count pass parallelizes to ~W with a cheap body; the fold is
W×B = 1,024 adds, negligible. Expect **~0.9–1.0x of the OpenMP privatized
form** on 4 cores — but only because W matched the machine. On an 8-core host
the same source uses 4 lanes and delivers half the available throughput; on a
2-core host it oversubscribes by 2 and the four calls are run inline by the lane
budget, costing nothing but gaining nothing. **The source is tuned to a machine.**

Counting sort: Amdahl-bound by the sequential scatter. If count is 25% of the
runtime and scatter is 60%, the best possible speedup on 4 cores is
`1/(0.6 + 0.4/4)` = **1.43x** against a native parallel counting sort's ~3x.
About **0.5 of native.**

Memory: BFS dense form needs O(V) bitmaps per level instead of O(frontier) —
usually smaller in absolute terms (one byte per vertex) but always paid.
Histogram needs W×B counters, same as the native privatized form.

**Strictly worse case:** BFS on a high-diameter graph (above), and any histogram
on a machine whose core count differs from the literal in the source.

### (e) Verdict

BFS: **restructure, bounded loss** on low-diameter graphs; **bounded loss that
is asymptotic, not constant** on high-diameter graphs — the dense form is the
wrong algorithm there.
Histogram / counting sort: **restructure, bounded loss** — W is a source
constant, and the scatter phase stays sequential (≈0.5 of native for counting
sort). **`[new]`: with the affine-range extension the histogram becomes
restructure, no loss**; counting sort stays bounded loss, because its verdict is
set by the sequential scatter and not by W.

---

## 14. Stencil / simulation time-stepping with double buffering and halo exchange

### (a) Native shape

```c
for (int t = 0; t < steps; ++t) {
    #pragma omp parallel for collapse(2)
    for (int y = 1; y < H-1; ++y)
      for (int x = 1; x < W-1; ++x)
        next[y*W+x] = 0.25f*(cur[(y-1)*W+x] + cur[(y+1)*W+x]
                           + cur[y*W+x-1]   + cur[y*W+x+1]);
    swap(cur, next);
    exchange_halos(cur);            // MPI, if distributed
}
```

### (b) Whitefoot shape — 1-D: direct

```whitefoot
fn step_1d(cur: &buffer<f64>, next: &uniq buffer<f64>, n: own u64)
  -> result: own u64 reads(cur, next), writes(next) contract {
  requires n <= len_of(deref(cur));
  requires n <= len_of(deref(next));
  requires 2_u64 <= n;
} {
  for (i in 1_u64..n - 1_u64) {
    // write: next[i], affine a=1,b=0. reads: cur, a whole root B never writes.
    // Footprints disjoint across iterations: [PAR-2] permits.
    set deref(next)[i] = blend(l: deref(cur)[i - 1_u64],
                               c: deref(cur)[i],
                               r: deref(cur)[i + 1_u64]);
  }
  return n;
}
```

### (b′) Whitefoot shape — 2-D: **the flat row-major form is denied**

```whitefoot
for (y in 1_u64..h - 1_u64) {
  for (x in 1_u64..w - 1_u64) {
    set deref(next)[y *wrap w +wrap x] = /* … */;
    //  ^ inner loop binder is x; canonical offset is 1*x + (y*w).
    //    `y*w` is a symbolic term, not a mathematical integer constant.
    //    [PAR-2] DENIES. The outer loop over y denies for the mirror reason.
  }
}
```

Two expressible replacements. First, **rows as their own buffers**, so the write
is `row[x]` with `b = 0` — but obtaining `&uniq next_rows[y]` inside the loop is
a borrow formed by a non-call statement, which PAR-2 also denies. So this only
works if the row loop is the *outer* loop and each iteration writes exactly one
element of a row-pointer array — which is not what a stencil does.

Second, and the one that works today, **a hand-written W-way band decomposition
under PAR-1**:

```whitefoot
// Restructuring note: the 2-D loop nest becomes W hand-written band calls.
// W is a source constant. [new] mut_slice_of range loans; the four bands are
// proved disjoint by linear arithmetic over `band = (h - 2) / 4`.
fn step_2d(cur: &buffer<f64>, next: &uniq buffer<f64>, w: own u64, h: own u64)
  -> result: own u64 reads(cur, next), writes(next) contract {
  requires 6_u64 <= h;
  requires len_of(deref(cur)) == w *wrap h;
  requires len_of(deref(next)) == w *wrap h;
} {
  let band = (h - 2_u64) / 4_u64;
  region {
    let b0 = mut_slice_of(&uniq deref(next), 1_u64 *wrap w,                    (1_u64 +wrap band) *wrap w);
    let b1 = mut_slice_of(&uniq deref(next), (1_u64 +wrap band) *wrap w,       (1_u64 +wrap 2_u64 *wrap band) *wrap w);
    let b2 = mut_slice_of(&uniq deref(next), (1_u64 +wrap 2_u64 *wrap band) *wrap w, (1_u64 +wrap 3_u64 *wrap band) *wrap w);
    let b3 = mut_slice_of(&uniq deref(next), (1_u64 +wrap 3_u64 *wrap band) *wrap w, (h - 1_u64) *wrap w);
    // Four disjoint exclusive loans; `cur` is shared by all four. One [PAR-1] chain.
    let r0 = band_rows(cur: cur, out: &uniq b0, w: w, y0: 1_u64,                     y1: 1_u64 +wrap band);
    let r1 = band_rows(cur: cur, out: &uniq b1, w: w, y0: 1_u64 +wrap band,          y1: 1_u64 +wrap 2_u64 *wrap band);
    let r2 = band_rows(cur: cur, out: &uniq b2, w: w, y0: 1_u64 +wrap 2_u64 *wrap band, y1: 1_u64 +wrap 3_u64 *wrap band);
    let r3 = band_rows(cur: cur, out: &uniq b3, w: w, y0: 1_u64 +wrap 3_u64 *wrap band, y1: h - 1_u64);
    return r0 +wrap r1 +wrap r2 +wrap r3;
  }
}
```

**`[new]` — the affine-range extension turns this into an ordinary loop, and
this is the case where it matters most.** A row of a row-major grid is
`[y*w, y*w + w)`, which is exactly an affine range `[c*i + b, c*(i+1) + b)` with
`c = w` and `b = 0`. So if [PAR-2] admits an iteration-exclusive loan on an
affine range, the whole hand-written four-way decomposition above collapses to:

```whitefoot
for (y in 1_u64..h - 1_u64) {                     // [new] affine range loan
  let row = mut_slice_of(&uniq deref(next), y *wrap w, y *wrap w +wrap w);
  set done[y] = stencil_row(cur: cur, out: &uniq row, w: w, y: y);
}
```

One caveat, stated because it decides whether the extension reaches this case at
all: `w` is a runtime value, not a literal. The *element* rule requires `a` and
`b` to be mathematical integer constants because it refines the write footprint
to a single element syntactically. A *range* rule does not need that — the
disjointness of `[c*i, c*(i+1))` across distinct `i` is `c >= 0` plus
monotonicity, which is linear arithmetic over a symbolic `c`, and linear
arithmetic is already the obligation the accepted range loans carry. **So the
same range-loan rule does cover `[y*w, y*w+w)`, provided `c` may be a
loop-invariant value rather than a literal.** If it is restricted to literals, it
does not reach 2-D at all and this section's verdict is unchanged.

*What changes in the verdict:* 2-D stencil goes from "restructure, bounded loss
(hand-written W-way bands, W a source constant)" to **restructure, no loss** —
the loop is written the way anyone would write it, the compiler chunks the row
range, and the band count matches the machine.

*What it does not fix:* the inner element write `next[y*w + x]` of §0.2 stays
denied. That is the element rule's constant requirement, a separate narrowness,
and the range extension routes around it (write rows, not elements) rather than
removing it. A kernel that genuinely needs the inner loop parallel — a very wide,
very short grid, where there are fewer rows than cores — is still denied.

The time loop alternates buffers with a parity branch rather than a swap:

```whitefoot
for @steps (t in 0_u64..steps) {
  if even(t) { let m = step_2d(cur: &a, next: &uniq b, w: w, h: h); }
  else       { let m = step_2d(cur: &b, next: &uniq a, w: w, h: h); }
}
```

### (c) What changes structurally

- **Halo exchange disappears entirely.** There is one address space and one
  control flow; the halo exists in MPI/threaded codes only because the domain is
  split across memories or because a thread must not read a neighbour's
  in-progress write. Here `cur` is read-only during the step and every lane reads
  whatever it likes. This is a real, structural simplification: the ghost cells,
  the pack/unpack buffers, the send/recv pairing, and the deadlock class that
  comes with it are all gone.
- **1-D is free. 2-D costs a hand-written W-way decomposition** (or a
  row-of-buffers layout that hurts locality and does not compose with PAR-2
  either).
- **The buffer swap is a parity branch**, doubling the call site. A `replace`-
  based swap would avoid that; it is not needed in the inner loop, so the cost
  is cosmetic.
- Boundary conditions are ordinary source and unchanged.

Proof obligations: band disjointness (linear arithmetic over `band`, `w`, `h` —
the `requires 6 <= h` is there to keep `band >= 1`), and the per-band bounds so
`cur[(y-1)*w+x]` is in range, which needs `y0 >= 1` and `y1 <= h-1` in
`band_rows`'s contract.

### (d) Performance expectation

1-D: the measured map ratio, **1.03–1.14 of the best reference**, or better —
the `fir` kernel is structurally a 1-D stencil with 64 taps and Whitefoot is
*fastest* on it (0.906–0.954 across both hosts).

2-D under the band decomposition: the inner work is identical to the native
band-parallel form, so throughput should be within ~5–10% — a band call is one
outlined call per lane, not one per row, so the fork/join cost is 4 edges per
time step. At a 1,024×1,024 grid and ~1 ns/cell, a step is ~1 ms and 4 fork-join
edges are ~5 µs: **0.5% overhead**. The loss is not performance; it is that W is
a literal.

Memory: better. Two grids, no ghost regions, no pack buffers.

**Strictly worse case:** a small grid with many time steps — 64×64 at 4 µs/step.
Back-to-back steps keep the lanes hot inside the 1,000 µs idle window, so the
per-step fork-join is spin rounds at 11.6 ns and the overhead is **well under
1%** — the corrected park figure does *not* hurt here, because the loop never
lets a lane park. A simulation whose steps are separated by more than the idle
window (a step per frame, a step per request) pays the Linux **10–16 µs**
park-and-wake per step instead, which against a 4 µs step is
**250–400% overhead**. Native OpenMP
with a persistent team and a barrier has the same problem and solves it with
`#pragma omp parallel` hoisted outside the time loop, which Whitefoot cannot do:
each step is its own fork-join because the time loop is sequential and every
overlap is fork-join. **This is a real, structural loss for small-domain,
many-step simulations, and it is roughly the barrier cost divided by the step
cost.**

### (e) Verdict

1-D: **direct.**
2-D: **restructure, bounded loss** — a hand-written W-way band decomposition
with range loans, W as a source constant. **`[new]`: with the affine-range
extension of (b′) — and provided the range rule's `c` may be a loop-invariant
value, since a row is `[y*w, y*w+w)` — 2-D becomes restructure, no loss**, an
ordinary `for` over rows. The inner `next[y*w+x]` element form stays denied
either way. Halo exchange is deleted, which is a
gain. Small-domain many-step simulations lose the persistent-team optimization
and pay a fork-join per step.

---

## 15. Parallel sort and prefix sum

Sample sort, parallel quicksort partition, scan.

### (a) Native shape

```cpp
// prefix sum: Blelloch / three-pass
parallel_for(0, nb, [&](int b){ bsum[b] = reduce(a + b*C, C); });
exclusive_scan(bsum, bsum + nb, base, 0);
parallel_for(0, nb, [&](int b){ scan_into(a + b*C, out + b*C, C, base[b]); });

// sample sort
parallel_for(...)  local_histogram_of_splitters();
exclusive_scan(global_offsets);
parallel_for(...)  scatter_to_buckets();          // irregular writes
parallel_for(...)  sort_each_bucket();
```

### (b) Whitefoot shape — prefix sum

Pass 1 is a clean PAR-2 loop and needs nothing new: each iteration writes one
element of the block-sum array, and the input is a read-only root.

```whitefoot
fn block_sums(a: &buffer<u64>, bsum: &uniq buffer<u64>, nb: own u64, c: own u64)
  -> result: own u64 reads(a, bsum), writes(bsum) contract {
  requires nb <= len_of(deref(bsum));
  requires nb *wrap c <= len_of(deref(a));
} {
  for (b in 0_u64..nb) {
    // bsum[b]: affine a=1,b=0. reduce_range reads root `a`, never written here.
    set deref(bsum)[b] = reduce_range(a: a, first: b *wrap c, end: b *wrap c +wrap c);
  }
  return nb;
}
```

Pass 2 is a sequential exclusive scan over `nb` entries — cheap and correct.

Pass 3 is the wall. It must write a contiguous *range* of `out` per block, so
under PAR-2 it needs an exclusive loan on `out` rooted outside the loop:
**denied**. It drops to PAR-1 with W in the source, exactly as in architectures
7, 13 and 14.

**`[new]` — this is the cleanest case for the affine-range extension.** Pass 3's
block `b` writes exactly `[b*c, (b+1)*c)`, which is the affine range
`[c*i + b, c*(i+1) + b)` with offset zero — the literal shape of the rule. With
it, pass 3 is the mirror image of pass 1:

```whitefoot
for (b in 0_u64..nb) {                            // [new] affine range loan
  let win = mut_slice_of(&uniq out, b *wrap c, b *wrap c +wrap c);
  set written[b] = scan_into(a: a, out: &uniq win, first: b *wrap c,
                             end: b *wrap c +wrap c, base: deref(base)[b]);
}
```

*What changes in the verdict:* prefix sum goes from "restructure, bounded loss
(W in source)" to **restructure, no loss** — three passes, all written as
ordinary loops, none of them naming a lane count. *What does not change:* the
sort verdicts below, whose bound is the sequential scatter and the sequential
merge, neither of which is a range question.

The form without the extension:

```whitefoot
// Restructuring note: the writeback pass is W hand-written calls over W range
// loans. W is a source constant. [new] mut_slice_of.
region {
  let o0 = mut_slice_of(&uniq out, 0_u64,   q);
  let o1 = mut_slice_of(&uniq out, q,       2_u64 *wrap q);
  let o2 = mut_slice_of(&uniq out, 2_u64 *wrap q, 3_u64 *wrap q);
  let o3 = mut_slice_of(&uniq out, 3_u64 *wrap q, n);
  let s0 = scan_into(a: &a, out: &uniq o0, first: 0_u64,   end: q,  base: base0);
  let s1 = scan_into(a: &a, out: &uniq o1, first: q,       end: 2_u64 *wrap q, base: base1);
  let s2 = scan_into(a: &a, out: &uniq o2, first: 2_u64 *wrap q, end: 3_u64 *wrap q, base: base2);
  let s3 = scan_into(a: &a, out: &uniq o3, first: 3_u64 *wrap q, end: n, base: base3);
}
```

### (b′) Whitefoot shape — sort

- **Sample/counting the splitters:** architecture 13's histogram — PAR-1, W in
  the source.
- **Offsets:** the prefix sum above.
- **Scatter to buckets:** `out[pos[key[i]]++] = i` — data-dependent write.
  Denied by every rule. **Sequential.**
- **Sort each bucket:** PAR-1 over W range loans (one per bucket, if buckets are
  contiguous windows of one output buffer), or PAR-2 if each bucket's sort could
  be expressed as "write one element per iteration", which it cannot.
- **In-place quicksort partition** (Hoare two-pointer) is a data-dependent write
  into the array being read: denied, sequential, at every level. Only the
  *recursion* after the partition parallelizes, via PAR-1 sibling calls, which
  is architecture 2 — which, on the 2026-09-12 tables, is parity at W≤4
  (0.97–1.07) rather than the 1.58x earlier drafts carried.

### (c) What changes structurally

- **Scan becomes three passes with a hand-fixed writeback width**, which is what
  a native implementation does anyway except for the hand-fixed part.
- **Sort loses the parallel partition.** Every comparison sort's parallel form
  rests on either an irregular scatter (sample sort) or an in-place partition
  (quicksort). Both are data-dependent writes. What is left is: parallel
  *per-block* sorts into disjoint output windows (PAR-1, W fixed) plus a merge.
  A parallel merge is itself a data-dependent write and is sequential, so the
  merge tree is `log2(W)` sequential passes over n elements.
- **The realistic Whitefoot sort is: W-way block sort (parallel) + log2(W)-deep
  sequential merge.** For W=4 that is one parallel pass and 2 sequential passes
  over n.

Proof obligations: the range-loan disjointness (constants and one quotient), the
per-block bounds, and — for the merge — the ordinary index obligations. Nothing
exotic; the cost is in the algorithm, not the proof.

### (d) Performance expectation

**Prefix sum:** near parity. All three passes exist in the native form; the only
Whitefoot-specific cost is that pass 3 has W lanes fixed at source time.
Throughput estimate on 4 cores: `2n` parallel reads/writes plus a tiny scan, so
~3.5x of sequential against a native ~3.7x — **~0.95 of native.**

**Sort:** clearly worse. Model it: a W=4 block sort costs `(n/4)·log(n/4)`
comparisons per lane in parallel, then 2 sequential merge passes at `2n`
element moves. Against a sample sort that is ~3x parallel throughout. Rough
estimate: Whitefoot reaches `1 / (0.25·(log(n/4)/log n) + 2·(n / (n log n)))` —
for n = 10^7, log n ≈ 23: parallel part ≈ 0.25 · 21/23 ≈ 0.23 of the work in
parallel, merge ≈ 2/23 ≈ 0.087 of the work sequential, giving a speedup of
about `1/(0.23 + 0.087)` ≈ **3.2x** against sequential — actually competitive,
because merging is cheap relative to sorting at large n. At small n (10^4,
log n ≈ 13) the merge fraction rises to 2/13 = 0.15 and the speedup falls to
~2.6x. So sort is **0.7–0.9 of a good parallel sort** on 4 cores, which is
better than I expected before doing the arithmetic and is worth checking with a
real measurement before believing.

**Counting sort / radix sort:** worse, per architecture 13 — the scatter is the
dominant pass and it is sequential. ~0.5 of native.

Latency and memory: the block-sort-plus-merge form needs a second full buffer
(so does sample sort). No difference.

**Strictly worse case:** in-place sorting under a memory constraint. Every
Whitefoot parallel sort needs out-of-place output windows, because in-place
partitioning is a data-dependent write. Memory doubles.

### (e) Verdict

Prefix sum: **restructure, bounded loss** — three passes with W in the source
for the writeback; ~0.95 of native. **`[new]`: with the affine-range extension
this becomes restructure, no loss** — pass 3 is an ordinary counted loop and no
lane count appears in the source.
Comparison sort: **restructure, bounded loss** — no parallel partition, no
parallel merge; block-sort-plus-sequential-merge at ~0.7–0.9 of native on 4
cores, and out-of-place only.
Radix/counting sort: **restructure, bounded loss (large)** — the scatter is
sequential, ~0.5 of native.

---

## 16. Background periodic thread (timer, flush, logger) alongside main work

### (a) Native shape

```c
void *flusher(void *arg) {
    for (;;) {
        nanosleep(&(struct timespec){.tv_sec = 1}, NULL);
        pthread_mutex_lock(&log_m);
        write(fd, buf, used); used = 0;
        pthread_mutex_unlock(&log_m);
    }
}
pthread_create(&t, NULL, flusher, NULL);
main_work();                                     // never thinks about flushing
```

### (b) Whitefoot shape

```whitefoot
// Restructuring note: the background thread becomes a phase in the outer loop,
// guarded by a clock read. The lock disappears because there is no second
// agent. Jitter is bounded by the longest phase, not by the timer.
fn service(work: &uniq Work, log: &uniq LogBuffer, out: &uniq OutputStream,
           rounds: own u64) -> result: own u64
  reads(work, log, out), writes(work, log, out) {
  let last = now_ms();                                   // [new] monotonic clock read
  for @rounds (r in 0_u64..rounds) {
    let did = do_chunk(work: &uniq deref(work), log: &uniq deref(log));
    let t = now_ms();                                    // [new]
    if t - last >= 1000_u64 {
      let flushed = flush(log: &uniq deref(log), out: &uniq deref(out));
      set last = t;
    }
  }
  return 0_u64;
}
```

Or, if the flush should overlap the next chunk, one PAR-1 window with two
buffers — the double-buffered form of architecture 4:

```whitefoot
// flush the filled buffer while the next chunk fills the other
let did     = do_chunk(work: &uniq deref(work), log: &uniq deref(fill));
let flushed = flush(log: &deref(drain), out: &uniq deref(out));
swap(fill: &uniq deref(fill), drain: &uniq deref(drain));   // [new] or parity
```

### (c) What changes structurally

- **The lock disappears.** There is no concurrent writer to the log buffer, so
  the mutex, the `used` counter's atomicity and the flush-during-append hazard
  are gone. That is a genuine simplification.
- **The timer becomes a poll.** The main loop must reach the check. If a phase
  runs for 10 s, the 1 s flush happens at 10 s. **Timer jitter equals the
  longest non-interruptible phase**, and the writer controls it only by making
  phases shorter.
- **Every background duty must be named in the main loop.** A program with four
  background threads (flush, metrics, heartbeat, GC) becomes four guarded phases
  in one loop, and their interleaving is the writer's, in source order.
- **A watchdog is not expressible.** "If the main work hangs, fire" requires an
  agent that runs when the main work does not. There is none. The best available
  substitute is a deadline the *caller* of a long operation checks, which cannot
  detect a hang inside that operation.

### (d) Performance expectation

Throughput: better. One clock read per round (a `clock_gettime` via vDSO, ~20 ns)
replaces a thread, a mutex pair per append (~20–40 ns each) and a wake per
second. For an append-heavy logger this is a clear win: **appends get faster by
the lock cost**, which on a contended log is the whole cost.

Latency: the flush's own latency is unchanged; the *timeliness* is worse by the
phase length.

Memory: one fewer thread stack (8 MB of virtual, ~8–64 KB resident), no
condition variable.

**Strictly worse case:** a heartbeat with a hard deadline next to a phase longer
than the deadline — e.g. a 100 ms cluster heartbeat next to a 500 ms compaction.
The heartbeat is missed and the node is evicted. The fix is to split the
compaction into 50 ms chunks, which costs a fork-join per chunk (hot rounds at
11.6 ns spin rounds, since the chunks are back to back; negligible either way
against 50 ms even at the 10–16 µs park figure) and costs the writer the restructuring. This is architecture 20 in
miniature.

### (e) Verdict

**Restructure, no loss** for flush/metrics/logging, with a throughput gain from
the deleted lock. **Bounded loss** for anything with a hard deadline: timer
jitter equals the longest phase. **Not expressible** for a watchdog that must
observe a hang.

---

## 17. Thread-pool server: one worker per request, blocking I/O per worker

Apache prefork/worker, a Rust `std` server with a fixed pool.

### (a) Native shape

```c
for (;;) {
    int c = accept(listen_fd, NULL, NULL);
    pool_submit(&pool, handle_conn, (void *)(intptr_t)c);
}
// worker thread
void handle_conn(void *arg) {
    int c = (int)(intptr_t)arg;
    for (;;) {
        ssize_t n = read(c, buf, sizeof buf);     // blocks this worker only
        if (n <= 0) break;
        write_all(c, buf, n);
    }
    close(c);
}
```

### (b) Whitefoot shape

Under §0.1's model there is no worker and no stack per connection. There is one
array of connection rows, one array of buffers, one array of `Pending` slots, and
one loop that waits on the batch, advances every completed row as a [PAR-2]
iteration, and reissues.

```whitefoot
// Restructuring note: worker -> row. The per-connection state machine that a
// thread kept in its call stack is written out as `phase` in the row, because
// the row is what survives the wait.
struct ConnRow {
  phase: ConnPhase;          // Accepting | Reading | Writing | Closing | Free
  handle: ConnHandle;
  filled: u64;
  sent: u64;
}

fn advance(row: own ConnRow, buf: &uniq MutSlice<u8>, done: own Completion)
  -> result: own ConnRow reads(row, buf, done), writes(buf) {
  doc "Pure row transition for one completed operation. Touches this row and
       this row's buffer and nothing else, which is what makes the batch loop
       a [PAR-2] loop.";
  match done {
    ReadBytes(next: n) => {
      if n == 0_u64 { return ConnRow(phase: Closing(), /* … */); }
      return ConnRow(phase: Writing(), filled: n, sent: 0_u64, /* … */);
    }
    Wrote(next: n) => {
      if n >= row.filled { return ConnRow(phase: Reading(), /* … */); }
      return ConnRow(phase: Writing(), sent: n, /* … */);
    }
    Failed(error: e) => { return ConnRow(phase: Closing(), /* … */); }
  }
}

command fn main(/* … */) -> status: own ExitStatus /* … */ {
  let rows  = buffer_new(1024_u64, ConnRow(phase: Free(), /* … */));
  let bufs  = buffer_new(1024_u64 *wrap 4096_u64, 0_u8);     // one 4 KiB window per row
  let slots = pending_array(1024_u64);                       // [new] owned Pending slots
  let live  = 0_u64;
  loop @server {
    // 1. one blocking wait for the whole batch
    let ready = wait_batch(slots: &uniq slots);              // [new] returns the ready count,
                                                             //       compacting ready indices
    if ready == 0_u64 { break @server; }

    // 2. the handlers: one [PAR-2] loop over the completed batch.
    //    Iteration k writes rows[k] (affine a=1,b=0) and its own buffer window.
    //    [new] affine range loan for the window; see §0.1 consequence 3.
    for (k in 0_u64..ready) {
      let win = mut_slice_of(&uniq bufs, k *wrap 4096_u64, k *wrap 4096_u64 +wrap 4096_u64);
      set deref(rows)[k] = advance(row: deref(rows)[k], buf: &uniq win,
                                   done: take_completion(slots: &uniq slots, at: k));
    }

    // 3. sequential reissue pass: hands each advanced row its next operation.
    set live = reissue(rows: &uniq rows, bufs: &uniq bufs,
                       slots: &uniq slots, ready: ready, listener: &bound);
  }
  return exit_status(code: 0_u8);
}
```

A deep helper has the two routes the brief fixes. Either it does the I/O in
place, which blocks the whole program and is therefore only right when nothing
else is outstanding:

```whitefoot
fn read_header_blocking(conn: &uniq ConnHandle, into: &uniq MutSlice<u8>)
  -> result: own Completion reads(conn, into), writes(conn, into) {
  let p = op_start_read(conn: &uniq deref(conn), into: &uniq deref(into));  // [new]
  return op_finish(p: move p);                                              // [new] blocks
}
```

or it returns the operation it wants as data, and the outer loop issues it:

```whitefoot
fn want_next(row: &ConnRow) -> result: own OpRequest reads(row) {
  doc "No I/O here. The caller's reissue pass turns this into an op_start.";
  match row.phase {
    Reading() => { return ReadInto(length: 4096_u64); }
    Writing() => { return WriteFrom(start: row.sent, end: row.filled); }
    Closing() => { return CloseIt(); }
    Free()    => { return AcceptOne(); }
  }
}
```

### (c) What changes structurally

- **The handler is no longer written blocking.** This is the honest cost of
  setting the staged loop aside, and it is the one that matters most to the
  language's pitch. `handle_conn`'s position in its own call stack is gone, so
  the per-connection state machine that a thread pool never has to write —
  `phase`, `filled`, `sent` — is back, written out by hand. Whitefoot removes the
  *callback*, not the state machine: control returns to one legible loop instead
  of to a dispatcher, and each transition is an ordinary function on a row, but
  the writer still names the phases.
- **A deep call chain cannot await.** A helper five frames down either blocks the
  whole program or returns its wanted operation as data. Real programs will do
  the second, which means the operation request type (`OpRequest` above) becomes
  part of every interface that might do I/O — a colouring, of a different kind
  from `async fn`, and one the type system makes visible rather than viral.
- **Per-connection state and its buffer are rows.** `rows[k]` is an affine
  element write; the buffer window is an affine range (`[new]`, §0.1 consequence
  3). Without the range extension the handler loop cannot hand each iteration its
  own window from one pooled buffer, and the writer falls back to a
  `buffer<buffer<u8>>` or to PAR-1 with W in the source — so **architecture 17
  is one of the places the range-loan question is load-bearing, not cosmetic.**
- **Shared per-request state is a phase, not a lock.** A hit counter, an access
  log, a session table: each handler writes its own row, and one sequential pass
  after the loop merges. That is T2, priced there. Note what is *gone* relative
  to the staged-loop design: there is no index-ordered retirement constraint, so
  connection 500's shared write does not wait for connections 0–499. The batch
  model's serialization is one merge pass per round, which is simpler and which
  the writer can see.
- **The loop is unbounded and needs no trip count.** `loop @server` with
  `wait_batch` is the natural shape; nothing forces a fixed connection count.

Proof obligations: `ready <= len_of(rows)` and the window arithmetic
(`k*4096 + 4096 <= len_of(bufs)`) as header invariants or `requires` clauses;
that each `Pending` is either owned by its slot or consumed exactly once, which
is ordinary affine accounting and is the part the type system does for free.

### (d) Performance expectation

**No measurement in this repository covers this model.** The 0.58–0.74 figures
in §0.3 were produced by the **retired staged runtime** with one parked stack per
connection, and are not this design's expectation; they are evidence about that
runtime.

The estimate anchors on the bundle's own batched io_uring reference, because the
Whitefoot program becomes *the same loop shape*: submit a batch, wait, handle
completions, resubmit. That reference does **315k round trips/s at 64
connections and 349k at 1,024** on the 4-core host. Whitefoot's version of that
loop differs by exactly four things, and only four:

1. **Bounds checks erased by proof, not by branch.** Every buffer access in
   `advance` has its obligation discharged statically, so the emitted inner loop
   has no check the C reference does not also have. This is a wash at worst and a
   small gain where the C reference bounds-checks defensively. **Estimate: 0 to
   −2% cost, i.e. neutral to slightly favourable.**
2. **The sequential reissue pass.** The reference submits from inside its
   completion handler; Whitefoot walks the advanced rows once more. That is one
   extra pass over `ready` rows, touching a row and a slot each — call it 10–20 ns
   per connection against a round trip that the reference does in ~3.2 µs of
   wall time per connection at 64 connections (315k rt/s over 64 conns). **Estimate:
   +0.3 to +0.6%.**
3. **The batch-wait syscall.** One `wait_batch` per round rather than one
   `io_uring_enter` per round — the same call, at the same place in the loop. The
   io-model record measured the cost of getting this wrong: reaping *one*
   completion per progress pass instead of 64 was the serial resource that held
   the old design at 35k rt/s, and raising the reap budget to 64 tripled it. So
   the requirement is a batch reap, which is what `wait_batch` is. **Estimate:
   neutral, conditional on the implementation reaping the whole ready set.**
4. **The [PAR-2] handler loop is a fork-join per round.** The reference's handler
   loop is sequential on one thread; Whitefoot's can use W lanes. For a 64-byte
   echo the body is ~100 ns and the round is ~64 × 100 ns = 6.4 µs of work, so a
   *hot* round (spin rounds at 11.6 ns — a saturated server never lets its lanes
   park) is free and the parallelism is a gain. For an **idle** server the lanes
   park and the next round pays the Linux **10–16 µs** park-and-wake, which
   against a 6.4 µs round is a **150–250% overhead**. A lightly loaded server
   should therefore run the handler loop sequentially, and nothing in the source
   says which it is. **This is the one place the corrected park figure changes an
   I/O verdict.**

Net throughput estimate: **0.95–1.0 of the batched io_uring reference on a
saturated server** (≈300–349k rt/s at 64–1,024 connections on a 4-core host),
and **materially worse on a lightly loaded one** unless the handler loop stays
sequential there.

Latency: **one batch round.** An operation that completes just after its round's
`wait_batch` returns waits for the next round. At 64 connections and a 6.4 µs
round that is a p50 addition of ~3 µs and a p99 of ~6 µs — small. At 1,024
connections and a ~100 µs round it is ~50 µs p50 — which is the same order as the
reference's own 2,408 µs p50 at that fan-out, so the batch quantum is not the
dominant term there. Compared with a thread pool, latency is *better*: no
`pthread_create`, no kernel context switch per blocking read (~1–3 µs each).

Memory: **a row plus a buffer per connection**, which is the same order as
epoll's per-connection state and is where the batch model decisively beats the
retired stackful one. At a 64-byte `ConnRow` and a 4 KiB buffer that is ~4.1 KiB
per connection; buffers can be pooled and handed out only to active rows, in
which case an idle connection costs ~64 bytes. **The earlier draft's "stack-sized
per-connection memory" no longer applies.**

**Strictly worse case:** a lightly loaded server whose handler loop is
parallelized — 150–250% overhead per round from the park-and-wake — and any
program whose handlers genuinely need a deep call chain to suspend, which must
be rewritten as returned operation requests.

### (e) Verdict

**Restructure, bounded loss.** The loss is in *shape*, not in performance: the
per-connection state machine comes back as a row, and a deep helper must return
its wanted operation as data instead of awaiting. Throughput is estimated at
0.95–1.0 of a batched io_uring reference on a saturated server; latency is one
batch round; memory is a row plus a buffer, comparable to epoll. Marked `[R]`:
the handler loop needs the affine-range extension to hand each iteration its own
buffer window from one pool.

---

## 18. Event loop / reactor: many connections, callbacks, mostly waiting

libuv, a raw epoll loop, node.js.

### (a) Native shape

```c
for (;;) {
    int n = epoll_wait(ep, evs, MAXEV, timeout_ms());
    for (int i = 0; i < n; ++i) {
        conn_t *c = evs[i].data.ptr;
        switch (c->state) {                       // hand-written state machine
        case READING_HEADER: on_header(c); break;
        case READING_BODY:   on_body(c);   break;
        case WRITING:        on_write(c);  break;
        }
    }
    run_expired_timers();
}
```

### (b) Whitefoot shape

**This is the architecture the batch model matches most exactly.** Line for line:
`epoll_wait` is `wait_batch`, the dispatch `switch` is a `match` on the row's
phase, the callback is an ordinary function returning the new row, and the
reactor's implicit "register interest again" is the explicit reissue pass.

```whitefoot
loop @reactor {
  let ready = wait_batch(slots: &uniq slots);                // [new] ~ epoll_wait
  if ready == 0_u64 { break @reactor; }
  for (k in 0_u64..ready) {                                   // ~ the dispatch switch,
    let win = mut_slice_of(&uniq bufs, k *wrap 4096_u64,      //   but parallel and proved
                           k *wrap 4096_u64 +wrap 4096_u64);  //   [new] affine range loan
    set deref(rows)[k] = on_event(row: deref(rows)[k], buf: &uniq win,
                                  done: take_completion(slots: &uniq slots, at: k));
  }
  set live = reissue(rows: &uniq rows, bufs: &uniq bufs,
                     slots: &uniq slots, ready: ready, listener: &bound);
}
```

and `on_event` is the `switch`, written as a total function:

```whitefoot
fn on_event(row: own ConnRow, buf: &uniq MutSlice<u8>, done: own Completion)
  -> result: own ConnRow reads(row, buf, done), writes(buf) {
  match row.phase {
    ReadingHeader() => { return after_header(row: move row, buf: buf, done: move done); }
    ReadingBody()   => { return after_body(row: move row, buf: buf, done: move done); }
    Writing()       => { return after_write(row: move row, buf: buf, done: move done); }
    Closing()       => { return ConnRow(phase: Free(), /* … */); }
  }
}
```

Timers: `run_expired_timers()` needs a timer that `wait_batch` can return, i.e. a
`Pending` whose completion is "the deadline passed". **If the I/O API does not
supply one, per-connection timeouts are not expressible** — there is no agent to
run them and no deadline to wait on. With one, a timer is just another slot and
the reactor's timer heap becomes an ordinary sorted row array plus one timer
pending for the earliest deadline.

### (c) What changes structurally

- **Callbacks are replaced by a dispatch `match` on a row, not deleted.** The
  gain over libuv is real but narrower than the staged design promised: control
  flow is one legible loop rather than a callback graph, every transition is a
  total function whose effects are declared, and there is no inversion of control
  — but the state machine itself is still written by hand.
- **The reactor's "re-arm interest" becomes an explicit reissue pass**, which is
  an improvement in legibility: the thing libuv does implicitly (`uv_read_start`
  keeps re-arming) is a statement the writer can see and reason about.
- **The dispatch loop is parallel, which libuv's is not.** A reactor handles its
  ready set on one thread; a [PAR-2] loop over the same set can use W lanes,
  because each iteration writes only its own row and its own buffer window. For
  CPU-bearing handlers (parsing, compression, TLS record processing) this is a
  structural advantage over a single-threaded reactor and is the main reason to
  prefer this shape to node.js's.
- **Timers must become pendings.** See above; unchanged from the earlier draft
  and still the top item for the API.

### (d) Performance expectation

Throughput: the §17 estimate applies unchanged — **0.95–1.0 of a batched
io_uring reference on a saturated server** (315k rt/s at 64 connections, 349k at
1,024 on the 4-core host), the four deltas being erased bounds checks (neutral to
−2%), the reissue pass (+0.3–0.6%), the batch-wait syscall (neutral if the whole
ready set is reaped), and the parallel handler loop (a gain when saturated, a
150–250% per-round overhead when lanes park on a lightly loaded server).

Against **libuv** specifically rather than raw io_uring, the comparison should be
better than that: libuv adds a callback dispatch, a handle/request object per
operation and a `uv_` layer above epoll, and Whitefoot adds a row write and a
reissue pass. Estimate: **parity to 1.15x libuv's throughput on a saturated
server**, with the parallel dispatch loop the reason for the upside.

Latency: one batch round, as in §17 — ~3 µs p50 added at 64 connections, ~50 µs
at 1,024. A reactor has the same quantum (`epoll_wait` returns a set and the
loop walks it), so this is not a Whitefoot-specific cost.

Memory: **a row plus a buffer per connection.** Against libuv's ~200–500 bytes of
state per idle connection, a Whitefoot row is the same order and the buffer is
the difference — and buffers can be pooled and held only by active rows, so an
idle connection costs a row. **This reverses the earlier draft's finding: the
batch model does not pay a stack per idle connection, so the 30x memory ratio
against libuv is gone and the C10K/C1M ceiling with it.** What remains is
ordinary buffer accounting, the same problem every reactor has.

**Strictly worse case:** a lightly loaded reactor with the handler loop
parallelized (park-and-wake per round), and any reactor that needs a timer the
API does not supply.

### (e) Verdict

**Restructure, no loss.** The batch model is the reactor, with the callback graph
replaced by a dispatch `match` and the ready-set walk replaced by a parallel
loop. Throughput is estimated at parity to 1.15x libuv and 0.95–1.0 of raw
batched io_uring; memory is a row plus a pooled buffer, comparable to epoll.
Conditional on the API supplying a timer pending — without one, per-connection
timeouts are **not expressible**. Marked `[R]` for the per-iteration buffer
window.

---

## 19. Async/await task runtime: thousands of tasks, deep awaits, timers, cancellation

tokio, C++20 coroutines.

### (a) Native shape

```rust
let mut set = JoinSet::new();
for job in jobs {
    set.spawn(async move {
        let conn = pool.get().await;                       // deep await
        let rows = timeout(Duration::from_millis(50),
                           conn.query(&job.sql)).await??;  // timer + cancellation
        cache.insert(job.key, rows.len()).await;           // shared, concurrent
        Ok::<_, Error>(rows.len())
    });
}
while let Some(r) = set.join_next().await { tally(r?); }
```

### (b) Whitefoot shape

Four features, and the batch model answers them differently from the staged one.

**Thousands of homogeneous tasks: expressible, and it is §17's loop with a
different row type.** `JoinSet` is the row array; `join_next` is `wait_batch`.

```whitefoot
// Restructuring note: task -> row; await point -> phase; JoinSet -> the arrays;
// join_next -> wait_batch. The task's own control flow is written as phases.
struct JobRow {
  phase: JobPhase;      // Getting | Querying | Caching | Done | Failed
  job: Job;
  rows: u64;
  deadline: u64;        // absolute, checked against the timer pending
}

loop @tasks {
  let ready = wait_batch(slots: &uniq slots);                 // [new]
  if ready == 0_u64 { break @tasks; }
  for (k in 0_u64..ready) {                                    // [PAR-2] over the batch
    let win = mut_slice_of(&uniq bufs, k *wrap 8192_u64,       // [new] affine range loan
                           k *wrap 8192_u64 +wrap 8192_u64);
    set deref(state)[k] = advance_job(row: deref(state)[k], buf: &uniq win,
                                      done: take_completion(slots: &uniq slots, at: k));
  }
  set live = reissue_jobs(state: &uniq state, slots: &uniq slots, ready: ready);
  // the tally is a sequential merge pass, not an accumulator in the [PAR-2] loop
  set tally = merge_finished(state: &uniq state, ready: ready, tally: tally);
}
```

**Deep awaits: this is the real loss, and it is worse here than in §17.** A tokio
task awaits five frames down inside `conn.query`. Under the batch model that
helper must either block the whole program — unacceptable with a thousand tasks
outstanding — or return its wanted operation as data. So **every function on a
path that may await becomes a function returning an `OpRequest`**, and the task's
logical call stack is flattened into the row's `phase` enum. A three-await task
is three phases; a task that awaits inside a loop inside a helper is a phase plus
a counter plus a sub-phase. **This is the single largest source-level cost of the
batch model, and it grows with await depth, not with task count.**

**Timers: expressible if and only if the API supplies a timer pending**, as in
§18. `deadline` in the row plus one timer slot for the earliest deadline is the
standard implementation, and it works.

**Cancellation: expressible only as a cancel operation on a `Pending`.** If the
API provides `op_cancel(&uniq p)` — request that an outstanding operation
complete early with a cancelled outcome — then per-task cancellation *is*
reachable: the reissue pass cancels the slot whose row's deadline has passed and
the next round sees a cancelled completion. **This is a genuine improvement on
the staged design, where the only operation was "drain everything and stop".**
What stays out of reach is cancelling work that is not represented as a
`Pending`: a task blocked in pure computation, a task between phases, or a
cancellation that must take effect before the next round.

**Heterogeneous dynamic spawn: still not expressible.** `set.spawn(…)` from
arbitrary depth with tasks of different shapes has no analogue. The row type must
be sized for the worst case across a fixed set of task *kinds*, and a new kind is
a new arm and a new row field, not a new closure. A program that spawns a
variable number of variously-shaped tasks from inside a task must restructure to
a fixed kind enum.

One thing the earlier draft claimed here is **withdrawn**: the staged loop's
index-ordered retirement admitted non-associative accumulators, so `set tally =
tally +wrap r.rows` needed no associativity argument. Under the batch model the
handler loop is an ordinary [PAR-2] loop, so that latitude is gone — a float or
`Result`-routed tally must be a sequential merge pass after the loop, exactly as
in T2. The asymmetry between the I/O loop and the compute loop disappears with
[PAR-3].

### (c) What changes structurally

- **Tasks are rows and the runtime is the loop** — legible, allocation-free, and
  with no scheduler to tune.
- **Await depth becomes phase count, by hand.** The colouring is real, it is
  visible in the types (`-> OpRequest`), and it is the price of one control flow.
- **Results are rows sized for the worst case across kinds.**
- **`cache.insert(...).await` — shared state from many tasks — is architecture 7
  again**, and becomes a per-row entry plus a merge pass.
- **A slow task holds its slot but not a stack**, so the cost of a stuck
  operation is a row and a buffer, not a suspended call stack.

### (d) Performance expectation

Throughput: the §17 estimate, since it is the same loop —
**0.95–1.0 of a batched io_uring reference on a saturated runtime**. Against
tokio specifically the comparison should be favourable: tokio pays a boxed
future, a waker, an `Arc` and a scheduler enqueue per task wake, and Whitefoot
pays a row write plus a slot reissue. Estimate: **1.0–1.2x tokio's throughput**,
with the parallel handler loop as the upside and no measurement to support it.

Latency: one batch round, plus whatever the phase decomposition adds by forcing a
task through the loop once per await rather than resuming it in place. A task
with three awaits visits the loop three times; tokio resumes it three times too,
so this is a wash except for the round quantum.

Memory: **a row plus a (poolable) buffer per task**, against tokio's ~100–300
bytes per future plus its buffers. Same order. The stack-per-task ceiling of the
retired design is gone.

**Strictly worse case:** deep await chains. A task with eight await points in
nested helpers becomes an eight-phase state machine threaded through the row, and
the source cost is large even though the runtime cost is not. Second: a workload
needing cancellation of something that is not an outstanding `Pending`.

### (e) Verdict

**Restructure, bounded loss** for a homogeneous task set: tasks are rows, the
loop is the runtime, throughput is estimated at 0.95–1.0 of batched io_uring and
1.0–1.2x tokio, memory is a row plus a pooled buffer. The bound is **await depth
→ hand-written phases**, which is the largest source-level cost in the I/O half.
Per-task cancellation is **expressible via a cancel operation on a `Pending`**
and not otherwise. Timers depend on the API supplying a timer pending.
**Not expressible:** dynamic heterogeneous spawn, and cancellation of work not
represented as a `Pending`. Marked `[R]` for the per-iteration buffer window.

---

## 20. Responsive loop plus a long computation

A service that must keep answering while a 10-second compute runs.

### (a) Native shape

```c
pthread_create(&t, NULL, long_compute, &job);     // 10 s, CPU-bound
for (;;) {                                        // main thread stays responsive
    int c = accept(...);
    handle(c);                                    // p99 unaffected by the compute
}
// the OS preempts; neither side thinks about the other
```

### (b) Whitefoot shape

There is no preemption and no second control flow. The computation must be
chopped by the writer and resumable as data, and the service must be the batch
loop of §17. The two are interleaved in one outer loop.

```whitefoot
// Restructuring note: the background thread becomes one more phase of the
// service loop, and the computation must be rewritten as an explicit resumable
// state machine — a work stack in a buffer, not a call stack.
struct Compute {
  stack: buffer<Frame>;        // the recursion, as data
  depth: u64;
  done: Bool;
  accumulated: f64;
}

loop @serve {
  // 1. non-blocking peek at the batch, so the compute is not starved by a
  //    blocking wait_batch when nothing is ready.
  let ready = wait_batch_deadline(slots: &uniq slots, by: now_ms() +wrap 1_u64); // [new]

  // 2. the service handlers, exactly §17's [PAR-2] batch loop
  for (k in 0_u64..ready) {
    let win = mut_slice_of(&uniq bufs, k *wrap 4096_u64,          // [new] range loan
                           k *wrap 4096_u64 +wrap 4096_u64);
    set deref(rows)[k] = advance(row: deref(rows)[k], buf: &uniq win,
                                 done: take_completion(slots: &uniq slots, at: k));
  }
  set live = reissue(rows: &uniq rows, bufs: &uniq bufs, slots: &uniq slots,
                     ready: ready, listener: &bound);

  // 3. one bounded slice of the computation. `budget` is the p99 knob.
  let stepped = compute_chunk(job: &uniq job, budget: 200000_u64);
  if deref(job).done { break @serve; }
}

fn compute_chunk(job: &uniq Compute, budget: own u64) -> result: own u64
  reads(job), writes(job) contract {
  requires budget <= 1000000_u64;
} {
  doc "Advances the explicit work stack by at most `budget` steps and returns.
       The recursion is in `job.stack` rather than in this function's frames,
       because nothing can interrupt a Whitefoot call.";
  /* pop, expand, push, accumulate, at most `budget` times */
}
```

The service and the compute can also be one [PAR-1] window per round — they are
footprint-disjoint, `compute_chunk` touching only `job` — which lets the compute
chunk run on a lane while the handler batch runs on others. Both still finish
before the round ends.

### (c) What changes structurally

**Part 1: the computation must be written in resumable form.** Unchanged from the
earlier draft and unaffected by the I/O model. Nothing interrupts a Whitefoot
call, so a computation that must yield holds its own continuation as data: an
explicit work stack, an iteration counter, a phase enum. For a loop over an array
this is trivial; for a recursive solver, tree search or constraint propagator it
means hand-writing the stack. Note the irony: that recursion was expressible and
parallel under PAR-1 (architecture 2, now at parity); making it *interruptible*
is what forces the explicit stack.

**Part 2: the chunk budget is the response-latency knob, in the source.** Worst
case response latency is one chunk plus one round. The writer picks `budget`, and
can adapt it by reading the clock each round — a feedback loop the writer writes,
which works, and which an OS scheduler does for free.

**Part 3 — and this is where the batch model is much better than the staged
one.** Under the staged design the service side was a loop over connections and
could not be composed with a chunked compute in one construct, so request
throughput was capped at **one request per chunk**. Under the batch model the
service side is `wait_batch` + a [PAR-2] handler loop, and *a whole batch of
requests is served per round*. The cap becomes **one batch per chunk**, which at
64 ready connections per round is 64x better and is no longer a structural
disqualifier. The remaining requirement is a `wait_batch` that can return
without blocking indefinitely — a deadline or poll form — otherwise a quiet
moment blocks the loop and the computation stops. **That is the third item for
the I/O API.**

### (d) Performance expectation

Compute throughput: nearly unchanged by the chunking, which costs one round per
chunk; the rounds are back to back so the lanes stay hot and a round is spin
rounds at 11.6 ns. What does cost is the resumable form: a hand-written work
stack typically runs 10–30% slower than native recursion because the frame cannot
stay in registers. **Estimate: 1.1–1.3x the runtime of the native computation**,
from the rewriting, not from the chunking.

Response latency: native with OS preemption is unaffected by the background
compute (~50–200 µs p99). Whitefoot's p99 is one chunk plus one round. At a 1 ms
chunk that is ~1 ms — **5–20x worse p99**. Buying it back by shrinking the chunk
is where the corrected park figure bites: **if the round is hot** (a saturated
server, lanes inside the 1,000 µs idle window) a 100 µs chunk brings p99 to
~150 µs at negligible overhead; **if the loop is quiet** and the lanes park, each
round costs the Linux **10–16 µs** park-and-wake and a 100 µs chunk carries
**10–16% overhead** rather than the ~2% an earlier draft estimated. The honest
statement: *the recovery is cheap exactly when the server is busy, and expensive
exactly when it is idle* — which is tolerable, since an idle server has latency
headroom, but it is the opposite of a scheduler's behaviour and worth knowing.

Request throughput: **one batch per chunk**, not one request per chunk. At 64
ready connections and a 1 ms chunk that is ~64,000 requests/s, against the native
design's unbounded rate. For most services this is no longer the binding
constraint.

Memory: the explicit work stack, sized for the worst-case depth, allocated up
front — a real allocation the native recursion got from the thread stack for
free.

**Strictly worse case:** a service with a hard sub-100 µs p99 next to a
long computation, on a lightly loaded box. The chunk must be small, the lanes
park between rounds, and the park-and-wake becomes a double-digit percentage of
the loop.

### (e) Verdict

**Restructure, bounded loss.** The dominant cost is unchanged and is the largest
source-level tax in the catalog: the computation must be hand-written in
resumable form (an explicit work stack, 1.1–1.3x slower). Response p99 becomes a
writer-chosen chunk budget, recoverable to near-parity on a busy server and at
10–16% overhead on a quiet one. **The throughput cap improves from one request
per chunk to one batch per chunk under the batch I/O model**, which removes the
earlier draft's disqualifying finding. Conditional on `wait_batch` having a
deadline or poll form; without one, a quiet moment stops the computation.

---


## 21. Parallel search with early exit (added)

`std::find_if(par_unseq, …)`, Rayon's `par_iter().find_any()`, a parallel
constraint check that stops at the first violation. Added because it is common,
because it is not covered by architecture 1, and because the rule that refuses it
is one line.

### (a) Native shape

```cpp
auto it = std::find_if(std::execution::par_unseq, v.begin(), v.end(), bad);
// or
let hit = items.par_iter().position_any(|x| x.is_bad());
// workers poll a shared atomic flag and abandon their chunk when it is set
```

### (b) Whitefoot shape

The obvious form is **denied**, because PAR-2's last condition refuses any body
statement whose normal continuation does not reach the binder update:

```whitefoot
for (i in 0_u64..n) {
  if bad(x: deref(v)[i]) { return i; }      // return_stmt in B: DENIED
}
```

The expressible parallel form drops the early exit and uses `imin`, which *is*
an admitted accumulator:

```whitefoot
// Restructuring note: early exit is replaced by a full scan with an imin fold.
// The loop always does all the work; "not found" is encoded as the type max.
fn first_bad(v: &buffer<Item>, n: own u64) -> result: own u64 reads(v) contract {
  requires n <= len_of(deref(v));
} {
  let best = 18446744073709551615_u64;             // u64 max = imin identity
  for (i in 0_u64..n) {
    let candidate = if bad(x: deref(v)[i]) { give i; }
                    else { give 18446744073709551615_u64; };
    set best = imin(best, candidate);              // one accumulator, admitted
  }
  return best;
}
```

### (c) What changes structurally

- **The search always scans everything.** There is no way to tell the other lanes
  to stop, because there is nothing to tell and no shared flag to set.
- **`bad(x)` must be total.** If the predicate can fail (a `Result`), the
  `propagate_let_rhs` form denies the loop, so failure must be encoded into the
  accumulator's value domain too — e.g. `ior` a flag alongside, which is a second
  accumulator, which is denied. So a fallible predicate forces a sequential loop
  or a two-pass structure.
- **The answer is now the first index, deterministically**, where `find_any` and
  `par_unseq` return an arbitrary match. That is a correctness *gain* — the
  result no longer depends on scheduling — and it is why `imin` is the right fold
  rather than "any".

### (d) Performance expectation

When the match is near the end or absent: parity with the native parallel scan,
at the measured map ratio (1.03–1.14).

When the match is early: **arbitrarily worse.** A native parallel `find_if` on a
100M-element array with a match at element 1,000 does roughly (chunk size ×
workers) element evaluations before all workers see the flag — call it a few
thousand. Whitefoot does 100,000,000. That is a **10^4–10^5x work amplification**
on the good case, and it is the case the native algorithm is chosen for.

Against a *sequential* early-exit scan, Whitefoot's parallel full scan is also
worse whenever the match is in the first `1/W` of the array — which for a
uniformly distributed match is 25% of the time on 4 cores.

Latency: proportional to `n`, always. Memory: unchanged.

**Strictly worse case:** validation passes that almost always find their answer
immediately — "is any record malformed", "does this input contain a byte > 127".
The sequential early-exit loop is the right program, and the writer should write
that, which means noticing that the parallel form is a pessimization. There is no
diagnostic for this; the ledger says `PAR permitted`, and permitted is not
profitable.

### (e) Verdict

**Restructure, bounded loss — and the bound is the match position.** Early exit
is not expressible in a parallel loop; the `imin` substitute always does all the
work, which is parity when the answer is late and catastrophic when it is early.
The determinism is a genuine gain.

---

## 22. Summary table

Confidence: **M** = grounded in a measurement in this repository; **R** =
grounded in a specification rule read directly; **E** = estimate with stated
reasoning and no measurement.

**`[R]`** marks a verdict that **depends on the range-loan extension** of §0.1
consequence 3 — [PAR-2] admitting an iteration-exclusive loan on an affine range
`[c*i + b, c*(i+1) + b)`. The verdict shown is the one *with* the extension; the
column "without `[R]`" gives the verdict if [PAR-2] keeps handing out elements
only.

| # | Architecture | Verdict | Without `[R]` | Structural cost | Expected perf vs native | Conf |
|---|---|---|---|---|---|---|
| 1 | Data-parallel map / reduce | direct (map); bounded loss (reduce) | — | one accumulator; no float folds; no early exit | 0.87–1.14 of best ref (measured); float reductions sequential → 3–4x loss on that phase | M |
| 2 | Recursive fork-join | **restructure, no loss** at W≤4 | — | a `requires depth <= K` ceiling that propagates; no writer grain knob | **0.969–1.029** on hosted ubuntu (runs 34667725821 / 34668156035 / 34668796717, fastest in two blocks), **1.000–1.073** on the M1 Pro; W=1 tax 1.4%. Bounded loss only at W≥CPU count on an asymmetric host: **1.352 at W=8**, 2.2–2.8 at W=16 | M |
| 3 | Pipeline with stages + queues, stateful stage | restructure, bounded loss | — | queues → N+1 batch buffers; stage count written in source; per-item → per-batch latency | throughput within 10–20% when stages regular (hot rounds at 11.6 ns); **1.3–2x worse** with a high-variance stage; latency = batch size | E |
| 4 | Producer–consumer, bounded queue | restructure, bounded loss | — | backpressure → batch size constant; no continuous rate adaptation | throughput ±10% or better (mutex per item deleted); latency +1 batch; **1.5–2x worse** under bursty arrivals | E |
| 5 | Task DAG, static | direct | — | precise effect rows become a perf property | parity; 2.98x/4 cores measured on the sibling-pair path | M |
| 5′ | Task DAG, dynamic | restructure, bounded loss | — | level barriers; sequential frontier compaction; rows not objects | within ~15% for wide shallow DAGs; **2–5x** makespan for deep skewed ones | E |
| 6 | Game job system | restructure, bounded loss | — | one extra phase per spawn depth; sequential merge per phase; no grain control | ~1% of a 60 Hz frame for merges (hot rounds); **1.2–1.5x worse** on fine-grained frames (grain hazard measured at 1.40x on `wfgrep`) | M/E |
| 7 | Concurrent hash map | **not expressible** (concurrent insert+lookup); **`[R]` restructure, no loss** (build-then-freeze) | restructure, bounded loss — W a source constant | no cross-shard probing; insert latency = batch period | build 0.6–0.9 of native; **frozen lookup 1.2–1.5x faster** | R/E |
| 8 | Read-mostly RwLock / RCU | **restructure, no loss** (structural win) | — | publish latency bounded by read-phase length | reader path **1.5–3x faster** than RwLock, parity with RCU; memory strictly lower | R/E |
| 9 | Lock-free SPSC/MPSC ring | **not expressible** | — | no "publish and keep going" construct exists | throughput parity or better; **handoff latency 100–750x worse** — floor ~10–30 µs against 40–100 ns, because an intermittently-fed loop pays the measured **10–16 µs Linux park-and-wake** per round | R/E |
| 10 | Actor model | restructure, bounded loss (large) | — | sequential delivery scatter per round; compile-time fan-out cap; chain latency in rounds | ~1.4x worse at W=4, ~2.6x at W=16 (Amdahl on the scatter); chain latency **100–1,000x**; sparse actor sets **1,000x+** work amplification | E |
| 11 | Per-core shared-nothing | restructure, bounded loss | — | round barrier where native has none; cross-core msgs → per-round scatter | **<1% barrier while saturated** (hot rounds); **300–500% per round at burst=32 once lanes park**; latency ~10–25x DPDK at burst=256 | R/E |
| 12 | Structured concurrency / scoped threads | **direct** | — | none | parity; 2.98x on 4 cores measured (75% of ideal); faster than `thread::scope` for short tasks | M |
| 13 | BFS, low-diameter | restructure, bounded loss | — | sparse frontier → dense bitmaps; no atomic parent claim | ~0.9x of an OpenMP direction-optimizing BFS | E |
| 13′ | BFS, high-diameter | restructure, **asymptotic** loss | — | dense form is the wrong algorithm | **O(V·D) vs O(V+E)** — 1,000x on a 1M-vertex, diameter-1,000 graph | R |
| 13″ | Histogram | **`[R]` restructure, no loss** | restructure, bounded loss — W a source constant | W copies of the counters; one merge pass | ~0.9–1.0 of the OpenMP privatized form, and now on a lane count that matches the host | R/E |
| 13‴ | Counting / radix sort | restructure, bounded loss | — (verdict set by the scatter, not by W) | the scatter stays sequential under every rule | **~0.5** of native (Amdahl on the scatter) | R/E |
| 14 | Stencil, 1-D | **direct** | — | none; halo exchange deleted | 0.91–1.14 of best ref (`fir` measured fastest) | M |
| 14′ | Stencil, 2-D | **`[R]` restructure, no loss** — an ordinary `for` over rows, since a row is the affine range `[y*w, y*w+w)` | restructure, bounded loss — hand-written W-way bands, W a source constant | requires the range rule's `c` to be a loop-invariant value, not a literal; the inner `next[y*w+x]` element form stays denied either way | within 5–10%; **<1% barrier** for back-to-back steps, **250–400%** if steps are further apart than the idle window | R/E |
| 15 | Prefix sum | **`[R]` restructure, no loss** | restructure, bounded loss — W a source constant for the writeback | three passes | ~0.95 of native | E |
| 15′ | Comparison sort | restructure, bounded loss | — (verdict set by the scatter and the merge) | no parallel partition, no parallel merge; out-of-place only | ~0.7–0.9 of a good parallel sort at 4 cores | E |
| 16 | Background periodic thread | restructure, no loss (flush/metrics); bounded loss (hard deadlines); **not expressible** (watchdog) | — | every duty named in the main loop; jitter = longest phase | appends faster (lock deleted); timeliness worse by the phase length | R/E |
| 17 | Thread-pool server | **`[R]` restructure, bounded loss** — the loss is shape, not speed | same verdict; per-iteration buffer windows need a `buffer<buffer<u8>>` or PAR-1 with W in source | the per-connection state machine returns as a row `phase`; a deep helper returns its wanted op as data | **estimate 0.95–1.0 of the batched io_uring reference** (315k rt/s at 64 conns, 349k at 1024, 4-core host): erased bounds checks 0 to −2%, reissue pass +0.3–0.6%, batch-wait syscall neutral if the whole ready set is reaped, parallel handler loop a gain when saturated and **+150–250% per round when lanes park**. Latency = one batch round. Memory = **a row plus a poolable buffer** | E |
| 18 | Event loop / reactor | **`[R]` restructure, no loss** — the batch model *is* the reactor | same verdict, same caveat as 17 | callback graph → a dispatch `match` on a row; the state machine is still written by hand; timers need an API timer pending (**not expressible** without one) | **parity to 1.15x libuv**, 0.95–1.0 of raw batched io_uring; the dispatch loop is parallel where a reactor's is not. **Memory a row plus a pooled buffer — the earlier "30x worse per idle connection" is withdrawn with the stackful design** | E |
| 19 | Async/await runtime | **`[R]` restructure, bounded loss**; **not expressible** (dynamic heterogeneous spawn; cancelling work not represented as a `Pending`) | same verdict, same caveat as 17 | **await depth → hand-written phases** — the largest source cost in the I/O half; per-task cancellation *is* reachable via a cancel operation on a `Pending` | estimate 0.95–1.0 of batched io_uring, **1.0–1.2x tokio**; memory a row plus a pooled buffer. The staged design's non-associative-accumulator latitude is **withdrawn** | E |
| 20 | Responsive loop + long compute | restructure, bounded loss (largest source tax) | — | hand-written resumable work stack; chunk budget = p99 knob; needs `wait_batch` to have a deadline or poll form | compute 1.1–1.3x slower (resumable form); p99 recoverable to near-parity on a **busy** server and at **10–16% overhead** on a quiet one; **throughput cap improves from one request per chunk to one batch per chunk** | R/E |
| 21 | Parallel search with early exit | restructure, bounded loss | — | no early exit; full scan with `imin` | parity when the answer is late; **10^4–10^5x** work amplification when early; determinism gained | R |

---

## 23. The recurring transformations, and what each costs inherently

Eleven transformations account for every restructuring above. Each is listed
with the cost that is *inherent* to it — not an implementation cost that could be
optimized away.

**T1. Queue → double-buffered batch.** (3, 4, 9, 16)
A channel between two agents becomes N+1 buffers rotated in a PAR-1 window.
*Inherent cost:* per-item latency becomes per-batch latency, and the round costs
`max(stage)` where the queue amortized `mean(stage)`. Jitter absorption goes to
zero. The batch size is a source constant that trades latency against barrier
overhead.

**T2. Shared mutable structure → per-lane private + deterministic merge.**
(6, 7, 10, 13, 17)
*Inherent cost:* W copies of the structure (memory), one extra merge pass over
W×|structure| (time), and — because PAR-2 cannot hand out per-lane windows —
**W becomes a literal in the source**. The merge pass is an Amdahl term that
grows relative to the parallel part as W rises.

**`[new]` — only the last third of that cost is inherent.** With an
iteration-exclusive loan on an affine range `[c*i + b, c*(i+1) + b)` admitted by
[PAR-2], the per-lane window comes from the loop index, `lanes` is a runtime
value, and the transformation is written as an ordinary counted loop:
`for (lane in 0..lanes) { let win = mut_slice_of(&uniq shared, lane*S, lane*S+S); … }`.
**W stops being a source constant.** What stays inherent is the rest: W copies
(memory) and the merge pass (an Amdahl term). So T2 survives the extension as a
real transformation with a real price; it simply stops also being a portability
defect. Verdict movements this causes are listed at architectures 7, 13″, 14′
and 15, and marked `[R]` in the summary table.

**T3. Lock → phase boundary.** (7, 8, 16)
A critical section becomes "these statements are in different phases".
*Inherent cost:* the writer must be able to partition the program's accesses into
phases, and a writer that must publish *during* a read phase cannot. Publish
latency becomes the phase length. Against this, the lock's own cost (15–40 ns
per acquisition, unbounded under contention) is deleted, which is often a net
gain.

**T4. Thread → phase in the outer loop.** (3, 10, 11, 16, 20)
A long-lived agent becomes a guarded block the main loop reaches every round.
*Inherent cost:* the agent runs only when the loop reaches it, so its timeliness
is bounded by the longest phase; and it cannot observe the main work stalling,
which is why a watchdog is not expressible.

**T5. Actor/connection/core state → a row in an array.** (6, 10, 11, 17, 19)
*Inherent cost:* work becomes proportional to the *number of rows* rather than
the number of *active* rows. A sparse active set costs a full pass to find, and
compacting it is a scatter, which is sequential. Row types must be sized for the
worst case across kinds.

**T6. Data-dependent write → sequential scatter pass.** (5′, 10, 11, 13, 15′)
Any `x[f(data)] = v` is denied by PAR-2's affine condition and by PAR-1's
unresolved-place rule. It becomes its own sequential pass.
*Inherent cost:* an Amdahl term whose size is the scatter's share of the
algorithm. For counting sort and radix sort that share is the majority, which is
why those verdicts are ~0.5.

**T7. Per-lane output range → a hand-written PAR-1 window with range loans.**
(7, 13″, 14′, 15, 15′)
*Inherent cost:* **W is a source constant.** A program written for 4 lanes gets
4 lanes on a 64-core machine. This is the single most consequential recurring
cost in the catalog, and it is a direct consequence of PAR-2's rule that an
exclusive loan on a place rooted outside the loop denies permission.

**`[new]` — this transformation is not inherent at all; it exists only because
of that one rule, and the accepted range loans plus an affine-range condition
delete it outright.** `[c*i + b, c*(i+1) + b)` is disjoint across distinct `i`
by the same linear arithmetic the element case already uses, so the window can
come from the loop index rather than from a hand-written window per lane. T7
would then have no occasions left: architectures 7, 13″, 14′ and 15 each move
from PAR-1-with-W-in-source to an ordinary [PAR-2] loop (see their `[new]`
notes), and 15′ (sort) keeps its verdict for other reasons — the sequential
scatter and the sequential merge. **Of the eleven transformations in this list,
T7 is the only one that is a defect rather than a price.**

**T8. Early exit → full scan with an admitted fold.** (1, 21)
*Inherent cost:* all the work, always. Parity when the answer is late; a 10^4x
amplification when it is early. Buys determinism.

**T9. Sparse frontier → dense pass.** (5′, 13)
*Inherent cost:* O(V) per level instead of O(frontier). Free on low-diameter
graphs, asymptotically wrong on high-diameter ones.

**T10. Callback state machine → a straight-line function on a parked stack.**
(17, 18, 19)
The one transformation that runs the other way: Whitefoot deletes structure
rather than adding it.
*Inherent cost:* memory per suspended unit becomes a stack rather than a state
record — a 30x+ ratio for idle connections, and the ceiling on concurrency.

**T11. Preemptible computation → hand-written resumable state machine.** (20)
*Inherent cost:* the recursion moves from the call stack into a buffer the writer
manages (1.1–1.3x slower, plus the worst-case-depth allocation), and response
latency becomes a chunk budget the writer chooses. There is no partial version of
this transformation: either the computation can be resumed at a writer-visible
point or it cannot be interrupted at all.

---

## 24. The four architectures where the model is weakest

Judged by: how far the Whitefoot program is from the native one, how common the
shape is in real systems code, and whether the loss is a constant factor or a
change of kind.

**The list changed in this revision.** Very high fan-out I/O (18, 19) leaves it:
its finding was a stack per idle connection, which belonged to the staged design
that is now set aside; under the batch model an idle connection is a row plus a
poolable buffer. **Deep await chains flattened into hand-written phases** takes
its place as §24.4 — the cost the stackful design was hiding. Architecture 2
(recursive fork-join) was never on this list and is now measured at parity, so
nothing moves on its account.

### 24.1 Shared mutable structure under concurrent read/write (architecture 7)

**Why it is the weakest.** It is not a performance loss; it is an
inexpressibility. [CAP-1] states the kernel defines no sharing classification
beyond `own`/`&`/`&uniq`, so there is no way to say "this structure synchronizes
internally". Every concurrent map, concurrent queue, concurrent set, refcounted
cache and interner in real systems code is this shape.

**What a real program looks like.** A DNS resolver's cache, which in Rust is one
`DashMap<Name, Answer>` touched by every request handler, becomes:

```whitefoot
// Phase 1: every in-flight request's lookup, against the frozen table.
for (i in 0_u64..in_flight) {
  set deref(hit)[i] = lookup(table: &table, key: deref(req)[i].name);
}
// Phase 2: the misses become upstream queries — a staged loop, K in flight.
for @resolve (i in 0_u64..in_flight) {
  if deref(hit)[i].missing {
    set deref(answer)[i] = query_upstream(name: deref(req)[i].name, /* … */);
  }
}
// Phase 3: sequential — fold this round's answers into the table.
let inserted = merge_answers(answer: &deref(answer), n: in_flight,
                             table: &uniq table);
```

Three phases per round instead of one call per request, and — the part that
changes observable behaviour — **two requests for the same name in one round both
miss and both query upstream**, because there is no shared structure for the
first to publish into before the second looks. The native cache collapses them.
The writer must either accept the duplicate work or add an intra-round
deduplication pass (a sort of the round's keys, which is architecture 15′).

**Is the loss acceptable?** For a batch or request-response system, yes, and the
determinism is worth something. For anything whose correctness or efficiency
rests on a shared structure being *current* — a routing table, a lock manager, a
JIT's code cache, an interner under concurrent compilation — this is a
disqualifying restriction, and no amount of proof work changes it, because the
restriction is the point of the model.

### 24.2 Actor systems and arbitrary message graphs (architecture 10)

**Why.** Two independent losses compound. The delivery scatter is sequential and
is an Amdahl term that grows with W (1.4x at W=4, 2.6x at W=16, worse beyond).
And BSP rounds turn hop latency into round latency, a 100–1,000x regression on
any request/response chain. Add the compile-time fan-out cap per actor, which is
an expressiveness limit rather than a cost, and the sparse-active-set problem
from T5.

**What a real program looks like.** A protocol state machine per session —
say 50,000 sessions in a signalling server, of which ~200 are active per
millisecond:

```whitefoot
for @rounds (r in 0_u64..rounds) {
  // The active set must be found. A dense pass over 50,000 rows to find 200,
  // or a compaction pass, which is a scatter and therefore sequential.
  for (s in 0_u64..sessions) {
    set deref(state)[s] = step(session: deref(state)[s],
                               inbox: &inbox, at: deref(inbox_at)[s]);
  }
  let moved = deliver(/* sequential scatter */);
  if moved == 0_u64 { break @rounds; }
}
```

50,000 rows scanned to advance 200, every round. At 20 ns per inactive row that
is 1 ms of pure overhead per round, against Akka's 200 scheduler dequeues at
~100 ns each = 20 µs. **A 50x work amplification on the dominant term**, before
the delivery scatter.

**Is the loss acceptable?** For a bulk-synchronous graph computation dressed up
as actors (PageRank, label propagation, belief propagation), yes — those are BSP
algorithms anyway and the translation is honest. For a genuine actor system — a
telecom switch, a game server's entity messaging, an Erlang-style supervision
tree — no. The model is a poor fit and the program that results is not the same
program.

### 24.3 Responsive service alongside a long computation (architecture 20)

**Why.** It is the one place where the absence of preemption, not the absence of
shared state, is what bites. Every other architecture loses concurrency of *data
access*; this one loses concurrency of *attention*. And the tax is paid in source
form: the computation must be hand-written as a resumable state machine, which is
the largest single rewriting burden in the catalog, and one that a language whose
pitch is "write it sequentially" is uncomfortable asking for.

**What a real program looks like.** A build server that must answer status
queries while linking:

```whitefoot
struct LinkJob {
  worklist: buffer<SectionId>;   // the link's own recursion, as data
  head: u64;
  emitted: u64;
  phase: LinkPhase;              // an enum the writer maintains by hand
}

for @serve (r in 0_u64..rounds) {
  let answered = answer_status(listener: &l, handles: &uniq handles);
  let advanced = link_chunk(job: &uniq job, budget: 50000_u64);
  if deref(job).phase == LinkPhase::Done() { break @serve; }
}
```

Everything that was a recursive descent over the section graph — the natural way
to write a linker's relocation pass — becomes `worklist`, `head` and `phase`, and
every function that used to recurse becomes a function that pushes. That is
perhaps 30–50% more source and a meaningful drop in legibility, which is
precisely the currency the language is trying to earn.
The throughput cap is real but is no longer disqualifying: under the batch I/O
model the service side serves *a whole ready batch* per round, so the cap is one
batch per chunk rather than one query per chunk. At 64 ready connections and a
1 ms chunk that is ~64,000 queries/s. The earlier draft's "one request per chunk"
finding belonged to the staged design and is withdrawn.

**Is the loss acceptable?** For a service where the long computation is rare and
the latency target is loose (a batch job with a health endpoint), yes, easily.
For an interactive system where a long computation is the normal state — an IDE's
language server, a database running a long analytical query beside OLTP traffic,
a game server doing pathfinding — the rewrite is severe, and what decides it is
the source tax rather than the throughput: the solver, the query engine or the
pathfinder must be written as an explicit work stack. Response p99 is then a
chunk budget, cheap to shrink on a busy box and costing 10–16% on a quiet one.

### 24.4 Deep await chains flattened into hand-written phases (architecture 19, and 17–18 with it)

**Why this replaces the earlier fourth entry.** The previous draft named very
high fan-out I/O, on the grounds that a suspended call is a stack and an idle
epoll connection is a struct — a 30x memory ratio and a connection ceiling.
**That finding is withdrawn with the stackful design.** Under §0.1's batch model
an idle connection costs a row plus a poolable buffer, which is the same order as
libuv's per-connection state, and the C10K/C1M ceiling goes with it.

What remains, and what is now the fourth weakest point, is the cost that the
stackful design was *hiding*: with one control flow and no suspendable call, a
function that wants to await five frames down cannot. It must return the
operation it wants as data, and the task's logical call stack must be written out
by hand as phases in a row. **The cost scales with await depth, not with task
count**, so it is invisible in a benchmark with one await and severe in real
service code, where three to eight await points spread across helper functions is
ordinary.

**What a real program looks like.** A database client's `run_query`, which in
tokio is one function with four awaits:

```whitefoot
// tokio: let c = pool.get().await; let s = c.prepare(sql).await;
//        let r = s.query(args).await; cache.put(key, r.len()).await;
enum JobPhase { WantConn() | WantPrepare() | WantRows() | WantCache() | Done() | Failed() }

struct JobRow {
  phase: JobPhase;        // the call stack, flattened
  job: Job;
  stmt: StmtId;           // what `prepare` produced, carried across the wait
  rows: u64;
  deadline: u64;
}

fn advance_job(row: own JobRow, buf: &uniq MutSlice<u8>, done: own Completion)
  -> result: own JobRow reads(row, buf, done), writes(buf) {
  match row.phase {
    WantConn()    => { return JobRow(phase: WantPrepare(), /* … */); }
    WantPrepare() => { return JobRow(phase: WantRows(), stmt: stmt_of(done: &done), /* … */); }
    WantRows()    => { return JobRow(phase: WantCache(), rows: rows_of(done: &done), /* … */); }
    WantCache()   => { return JobRow(phase: Done(), /* … */); }
    Done()        => { return move row; }
    Failed()      => { return move row; }
  }
}

fn want_next(row: &JobRow) -> result: own OpRequest reads(row) {
  match row.phase {
    WantConn()    => { return GetConn(); }
    WantPrepare() => { return Prepare(sql: row.job.sql); }
    WantRows()    => { return Query(stmt: row.stmt, args: row.job.args); }
    WantCache()   => { return CachePut(key: row.job.key, value: row.rows); }
    Done()        => { return Nothing(); }
    Failed()      => { return Nothing(); }
  }
}
```

Four awaits become a six-arm enum, two `match` functions, and three carried
fields — and every helper on the path (`pool.get`, `prepare`, `query`) has its
signature changed from "does the I/O" to "says what I/O it wants". A loop with an
await inside it adds a counter and a sub-phase. That is the shape, and it is
mechanical rather than hard, but it is a great deal more source than the tokio
version, and it is the thing the language's "write it sequentially" pitch is
least able to claim here.

**Is the loss acceptable?** For a service with shallow awaits — accept, read,
write, close, which is architectures 17 and 18 — comfortably: four phases is a
reasonable price for one legible loop, declared effects and no callback graph,
and the batch model's dispatch loop is *parallel* where a reactor's is not. For
deep, composed async code — a client library built on a connection pool built on
a TLS layer built on a socket, each layer with its own await points — the
flattening cuts across the layering, because a layer that wants to await must
publish that fact in its return type all the way up. That is a real constraint on
how I/O libraries can be factored in this language, and it deserves an answer in
the I/O API rather than a note in a catalog.

**Runner-up, for the record.** Architecture 13′ (BFS on a high-diameter graph) is
the only remaining *asymptotic* loss in the catalog — O(V·D) against O(V+E),
1,000x on a 1M-vertex diameter-1,000 graph — and would be the fifth entry. It is
excluded from the four only because the shape is narrower than the other four in
real systems code.

---

## 25. Three things this catalog would put in front of the I/O API design

Not recommendations about the model — observations that fall out of the catalog
and that the I/O API can decide either way.

1. **Inline completion is worth 3x, measured, and the corrected park figure
   makes it worth more.** The E5 experiment — a socket transfer the host can
   answer at once completes on the submitting thread, with no ring, no park and
   no wake — moved the echo server from 69.5k to 207k round trips per second.
   Whatever `op_start` / `op_finish` look like, the case where the operation is
   already satisfiable must not be forced through the completion path. The core
   costs are **31.7–32.9 ns for an inline terminal** against a **10–16 µs Linux
   park-and-wake** (`sched/core.c` 10.3 µs; `park-on-miss-measurements` 16.2 µs)
   — a ratio of **300–500x**, not the 50x an earlier draft assumed from the
   Darwin arm64 figure. This is the single highest-leverage decision in the API.

2. **A timer pending decides whether architectures 18, 19 and 20 are expressible
   at all.** Per-connection timeouts, periodic duties inside the batch loop, and
   deadline-based cancellation all need `wait_batch` to be able to return "the
   timer, not the socket". Without one, every timeout must live inside an
   individual operation's contract, and a program cannot impose a deadline on an
   operation whose implementation does not offer one. Architecture 20 needs a
   little more: a **deadline or poll form of `wait_batch`**, so a quiet moment
   does not block the loop and stop the interleaved computation.

3. **How a deep function says it wants to do I/O is the API's most consequential
   shape decision, and it is the one this catalog is least able to settle.** With
   [PAR-3] set aside, a helper five frames down either blocks the whole program
   or returns its wanted operation as data. The second is what real programs will
   do, so the `OpRequest`-style return type propagates up every layer that might
   do I/O — a colouring that the type system makes visible rather than viral, but
   a colouring nonetheless, and the mechanism behind §24.4. Two things follow:
   the operation-request vocabulary is part of the *language's* I/O surface and
   not an application concern, since every library will have to speak it; and a
   cancel operation on a `Pending` is worth having, because it is the only way
   per-task cancellation becomes expressible at all (§19), and it is cheap
   relative to the alternative of drain-everything-and-stop.
