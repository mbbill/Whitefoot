# 0073 — Generics with regions, and the container they carry

Owner: lead. Workspace: `batch-0073` worktree. Base: main at the 0072 merge.
Registered: 2026-08-19 under the ACTIVE Current Plan (W1, W2, W3, W5).

## Authority

The ACTIVE `docs/current-plan.md` (owner direction 2026-08-19). Any
specification bytes this batch produces are marked candidates awaiting
the owner's exact-byte approval; nothing activates on this branch.

## Scope

- W1: design and implement generics with region parameters, closing the
  capability stop at `compiler/src/semantic/check/generics.rs:198`.
- W2: the growable generic container over [SET-2].
- W3: `wfgrep`'s entry collection, with the 64-entry and depth-16 bounds
  removed rather than raised, and completeness pinned on a four-thousand
  entry directory.
- W5: adversarial exit audit and the owner packet.

W4 (reuse form) lands nothing unless W3 produces evidence that the
existing multi-file compilation unit fails a real consumer.

## Out of scope

The `requires`/`ensures` redesign; the arithmetic-trap audit; any
module or import mechanism; activation of any candidate; merging to main.
