# Proof-derived parallelism v1 — measured results (batch 0074)

What this file reports: what one real Whitefoot program does when the
compiler's permission judgment is actualized on worker lanes, measured against
the same program executing sequentially. It is the evidence half of
`DESIGN.md`; the design rationale and the deferred items live there.

Nothing here is a promise about other programs. A speedup measured on one
workload is a fact about that workload.

> **The Amdahl conclusion of sections 6 to 9 is superseded, 2026-08-23.** Every
> number below was measured while `layout_banded` — the claim-bearing fold,
> §8.7's 33.9% of the program — was permanently sequential, because the
> permission judgment refused to actualize a pair whose call closure reached a
> `claim`. Batch 0078-C withdrew that refusal under the owner's chartering
> direction of that day, so the fold that the tables treat as the unparallelizable
> serial share is now overlapped like any other. The measurements stand as
> measurements of the code that produced them; the *ceiling* they compute for
> this demo does not. The grant counts before and after, on the same source and
> the same runtime, are in `docs/ongoing/0078-loop-permission.md`.

**Actualization is compile-time opt-in.** `whitefootc --par` is what emits an
outlined call, a lane offer, and a join; the default compilation emits none of
them. Section 4's own measurement is why: the outlining alone, with no runtime
linked and no worker requested, cost about 1.2x on this demo and 2.1x on
`fib(38)`, which contradicted the design's stated default of byte-identical
behavior. Every table below therefore names which compilation it measured.
`WF_WORKERS` remains the runtime knob for a program that was built with
`--par`, but it is no longer *unchanged*: batch 0077's L1 (`62e30831`) made an
unset or empty setting ask for one lane per logical CPU, where it previously
meant no pool. Rows below that name an unset `WF_WORKERS` were taken under the
old sense and reproduce today with `WF_WORKERS=1`.

## 1. The program under measurement

`tests/programs/par_layout.wf` is a box-tree layout pass. It builds one tree of
depth 6 (63 branch nodes, 64 leaves), fills a shared 8,192-entry metric table,
and then folds the tree twice — once per repetition, 800 repetitions each:

- `layout` measures each node against the **whole** metric table. The walk is
  bounded by the table's own length, so its index obligation is discharged by
  the checker and the function's whole call closure is claim-free.
- `layout_banded` measures each node against a **caller-supplied prefix** of
  the same table. The bound comes from the caller, so the index obligation is
  carried by one `claim` in `measure_band`.

Nothing else differs. Both folds have the same recursive shape, write into the
same kind of per-node slot, and read the same shared table. The program then
publishes the exact bits of both fold results as hexadecimal, so a divergence
anywhere in either tree is a divergence in the published bytes.

The compiler's own account of the two folds, from `whitefootc --par-ledger`,
retaken at the branch tip on 2026-08-22. Batch 0076's Dig 8 (`974d5513`)
replaced the adjacency rule with a judged window, which added the `PAR chain`
lines and the interposed-statement denials and reworded the condition-1 reason;
the transcript this document carried was the pre-Dig-8 one.

```
PAR permitted   tests/programs/par_layout.wf:19  pair(build, build)  eligible
PAR chain       tests/programs/par_layout.wf:19  run(build, build)  2 members through line 20
PAR denied      tests/programs/par_layout.wf:106  pair(cascade, measure_words)  condition 1: the operands of s2 read what s1 defines
PAR denied      tests/programs/par_layout.wf:113  pair(cascade, measure_words)  condition 1: the operands of s2 read what s1 defines
PAR denied      tests/programs/par_layout.wf:114  pair(measure_words, layout)  condition 1: the operands of s2 read what interposed statement 1 defines
PAR permitted   tests/programs/par_layout.wf:116  pair(layout, layout)  eligible
PAR chain       tests/programs/par_layout.wf:116  run(layout, layout)  2 members through line 117
PAR denied      tests/programs/par_layout.wf:131  pair(cascade, measure_band)  condition 1: the operands of s2 read what s1 defines
PAR denied      tests/programs/par_layout.wf:138  pair(cascade, measure_band)  condition 1: the operands of s2 read what s1 defines
PAR denied      tests/programs/par_layout.wf:139  pair(measure_band, layout_banded)  condition 1: the operands of s2 read what interposed statement 1 defines
PAR permitted   tests/programs/par_layout.wf:141  pair(layout_banded, layout_banded)  not-actualizable: 1 claim site via measure_band
(superseded 2026-08-23: this line now reads `pair(layout_banded, layout_banded)  eligible`)
```

