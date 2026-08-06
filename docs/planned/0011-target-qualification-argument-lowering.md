# 0011 — Target-qualification table and argument/path lowering

**Planned task.** Decomposed from `docs/current-plan.md` Work item 2 (wave 5
of 8; task 6 of 11). Not yet claimed — claiming fills in `Status`, `Owner`,
workspace, and `Base revision` and moves this file unchanged in number to
`docs/ongoing/` per `docs/WORKFLOW.md`. This record authorizes nothing
beyond Work item 2 itself; if `docs/current-plan.md` is replaced before this
task is claimed, delete this file unless the new plan explicitly retains
it.

- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, fourth bullet
  ("target-qualification table plus the static native macOS/Linux
  lowering"). Implements `spec/kernel-spec-v0.18.md`'s `QUAL-1`/`QUAL-2`/
  `QUAL-3`, `HOST-1`/`HOST-2`/`HOST-3`, `PATH-1`/`PATH-2`; `SYS-9`
  (`Args`/`HostString`/`RelativePath` contracts) and `SYS-13`
  (`ExitStatus`); and `PROG-3`'s status-mapping. Claimable only while
  `docs/current-plan.md` remains `ACTIVE`.

## Goal

Implement the `(spec version, semantic ID, target, program kind)` →
`(approved implementation version, private ABI symbol)` qualification table
as a private stage immediately before LLVM emission, extending the existing
target-qualification stage. Include the required-target-guarantee check for
command-lifetime argument backing (a target that can supply neither stable
native argv backing nor a pre-entry snapshot fails qualification — never
silent invalidation). Implement the command bootstrap (one-time
SIGPIPE-to-ignored normalization before entry per `QUAL-3`;
`ExitStatus`-to-process-status mapping per `PROG-3`) and the native
lowering for the argument/path operation cluster: `arg_get` /
`args_count` / `host_bytes_len` / `host_copy_bytes` / `host_utf8_len` /
`host_copy_utf8` / `relative_path`, plus logical-consume release for
`Args`/`HostString`/`RelativePath`/`ExitStatus` (no host call, no external
effect).

## Direction and invariants

- Static whole-build selection: no runtime operation-ID switch, target
  tag, per-call dispatch table, or handle-table lookup (`QUAL-3`). A hot
  transfer is required bounds/address checks, at most one direct host
  call, a count/outcome check, and a cold error mapper; the compiler
  wrapper must be inlined or shown immaterial.
- `arg_get` is an inline pointer/length lease over immutable command
  backing: no allocation, no byte copy, no Unicode restriction on the raw
  byte route.
- `host_copy_bytes`/`host_copy_utf8` validate `offset`/`capacity` before
  touching source or destination — trap on overflow or an out-of-bounds
  range before any read or write; a recoverable failure (`CopyTooSmall`,
  `Utf8CopyInvalid`, `Utf8CopyTooSmall`) leaves the whole destination
  buffer unchanged.
- `relative_path` is validation plus an inline retype of the same lease
  representation — no allocation, no copy.
- Qualification failure (a missing table entry, an unmet target
  guarantee) is a target-qualification failure under the existing
  non-source-rejection form (the same shape today's `TargetLayoutFailure`
  already uses) — never a source-language rejection.

## Method

Extend the existing `compiler/src/backend/target.rs` qualification
machinery (`validate_program`/`validate_function`/`TargetSelection`) with
the semantic-ID qualification table (a fixed Rust enum and match, no WIT
parser, semver registry, or dynamic loader) and the
command-lifetime-backing guarantee check. Add a new emitter module (for
example `compiler/src/backend/emitter/system.rs`, alongside the existing
per-family modules such as `emitter/integer.rs` and `emitter/buffer.rs`)
for the argument/path operation lowering, following that existing
one-module-per-operation-family pattern. Extend the `i32 @main()` wrapper
task 0008 introduced a branch for in `compiler/src/backend/emitter.rs` to
actually construct `Args` and materialize the command bootstrap (including
the SIGPIPE normalization), and to map a returned `ExitStatus` to the
wrapper's `i32` result.

## Scope and expected touch set

- `compiler/src/backend/target.rs` (qualification table, target-guarantee
  check)
- `compiler/src/backend/emitter.rs` (bootstrap, `main` wrapper argument
  construction, `ExitStatus`-to-int mapping)
- New: `compiler/src/backend/emitter/system.rs` (or similarly named
  sibling module) for argument/path operation lowering
- `compiler/src/backend/tests.rs` and a new `compiler/src/backend/tests/`
  file for the argument/path lowering shape
- Read-only precedent: `compiler/src/backend/emitter/buffer.rs`, the
  closest existing example of a runtime-length value's checked-operation
  lowering.

## Dependencies and integration order

Depends on task 0010 (checked-IR resource identities/cleanup must exist to
lower) and task 0008 (the entry-form checked-program shape). Task 0012
depends on this task (shares the qualification-table mechanism and
module).

## Validation

`make -C compiler check`; inspect emitted LLVM text for each hot operation
(reusing the `emitted_function`-style helper pattern already used by
`compiler/src/backend/tests/effect_attributes.rs`) to confirm no
allocation/copy call or indirect call on the argument-lease path; a
compiled-and-run program exercising `arg_get` with a non-UTF-8 argument
round-trips the raw bytes unchanged; a test double target lacking the
argv-backing guarantee fails qualification as expected. A claimed task
lands only through lead review per the executor lane in
`docs/WORKFLOW.md`.

## Done-when

The qualification table and argument/path lowering exist, satisfy the
structural cost inspection above, and a command program can read its own
arguments and construct a `RelativePath`; `make -C compiler check` green.
