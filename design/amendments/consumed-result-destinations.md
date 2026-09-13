Node: compiler/storage-placement

Decision: A consumed aggregate may reuse a same-typed whole result or one consuming field of a complete struct result after the callee has captured its inputs, with the complete parent allocation and all other live storage retaining ordinary interference checks, because the retained priority helper exposed redundant transfers of its entire inline run and this reuse removes them without changing the source ownership boundary, instead of allowing reuse only for whole results or guaranteeing same-place behavior in the language.

Rejected:
- Treating one consumed child as permission to reuse storage belonging to the complete result: rejected because later sibling reads and the parent's size and alignment remain live obligations, including when the returned child is smaller than its parent.
