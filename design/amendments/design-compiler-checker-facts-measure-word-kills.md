Node: compiler/checker-facts

Decision: A row entry naming one measure word kills that word's facts alone, and the relation between a written word and the words it reaches is the identity, because x1's [MSR-2] states that "`len`, `cap` and `head` are three disjoint words of P's descriptor storage" and that an entry naming one "overlaps that word alone, so it kills the `len` facts of the actual and no `cap` or `head` fact", instead of the previous relation in which a written word could reach two others through the complement cell, which no longer exists.

Decision: The two `Array` rows carry a `len` cell and no `cap` cell, so an `Array` place registers no capacity term and neither `P.len <= P.cap` nor `P.head <= P.cap` is emitted over one, because x1's [MSR-1] table answers *absent* for both `Array` rows' `cap`, an array being its own extent with no second capacity quantity, instead of keeping the old cells that answered `cap` with the same number as `len`, which published a standing equality the specification no longer states.

Decision: The capacity identity leaves the automatic affine-premise sequence entirely rather than being weakened, because x1 deletes `P.len + P.room = P.cap` with the `room` measure it relates, so the identity has no third term and [ENT-6]'s sequence now begins empty and carries only the canonical source facts, instead of retaining a degenerate premise over two terms, which the implicit `P.len <= P.cap` ordering already publishes.

Rejected:
- Keeping `room` as a derived reader over `cap - len` outside the measure table: rejected because [FORM-1]'s one-quantity-one-spelling reading gives a writer two ways to say the same thing, and the requirement `deref(w).len < deref(w).cap` that [PRE-1] now writes is the spelling the specification chose.
