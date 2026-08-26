# Current Plan: capability-based completion I/O

Status: IMPLEMENTED AND GATED CANDIDATE on
`codex/io-model-completion-rebuild`; exact-byte owner review and activation
remain.

Active language authority: v0.36,
`fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62`.

This plan starts from main revision
`eab81a335addfb0ae060735771d4e98891dec2ea` and the settled first-principles
record. The v0.37 specification at `spec/kernel-spec.md` is a work-branch
candidate which supersedes the exact active v0.36 bytes. It is not a
merge-ready ACTIVE identity.

## Objective

Let ordinary Whitefoot calls keep independent outside operations in flight
without giving the writer a blocking API, async syntax, scheduling marker,
future, callback, global world identity, or unsafe escape. The compiler proves
which capability fragments may coexist. The runtime chooses inline, native
completion, scheduler progress, or a bounded target-only helper from target
facts and measured cost.

The deciding example is two positioned reads:

```whitefoot
let left = read_at(file: &file, destination: &uniq left_bytes, file_offset: 0_u64, start: 0_u64, end: left_end);
let right = read_at(file: &file, destination: &uniq right_bytes, file_offset: 4096_u64, start: 0_u64, end: right_end);
```

The file fragments are Free and the destination loans are disjoint, so both
operations may be submitted. Changing the two destinations to one buffer makes
ordinary memory proof refuse the overlap. Changing the calls to two writes on
one Output produces Ordered reservations instead: both may be pending, but
source attribution fixes byte order.

## Selected design

The complete derivation is in
`research/investigations/io-model/FIRST-PRINCIPLES.md`, the concrete API in
`DESIGN.md`, the experimental-branch audit in `IMPLEMENTATION-AUDIT.md`, and
measurements in `RESULTS.md` under the same investigation directory.

The selected boundaries are:

1. `external` and `blocks` are not effects or reserved words.
2. `reads` and `writes` may name an ordinary region or one direct formal
   capability parameter. Borrow lifetime and logical authority remain
   separate identities.
3. Each system family refines a capability access into a logical root,
   fragment, and Free, Ordered, or Exclusive relation.
4. A finite operation has distinct result-ready, payload-released,
   authority-released, and terminal facts even when one transition publishes
   all four.
5. Capacity is bounded. `wait-capacity` transfers nothing to the target and is
   not a writer-visible `WouldBlock` result.
6. Target code publishes results and wakes the scheduler. It never invokes a
   writer continuation or receives a writer function pointer.
7. A false claim cannot occur in a correct program, so no normal operation
   path reads a trap latch or carries trap-specific state.
8. Completion I/O is the shipped default. `--par` additionally enables
   compute overlap; pure compute output is byte-identical to the strict
   sequential reference and links no completion runtime.

## Implemented

### Language and compiler

- v0.37 capability-effect grammar, resolution, type checking, exact
  checked-both-ways rows, contract equality, release contribution, call
  substitution, and command-entry support.
- Closed-world capability-result origin fixed point with optional, fresh, and
  finite formal-origin components. Moves, match/give, recursive wrappers,
  loop backedges, and releases cannot wash an existing root into a fresh one.
  The executable implementation is complete for values carrying at most one
  runtime root. A product that may carry several roots reaches an explicit
  `CapabilityResultOrigin` unsupported result only after ordinary source
  judgments; it is never published with an empty effect or authority summary.
- Compiler-derived target-action and family-fragment summaries through the
  concrete call graph. Relations come from a family-owned fragment-pair table,
  not from either fragment alone; Ordered edges retain their attribution.
- Direct system calls in proof-derived pair and loop analysis, with memory,
  operand, loan, consumed-value, capability, and exit facts kept separate.
- Ordered reservation edges retained in checked metadata and IR. Direct
  same-block Output runs of 2–16 calls are admitted all-or-none, submitted in
  source attribution order, and committed as one batch. DirectorySource and
  unsupported shapes retain their edge and execute sequentially. Nonadjacent
  same-root edges survive through unrelated members.
- `read_at` with explicit offset and Free same-root reads;
  `DirectorySource` with Ordered `directory_next`; shared Ordered Output.
- `Interrupted` and `WouldBlock` removed from writer outcomes. No-progress
  interruption and readiness refusal stay inside target progress.

### Runtime and target adapters

- Preallocated bounded operation slots and target queues.
- Independent direct file groups of 2–64 operations reserve every completion
  slot all-or-none before target handoff, including the source-last member.
  This removes partial-admission hold-and-wait while preserving free target
  execution. Empty ranges complete reserved tokens without a host transfer.
