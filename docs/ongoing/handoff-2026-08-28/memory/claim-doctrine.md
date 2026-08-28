---
name: claim-doctrine
description: "claim is NOT assert: it is an always-true lemma bridging checker incompleteness, review-verified; a claim that can fail on a reachable input is a misused claim (use typed Err); fully-reviewed programs cannot trap"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ab659700-c818-4271-84f2-e403a962b810
  modified: 2026-08-21T08:22:06.056Z
---

Owner correction, 2026-08-21 (after I wrote a debate example describing a
claim as "必失败 on this input"): **claim 不可能失败,claim 不是 assert。**

The doctrine, in the owner's own logic:
- A claim states a proposition that IS TRUE for every reachable state; it
  exists only because the current checker cannot derive it (checker
  incompleteness), not because the fact is uncertain. With a complete SMT
  solver or a human proof, the claim would not need to exist.
- Therefore: a claim that would fail on some admissible input is NOT a
  claim — that situation is a typed expected failure (`Result`/`Err`,
  ERR-4). Writing it as a claim is a program defect / construct misuse.
- The runtime trap on a claim is the LAST LINE OF DEFENSE against the
  human review being wrong — not control flow. "如果所有claim都被review
  证明过,那么这个程序不可能trap。"
- Review treats each claim as a lemma to verify; the `because` string is
  the proof sketch offered to the reviewer.
- Coherent with the owner's earlier analogy: a checker-provable claim is
  redundant → compile error; a false claim is a defect; claims occupy
  exactly the true-but-currently-unprovable gap, which narrows
  monotonically as the checker strengthens.

**How to apply:** never write or describe example claims that "fail on
input X" as normal usage — label such probes explicitly as deliberately
falsified review-failure simulations that exercise the trap path
mechanically. In parallelism design: the trap/arbitration machinery is
insurance for the review-failure path (which a fully reviewed program
never takes) — its purpose is keeping the DEFECT SIGNAL deterministic
(DIAG-3 byte identity), and its runtime costs live exclusively on the
never-path. Owner has another agent fixing this misconception where it
appears (2026-08-21); don't duplicate that work in repo files. See
[[whitefoot-parallelism-doctrine]].

**2026-08-25, the ruling generalized (I/O design round).** The trap-free
gate tried to return a second time: both adversarial reviewers of the I/O
foundation proposed requiring trap-free closures for overlap windows that
contain world writes, because a defective execution could publish bytes
source order never publishes. The owner rejected it on the standing
doctrine: "一旦trap,那么说明这个程序有问题…不存在'本该trap'这种事情"。
The general form, to reuse whenever this shape reappears: **when a
feature makes a defective execution's observables schedule-dependent, the
repair is to WIDEN the erroneous-execution promise (the schedule may
select those observables too), never to NARROW permission for correct
programs. TCB costs are allowed only on the trap path itself** (quiesce,
latch, serialized record) because correct programs never execute it.
Deterministic reproduction stays free at WF_WORKERS=0.

**2026-08-25, institutionalized:** the ruling is now constitution
THEOREM T3 (docs/constitution.md) — cite T3, not history. Owner's
structural correction: it is a DERIVED conclusion, not a rule — the
load-bearing premise is W3's claim discipline (reviewed always-true
lemmas). If a future construct admits claim-like predicates that are
not reviewed always-true lemmas, T3 does not extend to them until the
derivation is redone.
Prompt hygiene learned from the recurrence: any design/audit agent
mandate that touches claims, traps, or overlap permission MUST carry the
T3 pointer plus docs/ongoing/0078-loop-permission.md and the design
tree's permission-judgment .alt — the reviewers re-proposed the removed
gate because my prompts routed them to the spec and to batch 0074 (the
PRE-redirect design) and never to the ruling. Independent agents
converging is NOT confirmation when they share the same missing context;
ruling history is the lead's to inject.