Those lines are the same with and without `--par`: the judgment is pure, so the
ledger is a property of the source rather than of the compilation asked for.

Both child pairs are **permitted** — the judgment's four conditions hold for
each. Only the claim-free one is **eligible**, and the module emitted by
`whitefootc --par` shows the distinction directly: `@wf_layout` carries an
outlined thunk, a lane offer, and a join, and `@wf_layout_banded` names no part
of the runtime at all. The default compilation of the same file names no part
of the runtime in either function.

That is the whole experiment in one program: one claim site in one callee is
the entire difference between a fold that scales and a fold that does not, and
the ledger says so before anything is run.

## 2. Machine and build

- Apple M4, 10 cores: 4 performance + 6 efficiency. 16 GB. macOS 26.5.2.
- Apple clang 21.0.0, target `arm64-apple-darwin25.5.0`, linked at `-O2`
  (`HOST_OPTIMIZATION_ARGUMENTS`), the same flags `whitefootc` uses.
- Runtime: `compiler/src/backend/par_runtime.c`, pthreads, lane-budget policy
  (an offer is granted only when a worker is idle; otherwise the caller runs
  the work inline).
- Two compilers, both `--release`: this branch, and the non-outlined baseline
  built from `main` at `4f01bab6`. Where a table names a baseline, that is the
  build it means.
- Machine otherwise idle; on wall power; no other measurement running.

## 3. Protocol

- **Interleaved.** One round runs every worker setting once, in a fixed
  rotation `1, 2, 4, 8`, and the rounds repeat. Thermal drift and background
  load therefore spread across every configuration instead of concentrating in
  whichever ran first. `WF_WORKERS` is read once at pool start, so a single
  process cannot switch configurations; the rotation is across process
  invocations, which is the finest interleaving this runtime allows. That is a
  weaker instrument than in-process A/B and is stated as such.
- **N = 9 rounds** for the whole program and for the eligible phase, **N = 7**
  for the not-actualizable phase. Minimum, median, and maximum are all
  reported; speedups are quoted from both the minimum and the median.
- **Byte comparison** on every single run: the harness compares each run's
  complete standard output against the first run's and stops on the first
  difference. No difference was observed in any run reported below.
- **Resolution rule.** A difference under 20% between two configurations is
  reported as unresolved rather than as an effect.
- Timing brackets the whole process, so process start, tree build, table fill,
  and output are inside every number.

## 4. Whole program

800 layout repetitions plus 800 banded repetitions, N = 9. **Every row below is
the `--par` compilation**; the default compilation is section 4.1.

| workers | min (ms) | median (ms) | max (ms) | spread | speedup (min) | speedup (median) |
|---------|---------:|------------:|---------:|-------:|--------------:|-----------------:|
| 1       |    715.5 |       729.7 |    738.1 |   3.2% |         1.00x |            1.00x |
| 2       |    491.4 |       504.6 |    520.6 |   5.9% |         1.46x |            1.45x |
| 4       |    398.8 |       405.0 |    415.1 |   4.1% |         1.79x |            1.80x |
| 8       |    400.5 |       407.0 |    425.8 |   6.3% |         1.79x |            1.79x |

Reference, same module linked with **no runtime at all** — the module's own weak
definitions answer and every offer is refused: min 715.8 ms, median 718.4 ms,
max 735.6 ms. That is statistically indistinguishable from `WF_WORKERS=1`, and
every published byte was identical to every other run's.

