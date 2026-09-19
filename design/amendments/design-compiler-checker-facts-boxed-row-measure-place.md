Node: compiler/checker-facts

Decision: A runtime-capacity `Array<T>`, `Slots<T>`, or `Ring<T>` is refused at the written `type` wherever that occurrence is not the one type argument of a `Box`, judged over the written syntax rather than over a substituted instance, because [TYPE-9] states exactly that placement and locates the error at the complete `type`, and a generic parameter bound to `Box<Slots<T>>` writes no runtime-capacity form of its own, instead of checking the formed checked type at each position that stores one, which would have to re-refuse the same shape at a field, an element, a parameter, a local and a type argument separately and would refuse the prelude's own boxed rows.

Decision: A measure member written on a place whose selected type is a `Box` whose content is measured steps into that content first, and only inside a compiler-owned [PRE-1] row, because [OP-14] states that at a boxed argument the row's measure place instantiates as `window.inner` and the clause reads `window.inner.len`, while source code reaches a `Box`'s content by writing the field `inner` itself [TYPE-9], instead of admitting the step everywhere, which would make `b.len` a legal source spelling that [TYPE-9] does not give a `Box`.

Rejected:
- Rewriting the row's clause text per instance so the boxed shape writes `window.inner.len` literally: rejected because the row is one [PRE-1] record whose bytes the specification fixes, and a per-instance text would make the checked clause differ from the declared one.
