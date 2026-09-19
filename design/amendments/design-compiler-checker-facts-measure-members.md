Node: compiler/checker-facts

Decision: A written place ends at a trailing `.len`, `.cap`, `.room` or `.head` suffix, which the place walker records as the measure the place reads rather than as a field selection, and a further suffix after one is refused, because [OP-15] reads a measure as a member of the measured place and [MSR-1] gives a measure no storage below itself, so a measure is always the last written step and the place it is read over is everything before it, instead of resolving the suffix as a field and relying on the reserved-name refusal to catch it, which refuses the reads [TYPE-10] admits together with the writes it forbids.

Decision: [TYPE-10]'s refusal is sited at a write of a measure and at any use of a window part, and never at a read of a measure, because the rule refuses "a write to a measure, and a read of a window part, a `borrow_expr` over one, and a write of one" and says nothing about a measure read, which [OP-15] is the rule that admits, instead of refusing all eight reserved spellings at every field position, which is the reading that rejects the specification's own prelude.

Decision: A `Box` result whose content the measure table gives a row is an admitted [FN-9] selector datum, and a selector atom that reads a measure stands for a `u64` whatever its measured place's own type is, because [MSR-1] admits a measure place formed with field-selection steps and [TYPE-9] reaches a `Box`'s content by exactly one such step, `b.inner`, which is how every boxed construction record states its own `ensures`, instead of admitting only a result whose own declared type the measure table gives a row.

Rejected:
- Treating a measure member as an ordinary field of a compiler-synthesized descriptor struct: rejected because [TYPE-10] states that the eight spellings occupy no declaration domain, so a synthesized field would give them one and would make a measure's support the field's storage rather than the measured place's [MSR-2].
