Node: compiler/prelude-records

Decision: The [PRE-1] declaration record reader represents the specified `fn_decl` head with `fn_sig`'s generated node sequence and `fn_decl`'s generated optional-generics node spliced in after the leading `"fn" IDENT`, and the shape verifier checks that same sequence, because [PRE-1] expressly defines a record as that head without a source body and keeps its final semicolon as table punctuation, while an interface member's `fn_sig` has no generic header, so reusing the generated nodes preserves both written forms without inventing grammar data, instead of adding a writer-visible production or giving a bodyless prelude declaration an empty body.

Decision: [FORM-3]'s reserved-name check treats a prelude declaration role exactly like a source declaration role, because [PRE-1] grants its origin only a diagnostic ordinal and the measure and window-part names no longer occupy a reserved declaration domain, so records using `next` need no exception from the remaining operation-family and mode-word reservations, instead of retaining the retired prelude-origin exemption.

Rejected:
- Reading a [PRE-1] record as a `fn_decl` with an empty body: rejected because the record would then carry a Whitefoot body for [FN-9] to verify, which [PRE-1] states it does not have, and the empty body would fail the ordinary result check of every record that returns a value.
- Duplicating the record's generic header and signature grammar in a handwritten parser: rejected because [PRE-1] defines the record through the generated source productions, so a second grammar inventory could drift when those productions change.
