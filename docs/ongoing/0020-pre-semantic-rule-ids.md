# 0020 — Rule ids for pre-semantic rejections

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 3
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `d84643b`

## Goal

Populate `CompilationFailure::rule_id` at the Lexing, CanonicalSource,
Parsing, and Resolution stops (45 corpus cases arrive as reject-without-rule)
so the conformance adapter can compare cited rules at every stage; the
diagnostics already know their rules (DIAG-1 attribution) — this is
plumbing, not new judgment. No diagnostic text or verdict changes.

## Validation, stop, and closure

The 45 cases' rejections carry their expected rule ids through the adapter;
no other case's outcome moves; unpiped gates. Close to done with the lane
delta.

## Progress

Implemented. Each pre-semantic stage's already-selected rule is published
through `CompilationFailure::rule_id`: `SourceIssueKind::rule_id`
(DIAG-1's raw-lexical clause set), `TerminalIssueOwner::id`,
`SyntaxRule::id` (parsing and the FORM-2 canonical audit), and
`ResolutionRule::id`. One private `CompilationFailure::source` constructor
now serves all six source stops, so no stage can publish a rejection
without its rule. No diagnostic text, location, verdict, or judgment
changed; the semantic stage's attribution is unchanged and now shares the
same constructor. Terminal classification was included because it is the
remaining pre-semantic source stop and already carried its owner; it moves
no corpus case.

Lane delta measured on this branch, before and after the change (base
`d84643b`; tasks 0019 and 0021 move the same tally concurrently):
`Pass=242 Fail=123 Skip=14` → `Pass=276 Fail=89 Skip=14`. The moved set is
exactly the 45 reject-without-rule cases (Lexing 3, CanonicalSource 1,
Parsing 17, Resolution 24): 34 now pass, 11 still fail, and no other case
moved in either direction.

Findings for routing, not edit targets here — the 11 are cases whose real
attribution divergence the plumbing now makes visible:

- `op2-neg-div-wrap`, `op7-neg-missing-prefix`, `op8-neg-rotate-trap` want
  OP-2/OP-7/OP-8; the unknown OPNAME is rejected at OP-1 family lookup.
- `form3-neg-typeid-fn-name`, `x-form-form3-enum-name-ident` want FORM-3;
  the parser attributes GRAM-2.
- `x-eff-dup-reads-effect` wants EFF-1; the duplicated `reads('r)` is
  rejected at OWN-3 as an unresolved region.
- `x-enum-option-context-free-constructor`, `own1-neg-match-move-through-borrow`,
  `own5-neg-match-borrow-affine-payload-move`, and the two positives
  `own13-pos-uniq-match-payloads`, `own1-pos-match-copy-payload-reuse` all
  hit GRAM-10 match-binder freshness. `resolution/engine/inventory.rs`
  rejects a binder spelled the same as its paired field
  (`declaration.spelling == paired_field.spelling`) with no earlier binder
  and no arm-entry conflict; whether GRAM-10 requires that is a spec
  question outside this task.

The adapter's `#[ignore]` reason still states 123 failures in four causes.
Its bucket 1 is now resolved and its count is stale, but 0019 and 0021 move
the same number, so it is left for the lead to reconcile once all three
land rather than rewritten to a value that is wrong on the integration
branch.
