Node: compiler/checker-facts

Decision: A `deref` of a reference variable whose path set has more than one member, which [REF-1] produces at a control-flow join, stops as an unimplemented compiler capability rather than being checked against every member, because a written place selects one storage and the checked program hands lowering one addressed path, so the checker would have to publish a place it cannot name, while every *judgment* over such a reference -- its validity, its invalidation, its call-site comparison -- already runs over the whole set and is unaffected, instead of resolving the union to its first member, which would check one path and lower it while the program may take another, or refusing the join as a source error, which [REF-1] explicitly admits.

Rejected:
- Picking the first member of the set: rejected because the members differ exactly when the two incoming edges name different storage, so the one the checker picked is the wrong one on the other edge, and the error is silent.
- Refusing the program at the join: rejected because [REF-1] states the union rule as an admission and gives it its own check discipline, so a refusal would reject a program the language accepts on a representation ground.
