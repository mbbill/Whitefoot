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

**One is always a legal answer**: the runtime may carry one iteration in flight
and that is the schedule the sequential program already runs, so no answer this
query gives can make a correct program fail. That is why the weak fallback a
link without the completion unit gets returns one.

Stage A does not demonstrate it, and the record earlier said it did. The value
the emitter binds has no consumer yet: what carries and what drains is decided
entirely by the pipeline's block set, so answering one restricts nothing and
reproduces nothing. It is harmless here only because one call site still owns
one operation record, which keeps at most one operation of a site in flight
whatever the answer. The claim becomes checkable when Stage B's driver consumes
the answer, and checking it there is Stage B's evidence to produce.

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
- the retire-and-retry path below rings the doorbell for the re-attempt it
  stages, while the thread that decided to stage it still owns the decision —
  including where the submission queue was full, which is the one case an
  earlier repair left staged behind a completion-only sleep.

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

The rule is about a *moment*, and it is one rule over one ledger.

The moment is the host attempt. A refused open must ask "has this runtime given
a descriptor back since I asked the host", not "is anything running now": an
operation that retired between the failing `openat` and the decision has
already made room, and a rule that read the present would answer from a world
where nothing is left in flight and publish an exhaustion that no longer
exists.

The ledger is process-wide, because a descriptor is. It holds one retirement
generation, incremented whenever any operation on any engine retires — a close
completing, a refused open being published, any completion that ends an
operation — and one in-flight count, which every operation either engine
accepts joins, blocking direct calls included for as long as they execute. It
lives with the completion core (`contract.h`, `runtime.c`) because the core is
the one unit both target engines and the bridge link; the bounded POSIX adapter
and the Linux ring are qualified separately and share no other code.

Every refused open, whichever engine attempted it, then follows four steps:

1. read the generation before the host attempt;
2. on `EMFILE`/`ENFILE`, if the generation has moved since, re-attempt once;
3. otherwise, while an operation is in flight anywhere else, wait for the next
   retirement, then re-attempt once;
4. otherwise publish the refusal, which is the outcome source order produces
   too.

Exactly one re-attempt per refused open, so an exhausted host cannot turn one
`open_file` call into unbounded work; if the second attempt also fails, that is
the outcome source-order execution produces and the program sees it. A retry is
counted exactly where the second host attempt is made, so a published refusal
never reads as one. Cost is paid only on the exhausted path, so nothing on the
correct path changes and T3 holds.

Waiting terminates because a waiter counts itself out of "in flight anywhere
else": when every operation still in flight is a waiter, the earliest of them
publishes its refusal, and that publication is itself a retirement which
releases the others. Two more things are counted out alongside the waiters, and
both are the same fact — an operation nobody may wait for, because waiting for
it would be waiting for the waiter. One is local: the operations this waiter's
own thread would have to run itself, which is the queue a drained sibling's
suspended caller still owes and is nothing at all for a held ring entry, since
that blocks no thread. The other is global: an operation whose thread is inside
another operation is *deferred* for as long as it is.

How each route waits is mechanical, and it is the only per-route thing left.
The ring holds the refused entry — in a state no doorbell may stage — and asks
the ledger again on every progress pass; no thread blocks, and the doorbell for
a staged re-attempt rings immediately, including where the submission queue was
full, so a re-attempt never sits behind a completion-only sleep. The bounded
adapter's thread first runs the work its own queue still owes, oldest first,
which is the order the program wrote and the order that reproduces the
sequential outcome, and only then sleeps on the ledger's signal. A blocking
direct open waits by driving the engines and parking on the runtime's own
endpoint, because on a target with a ring it may be the only thread that can
reap the completion it is waiting for.

### What this record said before, and what was false

Five shapes of this were wrong before the rule was stated that way. None was
caught by a test; the first two were found by the Stage A verification, the
next two by re-verification of the repair, and the fifth by reading the repair
for the same class of defect.

- the ring re-staged the refused open at the end of the *same* reap pass, so
  the source order `close(held); open(path)` — which succeeds sequentially —
  lost the race to that close and published `Err(EMFILE)`, deterministically,
  at every helper setting;
- the adapter asked the host a second time only when it found queued work to
  drain. With several helpers each simultaneous open lands on its own helper
  and looks at an empty queue, so the descriptors a retirement would return —
  held by operations running on the *other* helpers — were never waited for and
  the refusal was published instead;
