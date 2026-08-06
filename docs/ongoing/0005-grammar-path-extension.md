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

**Mechanism (implemented): staged grammar tables selected by contract
identity.** The compiler carries two committed table sets. The active v0.17
set (`generated.rs`, terminal inventory) is byte-untouched except two new
`Production` enum variants that the active tables never reference. A new
staged set (`compiler/src/syntax/grammar/staged.rs`) describes the complete
v0.18 candidate grammar: 64 productions, 67 fixed terminals (+`as`,
`external`, `blocks`), 74 strong-LL(2) decisions, 1925 SELECT_2 rows. A
committed snapshot of the candidate's three frontend-contract sections
(`staged_frontend.md`) pins the exact contract those tables describe;
`STAGED_SYNTAX_CONTRACT_HASH` (its SHA-256) is the selection token. The one
shared lexer/classifier/parser engine selects tables by the contract identity
the caller names: the production driver always passes the active hash and is
behaviorally unchanged; only the verifier passes the staged hash. `finalize`
fails closed on staged derivations (no v0.18 FORM-2 semantics exist). The
verifier accepts a candidate whose contract byte-equals either the active
spec's sections (grammar-preserving path, unchanged) or the staged snapshot
(then checks the staged inventory, decision coverage, and cross-arm
disjointness, and runs the real lexer, classifier, and parser over the
unlabelled entry and the candidate's canonical four-input command-entry
header); anything else fails closed. The staged tables were produced offline
by a one-shot generator validated by reproducing the committed v0.17 tables
exactly — structure byte-identical, all 72 decisions' 1839 SELECT rows and
atom metadata set-identical — before emitting the v0.18 set; the generator
lives outside the repository per hygiene rules. For the activation task: that
generator currently sits at `/Users/bytedance/do_not_scan/wf-grammar-gen` on
the owner's host and reproduced the committed v0.17 tables exactly, so the
activation work can decide whether to wire it in or regenerate independently.

## Progress

- **Done:** task registered; executor implementation complete in worktree:
  verifier verifies the candidate through the staged path
  (`staged candidate contract verified ... 64 productions, 74 decisions, 75
  terminal predicates`) and still verifies the active spec unchanged;
  `make -C compiler check` and `make check` green before and after; focused
  tests added (staged parse of the kind-declaring entry and effect rows,
  staged reservation of the three new spellings, active-path spot checks that
  a `program_kind` entry and an `external` row still reject and that `as`,
  `external`, `blocks` remain active identifiers, staged-contract fail-closed
  near-misses, finalize fail-closed).
- **Done (rebase):** rebased onto `85c0f5c` (hostile-review candidate fixes)
  and re-extended the staged contract snapshot and its hash; the amendments to
  the frontend sections are prose-only (GRAM-11 wording; one EFF-1 sentence),
  so the staged grammar tables are unchanged and the verifier reports the same
  64/74/75 counts.
- **Current:** awaiting lead review of the worktree branch.
- **Next:** lead review; land; move this record to `docs/done/`.

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
