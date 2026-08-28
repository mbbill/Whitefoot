---
name: residue-hunt-review-axis
description: "Owner-derived standing review axis — re-derive every construct from the kernel's own principles; hunt design residues (imported habits, proliferation, mechanism duplication)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: f2692882-d1e0-42f4-b979-c7c69aed0c3d
  modified: 2026-08-07T06:01:46.806Z
---

Established 2026-08-07 after the owner's taste caught `index_get` surviving
the full draft→adversarial-review→re-verify→polish cycle: formal review
axes (soundness, determinism, fidelity, craft) check local correctness,
never coherence with the design's own first principles.

**The axis, four questions applied per construct:**
1. Re-derivation: would we invent this today from the current kernel
   (proof / branch / claim; classification at provenance site) if it did
   not exist?
2. Imported habit: does it exist because our machinery needs it, or
   because another language (usually Rust) has it? (`get` exists in Rust
   because a branch doesn't license unchecked `[]` there; WF's discharge
   makes it redundant.)
3. Proliferation: if its justification stands, what else does the same
   justification demand? Unbounded family ⇒ wrong justification.
4. One mechanism per concern: failure-as-value belongs to the branch with
   caller-vocabulary else; no operation carries a private second copy.

**Why:** the owner said "你不能总依赖我的品味" — encode the lens into every
adversarial review prompt as a standing axis alongside soundness.

**How to apply:** include the residue-hunt axis in every spec/design review
agent prompt; periodically re-sweep already-landed surfaces with it (the
v0.21 CLM/ENT/SYS surface is the first backlog). Honest limit: the axis
enforces only principles already articulated — genuinely new criteria
(like the owner's T4 globality objection) still come from humans. See
[[whitefoot-purpose]] and [[explain-with-code-first]].