**Correction (batch audit, 2026-08-21).** That reference is the *same emitted
module* linked differently, so it measures the runtime's absence and not the
lowering's cost, and the sentence that stood here — "the default-off path costs
nothing measurable" — did not follow from it. What follows in section 4.1 is
the comparison that does: the same source through a compiler that has no
overlap lowering at all.

4 versus 8 workers differs by 0.4%: **unresolved** by the 20% rule. The machine
has 4 performance cores, so more lanes than that buy nothing here.

## 4.1 The compile-time option, against a non-outlined build

The only honest baseline is the same source compiled by a compiler with no
overlap lowering: `whitefootc` built from `main` at `4f01bab6`. Re-measured on
the same machine after the lowering became opt-in, interleaved rotation across
configurations, N = 9 rounds, byte-comparing every run:

| compilation and execution | best (ms) | median (ms) | vs baseline |
|---------------------------|----------:|------------:|------------:|
| baseline compiler from `main` | 762.1 | 781.4 | 1.00x |
| branch, **default** (no `--par`) | 742.6 | 766.2 | 1.03x |
| branch, default, `WF_WORKERS=4` | 743.0 | 767.0 | 1.03x |
| branch, `--par`, `WF_WORKERS` unset | 902.5 | 911.4 | 0.84x |
| branch, `--par`, `WF_WORKERS=1` | 872.7 | 900.5 | 0.87x |
| branch, `--par`, `WF_WORKERS=2` | 591.0 | 598.4 | 1.29x |
| branch, `--par`, `WF_WORKERS=4` | 452.5 | 457.6 | **1.68x** |
| branch, `--par`, `WF_WORKERS=8` | 445.5 | 450.7 | 1.71x |

All 72 runs published identical bytes.

Three readings, in the order they matter.

**The default now costs nothing measurable, and this time the reference is a
non-outlined build.** 742.6 ms against 762.1 ms is within the 3% run-to-run
spread of section 4's own table, and the default build names no runtime symbol
at all — verified structurally as well as by timing: all twenty single-source
corpus programs emit modules byte-identical to the baseline compiler's apart
from the one-line `; QUAL-1 qualification: specification v0.33/v0.34` stamp.
`WF_WORKERS=4` on a default build is 743.0 ms, because there is nothing in that
program for a worker to take.

**Switching the lowering on costs about 1.2x before any worker runs.** 902.5 ms
unset against 762.1 ms baseline. That is the outlining: the thunk passes its
arguments through a memory frame and is reached through a function pointer, so
the call cannot be inlined, and the weak `wf__par_try_fork` cannot be folded
away because a linker may replace it. It is the price of asking, and it is now
only paid by a build that asks.

> **Superseded for this program by batch 0076 Dig 3 (2026-08-21).** The 1.2x
> was the pre-Dig-1 unconditional per-activation hand-out frame. Re-measured
> on the same program with the compiler at `826cea41`, interleaved, N = 11:
> default 0.7486 s against `--par` `WF_WORKERS=1` 0.7481 s — 1.00x, no tax.
> The tax was never a property of outlining as such; it was F1. It remains
> real where the grain is fine: `fib(38)` still measures 2.6x (0.0839 s
> default against 0.2201 s `--par` unset) at the same commit.

**The win at four workers is 1.68x against the non-outlined baseline** and 1.93x
against the `--par` build's own `WF_WORKERS=1`. The second figure is the one the
phase decomposition in section 5 explains; the first is the one a user gets.

## 5. Phase decomposition and the Amdahl share

The same program with one repetition loop set to zero isolates each phase.
Sequential (`WF_WORKERS=1`) minima: eligible fold **481.2 ms**,
not-actualizable fold **246.5 ms**, everything else (build, fill, publish)
**below the 10 ms clock resolution**.

**Eligible phase alone**, N = 9:

