# Whitefoot compiler

This directory is one safe-Rust crate containing the active compiler. It is an
implementation, not a collection of stable libraries: module boundaries are
private design choices and should change when the next compiler capability
needs them.

The implemented path is currently:

```text
ordered source bundle
  -> lossless lexer
  -> terminal classification
  -> strong-LL(2) parser
  -> finalized source-bound syntax tree
  -> exact FORM-2 validation
  -> direct active-specification lexical name resolution
  -> semantic and ownership checking
  -> private checked program
  -> target-independent typed control-flow IR
  -> selected-host layout and target-domain qualification
  -> conservative textual LLVM
  -> host executable
```

The frontend targets the exact bytes of
`../spec/kernel-spec-v0.18.md`. `cargo run --bin whitefoot-spec` checks that
those bytes are the approved candidate and that the terminal and grammar data
name the same specification identity. The committed grammar tables are
ordinary compiler data. The exact specification identity is versioned data;
compiler stage, type, and API names remain stable across grammar-preserving
specification bumps instead of acquiring a `V0_xx` suffix. For a specification
proposal, run the native verifier through this compiler:

```sh
cargo run --bin whitefoot-grammar -- \
  ../governance/spec-evolution/kernel-spec-vN-candidate.md
```

It verifies that a grammar-preserving proposal keeps the active
specification's complete canonical-format, lexer, and grammar contract
byte-for-byte, checks the committed terminal inventory and every strong-LL(2)
decision, and runs the real lexer and parser over the active tables. A
proposal that changes that frontend contract fails closed: a structural change
must first extend this same native path rather than reviving an independent
grammar engine.

The v0.18 system-interface surface parses under the active grammar but stops
as an explicit unsupported compiler capability pending its implementation
tasks: a kind-declaring entry (`program_kind`, which admits the system
declaration domain), a labelled entry input (`input_label`), and the
`external`/`blocks` effect categories.

FN-7's kind-declaring judgment has one home, `syntax::entry_form`, which reads
it from finalized syntax alone: a unit is kind-declaring exactly when a
`program_kind` node exists, independent of names, types, and effect rows. The
resolver takes that decision in DIAG-1's stage order, after complete unit-wide
FN-8 requires-block admission and before declaration inventory, so an FN-8
rejection outranks the kind-declaring unit's unsupported stop.

The resolver covers every active-specification declaration, lexical-use, and deferred
owner/member role through one grammar-driven path, including exact scopes,
visibility, reservations, collisions, and deterministic diagnostics.

The implemented scalar families support exact fixed-width integers, strict
`f32` and `f64`, `unit`, `Bool`, and unit, integer, and finite floating-point
constants. The function path supports nongeneric functions and the bounded
explicit source-generic subset described below, with locals, direct calls,
returns, and exact source effect-row checking. The integer operation family
includes wrapping and trapping add/subtract/multiply, checked
add/subtract/multiply/divide/remainder, integer
absolute value and negation in all three modes, integer comparisons, Boolean
operations, the remaining OP-8 integer family, and nominal tag equality.
That integer family includes trapping division/remainder, bitwise operations,
shifts, rotates, bit counts, byte swap, high multiply, saturating arithmetic,
and min/max. Every distinct integer-to-integer `cvt` pair uses one exact
conversion path. The 18 value-preserving widening pairs return the destination
directly; the other 38 pairs return `Result<Dst, NarrowError>` after an exact
representability check, never a visible truncation. Checked division and remainder guard
divisor zero and signed minimum/-1 before the partial LLVM instruction and
produce the exact `Result<T, DivError>` variant. Absolute value uses
defined-edge `llvm.abs` for every signed width: wrapping retains the minimum
value, trapping emits OP-2, and checked returns `Err(Overflow())`. Negation uses
modular `sub 0, x` for wrapping and signed-subtraction overflow detection for
trapping and checked modes, with no `nsw`/`nuw` promises.

