Node: compiler/storage-placement

Decision: A consumed aggregate may reuse a same-typed whole result or the storage of one consuming field of a complete struct result once the callee has captured its inputs, with complete parent size, alignment and other live storage retaining the ordinary interference checks, because STOR-7 permits relocation without observable address identity and the retained priority helper exposes redundant whole-run transfers this reuse removes, instead of restricting reuse to whole results or promising a stable address in the language. WIN-3's whole-owner consume ends sibling uses; it does not remove the complete allocation's layout obligations.

Rejected:
- Treating one consumed child as permission to reuse an undersized or misaligned parent result allocation: rejected because the complete result still has to fit, including when the returned child is smaller than its parent.
