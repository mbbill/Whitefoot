# wfc

This directory contains the production Whitefoot compiler, written in Whitefoot.
Python `prototype/democ` bootstraps wfc through the facts-off self-hosting
fixpoint. The project then freezes democ as an independent differential oracle;
accepted-language growth continues in wfc.

The compiler currently uses fixed-capacity structure-of-arrays tapes backed by
primitive buffers. Token and node counts are bounded from the source size, so
bootstrap does not require a Rust compiler, growable collections, `pool`, or
general generics. This is the protected compiler baseline rather than a
language-wide data-layout rule. Future container acceptance uses the current
SoA compiler as its control and does not include a wfc migration.
`sources.txt` is the deterministic declaration order for the current whole-program
unit.

The permanent lexer and parser cover the complete syntax used by the compiler
itself: top-level declarations, modes and effect rows, nested types,
places and borrows, constructors and calls, recursive blocks, loops, regions, and
matches. Every AST node owns a distinct token, so node capacity is bounded by token
count. `test_self_parse.py` concatenates the exact `sources.txt` unit, parses it twice,
requires identical token and AST tapes, and runs the kind-agnostic structural
validator over every node. This is the permanent bootstrap grammar gate, not a sample
fixture.

The semantic layer has exact-byte scoped symbols, atomic global and type-member
indexing, structural type equality, and exact type-name/array-size resolution.
Its current whole-unit capability driver is legacy recovery debt: it audits
compiler functions in source order and can publish `CLEAN` through exact
whole-function and subtree profiles. `Unsupported` means only not certified; it
makes no legality claim. The exact-profile route is frozen and must not be
extended.

The production architecture recorded in `../THE-PLAN.md` replaces that route
with one syntax-directed semantic checker, atomic whole-unit acceptance, a
versioned unit-bound elaborated `CheckedUnit`, and one generic lowerer. Delivery
slices add specification-rule transitions to those shared paths and remove the
profiles they supersede; they never become runtime function families. Body
status, unit acceptance, lowering authority, and optimizer facts are separate
gates. Type names and constructor names are deliberately separate namespaces:
this permits the prelude's `Overflow` type and `Overflow()` constructor while
still rejecting duplicate constructors across enums. The prelude names are
recognized from exact bytes and cannot be redeclared. The retained scalar path
contains useful type and node-fact machinery, but its whole-body admission
branch is recovery debt and receives no exemption from the unified pipeline.
`src/source_names.wf` compares
names byte-for-byte without hashes. `src/output.wf` is the capacity-aware, two-pass
byte sink used by the LLVM emitters. `make check` compiles the whole compiler with
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
the body-semantic pass populates them. `wfc_frontend_run(source, report)` remains the
public convenience seam by allocating an `AnalyzedUnit` and forwarding to the retained
pipeline.
Its report identifies the first failed stage and preserves spans, node references, and
the counts completed so far. `test_frontend.py` drives both a mixed unit and the exact
compiler source through this one ABI and checks deterministic counts;
`test_frontend_retained.py` proves successful and failed analyses leave reusable,
guarded internal tapes accessible to the next compiler stage. The final public
ABI remains `wfc_compile(source, output, report)`, which will add LLVM output without
accepting caller-forged internal tapes.

`semantic_body_run` now resolves and types the compiler's scalar byte predicates into
a compact `TypeTape` plus dense `NodeFacts`; `llvm_scalar_emit_function` consumes those
facts without resolving operation names again. The first vertical gate compiles the
real `lexer_is_lower` function to deterministic SSA LLVM, asks clang to load it, and
checks all 256 possible `u8` inputs. The same fact-driven path also emits the analogous
uppercase predicate, while hostile names, types, links, facts, and asymmetric tape
capacities fail closed.

Stage 0 is invoked with optimizer facts disabled for wfc builds until wfc's effect
checking is complete. Parsing now expands grammar slice by slice; semantic checking and
LLVM emission follow. The self-hosting fixpoint uses an in-memory library ABI first, so
runtime I/O, a standalone launcher, and drops do not block it.
The current bootstrap runtime therefore leaks the temporary `buffer_new` allocations
after each front-end call. This is a deliberate, short-lived limitation of stage 0;
compiler-owned drops will replace it before the API is treated as production runtime
surface.

Bootstrap target:

```text
democ -> wfc0
wfc0  -> wfc1.ll -> wfc1
wfc1  -> wfc2.ll
wfc1.ll == wfc2.ll
```

`../THE-PLAN.md` is the sole source for execution order, phase gates, current
coverage, and the route through self-hosting and later language work.
