---
name: subagent-cost-control
description: Owner is cost-sensitive about subagent token burn; constrain spawning and use cheaper models for bulk work
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-25T07:00:26.759Z
---

During a large archive-mining task (2026-07-28), broad "be complete" Explore/general-purpose prompts caused both mining agents to recursively spawn ~19 descendants (~19 MB of transcripts, part at Fable pricing). The owner objected to the cost mid-run.

**Why:** subagents can spawn their own subagents; an open-ended "read everything, be complete" prompt over a large corpus invites fan-out. Polite shutdown_requests are not processed while an agent is mid-task (one agent ran its entire task to completion after receiving one), and anonymous grandchildren are unreachable — TaskStop works only for named top-level agents; SendMessage cannot route to raw IDs of running anonymous children.


2026-08-17, owner corrected me AGAIN (annoyed): "别每次全开fable5,说了简单的
事情用opus怎么记不住呢". The failure mode: omitting the model parameter
makes every agent INHERIT the session model (fable) silently. The rule is
mechanical: **NEVER omit `model` on any Agent or workflow agent() call.**
Reading/inventory/mechanical sweeps → model: 'opus' (or sonnet for bulk
mining). Fable ONLY for judgment-heavy work (spec surgery, adversarial
synthesis, design), and only when the complexity genuinely needs it.

**How to apply:** (1) For bulk reading/mining tasks, use `model: "sonnet"` (owner asked explicitly: "use opus or sonnet to save me some tokens"). (2) Add "do not spawn any subagents" to mining/survey prompts unless fan-out is deliberately wanted. (3) Give every spawned agent a name so TaskStop can reach it. (4) To stop a runaway tree, TaskStop the named root immediately — don't rely on shutdown_requests. (5) Watch `<session-dir>/subagents/*.jsonl` sizes/mtimes to detect live burn.

**Model-tier split (owner, 2026-08-05, Whitefoot session):** "最困难的交给Fable5，剩下的给Opus5" — reserve `model: "fable"` (or inherit) for only the genuinely hardest work (novel spec-semantics drafting, resolver/effect-checker surgery, adversarial verification of the trickiest material); default every other executor, reviewer, and drafter to `model: "opus"`. Sonnet stays for trivially mechanical high-volume conversion only.

**Update 2026-08-07:** owner's Fable quota is low. Policy: default every
agent to Opus 5 (executors, drafters, reviewers — the review axes are
codified well enough for checklist-driven Opus). Fable ONLY for genuine
impasses or soundness-final review, announced to the owner first. Running
Fable agents finish their current assignment; no new Fable spawns without
cause.
Refined same day: no pre-announcement needed — lead judges; hard things go
to Fable, everything else Opus 5. "困难的事情给它就好."
Ops lesson 2026-08-07: NEVER pipe gate commands through tail/grep in a &&
chain — the pipe masks the exit code (a red make committed as green once).
Check exit codes directly; run gates as their own command.
Workflow ruling 2026-08-07: owner wants LINEAR main history — integrate
task branches by rebase onto main + fast-forward only; no merge commits.
Existing merge bubbles: cleanup deferred; flag the SHA-citation cost
(APPROVALS/task records cite commit SHAs; history rewrite orphans them)
before any rewrite.
2026-08-07 (later): Fable quota EXHAUSTED. Do not spawn Fable agents at all
until the owner says otherwise — Opus 5 for every subagent, including hard
semantic work. Lead session itself also runs Opus 5 now.
Owner ruling 2026-08-07 on task sizing: prefer LONG tasks to one
context-loaded agent over farming small pieces to fresh agents — a new
agent must reload the whole picture and still won't know the goal.
Corollary: when an agent already holds the context, give it the tail;
the lead does the truly small steps itself (the lead has full context);
never spawn a fresh agent for a one-line finish.

**Standing rule 2026-08-20 (owner, supersedes the 08-07 "no Fable
subagents" freeze):** Fable subagents are allowed again, but ONLY for the
genuinely hardest divergent/design thinking ("最需要思考和发散的事情");
everything else — search, retrieval, paper/article reading, surveys,
mechanical work — goes to Opus ("Opus有Fable几十倍的用量"). ALWAYS pass an
explicit `model:` on every Agent/agent() call, never inherit. The lead
session itself is Fable and must also conserve its own tokens: delegate
anything an agent can do, keep lead turns for synthesis and judgment.

**Standing rule 2026-08-25 (owner): codex CLI as the subagent pool when
Claude usage runs low.** Owner: "Claude code用量剩余不多,你还像以前一样从
命令行运行codex,用 sol ultra,那个还剩很多随便用。用法就和你自己起Opus
subagent一样,可以并行跑。" Mechanics verified: `~/.codex/config.toml`
already defaults to model gpt-5.6-sol + model_reasoning_effort ultra, so
plain `codex exec` is sol-ultra. Pattern: write the mandate to a prompt
file, run `codex exec -C <repo> -s read-only - < prompt.md > out.md 2>
err.log` as a background Bash task (run_in_background; harness notifies on
exit); read-only sandbox for auditors, default full-access only when the
agent must write probes. Parallel launches are fine. Treat reports as
claims: verify load-bearing anchors, same as Opus subagents.

**Standing rule 2026-08-18 (owner, batch 0071 night):** Fable at 12%,
Opus effectively unlimited. The two in-flight Fable executors (E1 buffer
lowering, E3 check dissolution) finish their briefs; after that, NO new
Fable subagents at all — remaining Fable is reserved for the LEAD session
itself, because the owner explicitly does not want Opus as the main
agent. Every future subagent this night and beyond: opus (sonnet for
mechanical bulk). This also means the lead should spend its own turns
frugally — delegate bulk reading to opus, keep integration tight.

**2026-08-27 addendum (connection drops):** three agents lost 1-2 hours
of uncommitted work each to "Connection lost mid-response" / login
outages. Every implementation prompt must now say: commit a WIP state on
the branch right after reading the worktree, and commit after each
coherent step. Resume prompts: "read git status/diff completely,
reconstruct, discard nothing, first action = WIP commit".

**Workflow retry hazard (2026-08-27):** a Workflow whose agent "stalls"
or hits an API error RETRIES it with a fresh agent in the SAME worktree
(journal shows two `started` lines per key). Launching a second resume
workflow on that worktree therefore produces two live agents committing
into one branch (happened on 0092: three actors). Before relaunching a
resume, TaskStop the old workflow by its task id; never SendMessage a
workflow agent by raw agentId (it resurrects a dead agent).

**Confirmed 2026-08-27 late:** ANY SendMessage to a workflow agent's raw
agentId (even a one-line reminder) "resumes" it = spawns a second copy
sharing the same worktree and branch; the two then cancel each other's
CI runs. Never message workflow agents. If a course correction is
needed: TaskStop the workflow, then spawn a NAMED Agent (Agent tool,
not Workflow) for the remainder; named agents accept messages safely.
