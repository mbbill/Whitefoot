# Current Plan

Status: PROPOSED — awaiting owner selection; a proposed plan authorizes no
execution. The previous ACTIVE plan's milestone (baseline and consolidation,
tasks 0019-0022) completed on 2026-08-06.

Derived from: [Direction Outline revision 13](roadmap.md), items `CAND-8`,
`PERF-1`, `FLOOR-1`, `STORE-1`, and `VERIFY-2`

## Goal

Close the measured compute gap's first attributed cause and the largest
known capability gap, on the same evidence discipline: one preregistered
optimization slice against the scalar double-walk shape that task 0022
attributed as primary, and the general borrow-mode parameter capability
that 44 conformance cases and STORE-1 wait on. The deliverable remains
knowledge: either the gap closes through a legal source shape and ordinary
lowering, or the exact obstruction becomes a named language/lowering
finding with a witness.

## Work

1. **One attributed-cause optimization slice (PERF-1).** Preregister the
   expected code-shape consequence and a falsifier, then attempt the fused
   single-pass scan+match source shape (and any other legal shape the
   catalog admits) for `wfgrep`'s inner loops against the frozen baseline
   corpus. Credit requires the preregistered binary delta plus the measured
   ratio clearing the frozen rules; if NO legal shape reaches a vectorized
   or materially faster form, the obstruction is recorded as a FLOOR/
   lowering finding with a minimal witness — that negative is a full
   success for the probe. The bounds-trap secondary (~18% ceiling) is
   touched only if the primary closes and the residual is re-attributed.
2. **General borrow-mode parameters and let-borrows** of scalar and enum
   types (unsupported specified capability; task-0021 finding): implement
   on the normal path; the 44 waiting conformance cases flip runnable and
   must pass; wfgrep is untouched.
3. **Attribution-divergence investigations** (the 15 recorded at
   0019/0020): each to a compiler-defect fix with regression or a
   protected-expectation finding returned to the owner; plus execution of
   the gram5 one-token protected amendment on the owner's ruling.
4. **Return and replace**: rerun the frozen baseline after item 1, record
   results, update the outline, and replace this plan naming the next
   attributed cause, the traversal-widening proposal, or a park.

## Verification

- Item 1 is one cause, one slice: byte-identical frozen work, preregistered
  falsifier, same-source causal ablation before any mechanism credit;
  facts-off behavior and every required check unchanged; §9.1 gates and the
  oracle hold on every accepted shape.
- Item 2 changes no specification byte (v0.19 already admits the modes);
  gates green throughout; the 44 flips carry per-case run evidence.
- Item 3 touches protected material only per explicit owner rulings.

## Done when

- the optimization slice has a credited win or a recorded negative with its
  witness, and the baseline is rerun either way;
- the general borrow capability lands with the 44 cases green;
- the divergences are each fixed-with-regression or returned to the owner,
  and the corpus lane's remaining red (if any) is entirely owner-ruled; and
- this plan is replaced naming the next slice or blocker.

## Not in this stage

- No directory traversal, parallelism, new system families, or STORE-2
  growth mechanism; no specification change; no PROOF-1 implementation
  unless item 1's residual re-attribution lands on the traps.

## Parallel research

None proposed.
