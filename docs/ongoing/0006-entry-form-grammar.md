# 0006 — Entry-form grammar productions and kind-declaring predicate

Live coordination record. It reports how authorized work is being carried
out; it is not authority and cannot expand or resequence work.

- **Status:** IN PROGRESS
- **Owner / workspace:** executor agent / isolated worktree, lead-reviewed
- **Base revision:** `e648713`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, first bullet
  ("compiler front-end ... entry form"), derived from Outline revision 7
  (`BOUND-1`, `CAND-8`). Implements `spec/kernel-spec-v0.18.md`'s `GRAM-2`
  grammar delta (new `program_kind` and `input_label` productions on
  `fn_decl`/`param`) and `FN-7`'s kind-declaring trigger, per dossier §11.
  Claimed under Work item 3's executor fan-out while
  `docs/current-plan.md` remains `ACTIVE`.

## Goal

Close out the entry-form grammar layer that the v0.18 activation commit
(`9768bae`) already landed, and leave the kind-declaring predicate exposed in
a shape tasks 0007/0008 can consume as a positive admission input. Activation
promoted the two productions into the active tables, parses and finalizes
both entry shapes, computes the syntactic kind-declaring predicate before
scope building, and gates kind-declaring units as explicit unsupported
capability (`ResolutionOutcome::Unsupported`). This task audits that landing
against the checklist below, fills any gap it finds — the expected residue is
FORM-1/FORM-2 canonical byte-format and reject coverage for the new
constructs plus parser-diagnostic quality at the new decisions — and repoints
the predicate's surface from the unsupported gate to an accessor 0007 builds
on.

## Direction and invariants

- Implement exactly `spec/kernel-spec-v0.18.md`'s `GRAM-2` grammar bytes for
  `program_kind` / `input_label` / the amended `fn_decl` and `param`
  productions — no invented punctuation. The fixed `"as"` atom is
  IDENT-ineligible under FORM-3 (measured cost in the current corpus is
  zero: every existing bare `as` occurs inside a `doc` STRING interior).
- The "kind-declaring" judgment must be syntactic and total, decided
  strictly before declaration inventory, and must not consult resolved
  names, types, effect rows, or even the kind IDENT's own validity — only
  whether a `program_kind` child is present on some top-level `fn_decl`.
  This is the dossier §11.1 circularity argument: diagnostics admit names
  before resolution, so keying visibility on resolved types would be
  circular.
- The unlabelled `fn main() -> own unit` form must remain byte-for-byte
  accepted with its unchanged four legal effect rows; no existing
  conformance verdict may change.
- Out of scope: this task does not make `Args`/`ExitStatus`/etc. resolvable
  (task 0007), and does not touch FN-7's signature/effect admission (task
  0008). A program using the new grammar will still fail resolution
  (unknown name) until 0007/0008 land — expected, and must not be
  misreported as a grammar rejection.

## Method

Audit the activation landing against the Direction bullets: canonical
FORM-1/FORM-2 byte-format acceptance and reject tests for `program_kind` and
`input_label` (rendering reuses the existing attachment sets; recognition
must be covered), reject coverage for malformed label shapes at the grammar
level, parser-diagnostic quality at the two new LL(2) decisions, and a
`program_kind` on a non-`main` declaration (parses; FN-7 admission in task
0008 owns the rejection). Then expose the kind-declaring predicate as a
stable accessor consumed by 0007/0008 instead of being derivable only from
the unsupported gate's control flow, keeping the stage order (FN-8 admission
→ kind-declaring decision → declaration inventory → lexical resolution)
unchanged.

**Already landed by activation (`9768bae`), verified by the lead:** the
grammar tables (64 productions, `as`/`external`/`blocks` reserved), the
parse/finalize path over both entry shapes (the all-production fixture
carries a kind entry with two labelled inputs), the syntactic kind-declaring
predicate in the resolution engine, the table-checked carrier classification
(`RawRoleKind::TableChecked`), and the three unsupported gates. Do not
re-implement any of it; audit it against this record's Direction bullets and
Validation list, and fill only what is genuinely missing.

## Progress

- Completed: claimed at base `e648713`; refreshed the integration branch and
  read `docs/WORKFLOW.md`, `docs/current-plan.md`, the cited
  `spec/kernel-spec-v0.18.md` rules, and `mcts_mem/whitefoot/system-interface*`.
- Completed: audit of the activation landing. Confirmed already complete —
  the promoted grammar tables and the two productions, the parse/finalize
  path over both entry shapes, `as`/`external`/`blocks` reserved from IDENT,
  the `RawRoleKind::TableChecked` carrier classification, the three
  unsupported gates, and FORM-2 rendering of the entry header with no
  renderer amendment. One defect found (below); the rest of the residue was
  missing evidence rather than missing behavior.
