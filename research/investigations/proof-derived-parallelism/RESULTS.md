# Proof-derived parallelism v1 — measured results (batch 0074)

What this file reports: what one real Whitefoot program does when the
compiler's permission judgment is actualized on worker lanes, measured against
the same program executing sequentially. It is the evidence half of
`DESIGN.md`; the design rationale and the deferred items live there.

Nothing here is a promise about other programs. A speedup measured on one
workload is a fact about that workload.

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

The compiler's own account of the two folds, from `whitefootc --par-ledger`:

```
PAR permitted   tests/programs/par_layout.wf:19  pair(build, build)  eligible
PAR denied      tests/programs/par_layout.wf:106  pair(cascade, measure_words)  condition 1: an argument of s2 uses the result of s1
PAR denied      tests/programs/par_layout.wf:113  pair(cascade, measure_words)  condition 1: an argument of s2 uses the result of s1
PAR permitted   tests/programs/par_layout.wf:116  pair(layout, layout)  eligible
PAR denied      tests/programs/par_layout.wf:131  pair(cascade, measure_band)  condition 1: an argument of s2 uses the result of s1
PAR denied      tests/programs/par_layout.wf:138  pair(cascade, measure_band)  condition 1: an argument of s2 uses the result of s1
PAR permitted   tests/programs/par_layout.wf:141  pair(layout_banded, layout_banded)  not-actualizable: 1 claim site via measure_band
```

Both child pairs are **permitted** — the judgment's four conditions hold for
each. Only the claim-free one is **eligible**, and the emitted module shows the
distinction directly: `@wf_layout` carries an outlined thunk, a lane offer, and
a join, and `@wf_layout_banded` names no part of the runtime at all.

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

800 layout repetitions plus 800 banded repetitions, N = 9.

| workers | min (ms) | median (ms) | max (ms) | spread | speedup (min) | speedup (median) |
|---------|---------:|------------:|---------:|-------:|--------------:|-----------------:|
| 1       |    715.5 |       729.7 |    738.1 |   3.2% |         1.00x |            1.00x |
| 2       |    491.4 |       504.6 |    520.6 |   5.9% |         1.46x |            1.45x |
| 4       |    398.8 |       405.0 |    415.1 |   4.1% |         1.79x |            1.80x |
| 8       |    400.5 |       407.0 |    425.8 |   6.3% |         1.79x |            1.79x |

Reference, same module linked with **no runtime at all** — the module's own weak
definitions answer, every offer is refused, and this is exactly today's
execution: min 715.8 ms, median 718.4 ms, max 735.6 ms. That is statistically
indistinguishable from `WF_WORKERS=1`, so the default-off path costs nothing
measurable, and every published byte was identical to every other run's.

4 versus 8 workers differs by 0.4%: **unresolved** by the 20% rule. The machine
has 4 performance cores, so more lanes than that buy nothing here.

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

## 9. Reproducing

From the repository root, with the compiler built:

```
cargo run --manifest-path compiler/Cargo.toml --release --bin whitefootc -- \
    --par-ledger -o /tmp/par_layout tests/programs/par_layout.wf
/tmp/par_layout                 # today's execution: no pool, no lanes
WF_WORKERS=4 /tmp/par_layout    # four lanes offered
```

Both print the same two hexadecimal values. The gate's own copy of the
comparison is `cargo test --manifest-path compiler/Cargo.toml --test programs
programs::parallel`.

The deciding research probes behind the design are in `probes/` beside this
file, with `probes/README.md` stating what each one settles.
