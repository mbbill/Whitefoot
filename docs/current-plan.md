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

Slices 1–3 landed on `io/t4-resource-relations` (PR #13) on 2026-09-04;
slice 4 is in progress there.

1. Specification candidate + catalog + emitter + runtime counter (the tree
   compiles; old programs fail at the new signature). Done.
2. Corpus and fixture rewrite; conformance verdicts; new cases. Done.
3. Retirement ledger deletion and its tests; capacity accounting at start.
   Done: the ledger, its award order, the retire-and-retry paths on every
   route, the Windows open-order tickets and resource-attempt lock, and the
   two probes that scripted them are gone.
4. Docs (`docs/patterns.md` P9/P12, roadmap BOUND-1 current state, derivation
   ledger); canonical `make check` green; batch record in `docs/done/`.

### Decided 2026-09-04: a refused open hands its permit back

The owner's rule: the error is the system's, so the open keeps it; the
permit is ours, so it comes back where the checker can see it. The four opens
answer with `FileOpenOutcome`, `DirectoryOpenOutcome`, and
`SourceOpenOutcome` (`Opened(value: …)` / `OpenFailed(error: …, permit: …)`),
no count changes on a failure, and `propagate` no longer applies to an open.
Landed on the branch (`docs/done/0106-backed-file-permit.md` §7).

## Batch 2: park on miss

Sequenced after batch 1. The plan file is the specification of the work; its
§11 enumeration harness is the merge gate and the replay tool. The plan's
decisions file records why each mechanism is shaped as it is; implementation
findings that contradict it are folded back into the plan on this branch
before the code lands.

### Slices (started 2026-09-04 on `io/t4-resource-relations`)

0. **Measurements before a line of the compiler changes** (design §12). The
   decisive one first: the cost of one hand-written stack switch against the
   2.2 µs park-and-wake figure the tree measured, on this host; a switch that
   is not well under it removes the design's reason to exist. Home:
   `research/experiments/park-on-miss-switch-cost/`. The four-stage chain in
   C on io_uring runs on the Linux runner later in the batch; record growth
   per frame is reported from the stack ledger once slice 2 exists.
1. **The scheduler core and its enumerator** (design §5–§7.1, §11). One C unit
   that reaches shared state only through the seven primitives of §7.1, named
   in one header; a cargo test that compiles that unit against a replacement
   header and drives it with a controlled scheduler enumerating every
   interleaving of primitive steps for (T=1,S=2), (T=1,S=3), (T=2,S=3),
   (T=2,S=4), checking every §11 item at every step. This slice is the merge
   gate; nothing in slices 2–3 lands while it fails.
2. **Emitter** (design §8): the completion record as one opaque block of the
   submitting frame with its size and alignment as an ABI constant asserted on
   both sides; one lowering for every I/O operation, submit then join; the
   direct family, the inline arm, `stackless.rs` and the Windows verdict fork
   removed; compute join order reversed through one `compute_join_order`.
3. **Runtime** (design §7): the core replaces both parallel runtimes and both
   writer schedulers; the bridge, adapters and rings find the record by
   address and lose their pools, capacity waits and tokens; the I/O joins park
   the stack; the floor's entry runs `wf__main_body` on a pool stack selected
   at link time; the Windows twins in `wf_floor_windows.c` and
   `windows_bridge.c`.
4. **Replay, the remaining measurements, docs, record** (design §11 item 24,
   §12): the enumerator's recorder replays a run's data and completion order;
   park cost and per-frame record growth measured; `LOOP-PIPELINE.md` §3.4
   and the roadmap's two stackless items edited in place; batch record.

Batch 2 is done with the surface the language has today: read-only files,
directory enumeration, and the two standard outputs. That surface reaches
every state of the scheduler through the enumeration harness and injected
stubs, and it cannot exercise a real wait: a cached read does not wait, and a
cold read waits briefly and uniformly.

## After batch 2: the order the owner agreed on 2026-09-04

The surface is validated first and widened one API at a time, and every new
API passes the same T4 question the permit passed: the resource is a value in
the signature, and every dependency between it and an existing resource is
one the checker can see.

3. **The first API that waits: `command.stdin` read.** One operation, one
   resource, the same completion path. It is the first workload on which the
   park-on-miss scheduler faces a real wait, so the first one whose numbers
   say anything about the scheduler. Pipes and sockets are its kin and follow.
4. **File write and create.** The threshold for writing real programs (the
   compiler itself has to write files) and the second examination of the
   resource accounting: write handles, the namespace effect of a create, and
   their dependencies on directory reads, all expressed on the API the way the
   permit is.
5. Only then the rest of the roadmap's list: clock, timers, network,
   cancellation, namespace mutation.
