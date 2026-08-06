- System types and operations resolve from a distinct compiler-owned system-declaration domain that is neither the prelude nor the gated boundary family.
- The domain is signature-shaped: system calls take named field-initializer arguments like user functions, not positional table operands.
- Domain visibility is keyed on the syntactic program-kind declaration alone; a unit whose entry declares no kind sees no system names.
- The amendment surface is a third admitted source in TYPE-6's nominal, constructor, and lexical-IDENT rows, OP-1 callee sourcing, PROG-1 name definition, and one new DIAG-1 collision rank and origin kind.

## Facts

- 2026-08-05 measurement: none of the 64 first-slice spellings — 14 nominals, 39 constructors, 11 operation names — occurs as a token in any of the 466 active `.wf` sources, so either route's reservation cost is prospective, landing on future writers and the growing conformance corpus. (sourced)
- 2026-08-05 statement: recorded fallback — if the syntactic conditional-visibility mechanism is declined during specification drafting, the selection falls back to the prelude extension, which is then strictly cheaper: without that mechanism this route carries an extra TYPE-6 row, a PROG-1 amendment, and a new DIAG-1 rank and origin kind for no remaining advantage. (sourced)
- 2026-08-05 rationale: GRAM-11 partitions call spelling by declaration domain — user functions take named arguments, OP-1 table operations take positional operands — and every system call in the selected design is written with named arguments, so the operation-table home was excluded by spelling before preference entered. (sourced)

- 2026-08-06 pitfall: a system operation's parameter names are call-site fieldinit IDENT spellings, so they must avoid every fixed terminal spelling; v0.18 shipped `arg_get(..., index: ...)` with `index` a fixed GRAM-5 atom, making every complete legal call underivable until the v0.19 rename to `position` — future inventory additions must sweep parameter, field, and label spellings against the full fixed-terminal set before approval. (code)
## Moves

- 2026-08-05 (8f7055fc) replaced [[prelude-extension]]: extending PRE-1 reserves the slice's nominals and constructors, including ordinary spellings such as Other, NotFound, Interrupted, and ReadOnly, unconditionally in every program whether or not it performs I/O, and would put function signatures in the prelude for the first time; a distinct domain scopes reservation to units that declare system inputs (sourced)
- 2026-08-05 (8f7055fc) replaced [[gated-boundary-family]]: the specification fixes exactly one gated boundary-construct family with one shared per-fact obligation record, so housing system operations there merges them with FFI extern frames against the required system/FFI separation; the system interface needs a home that is not the foreign-trust wall (sourced)
