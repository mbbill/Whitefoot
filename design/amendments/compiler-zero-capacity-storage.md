Node: compiler/storage-representation

Decision: A constant-capacity zero Array, Slots or Ring omits its nonexistent element's target alignment by representing the empty payload as a zero-length byte array, while Slots and Ring retain their ordinary descriptor words, because OP-9 repeats the element layout zero times and nested owners must be qualified against the representation actually emitted, instead of preserving a typed empty tail that can enlarge the header and stride beyond the calculated layout or rejecting an otherwise representable empty owner. Runtime-capacity tails remain typed because they can hold elements; a nonzero capacity of a zero-size element is a different case and keeps its ordinary coordinate rules.

Rejected:
- Retaining the high-alignment typed tail and rejecting nested zero-capacity owners under the existing language ceilings: rejected because no element is present to require that alignment, and the existing zero-capacity Array representation supplies a semantics-preserving omission without reducing container capability.
- Treating a zero-length typed LLVM array as alignment-free: rejected because its element still determines the LLVM ABI alignment, so header, stride and parent-object qualification can disagree with actual emission.
