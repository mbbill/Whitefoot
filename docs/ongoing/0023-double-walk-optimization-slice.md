# 0023 — Attributed-cause optimization slice: the scalar double-walk

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 1 (PERF-1, one
  attributed cause)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `7240f84`

## Goal

Close or characterize task 0022's attributed primary: the fused single-pass
scan+match source shape (and any other catalog-legal shape) against the
frozen baseline corpus, preregistered with an expected code-shape
consequence and a falsifier before any measurement. A credited win needs the
preregistered binary delta plus the ratio clearing the frozen rules with
byte-identical work; a demonstrated inability of every legal shape to reach
a materially faster form is a full success recorded as a FLOOR/lowering
finding with a minimal witness. The bounds-trap secondary is out of scope
unless the primary closes and the residual re-attributes to it.

## Progress

- Claimed at base `7240f84`. The current `tests/programs/wfgrep.wf` bytes
  (task 0021's helper-decomposed form) are SHA-256
  `7c7833906e9b8bf512eac3499e30bda50e49ecffd971650a8e15c036be137595`;
  they differ from the bytes the 0022 baseline froze, so this slice first
  takes a fresh same-protocol baseline of the current bytes, then measures
  every candidate shape against that baseline.
- Preregistered `research/experiments/wfgrep-double-walk/PROTOCOL.md`
  (three shapes: hoisted first byte, fused scan+match, SWAR word scan;
  expected consequences and falsifiers; the 0022 corpus/statistics
  inherited) and froze it before any comparative number; pre-freeze
  correctness-only development disclosed there.
- Ran `wfgrep-double-walk-1` complete (AC power, all null gates but one
  passed; `many` degraded to w=2.22% and is precision-limited). Fresh
  current-bytes baseline reproduced 0022 (0.650/0.657 material loss on
  the scan cases). S1 1.140/1.139, S2 1.150/1.145 — both material
  improvements vs the fresh baseline; S3 0.896/0.899 regression with its
  preregistered obstruction witness confirmed (no wide load forms; each
  index keeps its own trap-guarded check). Quantitative sub-predictions
  that missed are recorded as such in RESULTS.md.
- Credited and landed S2 as `tests/programs/wfgrep.wf`; re-derived
  `DECLARED_FUNCTIONS` in the §9.1 gate from the new source (drops
  `line_matches`); nine-case oracle, ten gates, `make -C compiler check`,
  and `make check` all green, unpiped (exit 0).
- Rerun-baseline deliverable: the conditional confirm phase measured the
  landed shape against grep — 0.753/0.762 on the scan cases (from
  0.650/0.657), `dense` now a material win 1.160. Residual re-attributed
  to the serial per-byte step against SIMD striding (~3.8 cycles/byte
  latency floor; instruction removal saturates), a FLOOR/lowering
  finding, NOT re-attributed to the bounds traps — the secondary stays
  untouched per the plan boundary.
- Canonical evidence: `research/experiments/wfgrep-double-walk/RESULTS.md`
  and `raw/wfgrep-double-walk-1.jsonl` (SHA-256 `912ed5ee…`). Ready for
  lead review and terminal disposition.

## Validation, stop, and closure

§9.1 gates and the nine-case oracle hold on every accepted shape; facts-off
behavior and required checks unchanged; same-source causal ablation before
mechanism credit; unpiped gates. Close to done with the rerun baseline
either way.
