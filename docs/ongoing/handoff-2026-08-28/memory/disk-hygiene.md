---
name: disk-hygiene
description: Disk fills up from cargo target/ dirs in the ~100 worktrees and git-archive exports under do_not_scan and .claude/worktrees; delete only target dirs of finished worktrees, never those of running agents
metadata:
  type: project
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-27T14:30:00.000Z
---

2026-08-27: the owner reported the disk nearly full (1.5 GiB free of
460). Cause: every worktree and every git-archive export under
$SCRATCH and $REPO/.claude/worktrees
carries its own cargo `target/` (2-12 GB each; 122 of them). Deleting
only the `target`/`linux-target` directories of finished worktrees freed
~65 GB with zero risk (build artifacts, rebuildable); worktrees of
running agents were excluded by name.

**How to apply:** when spawning many agents in worktrees, expect ~2-6 GB
per worktree of build output; before a long multi-agent night check
`df -h /`; clean with
`find <roots> -type d \( -name target -o -name linux-target \) -prune -print | grep -Ev '<in-use>' | xargs rm -rf`.
Never delete a worktree directory itself without `git status` there
(uncommitted work); never touch a running agent's worktree. Docker
images (`docker system df`) are a secondary ~3 GB.
