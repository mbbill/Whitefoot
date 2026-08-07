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
