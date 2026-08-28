# 0095 — staged loop pipeline, Stage A prerequisites

Branch `batch/0095-loop-pipeline`, from `main` at `b2e2e267`.

The design is `research/investigations/io-model/LOOP-PIPELINE.md` §1, §3 and §7
"Batch 2", with its §9 probe results as the measurement of record, and
`research/investigations/io-model/FIRST-PRINCIPLES.md` §13-16 as the ownership
contract the runtime work has to keep.

Stage A lands the four prerequisites §3.9 and §7 name. **None of them changes a
permission verdict or a published byte**, and no writer-visible surface is
added anywhere: there is no attribute, environment variable, or source spelling
for a window, a depth, or a schedule. Stage B lands the judgment ([PAR-3] in
`compiler/src/semantic/staged_permission.rs`), the lowering, and the
measurement, and this record moves to `docs/done/` then.

## 1. `wf__completion_window(span, slot_bytes, ceiling)`

`compiler/src/backend/completion/bridge.c`. The runtime's own answer to how
many iterations of one loop it will carry in flight, asked once per loop entry
and never per iteration — the discipline `wf__par_split_budget` already keeps.

Each argument is a bound and none is a request; a zero places no bound. The
runtime's own bound is **half** its process-wide operation capacity, because a
loop owning every record would push the rest of the program onto the
capacity-wait path, where a full ring degrades to a blocking direct call
(§3.9 item 6). Half of 64 is 32, which is where the hand-written ceiling
program measured its optimum (§9.2). Where the ring is the target engine the
ring's capacity bounds it too; where a bounded helper pool is,
`WF_BRIDGE_MAX_HELPERS` does. A 4 MiB byte budget divided by `slot_bytes` is
the last bound, so the design's 16 MiB privatized buffer gets one.

**One is always a legal answer** and reproduces the sequential program exactly,
so the query can never make a correct program fail. That is why the weak
fallback a link without the completion unit gets returns one.

The fallback is `COMPLETION_WINDOW_FALLBACK` in
`compiler/src/backend/emitter/completion.rs`, emitted only where a module
actually asks for a window — the same discipline `PARALLEL_SPLIT_BUDGET_FALLBACK`
already keeps, and the reason no module that stages no loop changes by a byte.

## 2. Deferred io_uring doorbell

`compiler/src/backend/completion/linux_io_uring.c`. Staging a submission-queue
entry is a store to a page the kernel already maps, so it costs no system call;
`io_uring_enter` is what tells the kernel the entry exists. Probe A measured
15,360 enters against 2,048 deferred, taking 15.6 ms of system CPU and 14.4 ms
of wall time off the eight-wide program — in a barrier-bound single-lane
program the saved kernel CPU is all on the critical path (§9.1).

The doorbell now rings from four places, and between them they are what makes
deferring safe:

- `wf_linux_io_uring_progress` kicks before it reaps or sleeps, and every join
  and park in the bridge reaches the kernel through it;
- the submission path kicks when the submission queue is full, before it
  declares a capacity wait, because a deferred queue fills with this thread's
  own backlog first;
- `wf_bridge_flush_target` kicks before every blocking direct host call and
  before the demoted-open path hands its caller back to one, so a submitted
  open can never sit unkicked behind a blocking `openat` (§3.9 item 3);
- the retire-and-retry path below kicks its re-attempt itself.

`wf_linux_io_uring_statistics.submission_enters` counts them, and the distance
between it and `submissions` is the whole of what deferring buys.

## 3. Retire-and-retry on descriptor exhaustion

Design §2.10. A schedule that keeps several iterations in flight holds several
descriptors where source order holds one, so a host limit the sequential
program never reaches could turn a correct program's `Ok` into an
`Err(ResourceExhausted)`. [SYS-10] is explicit that a `FilePermit` promises no
native descriptor, and [PAR-1]'s exhaustion clause excuses only what an
implementation spends on overlapping — never the descriptors the program's own
opens consume.

So an open the host refuses for want of a descriptor is not published. Both
target routes retire the work the runtime still owns and re-attempt once:

- the ring holds the entry out of the reap pass as retry-pending, so every
  other ready completion — the closes among them, whose descriptors the kernel
  has already returned by the time their completion exists — publishes first,
  and the next kick re-stages the open behind them;
- the bounded POSIX adapter runs its queued work, which is work the sequential
  execution performs anyway, and then re-attempts.

Exactly one re-attempt, so an exhausted host cannot turn one `open_file` call
into unbounded work; if the second attempt also fails, that is the outcome
source-order execution produces and the program sees it. Cost is paid only on
the exhausted path, so nothing on the correct path changes and T3 holds.

**What the runtime cannot do, and Stage B must.** §2.10 says the adapter
"completes every older slot in index order (which runs their compiler-derived
closes)". A compiler-derived close is writer code, and completion never invokes
writer code (FIRST-PRINCIPLES.md §14), so the runtime can only give back the
descriptors held by operations *it* still owns — a close already submitted, not
one the driver has not reached. That covers the pipeline's steady state, where a
slot's close is submitted when its read retires and is often still in flight
when the slot's next open is. It does not cover a window whose older slots have
not been driven that far, so the driver has to retire its own slots in index
order on a published `ResourceExhausted` before it treats one as the program's
answer. Stage B owns that half.

