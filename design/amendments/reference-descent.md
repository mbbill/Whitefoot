Node: language/ownership/reference-rebinding

Decision: Loop-carried references use one finite possible-location description per ultimate root, retaining a fixed static shape when it agrees and otherwise a containing prefix with an unknown descendant tail, because owned-list and tree cursors need unbounded runtime depth while monotone prefix shortening and finite roots give deterministic terminating checking, instead of refusing every self-extending path or promising convergence after one further body visit.

Rejected:
- Retain the static-shape prohibition and its refusal of recursive descent: rejected because recursion and indexed pools do not supply the requested iterative cursor edits; this replaces the node's second decision and its final rejected alternative, on the grounds in research/investigations/wildcard-path/DESIGN.md.
- Equate targets with equal descendant covers: rejected because two cursors can select different nodes with different values under the same root, so a cover supports possible interference but never target equality.
