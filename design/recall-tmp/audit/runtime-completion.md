# Audit: the completion (I/O) runtime against the design tree

Module: `compiler/src/backend/completion/` — `runtime.c`, `bridge.c`/`bridge.h`,
`contract.h`, `native_contract.c`/`.h`, `file_adapter.c`/`.h`,
`file_posix.c`/`.h`, `file_windows.c`, `linux_io_uring.c`/`.h`,
`windows_iocp.c`/`.h`, `wait_host.c`, `wait_windows.c`, `socket_address.h`
(about 9,000 lines excluding tests). `harness.c` and the five `*_probe.c`
files (about 5,500 lines) are tests, read as evidence of intended behavior
per the task brief, not audited as targets in their own right. Direction:
code to tree — finding choices the code embodies that `design/` does not
record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/language/system-interface.md` and both of its children
(`declaration-home.md`, `directory-enumeration.md`),
`design/compiler/parallel-lowering.md` and its two children
(`parallel-runtime.md`, `two-worlds.md`), `design/compiler/resource-exhaustion-floor.md`,
and `design/language/effects.md`. Also `compiler/Makefile`'s completion targets
and `.github/workflows/io-hosts.yml`, and the relevant sections of
`spec/kernel-spec.md` ([SYS-1] through [SYS-18]).

Three scope notes:

- `design/language/system-interface/declaration-home.md` and
  `design/language/effects.md` are compile-time/name-resolution and
  type-system concerns with no implementing code in this C runtime at all
  (effects are erased before lowering, and this module never resolves a
  name); they are omitted from section 1 rather than force-fit, the same
  treatment the lexer-and-syntax audit gave `name-resolution.md`.
- `design/compiler/resource-exhaustion-floor.md` governs stack/heap
  exhaustion, which is `backend/wf_floor.c`/`wf_floor_windows.c` — outside
  this module's file list — and nothing in `completion/` implements or
  overrides it; it is cited only where this module's own, different
  fail-stop discipline (§3 item 3) needs contrasting with it.
- `[SYS-10]`'s `HandleFactory`/`HandlePermit` credit count, which several
  comments in this module cite, is likewise implemented entirely outside it
  (`backend/wf_floor.c`); this module only consumes the descriptors that
  bookkeeping already authorized.

Git history note: this worktree's clone is complete back to the true root
`7c1d7641` (2026-07-07), confirmed directly (`git log -S`, `git show`)
rather than assumed from the other audits' notes. Every "Reason" below cites
the introducing commit by hash and date where `git log -S` on a distinctive
token found one; "no reason recorded" means neither the commit nor its
successors say anything beyond what the code comment already states.

## 1. Covered by the tree

- `design/language/system-interface.md` (decision 2: resource contracts
  state outcomes, ownership, cleanup, capacity and target guarantees, while
  completion and host scheduling are lowering concerns, because a source
  contract must be checkable without a scheduler in the language) — the
  whole module exists on the far side of that line: `contract.h`'s opening
  comment states it directly ("This is an internal compiler/runtime ABI,
  not a writer-visible API"), and no Whitefoot type, effect row, or region
  anywhere in `compiler/src/backend/completion/` names a ring, a queue, a
  wait set, or a helper thread.
- `design/language/system-interface.md` (decision 3: arguments and paths
  preserve the target host's bytes with explicit conversion) —
  `contract.h::wf_file_request`'s `open_at.path` (`const char *path`, "The
  submitting frame's own bytes, live until the join," lines 135-150); every
  open and enumerate call in `file_posix.c::wf_file_execute_once`,
  `file_windows.c::wf_file_windows_open_at`, and
  `linux_io_uring.c::wf_linux_stage_entry_locked` passes that pointer to the
  host call unconverted, with no re-encoding or copy.
- `design/language/system-interface/directory-enumeration.md` (decision 1:
  one-attempt transfer of a bounded batch into the caller's range, the
  handle owning a cursor and no storage) — `contract.h`'s
  `WF_FILE_DIRECTORY_NEXT` request (buffer, count, and a scratch `position`
  cell only, lines 198-211, "never a component of the `DirectorySource`
  value"); `file_posix.c`'s single `getdents64`/`__getdirentries64` call per
  submission with no internal retry loop; Linux leaves the position cell
  untouched because "the whole cursor" lives in the descriptor, exactly as
  the decision requires.
- `design/language/system-interface/directory-enumeration.md` (decision 2:
  an enumerated name reaches source only as bytes, self/parent entries
  unfiltered, no fixed order) — neither `file_posix.c`'s directory-batch
  case nor `file_windows.c::wf_file_windows_directory_next` filters,
  reorders, or interprets the host's raw batch; the bytes reach the
  destination the caller named exactly as the kernel wrote them.
- `design/compiler/parallel-lowering.md` (decision: native queues, helper
  lanes, wakeups, and completion ports are target-private protocol state,
  never Whitefoot shared storage) — `linux_io_uring.h` and `windows_iocp.h`
  each open with a "Target-private protocol state" comment making this
  exact claim; the file adapter's intrusive queue
  (`file_adapter.h::wf_file_adapter`), the mmap'd io_uring rings, and the
  `HANDLE port` are C-only structures with no Whitefoot type, region, or
  effect naming them.
- `design/compiler/parallel-lowering/parallel-runtime.md` (decision 4: an
  I/O join progresses completion and then waits through the native backend
  on that same stack, without helping compute tasks) —
  `bridge.c::wf_bridge_join` (lines 985-999): claim and run its own queued
  record, call `wf_bridge_progress` (ring reap, or one queued request when
  no helper exists), then either a bounded spin or `wf_bridge_park`, which
  reaches `wf_completion_park_if_unchanged` or the ring's own park. No path
  through this module reaches the compute deque, steal, or help machinery.
- `design/language.md` (decision 3: an accepted program has one observable
  behavior; no build mode, flag, or switch changes what it does) — the
  `WF_IO_HELPERS`/`WF_IO_NOCACHE`/`WF_IO_NO_NATIVE_RING`/
  `WF_REQUIRE_WINDOWS_IOCP` family of environment-read runtime policies this
  module itself acts on (`bridge.c:161-212`, `:323-327`, `:546-554`;
  `file_posix.h:107-117`; the harness-only `WF_REQUIRE_LINUX_IO_URING` is
  explicitly not one of them — `bridge.c:520`'s own comment: "the harness's
  setting and not [the bridge's]") is documented at each site as changing
  no accepted program's output, only which engine executes it, and is
  tested as such: `completion-default-route-test` runs the identical probe
  on both routes (`compiler/Makefile:292-308`), and
  `.github/workflows/io-hosts.yml`'s `completion-windows` job compiles one
  `.wf` TCP program once and diffs its behavior across
  `WF_REQUIRE_WINDOWS_IOCP=1` and `WF_IO_NO_NATIVE_RING=1` runs.

## 2. No decision needed

Contract and record:

- The completion record and the host wait set are C11 opaque byte blocks
  (`wf_completion_ring_state`, `wf_completion_wait`) sized and aligned by
  `_Static_assert` against a platform-specific struct defined in a unit the
  shared header never includes (`windows_iocp.c::wf_windows_iocp_state`,
  `wait_host.c`/`wait_windows.c`) — the ordinary PIMPL-by-byte-block
  technique for a header two platforms share.
- Every `_Atomic` field across `contract.h`, `file_adapter.h`,
  `linux_io_uring.h`, and `windows_iocp.h` names an explicit memory order at
  every access rather than relying on a default, and the `issued`/`state`
  pair's release-then-acquire ordering is called out by name at its
  declaration — ordinary correct C11 atomics for a genuinely
  multi-writer/multi-reader structure, not a project-specific policy.
- Every host-call seam a test needs to script (`WF_COMPLETION_PREAD`,
  `WF_FILE_OPENAT`, `WF_FILE_MONOTONIC_NS`, `WF_COMPLETION_POLL`,
  `WF_COMPLETION_GETDENTS64`/`GETDIRENTRIES64`, `WF_COMPLETION_WAIT_RETURN`)
  is a compile-time `#define` substitution, never a runtime function
  pointer or vtable; the Makefile's own comment states the reason
  (`compiler/Makefile:91-93`): a build that named a different set "would
  run a different runtime from the one the ordinary build runs."