- Completed: gap fill. The exact FN-7 canonical command-entry header now
  passes the FORM-2 audit byte for byte, and the existing trivia-mutation
  sweep (extracted to one helper) proves no other trivia spelling of it
  renders. Added grammar reject coverage for five malformed `input_label`
  shapes at their exact boundary and expected predicate, the complete
  expected sets at both new LL(2) decisions (`program_kind?` reports DIAG-1
  attribution row 4 at the IDENT expecting `fn`; `input_label?` expects `.`
  and `:`), and the non-entry `program_kind` / non-entry `input_label` parse
  cases.
- Completed: defect fix. `check_system_declaration_support` ran *before* the
  FN-8 requires-block admission pass, so in a kind-declaring unit an FN-8
  hard error was masked by the unsupported stop. DIAG-1 fixes the order —
  "only complete FN-8 admission permits the [SYS-3] system-admission
  decision, only that decision permits declaration inventory". Moved the
  decision after `check_requires_blocks` with a regression on both sides.
- Completed: accessor. The judgment now has one home,
  `compiler/src/syntax/entry_form.rs`, exposing crate-internal
  `unit_program_kind(&FinalizedTopology) -> Option<NodeId>`: `Some` is the
  FN-7 judgment and carries the DIAG-1 `SourceNode`, `None` is its negation.
  Resolution's SYS-3 gate reads it instead of rederiving a local scan.
- Current: awaiting lead review; `make -C compiler check` and `make check`
  green, native grammar verifier green on the active spec (64/74/75).
- Next (lead): review and land, then close this record into `docs/done/`.

## Scope and expected touch set

- `compiler/src/syntax/grammar.rs`, and wherever the generated LL(2)
  decision tables live — two new productions, one new terminal spelling for
  `"as"` (verify against post-activation state per the Method note above).
- `compiler/src/syntax/parser/*`, `compiler/src/syntax/parser/finalize/*`
  (topology, shape, canonical) — parse and finalize the new node shapes.
- `compiler/src/resolution/engine/admission.rs` (or a new sibling module) —
  the pre-inventory kind-declaring predicate.
- `compiler/src/syntax/tests.rs`,
  `compiler/src/syntax/parser/finalize/tests/*` — new parser/finalize unit
  tests, independent of full compilation.
- Read-only: `spec/kernel-spec-v0.18.md`, `mcts_mem/whitefoot/system-interface*`.

Actual touch set (rebase warning for concurrent workspaces): new
`compiler/src/syntax/entry_form.rs`; `compiler/src/syntax/mod.rs`;
`compiler/src/resolution/engine.rs`; and tests in
`compiler/src/resolution/tests.rs`, `compiler/src/syntax/parser/tests.rs`,
`compiler/src/syntax/parser/finalize/tests/canonical.rs`, plus
`compiler/README.md`. The grammar tables and the parse/finalize path were
already correct and are untouched. The accessor went to `syntax` rather than
`resolution/engine/admission.rs` because 0008 reads it from semantic checking,
which cannot reach the resolution engine's private modules.

## Dependencies and integration order

None beyond v0.18 activation (Work item 1). Tasks 0007 and 0008 both depend
on this task's terminal/production shapes and the kind-declaring predicate's
surface being stable before they consume it.

- `docs/ongoing/0007-system-declaration-domain.md` is live concurrently under
  an explicit lead-authorized semantic overlap. It consumes this task's
  kind-declaring accessor as its `SYS-3` admission trigger. Integration order:
  this task lands first; 0007 refreshes, rebases onto it, and reruns its
  gates.

For 0007: `crate::syntax::unit_program_kind(topology)` is the kind-declaring
flag; `.is_some()` gates the system branch of `resolve_uses`, and the returned
`NodeId` is the DIAG-1 location. For 0008: the per-declaration question ("does
*this* `fn_decl` carry a `program_kind`?") is a different query and already has
a helper — `TreeView::first_child_with(node, Production::ProgramKind)`.

## Validation

`make -C compiler check`; the native grammar verifier passes against the
active v0.18 spec; new parser/finalize unit tests cover both entry shapes, a
`program_kind` on a non-`main` declaration (parses here; FN-7 in task 0008
rejects it), and units with and without a kind-declaring entry. A claimed
task lands only through lead review per the executor lane in
`docs/WORKFLOW.md`.

## Done-when

The audit checklist is green with any gaps filled, the kind-declaring
predicate is exposed as a stable accessor for 0007/0008, and
`make -C compiler check` is green with no existing test regressed.
