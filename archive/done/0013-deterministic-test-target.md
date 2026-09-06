# 0013 — Deterministic test target

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 6; lead-authorized overlap with 0012 (0012 landed first; this task
  paused honestly at that boundary, then rebased and completed)

## Outcome

The deterministic test implementation is a second qualified target column,
not a simulator: `SystemTarget` gained a `HostFacilities` column naming the
five host-reaching facilities; the deterministic column names `wf_test_*`
siblings answered by one generated C translation unit linked into the test
artifact, so the SAME emitted lowering is under test and the host trace
exposes attempt counts per descriptor. `HostFacilities::DeterministicTest`
is `#[cfg(test)]` — unreachability from normal compilation is
build-enforced. All four fault-injection cases 0016 consumes are green,
each with a control: close-EINTR (one attempt, never retried), mid-stream
ReadFailed (progress then EIO, drain stops as ReadFailed, no retry), forced
short write (Ok(1), one attempt, no range completion), close/writeback-only
sink failure (exactly one close fires — the DirectoryRead's; neither Output
closes, so the failure cannot reach the program, per SYS-12). Cases 2-3
re-run 0012's own contract programs unchanged on both columns. The 0012
handoff hazards are all closed, the DIAG-3 trap-writer hazard as a test
(native @write on both columns; a scripted short write cannot truncate a
trap record). The dossier §6.10 proportionality clause never triggered.

## Evidence and validation

- Landed commits: `140a31b` … `628bc69` (including the honest interim
  WAITING state at `d7b7a8f`, kept as history). Both gates green by unpiped
  exit codes; lib tests 427.

## Follow-ups

- 0016 consumes `HostScript`/`HostOutcome`/`run_on_deterministic_host`/
  `DeterministicRun` per this record's Dependencies section; the newly
  scriptable `Accept(0)` WriteZero case is lead-flagged as a 0016 addition
  (upgrading 0012's emitted-shape-only WriteZero evidence to behavioral).