Scalar `f32` and `f64` run end to end through semantic checking, typed IR,
LLVM emission, and host execution. Source literals use the unique canonical
finite spelling and retain their exact IEEE bits. All 24 direct floating-point
operations execute for both widths without emitted fast-math flags, including
the specified NaN and signed-zero cases. With the integer rows above, `cvt`
covers all 90 ordered pairs of distinct concrete numeric primitives: 29 are
total and return the destination directly; 61 return
`Result<Dst, NarrowError>` under OP-6's exact-conversion rules. The 16 specified
equal-width numeric `reinterpret` pairs preserve source bits. Floats compose
with calls, loop-carried locals, structs, constants, fixed arrays, primitive
runtime buffers, checked indexing, and SET-1.

Acyclic source structs and enums, including reachable concrete generic nominal
instances, flow through the same path with construction, nested projection,
statement/value matching, `give`,
per-site exhaustiveness checking, whole-binding affine moves, and explicit
reverse-order cleanup edges. Struct fields may own buffers; whole and partial
owner cleanup expands to exact projected buffer frees, and consuming field
projections skip only the transferred subtree. Resource-bearing source enums
and concrete `Option`/`Result` instances retain one checked owner drop; the
backend switches on the active tag and recursively cleans only that variant's
resource fields. A consuming match transfers the payload without also dropping
the enum root. SET-1 supports live own-mode copy locals and nested copy
fields, rejects affine replacement under STOR-1, and rechecks target liveness
after the right-hand side. Semantic success produces the only lowering
authority. Concrete fixed arrays support
decimal or earlier-integer lengths, complete `array_new` initialization,
immutable static const tables, `len`, checked index reads, and target-before-RHS
checked indexed writes for direct local roots. The IR retains required checks,
source trap sites, checked set paths, and cleanup. Runtime-length primitive
buffers use a `{data pointer, u64 length}` value, checked OP-9 byte-size
multiplication, a separate selected-target domain guard before allocation,
complete fill initialization, OP-4 reads and target-before-RHS writes,
cross-function affine transfer, and
compiler-derived `free` on normal owner exits. Buffer fields retain exact
projected roots through length, read, and write operations without
re-evaluating source paths.

The source-generic path is a finite monomorphizing subset, not complete generic
support. Functions, structs, and enums support unbounded type parameters, the
built-in `Int` and `Float` bounds, and integer-typed const parameters; every
type and const argument is explicit. Templates are checked symbolically, then
each reachable kind-correct concrete instance is rechecked through the
ordinary semantic path and receives a concrete nominal identity or
collision-free internal function symbol before normal lowering. Acyclic nested
calls and nominal discovery, forwarded const parameters, generic arrays and
primitive buffers, and symbolic and concrete `Option` and `Result` instances
use that same path. There is no argument inference, backend-level generic IR,
or cross-instance body sharing. Generic call cycles, generic functions with
region parameters or `requires`, and type-dependent generic `cvt` or
`reinterpret` are explicit unsupported capabilities. Generic source contracts,
source-contract bounds, and region-bearing generic arguments instead receive
their v0.18-specified source rejections.

The first lexical borrow family adds caller region parameters, local region
blocks, shared and unique buffer holders, explicit `deref`, resolved
field-prefix overlap, and ultimate-origin `reads`/`writes` effects. Borrowed
buffer descriptors cross ordinary calls by value, but only the original owner
is cleaned up. Distinct struct fields can therefore be uniquely passed to a
fill helper and then shared with a fold helper without transferring either
allocation. The backend remains conservative LLVM without unearned overflow
flags or check elision.

Effect rows are checked as exact source-level summaries for every admitted
function. `pure` is the empty effect row, not a termination claim. The
implemented executable paths otherwise track `reads('r)`, `writes('r)`,
`allocates(heap)`, and `traps`, union local expression effects, propagate callee
heap and trap effects, and substitute formal read and write regions onto actual
borrowed-storage and slice origins. The computed row must equal the declared
row, so both missing and superfluous capabilities reject under EFF-2. These
facts currently stop at semantic checking and static-contract compatibility.
The backend emits no effect-derived LLVM function attributes or alias metadata,
licenses no check elision from an effect row, and never emits `willreturn`;
v0.18 has no termination checker.

Target qualification is one private stage immediately before LLVM emission.
The compiler executable fixes an exact aarch64 or x86-64 macOS/Linux triple and
DataLayout, checks concrete representations, statics, source-call ABI objects,
actual emitted stack slots, and complete frames with checked arithmetic, and
reports unrepresentable materialization as a target failure without a source
rule. The checked program and IR retain allocation and element-address
obligations. Fixed-array bounds plus static layout discharge address
representability; buffer bounds plus the successful allocation invariant do
the same; buffer allocation retains an exact non-language guard before
`malloc`. This is not a language array limit, stack-capacity prediction, hidden
heap fallback, or optimizer fact.

