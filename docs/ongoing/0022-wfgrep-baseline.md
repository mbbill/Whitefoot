# 0022 — Zero-change wfgrep baseline

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 1 (PERF-1)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `d84643b`
- **Frozen program bytes:** `tests/programs/wfgrep.wf` SHA-256
  `d5f94c1a0f9bd3d2e2b014f39f01f19730e85fb626733f8c0780366179732caa` — the zero-change
  baseline measures exactly these bytes regardless of later refactors

## Goal

Preregister and run the first honest performance measurement of the frozen
sequential `wfgrep` against a pinned comparator (`grep -h -F`), then profile
and attribute the first material divergence per the PERF-1 layer chain. The
RG-BASE lesson binds: host cache-position noise defeated a 3% precision gate
once; the protocol must state its noise controls and materiality rules
before any number is read. The scalar newline scan retaining its bounds trap
is the preregistered first attribution suspect — confirmed or refuted by
profile, never assumed.

## Progress

- Frozen program SHA re-verified; built through the ordinary `whitefootc`
  path (clang -O2). Comparator pinned: system BSD grep 2.6.0-FreeBSD.
- Corpus generated and digest-pinned (`MANIFEST.txt`); output identity
  (byte-equal stdout, equal exits) verified for all five cases before any
  timing.
- `research/experiments/wfgrep-baseline/PROTOCOL.md` preregistered: noise
  controls per the RG-BASE lesson (explicit warm pass, balanced order,
  null comparison before and after with a 2% demonstrated-precision gate,
  power-state records), bands, and the attribution plan with the scan-trap
  suspect and its falsifier. That commit was the freeze; timing followed.
- Run `wfgrep-baseline-1` complete; all null gates within precision (two
  sub-percent position biases disclosed). Ratios (grep/wfgrep): large
  0.647 [0.643, 0.653], nomatch 0.656 [0.649, 0.661], dense 1.105
  [1.101, 1.130] (win, fragile margin), many 0.605 [0.599, 0.610]; floor
  1.43 ms vs grep 1.68 ms. Attribution with profile, counters, and a
  same-provenance C syscall control: the dominant many-files loss is the
  host's per-open cost for unsigned local binaries (not a Whitefoot
  layer); the compute loss is the scalar double-walk shape (literal
  matcher 13/22 instr/byte over newline scan 9/22, model matches measured
  22.3), preregistered scan-trap suspect refuted as primary, confirmed
  secondary with a ~18% instruction-share ceiling. Canonical evidence:
  `research/experiments/wfgrep-baseline/RESULTS.md`. Ready for lead
  review and close to done.

## Validation, stop, and closure

Protocol committed before measurement; results (win, parity, or attributed
loss — all honest closures) recorded in
`research/experiments/wfgrep-baseline/RESULTS.md`; if attribution cannot
distinguish causes within the preregistered precision, that inability is the
recorded result. Unpiped gates. Close to done.
