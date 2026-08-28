# Stage A adversarial verification — checkpoint notes

Target: commit 14c89cf3 on batch/0095-loop-pipeline. Base: main b2e2e267.

## Layout (recovered 2026-08-28, prior run had no NOTES.md)
- $SCRATCH/wf-0095-verify/A   = git archive of 14c89cf3 (emitter.rs md5 fc11835847939e390fb4d47978232e4f, matches `git show 14c89cf3:...`)
- $SCRATCH/wf-0095-verify/B   = git archive of b2e2e267 (emitter.rs md5 17c8bd8721def9fb9e0ef12722bd1c2a, matches `git show b2e2e267:...`)
- $SCRATCH/wf-0095-verify/main = second archive of b2e2e267 (same md5)
- CARGO_TARGET_DIR: tgt-A (for A), tgt-B (for B). Both already contain gate/whitefootc.
- ctmp/ = prebuilt C harness, core-read-probe, pure-compute
- oracle-out/ = leftover a.ll/b.ll from a partial IR oracle run (identical, 428468 bytes)

## Step log

### Step 1 — macOS completion suites on A (attack 5, macOS half)
Ran in $SCRATCH/wf-0095-verify/A/compiler with
COMPLETION_TMP=$SCRATCH/wf-0095-verify/ctmp-mac.
- completion-test        EXIT=0 (PASS at helpers 0/1/4 and NOCACHE)
- completion-sanitize    EXIT=0
- completion-core-read-tsan EXIT=0
- completion-tsan        EXIT=2  <-- FAILURE
  helpers=0 PASS, helpers=1 PASS, helpers=4:
  `completion harness: src/backend/completion/harness.c:2985: check failed:
   wf__completion_open_exhaustion_retries() > retries_before`
  Log: $SCRATCH/wf-0095-verify/logs/mac-completion-tsan.log
  Record claims "The whole harness runs clean under the thread sanitizer at
  zero, one and four helpers on macOS" (docs/ongoing/0095-loop-pipeline.md).
  NEXT: check reproducibility (N runs), and whether it also fails without TSan.

### Step 2 — the failure is reproducible, and it is in `make check`
Binaries: $SCRATCH/wf-0095-verify/ctmp-mac/{harness,harness-tsan}
- `TSAN_OPTIONS=halt_on_error=1 WF_IO_HELPERS=4 ./harness-tsan .`  x20 -> 15 FAILED
- `WF_IO_HELPERS=4 ./harness .` (the completion-test build) x30 -> 1 FAILED
- helpers=0 x30 -> 0 failed; helpers=1 x30 -> 0 failed
Failing test: `test_bridge_open_exhaustion_is_retried_once`
(harness.c:2907, assertion at :2985).
`make -C compiler completion-test` is a prerequisite of canonical `check`
(compiler/Makefile:40), and it runs the harness at WF_IO_HELPERS=4.
Root cause (read from file_adapter.c): `wf_file_retire_and_retry` drains only
this adapter's *queued* work; if the queue is empty (`drained == 0`) it returns
the refusal unchanged and never increments `stat_exhaustion_retries`. With four
helper threads the three test opens are taken by three different helpers, each
of which finds nothing left to drain, so no retry is counted and none happens.

### Step 3 — flake rate and first Linux-container attempt
- macOS, completion-test build, `WF_IO_HELPERS=4 ./harness .` x200 -> **13 failures (6.5%)**
  ($SCRATCH/wf-0095-verify/logs/flake-rate-mac.txt)
- First Linux container run (wf0095v from image wf-io-bench:linux, aarch64,
  io_uring enabled) used COMPLETION_TMP on the /verify bind mount and default
  seccomp; all four targets failed for ENVIRONMENT reasons, not branch reasons:
  * completion-test  harness.c:1524 open failure (bind-mount scratch dir)
  * completion-sanitize harness.c:1540 unlinkat(AT_REMOVEDIR) failure (same)
  * completion-tsan / core-read-tsan: TSan CHECK failed
    tsan_platform_linux.cpp:282 personality(ADDR_NO_RANDOMIZE) -> docker seccomp
  RETRY with container-local COMPLETION_TMP and `--security-opt seccomp=unconfined`.

### Step 4 — Linux container, done properly: GREEN (attack 5, Linux half)
Container wf0095v (image wf-io-bench:linux, aarch64, kernel 6.8.0, io_uring
enabled, `--security-opt seccomp=unconfined`, COMPLETION_TMP=/root/ctmp):
- completion-test          EXIT=0 (incl. `native-adapter-probe target=linux-io-uring status=pass`
  and the WF_REQUIRE_LINUX_IO_URING=1 run)
