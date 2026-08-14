# 0063 — Stage 8b real-consumer migration and behavior oracles

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-14, Workstream 8b. It is not yet claimed; claim only from the reviewed
held H4 recorded by task 0062.

- **Authority:** the ACTIVE Current Plan, Workstream 8b `verified
  normal-return postconditions`, under Direction Outline revision 38 item
  `PROOF-8`
- **Frozen candidate:** commit `7a293861ddb0aad0cd8d494ee8f4bfa5f6cbba29`,
  specification SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`
- **Execution premise:** the exact lead-reviewed H4 commit recorded by task
  0062; claim and integrate only onto H4

## Goal

Migrate the five frozen real sources through the one ordinary compiler path so
all fourteen `read_bits` and twenty `append_slice` rows derive their intended
facts, including the single wfgrep A10 repair, while preserving output, error,
cleanup, effect, status, required checks, and invalid-domain behavior.

## Frozen source map and method

- Touch exactly `tests/programs/{raw_deflate.wf,raw_deflate_dynamic.wf,
  raw_deflate_dynamic_decode.wf,raw_deflate_boundary.wf,wfgrep.wf}` plus the
  existing Rust program-oracle owners in
  `compiler/tests/programs/{raw_deflate.rs,wfgrep.rs}` and necessary focused
  ordinary semantic tests.
- Add `mask: own u64` to `read_bits`, remove its current body-local
  shift/subtraction mask construction, use the formal in the existing unsigned
  `iand`, and declare `ensures Ok(value: result) { check result <= mask; }` in
  the exact frozen surface. The existing count still governs bit consumption.
  Pass literal u64 masks for twelve fixed counts:
  1→1, 2→3, 3→7, 4→15, 5→31, and 7→127. For the two variable counts, compute
  caller-scope `high = ishl.wrap(1_u64,count)` and `mask = high -wrap 1_u64`
  before the child region so both supports survive. Keep all fourteen current
  direct match and immediate bare payload-to-outer `set` shapes.
- On both distinct `append_slice` declarations, compute capacity and the
  admitted `filled <= capacity` condition. Preserve an explicit initial
  branch: on the admitted edge, iterate the counted
  `for @append at in filled..capacity`, compute
  `taken = at -wrap filled`, return `at` when `taken >= len(text)`, otherwise
  copy `text[taken]` to `destination[at]`, and return capacity on exhaustion;
  on the non-admitted edge, return the original `filled` and write no
  destination byte. Add the matching admitted-domain requirement and plain
  result relation `result <= len(deref(destination))`. Complete and U must
  discharge; the invalid return is exactly B-refuted. Do not merge the two
  declaration identities.
- Repair A10 only after the host-copy child region. Keep `prior_length` and an
  outer copied count, form `candidate_length`, then use `value_if` to give the
  candidate on the proved bound or the prior length otherwise. Use the one
  `bounded_length` binding as receiver and `filled` actual for the separator
  and every later append, and as the final `publish_all` length. Never write
  back through old `length` or add variable-addition entailment.
- Commit reviewed H5 on H4, record it, and leave this task `WAITING` before
  0064 claims.

## Invariants and validation

- Replay the exact 34-row map: read `4+3+7=14`; append `8+12=20`; all intended
  relations discharge, R13/R14 join to the common `<128` result, and A11–A16
  discharge only after A10.
- Preserve every frozen real success/error/cleanup/effect/status and required
  runtime-check oracle. Facts-on/off behavior and emitted results remain
  identical; no new runtime fallback, check, trap, ABI, system, or lowering
  path is allowed.
- Add clause-stripped runtime controls for both append declarations. At
  capacity 3 / filled 4, both empty and nonempty text return 4 and leave bytes
  unchanged. The invalid selected exit is C/U discharged by the S4 false-edge
  contradiction and exactly B refuted; accepted callers publish only through
  same-view FN-8 proof.
- Focused semantic, raw-DEFLATE, and all nine wfgrep program tests pass, along
  with exact 14/20 root assertions and behavior differentials. Any preapproval
  full gate has only the known activation-chain stop and is not called green.

## Scope exclusions, stop, and done-when

No candidate spec/archive/ledger, protected conformance, approval ledger,
compiler semantic rule, generated frontend, lowering, runtime, ABI, runner,
adapter, Makefile, or gate-wiring change belongs here. Stop on source census
drift, an unresolved/refuted row, behavior drift, merged declaration identity,
need for another caller repair, general equality/arithmetic, recognizer,
runtime fallback, or any path beyond the exact five sources and owning tests.

The handoff is implementation-complete when reviewed H5 preserves every
behavior oracle and proves all 34 rows, and 0064's premise names H5. It remains
`WAITING` until atomic activation.
