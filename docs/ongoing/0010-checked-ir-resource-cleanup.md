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

- Completed: claimed at base `48e53f0` (0009 terminal, its surfaces read).
- Current: reading the existing `buffer`/`box` cleanup representation on both
  the checked-program and typed-IR sides before extending it.
- Next: system-resource identity + release-action IR operations; move the
  lowering stop to the native-emission boundary; boundary tests.

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
