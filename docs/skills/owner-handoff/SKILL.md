---
name: owner-handoff
description: Hand work back to the Whitefoot owner in the owner's language - decision cards first, then the result, specification revisions and what the work found along the way. Use when stopping for the owner - a task is done, an amendment needs a ruling, or a finding awaits direction. Not for progress notes while work continues.
---

# Owner handoff

The owner rules on the design and merges. A handoff exists so that the owner
sees every decision without re-reading the work: lead with what needs the
owner and keep supporting detail one step away, in the PR, the amendment or
the investigation. Write in the owner's language; repository artifacts stay
English.

1. **Decision cards.** One row for each amendment awaiting a ruling, each
   finding awaiting direction and each other choice the owner must make, or
   "none":

   | # | Node or topic | Now → proposed | Recommendation and decisive reason | Reversible? | Owner judges |
   |---|---|---|---|---|---|

   Link the amendment or evidence that holds the problem, alternatives,
   tradeoffs and uncertainty. Bring one of them into the card only when it
   decides the recommendation.
2. **Result.** A few lines: what changed, the validation actually run (full
   gate or focused, and the tested revision), what remains unverified, and the
   PR link.
3. **Specification revisions.** Whenever `spec/kernel-spec.md` changed: which
   rules changed, their before and after behavior, and why they were selected.
   A version number or PR link does not replace this.
4. **Found along the way.** A short paragraph summarizing the PR's section of
   that name: what was fixed, what was recorded in `docs/todo.md` and what was
   declined, with reasons, or one line naming the areas worked in when nothing
   was found. The noticing happens while working, under AGENTS.md's "Fix or
   record what you notice"; this step only reports it.