- Checked, explicit-width arithmetic before every submit
  (`(uint64_t)(size_t)count != count`, the `INT64_MAX` offset check in
  `wf__completion_file_pread_submit`) — ordinary overflow-safety practice,
  unrelated to language semantics.

Errors and retries:

- EINTR and readiness refusal (`EAGAIN`/`EWOULDBLOCK`) are absorbed inside
  the direct-execution retry loop for every transfer kind
  (`file_posix.c::wf_file_execute_direct`) and never reach a completion
  record as a distinct outcome; `connect`, `listen`, and `close` are
  excluded from that loop for reasons POSIX itself forces (an interrupted
  connect continues in the kernel and a retry answers `EALREADY`; retrying
  a listen would create a second socket; a close attempt has already
  consumed authority over the descriptor) — this is a mechanical
  consequence of [SYS-7]'s closed `IoError` class list, which has no
  "interrupted" or "would-block" variant for either path to report instead.
- Descriptor-status verification after an open
  (`file_posix.h::wf_file_kind_outcome`) is one small function shared
  verbatim by the POSIX adapter and `linux_io_uring.c::wf_linux_decide_open`,
  so a FIFO is refused identically whichever engine opened it — the
  specification's own `open_file`/`FileOpenOutcome` table ([SYS-2]) leaves
  no room for the two engines to disagree.
