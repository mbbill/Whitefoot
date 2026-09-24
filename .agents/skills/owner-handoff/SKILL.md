---
name: owner-handoff
description: Report to the Whitefoot owner in the owner's language when handing back work, asking for a design ruling or other decision, or presenting findings that await direction. Leads with compact decision cards, then the result, specification revisions and a design-suitability note. Use at the end of every task and whenever the owner must decide something.
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
4. **Design suitability.** A separate short paragraph: concerns and
   improvement opportunities with their disposition (fixed, deferred to
   `docs/todo.md`, or declined) and reasons, or one line naming the assessed
   scope when none were found.
