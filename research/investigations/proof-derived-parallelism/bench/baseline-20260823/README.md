# Baseline, 2026-08-23 — the current reference

This snapshot supersedes `baseline-20260822/` as the reference measurement.
`baseline-20260822/` and `baseline/` are kept unchanged as the 2026-08-22 and
2026-08-21 records; nothing here edits either.

- **HEAD:** `0c56d5bd` on `par/loop-permission` — the rebased lineage, so this
  is the first authoritative rotation that includes `main`'s compiler delta.
- **N = 18** rounds over all 13 configurations and all 14 cells: 182 cells per
  round, **3,276 runs**, every one exiting 0.
- **Full protocol**, `PROTOCOL.md` as amended 2026-08-22: every cell visited
  once per round in a fixed rotation, no cell run twice in a row, no
  configuration run as a block.
- **Byte comparison over the real published bytes of every run**: 1,404
  Whitefoot outputs and 1,872 Rust outputs compared by `cmp`, every run of
  every configuration identical within each language and across them
  (`byte_comparison.txt`). The 3,276 runs carry exactly 13 distinct published
  values, one per configuration.

## Machine state — this is the quiet pass

The pass ran 05:48 to 06:08 with the machine deliberately cleared of other
agent sessions beforehand. One-minute load average 1.65 before and 3.98 after;
per-round, in order:

```
1.62  3.23  5.10  5.65  4.77  4.07  4.45  5.42  6.02
5.40  4.64  3.85  4.89  4.36  3.51  4.39  5.98  4.29
```

Most of that load is the benchmark's own worker lanes. The only competing
processes observed mid-pass were the standing corporate agents (Microsoft
Defender `wdavdaemon`, CorpLink), never above 13% CPU; no process exceeded 30%
at any check, and no round is annotated for contamination.

**This is visible in the instrument.** Against `baseline-20260822/`, whose pass
ran at load 2.68 rising to 8.44, mean cell spread falls from **112% to 27%**,
worst cell spread from **411% to 146%**, and cells exceeding 20% spread from
**182 of 182 to 93 of 182**. The minima are what the protocol reports and they
moved far less than the spreads did — which is the evidence, recorded here
rather than asserted, that the prior pass's minima survived its contention.

## How the rotation was driven, and the one protocol note

The 18 rounds were run as five foreground invocations of `run_bench.zsh`
(2 + 4 + 4 + 4 + 4) using its `OFFSET` argument, which appends to one
`results.tsv` with continuous round numbering, rather than as a single
`rerun.zsh 18`. This was a harness constraint on the executor, not a
measurement choice. It is protocol-neutral: the rotation inside each round is
untouched, round numbering is continuous, and no cell runs twice in a row
across a seam either, because each round ends on `grid_d21_w256 rs_cut/8` and
the next begins on `bal_d8_w16 wf_seq`. `compare_outputs.zsh` and
`make_tables.zsh` then ran once over the completed 18-round set, exactly as
`rerun.zsh` runs them.

## What moved against `baseline-20260822/`, and what did not

Nine of 182 cells moved outside the protocol's 0.83x-1.20x band. **All nine are
`wf_par`, all on the `bal` family, all at 2 or 4 lanes, all at 64 or 192 words
per node, and all in the same direction (slower).**

| cell | 2026-08-22 | 2026-08-23 | ratio |
|---|---:|---:|---:|
| `bal_d8_w64` `wf_par/2` | 0.2655 | 0.3201 | 1.21x |
| `bal_d8_w192` `wf_par/2` | 0.3147 | 0.3960 | 1.26x |
| `bal_d8_w192` `wf_par/4` | 0.1650 | 0.2097 | 1.27x |
| `bal_d10_w64` `wf_par/2` | 0.2623 | 0.3170 | 1.21x |
| `bal_d10_w192` `wf_par/2` | 0.3117 | 0.3992 | 1.28x |
| `bal_d10_w192` `wf_par/4` | 0.1612 | 0.2061 | 1.28x |
| `bal_d12_w64` `wf_par/2` | 0.2691 | 0.3233 | 1.20x |
| `bal_d12_w192` `wf_par/2` | 0.3115 | 0.3968 | 1.27x |
| `bal_d12_w192` `wf_par/4` | 0.1638 | 0.2040 | 1.25x |

Four controls bound where this can live, and each excludes something:

- **The sequential floor did not move.** All 13 `wf_seq` minima reproduce
  within 1%, except `bal_d8_w16` at 1.04x. `main`'s compiler delta, the
  expected source of movement, did not reach Whitefoot's sequential codegen.
- **The Rust twin did not move.** All 52 `rs_rayon` cells lie between 0.96x and
  1.01x. Both languages ran interleaved in the same rotation under the same
  load in both passes, so if machine state were doing this, rayon at 2 and 4
  threads on the same configurations would move too. It does not.
- **The `--par` binary's sequential world did not move.** All 13 `wf_par/1`
  cells lie between 0.93x and 1.01x. The change is in the parallel clone only.
- **The permission judgment did not change what is actualized.**
  `--par-ledger` reports the same two eligible pairs (`build`, `layout`) with
  the same two-member chains on `bal` and on `skew` alike.

**Quantified, the move is a fixed amount of added parallel work.** Multiplying
each cell's penalty by its lane count gives a constant per configuration,
independent of lane count — which is what "added work, spread across the
lanes" looks like and what "added serialization" does not:

| config | x1 lane | x2 lanes | x4 lanes | x8 lanes |
|---|---:|---:|---:|---:|
| `bal_*_w16` | -0.05 .. -0.02 | +0.01 | +0.01 | +0.00 .. +0.02 |
| `bal_*_w64` | +0.00 | +0.11 | +0.11 | +0.08 .. +0.11 |
| `bal_*_w192` | +0.00 | +0.16 .. +0.18 | +0.16 .. +0.18 | +0.16 .. +0.17 |
| `skew_*` | +0.00 | -0.00 | -0.03 .. -0.00 | -0.04 .. -0.00 |
| `grid` | -0.00 | -0.01 | -0.01 | -0.00 |

So the parallel clone gained roughly **0.11 core-seconds at 64 words and 0.17
at 192 words** on the balanced tree, and nothing at 16 words, nothing on the
skewed tree, and nothing on the index split. The three `bal` depths agree with
each other because `reps` is fitted per configuration to hold total node count
near-constant, so this is per-node cost scaling with per-node data volume, with
a threshold between 16 and 64 words.

**The trap latch is not the cause.** `skew` and `grid` carry claims and hand
out to lanes exactly as `bal` does, so any latch-conditional emission would
reach them too, and they are flat. Attribution beyond this needs a compiler
bisect across the rebase over `backend/emitter/parallel.rs` and
`backend/par_runtime.c`; that was deliberately not run here. **Reported, not
chased.**

## Effect on the headline verdicts

Against rayon at matched worker counts (52 cells): **22 Whitefoot wins, 25
unresolved, 5 losses**, where the prior pass read 24 / 28 / 0. The five losses
are exactly five of the nine cells above.

At the shipped defaults on both sides (13 cells): **11 Whitefoot wins, 2
unresolved, 0 losses — unchanged from the prior pass.** The regression sits at
2 and 4 lanes and has largely decayed by 8 lanes, so it does not reach the
column that answers what an untuned program gets.
