# 0030 — compiler grammar-path extension for batch 1

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` selected slice items 1 and 3,
  and the owner's 2026-08-07 approval of
  `governance/spec-evolution/obligation-discharge-batch1-candidate.md`
  (`governance/APPROVALS.md`)
- **Owner / workspace:** exec-0030 /
  `/Users/bytedance/do_not_scan/wf-0030-worktree`, branch
  `task/0030-grammar-path-extension`
- **Base revision:** d459b49
- **Dependency:** none (candidate approved; this task gates v0.21 candidate
  generation per ruling O1)

## Goal

Extend the compiler's native grammar path so the batch-1 grammar delta
verifies: add the `claim_stmt` production (per the candidate §2), the two
tokens `claim` and `because`, and the `index_get` reserved-name row, through
the lexer, parser, and generated syntax data. Success criterion: the native
grammar verifier accepts the candidate grammar with 65 productions and 77
terminal predicates (the candidate §3's post-extension expectation), while
the approved v0.20 grammar continues to verify unchanged (64/74/75).
Grammar path only — no checker, entailment, or lowering semantics; those
are later tasks in this slice. `make -C compiler check` green before and
after.

## Notes

The verifier fail-closed run recorded in candidate §3 is the reproduction
of the current blocker. A discovery outside the candidate's grammar delta
(e.g. an LL(2) conflict the candidate did not predict) stops the task and
is reported with evidence, never absorbed.
