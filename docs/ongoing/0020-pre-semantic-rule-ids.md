# 0020 — Rule ids for pre-semantic rejections

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 3
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** (executor fills at claim)

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