- completion-sanitize      EXIT=0
- completion-tsan          EXIT=0 (helpers 0/1/4)
- completion-core-read-tsan EXIT=0
Logs /verify/logs/lin2-*.log. So the flake is macOS-only, which matches the
root cause: on Linux the three opens go to the ring, where the retry gate is
`in_flight > 1` (deterministically 3 here); on macOS they go to the helper
pool, where the gate is "this helper found queued work to drain".

### Step 5 — doorbell flush-point audit (attack 1, static half)
`wf_bridge_flush_target()` call sites in bridge.c: 846 (demoted open),
1053 pread_direct, 1097 write_direct, 1130 open_at_direct, 1155 status_direct,
1179 close_direct. The one direct entry point with no flush is
`wf__completion_directory_next_direct` (bridge.c:1187) — NOT a defect: on
Linux it returns ENOTSUP without a host call, and the `#if defined(__APPLE__)`
body only runs where there is no ring.

### Step 6 — CI at 14c89cf3 (attack 6)
- io-hosts  run 33155411936  SUCCESS (completion-linux, completion-windows,
  bench-linux, bench-macos-read, bench-linux-read)
- gate      run 33155411971  FAILURE: gate-linux only; gate-macos green (4m50s)
  gate-linux job 98796901955, `make check` -> conformance adapter
  Pass=503 Fail=6 Skip=1, all six `TargetQualification(MissingMapping(
  Operation(12)))`, exactly the six the record names. Confirms the record's
  [QUAL-1] explanation; nothing else fails there.
- IMPORTANT: `completion-tsan` runs only in the io-hosts **Linux** job. On
  macOS the only thing that runs the harness is `make check` ->
  `completion-test`, i.e. the 6.5%-flaky path. gate-macos going green at this
  revision is luck, not evidence; the record's claim that the harness "runs
  clean under the thread sanitizer at zero, one and four helpers on macOS" is
  falsified by Step 2 (15/20 failures).

### Step 7 — IR oracle (attack 4): the claim HOLDS
$SCRATCH/wf-0095-verify/oracle.sh, A vs B binaries rebuilt
from their own exports (`cargo build --profile gate --bin whitefootc`).
630 sources x 3 passes (default, --par, --no-overlap):
  total=1890 emitted=807 no-module=1083 differ=0
No status difference, no IR difference, no stderr difference. Matches the
record's "630 sources each, 269 of which emit a module, all three passes
byte-identical". The pipeline flag is off and the emitted bytes do not move.

### Step 8 — deferred doorbell (attack 1): the claim HOLDS
Probe $SCRATCH/wf-0095-scratch/verify_probe.c, built
against A's completion sources, run in wf0095v.
Schedule: submit an open, DO NOT join, then make a blocking direct open of a
FIFO (O_RDONLY blocks until a writer appears). A watcher thread reads
`wf__completion_linux_io_uring_submission_enters()` *while the main thread is
still inside that blocking call*.
  Linux io_uring route, helpers 0/1/4:
    probe1: route=io_uring
    probe1: enters before=2, seen while blocked=3
  So the doorbell rang BEFORE the blocking call, not after it, and the
  submitted open joined Ok afterwards. macOS: route=posix-helpers, no ring, no
  enter counter, submitted open still joins Ok.

### Step 9 — retire-and-retry (attack 2): REFUTED on the io_uring route
Probe case A, the exact LOOP-PIPELINE.md 2.10 scenario, in SOURCE ORDER:
  1. warm the bridge, then hold the last descriptor
     (RLIMIT_NOFILE soft = held + 1, so the table is full);
  2. `wf__completion_file_close_submit(held, &close_token)`   <- source order
  3. `wf__completion_file_open_at_submit(AT_FDCWD, path, ...)`    first
  4. join the open.
A sequential execution closes, frees the descriptor, and the open succeeds.
Linux io_uring route, helpers 0, 1 and 4, and 15/15 repetitions at helpers=1:
    probe2 caseA: value=-1 error=24 outcome=1 retries=0->1
    probe2 caseA diag: close join value=0 error=0
    probe2 caseA diag: later open value=6 error=0 outcome=0