| workers | min (ms) | median (ms) | max (ms) | speedup (min) | speedup (median) |
|---------|---------:|------------:|---------:|--------------:|-----------------:|
| 1       |    481.2 |       514.1 |    539.4 |         1.00x |            1.00x |
| 2       |    254.6 |       270.3 |    418.6 |         1.89x |            1.90x |
| 4       |    161.5 |       163.0 |    290.0 |         2.98x |            3.15x |
| 8       |    160.3 |       163.2 |    201.9 |         3.00x |            3.15x |

**Not-actualizable phase alone**, N = 7:

| workers | min (ms) | median (ms) | max (ms) | speedup (min) |
|---------|---------:|------------:|---------:|--------------:|
| 1       |    246.5 |       247.6 |    257.4 |         1.00x |
| 2       |    247.7 |       249.4 |    254.5 |         1.00x |
| 4       |    247.1 |       248.7 |    249.8 |         1.00x |
| 8       |    245.1 |       246.7 |    252.8 |         1.01x |

Every difference in that second table is under 5%: **unresolved**, which is the
expected reading — one claim site keeps the whole fold sequential, and the
measurement sees exactly that.

**Observed Amdahl share.** Eligible work is 481.2 / 727.7 = **66.1%** of the
program; the not-actualizable fold is **33.9%**. Composing the eligible phase's
own measured scaling with that share predicts the whole-program speedup:

| workers | phase speedup | predicted whole | measured whole |
|---------|--------------:|----------------:|---------------:|
| 2       |         1.89x |           1.45x |          1.46x |
| 4       |         2.98x |           1.78x |          1.79x |
| 8       |         3.00x |           1.79x |          1.79x |

The prediction matches the measurement to within 1% at every worker count, so
the whole-program result is fully explained by the phase decomposition. There
is no unaccounted overhead at this granularity.

The eligible phase reaches 2.98x on 4 performance cores — 75% of the ideal 4x.
The 25% shortfall is the tree's own critical path (the root's two subtrees
cannot start before the root's `cascade` and `measure_words` finish), the
lane-budget policy's inline fallback, and the fork/join edges themselves.

## 6. What the runtime actually granted

The pool's own grant counter, read by an observer at process exit (this counter
is runtime-internal; no Whitefoot construct can name it):

| workers | lanes granted | offers made | granted |
|---------|--------------:|------------:|--------:|
| 1       |             0 |      50,463 |    0.0% |
| 2       |           801 |      50,463 |    1.6% |
| 4       |         2,529 |      50,463 |    5.0% |
| 8       |         8,031 |      50,463 |   15.9% |

`WF_WORKERS=1` starts no pool and grants nothing, which is why its timing
matches the no-runtime link exactly. Above that, the lane-budget policy refuses
95% or more of all offers: the fork is attempted at every branch node, and
almost all of them find every worker busy and run inline. Granting a small
fraction of a very large number of offers is the policy working as designed
rather than a missed opportunity — the granted lanes are the ones near the root,
which carry the large subtrees.

This table is also the answer to a "green is not coverage" objection: a repeat
test that only compared bytes would pass just as well against a pool that
silently granted nothing. It did not.

## 7. Determinism

Across every run reported in this file — 36 whole-program runs at four worker
counts, 36 eligible-phase runs, 28 not-actualizable-phase runs, and 9 runs of
the module linked with no runtime at all — the published bytes were identical:

```
420a993efa7437a1 41fa962893d45299
```

Two IEEE-754 binary64 values, bit for bit, from a fold that writes into 127
distinct tree slots and sums them back up. A single misordered write or a
single missed join anywhere in the tree changes the low bits of the root sum
and therefore changes those characters.

The gate carries this as `compiler/tests/programs/parallel.rs`, which compares
the default execution against `WF_WORKERS=2` and `WF_WORKERS=4`, and as the
in-crate repeat test in `compiler/src/backend/tests/parallel.rs`, which
additionally reads the grant counter so a silently sequential link cannot pass.

## 8. Limitations

Stated plainly, because each one bounds what the numbers above mean.

1. **One workload, one machine, one shape.** A recursive fold over a heap tree
   with a heavy per-node body is the shape this design was built for. Nothing
   here measures loops, buffer splitting, or I/O — those are `DESIGN.md` §2
   "Out" and have their own triggers.