- Release-complete resources ([SYS-5]) close with one native attempt and a
  discarded diagnostic, never retried, in every engine
  (`file_posix.c`'s `close`/`shutdown` cases,
  `linux_io_uring.c::wf_linux_decide_open`'s error path,
  `file_windows.c::wf_file_windows_close`) — the specification states this
  exact discipline for `close_read`/`close_directory`/`close_listener`
  itself; the runtime has no alternative to transcribe faithfully.
- `wf_linux_io_uring_init` refuses to start the ring without
  `IORING_FEAT_NODROP` (`linux_io_uring.c:237-243`) — a correctness
  requirement rather than a policy choice: a dropped CQE would strand an
  operation the runtime has already told the caller is accepted, which the
  "every submit path ends in a published record" invariant forbids
  outright.

Sockets and threads:

- `socket_address.h`'s conversions (`wf_socket_native_from_address`,
  `wf_socket_address_from_native`, `wf_socket_publish_peer`) are one set of
  `static inline` functions shared by all four socket-issuing engines
  (`file_posix.c`, `file_windows.c`, `linux_io_uring.c`, `windows_iocp.c`) —
  ordinary avoidance of duplicated, easily-diverging byte-order logic, and
  the header's own comment states exactly that reason.
- `IORING_SETUP_COOP_TASKRUN` is requested and the ring quietly retries
  setup without it on `EINVAL` (`linux_io_uring.c:205-231`, for kernels
  before 5.19) — an ordinary two-attempt capability probe, the same shape
  as any feature-detection code.
- The file adapter's helper threads are plain detached threads with no
  join-by-handle; shutdown waits for a `live_helpers` counter to reach zero
  under the queue lock instead (`file_adapter.c::wf_file_await_helpers`) —
  ordinary technique once a runtime commits to detached rather than
  joinable worker threads.
- Windows' socket transfers never attempt the synchronous fast path POSIX's
  `wf_file_transfer_now` does (`file_windows.c:680-684` always answers 0);
  the comment states the reason directly — a socket this adapter dispatches
  to Winsock is a genuinely blocking descriptor, so every ready-or-not
  answer already comes from whichever engine (the port or the adapter) the
  bridge chose. A platform difference the code explains, not an
  unaddressed gap.

Testing and build:

- `compiler/Makefile`'s `completion-test` links the harness against the
  exact production sources (`COMPLETION_SOURCES`) plus a small,
  fully-named set of test-only substitutions, and separately builds a
  `pure-compute` probe whose linker output is `nm`-checked to carry no
  `wf_completion_` symbol at all (`compiler/Makefile:255-261`) — ordinary
  link-boundary testing, ordinary CI structure.
- `native_adapter_probe.c` (Linux) exits 77 when io_uring is unavailable,
  which `make check`'s `completion-test` tolerates but
  `.github/workflows/io-hosts.yml`'s `completion-linux` job explicitly
  turns into a hard failure (lines 54-82 of that file) — an ordinary
  "optional locally, mandatory on the evidence host" split, not a language
  or runtime policy.

## 3. Choices without a node

