# 0011 — Target-qualification table and argument/path lowering

Live coordination record. It reports how authorized work is being carried
out; it is not authority and cannot expand Work item 2.

- **Status:** `IN PROGRESS`
- **Owner:** executor agent `exec-0011`
- **Workspace:** worktree `agent-a05157aec0c262f16`, branch
  `worktree-agent-a05157aec0c262f16`
- **Base revision:** `6413979` (`docs: close task 0010`)
- **Authority:** `ACTIVE` `docs/current-plan.md` Work item 2, fourth bullet
  ("target-qualification table plus the static native macOS/Linux
  lowering"). Implements the active `spec/kernel-spec-v0.19.md`'s
  `QUAL-1`/`QUAL-2`/`QUAL-3`, `HOST-1`/`HOST-2`/`HOST-3`,
  `PATH-1`/`PATH-2`; `SYS-9` (`Args`/`HostString`/`RelativePath`
  contracts) and `SYS-13` (`ExitStatus`); and `PROG-3`'s status-mapping.
  (The planned record was written against v0.18; v0.19 is the active
  version and renamed `arg_get`'s parameter to `position` — task 0018.)
  Claimable only while `docs/current-plan.md` remains `ACTIVE`.

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

## Progress

Complete, awaiting lead review. Both gates green by unpiped exit code
(`make -C compiler check`, `make check`); lib tests 391 → 404.

- The `(specification version, semantic ID, target, program kind)` table,
  the parallel resource rows that fix each opaque type's representation and
  release code, and the `QUAL-2` guarantee checks live in
  `compiler/src/backend/qualification.rs`, consulted once before layout and
  emission. A qualification stop is `BackendFailure::TargetQualification`
  →`CompilationStage::TargetQualification` /
  `CompilationFailureKind::TargetQualification`, citing no language rule.
- Approved implementations are emitted as `alwaysinline` private wrappers
  (`@wf.sys.<operation>.v1`) with one direct call per use site, plus a
  `; QUAL-1 ...` record in the module naming each resolved row.
  `args_count`, `arg_get`, `host_bytes_len`, `host_copy_bytes`,
  `host_utf8_len`, `host_copy_utf8`, `relative_path`, and `exit_status`
  are implemented; the shared complete UTF-8 validator serves both text-route
  operations.
- The `command` bootstrap establishes the `QUAL-2` backing from the native
  argument vector (no snapshot, no copy), installs the ignored
  write-to-closed-pipe disposition once before entry, opens the initial
  working directory for `command.cwd`, supplies the two `Output`
  descriptors, invokes the entry once, and maps the returned `ExitStatus`
  to the process status exactly. A start failure exits `71` before entry
  and produces no `ExitStatus`.
- Releases: logical consume and source detach emit no code; `DirectoryRead`
  and `ReadFile` emit exactly one direct `@close`. The checked program's
  `SYS-5` record is cross-checked against the table row.

Three decisions the lead should confirm:

1. `SYS-8`'s range trap needed a per-site `DIAG-3` record that
   `IrOperation::SystemCall` did not carry, so the checked program and IR
   now carry `trap: Option<TrapSite>` for a `traps` row. Its `rule_id` is
   `SYS-8` (the rule stating the failing runtime condition) and its
   `node_path` is the operation `call`, matching `DIAG-3`'s
   table-operation-contract-check convention. This reaches outside the
   listed touch set into `semantic/` and `lowering/`; 0012's `read_once`
   and `write_once` inherit it.
2. `command.args` exposes the complete native argument vector, so position
   0 is the invoked name. Dropping it would discard an argument the target
   supplied, which `HOST-1` forbids; nothing in the specification fixes the
   other reading. Task 0014's runtime lane depends on this.
3. The qualification table is a sibling module rather than a submodule of
   `target.rs`: `compiler/src/backend/target/` is swallowed by
   `.gitignore`'s `target/` rule, so a file there is never committed.

## Scope and expected touch set

- `compiler/src/backend/qualification.rs` (new: qualification table,
  target-guarantee check) and `compiler/src/backend/target.rs` (resource
  layout from the qualified representation)
- `compiler/src/backend/emitter.rs` (bootstrap, `main` wrapper argument
  construction, `ExitStatus`-to-int mapping)
- New: `compiler/src/backend/emitter/system.rs` (or similarly named
  sibling module) for argument/path operation lowering
- `compiler/src/backend/tests.rs` and a new `compiler/src/backend/tests/`
  file for the argument/path lowering shape
- Read-only precedent: `compiler/src/backend/emitter/buffer.rs`, the
  closest existing example of a runtime-length value's checked-operation
  lowering.
- Beyond the planned set, and required by the `SYS-8` trap record:
  `compiler/src/semantic/model.rs`,
  `compiler/src/semantic/check/expressions/calls/system.rs`,
  `compiler/src/lowering.rs`, `compiler/src/lowering/builder.rs`, and
  `compiler/src/driver.rs` for the qualification-failure class.

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

Met. Thirteen tests in `compiler/src/backend/tests/system.rs` carry the
evidence, including the optimized-module inspection of the lease path, the
non-UTF-8 round trip, and a probe target that withholds each `QUAL-2`
guarantee. Five conformance cases whose reason named this task
(`accept-sysentry-command-all-inputs`,
`accept-sysentry-command-no-inputs`, `accept-sysrelease-return-unit-declared`,
`accept-syseff-conditional-release-union`,
`accept-syseff-pure-immutable-only`) moved from `pending` to `runnable`
after each was compiled, linked, and run; their `expect` and `doc` are
byte-unchanged. `open_read`, `read_once`, and `write_once` remain the one
explicit `UnsupportedSystemInterface` stop, for task 0012.
