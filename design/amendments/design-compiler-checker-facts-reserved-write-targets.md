Node: compiler/checker-facts

Decision: A `set` target path carrying any of [TYPE-10]'s eight reserved measure and window-part spellings is refused as that rule's own violation before the target's field walk runs, because [TYPE-10] refuses "a write to a measure, and a read of a window part, a `borrow_expr` over one, and a write of one" while the field walk has no member of those names to find and reports the base as a struct missing a declared field, so `set r.len = 2_u64` and `set r.next = 5_u64` cited a type mismatch on the window instead of the rule that forbids them, instead of leaving the refusal to the field walk, which cannot name the rule because it never learns that the spelling is reserved.

Decision: The read side of that refusal stays where it already is, in the place walker that resolves a written member, because [OP-15] admits a measure member read and only [TYPE-10]'s window parts are refused there, so read and write do not share one test.
