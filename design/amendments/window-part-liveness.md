Node: language/ownership

Decision: An indexed position of a window is separated from the window's append slot, its free slots or its last filled slot only where the state in which the two places are compared proves the index below the window's length, a place formed or referenced in that state being live by its own bounds obligation, a fact's place at a killing event by that event's entry state, and an index an effect row takes from another argument by the call's entry state, because an index at or beyond the length can name the very slot an append fills, instead of assuming every indexed position live. The [readonly-field term investigation](../../research/investigations/readonly-field-terms/DESIGN.md#soundness-every-change-is-an-overlapping-write) records the probe that exposed the gap and the cases that pin the rule.

Rejected:
- Assuming every indexed position live: rejected because a fact's place need not have been bounded where the event happens, and a requirement over an element not yet appended kept its fact across the append that created the element.
- Never separating an indexed position from the append slot: rejected because an appending call must be provable not to disturb a live reference or a fact about a filled slot, which is the reason the window parts exist.
- Treating an index an effect row takes from another argument as never live: rejected because a call whose requirement bounds that index proves it at entry exactly as it proves two indices distinct, and refusing it would reject aliasing calls whose positions the part definitions already separate.
