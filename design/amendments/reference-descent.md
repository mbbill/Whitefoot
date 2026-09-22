Node: language/ownership/reference-rebinding

Decision: Loop-carried references use one finite possible-location description per ultimate root, retaining the finite entering static shapes while contributions agree and otherwise a containing prefix with an unknown descendant tail, because owned-list and tree cursors need unbounded runtime depth while monotone prefix shortening and finite roots give deterministic terminating checking without losing established entry-shape separations, instead of refusing every self-extending path or promising convergence after one further body visit.

Rejected:
- Retain the static-shape prohibition and its refusal of recursive descent: rejected because recursion and indexed pools do not supply the requested iterative cursor edits; this replaces the node's second decision and its final rejected alternative, on the grounds in research/investigations/wildcard-path/DESIGN.md.
- Equate targets with equal descendant covers: rejected because two cursors can select different nodes with different values under the same root, so a cover supports possible interference but never target equality.
- Retain arm-exit loss of a payload refinement as a reason in the rejection of lexical region blocks: rejected because the selected payload's existence survives that lexical exit; the refusal of region blocks remains grounded in the unchanged explicit mutation, move and scope-end invalidations.
