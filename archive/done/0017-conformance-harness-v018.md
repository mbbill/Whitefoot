# 0017 — Conformance harness v0.18 lane

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `0b3ecde`, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2
  (split from task 0014's bullet)

## Outcome

The corpus now expresses and checks the v0.18 first slice ahead of the
compiler chain. Manifest schema gained a closed `arrange` object (hex argv
bytes, hex stdin, file/directory fixtures, opaque-sink redirection) validated
by `runner.py`; `unsupported` landed as a first-class expectation kind
matched on verdict kind only, reserved for QUAL-1 qualification-failure and
QUAL-2 startup-refusal spec-level stops. The corpus pin moved to
`spec/kernel-spec-v0.18.md` with coverage 119/119 (90 by case, 30 by
annotation) and a regression pinning that `pending` cases count. 24 additive
compile-time cases landed (entry/kind/visibility, reserved kinds, label
near-misses, effect rows and release attribution): 4 runnable and verified
against the compiler today, 20 `pending` with reasons naming their gating
task (0007/0008/0009), each verified through lexical, grammar, and FORM-2
stages as a real syntax oracle.

A registered-record defect was escalated and ruled mid-task (Option B): the
record had directed compiler-gated constructs to carry `expect: unsupported`,
which would have scheduled ~20 protected-expectation edits when the compiler
tasks land; the ruling keeps `expect` as the spec verdict and puts toolchain
readiness in `status`, so later tasks flip only `pending` → runnable.

## Evidence and validation

- Landed commits: `d141e3c` (claim), `0b3ecde` (implementation).
- `make check` green on main after landing with the v0.18 coverage line;
  runner self-tests 18 pass; existing manifest entries and case sources
  byte-untouched (+37/-0 manifest diff).
- Catalog-vs-spec divergences resolved in the spec's favor and recorded in
  the case docs (SYS-1 collision → reject citing TYPE-6 at DIAG-1 rank 5;
  ExitStatus-from-u8 → FN-1; one catalog case ID renamed).

## Follow-ups

- Task 0014 consumes the schema for the runtime lane; tasks 0007/0008/0009
  flip their named `pending` cases as they land.
- Owner-level note (protected surface, untouched): pre-existing
  `own13-pos-borrow-match-live.wf` is an `accept` case with no `fn main`,
  which FN-7 rejects as a complete unit; latent until an execution adapter
  runs it.
- Two effect cases rest on the EX-1-consistent reading that a borrow of a
  current-function own root contributes no region effect; compiler-checkable
  once 0007 lands.
