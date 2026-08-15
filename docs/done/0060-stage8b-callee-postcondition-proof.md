# 0060 — Stage 8b callee postcondition proof and summaries

- **Status:** `DONE` (2026-08-15)
- **Authority:** the owner-approved ACTIVE Current Plan, Workstream 8b under
  Direction Outline item `PROOF-8`
- **Reviewed H2:**
  `239d952c714aef253a2be515580abc8fafa211d0`

## Outcome

H2 replaced post-hoc counterfactual rewalks with one structural C/U/B flow,
one view-tagged function-local derivation ledger and event stream, and one
finish/remap. It installed the two measured unsigned S7 sources, selected-exit
and aggregate proofs, entry-image invalidation, deterministic concrete-call
SCC scheduling, and atomic callee summaries. Same-SCC summaries, caller S12,
foreign derivation IDs, and a summary fixed point remain excluded.

Focused state, entailment, provenance, postcondition, root/metadata, formatting,
and warnings-denied lint validation passed; independent concentrated review
ended with P0/P1/P2 all zero. The owner-approved atomic v0.28 activation
containing this record installs H2 with the exact active specification.

Stage 9a is the next execution item. This file is frozen coordination history,
not current authority.
