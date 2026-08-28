---
name: gate-time-budget
description: Owner rule 2026-08-27 — the canonical gate (make check) must finish within 5 minutes on every host, local and CI; slow sampling tests are restructured, never weakened; agents never wait in foreground loops
metadata:
  type: feedback
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-27T21:40:00.000Z
---

2026-08-27, owner: "不管哪个平台测试都要压到一定时间以内,比如5分钟,不然
老是出这种问题非常恶心,等个半天". Context: the first GitHub gate runs
took 35-50 min (a temporary diagnostic step doubled them), then 18 min
on the 4-core Linux runner, of which the library suite was 980 s
(~90 s locally): five process-spawning sampling tests (parallel repeat
6.9 min, trap_latch race 3.5 min, ...) owned it, and batch 0090 had just
enlarged their sample sizes to be robust on runners.

Also measured the same day: 41% of all agent tool time (9 of 21.8 h)
was foreground wait loops (`until … sleep`, `gh run watch`) capped at
the 10-min Bash limit; those waits tripped the Workflow stall detector
(180 s) which spawned duplicate agents into the same worktree.

**Why:** iteration speed is the whole game for an agent-driven project;
a 20-40 min gate turns every fix into an hour and every wait into a
collision.

**How to apply:** (1) budget: every CI job `timeout-minutes` ~8, target
< 5 min; local `make check` < 3 min; print the slowest tests per job.
(2) cut cost by structure only: link once run N times, sample sizes
from measured rates with early exit on existential claims, shared
fixtures, sampling tests in their own binary/job, parallel CI jobs
whose union == `make check`; never narrow an assertion or unwire a
test. (3) agents: never wait in the foreground > 2 min; use
run_in_background for CI/gate waits. Batch 0093 implements (1)-(2).