- **"an open the host refuses ... is not published while this runtime is still
  holding a descriptor it could give back" was false across engines.** Each
  engine asked only about its own operations. A program with a read in flight
  on the helper adapter and an open refused on the ring published `Err(EMFILE)`
  with no re-attempt at all, deterministically at zero, one and four helpers.
  That program is an ordinary one rather than a contrivance:
  `wf__completion_file_read_submit` has no ring route, so a read and an open
  submitted together are on two different engines by construction;
- **"Each of them then gets its one re-attempt" was false on the adapter
  route.** A refused open ran a *sibling* open out of its own queue and
  published that sibling's refusal with no second attempt at all, while
  spending its own on a close that had not finished. The source order
  `close(held); open(p); open(p)` produces exactly one `Ok`; that shape
  produced none, 81 times in 200 at four helpers and 74 in 200 at two on macOS,
  and 112 in 200 before the first repair;
- the give-up decision read the generation, then the in-flight and waiter
  counts. A retirement whose generation increment landed after the first read
  and whose in-flight decrement landed before the second was invisible to both,
  so the decision could publish a refusal a descriptor had already answered. It
  re-reads the generation at that exit now, and the ledger's schedule point is
  named so a test can stand exactly between the two reads.

Each of the three routes is covered by a test that fails without the fix, and
so is the third exit above:
`test_bridge_open_waits_for_the_other_engine`,
`test_bridge_one_of_two_opens_behind_a_close_succeeds`,
`test_bridge_open_behind_a_submitted_close_succeeds`,
`test_open_exhaustion_waits_for_another_engine` and
`test_a_retirement_between_the_ledger_reads_is_not_missed`.

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
and each of them must retire the operations that reach it.
`pipeline_outstanding` records what the region handed out, together with the
block that handed it out, and each exit is given exactly what the carrying
blocks that actually reach it left in flight. Which blocks those are is read
from the edges (`pipeline_feeder_blocks`), not from how far the walk has got, so
a drain retires the same window however the blocks around it are numbered.

That is both halves and for every drain, the first one the walk reaches
included. The walk arrives at an exit holding the straight-line simulation,
which on a branching region is both too little and too much: an operation
started on a branch it has not passed is missing, and one started on a
*sibling* branch that cannot reach this exit at all is still in hand. Adding the
first without removing the second would emit a join for an operation no path
through that block ever started, reading a token no target ever wrote. An
earlier repair seeded only the later drains and left the first with whatever the
walk had; `a_drain_retires_only_what_the_branches_that_reach_it_started` is the
test, and with the removal deleted it shows that exit joining two operations
where one branch reaches it.

Two properties of the descriptor are refused outright as
`BackendFailure::UnretiredCompletionOperation`, both before a line of the
function is emitted:

- a carrying set no exit leaves, which has a path on which an accepted
  operation is never joined at all;
- a carrying block that starts an operation and is numbered at or after one of
  its own drains. Blocks are emitted in index order, so that drain is walked
  into before the hand-out it must retire exists and would emit a bare
  terminator with the operation still owned by a target — silently, because a
  function carrying a pipeline is exempt from the straight-line check at the
  end of emission, exactly because a carrying block may legitimately be the
  last block emitted. A latch numbered after a typed exit it reaches is fine
  and stays admitted: a block that starts no operation leaves that exit nothing
  to be missing. Stage B has to produce a descriptor whose hand-outs precede
  the exits that retire them, or teach the emitter to write its blocks in an
  order that does; the refusal is what keeps the difference from being silent.

`emit_stackless_root` does not consult any of this, and does not need to:
`StacklessPlan::build` admits only a single-block function ending in a return,
and a staged loop has a loop. If Stage B ever widens either side, the two paths
have to be reconciled rather than left to disagree quietly.

What this does **not** yet carry is the per-slot storage index. One call site
still owns one operation record, so a site inside a carrying region that
submits again while its earlier operation is outstanding is refused rather than
handed the first operation's storage. That is the driver's work (§3.4, §3.6
item 2), and the driver is Stage B.

Neither does it carry the wait a K-slot ring needs. The rule above says a
carrying block never joins, which is right for the back edge and not enough for
the body: reusing slot *i* means waiting for exactly the one older operation
that holds it, and nothing else. Stage B has to add a selective
single-operation join for that — a primitive the rule admits, since it retires
one named operation rather than everything outstanding — and must not reach it
by exempting the carrying block from the rule.