Concrete `requires` blocks are checked executable prologues. The semantic
checker admits their restricted own-copy, pure-total ANF subset, retains the
final OP-5 check separately from the body, and combines prologue and body
effects exactly. Lowering executes the prologue after parameter binding and
before the body. Callers do not prove it, and it is never turned into
`llvm.assume` or used to remove a required check. A borrowed-output capacity
program exercises this path through the ordinary loop, buffer, effect, and
cleanup implementation.

The v0.18 compiler retains the static contract family introduced in v0.16 and
checks it before checked-program publication.
A nongeneric source contract contributes its source-ordered unique member
signatures and laws. Each source conformance has one exact concrete subject,
one coherent source-contract key, and exactly one declared-order binding for
every member. A binding names an ordinary nongeneric, `requires`-free top-level
function; compatibility reuses the complete callable signature and compares
normalized read, write, allocation, and trap capabilities after positional
region alpha-renaming. Law-bearing conformances then pass the closed FN-4
discharge before semantic success. The checked program retains the contract
table, complete binding vectors, and base law derivations as semantic evidence.

That evidence is deliberately non-executable. Lowering reads the same ordinary
checked functions and operations as before, ignores the contract metadata, and
creates no contract object, dictionary, vtable, indirect call, runtime check,
ABI component, or optimizer fact. A bound function is emitted only through its
normal direct function path. v0.18 has no contract-member call operation, and
generic source contracts and source-contract generic bounds receive their
specified FN-3 rejections rather than becoming unsupported compiler features.

Concrete PRE-1 `Option<T>` instances reuse the same checked nominal, typed IR,
and LLVM representation as source enums and `Result<T, E>` for every currently
supported payload. `None` and `Some` cross ordinary function, return, and match
boundaries; nested Options are concrete nominal instances, not erased values.
A shared-borrow byte scanner returns `Option<u64>` through this path. A
fallible byte transform exercises owned-buffer Result success, error, matching,
and abandonment cleanup through the same representation.

The implemented borrow family covers buffer owners, whole acyclic struct
owners, copy-field projection, caller-visible read/write effects, and
statement-scoped shared or mode-compatible unique child reborrows around one
call. Boxes, direct own-rooted projected arrays, and direct read-only slices
over arrays, primitive buffers, and immutable const arrays use the same checked
place, cleanup, and backend paths. The compiler deliberately stops before
returned borrows, borrow-producing branch joins, arenas, slices formed through
borrow holders, non-flat slice elements, and non-buffer borrow-backed SET-1
targets. Direct own returned slices use v0.18's finite-origin rules through the
normal semantic, lowering, and unchanged slice-descriptor path. A source
claim belongs to its named data region rather than to one descriptor binding,
so nested scopes preserve it, control joins take its conservative union, and
only leaving that named region releases it.
Non-flat direct slice types likewise retain their language status while the
compiler lacks their value path. Region-bearing function and nominal generic
arguments, region-bearing box or arena content, borrow-mode direct-slice
results, and slice-valued value matches instead receive the specified FN-2,
STOR-5, FN-1, and OWN-5 source rejections. Unimplemented active-specification
families stop as unsupported rather than becoming source-language rejections.
Whole-unit ERR-2 variant-addition edit-list enumeration remains future work.

The ordinary compiler path also executes repository-owned program witnesses for
numeric and generic composition, text and binary transforms, hashing, heap and
borrowed storage, image and signal kernels, network formats, and stored, fixed,
and dynamic raw-DEFLATE blocks under `tests/programs/`. These are regression
evidence for the implemented surface, not external-project validation or
performance claims.

Compile a source file through the normal path with:

```sh
cargo run --bin whitefootc -- source.wf -o program
cargo run --bin whitefootc -- --emit-llvm source.wf
```

There is deliberately no artifact protocol, replay layer, resource-profile
product, or compatibility boundary in front of this path.

Run the compiler gate with:

```sh
make check
```
