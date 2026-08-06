# 0008 — Entry-form admission and exit-status wiring

Live coordination record. It reports how authorized work is being carried
out; it is not authority and cannot expand or resequence work.

- **Status:** IN PROGRESS
- **Owner / workspace:** executor agent / isolated worktree
  `worktree-agent-a38599fe34c920d17`, lead-reviewed
- **Base revision:** `907076a`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, first bullet
  ("entry form"). Implements `spec/kernel-spec-v0.18.md`'s `FN-7` and
  `PROG-3`. Claimed while `docs/current-plan.md` remains `ACTIVE` (wave 3
  of 8; task 3 of 11; runs concurrently with task 0009).

## Goal

Extend the compiler's `main`-header check to admit the second entry shape —
a `command`-kind `main` whose parameters carry the four standard-input
labels bound to `Args`/`DirectoryRead`/`Output`/`Output` and whose result is
`own ExitStatus` with an effect row drawn from
`{allocates(heap), external, blocks, traps}` — while keeping the existing
zero-parameter `own unit` shape's exact four legal rows unchanged. Wire
`ExitStatus`'s returned value through to the process exit code per `PROG-3`
(normal completion only; trap and start failure stay outside it).

## Direction and invariants

- Ordinal identity selects the supplied standard input, never type identity
  (`command.stdout` and `command.stderr` share a type but are two distinct
  inputs); selected labels must appear in strictly increasing table order;
  an unlabelled parameter on a kind-declaring entry, an out-of-order or
  foreign-prefix or repeated label, or a wrong mode/type is an FN-7
  rejection at that `param` node.
- A `call` whose callee resolves to a kind-declaring entry is a hard FN-7
  error — the entry is invoked only by program start, never by source.
- The unlabelled shape's four effect rows and `own unit` result are
  unchanged; no existing conformance byte may regress.
- Out of scope: this task admits the entry's signature and wires
  exit-status mapping. It does **not** implement effect-row legality
  checking against exhibited effects for the command entry's body — that is
  `EFF-2`'s job (task 0009). This task's row admission answers "is this row
  one of the ones this form allows," not "does the declared row match what
  the body exhibits." Do not duplicate task 0009's logic here.

## Method