2. **The Amdahl share is a property of this program's chosen repetition
   counts.** 800 and 800 were picked to put a substantial, deliberately visible
   sequential fraction next to the eligible one. A program with no claims at all
   would show the eligible phase's scaling directly; one that is mostly
   claim-bearing would show almost none.
3. **Interleaving is across processes, not within one.** `WF_WORKERS` is read
   once at pool start. A configuration switch inside one process would be a
   stronger instrument and does not exist.
4. **Grain is not tuned.** The fork is offered at every branch node regardless
   of subtree size. On a tree whose per-node body is cheap, that overhead
   dominates and the overlapped execution is *slower*: an earlier draft of this
   same program with a 16-entry table and a depth-12 tree measured 0.08 s
   sequential and 0.17 s at four workers, a 2.1x **slowdown**. Profitability is
   not permission, and v1 ships no profitability policy beyond the lane budget.
   That measurement is the concrete case for the heartbeat successor recorded
   in `DESIGN.md` §5.
5. **The efficiency cores do not help.** 8 workers on a 4P+6E machine measured
   the same as 4. The runtime has no core-class awareness and asks for none.
6. **The byte comparison proves agreement, not the absence of a race.** It is a
   strong test — 127 slots, a strictly ordered float reduction, and a
   bit-exact publication — but a race that never manifested in 109 runs would
   look exactly like this. The load-bearing argument for correctness is the
   permission judgment and its per-condition tests, not this table.

The batch audit added three more, each measured rather than argued.

7. **Every comparison in sections 4 to 7 has the same emitted module on both
   sides.** The "no runtime" reference, the repeat test, and the phase tables
   all link one module two ways, so a defect introduced by the lowering itself
   is present in the reference and compares equal. The audit found exactly such
   a defect this way — a moved operand read — and it is repaired; the durable
   guard is now the in-crate differential
   `the_overlapped_lowering_agrees_with_the_lowering_that_hands_nothing_out`,
   which emits one source both ways — the default compilation and `--par` — and
   compares the two programs.
8. **Asking for the lowering costs something before any worker runs.** The
   outlined thunk passes its arguments through a memory frame and is reached
   through a function pointer, so the call cannot be inlined, and the weak
   `wf__par_try_fork` cannot be folded away because a linker may replace it.
   Once the lowering became compile-time opt-in that cost belongs to `--par`;
   this row is what a build pays for asking. On `fib(38)` — plain two-way
   recursion whose child pair is `eligible`, whose per-call body is a handful
   of instructions — the same source through both compilers, interleaved,
   N = 7:

   | compilation and execution | best (ms) | median (ms) | vs baseline |
   |---------------------------|----------:|------------:|------------:|
   | baseline compiler from `main` | 79.7 | 81.0 | 1.00x |
   | branch, **default** (no `--par`) | 76.7 | 80.0 | 1.04x |
   | branch, `--par`, module linked with no runtime | 149.3 | 156.7 | **1.9x slower** |
   | branch, `--par`, `WF_WORKERS` unset | 265.5 | 274.2 | **3.3x slower** |
   | branch, `--par`, `WF_WORKERS=4` | 1006.0 | 1177.2 | **12.6x slower** |

   All five publish `0000000002547029`. The first two rows are the whole point
   of the option: on the program where the lowering is worst, the default build
   is the baseline build. The rest is the grain hazard, unchanged in magnitude
   from the closure measurement (2.1x / 3.9x / ~17x there), and now reachable
   only by asking for it.

   > **The lane rows are superseded by batch 0076 Dig 2; re-measured by Dig 3
   > (2026-08-21) at `826cea41`, same source, interleaved, N = 15, all
   > publishing `0000000002547029`:** default 0.0839 s, `--par` unset 0.2201 s
   > (2.6x), `WF_WORKERS=2` 0.1933 s, `=4` **0.1112 s**, `=8` **0.0767 s**.
   > The 12.6x-slower lane row is gone — work stealing turned it into 1.33x at
   > four workers and 0.91x at eight, i.e. faster than the default build. The
   > opt-in tax itself (2.6x) survives, because this program's grain is a
   > handful of instructions per call and the frame cannot amortise.
   >
   > **Superseded again, 2026-08-22:** the 2.6x opt-in tax did not survive
   > either. Dig 7 landed the sequential clone, so a `--par` build that is not
   > granted lanes runs the sequential code, and `fib(38)` re-measured at
   > **1.00x** (0.2349 s -> 0.0791 s). Note also that "`--par` unset" in the row
   > above means the pool-off execution, which is what an unset `WF_WORKERS`
   > meant when the row was taken; batch 0077's L1 (`62e30831`) changed an unset
   > setting to mean one lane per logical CPU, so reproducing that row today
   > needs `WF_WORKERS=1`.
