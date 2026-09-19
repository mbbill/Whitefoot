Node: compiler/storage-representation

Decision: A runtime-capacity run's checked root carries the complete path from its binding, including the cell content step [TYPE-9], rather than a list of struct field ordinals, because every such run exists only as the content of a `Box`, so a field-ordinal list could represent no reachable run at all and the checker stopped as an unsupported composite value at `data.inner[i]`; the resolved place [OWN-7] and the [FN-9] goal place are both read from that path, instead of dropping the content step from the root and keeping it only in a separate resolved place, which made the goal place of `data.inner.len` the cell and no published relation over the run ever matched it.

Decision: Lowering walks the same path, reading a field by projection and a cell's content by the ordinary cell dereference, because a cell is a real nominal whose content is loaded and not a transparent wrapper, instead of skipping the content step, which typed the cell itself as the run and stopped lowering.

Decision: An [FN-9] selected-return classification of a measure over such a run supplies no datum, because the classification walks declared struct fields and has no representation for the content step, instead of classifying the cell's own place, which is not the measured place the clause names.
