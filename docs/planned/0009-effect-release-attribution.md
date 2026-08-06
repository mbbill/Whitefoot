# 0009 — Effect-checking extensions and release attribution

**Planned task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 3
of 8; task 4 of 11; runs concurrently with task 0008). Not yet claimed —
claiming fills in `Status`, `Owner`, workspace, and `Base revision` and
moves this file unchanged in number to `docs/ongoing/` per
`docs/WORKFLOW.md`. This record authorizes nothing beyond Work item 2
itself; if `docs/current-plan.md` is replaced before this task is claimed,
delete this file unless the new plan explicitly retains it.

- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, second bullet
  ("effect-checking extensions and release attribution"). Implements
  `spec/kernel-spec-v0.18.md`'s `EFF-1`/`EFF-2`/`EFF-3`/`EFF-5`, `FN-3`, and
  `STOR-3`, consuming `SYS-5` (completion policy and the per-type
  release-row table) and `SYS-7` (`IoError`). Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.

## Goal

Teach the effect checker the two new payload-free categories (`external`,
`blocks`) in the canonical row grammar; extend EFF-2's "exhibits" judgment
to be the union of the existing syntactic contribution and a new release
contribution (the union, over every normal control-flow edge, of the
release row of every resource that edge may release); and extend FN-3's
contract-conformance row normalization from four capabilities to six so a
`pure` contract member can no longer bind a function whose only effect is a
hidden release.

## Direction and invariants

- `external`/`blocks` are payload-free, take a fixed position between
  `allocates` and `traps` in EFF-1's canonical order, and `pure` excludes
  both.
- The release contribution has no syntactic occurrence anywhere in a
  declaration; it comes entirely from `STOR-3` per-type release-row data
  (the empty row for `Args`/`HostString`/`RelativePath`/`ExitStatus`/
  `Output`'s logical-consume-or-detach; `external, blocks` for
  `DirectoryRead`/`ReadFile`'s close attempt) — given as an exact table in
  `SYS-5`. This task consumes that per-type data — task 0007 registers the
  types; the release row itself is fixed content from that table, not a
  choice this task makes.
- `IoError` is not tag-only: `SYS-7` declares every one of its thirty
  variants with two fields, `code: u32` and `origin: u8`, so an `IoError`
  value is affine (moved or matched, never copied) and carries no release
  action or row of its own — only a resource value it might transitively
  own (for example inside a `Result<ReadFile, IoError>`) is released by
  task 0007's per-type table. This affects task 0010's checked-IR cleanup
  handling of outcome payloads, not this task's row computation directly.
- Existing `buffer<T>`/`box<T>`/arena releases keep their empty row and are
  explicitly **not** retrofitted into effect rows — no pre-existing
  accepted program's legal row may change.
- The load-bearing regression the whole extension exists to make correct:
  a nongeneric function whose only parameter is `own ReadFile` and whose
  complete body is exactly `return unit;` must be rejected when declared
  `pure` and accepted when declared `external, blocks` — matching the
  conformance flagship pair (`accept-sysrelease-return-unit-declared` /
  `reject-syseff-return-unit-omitted`).
- Release attribution reuses the existing FN-1 conservative structural
  normal-control graph — the same graph the current buffer/box drop-edge
  insertion already uses. This task does not invent a new dataflow
  analysis.

## Method

In the existing FN-1 normal-edge / cleanup-analysis code (the buffer/box
affine-move and reverse-order cleanup-edge logic under
`compiler/src/semantic/check/cleanup.rs`), extend the per-edge owner
disposition bookkeeping to recognize the seven system resource types and
look up each one's release row from task 0007's per-type data; union that
row into the function's exhibited row wherever the existing code already
unions syntactic call effects, in `compiler/src/semantic/check/control.rs`.
Add `external`/`blocks` fields to the effect-row representation (wherever
`EffectRow` is defined, likely `compiler/src/semantic/model.rs`) and to the
EFF-1 canonical-order validator. Extend `NormalizedEffects` in
`compiler/src/semantic/check/contracts.rs` — currently five fields
(`reads`, `writes`, `allocates_heap`, `allocates_arenas`, `traps`) at
approximately lines 39-46 — with two new boolean fields; its equality is
derived, so no separate comparison-site edit is needed. Add the new
diagnostic location for a release-attributed mismatch (the function's
`effects` node, since a compiler-derived release has no source occurrence
to point at) alongside the existing `.trap`-induced EFF-2 mismatch
location.

## Scope and expected touch set

- `compiler/src/semantic/model.rs` (effect-row representation)
- `compiler/src/semantic/check/control.rs` (EFF-1/EFF-2 row computation
  and validation)
- `compiler/src/semantic/check/cleanup.rs` (release-edge analysis extended
  to system types)
- `compiler/src/semantic/check/contracts.rs` (`NormalizedEffects`, ~39-46;
  FN-3)
- `compiler/src/semantic/mod.rs` (diagnostic location/issue kind for a
  release-attributed mismatch)
- New tests: a new file under `compiler/src/semantic/tests/` covering the
  canonical case, the conditional-release-on-one-`match`-arm union case,
  the superfluous-declaration case, and the FN-3 pure-contract-member case.

## Dependencies and integration order

Depends on task 0007 (the opaque resource types and their per-type release
rows must exist as consultable data). Does not depend on task 0008 — the
canonical case only needs a helper function taking `own ReadFile`, not the
command entry form. Runs concurrently with task 0008 (wave 3). Task 0010
depends on this task.

## Validation

`make -C compiler check`; new tests for the canonical accept/reject pair,
the conditional-release-union accept case and its narrow-declaration
negative, the declared-but-unexhibited superfluous case, and the FN-3
pure-member-with-release rejection; confirm zero existing conformance case
changes verdict (`buffer`/`box`/arena rows stay empty). A claimed task
lands only through lead review per the executor lane in
`docs/WORKFLOW.md`.

## Done-when

The six-capability FN-3 normalization and the release-attributed EFF-2
union are implemented and covered by the canonical case plus its three
companion cases; no pre-existing accepted program's legal row changes;
`make -C compiler check` green.