The retry DID fire (`wf__completion_open_exhaustion_retries` 0 -> 1) and still
published `Err(EMFILE)` = [SYS-7] ResourceExhausted. The close then joins Ok
and a later open of the same path succeeds with descriptor 6, so the
descriptor was free: the refusal the program saw is the pipeline's, not the
host's. macOS (POSIX helper route) gets this case right: value=3, error=0,
outcome=SUCCEEDED at helpers 0/1/4.
Root cause: `wf_linux_publish_completion` marks the entry RETRY_PENDING and
`wf_linux_io_uring_progress` re-kicks it at the end of the SAME reap pass. The
pass only sees CQEs already posted when it read `tail`; the close's completion
is not among them, so the re-attempt races the close and loses. The design
text ("the closes among them, whose descriptors the kernel has already
returned by the time their completion exists") assumes a completion that does
not exist yet. Exactly one re-attempt is allowed, so the loss is permanent.
Why no test caught it: `test_open_exhaustion_retires_owned_work_and_retries`
drives `wf_file_adapter_*` directly — the POSIX adapter, not the ring — and
`test_bridge_open_exhaustion_is_retried_once` asserts only that all three opens
FAIL and that the counter moved. Nothing asserts an Ok is preserved on the ring.

### Step 10 — the window query at K=1 (attack 3): claim NOT ESTABLISHED
A throwaway test (added to the export, then removed) emitted the staged probe
with `IrCompletionWindow::new(1, 0, 1)` and compared it with the sequential
emission of the same source:
    VERIFIER    %t0 = call i64 @wf__completion_window(i64 1, i64 0, i64 1)
    VERIFIER  uses of %t0 elsewhere in the function: 0
    VERIFIER staged==sequential at K=1: false
    VERIFIER sequential joins ["entry"] staged joins ["bb2", "bb7"]
So in Stage A the window's answer is inert: nothing consumes `%t0`, and
carrying is decided entirely by the pipeline's block set. K=1 does not produce
the sequential bytes and does not restrict the schedule. It is behaviourally
harmless today only because one call site still owns one operation record, so
at most one operation is ever in flight. The record's "One is always a legal
answer, and it reproduces the sequential program exactly, so this query can
never make a correct program fail" is a statement about a driver that does not
exist yet; nothing in Stage A demonstrates it.
The runtime side of the query is checked and does hold:
`test_completion_window_answers_at_the_boundaries` passes on both hosts, and
the weak fallback is `ret i64 1`, emitted only where a module asks for one
(`the_window_fallback_is_emitted_only_where_a_module_asks_for_one`).

### Step 11 — the branch's own tests
- `cargo test --profile gate --lib backend::tests::completion` on the A export:
  30 passed, 0 failed (includes the three new tests).
- A local canonical `make check` was NOT run on the export: `git archive` leaves
  no `.git`, so `repository-invariants` fails immediately ("fatal: not a git
  repository"). CI's gate-macos at 14c89cf3 is green and gate-linux fails only
  on the six [QUAL-1] conformance cases (Step 6), so a local rerun would add
  nothing that the 200-run flake measurement in Step 2 does not already say.
  Judgment call: not worth disturbing another agent's worktree
  ($SCRATCH/wf-0095-pipeline) to get a .git.
- Closes DO go to the ring on Linux (`wf__completion_file_close_submit` ->
  `wf_bridge_submit_linux_close` -> `wf_bridge_submit_linux`), so the Step 9
  scenario really is one ring holding both operations.

### Step 12 — Linux flake control
- `WF_IO_HELPERS=4 /root/ctmp/harness /root/ctmp` x100 -> 0 failures
- `WF_REQUIRE_LINUX_IO_URING=1 WF_IO_HELPERS=4 ...` x30 -> 0 failures
So the Step 2 flake is specific to the POSIX helper route (macOS), as the root
cause predicts.

## VERDICT: REFUTED
Two attacks landed.
1. `test_bridge_open_exhaustion_is_retried_once` fails 13/200 (6.5%) in the
   `make check` build at WF_IO_HELPERS=4 on macOS and 15/20 under
   `completion-tsan`. `wf_file_retire_and_retry` counts and performs a retry
   only when it found queued work to drain; with four helpers each of the
   three test opens lands on its own helper and finds none.
2. On the io_uring route, retire-and-retry does not do what it is for: with
   RLIMIT_NOFILE narrowed, source order `close(held); open(path)` yields Ok but
   the pipeline publishes `Err(EMFILE)` 15/15, because the re-attempt is staged
   at the end of the same reap pass and races the close it was supposed to
   have retired.
Attacks 1 (deferred doorbell), 4 (IR oracle) and the Linux half of 5 hold.
Attack 3's claim is not established by anything in Stage A: the window's answer
is inert.
