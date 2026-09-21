Node: language/ownership/reference-validity

Decision: A checked reference to an existing payload retains that selected place after the selecting match ends, while any newly written payload selection still requires a current variant fact, because ending a lexical proof scope does not change the selected storage and iterative descent must carry its selected address across the backedge, instead of invalidating an unchanged address solely when its selecting arm ends.

Decision: A reference passed to a call is protected only from content writes at or below its own captured target, and is still invalidated by another actual's destructive ancestor write, because argument membership cannot keep removed storage alive, instead of exempting every actual reference from post-call invalidation.

Rejected:
- Keep the second decision's refusal of a payload reference surviving its selecting test: rejected because the ownership witness preserves only the already selected place and publishes no variant fact for a new selection; enum replacement and owner destruction still invalidate it.
- Keep the fifth decision's blanket argument exemption: rejected because a permitted call can replace an ancestor through one actual while a different unused actual names its destroyed descendant.
