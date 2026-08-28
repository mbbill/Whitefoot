# Stage A round-3 adversarial re-verification — checkpoint notes

Target: 64ef548e on batch/0095-loop-pipeline (repair of ad04ae2d, which was REFUTED on N1/N2).
Prior: $SCRATCH/wf-0095-verify/A/NOTES.md (round 1, 14c89cf3)
       $SCRATCH/wf-0095-verify/A2work/NOTES.md (round 2, ad04ae2d)

## Layout
- export A3 = $SCRATCH/wf-0095-verify/A3 (git archive 64ef548e)
- negative control export A2 = .../A2 (ad04ae2d), A = .../A (14c89cf3), B/main = b2e2e267
- CARGO_TARGET_DIR for A3: $SCRATCH/wf-0095-verify/tgt-A3

## Step log
### Step 0 — export done

### Step 1 — builds + baseline suites started
- A3 export at 64ef548e; compiler built into tgt-A3 (gate, EXIT=0).
- macOS harness/tsan/sanitize built from A3 sources into A3/mac (build_harness.sh).
- Linux container wf0095v3 (wf-io-bench:linux, aarch64 6.8.0, seccomp=unconfined),
  sources /root/src3, binaries /root/bin3.
- macOS harness x200 helpers 4/1/0 -> fail=0 stall=0 each.
- macOS TSan x20 at helpers 4/2/1/0 -> fail=0 stall=0 each (90/180 s watchdog per run).
- macOS harness x200 helpers=2 also queued.
### Step 2 — N2 (a1n_probe, close(held); open; open) — FIXED, with negative control
macOS, 200 reps each: A3 lost_ok=0 at helpers 4/2/1/0.
NEGATIVE CONTROL A2 (ad04ae2d): lost_ok=91/200 at helpers=4, 115/200 at helpers=2, 0 at 1/0.
### Step 3 — Linux container wf0095v3
- harness x200 at helpers 4/1/0 and x200 at 2, harness-tsan x20 at 4/2/1/0,
  harness-sanitize: no .fails and no .stalls files -> 0 failures, 0 stalls.
- verify_probe x15 at helpers 4/2/1/0, A3 and A2: 120/120 PASS both.
### Step 4 — N1 (attack_probe A4, ring open waits for helper-route read) — FIXED
Linux, x15 at helpers 4/1/0 (45 runs): A3 `attack_probe: PASS (failures=0)` every run;
  `A4 route: read ring+0 fallback+1` / `open ring+1 fallback+0` (genuinely cross-engine),
  `A4: open value=8 error=0 outcome=0 wrote=1 retries+1`.
NEGATIVE CONTROL A2: 45/45 `attack_probe: FAIL (failures=3)`, A4 open value=-1 error=24, retries unchanged.

### Step 5 — RESUMED (new copy of the agent, 2026-08-28 ~13:55)
Recovered state: HEAD of $SCRATCH/wf-0095-pipeline is 64ef548e (clean).
Done already: builds, macOS suites (mac-suite.log: 200x at helpers 4/2/1/0 fail=0 stall=0;
tsan 20x at 4/2/1/0 fail=0 stall=0), N2 (n2.log: A3 lost_ok=0 at all helper counts,
A2 control 91/200 at h=4 and 115/200 at h=2), Linux container wf0095v3 (still Up),
N1 (attack_probe A4 45/45 PASS on A3, 45/45 FAIL on A2), IR oracle (oracle3.log
total=1890 emitted=807 differ=0).
Probes X1..X7 written in probes/x_probe.c but NOT yet run (run/x is empty).
probes/x2.c is a partial rewrite whose only real change is a stricter X6.
TODO: merge x2's X6 into x_probe, add X8 (SQ-full retry-pending + completion-only
sleep), run x_probe on both hosts with A3 and A2 controls, read the repair source for
the two LATENT items, cargo tests, CI logs, verdict.

