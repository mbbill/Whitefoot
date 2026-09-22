Node: design/language/name-resolution.md

Decision: Invariant names admit every lexical IDENT without OP-1's additional operation and mode-word reservation, because their declaration and premise positions select a separate proof-only domain that cannot compete with callable lookup or field-token formation, instead of coupling proof names to the runtime operation inventory.

Rejected:
- Reserve operation and mode words from invariant declarations: rejected because invariant names are neither callable nor field names, so neither operation-versus-function resolution nor OPNAME maximal munch supplies a reason to exclude these spellings from the separate proof domain.
