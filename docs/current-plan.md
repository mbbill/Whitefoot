# Current Plan

Status: ACTIVE — the owner approved this plan on 2026-08-05; specification
bytes still require exact approval under the specification-change workflow

Derived from: [Direction Outline revision 7](roadmap.md), items `CAND-8`,
`BOUND-1`, `PERF-1`, `VERIFY-1`, and `VERIFY-2`

## Goal

Turn the selected system-capability architecture into language and compiler
reality: activate one v0.18 specification batch containing exactly the
dossier's first-command-slice deltas, implement that slice on the normal
compiler path for macOS/Linux, and return to the frozen sequential `wfgrep`
checkpoint with its correctness oracle and cost gates.

Per the owner's 2026-08-05 framing, the deliverable is what this slice proves
about the language: every dossier §9.1 cost gate observed, every blocker
classified and honestly reported, negative results retained. Completing
`wfgrep` is the pressure source, not the completion condition.

## Authority and evidence

- Selected architecture (owner, 2026-08-05, including Route C for the
  declaration home with the recorded fallback to a prelude extension):
  [dossier](../research/investigations/system-capability-architecture/DOSSIER.md)
  and its
  [31-issue review record](../research/investigations/system-capability-architecture/decisions.json).
- The specification-change workflow in [WORKFLOW.md](WORKFLOW.md) governs Work
  item 1; the owner's exact byte approval of the candidate is still required
  there and is not granted by activating this plan.
- The loan/freeze candidate vacated the v0.18 slot on 2026-08-05 and remains
  parked evidence under `STORE-1`.

## Work

1. **v0.18 specification batch (sequential; specification-change workflow).**
   Draft `governance/spec-evolution/kernel-spec-v0.18-candidate.md` from v0.17
   with exactly the dossier §11/§11.1 inventory: the command entry form with
   exact standard input labels (the unlabelled `fn main` entry remains
   admissible); the seven fixed opaque types, the operation set including the
   raw-byte pair, and the complete outcome inventory (`Result` instantiations
   plus `ReadOutcome`); `external` and `blocks` effect categories with the
   EFF-1 row-grammar, EFF-2 attribution, FN-3 normalization, and STOR-3
   release extensions scoped to the new resource families; the Route C
   system-declaration domain (TYPE-6 three-row extension, OP-1, PROG-1, the
   new DIAG-1 rank and origin kind, and the syntactic program-kind visibility
   trigger); portable `IoError` classes; path and host-string rules with the
   command-lifetime backing guarantee in target qualification; and first-slice
   conformance expectations. Verify grammar with the native verifier, obtain
   exact owner approval, and activate atomically with every derived artifact.
2. **Decompose implementation into `docs/planned/` upon activation**, one
   independently integrable numbered task each with explicit dependencies:
   compiler front-end (system-declaration domain, opaque types, entry form),
   effect checking extensions and release attribution, checked-IR resource
   identities and cleanup, target-qualification table plus the static native
   macOS/Linux lowering, the deterministic test implementation, first-slice
   conformance execution, the sequential `wfgrep` program, and the §9.1 cost
   and §12.2 hostile test gates.
3. **Fan-out execution under the executor lane in WORKFLOW.md.** Executor
   agents claim planned tasks, implement in isolated worktrees, and land only
   through lead review. Blockers and plan defects stop the task and are
   reported as findings; no workaround closes a slice.
4. **Return to the `wfgrep` checkpoint.** Run the frozen sequential slice's
   correctness oracle and its scoped cost-shape gates; attribute any material
   loss per `PERF-1` before widening the project.

## Verification

- The candidate's semantic delta equals the dossier inventory — no additional
  capability rides along, and no listed delta is silently dropped.
- `make -C compiler check` and `make check` green before and after each landed
  task; first-slice conformance cases pass through the normal command path.
- The dossier §12.2 first-slice test list is implemented, including non-text
  arguments, invalid ranges, short reads/writes, broken pipes, redirection to
  one sink, close-error behavior, and the effect-attribution canonical case.
- The §9.1 native cost shape is inspected on emitted code: no allocation,
  copy, dispatch, handle lookup, or lock on the hot paths; the buffer gates
  use their two distinct controls.
- Executor escalations are reviewed against the blocker routing; any language
  gap found here enters the outline rather than being absorbed.

## Done when

- v0.18 is active with every derived artifact updated in the same change;
- the compiler compiles and runs the sequential `wfgrep` slice on macOS/Linux
  through the normal path, passing its correctness oracle;
- the §9.1/§12.2 gates are observed with evidence recorded in their canonical
  homes; and
- the outline and this plan are replaced to name the next slice or blocker.

## Not in this stage

- No directory traversal, ignore stack, parallel search, networking, clocks,
  randomness, async/wait, threads, child processes, buffered output
  publisher, or general FFI.
- No optimizer fact consumers; `PROOF-*` items enter only on an observed,
  attributed hotspot after the slice is correctness-green.
- No ripgrep timing claims; the 2x comparison waits for a preregistered suite
  on a wider slice.

## Parallel research

None proposed. The scan-floor and literal-line research results remain
standing evidence for the later performance gates.