- Captured generation checked before result storage changes.
- Product milestone state, exactly-one terminal publication, release/acquire
  result visibility, bounded drain, and lane-independent consume.
- One wake epoch for compute, target work, completion, admission, and capacity;
  completion-before-wait causes no syscall wake. Linux parks on one epoll set
  containing the io_uring fd and an eventfd. The eventfd remains a broadcast
  level fact until every already-announced waiter has left, preventing one
  waiter from consuming another token owner's wake.
- POSIX wakes one announced scheduler when exactly one is parked and broadcasts
  one epoch transition when several are parked. A completed target event is
  drained before its dependent writer frame enters the ready queue; that
  enqueue publishes its own compute epoch before any lane can park.
- An ordinary join registers its exact token only across the final
  recheck-to-park window. If another lane drains that token, the drainer clears
  the registration and publishes the consumability transition; uncontended
  drains pay no extra epoch.
- Typed POSIX/macOS helper fallback with zero-helper scheduler progress.
- Real Linux io_uring positioned read/write submissions, CQE-driven wake, and
  publication, executed on Linux 6.8.0 aarch64. Fatal post-handoff progress
  errors fail-stop instead of falling back or hanging.
- Real Windows overlapped ReadFile/WriteFile plus shared IOCP and a Win32-native
  completion core. The PE strict-cross-links, while production qualification
  remains fail-closed until it runs on Windows.
- Direct `read_at` and `write_once` overlap lowering plus a selective stackless
  slice: one single-block root suspension through zero-state tail wrappers to
  a file leaf can resume on any scheduler lane. Branches, loops, multiple
  suspension points, indirect calls, and non-tail suspended children retain
  the correct synchronous ABI.

### Evidence

- Completion core hostile harness, ASan/UBSan, repeated W0/W1/W4 stress,
  all-or-none batch pressure, staggered private-token multi-waiter wake,
  stale publication, terminal race, completion-before-wait, and zero-helper
  progress. The final exact-token drain fix passed 50/50 consecutive W0 runs
  after the unfixed binary had reproduced the deadlock at run 6.
- Compiler shape and executable tests prove that helpers receive typed target
  requests, not writer thunks, and that the second independent operation runs
  before a blocked first operation completes.
- Conformance structure 29/29, coverage 137/137, and native adapter
  `Pass=500 Skip=1 Fail=0`, with no verdict changes or deleted cases.
- Final matched O3 measurement: 35.85 ns for a core round trip; cached pread
  completion-progress adds 64.7 ns over direct. A one-waiter completion resume
  measured 1.625 us against condvar 1.542 us. These results retain direct
  depth-one specialization and reject a universal blocking-helper path.
- Rust library 1301/1301, maintained programs 56/56, conformance structure
  29/29, coverage 137/137, and native adapter `Pass=500 Skip=1 Fail=0`.

## Remaining sequence

1. Replace the current multi-root capability stop with a per-release-leaf
   origin tree, preserving each field through construct, move, projection,
   match, replace, call substitution, and derived release.
2. Generalize selective stackless lowering to branches, loops, multiple
   suspension points, indirect calls, and non-tail suspended children.
3. Actualize DirectorySource Ordered batches and the remaining finite
   may-suspend system operations without adding a writer-visible wait form.
4. Run cold-file, high-latency, network, and native Linux comparisons; run the
   Windows probe on a Windows runner and close its bounded multi-waiter wake
   proof before changing its qualification bit.
5. Add the designed network, timer, cancellation, deadline, and
   finish-required file-output catalog rows only with their target slices.
6. Before any merge, convert the candidate to an exact ACTIVE identity,
   archive v0.36 byte-for-byte, record the exact owner-approved spec and
   conformance boundary in `governance/APPROVALS.md`, and run canonical
   `make check` on that exact revision.

Every technical target of canonical `make check` has passed independently on
the candidate bytes. The canonical root invocation stops only at
`spec-archive-integrity`, which deliberately rejects CANDIDATE status before
activation.

## Non-negotiable boundaries

- No merge into `main` without owner approval of the exact revision.
- No trap-path tax on a correct operation.
- No whole writer wrapper on an I/O helper.
- No unbounded operation, completion, event, or payload queue.
- No target mechanism exposed as writer syntax.
- No environment alias fact promoted into language authority.
- No unsupported target reported as invalid source.
- No test, verdict, check, or failure path weakened to obtain a green gate.
