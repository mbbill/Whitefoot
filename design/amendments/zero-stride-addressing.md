Node: design/compiler/storage-representation.md

Decision: Ordinary target lowering substitutes zero only for the element-address operand of a step whose selected-target stride is zero, retaining logical indices, lengths and window coordinates in their other uses, because STOR-6 requires exact target-domain representations for the indices actually used in address formation while a zero-stride step has zero displacement for every admitted logical index, instead of rejecting representable zero-byte storage because its logical cardinality exceeds the signed address domain or weakening STOR-6 to permit unrepresentable address operands. The rule uses the checked target layout for the element of each step, including nested places, generated fill and cleanup walks, ranges and transfer-size calculations; an empty final field does not erase a nonzero-stride outer step.

Rejected:
- Depending on optional optimizer facts or LLVM dead-code elimination to erase a zero-stride address: rejected because qualification must hold for ordinary pre-optimization materialization and facts must not select target success.
- Replacing every logical index of a zero-byte element with zero: rejected because bounds, range lengths, window coordinates and source observations retain their original logical values.
