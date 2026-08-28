---
name: workflow-stall-kill
description: "The Workflow runtime interrupts a subagent whose single tool call runs longer than ~170 s and restarts it from scratch (\"[Request interrupted by user]\" in the transcript); agents with an inherently long foreground step loop forever with zero progress"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-28T16:35:33.971Z
---

Observed 2026-08-28 ~07:50 on batch 0096 round 4 and the 0100 gate
verifier: three copies of the same agent each died at the same step
(identical transcript length), each ~14 minutes apart. The transcript's
last record is `"[Request interrupted by user]"` at exactly 3 min 16 s
after the previous tool result — nobody interrupted; it is the stall
detector. The original is killed, not continued, so a duplicate that
reads the worktree finds no progress and repeats the step.

**Corrected cause (08:10):** the kills line up with the session's
recurring CronCreate wake-ups (a 15-minute "keep-alive" the owner asked
for): interruptions at 14:50:46Z / 15:06:35Z, cron prompts recorded at
14:51:47Z / 15:06:47Z, and agents in two unrelated workflows died in the
same second each time. A cron prompt being enqueued interrupts every
subagent that is mid-generation exactly like a user pressing Esc; agents
inside a long tool call survive. So a keep-alive cron kills the workers
it was meant to protect. Deleted the cron (CronDelete 5f6bae89); task
notifications wake the session anyway. Never run a recurring cron while
workflows are in flight. The long-foreground-command rule below is
still worth keeping, but it was not the cause here.

**Second correction (09:40):** the workflow itself then failed with
"agent stalled on all 6 attempts (no progress for 180000ms each)", so
the stall detector is real too: no tool event for 180 s = kill, six
kills = the workflow gives up. Agents trip it by passing a large
`timeout` to Bash for `make check` and corpus loops. The mechanical
guard that agents cannot mis-estimate: NEVER pass `timeout` to Bash —
the default 120 s limit auto-backgrounds a long command and returns a
tool event before 180 s; poll the output file afterwards. Both causes
(cron interrupts mid-generation; long foreground calls) were live at
once tonight.

**How to apply:**
0. Every workflow COMMON prompt starts with: never pass a Bash
   `timeout`; read NOTES.md and `git log -5` before acting (earlier
   copies may have finished most of the work); keep each Write/Edit
   under ~2 KB, build big tables with shell scripts.
1. Every workflow agent prompt starts with the HARD RUNTIME RULE: any
   command that can exceed 60 s goes through `run_in_background` with a
   log file, polled by separate short calls (`sleep 45` at most).
2. When a workflow's agent files show repeated short transcripts ending
   in "[Request interrupted by user]", TaskStop the workflow, add the
   rule to its script, and resume with `resumeFromRunId` (completed
   agents replay from cache; the killed one reruns live).
3. An agent whose single transcript keeps growing for hours (e.g. the
   0095 round-3 fix agent) is complying; leave it alone.
Related: [[subagent-cost-control]], [[gate-time-budget]].
