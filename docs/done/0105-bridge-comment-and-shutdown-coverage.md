# Batch 0105 — the refuted spin comment, and coverage for what 0096 left untested

Branch: `batch/0105-bridge-comment-and-shutdown-coverage`, from
`integration/2026-08-28c` at `100b37cd`. Deliverables: the rewritten
`wf_bridge_spin_for_completion` comment, a completion test for the
post-shutdown guard, an honest shutdown precondition in `file_adapter.h`, a
test-only Linux arm that reaches the POSIX adapter route, and this record.

## What the contract verification found

Batch 0096's round-3 runtime changes were re-measured on a Linux host with a
working io_uring. Verdict: three of the four changes held, one comment was
refuted, and three gaps were named that no test in the tree covered.

Held:

- the helper-growth cases (`test_pool_grows_when_operations_wait`,
  `test_helper_growth_stops_at_the_helper_storage`) decide what they claim —
  the clamp mutation is caught;
- `bridge_default_probe`'s route assertions are real assertions rather than a
  printed route split;
- the shutdown ordering itself is right: with the pre-92260349 ordering
  restored, a reader racing a shutdown draws a ThreadSanitizer report at
  `wf_file_adapter_queued` naming the destroyed mutex.

Refuted: the block comment in `wf_bridge_spin_for_completion`.

## 1. The refuted comment (`compiler/src/backend/completion/bridge.c`)

The comment claimed that a failed clock "leaves no bound at all: every later
sample answers zero and the deadline is never reached"; that on the native ring
the cost of removing the `started == 0` guard "is the whole run", because "this
spin never calls [`wf_bridge_progress`], so the loop's other exit cannot fire
on its own and the join never ends"; and cited a measurement that "the Linux
io_uring route makes no progress at all" without the guard.

All three are false against the code immediately below the comment. The
periodic sample is `if (now == 0 || now >= deadline) return 0;` — a failed
clock ends the spin at the first sample, so the bound exists and is at most 64
turns. And sibling lanes reaping the completion queue raise the ready-event
count, so the loop's other exit does fire without this thread calling
`wf_bridge_progress`.

Measured today on the 4-lane default probe, Linux, io_uring route:

```text
guard present                          116 ms   PASS
guard removed                          116 ms   PASS
guard and the `now == 0` term removed  117 ms   PASS
```

The comment now states only the two things that are true: the `now == 0` term
of the periodic sample is what bounds the spin under a failed clock, and the
`started == 0` early return only skips at most 64 futile counter reads before
that sample would have ended it anyway. The two measurement sentences are
deleted rather than restated — the Linux one was false, and the macOS one
cannot be re-measured from this host, so no replacement claim is made.

## 2. Coverage for the shutdown ordering (gap 1)

The 92260349 fix — `wf_file_adapter_shutdown` clears `initialized` before
destroying the condition variable and the mutex, and clears it whatever the
teardown reported — had no test. The verifier restored the old ordering and
every completion target stayed green.

`test_shutdown_refuses_every_later_entry` in
`compiler/src/backend/completion/harness.c` now grows the pool against a
blocked queue, shuts the adapter down, and asserts that
`wf_file_adapter_queued`, `wf_file_adapter_helper_count`,
`wf_file_adapter_transfer_runs_on_caller`, `wf_file_adapter_wait_verdict` and
`wf_file_adapter_set_helper_cap` are all refused at the guard, and that a
second `wf_file_adapter_shutdown` returns `EINVAL` rather than joining threads
that are gone and destroying the mutex twice. It is wired into the harness's
`main` beside the other growth cases, so it runs on all four
`completion-test` arms.

### Red/green

```text
green   shipped tree, WF_IO_HELPERS 0 / 1 / 4        PASS
red     the store skipped in wf_file_adapter_shutdown
        harness.c:2875 check failed:
          wf_file_adapter_helper_count(&adapter) == 0
```

The red side is the store *skipped*, which is exactly what the pre-92260349
conditional (`if (first_error == 0) { ... }`, after the destroys) does on any
teardown that reports an error.

