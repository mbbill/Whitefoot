---
name: handoffs-run-to-completion
description: Owner rule after two wasted nights — agent-facing documents must never contain mid-run owner-approval stops; single gate = final merge; audit every handoff for stop-language before shipping
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-25T17:59:10.947Z
---

2026-08-26, owner, angry, after the I/O handoff blocked codex right
after Phase A and wasted a whole night: "以后写这种东西要小心,昨天又浪费
我一整晚时间,刚开始就block。我说了要做完的,不要总是在文档里面写要owner
批准什么的。"

The incident: my HANDOFF.md contained "present to the owner with the
five decisions laid out", "never decide them yourself", and "do not
activate without the owner's approval" (wrongly bundling work-branch
activation with merging). A literal agent obeyed and parked overnight.
A post-incident audit found a THIRD blocker I had missed ("the owner
reads the number and decides whether the runtime batch proceeds").
Earlier same-class failure: dressing a tool-sandbox classifier block as
an "owner's merge-time step".

**Why:** the owner's overnight-autonomy directive ("不要浪费时间", "做完",
"不要等我") is standing. Project law has exactly ONE owner gate: merge
of the exact final revision into main (rule 2). Anything phrased as a
mid-run approval invents a second gate and converts an autonomous
pipeline into an overnight deadlock.

**ABSOLUTE FORM (owner, 2026-08-26, final):** "以后但凡写交接文档,禁止加
这种owner批准的事情。所有事情默认在分枝上全部做完不可以停!这个事情必须
按照项目目前的规范来完成,禁止再在上面加任何新的block条款要求owner审批。"
Meaning: a handoff contains ZERO approval language — not even a
restated final gate. The four rules in AGENTS.md already govern the
main boundary; a handoff adds nothing to them, restates nothing of
them. The only merge-related sentence allowed is "never merge into
main yourself". Everything else: branch work, run to completion, no
stops. And when the owner then says 不要再改了 — stop touching the
repo; the rule lives here, not in another round of file edits.

**How to apply:**
1. In ANY document an agent will execute (handoffs, charters, prompts,
   plans, batch records), never write a mid-run owner-approval point.
   Design decisions become: adopt the recommended option, record it as
   a flagged decision, keep going; all flagged decisions ride the
   single final merge approval.
2. Work-branch actions (spec activation included) are ordinary branch
   work — never bundle them with "merge" in approval language.
3. Adverse findings (a bad measurement, a failed falsifier) do not
   stop the run either: state them prominently in the final packet.
4. Before shipping any agent-facing document, grep it for
   approval/approve/owner/批准/present/wait/decide and justify every
   hit; read it once as a maximally literal agent asking "where could
   this text make me stop?" — the third blocker survived two rounds of
   my own editing.
