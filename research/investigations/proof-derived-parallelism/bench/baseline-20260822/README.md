# Baseline, 2026-08-22 — the current reference

This snapshot supersedes `baseline/` as the reference measurement. `baseline/`
is kept unchanged as the 2026-08-21 record; nothing here edits it.

- **HEAD:** `1e103492` on `par/proof-derived-parallelism`.
- **N = 18** rounds over all 13 configurations and all 14 cells: 182 cells per
  round, **3,276 runs**, every one exiting 0.
- **Full protocol**, `PROTOCOL.md` as amended on this date: every cell visited
  once per round in a fixed rotation, no cell run twice in a row, no
  configuration run as a block.
- **Byte comparison over the real published bytes of every run**: 1,404
  Whitefoot outputs and 1,872 Rust outputs compared by `cmp`, every run of
  every configuration identical within each language and across them
  (`byte_comparison.txt`).

## What is new here

The two `default` cells, added by the protocol amendment of this date:
`wf_par/default` is the `--par` binary with `WF_WORKERS` genuinely unset, and
`rs_rayon/default` is the same rayon code with no thread count named anywhere.
Both are what a program that configures nothing gets, and neither appeared in
`baseline/`. `t_defaults.md` is the table that reads them side by side.

## Machine state — read the spread column with this in mind

The run started at 04:14 with a one-minute load average of 2.68 and finished at
04:36 at 7.08. Corporate agents (Microsoft Defender `wdavdaemon`, `epsext`,
`netext`, CorpLink) were active throughout and were the main competing load.
Per-round one-minute load averages, in order:

```
2.68  6.23  6.10  4.27  5.65  6.26  7.12  4.79  4.12
4.71  4.91  4.80  4.22  8.44  7.45  7.77  7.48  7.46
```

Rounds 14-18 ran under the heaviest contention of the pass. The interleaving is
what makes this tolerable: contention lands on whichever cell is running rather
than on one implementation, so it inflates maxima and medians without moving
minima together. It shows up as spread — every one of the 182 cells exceeds 20%
spread and the mean is 112% — which is why the protocol reports **minimum** of N
and takes every ratio between minima. The min-of-18 numbers reproduce a
one-round probe taken at load 2.9 to within a few percent, which is the evidence
that the minima survived the contention; the medians and maxima did not, and
none of them is quoted.