What the case does not discriminate, measured rather than assumed: the literal
pre-92260349 ordering still passes it. On a clean teardown `first_error` is
zero, so the old code's store lands too — only later. The store's *position*
closes a concurrent window, and a concurrent window is not what a
single-threaded case decides; the verifier's evidence for the position is the
ThreadSanitizer report quoted above, which stays the evidence for it. This
case holds the other half, which is a real property on its own: the record
says it is unusable whatever the teardown reported. The test's own comment
says so, so a later reader does not mistake its scope. This diverges from the
prediction in the batch brief, which expected the double-shutdown assertion to
be the one that failed under `first_error == 0` semantics; it does not, and the
measurement above is what the record rests on.

## 3. An honest precondition list (`file_adapter.h`) (gap 2)

The shutdown precondition named `wf_file_adapter_submit` and
`wf_file_adapter_transfer_runs_on_caller` and read as an exhaustive list. It
was not one: `wf_file_adapter_queued` and `wf_file_adapter_set_helper_cap`
take `queue_lock` behind the same guard, and `wf_file_adapter_helper_count`
and `wf_file_adapter_wait_verdict` read record state behind it. The sentence
now names the class — no thread is inside any entry point of this adapter —
and says the two below are named because they are the two a delivered program
reaches on its own, not because overlapping the others is safe.

## 4. A Linux arm that reaches the adapter route (gap 3)

`bridge_default_probe`'s adapter-branch assertion — on the POSIX adapter
route the demand-driven policy must have declined a positioned read or grown a
helper — is dead code on any Linux host whose io_uring is available, because
the ring takes every positioned read. The negative control that batch 0096's
record rests on was therefore protected by the Darwin CI host alone.

`WF_IO_NO_NATIVE_RING=1` skips `wf_linux_io_uring_init` in
`wf_bridge_initialize`, which puts the process on the route a kernel without
io_uring already takes; no new runtime path is added. It is runtime policy of
the same class as `WF_IO_NOCACHE` and `WF_IO_HELPERS` and test-only within
that class: no Whitefoot source names it, no accepted program changes meaning
under it, no byte differs with it set. It is documented at its reader in
`bridge.c`, where `WF_IO_HELPERS` is documented, and read once there.

`completion-default-route-test` gains one arm that runs the same probe under
it on Linux, and the probe gains one assertion: if the setting is written and
the ring answered anyway, the run fails rather than passing as a second copy
of the native-ring arm.

Measured here, shipped build, 8 runs of the forced arm: PASS every time, with
`declined` between 12 791 and 15 999 of 16 000 positioned reads, so the arm is
live rather than vacuous.

```text
default arm   submitted=16000 declined=0     helpers=0  ring=16000  route=native-ring
forced arm    submitted=158   declined=15842 helpers=0  ring=0      route=posix-adapter
```

Mutation, 20 runs of the forced arm with the decline removed
(`wf_file_adapter_transfer_runs_on_caller` replaced by `0`): 12 FAIL, 8 PASS.
The 8 passes are not a hole. The assertion is a disjunction on purpose — the
policy's two branches are decided by the same measurement, and which one a run
gets is a property of how fast this host's reads are — and in those runs the
adapter grew helpers instead, which is the other branch of the same policy
actually firing. A mutation that removed both branches would be caught every
time.

## 5. Recorded, not acted on

- The init comment in `file_adapter.c` says a probe that re-initializes a
  record under live readers draws a ThreadSanitizer report between
  `atomic_init(&adapter->mean_execute_ns, 0)` and the acquire load of the same
  field in `wf_file_adapter_wait_verdict`. It did not reproduce on Linux in
  1000 init/shutdown cycles across 17 985 183 calls. That is not a refutation:
  the window is a few instructions wide, and the comment describes what the
  race would be rather than claiming a reproduction rate.
- `completion-windows-cross` was not runnable on this host — no mingw — so
  nothing in this batch is evidence about the Windows units.

## Approval classes

No specification change: `spec/kernel-spec.md` is untouched.

No conformance change: nothing under `conformance/` is added, modified,
deleted, or renamed. The cases added here are ordinary compiler tests in the
completion harness.
