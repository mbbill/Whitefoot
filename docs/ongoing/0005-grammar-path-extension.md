# Native grammar-path extension for the v0.18 candidate

This is a temporary live coordination record, not execution authority. Move
this same numbered record to `docs/done/` at terminal disposition.

- **Status:** `IN PROGRESS`
- **Authority:** `ACTIVE` [`docs/current-plan.md`](../current-plan.md) Work
  item 1, whose verifier step requires it: `whitefoot-grammar` fails closed on
  structural grammar changes until the compiler's own native path is extended
  (`compiler/README.md`)
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `7cc6302`

## Goal

Make `cargo run --manifest-path compiler/Cargo.toml --bin whitefoot-grammar --
governance/spec-evolution/kernel-spec-v0.18-candidate.md` verify the candidate,
while the production compile path remains byte-for-byte a v0.17 implementation.

## Direction and invariants

- Grammar deltas to support: `program_kind?` on `fn_decl`, `input_label?` on
  `param` with terminal spelling `as`; `external` and `blocks` in the EFF-1
  effect production with canonical order reads, writes, allocates, external,
  blocks, traps; strong-LL(2) discipline preserved.
- v0.17 acceptance must not change: every currently accepted program stays
  accepted with identical behavior, every currently rejected program stays
  rejected. If grammar-level and semantic-level rejection would swap for any
  protected conformance case, stop and report — do not edit any verdict.
- No `unsafe`; one normal path; a tool-scoped mechanism (for example staged
  grammar tables selected by the verifier) is acceptable if the production
  path's behavior is untouched — state the mechanism in the record.

## Method

Read `compiler/README.md` and the `whitefoot-grammar` binary to learn the
intended extension route, extend lexer/terminal inventory, parser, LL(2)
decision tables, and generated syntax data, then run the verifier against the
candidate and the full gate.

## Progress

- **Done:** task registered.
- **Current:** executor claimed; implementation in worktree.
- **Next:** verifier green on the candidate; `make -C compiler check` and
  `make check` green; lead review; land.

## Scope and expected touch set

- Primary: `compiler/src/syntax/`, generated syntax data, the
  `whitefoot-grammar` binary, focused compiler tests.
- Excluded: semantic checking rules, conformance verdicts, `spec/`,
  the candidate's bytes, lowering/backend.

## Dependencies and integration order

- **Prerequisites:** the integrated candidate at `7cc6302` (this base).
- The 0004 approval packet depends on this task's verifier result.

## Validation, stop, and closure

- **Validate:** verifier passes on the candidate; both gates green; a spot
  check that a `program_kind` entry and an `external` row still reject under
  v0.17 semantics on the production path.
- **Stop:** any needed change to semantic rules, protected expectations, or
  the candidate's grammar text stops the task for lead review.
- **Close:** land through lead review; move this record to `docs/done/`.
