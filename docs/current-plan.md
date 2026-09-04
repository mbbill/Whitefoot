# Current Plan: back the file permit, then park on miss

Status: IN PROGRESS on `io/t4-resource-relations` (PR #13). The previous
plan, source-carried proof, is IMPLEMENTED AND ACTIVATED as v0.40 and is
recorded in `docs/done/` and the v0.41 activation.

Active language authority: v0.41,
`899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761`,
which respells the six integer comparisons as symbols and delimits call-site
type application with `::`. `spec/kernel-spec.md` carries those exact ACTIVE
bytes until this branch's candidate is activated; the superseded v0.40 is
archived at `spec/kernel-spec-v0.40.md` and the merge-time record is in
`governance/APPROVALS.md`. Activation is branch content: nothing merges to
`main` until the owner approves the exact revision and canonical `make check`
passes on that revision. This document records technical direction and
sequencing; it grants no permission and adds no workflow gate.

## Outcome

Two batches, in this order, both on the one branch so the design documents
can change while the implementation teaches us what they got wrong.

1. **Backed `FilePermit` (specification v0.42).** Constitution T4 says a
   finite resource a system operation consumes is an owned value drawn from a
   factory whose capacity never exceeds what the target provides. Today's
   `FilePermit` is proof-only ([SYS-10]: never returned, promises no
   descriptor, erased before the native ABI), so `close` produces nothing the
   checker sees, a pipeline overlaps it with a later `open`, the host answers
   `EMFILE` for a schedule the sequential program never produces, and the
   runtime hides that with the descriptor retirement ledger
   (`compiler/src/backend/completion/contract.h:409-786`,
   `runtime.c:1142-1786`, `bridge.c:1271-1330`, Windows twins). This batch
   makes the permit real and deletes the ledger.
2. **Park on miss** (`research/investigations/io-model/PARK-ON-MISS.md`,
   63 recorded decisions). One scheduler for compute hand-outs and I/O
   completions; a join that misses parks its stack; the completion record
   lives in the submitting frame; no per-operation blocking fallback. Its
   implementation is gated on the enumeration harness its §11 specifies.

## Batch 1: backed `FilePermit`

### Language delta ([SYS-10], [SYS-11], [SYS-14], tables)

- `FileFactory` carries a capacity fixed at program start: the descriptors
  the target provides to this program, less the runtime's own and the
  handles the entry already holds. The capacity is not a source constant;
  it is observed only through `reserve_file`'s outcome.
- `reserve_file(factory: &uniq FileFactory) -> Result<FilePermit, IoError>`;
  the exhaustion member is `ResourceExhausted`, the program's own source-order
  outcome. Effects unchanged; still no host call.
- Three explicit closes return the permit, total, discarding the close
  diagnostic exactly as derived release does today:
  `close_read(file: own ReadFile) -> FilePermit`,
  `close_directory(directory: own DirectoryRead) -> FilePermit`,
  `close_directory_source(source: own DirectorySource) -> FilePermit`.
  Target contract may-suspend, terminal, like the opens. Derived (implicit)
  release still closes and burns the credit: FIRST-PRINCIPLES §12's first
  disposition. [SYS-11]'s "declares no separate explicit-close operation"
  is replaced.
- Consequence stated in [SYS-10]: an open holding a permit cannot fail for
  want of a descriptor; `ResourceExhausted` on an open names only honest
  target exhaustion (a limit changed outside the program, such as the
  system-wide file table).
- Not in this batch, named as the later step: a proof-admitted `reserve_file`
  whose domain (credit available) the checker discharges from a factory
  counter carried as a numeric fact, making exhaustion unrepresentable.

### Compiler

- `compiler/src/resolution/catalog.rs`: `reserve_file` result
  `ok_nominal(FILE_PERMIT, IO_ERROR)`; three close operations; effect rows.
- `compiler/src/backend/emitter/system.rs`: `emit_reserve_file` becomes a
  call to the runtime's credit counter (`wf__file_reserve`) returning the
  `Result` shape (precedent: `emit_relative_path`); the close operations lower
  to the existing native close plus the credit return; the permit's ABI
  representation stays the erased bit.
- `compiler/src/backend/qualification.rs`: ABI symbols and release table.
- Pinned sentences, driver fixtures, and the Rust test corpora that embed
  `reserve_file` programs are rewritten to the `Result` shape.

### Runtime

- One process-wide credit counter set at completion-runtime start from the
  target's descriptor budget (POSIX `RLIMIT_NOFILE` less the runtime's own
  descriptors and the entry's handles; Windows a fixed default the target
  exceeds). `wf__file_reserve` decrements or refuses; the close path returns
  a credit after the native close.
- The retirement ledger, its award order, `wf_bridge_retire_and_retry_direct`,
  `wf_completion_retirement_*`, the Windows twins, `retirement_interleave_probe.c`
  and the harness tests that exist only for them are deleted. An open that
  still meets `EMFILE`/`ENFILE` returns `ResourceExhausted` at once.

### Conformance and corpus

- `tests/conformance/cases`: every case that calls `reserve_file` moves to the
  `Result` shape with its verdict re-derived; new cases: reserve exhaustion is
  `Err(ResourceExhausted)` in source order; `close_read` returns a usable
  permit; a permit-holding open never answers `ResourceExhausted` for the
  program's own descriptors. Manifest entries and rule citations updated.
- `tests/programs` and `research/experiments` programs (56 files, 108 call
  sites) rewritten.

### Activation

- Work-branch mode: `spec/kernel-spec.md` carries
  `Status: CANDIDATE v0.42 supersedes v0.41 <digest>`; `make
  spec-candidate-integrity` passes. At merge: archive the v0.41 bytes at
  `spec/kernel-spec-v0.41.md`, install the ACTIVE v0.42 identity in
  `governance/APPROVALS.md`, update the digest-sync prose, META-5 delta, the
  derivation ledger, `compiler/src/spec.rs` and `spec_identity.rs`.

### Slices

1. Specification candidate + catalog + emitter + runtime counter (the tree
   compiles; old programs fail at the new signature).
2. Corpus and fixture rewrite; conformance verdicts; new cases.
3. Retirement ledger deletion and its tests; capacity accounting at start.
4. Docs (`docs/patterns.md` P9/P12, roadmap BOUND-1 current state, derivation
   ledger); canonical `make check` green; batch record in `docs/done/`.

## Batch 2: park on miss

Sequenced after batch 1. The plan file is the specification of the work; its
§11 enumeration harness is the merge gate and the replay tool. The plan's
decisions file records why each mechanism is shaped as it is; implementation
findings that contradict it are folded back into the plan on this branch
before the code lands.
