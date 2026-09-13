Decision: A direct slice carries a finite static set of ultimate storage origins, formation creates a singleton, every operation preserves the complete set, and a call computes its result's origin set from signatures alone, because a body-derived exact summary would make callable contracts depend on implementation bodies and need fixed points for recursive call groups, which conflicts with the signature-complete caller boundary, instead of body-derived origin summaries.

Decision: The boundary is direct slices only, with region-bearing generic arguments and stored slice content rejected rather than approximated, because per-leaf origin metadata inside generic or stored values would import unresolved retained-state, cleanup, and stored-borrow obligations, and rejecting those hiding positions is sounder than approximating them, instead of tracking origins through generics and stored values.

Rejected:
- An explicit return-origin annotation: rejected because it adds writer syntax plus body validation for a precision a writer can already express with separate formal regions.
- A body-derived exact origin summary: rejected because callable contracts would depend on implementation bodies and recursive call groups would need fixed-point handling.
- Per-leaf origin metadata inside generic or stored values: rejected because it imports unresolved retained-state, cleanup, and stored-borrow obligations.
- A fresh call or arena token authorizing returned arena storage: rejected because the backing allocation and cleanup obligation are not proved to survive the callee, so callee-created arena suppliers stay deferred rather than represented by an invented origin.
