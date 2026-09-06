# 0023 — Attributed-cause optimization slice: the scalar double-walk

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `73b11ca`/`a0a3491`, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 1

## Outcome

A credited win and the next attributed floor, under full preregistration
(protocol frozen before any comparative number; fresh same-protocol
baseline of current bytes reproduced 0022's 0.65). Three catalog-legal
shapes measured: S1 hoisted-first-byte 1.140/1.139; S2 fused single-pass
scan+match 1.150/1.145 — credited (preregistered code-shape delta present,
frozen rules cleared, byte-identical outputs, oracle and all ten §9.1 gates
green) and LANDED as `tests/programs/wfgrep.wf`; product ratio vs grep
0.65 → 0.753/0.762, dense 1.160 win. S3 SWAR regressed 0.896 with its
preregistered obstruction confirmed exactly (no wide loads form; all eight
byte loads survive behind distinct trap guards) — the minimal witness that
the one legal widening cannot lower to a wide step. Counter re-attribution:
both improved shapes saturate at ~3.8 cycles/byte — the residual 0.75 loss
is the serial per-byte step's latency/branch bound vs memchr's 16-byte SIMD
stride: a FLOOR/lowering question (check-aware wide scan), not a
source-shape question, and not the bounds traps (secondary untouched).
Honesty items recorded: two quantitative sub-predictions missed (the S2
wall miss IS the latency-floor finding); `many` precision-limited; two
drift episodes disclosed.

## Evidence and validation

- Canonical: `research/experiments/wfgrep-double-walk/` (PROTOCOL frozen at
  `ad9aa20`, RESULTS, three shape sources, SHA-pinned 2,100-sample raw).
  Landed commits `373ebac`..`a0a3491`. Both gates green by unpiped exit
  codes; `DECLARED_FUNCTIONS` gate re-derived from source per 0016's rule.
