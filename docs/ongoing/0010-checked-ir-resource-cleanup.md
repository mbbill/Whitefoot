# 0010 — Checked-IR resource identities and cleanup

This is a temporary live coordination record, not execution authority. Move
this same numbered record to `docs/done/` at terminal disposition.

- **Status:** `IN PROGRESS`
- **Owner / workspace:** executor agent `exec-0010` /
  `worktree-agent-ab29c90c092ad87ec`
- **Base revision:** `48e53f0`
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, third bullet
  ("checked-IR resource identities and cleanup"). Implements dossier §11's
  last bullet ("checked IR resource/effect identities, preservation and
  cleanup of opaque backing leases across move/match/store/return/call"),
  `spec/kernel-spec-v0.18.md`'s `EFF-5` (the stdout/stderr may-alias
  record) and `TRAP-1` (release actions run only on normal edges, never on
  a trap). Claimable only while `docs/current-plan.md` remains `ACTIVE`.

## Goal

Extend the checked-program/typed-IR representation so each of the seven
system resource values carries a compiler-owned resource identity that
survives move, match, struct/enum store, return, and call boundaries, and
so the release facts task 0009 computes at semantic-check time become
concrete IR-level cleanup nodes lowering can consume — mirroring how the
existing buffer/box cleanup facts already flow from semantic check into the
typed IR today.

## Direction and invariants

- The first slice has no duplicate/split/attenuation operation for any of
  the seven types, so alias tracking is trivial (one identity per value,
  ordinary move tracking). Do not build general alias-lattice machinery no
  current operation needs.
- Retain the stdout/stderr "may-alias" record even though nothing in the
  first slice reads it (dossier §6.6): it exists so a later cross-resource
  reordering fact cannot treat two `Output` owners as disjoint sinks. This
  is pure retention, not new checking logic.
- Cleanup nodes fire only on normal control-flow edges; a trap emits none
  (`TRAP-1`/`STOR-3`). This should fall out of reusing task 0009's
  normal-edge graph rather than requiring separate trap-path logic.
- Out of scope: this task does not implement native lowering of any
  release action (tasks 0011/0012) — it only makes the IR carry enough for
  lowering to do so correctly.

## Method

Before designing anything new, read the existing buffer/box resource and
cleanup-edge representation this task must parallel (the checked-program
side and the typed-IR side both already carry this for `buffer<T>`/
`box<T>` per `compiler/README.md`'s description of "whole-binding affine
moves, and explicit reverse-order cleanup edges"). Extend that
representation — likely in `compiler/src/semantic/tree.rs` for the
checked-program side and `compiler/src/lowering.rs` /
`compiler/src/lowering/builder.rs` /
`compiler/src/lowering/builder/storage.rs` for the typed-IR side — with a
system-resource identity variant and cleanup-edge records keyed to task
0009's per-type release rows, reusing the existing pattern rather than
inventing a second cleanup mechanism.

## Progress

- Completed: claimed at base `48e53f0`. The [SYS-5]/[HOST-3] contract joined
  the resolution catalog (`SystemResourceContract`: identity, release action,
  row, backing), with `system_release_row` now derived from it and a
  spelling-keyed pin. `CheckedDrop`, `CheckedProjectedDrop`, and
  `CheckedStatement::DropExpression` carry an explicit `SystemRelease`
  record, computed once by `release_of_type`, which EFF-2 attribution now
  reads instead of rederiving. `CheckedEntryForm::Command` carries the
  SYS-12 stdout/stderr may-alias link. The typed IR gained
  `IrNominalKind::SystemResource(contract)`, `IrOperation::SystemCall` by
  SYS-2 semantic identity, `IrDrop::release`, and `IrProgram::entry`; the
  `BorrowSystemResource` path lowers like a box borrow. The unsupported stop
  moved from `LoweringFailure` to `BackendFailure::UnsupportedSystemInterface`
  (stage `Backend`, kind `Unsupported`), refusing over the IR's own facts.
- Current: gates green; record and report.
- Next: nothing in this task's scope. 0011 consumes the exposed surface.

## Scope and expected touch set

- `compiler/src/semantic/tree.rs` (checked-program resource/cleanup
  representation)
- `compiler/src/lowering.rs`, `compiler/src/lowering/builder.rs`,
  `compiler/src/lowering/builder/storage.rs` (typed-IR resource identities
  and cleanup-edge construction)
- New tests: IR-shape assertions, likely alongside the existing
  `compiler/src/backend/tests/resource_enums.rs` pattern or a new sibling
  file, verifying identity/cleanup preservation across move, one arm of a
  `match`, a struct field, a function return, and a function-call
  boundary, for at least one release-complete type (`ReadFile`) and one
  logical-consume type (`Args`).

## Dependencies and integration order

Depends on task 0009 (needs the semantic-check-time release facts to
translate into IR). Tasks 0011, 0012, and 0013 depend on this task.

## Validation

`make -C compiler check`; new IR tests confirming identity/cleanup-edge
preservation across every listed boundary and confirming no cleanup edge
appears on a trap edge. A claimed task lands only through lead review per
the executor lane in `docs/WORKFLOW.md`.

## Done-when

The typed IR carries a system-resource identity and correct cleanup edges
for all seven types across every required boundary; `make -C compiler
check` green.
