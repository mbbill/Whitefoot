# Stage A REPAIR re-verification (adversarial) — checkpoint notes

Target: ad04ae2d on batch/0095-loop-pipeline (repair of 14c89cf3, which was REFUTED).
Prior verifier notes: $SCRATCH/wf-0095-verify/A/NOTES.md

## Layout
- export: $SCRATCH/wf-0095-verify/A2 (git archive ad04ae2d; verified md5
  linux_io_uring.c 37279bfcdd54051920c18b6e4bc1d4fa, file_adapter.c 4da239fe56699ee50ea01e2b8853fd6f)
- my work dir: $SCRATCH/wf-0095-verify/A2work
- reference export of 14c89cf3: $SCRATCH/wf-0095-verify/A
- main b2e2e267 export: $SCRATCH/wf-0095-verify/B

## Step log
### Step 0 — setup DONE
Read A/NOTES.md and verify_probe.c. Exported A2, digests match git.
NEXT: read the diff of the runtime changes (linux_io_uring.c, file_adapter.c, emitter.rs).

### Step 1 — macOS harness flake measurement (F2) — PASSES, with a negative control
Binaries in $SCRATCH/wf-0095-verify/ctmp2 built with the exact
COMPLETION_HARNESS_CFLAGS of each revision's Makefile.
- NEW (ad04ae2d) harness x200 helpers=4 -> 0 failures (twice: mac-200-h4.txt, mac-200-h4b.txt)
- NEW harness x200 helpers=0 -> 0; x200 helpers=1 -> 0
- NEGATIVE CONTROL: OLD (14c89cf3) harness x200 helpers=4 -> 20 failures (10%),
  every one `harness.c:2985: check failed: wf__completion_open_exhaustion_retries() > retries_before`
  So my setup does reproduce the refuted defect, and the repair removes it.
NEXT: TSan x20 macOS; then Linux container; then F1 probe; then F4; then IR oracle.

### Step 2 — macOS TSan (F2) — PASSES, with a negative control
- NEW harness-tsan x20 at helpers 4/1/0 -> 0 failures each
- NEGATIVE CONTROL: OLD harness-tsan x20 helpers=4 -> 15 failures (same 15/20 the first verifier saw)

### Step 3 — Linux container wf0095v2 (image wf-io-bench:linux, aarch64 6.8.0,
seccomp=unconfined, container-local /root tmp). Sources copied to /root/src2.
- verify_probe (unmodified, from wf-0095-scratch) x15 at helpers 0/1/4 -> 45/45 PASS
    probe1: route=io_uring; enters before=2 seen while blocked=3
    probe2 caseA: value=6 error=0 outcome=0 retries=0->1   <-- F1's lost Ok is back
    probe2 caseB: value=-1 error=24 outcome=1              <-- genuine exhaustion still Err
- NEGATIVE CONTROL: same probe built against A (14c89cf3) -> caseA value=-1 error=24, FAIL.
- harness x200 at helpers 4/1/0 -> 0 failures each
- harness-tsan x20 at helpers 4/1/0 -> 0 failures each (632 __tsan syms, real build)
- harness-sanitize (ASan+UBSan) -> PASS
NEXT: my own attacks (retry whose 2nd attempt fails; failing close; capacity wait while held),
F4 emitter, IR oracle, CI logs, F3 doc claims.

### Step 4 — my own attacks: TWO NEW LOST-Ok DEFECTS (both reproduced deterministically)
Probes: A2work/attack_probe.c, A2work/a1_probe.c, A2work/a1n_probe.c (built against the A2 export).

(N1) RING ROUTE IGNORES THE OTHER ENGINE.  Program: `read_file(pipe)` then
`open_file(path)` with the table full; an outside thread closes the held
descriptor and then writes the pipe byte.  Source order: the read finishes
(after the close), then the open succeeds.
  Linux, helpers 0/1/4, every run:  open value=-1 error=24 outcome=1 retries UNCHANGED
    A4 route: read ring+0 fallback+1   <- WF_FILE_READ has no io_uring route (bridge.c:717)
    A4 route: open ring+1 fallback+0
  macOS, same program, helpers 0/1/4: open value=5 error=0 outcome=0, retries +1  -> Ok.
  Cause: the ring's gate reads only `adapter->in_flight` and `stat_completions` of the
  RING; an operation in flight on the POSIX helper adapter is invisible, so
  retry_held == in_flight and the refusal is published with no re-attempt.
  F2's stated requirement is "if no other operation is in flight ANYWHERE".

