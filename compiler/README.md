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
`../spec/kernel-spec-v0.16.md`. `cargo run --bin whitefoot-spec` checks that
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

It compares the proposal's complete canonical-format, lexer, and grammar
contract with the active contract, checks the compiler's terminal inventory
and every strong-LL(2) decision, and runs the real lexer and parser. It fails
closed when a proposal changes that contract; a structural change must first
extend this same native path rather than reviving an independent grammar
engine.

The resolver covers every active-specification declaration, lexical-use, and deferred
owner/member role through one grammar-driven path, including exact scopes,
visibility, reservations, collisions, and deterministic diagnostics.

The implemented semantic families support exact scalar integers, unit,
`Bool`, integer and unit constants, nongeneric own-mode functions, locals,
direct calls, returns, `pure`, `traps`, and heap-allocation effects, wrapping and trapping
add/subtract/multiply, checked add/subtract/multiply/divide/remainder, integer
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
trapping and checked modes, with no `nsw`/`nuw` promises. Nongeneric acyclic
structs and enums flow through the same path,
including construction, nested projection, statement/value matching, `give`,
per-site exhaustiveness checking, whole-binding affine moves, and explicit
reverse-order cleanup edges. Struct fields may own buffers; whole and partial
owner cleanup expands to exact projected buffer frees, and consuming field
projections skip only the transferred subtree. Resource-bearing source enums
and concrete `Option`/`Result` instances retain one checked owner drop; the
backend switches on the active tag and recursively cleans only that variant's
resource fields. A consuming match transfers the payload without also dropping
the enum root. SET-1 supports live own-mode copy locals and nested copy
fields, rejects affine replacement under STOR-1, and rechecks target liveness
after the right-hand side. Semantic
success produces the only lowering authority. Concrete fixed arrays support
decimal or earlier-integer lengths, complete `array_new` initialization,
immutable static const tables, `len`, checked index reads, and target-before-RHS
checked indexed writes for direct local roots. The IR retains required checks,
source trap sites, checked set paths, and cleanup. Runtime-length non-floating
primitive buffers use a `{data pointer, u64 length}` value, checked OP-9
byte-size multiplication, a separate selected-target domain guard before
allocation, complete fill initialization,
OP-4 reads and target-before-RHS writes, cross-function affine transfer, and
compiler-derived `free` on normal owner exits. Buffer fields retain exact
projected roots through length, read, and write operations without
re-evaluating source paths.

The first lexical borrow family adds caller region parameters, local region
blocks, shared and unique buffer holders, explicit `deref`, resolved
field-prefix overlap, and ultimate-origin `reads`/`writes` effects. Borrowed
buffer descriptors cross ordinary calls by value, but only the original owner
is cleaned up. Distinct struct fields can therefore be uniquely passed to a
fill helper and then shared with a fold helper without transferring either
allocation. The backend remains conservative LLVM without unearned overflow
flags or check elision.

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

The v0.16 static contract family is checked before checked-program publication.
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
normal direct function path. v0.16 has no contract-member call operation, and
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
call. It deliberately stops before returned borrows, borrow-producing branch
joins, boxes, arenas, slices, and storage roots not handled by those general
paths. Those forms remain explicit unsupported compiler capabilities; they are
not accepted with incomplete loan checking. Unimplemented active-specification families stop
the same way rather than becoming source-language rejections. Whole-unit ERR-2
variant-addition edit-list enumeration remains future work.
Projected array roots, slices, and non-buffer borrow-backed SET-1 targets
remain unsupported until their place families exist; none of these gaps is
implied complete by the current gate.

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