**1. The native runtime every compiled program links against — this whole
module — is plain C with hand-checked memory and thread safety, and no
design-tree node states that split or what stands in for the guarantee
`design/compiler.md` gives the compiler itself.**
`design/compiler.md`'s decision 1 fixes the compiler as "one safe-Rust
crate ... forbidding unsafe code makes the compiler's own memory safety
Rust's guarantee rather than a review burden," and that scope is exact:
`compiler/src/lib.rs:1`'s `#![forbid(unsafe_code)]` covers `whitefootc`,
the Rust program that reads `.wf` and emits an object. It says nothing
about, and does not reach, the C sources under `compiler/src/backend/`
that `whitefootc` links into the target executable — raw pointers
(`wf_completion_record *`), manual `memset`/`memcpy`, hand-written C11
atomics, `pthread_mutex_t`, and `mmap`, checked by review and
`_Static_assert` alone, with no language mechanism forbidding an unsafe
escape the way Rust's does for the compiler.
Alternative: hold the target runtime to the same explicit discipline
statement the compiler gets — whether that means implementing it in a
memory-safety-checked language, confining and reviewing its unsafe
operations under a named, narrower policy than "forbidden," or simply
recording, once, why plain C is the accepted trusted-computing-base
language for the code every compiled program actually executes.
Where: every file in `compiler/src/backend/completion/`, none of them
`.rs`; the sibling `compiler/src/backend/sched/` and
`compiler/src/backend/wf_floor.c`/`wf_floor_windows.c` share the same
property and the same absence of a tree node for it.
Reason: none recorded. No design-tree node, architecture dossier, or commit
body found states why the linked runtime is C rather than a
safety-checked language, or what discipline is meant to substitute for
`design/compiler.md`'s Rust guarantee at this boundary; the retired
`compiler-architecture-*.md` dossiers (`archive/research/product-goal-design/`)
discuss a runtime/allocator/trap trusted-computing-base repeatedly but never
this specific question.
Effect: none on acceptance — this changes no program's behavior. It is a
gap in what the tree records about the project's own central premise
(CLAUDE.md: "There is no writer-accessible unsafe escape or runtime trap"),
since the trusted code every accepted program actually runs at every I/O
boundary is exactly the code that premise does not reach.