In `compiler/src/semantic/check.rs`, extend `Checker::check_main_header`
(currently the FN-7 enforcement site, around lines 454-527, invoked from
`check_program` before nominal/function collection) to branch on whether
the `main` `FnDecl` carries task 0006's `program_kind` node: the existing
branch keeps today's exact checks unchanged; a new branch validates the
standard-input table (ordinal order, label-to-type binding against task
0007's `Args` / `DirectoryRead` / `Output` / `ExitStatus` nominal IDs), the
`own ExitStatus` result, and that the written row is a subset of the
command-kind admitted categories in EFF-1 canonical order (not yet checking
exhibited-vs-declared equality — task 0009 owns that). Extend
`compiler/src/semantic/mod.rs`'s issue-kind surface for the FN-7 sub-cases
and their exact diagnostic locations (one per violation: `program_kind`
node, `input_label` node, `param` node, `rtype` node, `effects` node,
`generics`/`region_params` child, `call` node, or the `fn_decl` node as
fallback). Extend `CheckedProgramData` (`compiler/src/semantic/model.rs`,
where `main: FunctionId` currently lives) to record which entry shape was
admitted, since lowering needs to know which bootstrap to emit. In
`compiler/src/lowering.rs`, extend the existing `main_ordinal` handling so a
command entry's parameters bind to the sources task 0011's native bootstrap
supplies (this task defines the checked-program shape; task 0011 supplies
the actual native argv/cwd/output construction). In
`compiler/src/backend/emitter.rs`, replace the hardcoded `i32 @main()`
wrapper assumption (which currently asserts unit-result/no-parameters,
around lines 55-59 and 155-156) with a branch that, for a command entry,
emits the wrapper shape needed to receive standard inputs and map the
returned `ExitStatus` to the C `main`'s `i32` result per `PROG-3` — again,
task 0011 supplies the actual native construction this wrapper calls into.

## Progress

- Completed: claimed at base `907076a`; refreshed the integration branch and
  read `docs/WORKFLOW.md`, `spec/kernel-spec-v0.18.md` (`FN-7`, `PROG-3`,
  `GRAM-11`, `SYS-1`/`SYS-2`/`SYS-3`, `DIAG-1`), and the terminal records
  `docs/done/0006-entry-form-grammar.md` and
  `docs/done/0007-system-declaration-domain.md`.
- Completed (`006f181`): the complete FN-7 entry-form judgment in a new
  `compiler/src/semantic/check/entry_form.rs`, replacing `check_main_header`
  — closed kind table, closed standard-input table, the kind row's fixed
  result and admitted effect categories, the whole-unit `program_kind` /
  `input_label` placement rules, and the rejected source `call` to a
  kind-declaring entry; every rejection at the exact node FN-7 names.
  GRAM-11 named-argument admission for a callee resolving to a SYS-2
  operation. Both lead handoffs: the DIAG-1 stage-order repair, and
  `arg_get` covered by negative cases only.
- Current: record and report; the task's implementation is complete and both
  gates are green.
- Next: lead review and integration; task 0009 rebases onto this landing.

### Deliberate departures from the written Method, for lead review

1. **Diagnostic locations were repaired for both entry forms, not only the
   new one.** The Method says the unlabelled branch "keeps today's exact
   checks unchanged". Its *checks* are unchanged — the accepted and rejected
   program sets are byte-identical — but v0.18's FN-7 newly fixes a per-node
   location table for *every* FN-7 rejection, and the v0.17-era checker
   reported all of them at the whole `fn_decl`. Two divergent location
   schemes for one rule would have contradicted the single-normal-path rule,
   so one judgment now serves both forms. This is a compiler defect repair
   under `docs/WORKFLOW.md`'s blocker routing, with regressions pinning each
   located node.
2. **No `compiler/src/backend/emitter.rs` change, and only a boundary
   statement in lowering.** FN-7 fixes every `command` entry's result as
   `own ExitStatus`, so every admitted `command` entry names a system type
   and stops at the semantic system-use boundary; no such entry can reach
   lowering or the backend until the system type/call family lands
   (0009/0010) and 0011 supplies the native bootstrap. An emitter wrapper
   written now would be unreachable and untestable. Instead the checker
   records the admitted form (`CheckedEntryForm`) and `lower_checked` states
   that it implements exactly the unlabelled program start. The unsupported
   boundary moved right — from the entry declaration to the first resolved
   system use — it was not dropped.

## Scope and touch set as landed

- `compiler/src/semantic/check/entry_form.rs` (new; the FN-7 judgment)
- `compiler/src/semantic/check.rs` (stage sequence; the narrowed
  system-surface unsupported gate; `check_main_header` removed)
- `compiler/src/semantic/check/expressions/calls.rs` (GRAM-11 over SYS-2
  operation calls)
- `compiler/src/semantic/mod.rs` (nine FN-7 issue kinds; two superseded
  unsupported features removed)
- `compiler/src/semantic/model.rs` (`CheckedEntryForm`,
  `CheckedProgramData.entry`)
- `compiler/src/semantic/tree.rs` (`direct_identifiers`, for an
  `input_label`'s two IDENTs)
- `compiler/src/lowering/builder.rs` (the entry-form boundary statement)
- `compiler/src/driver.rs` (the superseded gate test; nine corpus negatives)
- `compiler/src/semantic/tests/entry_form.rs` (new; 16 tests)
- `tests/conformance/manifest.jsonl` (`status` and stale `reason` only)
- Not touched, against the Method: `compiler/src/backend/emitter.rs` and
  `compiler/src/backend/tests/` — see departure 2 above.

## Dependencies and integration order

Depends on task 0006 (grammar, landed `5cd1eef`) and task 0007 (the opaque
types must resolve for the entry's parameter/return types to type-check;
landed `c1178e8`). Both are terminal. Tasks 0009 and 0011 depend on this
task.

**Cross-link with task 0009 (`docs/ongoing/0009-effect-release-attribution.md`),
lead-authorized overlap.** Both tasks edit
`compiler/src/semantic/check.rs`. Integration order is fixed: **0008 lands
first; 0009 refreshes its base onto this landing, rereads the changed
`check_program` stage sequence, rebases, and reruns its gates.** The
semantic boundary between them is explicit and must not move by
last-writer-wins: this task owns the FN-7 entry-form admission judgment
(kind table, standard-input table, entry result, and the FN-7 subset test
over the written effect row); task 0009 owns EFF-2 exhibited-versus-declared
attribution and the acceptance of the `external`/`blocks` categories into
the checker's effect model. This task deliberately leaves the
`SystemEffectCategory` unsupported stop in place for 0009 to remove.

## Validation

`make -C compiler check`; new semantic tests for: a valid full four-input
entry; a valid zero-input command entry (admitted, per the deliberate
closure of that case); each FN-7 violation (out-of-order label, wrong
type, unlabelled parameter, a call to a kind-declaring entry, a
non-`ExitStatus` result); the unlabelled-`main` regression guard. A claimed
task lands only through lead review per the executor lane in
`docs/WORKFLOW.md`.

**As run.** `make -C compiler check` and `make check` green before and after;
lib tests 353 → 370, none removed, none weakened. The nine flipped
conformance cases were each run through `whitefootc` and produce exactly
their recorded `expect`, and each is additionally pinned in the compiler's
corpus-negative regression so the flip cannot silently rot while the corpus
has no adapter.

### Findings for the lead

- **`arg_get` positive case blocked (v0.19 requested).** As recorded at task
  0007's closure, SYS-2 names `arg_get`'s second parameter `index`, which
  FORM-3 excludes from IDENT, so GRAM-11 admits no complete legal `arg_get`
  call. The general named-argument rule is implemented and `arg_get` is
  covered by negative cases only; nothing was renamed.
- **Effect-row admission for the `command` form is currently unreachable.**
  Its inadmissible categories are exactly `reads`, `writes`, and
  `allocates(arena 'r)`, each of which needs a REGIONID that only region
  parameters could declare — and FN-7 rejects a region-parameter-bearing
  entry first. The rule is implemented as written; the reachable failure set
  is empty until a later version admits another category. The unlabelled
  form's row check is reachable and is tested both ways.
- **`reject-sysname-collision-in-kind-unit` now passes but was left
  `pending`.** It expects `reject`/`TYPE-6` and `whitefootc` produces exactly
  that. It is task 0007's rank-5 collision judgment, not this task's
  territory, so its status was not flipped; the lead may want to flip it when
  landing this change.

## Done-when

Both entry shapes are admitted exactly per `FN-7`/`PROG-3`; no existing
`fn main() -> own unit` conformance case changes verdict; the checked
program records enough for later tasks to bind real argv/cwd/output and
map `ExitStatus`; `make -C compiler check` green.
