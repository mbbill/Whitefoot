# 0026 — Check-aware wide-scan lowering slice

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 1
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `23bb7b06f2769611bf0859b6a3be5bf16e442384`

## Goal

Determine whether the compiler can lower the fused trap-carrying byte walk
to a wide stride with every required check preserved observably (exact trap
byte, exact DIAG-3 record): a pure lowering transform on the checked IR, a
compiler-derived obligation hoist provably preserving trap identity, or the
recorded conclusion that a PROOF-1-class verified fact family is required
first. Preregister expected code shape and falsifiers; a credited win reruns
the frozen baseline; a negative names the exact obstruction with a witness.

## Validation, stop, and closure

Facts-off correctness and required checks unchanged; trap identity
oracle-tested on hostile inputs (exact byte, exact record); §9.1 gates and
the wfgrep oracle hold; unpiped gates. A design question the checked-IR
semantics do not settle stops the task. Close to done with the rerun
baseline or the recorded negative.

## Current state (executor, 2026-08-06)

Complete on branch `task/0026-wide-scan`, awaiting lead review. Route (b)
credited under the frozen protocol
(`research/experiments/wide-scan-lowering/PROTOCOL.md`, frozen `3920e71`
before implementation): the compiler emits a check-aware 16-byte probe at
recognized byte-walk headers (`49e2663`), skipping only provably no-op
iterations; every trap stays in the unchanged scalar body. Measured
(RESULTS.md, run `wide-scan-lowering-1`): base/wide 1.431/1.428 material
on `large`/`nomatch` (`dense` 1.156, also material), product ratio vs the
pinned grep 0.753/0.762 → 1.069/1.071 (`dense` 1.346) — wfgrep now beats
the system grep on every compute-bound case. Trap-identity oracle
(`compiler/tests/programs/wide_scan.rs`) and recognizer decline tests
landed; both gates EXIT=0 unpiped. Head `b2b2acb`.