### Step 6 — RESUMED again (new copy, ~14:10)
Export digests re-verified against `git show 64ef548e:...` for runtime.c, contract.h,
bridge.c, file_adapter.c, linux_io_uring.c, emitter/completion.rs — all match.
x_probe DID run on macOS (run/x/mac-x.log, 15 reps x 4 helper counts x {a3,a2}).
Result: every run "FAIL (failures=5)" on BOTH a3 and a2, but the failures split:
  X1 PASS both.  X3 PASS both (worst wait <= 1 ms).  X4/X5 no assertion fired.
  X2 (close + cross-engine read + 4 opens, 50 reps/run): a3 sum lost_ok
      h=4: 15/800, h=2: 0, h=1: 0, h=0: 0
      a2 control h=4: 419/800, h=2: 411/800, h=1/0: 0
    -> the N2 CLASS SURVIVES at K=4 on a3, ~1.9%.  Under investigation with x2d.
  X6 and X7 assertions look like PROBE BUGS (they fire identically on a2 and on
    a3 and their expectations are self-inconsistent) — triage before claiming.

### Step 7 — RESUMED (new copy, ~14:15) — X2 TRIAGED: REAL, and the mechanism is named
Built probes/x_sel.c (x_probe with an XSEL env stage selector) and reran x2d at 1000 reps.
macOS, a3 (64ef548e):
  x_sel XSEL=2 (X2 alone)   h=4: 21/1000   h=2: 0/1000
  x_sel XSEL=12             h=4: 17/1000   h=2: 0/1000
  x_sel XSEL=1234567        h=4: 22/1000   h=2: 0/1000
  a2 control XSEL=2         h=4: 562/1000  h=2: 801/1000
  x2d 1000 reps h=4 use_read=1: lost=10/1000; use_read=0: 0/1000; h=2 both: 0/1000
DISCRIMINATOR: the loss needs the cross-engine READ.  Every lost rep prints
  `errs=24,24,24,24` and `retries=N->N+4`: all four opens made their ONE
  re-attempt and all four failed, then a plain open(2) immediately after
  returned a descriptor (direct=5).
MECHANISM: step 2 of the rule ("if the generation moved since the snapshot,
  re-attempt once") treats ANY retirement as "a descriptor came back".  The
  read retiring returns no descriptor, but it moves the generation, so each
  refused open burns its single re-attempt before the close has finished and
  then publishes.  With use_read=0 the only retirement available is the
  close's, so every open waits (step 3) and one wins -> 0/1000.
  The earlier 0/400 x2d result was a fluke (200 reps at 1.0% -> P(0)=13%).

### Step 8 — X2 on Linux: a REGRESSION against ad04ae2d
Container wf0095v3; src3 = 64ef548e, src2 = ad04ae2d (md5 verified against
`git show <commit>:compiler/src/backend/completion/<file>` for runtime.c,
contract.h, file_adapter.c, linux_io_uring.c, bridge.c).
x2d, 1000 reps per cell, `pipe; hold last fd; read(pipe); close(held); open x4`:
            use_read=1                use_read=0
  a3 h=4    7/1000                    0/1000
  a3 h=1    7/1000                    0/1000
  a3 h=0    121/1000                  0/1000
  a2 h=4/1/0  0/1000                  0/1000
x_sel XSEL=2 (same shape, 50 reps/run x 20): a3 59/1000 (h4), 56 (h2), 63 (h1),
  183 (h0); a2 0/1000 at every helper count.
Sample: `rep=392 LOST direct=8 retries=1249->1253 errs=24,24,24,24`
So on the ring route the predecessor NEVER lost this Ok and 64ef548e loses it
up to 12.1% of runs.  On macOS it is an improvement (562-801/1000 -> 10/1000)
but not a fix.

### Step 9 — RESUMED (new copy, ~14:30) — independent re-confirmation of X2 started
HEAD of the pipeline worktree is still 64ef548e, clean. Built probebin/a2/x2d (the
ad04ae2d control was missing on macOS). Launched my OWN x2d runs, 1000 reps each,
a3 and a2 x helpers {4,0} x use_read {1,0}:
  macOS -> A3/run/r3v/mac-x2d.log ; Linux (wf0095v3) -> /root/run3/lin-x2d-r3v.log
Read the rule implementation myself: contract.h:380-525 and runtime.c:1043-1346.
LATENT #1 (generation re-read at the give-up exit) IS closed: wf_retirement_state_now
re-reads wf_retirement_generation immediately before returning UNREACHABLE
(runtime.c:1312-1320). LATENT #1 therefore does NOT stand.
NEXT: record claims, X6/X7 triage, seed_pipeline_drain (LATENT #2), ring kick, CI logs.
