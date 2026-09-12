# Audit: compute scheduler runtime and native floor against the design tree

Module: `compiler/src/backend/sched/` (`core.c`, `core.h`, `entry.c`, `entry.h`,
`prim.h`, `prim_host.c`, `prim_windows.c`, plus the tests/instruments
`smoke.c`, `deque_probe.c`, `wake_probe.c`, `recursion_budget_probe.c`,
`grant_observer.c`), `compiler/src/backend/wf_floor.c`,
`compiler/src/backend/windows_runtime.c`, and
`compiler/src/backend/windows_namespace_probe.c` (~5,600 lines total). The
compute scheduler's work-stealing runtime and the two "floor" mechanisms that
give every Whitefoot program a defined stack/heap-exhaustion abort and a
compiler-owned descriptor budget. Direction: code to tree — finding choices
the code embodies that `design/` does not record, not the reverse.

Nodes read: `design/skill/SKILL.md`, `design/compiler.md`, `design/language.md`,
`design/compiler/parallel-lowering.md` and its children
`parallel-lowering/parallel-runtime.md` and `parallel-lowering/two-worlds.md`,
and `design/compiler/resource-exhaustion-floor.md`. Also read:
`design/recall-tmp/README.md` and `sources.md`, `design/log.md`'s newest
entries (the parallel-lowering subtree's own review history, dated today), the
finished audits in this directory as the format, `compiler/Makefile`, and
`compiler/src/backend/tests/sched.rs` as evidence. Sourced for reasons:
`git log -S`/`git blame` over the full history, `mcts_mem/whitefoot/parallelism.md`,
`research/investigations/compute-runtime/` (`RESULTS.md`), and, for one
historical cross-check, `research/investigations/proof-derived-parallelism/`
and `research/investigations/io-model/`.

Scope notes:

- `parallel-lowering/two-worlds.md`'s decisions 1-3 (the two-lowering split,
  the clone set, and `--par-sequential-refusal`'s re-entry into a callee's
  existing clone) are compiler *lowering* choices with no native-runtime
  component: they live in `compiler/src/backend/emitter/parallel.rs` and are
  checked by `compiler/src/backend/tests/parallel.rs`, neither in this file
  family. Only decision 4, the recursion-budget arithmetic itself, has a
  runtime half here (`wf__par_recursion_budget`); the per-node budget-carrying
  family that calls it is emitted IR, also outside this audit.
- `parallel-runtime.md` decision 4's may-suspend and I/O-join halves ("a
  may-suspend user call stays on its caller's stack and is never published to
  a compute worker," "an I/O join progresses completion and then waits through
  the native backend... without helping compute tasks") are implemented in
  `compiler/src/backend/completion/`, covered by the sibling
  `runtime-completion.md` audit in this directory. Only the pure-compute half
  (join/steal/spin/park on native threads) is checked here.
- `windows_runtime.c` and `windows_namespace_probe.c` mostly implement
  `spec/kernel-spec.md`'s [SYS-7]/[SYS-8]/[SYS-10] Windows target row (native
  `NtCreateFile`/`NtQueryDirectoryFile`/Winsock translation) — a
  `design/language/system-interface` concern, not a node this audit was asked
  to read. They are covered below only where they touch a resource,
  concurrency, or scheduler-adjacent choice; their file/socket-semantics
  translation is treated as mechanical and not itemized operation by
  operation.
- `compiler/src/backend/wf_floor_windows.c` is the Windows analog of
  `wf_floor.c` and is **not** in this audit's file list; it is named only for
  orientation and never audited.
- `research/investigations/proof-derived-parallelism/gap-hunt-findings.md` and
  `research/investigations/io-model/SCHEDULER-FINDINGS.md`/`PARK-ON-MISS.md`
  describe two earlier, since-retired scheduler generations (a fiber/
  managed-stack machine and a "v1" hand-out with no grain control). Their
  findings (e.g. "hand-out is unconditional, no grain control") are checked
  against the current code and found superseded, not treated as live defects;
  where one still bears on a current constant it is cited as historical
  confirmation, not as an open finding.

## 1. Covered by the tree