9. **Switching lanes on for a fine-grained program is a large loss, not a small
   one.** `wfgrep e compiler` over this repository's own source tree — the
   branch tip through `git archive`, so no build directory — interleaved,
   N = 11. This workload is about 30 ms, so the minimum is noise-prone and the
   median is the statistic to read:

   | compilation and execution | best (ms) | median (ms) | vs baseline median |
   |---------------------------|----------:|------------:|-------------------:|
   | baseline compiler from `main` | 28.7 | 30.1 | 1.00x |
   | branch, **default** (no `--par`) | 27.5 | 30.2 | 1.00x |
   | branch, `--par`, `WF_WORKERS` unset | 20.5 | 29.9 | 1.01x |
   | branch, `--par`, `WF_WORKERS=2` | 36.4 | 42.0 | **1.40x slower** |
   | branch, `--par`, `WF_WORKERS=4` | 35.0 | 42.0 | **1.40x slower** |
   | branch, `--par`, `WF_WORKERS=8` | 34.8 | 42.6 | **1.42x slower** |

   All 66 runs published identical bytes. The root here is smaller than the one
   measured at closure (0.38 s, which included the build directory, and where
   the same switch cost 2.2x), so the ratio is smaller; the sign and the cause
   are the same. `wfgrep` offers a lane for `pair(byte_at, byte_at)` three
   times, one of them inside its directory-entry comparator's byte loop, so it
   offers a lane per byte comparison of every sort. `sha256_abc` likewise
   offers one per round of its compression loop.

   Limitation 4's 2.1x was recorded as the concrete case for the deferred
   heartbeat policy; 12.6x on `fib(38)` and 1.4x on the project's own flagship
   program are one to two orders of magnitude past the 0.69x worst case the
   lane-budget rationale rests on. Nothing in v1 gates a fork on grain.

   What the compile-time option changes is who pays. Before it, that hazard was
   a shipped regression on every build of every program with an eligible pair.
   Now it is a property of the instrument: a build that says `--par` is asking
   to measure overlap on that program, and on a fine-grained program the answer
   is that overlap loses. The deciding evidence for the heartbeat successor is
   unchanged and still says a profitability policy is required work rather than
   a refinement; it is no longer a reason to hold a merge.

## 9. Reproducing

From the repository root, with the compiler built:

```
WFC="cargo run --manifest-path compiler/Cargo.toml --release --bin whitefootc --"

# the shipped default: the judgment is reported, nothing is handed out
$WFC --par-ledger -o /tmp/par_layout tests/programs/par_layout.wf
/tmp/par_layout                     # no thunk, no offer, no runtime linked

# the same source asking for lanes
$WFC --par -o /tmp/par_layout_par tests/programs/par_layout.wf
/tmp/par_layout_par                 # runtime linked, no pool started
WF_WORKERS=4 /tmp/par_layout_par    # four lanes offered
```

Both print the same two hexadecimal values. The gate's own copy of the
comparison is `cargo test --manifest-path compiler/Cargo.toml --test programs
programs::parallel`.

The deciding research probes behind the design are in `probes/` beside this
file, with `probes/README.md` stating what each one settles.
