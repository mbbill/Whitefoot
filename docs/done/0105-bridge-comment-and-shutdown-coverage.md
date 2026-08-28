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

Refuted: the block comment in `wf_bridge_spin_for_completion` — though the
re-measurement below narrows the refutation to the comment's attribution
rather than to all of it.

## 1. The refuted comment (`compiler/src/backend/completion/bridge.c`)

The comment claimed that a failed clock "leaves no bound at all: every later
sample answers zero and the deadline is never reached"; that on the native ring
the cost of removing the `started == 0` guard "is the whole run", because "this
spin never calls [`wf_bridge_progress`], so the loop's other exit cannot fire
on its own and the join never ends"; and cited a measurement that "the Linux
io_uring route makes no progress at all" without the guard.

The first claim is false against the code three lines below it. The periodic
sample is `if (now == 0 || now >= deadline) return 0;`, so a failed clock ends
the spin at the first sample — the bound exists, it is at most 64 turns, and
it is that term rather than the `started == 0` guard that provides it. The
cited Linux measurement is false with it: with the guard removed and the term
in place the ring route finishes in the ordinary time.

Re-measured here on the four-lane default probe with `wf_bridge_monotonic_ns`
forced to answer zero — the failed clock the comment is about:

```text
shipped (guard + `now == 0` term), ring       ~113 ms   PASS 6/6
guard removed, `now == 0` term kept, ring     ~105 ms   PASS 6/6
both removed, ring                                      one 12-run sample: 3 PASS, 9 hung
both removed, WF_IO_NO_NATIVE_RING=1                    samples disagree: 6/6 here; the
                                                        gate verifier's 44 runs had 2
                                                        hangs, one STUCK at the probe's
                                                        own 180 s watchdog
```

The last two rows say the old comment's *attribution* was wrong and its route
contrast does not survive either. An unbounded spin fails to end on the native
ring for the reason the old comment gave — this spin never reaps the
completion queue — and the forced-adapter route is not exempt: the gate
verifier hung it too, and its counters show why the route usually finishes
anyway (helpers=0 in most runs; the demand policy declines ~15 000 of 16 000
reads and the caller executes them inline, so nothing needs a helper to
publish). No route has a bound without the `now == 0` term. What the old
comment got wrong beyond that is which line provides the bound: it credited
the `started == 0` guard, and the guard is worth 64 counter reads.

The batch brief predicted the ring row would be a clean pass (117 ms), which
is why it read the whole route contrast as refuted. On this host that outcome
is a minority draw. Every count in the table above is a draw from one host,
not a rate; what is stable is the mechanism, and the rewritten comment states
the mechanism and labels its counts as draws.

The rewritten comment therefore says what the measurement supports: the
`now == 0` term is the bound, the early return only skips the turns before it,
and the term matters because on the ring the loop's other exit is not one this
thread can cause. The unreproducible sentences are gone — the macOS one is not
replaced by a guess, and its Linux analogue is now measured directly, on the
adapter route that batch item 4 below makes reachable here.

## 2. A post-shutdown guard test (gap 1's neighbourhood; the ordering itself stays TSan-evidenced)

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
`main` beside the other growth cases, so it runs on every `completion-test`
harness arm and under the sanitizer builds.

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
`declined` between 12 791 and 15 999 of 16 000 positioned reads (the gate
verifier's sample reached down to 3 692; the exact range is a draw), so the
arm is live rather than vacuous.

```text
default arm   submitted=16000 declined=0     helpers=0  ring=16000  route=native-ring
forced arm    submitted=158   declined=15842 helpers=0  ring=0      route=posix-adapter
```

Mutation, 20 runs of the forced arm with the decline removed
(`wf_file_adapter_transfer_runs_on_caller` replaced by `0`): 12 FAIL, 8 PASS
in this sample (the gate verifier's 20-run sample split 9/11; the split is a
draw, the discrimination is not).
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
