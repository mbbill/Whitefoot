---
name: explain-with-code-first
description: "Owner wants design explanations built around concrete code walkthroughs, not abstract terminology — confirmed twice in the trap-semantics discussions"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-21T21:09:55.091Z
---

Stated 2026-08-06 (twice, during the obligation-discharge design discussion):
abstract, terminology-dense explanations in Chinese "非常奇怪/看不懂"; the
format that works is: write a small concrete program (invented syntax fine),
then walk through it site by site, introducing each term only where the code
makes it visible. After the ensures walkthrough: "你这个解释比上面的好太多
了…以后可以就像这样来解释吗".

**Why:** the owner thinks in programs, not in construct taxonomies; Chinese
renderings of invented English terms compound the opacity.

**How to apply:** for any nontrivial design point, lead with a ~20-line code
example and attach the concepts to specific lines; keep tables/术语 as
summaries after the walkthrough, never as the explanation itself. See
[[whitefoot-purpose]].

2026-08-20 extension (owner, parallelism research): reports must be
SELF-CONTAINED — every cited system/term (e.g. TSan, Cilk, SP-bags, Iris,
work stealing) gets a short in-place explanation of what it is and why it
matters, at the point where it is first used. Do not assume familiarity
with concurrency/verification prior art; a name-drop without a mechanism
sketch is useless to the owner. Same code-first rule applies: show the
mechanism with a tiny example where possible.

2026-08-21 THIRD correction — THE STANDING DEFAULT, overriding calibration
judgment ("你现在就当我是个90岁老奶奶吧"): for EVERY owner-facing
explanation document, assume the reader knows NOTHING about what I did, why,
how, or ANY of my terminology — not even project-adjacent CS terms. The
owner said my pages read "就好像是给你自己看的" and that repeated
instructions to teach via examples "似乎没什么用". Concretely: (1) ONE story
thread — build the whole page around one tiny program, introducing each
concept only at the moment the story needs it, in dependency order, zero
forward references; (2) every term (递归、栈、线程、claim、缓冲区、环境变量、
字节、纳秒…) taught from zero at first use with a walked example — audit the
final draft sentence by sentence for any term used before taught; (3) every
machine output line quoted must be translated word-by-word into plain
Chinese in the next sentence; (4) every number derived in front of the
reader, never asserted; (5) no IR/assembly/internal machinery unless the
point cannot be made without it — and then taught from zero too; (6) the
register is zero-context, NOT low-intelligence: adult, precise, no cutesy
tone. Length is unlimited; clarity is the only metric. This is a tolerance
test: a bright person with no programming background must be able to follow
every sentence.

2026-08-20 SECOND correction, same day, harsher ("整个文章完全不知所云。
一塌糊涂"): concept cards at the top do NOT rescue body prose written in
agent-digest register. The failing paragraph was grammatical Chinese where
every clause hung on an internal codename invented during the research
itself (family-D3, D3-1/2/3, shared_decl, BOUND-1, SPSC, Tier A/B/C, T13,
F1, "两边界定理"). Rule: write for a reader who has read NONE of the
source documents. Internal finding IDs, direction codes, tier names,
research-invented theorem names are BANNED from owner-facing prose —
replace each with its content ("停驻的 lane 不归还工作线程,线程池耗尽,
程序在该中止的地方挂起"), or a walkthrough example. Spec rule IDs may
appear only WITH an in-sentence plain-language gloss. Compression is the
failure mode: fewer topics fully explained beats an index of findings.
Rewrite path: /explain-code register — every design idea enters through a
concrete program walked line by line, failures shown as concrete scenarios
(specific inputs, specific worker counts), verdicts as stories not chips.

2026-08-18 lesson: the owner caught an OWN-10-ILLEGAL teaching example in my
reborrow explanation (root declared inside the region it was borrowed at —
storage must outlive-or-equal the borrow's region tag; only arena content
gets the equals case, its lifetime is type-bound to the region). Teaching
examples in Whitefoot MUST be validated through the real compiler before
publishing, exactly as the explain-code skill demands — a plausible-looking
illegal example teaches a false rule.
