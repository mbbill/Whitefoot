# WF-SCAN-FLOOR

This is a temporary coordination record, not execution authority. Delete it
when the task is integrated, parked, replaced, or abandoned.

Status: ACTIVE — preregistration and current-compiler baseline

Authority: separate owner-approved bounded parallel research on 2026-08-05.
The owner approved `WF-SCAN-FLOOR` after reviewing its boundary: measure pure
in-memory byte scanning and line accounting through the current Whitefoot
compiler, compare equivalent Rust and C algorithms, attribute any material
gap, make no specification or compiler change, and update MCTS-Mem only with
carefully established design evidence.

Outline decisions this evidence may change: `PERF-1`, `FLOOR-1`, and
`FLOOR-2` in Direction Outline revision 6. A material retained-check finding
may inform a later `PROOF-1` proposal but cannot authorize it.

Base revision: `03fc7b2`

Workspace: `codex/wf-scan-floor` in
`/Users/bytedance/.codex/worktrees/73c2/Whitefoot`

## Goal

Determine whether current ordinary Whitefoot source can express two scanner
shapes at competitive same-algorithm machine quality: a full-pass byte
classifier with line accounting and an early-exit byte search. Separate source
shape, required checks, compiler lowering, LLVM recovery, final machine code,
and measurement noise before drawing a conclusion.

## Direction and invariants

- Use only active v0.17 operations already implemented by the normal compiler
  path: buffers, checked indexing, loops, Boolean dataflow, and ordinary
  integer operations.
- Compare equivalent work and algorithmic shape in Whitefoot, safe Rust, and C;
  optimized library `memchr` is a separately labeled ceiling, not the causal
  same-algorithm control.
- Preserve every required check. Facts-off behavior is the only Whitefoot path
  measured in this task.
- Bind every timed result to a correctness check and inspect optimized IR or
  final assembly before assigning a cause.
- A single scanner result is not a wfgrep product result and cannot support a
  2x-ripgrep claim.

## Method

Preregister exact source, deterministic inputs, repetitions, correctness
oracle, target/build identities, statistic, materiality bands, code-shape
observables, and stop rules. Implement one small self-contained experiment
bundle under `research/experiments/`. Run correctness first, then compile all
variants, inspect Whitefoot emitted LLVM and optimized final assembly, and only
then run paired timing. Investigate anomalous order, cache, optimizer, or setup
effects rather than appending favorable samples.

## Progress

Completed:

- owner authorization and conflict review against the in-flight
  system-capability architecture task;
- refreshed this workspace to base revision `03fc7b2`; and
- loaded the mandatory MCTS-Mem workflow;
- built both current-compiler Whitefoot kernels and equivalent C/safe-Rust
  controls behind the frozen harness boundary;
- passed all six small-case correctness checks; and
- inspected pre-timing IR and assembly: required raw bounds traps are removed
  by LLVM, the full Whitefoot/C loops share the same width-16 vector structure,
  and the early Whitefoot/C loops share the same scalar structure.

Current: freeze the complete protocol and sources in Git before observing any
comparative current-compiler timing.

## Scope and expected touch set

Primary expected paths:

- `research/experiments/wfgrep-scan-floor/`
- the relevant existing node or graduated fact under `mcts_mem/whitefoot/`,
  only if the completed evidence passes the MCTS admission test
- this coordination record

No numbered specification, compiler source, conformance expectation,
`docs/current-plan.md`, system-capability dossier, resource/effect/provider
model, runtime, regex engine, filesystem path, or wfgrep product source is in
scope. `docs/roadmap.md` changes only if the completed result materially changes
an already-authorized outline status; otherwise the experiment result remains
linked evidence for later integration review.

## Dependencies and integration order

This task is semantically independent of
`docs/ongoing/system-capability-architecture.md`: it has no system operation,
resource, external-effect, provider, command-entry, thread, or cancellation
semantics. Either result may be integrated first. Before integration, refresh
the base and preserve the system task's authoritative closure changes.

If the experiment exposes a desired specification, compiler, proof, intrinsic,
or strategy-lowering change, stop and return that attributed finding to the
owner. Such a change requires a later Current Plan or separate authorization.

## Validation, stop, and closure

Validation requires an independent byte oracle, identical work accounting,
optimized-code inspection, paired measurements with the frozen uncertainty
rule, hostile review for dead-code elimination and setup dominance, MCTS lint
after any tree edit, the experiment-local gate, and `make check` before final
integration.

Stop after classifying both scanner shapes as parity, material source-floor
gap, required-check pressure, compiler/LLVM lowering gap, algorithmic ceiling,
or inconclusive. Do not grow into substring search, regex, Unicode, traversal,
I/O, output publication, or parallel execution.

Close by recording the exact positive, negative, or inconclusive result in the
experiment bundle, updating only admitted MCTS evidence, completing review and
gates, and deleting this file in the same integration change. If parked or
abandoned, record that disposition and delete this file without converting
partial work into a conclusion.
