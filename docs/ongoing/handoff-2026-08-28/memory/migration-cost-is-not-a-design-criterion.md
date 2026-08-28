---
name: migration-cost-is-not-a-design-criterion
description: Never weigh migration or churn cost when judging a language design — the language must be right and migration is mandatory
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-19T05:59:55.242Z
---

2026-08-18, Whitefoot, owner shouting after the third repetition in one session:
"说了多少次了，不要管迁移成本，不要管迁移成本。我们他妈的在设计编译器，目标
是语言要完善，迁移是必须的。"

I kept smuggling churn into design arguments — "87 files", "44 protected
conformance approvals", "451 test files would need rewriting" — and in the
contract-surface study I was about to make migration cost a column of the
comparison table. Every one of those numbers is irrelevant to whether the
language is right.

**Why:** migration cost is real work, so it feels like a legitimate engineering
consideration, and it produces concrete numbers that look like evidence. But
this project builds a research compiler whose deliverable is the LANGUAGE.
Rewriting the corpus is a mechanical consequence of a decision, not an input to
it. Letting it weigh in means the accumulated corpus vetoes language
improvements — the tail wagging the dog again, the same defect as
[[tests-are-not-users]], one level up.

**How to apply:** when comparing language designs, the axes are language axes
only — does the surface state what is true; is there one spelling per concept;
is each rule stated once; is the construct derivable from kernel principles or
an imported habit; does it compose with what is coming; does it leave inert or
unexercised surface. Migration appears in a plan as WORK TO SCHEDULE, never in
a design comparison as a COST TO WEIGH. If a subagent's proposal ranks options
by churn, strip that axis before presenting it and re-rank on merit — the
ranking often changes. Say "migration is mandatory" and move on.
