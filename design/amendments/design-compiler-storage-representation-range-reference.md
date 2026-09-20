Node: compiler/storage-representation

Decision: A `&[T]` range reference [REF-4] is lowered as a pointer to the first element of the range and the element count, which is the representation the retired shared view had, because [MSR-1] gives `&[T]` exactly one measure, its element count, and [REF-3] keeps the reference from escaping the call, so nothing else of the origin has to travel with it, and the pointer-and-count pair is what every element address and every `len` read already needs, instead of a pointer with the origin's own descriptor beside it, which would carry a capacity and a window origin that no rule of the range reference reads.

Decision: A range reference is never addressed storage: no reference is formed to one, no storage holds one, and nothing is released through one, because [TYPE-8] makes `&[T]` a reference kind and not a type, so it is not a stored value at all, instead of admitting it wherever an owned value may be stored and relying on a later judgment to refuse those positions.

Decision: The measured-row identity of `deref(p).len` where `p` is a range reference is selected from the written base's kind and not from the type the dereference selects, because [TYPE-7] makes `deref` of a `&[T]` denote the run of elements the range names while the type it selects is the element type, so the row [MSR-1] gives `&[T]` cannot be recovered from that type at all, instead of reading the row off the selected type, which answers with the element's row or with none.

Decision: A measured element reached through a range reference retains one typed range-element place carrying the evaluated outer offset and every suffix below it, and lowering feeds the address produced by the ordinary range-address path to the ordinary container-measure operation, because the pointer-and-count range descriptor locates its elements but is not owned descriptor storage that can be represented as a container root, instead of coercing the range into owned storage or adding a second range-element measure representation in the IR.

Rejected:
- Keeping the v0.59 view type and its loan strength as the range reference's representation: rejected because the strength is the component that distinguished the two views and [REF-1] gives a reference no permission marker at all, so retaining it would keep a field every judgment must then ignore.