**2. The completion record is a block of the submitting frame, found by
address, with no pool, slot, token, or generation — so no I/O operation can
ever be refused for capacity, and its buffers and paths cross the ABI as
live loans rather than copies.**
Alternative: the prior design's fixed-size global slot pool, with
generation-checked tokens, a `WAIT_CAPACITY` refusal a submitter looped on,
and a fixed-capacity path buffer with a "path does not fit" demotion —
deleted by this change.
Where: `contract.h::wf_completion_record` (lines 314-345) and its ABI
constants (`WF_COMPLETION_RECORD_BYTES`, lines 353-376); `bridge.c::wf_bridge_begin`
(never answers a refusal, lines 1243-1258) and `wf_bridge_dispatch`
(lines 1282-1291); `file_adapter.c`'s intrusive, capacity-free queue
(`wf_file_enqueue_locked`, lines 685-720).
Reason (quoted, commit `3acc3e95`, 2026-09-05, "The completion record is
the frame's block, found by address"): "Design §7: the slot pool and its
tokens, claims, milestones, drains, consumes, dependent frames and capacity
waits are deleted from the completion core, the bridge, the file adapter
and the io_uring adapter ... Every submit ends in
`wf_completion_record_complete` ... The joins wait in place through the
core's own registration." The design record it implements
(`research/investigations/io-model/PARK-ON-MISS.md` §5) states the
grounding measurement directly: "every Whitefoot frame is static
(`stack_ledger.rs:38-42`), clang sizes it and the ledger reads that size,
so the ceiling is the stack and nothing else."
Effect: architecture. This is the single decision the rest of the module's
shape follows from — items 3, 4, and 6 below are all downstream of a record
that must fit in one fixed frame-reserved block and can never be refused.

**3. A ring's absence or explicit refusal at startup is an ordinary,
silent fallback to the shared blocking-helper adapter; the identical
failure occurring after the runtime has already accepted an operation is a
process-ending fail-stop, with no fallback and no writer-visible `IoError`.**
Alternative: treat a post-acceptance ring failure the same as a startup
refusal — retry the stranded operation on the bounded adapter, or report it
as an ordinary `IoError` — rather than aborting the process.
Where: `bridge.c::wf_bridge_initialize` (silent fallback,
`!wf_bridge_ring_start() && !wf_bridge_ensure_file()`, lines 814-833) versus
`wf_bridge_ring_offer`/`wf_bridge_ring_progress`/`wf_bridge_ring_park`'s
calls to `wf_bridge_fail`/`wf_bridge_fail_with_code` on any post-acceptance
target error (lines 371-447, 616-698); `linux_io_uring.c::wf_linux_record_progress_error`;
`windows_iocp.c::wf_windows_iocp_fail` (lines 95-99, 661-664, 836-841).
Reason: stated directly at the ring-progress fail site
(`bridge.c:408-412`, inside `wf_bridge_ring_progress`): "Target ownership
has already transferred. Falling back now would duplicate an operation,
and ignoring the error would strand its owned operation forever. A
target-runtime failure is a fail-stop TCB defect, not a writer-visible
IoError." The offer site states the same judgment in different words
(`bridge.c:378-382`): "The kind and shape were both answered before the
record was offered, so any other answer is a target-runtime failure."
`windows_bridge_init_fail_stop_probe.c`'s own
header names the asymmetry as the point of the probe it is: "a *refused
ring* is no longer a failure on this platform ... What has to stop the
process is a bridge with neither engine." No commit body was found that
weighs this fail-open/fail-stop split against the alternative of degrading
every post-acceptance failure to the bounded adapter as well.
Effect: safety/reliability, not acceptance. Tested directly by
`.github/workflows/io-hosts.yml`'s "Require Windows bridge initialization
to fail stop" job (exit code 86) and by `native_adapter_probe.c`; a
deliberate choice, well-tested, but recorded nowhere but the code and the
CI job that exercises it.

**4. The completion record's fixed 160-byte, 8-word ABI budget — sized to
the largest single operation, `open_at` — is what keeps Windows' `AcceptEx`
and a positioned stream write or `stat` off the completion port, not a
qualification gap.**
Alternative: grow the record (or give `accept` its own separately-owned
buffer outside the fixed block) to admit `AcceptEx`'s 88-byte address pair,
paying the larger per-operation footprint on every I/O-carrying stack frame
in every Whitefoot program, not only ones that accept connections.
Where: `contract.h`'s `WF_COMPLETION_RECORD_BYTES = 160` and its two
`_Static_assert`s (lines 353-376); `windows_iocp.c:218-241`'s comment
explaining the omission; `file_windows.c:644-654`'s `default:` case
refusing `WF_FILE_PWRITE`/`WF_FILE_STATUS` as an outcome rather than a
crash, because the completion port has no overlapped form for either on
this target row.
Reason (quoted, `research/investigations/io-model/NETWORK.md` §5, dated
2026-09-06 by its own "Landed for Windows" note): "`AcceptEx` is **not**
used, and the reason is a measurement: its address pair is
`2 * (sizeof(sockaddr_in6) + 16)` = 88 bytes of caller storage that must
live until the operation completes, the completion record is exactly 160
bytes on this platform ... and the record may not grow." The code comment
at `windows_iocp.c:218-241` restates this near-verbatim.
Effect: performance only on Windows — every accept is the shared
blocking-helper adapter's ordinary `accept`, capped at
`WF_BRIDGE_MAX_HELPERS` (8) concurrent accepts in flight, never the
completion port's own concurrency.

**5. The helper-pool's demand-driven growth policy decides whether a
queued socket operation is genuinely overlapped by another OS thread at
all, and its known consequence — no more than `WF_BRIDGE_MAX_HELPERS` (8)
peers may wait at once on any host without a native ring — is an
acknowledged, open limitation with no tree entry.**
The policy itself (a ~20-microsecond measured-wait threshold sampled one
execution in sixteen, plus unconditional growth for any peer-bound
request) is well-reasoned and cited in place; what has no home is the
bound it leaves standing.
Alternative: the readiness-driven adapter named as the "next step" — one
`poll`/`kqueue`/`WSAPoll` over every queued descriptor, waited on from
inside the park a thread with nothing to run already enters, letting any
number of peers wait on one thread — which was never built.
Where: `file_adapter.c::wf_file_grow_helpers_locked` (lines 644-671),
`WF_FILE_OVERLAP_WAIT_NS = 20000` and `WF_FILE_EXECUTE_SAMPLE_INTERVAL = 16`
(lines 139-159), `wf_file_request_is_peer_bound` (lines 87-100);
`bridge.c`'s `WF_BRIDGE_MAX_HELPERS = 8` (line 57).
Reason (quoted, commit `8f06cbd6`, 2026-08-27, "io: measure completion at
the program level and fix what measuring found," for the growth policy
itself): "An unset WF_IO_HELPERS pinned one helper, the worst measured
setting for a program with width. The policy now grows on queue pressure
up to the machine's CPU count, and starts at zero where a native ring is
ready." The open bound is named directly in
`research/investigations/io-model/NETWORK.md` §5 ("Peer-bound requests are
a helper's," 2026-09-06): "The bound this leaves is explicit and is not
worked around ... **Open design item:** ... That is the next step for
Darwin and for `WF_IO_NO_NATIVE_RING`," describing the exact
`tests/programs/tcp_fanout.wf` failure (three workers waiting on a silent
peer, no thread left for a fourth connection) this policy was built to
contain rather than remove.
Effect: correctness-adjacent on the ring-less path (Darwin, or Linux/Windows
under the test-only `WF_IO_NO_NATIVE_RING`) — a peer wait beyond the eighth
concurrent one has no engine at all and queues with no timeout; the
research record raises this as open and unresolved, and no design-tree
node has since settled whether that is acceptable.

**6. `tcp_listen`/`tcp_accept`/`tcp_connect` never set `TCP_NODELAY` on any
socket, on any platform — reads as a dropped measured finding, not a
deliberate rejection.**
Confirmed by exhaustive search: no occurrence of `NODELAY` anywhere under
`compiler/src/backend/` (production or test-only) on either `HEAD` or
`origin/main`; the only `setsockopt` call in the whole module
(`windows_iocp.c:457`, `SO_UPDATE_CONNECT_CONTEXT`) is unrelated.
Alternative: set it once on every socket this runtime creates or accepts,
exactly as the benchmark harness's own `WF_TCP_NODELAY` setting already
does outside the shipped runtime.
Where: absence — `file_posix.c`'s `WF_FILE_SOCKET_LISTEN`/`_CONNECT` cases
(lines 306-371) and `wf_socket_open` (lines 69-80); `file_windows.c`'s
`wf_file_windows_socket_listen`/`_connect` (lines 372-439);
`linux_io_uring.c::wf_linux_io_uring_submit`'s inline `socket()` call
(line 727); `windows_iocp.c::wf_windows_open_connect_socket` (lines
356-377).
Reason (quoted, `research/investigations/io-model/SCHEDULER-FINDINGS.md`,
experiment 17, digested into the tree by commit `176bb16f`, 2026-09-11,
"Digest the 68 scheduler experiments before their branches go"): "At 64
peers and 64 KiB the same program and runtime change p99 from 41,667 to
2,410 us ... paired p99 0.0579 ... io_uring without the option runs at
about 1,562 requests/s with a 41..42 ms p99 in every large-payload
placement and gains 11.78x to 14.98x paired throughput when enabled ...
*Verdict:* **holds** — packet policy must be controlled before a tail is
attributed to the language or a continuation model. The bench sources
already set the option; the magnitude and the reason are recorded nowhere
else." That last sentence is the research record's own acknowledgment that
the finding never reached production code, and nothing in `docs/todo.md`
or any later commit tracks
it either.
Effect: performance only — no accepted program's bytes change — but a
specifically measured, specifically explained, order-of-magnitude tail and
throughput cost with no counter-argument recorded anywhere for leaving it
out.

**7. The connection half-close two-count table is a fixed `2^20`-entry
global array indexed by the raw native descriptor number, correct only for
as long as an assumption enforced nowhere in this module continues to
hold.**
`wf_file_connection_release` treats a descriptor at or above
`WF_FILE_CONNECTION_DESCRIPTORS` as never having a second half to release,
so such a connection is half-closed but its underlying object is never
handed the close that would actually release it.
Alternative: size or `_Static_assert` the table against the actual
`HandleFactory` descriptor ceiling it is documented to mirror (in
`backend/wf_floor.c`, outside this module), so the two constants cannot
drift apart silently; or key the two-count by something other than a raw
OS descriptor number.
Where: `file_adapter.h:246-272` (`WF_FILE_CONNECTION_DESCRIPTORS = 1u << 20`,
and the doc comment stating the cross-file assumption); `file_adapter.c:110-137`
(`wf_file_connection_halves`, `wf_file_connection_release`'s
range-guard returning 0 for an out-of-range descriptor with no other
action taken).
Reason: stated as an invariant, not derived: "The table covers every
descriptor the [SYS-10] handle factory can produce: that factory's own
capacity ceiling is this same number less the reserve it keeps for the
runtime's descriptors (`backend/wf_floor.c`), and the host hands out the
lowest free descriptor, so a connection's descriptor is always below this
bound." No `_Static_assert`, shared constant, or test found ties the two
files' numbers together; the claim is true today only because both
happened to be chosen consistently.
Effect: safety-adjacent, not currently reachable — a real gap only if
`backend/wf_floor.c`'s ceiling ever changes without this table changing
with it, which is exactly the failure mode a design-tree cross-reference
(or a compile-time assertion) would foreclose rather than merely assert in
prose.

**8. Descriptor-kind verification after a completed open reads the
descriptor's mode with one plain `fstat` on the reaping thread rather than
as a second, linked kernel-completion operation — a measured tradeoff
applied identically on Linux and (in its own Windows-shaped form) on
Windows, with no tree record of the general pattern it establishes.**
Alternative: `IORING_OP_STATX` linked to the open's SQE (what the code
replaced), keeping the reaping thread free of any host call at all.
Where: `linux_io_uring.c::wf_linux_decide_open` (lines 819-875, comment
explicitly contrasting the two designs); the same pattern, forced by a
different platform ABI, in `file_windows.c::wf_file_windows_open_at`
delegating to `wf__windows_completion_file_open_at_worker` rather than a
second overlapped call.
Reason (quoted, commit `f141d1e1`, 2026-08-27, "io: decide an open's kind
where it costs one syscall, not one round trip"): "Two ring round trips
cost more than the open they wrap: on the two-CPU Linux container the
eight-wide many-file program ran 152 ms against 116 ms for the bounded
adapter it replaced, and the four-wide one 203 ms against 140 ms ...
Reading the mode of a descriptor the kernel has already produced is an
inode read that cannot wait on anything, so it is one fstat where the
completion is reaped. The same programs then run 119 ms and 141 ms:
parity."
Effect: performance only, and the numbers above are the whole of the
recorded justification; no acceptance difference (both designs classify
the same descriptor the same way). A measured parameter of exactly the
kind `design/compiler/parallel-lowering/parallel-runtime.md` records for
the compute scheduler, with no equivalent entry for this module.

**9. A cluster of measured throughput-tuning constants in this module has
the same character as the compute scheduler's constants the tree already
records, but none of them has a decision-tree entry of its own.**
Specifically: the reap budget (`WF_BRIDGE_REAP_BUDGET = 64`, `bridge.c:74-82`),
the join's pre-park spin window (`WF_BRIDGE_JOIN_SPIN_NS = 10000`,
`bridge.c:919-937`), the window formula's default and byte budget
(`WF_BRIDGE_WINDOW_DEFAULT = 1024`, `WF_BRIDGE_WINDOW_BYTE_BUDGET = 4 MiB`,
`bridge.c:69-73`, `:1664-1695`), and the io_uring submission depth and
completion-queue size (`WF_LINUX_IO_URING_DEPTH = 64`,
`WF_LINUX_IO_URING_COMPLETIONS = 2048`, `linux_io_uring.h:35-47`).
Alternative: promote the ones with a real measured tradeoff into the tree
the way `parallel-runtime.md`'s idle-window and independent-map-splitter
decisions were, or decide once that this module's constants are
ordinary tuning left to code comments — either is a legitimate answer, but
none has been recorded.
Where: as listed above.
Reason: each constant's own comment cites a specific measurement in the
same style as the tree's recorded ones — `WF_BRIDGE_REAP_BUDGET`'s, for
one: "every idle thread took the submission lock to kick and the
completion lock to read a single entry, and 64 connections ping-ponging
through one ring made 720 thousand futex calls a run. At 64 the same run
makes 19 thousand and the round-trip rate doubles" — but no commit or
design record was found weighing whether constants at this granularity
belong in the tree at all.
Effect: none on acceptance; a governance-consistency question for the
owner, since `design/compiler/parallel-lowering/parallel-runtime.md`
already treats constants of this exact evidentiary shape as tree material
for the sibling compute scheduler.

## Summary

- Covered by the tree: 7
- No decision needed: 14 (4 contract/record, 4 errors and retries, 4
  sockets and threads, 2 testing/build)
- Choices without a node: 9 (1 project-premise gap, 5 architecture/backend
  choices, 1 likely defect, 1 unenforced cross-file invariant, 1
  governance/tuning-constant pattern)
