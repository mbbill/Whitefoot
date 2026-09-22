Node: design/language/data-model/kernel-minimality.md

Decision: The kernel owns the storage shapes together with exactly those operations that move a window boundary, relocate the slots below one, exchange the value held in an owned place, or change a runtime capacity, namely appending and taking at an end, inserting and removing at an index, moving all of one window into another or splitting one into two, growing a heap-resident prefix window, setting at a subscript, swapping two places, and the in-place update that hands a place's value to a function and commits the result, while growth policy, pools, arenas, keyed tables, clearing, truncation, replacing while keeping the old value, swap-remove, deques, and strings begin as source libraries whose costs are checked against the same native operation contract, because those kernel operations provide the authority source needs to preserve initialized storage but composition alone does not establish equal cost, as the [ordered Vector consumption comparison](../../research/experiments/container-representation/vector-library/RESULTS.md) distinguishes extra source-required movement from lowering costs, instead of a kernel container per policy or a blanket claim that every library composition has no extra runtime cost.

This replaces the node's sole Decision line, specifically its claim that the
listed policies compose "at no extra runtime cost". The existing rejected
alternatives remain. This proposal changes no kernel operation or source
acceptance rule. A measured material gap remains a reason to reopen the
relevant operation and representation; it is not declared acceptable merely
because the source program can be written.