## Evidence

- Harness (`compiler/src/backend/completion/harness.c`), run by
  `make -C compiler completion-test` at four helper settings and, on Linux,
  once more with `WF_REQUIRE_LINUX_IO_URING=1`:
  `test_completion_window_answers_at_the_boundaries`,
  `test_a_submitted_operation_is_kicked_before_it_waits`,
  `test_open_exhaustion_retires_owned_work_and_retries`,
  `test_open_exhaustion_waits_for_another_engine`,
  `test_bridge_open_exhaustion_is_retried_once`,
  `test_bridge_open_behind_a_submitted_close_succeeds`.
- Backend (`compiler/src/backend/tests/completion.rs`):
  `a_staged_loop_carries_completion_across_its_back_edge`,
  `the_window_fallback_is_emitted_only_where_a_module_asks_for_one`,
  `a_carrying_region_with_no_exit_is_refused`,
  `a_drain_emitted_before_the_hand_out_it_retires_is_refused`.
- Every test named above fails without the fix it covers, which is what makes
  it evidence rather than decoration. Each control is the shipped harness with
  one hunk removed, built into a scratch copy so no other agent's tree moves.
  `test_bridge_open_waits_for_the_other_engine`, with the ring restored to
  deciding from its own in-flight count, fails at zero, one and four helpers in
  the Linux container, on the check that says the runtime held the refused open
  rather than publishing it.
  `test_bridge_one_of_two_opens_behind_a_close_succeeds`, with owed work
  restored to running with no re-attempt of its own, fails at two and four
  helpers on macOS on `successes == 1` — both opens refused where source order
  produces one `Ok`.
  `test_a_retirement_between_the_ledger_reads_is_not_missed`, with the
  generation's re-read at the give-up exit deleted, fails at zero, one and four
  helpers on macOS: the decision answers `UNREACHABLE` where a descriptor had
  already come back.
  `a_drain_retires_only_what_the_branches_that_reach_it_started`, with the
  sibling removal deleted, reports the first exit joining two operations where
  one branch reaches it.
  `test_open_exhaustion_waits_for_another_engine`, with the adapter's old
  `drained == 0` give-up restored, fails 30 of 30 runs at zero, one and four
  helpers on macOS.
  `test_bridge_open_behind_a_submitted_close_succeeds`, with the ring restored
  to `14c89cf3`, fails 21 of 150 runs at one helper and 21 of 150 at four in
  the Linux container, where the shipped ring fails 0 of 150 at each under the
  same load in the same minute. That one control is a rate and not a certainty
  because the kernel is free to run the submitted close before the open, in
  which case the open is never refused and the old code has nothing to get
  wrong.
  `a_drain_emitted_before_the_hand_out_it_retires_is_refused` emits a module
  with the ordering check removed, and the loop's `break` exit in that module
  carries a bare `ret` with no `wf__completion_file_join` while the
  later-numbered typed exit has one.
- The two probes the verifiers wrote are kept and re-run rather than replaced:
  they are the shape a harness test cannot reach, a whole process narrowing its
  own `RLIMIT_NOFILE`. At this revision, at zero, one and four helpers on both
  hosts, `attack_probe` passes every case — including `A1` (`oks=1 errs=1`),
  `A3` (four refused together, four refusals, three re-attempts), `A4` (the
  cross-engine open, `value=8 error=0` on Linux where it was `-1/EMFILE`), `A5`
  (64 opens against a full table) and `A6` (300 repetitions, `lost_ok=0`) — and
  `verify_probe`'s deferred-doorbell and `close(held); open(path)` cases pass
  as before. The two-opens probe reports `lost_ok=0` in 200 repetitions at
  zero, one, two and four helpers on both hosts, against 81 in 200 at four and
  74 in 200 at two before this rule.
- Negative controls run by hand on Linux: restoring the immediate kick fails
  the doorbell test, and disabling the ring's retry fails the exhaustion test.
- `make -C compiler completion-tsan`, new here and wired into the io-hosts
  Linux job. `completion-core-read-tsan` links neither the bridge nor the ring
  by design, so the deferred doorbell's staging, the retire-and-retry hand-back
  and the readiness flag the flush reads had nothing checking them.