- `parallel-lowering.md` (decision: native queues, helper lanes, wakeups, and
  completion ports are target-private protocol state and never Whitefoot
  shared storage) — `compiler/src/backend/sched/core.c`'s `struct
  wf__par_lane`/`struct wf__par_slot` and every deque/wait-station field are
  C-internal state with no Whitefoot type; the whole cross-language ABI is
  four opaque-pointer entry points whose only exposed value is a raw frame
  address (`core.h:14-21`, "these u64 ABI values must stay 64-bit on Windows
  LLP64 as well as POSIX LP64"; `core.c:261-263`, "the emitted ABI carries
  only this opaque frame address").
- `parallel-lowering.md` (decision: on Windows a module with compute offers
  requires the compiler-owned runtime at link time; on every target an invalid
  worker configuration fails before the program body; a host that starts
  fewer workers than asked keeps the ones that started; a startup that yields
  none runs the parallel body through its ordinary-call fallback) —
  `compiler/src/backend/sched/entry.c::wf__sched_setting` (an out-of-range
  `WF_WORKERS` prints a diagnostic and calls `_Exit(1)` before any Whitefoot
  code runs, `entry.c:7-37`); `core.c::wf__par_start`'s partial-startup loop
  keeps every thread it managed to create (`core.c:1178-1198`);
  `wf__par_attach`/`wf__par_acquire_lane` return `NULL` once
  `wf__par_lane_count` is zero, so the compiled fallback (an ordinary call)
  runs (`core.c:1209-1249`). Exercised by `smoke.c`'s `owner-fail`,
  `worker-fail`, `partial`, and `partial-three` modes and by
  `.github/workflows/io-hosts.yml`'s live `WF_WORKERS=100` assertion
  (`whitefoot scheduler: WF_WORKERS must be an integer from 0 through 64`).
- `parallel-lowering/parallel-runtime.md` (decision 1: each worker thread owns
  a fixed-capacity deque of offer slots; an offer is a push onto the offering
  thread's own deque; idle threads steal the oldest entry; a join reclaims its
  own offer and runs it inline before waiting on a stolen one) —
  `core.c::wf__par_push`/`wf__par_pop`/`wf__par_steal` (`core.c:751-827`) and
  `wf__par_join` (`core.c:1266-1300`, pops its own lane first and calls
  `wf__par_wait` only once draining and stealing both find nothing).
- `parallel-runtime.md` (decision 2: a full deque refuses the offer and the
  call runs inline, so refusal falls on the deepest pending fork and the
  capacity doubles as a grain floor with no tuned constant) —
  `wf__par_acquire_lane` returns `NULL` once `lane->free_head < 0`
  (`core.c:1225-1249`), and every caller (`smoke.c::sum`; the pattern
  `tests/loop_split.rs` pins in emitted IR) runs the callee inline on that
  refusal instead of blocking.
- `parallel-runtime.md` (decision 3: a waiting join works instead of sleeping,
  draining its own deque and then stealing, and parks only when no work
  exists anywhere) — `wf__par_wait`'s loop checks the target's own `DONE` flag
  first, then scans and executes, before ever reaching `wf__par_stay_hot`'s
  park branch (`core.c:1010-1068`).
- `parallel-runtime.md` (decision 4, compute-only half: compute tasks run on
  persistent native worker threads and their ordinary stacks; a join executes
  an unstolen target on the joining thread; a stolen target lets the joining
  thread run and steal other compute work before a bounded spin, yield, and
  condition-wait slow path) — `wf__par_start` creates ordinary
  `pthread`/`_beginthreadex` threads once, for the life of the process
  (`core.c:1150-1207`); `wf__par_join`/`wf__par_wait`/`wf__par_worker_main`
  implement exactly this sequence (`core.c:1010-1132`, `1266-1300`). (The
  may-suspend and I/O-join halves are out of scope; see the scope note
  above.)
- `parallel-runtime.md` (decision 5: the deque stress probe, run under
  `make -C compiler sched-deque-test` and repeated under ThreadSanitizer, is
  the runtime's concurrency check, with separate startup, join, completion,
  and exhaustion tests for the boundaries it does not reach) —
  `compiler/Makefile`'s `sched-deque-test`/`sched-deque-tsan` targets build
  and run `deque_probe.c` exactly as described (`Makefile:320-336`); `smoke.c`
  supplies the separate startup (`owner-fail`/`worker-fail`/`partial*`/
  `startup-delayed`/`startup-partial`), completion-tail
  (`check_registered_wait_reuse`, the held-notification test), and ring-wrap
  cases the probe does not reach; the ten `WF_SCHED_TEST`-gated hooks in
  `core.c` (`wf_sched_test_before_wait`/`_before_steal_read`/`_after_steal`/
  `_before_done`/`_after_done`/`_signal_locked`/`_after_notify`/
  `_allow_owner_wait`/`_allow_worker`) are exactly the "hold native threads at
  otherwise unobservable race windows" mechanism the node and `smoke.c`'s own
  header describe. The Makefile's own comment on `sched-deque-tsan` continues
  the node's reasoning nearly verbatim: "The managed-stack enumerator is
  retired with its scheduler... It does not enumerate all interleavings,
  prove weak-memory ordering or liveness... this is not equivalent to the
  retired managed-stack state-space gate."
- `parallel-runtime.md` (decision 6: every worker thread runs on a stack the
  same size as the entry's, one runtime-owned constant exported across the
  language boundary rather than the environment's limit) — `entry.c`'s weak
  default `wf__floor_stack_bytes` (1 MiB, used only when `wf_floor.c` is not
  linked, `entry.c:49`) is overridden by `wf_floor.c`'s strong definition
  (`wf_floor.c:61`, returning `WF_FLOOR_STACK_BYTES`); `wf__par_start` passes
  exactly that value to every worker's `wf_prim_thread_start`
  (`core.c:1187-1188`). No `RLIMIT_STACK`/environment stack-size query appears
  anywhere in `prim_host.c` or `core.c`, matching the node's rejected
  alternative.
- `parallel-runtime.md` (decision 7: the idle-window architecture —
  `WF_PAR_IDLE_WINDOW_US` of 1,000 microseconds sampled once per 1,024 spin
  rounds, taken only when the pool's lanes at start are at or below the CPUs
  the process may run on, an oversubscribed or CPU-count-unknown pool keeping
  the fixed 1,024-round/16-yield bound, `WF_SCHED_TEST` forcing the window to
  zero, and the publish-epoch shortcut that turns a gated round into one
  shared-line read) — `core.c:141-150` (`WF_PAR_SPIN_ROUNDS`,
  `WF_PAR_IDLE_WINDOW_US`, the `WF_SCHED_TEST` override), `core.c:1150-1168`
  (`wf__par_start`'s oversubscription test, gating `wf__par_idle_window_us` on
  `wf_prim_online_cpus() != 0 && requested <= cpus`), `core.c:942-1005`
  (`wf__par_should_scan`/`wf__par_stay_hot`), `core.c:294-338` (the publish
  epoch). **Drift, evidentiary rather than behavioral:** the tree's decision
  line, extended today, cites "the 16.3 microsecond park-and-wake measured on
  the development host... against an 18.9 nanosecond scan round"
  (`design/compiler/parallel-lowering/parallel-runtime.md:13`), but `core.c`'s
  own comment on the same constants — also dated 2026-09-12 — says the file
  was "Re-measured at this revision on 2026-09-12... reads a park-and-wake of
  10.3 us... and a spin-round floor of 11.6 ns" and explicitly calls
  16.3 us/18.9 ns "the reading this comment carried before, taken on
  2026-09-11 on another machine of that class" (`core.c:25-34`). The 16.3/18.9
  figures are exactly `mcts_mem/whitefoot/parallelism.md`'s 2026-09-11 entries
  (`d47223c0`'s measurement and `5bf91fd7`'s decision, both "16.3 us"/
  "18.9 ns"), and today's `design/log.md` entry for this exact change says the
  parallel-runtime content "is recovered from
  `mcts_mem/whitefoot/parallelism.md`" — so the tree's citation is a faithful
  transcription of mcts_mem, one that `core.c`'s own re-measurement (also
  today) has already superseded. The derived constants (1,024 rounds,
  1,000 us) and the gating architecture are unaffected either way; only the
  tree's stated evidence for the round bound is stale relative to the code it
  describes as of the same day.
- `parallel-runtime.md` (decision 7's closing paragraph: each spin round also
  issues the hardware hint an SMT sibling needs, reasoned rather than measured
  in isolation) — `prim.h::wf_prim_spin_hint` (`prim.h:23-31`: `pause` on
  x86, `yield` on aarch64, `YieldProcessor()` on Windows), called once per
  round from `wf__par_stay_hot` (`core.c:980`, `994`).
- `parallel-runtime.md`/`two-worlds.md` (decision: the initial recursion
  budget is the runtime's own answer, `floor(log2(64 * lanes))` clamped to 24,
  one lane's answer when no pool exists, and zero from the weak stub every
  module carries) — `core.c::wf__par_recursion_budget` (`core.c:1383-1400`)
  computes exactly this; `WF_PAR_RECURSION_LEAVES_PER_LANE` (64,
  `core.c:250-252`) and `WF_PAR_RECURSION_MAX_BUDGET` (24, `core.c:255`) are
  the two named constants; the emitter's `define weak i64
  @wf__par_recursion_budget() { ret i64 0 }` (`emitter/parallel.rs`, outside
  this file family) is the "zero from the weak stub."
  `recursion_budget_probe.c` and `tests/sched.rs::
  recursion_budget_follows_the_pool_width_and_stops_at_its_ceiling` pin the
  exact table the node states (0/1→6, 2→7, 4→8, 8→9, 16→10) and the clamp at
  24 with `WF_PAR_RECURSION_LEAVES_PER_LANE` overridden to `1ull<<40`.
- `parallel-runtime.md` (decision 8: the independent-map splitter's work unit
  is 150,000 and its oversubscription cap 16 chunks per lane, both
  compile-time constants an A/B build can override) — `entry.c:139-141`
  (`WF_PAR_SPLIT_WORK_UNIT`, 150000, with a roughly 100-line comment
  reproducing the exact crossover measurements the node's reason cites) and
  `core.c:237-239` (`WF_PAR_SPLIT_OVERSUBSCRIBE`, 16); `wf__par_split_budget`
  (`core.c:1331-1370`) implements `span / ceil(work/weight)` combined with
  `lanes * WF_PAR_SPLIT_OVERSUBSCRIBE` by a minimum, exactly as `entry.c`'s
  own comment states; `WF_SPLIT_WORK` is the per-process diagnostic override
  the node's text also names.
- `design/compiler/resource-exhaustion-floor.md` (decision 1: exhausting the
  stack or the heap ends the process by a defined abort that first writes one
  fixed record naming only the exhausted resource class and no source
  construct, rule, function, address, depth, or size; an unavailable compute
  worker or task slot is a refused offer and never an exhausted resource) —
  `wf_floor.c`'s `WF_FLOOR_STACK_RECORD` (`"{\"resource\":\"stack\"}\n"`,
  `wf_floor.c:175`) and `wf__floor_handler`'s `abort()` on a classified guard
  hit (`wf_floor.c:252-255`) carry no rule/function/address/depth; contrasted
  directly with `wf__par_acquire_lane`'s ordinary `NULL` return on a full or
  oversized frame (`core.c:1229-1245`), which is an ordinary refusal, never an
  abort.
- `resource-exhaustion-floor.md` (decision 2: the entry runs on a stack the
  runtime sizes, and a fault below a thread's stack keeps the host's own
  disposition, because an explicit prologue check would spend the headroom it
  guards) — `wf_floor.c::wf__floor_install` installs one `SIGSEGV`/`SIGBUS`
  handler (`wf_floor.c:335-353`); `wf__floor_handler`'s `!guard_hit` branch
  restores `SIG_DFL` and re-raises rather than reporting (`wf_floor.c:225-250`),
  preserving "same signal, same status, same core dump" for a fault that is
  not the floor's own. No per-prologue stack-pointer check exists anywhere in
  this file family (the target's own stack-probing attribute the node names is
  emitted elsewhere, by the Rust backend).

## 2. No decision needed

Scheduler core and primitives:

- The work-stealing deque is implemented directly over C11 atomics (raw
  `top`/`bottom`/CAS, no borrowed library), with SEQ_CST reserved for exactly
  the operations the file's own correctness argument needs and relaxed
  elsewhere — the header names the defect class its atomics repair ("atomic
  ring cells and SC thief reads repair the two deque races," `core.c:1-4`),
  and this is the ordinary way to implement decision 1 above correctly, not a
  further choice.
- The publish epoch's RELEASE/ACQUIRE pairing, never SEQ_CST
  (`wf__par_epoch_bump`/`_read`, `core.c:331-338`) — the file's own inline
  proof ("WHY THE EPOCH CANNOT BE MISSED," `core.c:929-941`) shows this is the
  minimum ordering a monotonic "did anything change" counter needs.
- `_Alignas(WF_PAR_CACHE_LINE)` on every hot, contended field (`top`/`bottom`,
  the wait station, the steal counter, the epoch cell — `core.c:275-326`),
  with `WF_PAR_CACHE_LINE` fixed at 128 rather than split per architecture —
  ordinary false-sharing avoidance, and 128 is a conservative constant safe on
  both common 64-byte lines and machines with adjacent-sector prefetch; the
  file states which neighbor each padding protects.
- Two `_Thread_local` variables (`wf__par_self`, `wf__par_attached`,
  `core.c:664-666`) as the only way a thread learns which lane it owns — the
  ordinary mechanism for a thread-affine runtime with no other per-thread
  state to piggyback on.
- `wf__sched_once`'s hand-rolled CAS-based once-initializer
  (`entry.c:39-48`) instead of `pthread_once`/C11 `call_once` — needed because
  the same C11 source must compile unmodified on POSIX and Windows, which
  share no common once-primitive header.
- The free-list threaded through `slot.next_free` for the 64 fixed slots
  (`wf__par_prepare`, `core.c:1134-1148`) and the three-state
  `FREE`/`PENDING`/`DONE` slot lifecycle (`core.c:256-258`) — the ordinary
  allocator-free-list technique for a fixed-capacity, allocation-free pool.
- The weak-symbol seams (`wf__floor_stack_bytes`, `wf__sched_helper_ceiling`
  in `entry.c:49-50`; `wf__floor_attach_thread` in `prim_host.c:132`/
  `prim_windows.c:121`) letting a probe link only the pieces it needs and get
  an inert default for the rest — ordinary incremental-linkage technique,
  matching the Makefile's own grouping comment ("The core owns current-stack
  join/help/steal, entry.c owns configuration and startup policy, and the
  native leaf supplies threads and per-lane waiting," `Makefile:64-65`).
- The POSIX/Windows primitive split (`prim_host.c`/`prim_windows.c` behind one
  `prim.h`, with `core.c`/`entry.c` themselves platform-agnostic) — ordinary
  porting-layer structure.
- `wf__sched_setting`'s numeric-environment-variable parsing (`strtol`, a
  range and trailing-character check, `_Exit(1)` on any violation,
  `entry.c:7-37`) and `WF_PRIM_SETTING_BYTES` = 64 as its scratch buffer
  (`prim.h:58`) — ordinary defensive parsing for a process-level setting read
  before any Whitefoot code runs. The unchecked `errno` on a `strtol` overflow
  is provably harmless here: the resulting `LONG_MAX` always fails the very
  next `<= ceiling` comparison, so an astronomically large `WF_WORKERS` still
  fails closed — the same conclusion
  `research/investigations/proof-derived-parallelism/gap-hunt-findings.md`
  reached ("latent, not reachable") when it examined this exact overflow
  shape under the predecessor scheduler generation's naming
  (`WF_PAR_MAX_WORKERS`, since renamed and rearchitected).
- `wf_prim_online_cpus`'s preference order on both platforms (process affinity
  mask, then a platform-specific total, then a portable fallback,
  `prim_host.c:63-90`, `prim_windows.c:40-62`) — the ordinary "most specific
  fact first" pattern for one question answered two ways per platform, and
  both files state the same reason (a narrowed process is oversubscribed at
  its narrowed count regardless of the machine's total).
- `WF_SCHED_STATS`'s default-on steal counter (`core.h:11-12`, incremented in
  `wf__par_steal`, `core.c:822-825`) and the `WF_SCHED_REPORT`-gated
  diagnostic line (`entry.c:144-155`) — ordinary, effectively free (one
  relaxed atomic add per steal) instrumentation, exercised both on and off by
  the deque probe (`Makefile:320-327`, `sched.rs:79-94`); several
  `parallel-runtime.md`/`parallel-lowering.md` decisions cite this exact
  counter as their own supporting evidence, so it is foundational
  infrastructure rather than an independent policy question.

The floor and the Windows target row:

- The handler's async-signal-safety discipline (one `write(2,...)` then
  `abort()`, no allocation/stdio/lock, bounds captured outside signal context,
  `wf_floor.c:190-261`) — mechanically required by the node's own
  "async-signal-safe facilities" constraint, not an independent choice of
  mechanism.
- `WF_FLOOR_RED_ZONE_BYTES` = 128 applied uniformly on both qualified
  architectures, although only x86-64 has a nonzero ABI red zone
  (AArch64's is architecturally zero) — a harmless, conservative
  simplification (one constant, safe on both, dwarfed by the page-sized guard
  band it is added to) rather than a per-architecture policy question.
- `WF_FLOOR_ALTSTACK_BYTES` = 64 KiB for the handler's alternate signal stack
  (`wf_floor.c:135`) — generous headroom for a handler that does one syscall
  and one `abort()`; the one bug on record about this stack (fixed in batch
  0090's `25ac56ef`) was about *where* the mapping landed relative to the
  guard page, never about this size being wrong.
- `wf__handle_credits`'s atomic compare-exchange retry loop for spending one
  descriptor credit (`wf_floor.c:108-124`) — the ordinary lock-free counter
  pattern for a value multiple threads decrement concurrently.
- `windows_runtime.c`'s UTF-16↔UTF-8 conversion
  (`wf__windows_utf8_measure`/`_copy`, surrogate-pair validated) and native
  `NTSTATUS`→Win32 error mapping (`wf_windows_nt_error`,
  `wf__windows_error_from_socket`) — mechanical, [SYS-7]/[SYS-8]-cited
  translation of the already-decided Windows target row, reusing
  `qualification.rs`'s existing error-class table rather than inventing one
  (the socket-error function's own comment says so).
- `windows_runtime.c`'s descriptor registry (`wf_windows_registry`, one
  exclusive `SRWLOCK`-guarded, grow-only array indexed by CRT descriptor,
  `windows_runtime.c:408-450`) and `WF_WINDOWS_COMPONENT_MAX_BYTES` = 510
  (`windows_runtime.c:73`, exactly NTFS's 255-UTF-16-code-unit component limit
  in bytes) — an ordinary bookkeeping table for a fact no host query answers,
  and a mechanical consequence of the target filesystem's own limit,
  respectively.
- `windows_namespace_probe.c`'s numbered-failure-code pattern and exit-77
  skip-as-fail-closed convention, and `grant_observer.c`'s `atexit`-registered
  report instead of a `__attribute__((destructor))` — ordinary,
  well-commented test-instrument engineering. The latter is forced by a
  documented, previously-observed MSVC/UCRT ordering defect (a COFF
  terminator's `fputs` crashing after the UCRT's stream locks are released),
  not a stylistic choice; the former is consumed exactly as documented by
  `.github/workflows/io-hosts.yml` ("the runner could not create the
  reparse-point fixture... remains fail-closed").

## 3. Choices without a node

**1. `WF_FLOOR_STACK_BYTES` reserves 1 GiB of stack per thread — a chosen
round number, not a measured one.**
`wf_floor.c:57` fixes every thread's stack reservation, entry and compute
worker alike, at exactly one gibibyte
(`((size_t)1024u * 1024u * 1024u)`), and this is also the true ceiling on
recursion depth reachable through the scheduler, since `wf__par_start` gives
every worker thread this same size (`core.c:1187-1188`, covered under
decision 6 in section 1).
Alternative: any other generous-but-bounded reservation — a smaller one (the
introducing commit shows 8 MiB is not distinguishable in cost) or a larger
one.
Where: `compiler/src/backend/wf_floor.c:57-61`.
Reason (quoted, commit `178d4f69`, 2026-08-23, "backend: give exhaustion a
stack of our own and a defined death"): "The floor costs +0.078 ms per
process and +65,560 bytes of peak footprint, none of it the reservation: an
8 MiB thread and a 4 GiB thread measure the same, and so do a 16 KiB and a
64 KiB alternate stack." No commit before or after this one touches the
literal (`git log -S "1024u * 1024u * 1024u"` finds only this one), and
neither this commit nor any later one states why 1 GiB rather than another
generous value.
Effect: real but narrow. Sixteen lanes at 1 GiB each commit 16 GiB of address
space, measured harmless by the same evidence `parallel-runtime.md`'s own
decision 6 cites ("it costs nothing measurable... the pages are untouched"),
so the number is not costly — but it is a live ceiling with no recorded
comparison against a neighboring value, only the general principle that
bigger is free.
Draft Decision: "The runtime reserves 1 GiB of stack per thread, the entry
thread and every compute worker alike, because raising a stack reservation
costs address space alone — an 8 MiB and a 4 GiB thread measured identically
— while a very large runaway recursion should still end in a bounded time,
instead of a reservation sized to a specific measured depth or a still
larger one."

**2. `WF_SCHED_MAX_THREADS` fixes the runtime at 64 worker lanes, with a hard
pre-body failure above it, and no recorded reason for the number.**
`core.h:10` sizes every fixed-capacity array in the runtime
(`wf__par_lanes`, `wf__par_threads`, and the `WF_PAR_TRACE` rings) and is the
ceiling `wf__sched_setting` enforces on `WF_WORKERS`
(`entry.c:156-167`): a host with more than 64 online CPUs is silently
narrowed to 64 by default, while a user who explicitly asks for more than 64
workers gets a whole-process `_Exit(1)` before the program body runs
("`whitefoot scheduler: WF_WORKERS must be an integer from 0 through 64`",
live-tested in `.github/workflows/io-hosts.yml`).
Alternative: a higher fixed ceiling for larger current server core counts, or
a dynamically sized (heap-allocated at startup) lane table with no fixed cap.
Where: `compiler/src/backend/sched/core.h:10`; consumed throughout `core.c`
(e.g. `wf__par_lanes[WF_PAR_MAX_LANES]`, `core.c:287`) and
`entry.c::initialize` (`entry.c:156-163`).
Reason: no reason recorded. The literal `64` first appears as `WF_PAR_MAX_LANES`
in `408dd34e` (2026-08-21, "compiler: replace the lane scan with per-thread
work-stealing deques"), whose message is entirely about the offer protocol
and never mentions the thread ceiling; `74f7fa97` (2026-09-04, "Land the
scheduler core on real threads, and record the handoff point") later split
`core.h` out and introduced `WF_SCHED_MAX_THREADS` as the shared name, turning
`WF_PAR_MAX_LANES` into an alias for it (`core.c:10`), but that commit's body
does not mention the number either. `git log -S` on both names finds no
commit body discussing why 64 rather than another value.
Effect: acceptance-adjacent. A legitimate request for more than 64 workers on
a large host is a hard, whole-process failure rather than a silently narrowed
pool, and no design-tree node states that 64 is the intended ceiling for this
research compiler as opposed to an untouched placeholder from before
very-high-core-count hosts were a live consideration.
Draft Decision: "The runtime supports at most 64 worker lanes, fixed at
compile time and enforced as a hard startup failure for an explicit
`WF_WORKERS` above it, because a fixed cap keeps every lane table a static
array with no runtime allocation, instead of a dynamically sized lane table
or a higher or lower fixed cap."

**3. `WF_SCHED_FRAME_BYTES` caps a hand-out frame at 256 bytes, with no
recorded reason for the number.**
`core.h:5` is the largest frame `wf__par_acquire_lane` will ever hand out
(`core.c:1229-1231`: a request over this size returns `NULL` immediately,
so the caller's ordinary-call fallback runs and the call is never offered to
the pool at all).
Alternative: a smaller or larger fixed frame budget. The introducing commit
explains the *architecture* ("A lane's frame is bounded, and a call whose
frame is larger is simply never granted a lane") but not this specific
number.
Where: `compiler/src/backend/sched/core.h:5`;
`compiler/src/backend/sched/core.c::wf__par_acquire_lane`.
Reason: no reason recorded in `b6f496b4` (2026-08-21, "compiler: claim a lane
before building the hand-out frame," the commit that introduced the
claim-first frame design) or any later commit touching the literal `256`.
Effect: acceptance-adjacent for performance, not for correctness — a
recursive function whose per-call state exceeds 256 bytes always takes the
inline path, silently and by design, but the specific threshold separating
"offered" from "always inline" is unexplained and untested against
neighboring values.
Draft Decision: "A hand-out frame is refused a lane above 256 bytes, because
a lane's frame storage is fixed-size and allocation-free, instead of a larger
per-frame budget or a variable, heap-backed frame."

**4. `WF_PLACEMENT_PAD` is a second shipped-but-inert measurement instrument,
structurally identical to the already-ruled `WF_PAR_TRACE`, without its own
explicit disposition.**
`core.c:673-712` is a never-called, `noinline`, 587-byte function compiled
only under `#ifdef WF_PLACEMENT_PAD`, defined by nothing that ships — the
same "no shipped build, no gate target and no test defines it... `cmp` on the
object says so" property the file claims for `WF_PAR_TRACE` one section
above it. It exists so the compute A/B scoreboard can shift one arm's code
placement by exactly 587 bytes without changing its behavior, isolating code
placement as its own confound.
Alternative: delete it now that the question it was built to answer has been
measured and answered
(`research/investigations/compute-runtime/RESULTS.md`, "what code placement
alone is worth, with a shifted null arm"), the way a one-shot research script
is retired after use; or keep it on the same explicit footing the owner has
already given `WF_PAR_TRACE`.
Where: `compiler/src/backend/sched/core.c:673-712`.
Reason: `mcts_mem/whitefoot/parallelism.md` (2026-09-11, `e69592dc`) states
why it was built — isolating code-placement effects from the A/B scoreboard,
"so an arm can be identical in behaviour and shifted in placement" — the same
class of justification the owner has accepted for `WF_PAR_TRACE`. No
design-tree node or later commit says whether this second instrument should
now be removed or is meant to stay indefinitely.
Effect: none on any shipped build (the same byte-identity property
`WF_PAR_TRACE` claims for itself applies here). Purely a repository-hygiene
question — CLAUDE.md's "no bulk dumps... a script ships wired to a caller...
or an explicit one-shot deleted after use" — of whether a second, structurally
identical dead-by-default instrument needs the owner's own ruling or can be
assumed to fall under the one already given to `WF_PAR_TRACE`.
Draft Decision: "`WF_PLACEMENT_PAD` stays in `core.c` as a compile-time-gated
code-placement measurement instrument, defined by nothing that ships, on the
same footing as `WF_PAR_TRACE`, because both exist only to let the compute
A/B scoreboard isolate one confound at a time and neither changes any shipped
byte, instead of deleting it now that its first measurement is complete."

**5. The descriptor floor's reserve and ceiling are a third resource-limiting
mechanism in `wf_floor.c` that `resource-exhaustion-floor.md` does not
mention at all.**
The same file that implements the stack/heap abort also implements the
[SYS-10] `HandleFactory`'s native fact — how many descriptors the process may
still open — as a checked, source-visible credit rather than an abort:
`WF_FILE_RUNTIME_RESERVE` (64, `wf_floor.c:83`) is subtracted from the
process's descriptor limit to leave headroom for what the runtime itself
holds, and `WF_FILE_CAPACITY_CEILING` (2^20, `wf_floor.c:84`) caps an
`RLIM_INFINITY`/huge limit before the subtraction. `resource-exhaustion-floor.md`
scopes itself explicitly to "the stack or the heap the runtime supplies" and
explicitly excludes "an unavailable compute worker or task slot" from being
an exhausted resource, but says nothing about descriptors at all — a third
resource class, living in the very same file, on a third disposition (neither
abort nor silent refusal, but a counted, checkable `HandlePermit`).
Alternative: an unbounded reserve (scan the descriptor table instead of
guessing what the runtime holds — rejected in-code for its per-descriptor
syscall cost on a limit that "may be a million"), or a reserve/ceiling pair
recorded as its own decision beside the stack/heap one.
Where: `compiler/src/backend/wf_floor.c:63-124`
(`wf__file_capacity_init`, `wf__handle_reserve`).
Reason: no reason recorded for the specific numbers 64 and 2^20 in commit
history, mcts_mem, or the research investigations searched; the file's own
comment states the qualitative goal ("Being below the true limit is the whole
promise") but not why 64 reserved descriptors or a million-descriptor ceiling
are the right figures rather than other generous ones.
Effect: acceptance-relevant in a way stack/heap exhaustion is not — a program
that opens more files than this credit allows gets an ordinary, checked
`FileOpenOutcome`-shaped refusal rather than a process abort, which is a
deliberate and sensible difference from the stack/heap floor's own record —
but no node currently says that this file also owns a second, differently
disposed resource-limiting mechanism, or grounds its two numbers.
Draft Decision: "The descriptor factory reserves 64 descriptors for what the
runtime itself holds and caps its native limit at 2^20 before that
subtraction, because a scan of the descriptor table to find the runtime's own
usage costs one syscall per possible descriptor on a limit that may be a
million, instead of an exact accounting or an unbounded reserve."

**6. `compiler/Makefile`'s Windows cross-link boundary check cites a symbol,
`wf__sched_entry_stack`, that exists nowhere in the current codebase, and
describes the current weak/strong seam backwards — reads as a stale comment
carried over from a retired scheduler generation, not a deliberate choice.**
The `completion-windows-cross` target's boundary-check comment says: "`wf__sched_entry_stack`
is the core's own seam (design section 5): the floor declares it weakly in
its own translation unit, a link that carries the core overrides it, and a
link that does not takes the default beside the reference"
(`Makefile:543-546`). `wf__sched_entry_stack` does not appear in any current
`.c`/`.h`/`.rs` file (confirmed by a full-repository grep); it existed only
under the earlier, retired managed-stack scheduler generation
(`git log -S` finds it in `92b19e1a`, `babf5c7f`, `58d39d6a`, `7421a258`,
`4ff01e88`, `eb1ed585`, and last in `6816e9bd`, the 2026-09-10 commit that
replaced that whole generation with the current native-thread runtime). The
actual seam performing this exact role today is `wf__floor_stack_bytes`
(`design/compiler/parallel-lowering/parallel-runtime.md` decision 6, and
section 1 above) — and it is declared weakly in `entry.c` (the **core**,
`entry.c:49`) and strongly in `wf_floor.c` (the **floor**, `wf_floor.c:61`),
the reverse of what the comment states for which side is weak. The "design
section 5" cross-reference is also unresolvable: the current `design/` tree
has no numbered sections.
Alternative: the comment could name `wf__floor_stack_bytes` and describe it
correctly (core weak, floor strong) — the underlying `nm`-based
undefined/defined check a few lines later already handles weak COFF
externals generically (`w`/`W`/`v`/`V` join the defined set) and does not
itself depend on which specific symbol or which side is weak, so the
*mechanism* the comment is explaining still enforces the right invariant
today; only the prose illustrating it is wrong.
Where: `compiler/Makefile:543-548`.
Reason: no reason recorded; this reads as an ordinary rename left behind by
`6816e9bd`'s wholesale replacement of the managed-stack scheduler with the
current one, rather than anything deliberate. No later commit touches these
lines.
Effect: documentation only. The Windows cross-link boundary check itself
(the `nm`/`comm`-based comparison of undefined against defined symbols,
`Makefile:549-580`) is unaffected and still correct; a maintainer reading this
comment to understand the seam it is illustrating would be pointed at a
symbol that does not exist and told the wrong translation unit is the weak
one.

## Summary

- Covered by the tree: 14
- No decision needed: 18 (11 scheduler core/primitives, 7 floor/Windows)
- Choices without a node: 6 (5 architecture choices, 1 likely defect)
