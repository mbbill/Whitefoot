Decision: A reference variable may be rebound and a join retains all incoming targets, with every check required for every target, because a local name can denote alternatives without becoming storage or needing retyping, instead of one immutable resolved root per holder.

Decision: Loop-carried references use one finite possible-location description per ultimate root, retaining the finite entering static shapes while contributions agree and otherwise a containing prefix with an unknown descendant tail, because owned-list and tree cursors need unbounded runtime depth while monotone prefix shortening and finite roots give deterministic terminating checking without losing established entry-shape separations, instead of refusing every self-extending path or promising convergence after one further body visit.

Decision: Passing a reference uses the ordinary argument form and the callee's substituted effect row, because the call-site comparison already determines which accesses may coexist, instead of child reborrow forms with separate permission, region, suspension and endpoint rules.

Rejected:
- A permission marker on a reference (`&uniq`, `&mut`, or any other) selecting whether the callee may write through it: rejected because the effect row already states every write, and a second spelling of one fact can disagree with the first.
- A block-scoped reference form or a lexical region confining a reference's extent: rejected because validity is a fact established at formation and invalidated by named events, a write, move, replacement, or release of a proper prefix, or the end of the root local's scope, and a lexical container adds nothing that fact does not already carry.
- Retain the static-shape prohibition and its refusal of recursive descent: rejected because recursion and indexed pools do not supply the requested iterative cursor edits, on the grounds in research/investigations/wildcard-path/DESIGN.md.
- Equate targets with equal descendant covers: rejected because two cursors can select different nodes with different values under the same root, so a cover supports possible interference but never target equality.
