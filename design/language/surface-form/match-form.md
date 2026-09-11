Decision: The conditional form is selected by the scrutinee's type and never by the writer, a Bool condition takes `if` and an enum scrutinee takes `match`, because a Bool scrutinee's two arm labels are always exactly `True()` and `False()` in fixed order, so the arm ceremony carried no information the condition did not already carry, instead of match as the sole conditional.

Decision: A conditional value is a `let` initializer whose every arm or branch ends in exactly one `give` or diverges, because conditional initialization is the most common pattern an AI writer needs and delivering the value removes a mechanical helper function per conditional value, instead of extracting a helper function that returns from each arm.

Decision: The scrutinee is a full expression, because restricting it to a place taxes the sole conditional idiom with a mechanical temporary at every use and adds naming burden for a weak writer, instead of binding the value first and matching on the name.

Decision: An else-free conditional is the one spelling of the empty alternative, an empty `else` block is rejected, and an `else` holding exactly one conditional flattens to `else if`, because one construct has one spelling, instead of admitting both spellings of an empty alternative.

Rejected:
- Match as the sole conditional construct, including over Bool: rejected because a Bool scrutinee's two arm labels are always the same two in fixed order, so the ceremony carried no information the condition did not already carry.
- A scrutinee restricted to a place, binding computed values to a temporary first: rejected because it taxes the sole conditional idiom with a mechanical temporary at every use and adds weak-writer naming burden.
- Statement-only match with conditional initialization extracted into a helper function returning from each arm: rejected because conditional initialization is the most common pattern an AI writer needs, the helper idiom's only recorded ground was that it was cheapest to specify, and value delivery removes a mechanical helper function per conditional value.
