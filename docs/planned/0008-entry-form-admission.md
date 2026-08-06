# 0008 — Entry-form admission and exit-status wiring

**Planned task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 3
of 8; task 3 of 11; runs concurrently with task 0009). Not yet claimed —
claiming fills in `Status`, `Owner`, workspace, and `Base revision` and
moves this file unchanged in number to `docs/ongoing/` per
`docs/WORKFLOW.md`. This record authorizes nothing beyond Work item 2
itself; if `docs/current-plan.md` is replaced before this task is claimed,
delete this file unless the new plan explicitly retains it.

- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, first bullet
  ("entry form"). Implements `spec/kernel-spec-v0.18.md`'s `FN-7` and
  `PROG-3`. Claimable only while `docs/current-plan.md` remains `ACTIVE`.

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

## Scope and expected touch set

- `compiler/src/semantic/check.rs` (`check_main_header`, ~454-535)
- `compiler/src/semantic/mod.rs` (rule/issue kinds and diagnostic
  locations)
- `compiler/src/semantic/model.rs` (`CheckedProgramData.main` / entry-shape
  data)
- `compiler/src/lowering.rs` (`main_ordinal` and entry parameter binding,
  ~810, 834-835)
- `compiler/src/backend/emitter.rs` (the `i32 @main()` wrapper, ~55-59,
  155-156)
- New tests: a new file under `compiler/src/semantic/tests/` for
  entry-form cases; a new file under `compiler/src/backend/tests/` for the
  wrapper shape.

## Dependencies and integration order

Depends on task 0006 (grammar) and task 0007 (the opaque types must
resolve for the entry's parameter/return types to type-check). Runs
concurrently with task 0009 (wave 3) — neither depends on the other. Tasks
0009 and 0011 depend on this task.

## Validation

`make -C compiler check`; new semantic tests for: a valid full four-input
entry; a valid zero-input command entry (admitted, per the deliberate
closure of that case); each FN-7 violation (out-of-order label, wrong
type, unlabelled parameter, a call to a kind-declaring entry, a
non-`ExitStatus` result); the unlabelled-`main` regression guard. A claimed
task lands only through lead review per the executor lane in
`docs/WORKFLOW.md`.

## Done-when

Both entry shapes are admitted exactly per `FN-7`/`PROG-3`; no existing
`fn main() -> own unit` conformance case changes verdict; the checked
program records enough for later tasks to bind real argv/cwd/output and
map `ExitStatus`; `make -C compiler check` green.