- What the sanitizers actually say, replacing an earlier claim in this record
  that the harness ran clean under the thread sanitizer on macOS at every
  helper setting. At `14c89cf3` it did not:
  `test_bridge_open_exhaustion_is_retried_once` failed 13 of 200 runs (6.5%) at
  `WF_IO_HELPERS=4` in the `completion-test` build and 15 of 20 under
  `completion-tsan` — a flake inside canonical `make check`, since
  `completion-test` is one of its prerequisites and runs the harness at four
  helpers.

  Two separate defects fed that one symptom, and it took both fixes to clear
  it. The first is the adapter's give-up above. The second is the test itself:
  it asserted that a re-attempt had happened, which is not a property of three
  independent opens under the corrected rule, because a helper pool that
  finishes each open before the next is submitted holds nothing at the moment
  of each refusal and publishing the refusal unchanged is then correct. That
  assertion is now the bound the rule does promise — at most one re-attempt per
  refused open — and the property it was reaching for is carried by the two
  deterministic tests above. Its own comment says so.

- Repetition counts at this revision, at every helper setting the suites use
  and two hosts. macOS (`completion-test` build): 200 runs each at zero, one,
  two and four helpers, 0 failures; `completion-tsan` 20 runs at each of the
  same four, 0 failures. Linux container (`wf-io-bench:linux`, aarch64, kernel
  6.8, io_uring enabled, `--security-opt seccomp=unconfined`): 200 runs each at
  zero, one, two and four helpers, 0 failures; `completion-tsan` 20 runs at
  each of the same four, 0 failures; `completion-sanitize` (ASan + UBSan)
  passes. Two helpers is in the list because that is where the two-opens defect
  showed at 74 in 200 and the shipped suites do not run it. The probes above,
  15 runs each at zero, one and four helpers on both hosts, 0 failures.
  Earlier counts in this record measured earlier revisions and are superseded
  by these.
- `make -C research/experiments/io-completion-bench verify` on macOS: every line
  of every workload — the hand-written native shape, the `--no-overlap`
  sequential reference, and the shipping overlapped build — publishes
  `17098009301725298919 00000000000071024640`, `many_files_loop` among them.
  The runtime changes here are on the exhausted path; this is the check that
  they left the ordinary one publishing the same bytes.
- Toolchain coverage. `gate-linux` builds the harness with the host `cc`, which
  on `ubuntu-24.04` is GCC, while `io-hosts`'s `completion-linux` builds it with
  clang. A first push of this repair compiled on macOS clang and on container
  clang and still failed `gate-linux`, because GCC rejects a discarded `write`
  under `-Werror` where clang does not. The harness is now also built and run
  with `gcc` in the Linux container, which is the same diagnostic set the gate
  applies.
- IR identity: every `.wf` under `tests/programs`, `tests/codegen` and
  `tests/conformance/cases` compiled with `whitefootc --emit-llvm` at this
  revision and at `main`, under the default, `--par` and `--no-overlap` — 630
  sources each, 269 of which emit a module, all three passes byte-identical.

## Status

- [x] item 1 — window query, weak fallback, harness boundaries. The claim about
      what an answer of one means is now stated as a property of the runtime,
      not as something Stage A demonstrates
- [x] item 2 — deferred doorbell and its four flush points
- [x] item 3 — retire-and-retry as one rule over one process-wide retirement
      ledger, asked at the moment of the host attempt, with the two per-engine
      gates it replaces gone and a deterministic test for each route and for
      the one exit that could miss a retirement
- [x] item 4 — carrying and draining, with every drain including the first
      given exactly what the blocks that reach it started, the out-of-order
      descriptor refused, and the IR-identity oracle re-run: 630 sources,
      3 passes, 1,890 compilations, 807 modules, 0 differences
- [x] `completion-test`, `completion-sanitize`, `completion-tsan`,
      `completion-core-read-tsan` — green on macOS and in the Linux container,
      with the repetition counts above
- [x] canonical `make check` — green
- [x] `io-hosts` and `gate` run on the pushed branch. `completion-linux` is
      the one that matters most here: it runs the harness, the sanitizers and
      `completion-tsan` on a real x86-64 Linux kernel with io_uring, so both
      new exhaustion tests are exercised on the ring rather than only in the
      aarch64 container. `gate-linux` stays red on the six [QUAL-1]
      conformance cases below and on nothing else

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
