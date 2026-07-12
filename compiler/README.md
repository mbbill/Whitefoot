# xlc

This directory contains the production xlang compiler, written in xlang. Python
`prototype/democ` is stage 0 only: it compiles xlc until xlc can compile itself.

The compiler uses fixed-capacity structure-of-arrays tapes backed by primitive
buffers. Token and node counts are bounded from the source size, so bootstrap does
not require a Rust compiler, growable collections, `pool`, or general generics.
`sources.txt` is the deterministic declaration order for the current whole-program
unit.

Current milestone: the permanent lexer and parser cover the complete syntax used by
the compiler itself: top-level declarations, modes and effect rows, nested types,
places and borrows, constructors and calls, recursive blocks, loops, regions, and
matches. Every AST node owns a distinct token, so node capacity is bounded by token
count. `test_self_parse.py` concatenates the exact `sources.txt` unit, parses it twice,
requires identical token and AST tapes, and runs the kind-agnostic structural
validator over every node. This is the permanent bootstrap grammar gate, not a sample
fixture.

The semantic layer now has exact-byte scoped symbols, atomic global and type-member
indexing, structural type equality, and exact type-name/array-size resolution. Type
names and constructor names are deliberately separate namespaces: this permits the
prelude's `Overflow` type and `Overflow()` constructor while still rejecting duplicate
constructors across enums. The prelude names are recognized from exact bytes and cannot
be redeclared. The remaining expression, ownership, and effect checks grow from those
pieces. `src/source_names.xl` compares
names byte-for-byte without hashes. `src/output.xl` is the capacity-aware, two-pass
byte sink the LLVM emitter will use. `make check` compiles the whole compiler with
stage 0 and exercises these native C ABIs, including hostile capacities and malformed
internal tapes. The lexer oracle normalizes stage 0's obsolete broad dotted-word token
to the current closed-suffix `OPNAME` rule, so fields are `word . word` while only
`.wrap`, `.trap`, `.checked`, `.sat`, and `.strict` remain atomic operation names.
That distinction is intentional: OP-1 reserves every mode word and dotless operation
name from user binding sites, so `value.trap` is never a legal field place.
Type names and unqualified constructor names also have distinct symbol namespaces:
generic shapes such as `enum Marker { Marker(); }` are legal, while PRE-1 type and
constructor spellings are reserved and constructor names must still be unique across
enum declarations.

`lexer_run`, `parser_run`, and their tape structs are internal compiler/test seams.
The parser trusts the lexer's typed token tags and byte classification, while validating
tape lengths, ordered spans, and the unique source-end token before building any AST.
`AnalyzedUnit` retains the bounded token, AST, validation, symbol, type, and node-fact
tapes needed by later compilation stages. `frontend_unit_new` allocates those tapes
once and `frontend_analyze_into` resets and reuses them across lexing, parsing,
structural validation, global and type-member indexing, whole-unit type resolution,
and function-scope indexing. Type and node-fact tapes intentionally remain empty until
the body-semantic pass populates them. `xlc_frontend_run(source, report)` remains the
public convenience seam by allocating an `AnalyzedUnit` and forwarding to the retained
pipeline.
Its report identifies the first failed stage and preserves spans, node references, and
the counts completed so far. `test_frontend.py` drives both a mixed unit and the exact
compiler source through this one ABI and checks deterministic counts;
`test_frontend_retained.py` proves successful and failed analyses leave reusable,
guarded internal tapes accessible to the next compiler stage. The final public
ABI remains `xlc_compile(source, output, report)`, which will add LLVM output without
accepting caller-forged internal tapes.

Stage 0 is invoked with optimizer facts disabled for xlc builds until xlc's effect
checking is complete. Parsing now expands grammar slice by slice; semantic checking and
LLVM emission follow. The self-hosting fixpoint uses an in-memory library ABI first, so
runtime I/O, a standalone launcher, and drops do not block it.
The current bootstrap runtime therefore leaks the temporary `buffer_new` allocations
after each front-end call. This is a deliberate, short-lived limitation of stage 0;
compiler-owned drops will replace it before the API is treated as production runtime
surface.

Bootstrap target:

```text
democ -> xlc0
xlc0  -> xlc1.ll -> xlc1
xlc1  -> xlc2.ll
xlc1.ll == xlc2.ll
```

`PLAN.md` is an older design record. Its feature inventory and `pool` proposal are not
the current implementation plan.