(N2) ADAPTER DRAINS A SIBLING OPEN.  Program: `close(held); open(p); open(p)`
(the canonical K>1 pipeline shape).  Source order: exactly one Ok.
  macOS, WF_IO_HELPERS=4, 200 reps: lost_ok=81..84 (both opens Err while a plain
  open() straight afterwards returns fd 3, and the retry counter moved by exactly 1).
  helpers=2 -> 74/200 ; helpers=1 and 0 -> 0/200 ; opens=1 -> 0/200 (200 all_ok).
  Linux ring route: 0/200 at opens=1,2,3 and helpers 0/1/4.
  Pre-existing: the same probe on 14c89cf3 loses 112/200 at helpers=4, 37/200 at 2.
  So the repair reduces but does not remove it.
NEXT: confirm N2's mechanism; then F3 doc claims, F4 emitter, IR oracle, CI logs.

### Step 5 — N2 mechanism CONFIRMED
Diagnostic copy at A2work/diag/completion (file_adapter.c counts drained OPEN_AT work).
Every lost-Ok repetition coincides with exactly one `drained_opens` increment and exactly
one retry: a refused open drains a SIBLING OPEN out of the queue and runs it with
`wf_file_run_work(..., retire_and_retry = 0)` (file_adapter.c:576), which publishes that
sibling's refusal with no re-attempt at all; the draining open then spends its own single
retry while the close is still executing on a helper.  So the record's
"Each of them then gets its one re-attempt" is false on the adapter route.
NEXT: F3 read (done - claims corrected), F4 emitter, Rust tests, IR oracle, CI logs.

### Step 6 — F3, F4, tests, oracle
- F3: docs/ongoing/0095-loop-pipeline.md at ad04ae2d corrects both claims honestly, with the
  exact 13/200 and 15/20 counts, and states the window claim is NOT demonstrated by Stage A.
- F4: `cargo test --profile gate --lib backend::tests::completion` = 31 passed / 0 failed.
  NEGATIVE CONTROL: with the new ordering rule in validate_pipeline deleted,
  `a_drain_emitted_before_the_hand_out_it_retires_is_refused` FAILS (30 passed / 1 failed).
  emitter.rs restored, md5 back to f95ad2199059ffaa2a233161b397b29f.
- IR oracle A2 vs main b2e2e267: total=1890 emitted=807 no-module=1083 differ=0.
NEXT: CI logs; then the emitter over-join question; then final verdict.

### Step 7 — CI, local gate, A7
- CI at ad04ae2d: io-hosts 33169702910 SUCCESS (completion-linux on real x86-64 io_uring,
  including `native-adapter-probe target=linux-io-uring status=pass`, the
  WF_REQUIRE_LINUX_IO_URING run, completion-sanitize and completion-tsan; plus
  completion-windows, bench-linux, bench-macos-read, bench-linux-read).
  gate 33169702857: gate-macos SUCCESS, gate-linux FAILURE and only on
  `conformance adapter: Pass=503 Fail=6 Skip=1`, the six
  TargetQualification(MissingMapping(Operation(12))) cases the record names.
  `git diff b2e2e267 ad04ae2d -- compiler/src/backend/qualification.rs conformance/` is empty.
  The two earlier pairs (d36ac097, 2646af50) have the same shape.
- `make -C compiler check` on the A2 export, macOS: "== WHITEFOOT COMPILER GATE GREEN ==".
  (Root canonical `make check` needs a .git the export has not got; CI's gate-macos ran it.)
- A7 (my attack): an open held for retire-and-retry across a BLOCKING direct host call
  (`wf__completion_file_open_at_direct` of a FIFO) resolves and joins Ok; no hang.
  Linux helpers 0/1/4 x2 and macOS helpers 0/1/4: all PASS.

## VERDICT: REFUTED (F1-F4 are genuinely fixed; two lost-Ok defects of the same class remain)