## 4. Back-edge-tolerant joins

`compiler/src/backend/emitter.rs` and `emitter/completion.rs`, opting in on
`IrFunction::completion_pipeline` (`compiler/src/lowering.rs`). One rule
decides both halves of the staged schedule:

> A block the pipeline names never joins. Every block it does not name joins
> everything outstanding, in hand-out order.

The first gives a loop's back edge the right to carry work across it — today's
unconditional `emit_all_completion_joins` in `emit_terminator` is the whole of
the round barrier §3.4 measures. The second is the drain, and it needs no
separate machinery: retiring every outstanding operation in hand-out order is
what a block has always done.

The one thing a straight-line walk gets wrong is that a carrying region has
several exits — the loop's normal exit and every typed exit out of its body —
and each of them must retire the same operations. `pipeline_outstanding`
records what the region handed out and seeds each exit from it, so no path
leaves an accepted operation owned by nobody. A carrying set no exit leaves is
refused outright as `BackendFailure::UnretiredCompletionOperation`.

`emit_stackless_root` does not consult any of this, and does not need to:
`StacklessPlan::build` admits only a single-block function ending in a return,
and a staged loop has a loop. If Stage B ever widens either side, the two paths
have to be reconciled rather than left to disagree quietly.

What this does **not** yet carry is the per-slot storage index. One call site
still owns one operation record, so a site inside a carrying region that
submits again while its earlier operation is outstanding is refused rather than
handed the first operation's storage. That is the driver's work (§3.4, §3.6
item 2), and the driver is Stage B.

## Evidence

- Harness (`compiler/src/backend/completion/harness.c`), run by
  `make -C compiler completion-test` at four helper settings and, on Linux,
  once more with `WF_REQUIRE_LINUX_IO_URING=1`:
  `test_completion_window_answers_at_the_boundaries`,
  `test_a_submitted_operation_is_kicked_before_it_waits`,
  `test_open_exhaustion_retires_owned_work_and_retries`,
  `test_bridge_open_exhaustion_is_retried_once`.
- Backend (`compiler/src/backend/tests/completion.rs`):
  `a_staged_loop_carries_completion_across_its_back_edge`,
  `the_window_fallback_is_emitted_only_where_a_module_asks_for_one`,
  `a_carrying_region_with_no_exit_is_refused`.
- Negative controls run by hand on Linux, both of which failed the harness as
  they must: restoring the immediate kick fails the doorbell test, and
  disabling the ring's retry fails the exhaustion test.
- `make -C compiler completion-tsan`, new here and wired into the io-hosts
  Linux job. `completion-core-read-tsan` links neither the bridge nor the ring
  by design, so the deferred doorbell's staging, the retire-and-retry hand-back
  and the readiness flag the flush reads had nothing checking them. The whole
  harness runs clean under the thread sanitizer at zero, one and four helpers
  on macOS and, with `WF_REQUIRE_LINUX_IO_URING=1`, in the Linux container.
- IR identity: every `.wf` under `tests/programs`, `tests/codegen` and
  `tests/conformance/cases` compiled with `whitefootc --emit-llvm` at this
  revision and at `main`, under the default, `--par` and `--no-overlap` — 630
  sources each, 269 of which emit a module, all three passes byte-identical.

## Status

- [x] item 1 — window query, weak fallback, harness boundaries
- [x] item 2 — deferred doorbell and its four flush points
- [x] item 3 — retire-and-retry on both target routes
- [x] item 4 — carrying and draining, with the IR-identity oracle
- [x] `completion-test`, `completion-sanitize`, `completion-tsan`,
      `completion-core-read-tsan` — green on macOS and in the Linux container
- [x] canonical `make check` — green
- [x] `io-hosts` at `549a5a67` — every job green, including `completion-linux`,
      which runs the four new harness tests and the new `completion-tsan` step
      on a real x86-64 Linux kernel with io_uring, and `completion-windows`
- [x] `gate` at `549a5a67` — `gate-macos` green

## The one red job, and why it is not this branch's

`gate-linux` fails on six conformance cases, all with the same reason:

```text
Fail sys14-list-outcome-exhaustive            want Run(0)
Fail sys14-list-zero-range                    want Run(0)
Fail sys14-directory-release                  want Run(0)
Fail sys14-entry-kind-closed                  want Run(0)
Fail accept-sysfile-two-permits-shared-directory   want Accept
Fail accept-par3-staged-denied-opaque-cursor       want Accept
  TargetQualification(MissingMapping(Operation(12)))
conformance adapter: Pass=503  Fail=6  Skip=1
```

That is [QUAL-1]: Linux has no approved [SYS-14] directory-enumeration row, so
every case that enumerates a directory reaches `Unsupported` rather than its
declared verdict. It is not a source-language rejection and no verdict was
touched here.

The discriminating observation, rather than the plausible one: the concurrent
`batch/0096-darwin-handoff`, which changes the Darwin helper path and nothing
this branch touches, fails the same six cases with the same counts on the same
base. And `batch/0094-linux-directory-row` — "io: qualify Linux directory
enumeration and un-declare its host limits" — is green. This branch adds and
modifies no conformance content and does not touch
`compiler/src/backend/qualification.rs`; `git diff main` over that file is
empty. The row is that other branch's work, and `gate-linux` goes green here
when it lands.
