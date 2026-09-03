# Supporting direction notes

Status: NON-AUTHORITATIVE DETAIL.

The living [`Direction Outline`](roadmap.md) owns each idea's current state,
next evidence gate, and candidate-project links. This file preserves longer
mechanism sketches and possible first experiments; it does not order work.
An entry here does not change the language, compiler, runtime, or deployment
contract.

Whitefoot keeps facts in source that other languages discard. LLVM optimization
is one consumer of those facts. The ideas below ask whether the same checked
facts can also buy portability, automated tuning, stronger testing, safer
interop, or tighter deployment controls.

## A portable C backend

Whitefoot could emit C from the same checked typed IR that feeds the LLVM
backend. Users would write Whitefoot, retain Whitefoot's safety rules, and use
C toolchains to reach platforms that lack a supported LLVM path.

Standard C cannot express all LLVM properties. Per-instruction alias scopes,
`dereferenceable`, precise memory effects, `willreturn`, and several arithmetic
facts have no exact portable-C spelling. A sound backend therefore needs one
recorded disposition for each fact:

1. consume the fact while generating C;
2. encode it in portable C;
3. encode it in a reviewed compiler profile such as Clang, GCC, or MSVC; or
4. reject that target backend when it cannot preserve the already-checked
   operation without undefined behavior.

The backend must generate defined C. It must avoid signed overflow on every
admitted execution, out-of-bounds pointer formation, invalid shifts,
unjustified `restrict`, padding reads, alignment violations, and assumptions
that can become undefined behavior. The portable profile would promise correct
compilation. Named compiler profiles could make separate, measured performance
claims.

First experiment: lower a small corpus that covers bounds discharge, exclusive
borrows, checked arithmetic, claims, and effect attributes. Compile it with two
C compilers, compare results and claim failures with the LLVM backend, run C
sanitizers, and inspect both assembly and throughput. Stop if the backend needs
an unreviewable undefined-behavior assumption or cannot preserve a checked
operation.

## Proof-guided autotuning

An autotuner could generate several implementations of one checked operation:
different data layouts, unroll factors, SIMD widths, branch structures, or
specializations. The checker would admit variants after proving that they
preserve the source contract. A target benchmark would choose among them.

This separates semantic authority from cost selection. The benchmark runner
may choose the fastest proved variant, but benchmark noise cannot make an
unsafe variant legal.

First experiment: choose one bounded kernel with a scalar reference and two
plausible fast shapes. Freeze the input distribution and target, verify every
variant against the same differential corpus, then measure whether target
selection beats a fixed compiler choice without expanding the trusted base.

## A proof-gap performance coach

The compiler could explain each residual static obligation, retained claim,
and missed optimization in terms the writer can act on. A report might say
that an index is rejected because no dominating fact proves
`offset + 16 <= len`, that a hot path still executes a named claim, or that a
loop cannot vectorize because two live places may overlap.

An automated tool could propose a rewrite constrained to the canonical
patterns, run the checker and performance protocol, and present the source
diff, proof delta, and measurement for human approval. The tool would never add
an assumption, remove a claim, or weaken an obligation. It would change source
structure until the checker derives the needed fact.

First experiment: use the observational proof reports planned for the
facts-on compiler. Select ten rejected proof gaps, hot claims, or alias
barriers, generate one mechanical suggestion for each, and measure suggestion
validity, proof closure, code-shape change, and runtime change. Preserve every
failed suggestion as a regression for the diagnostic or rewrite rule that
produced it.

## Multiple backends as mutual oracles

Independent LLVM, C, and future WebAssembly backends could compile the same
checked program. A differential runner would compare values, typed outcomes,
external effects, and resource teardown. Each disagreement would produce the
smallest practical regression before a backend fix closes.

This approach can catch a lowering defect that source conformance misses. It
also gives the C backend value before its generated code reaches the LLVM
backend's performance.

First experiment: run the existing codegen corpus through LLVM and portable C.
Compile the C at low and high optimization levels with two compilers. Require
the same result and typed-failure class for valid inputs and boundary cases,
including failure paths.

## Safe C ABI capsules

Whitefoot could package a module as generated C plus a generated header. The
header would expose opaque validated handles, constructors, operations, and
drop functions instead of internal pointers or layouts. Boundary code would
validate lengths, tags, handle generations, ownership transitions, and error
paths before Whitefoot code receives authority.

This would let a C program consume a Whitefoot library without asking the C
caller to reproduce Whitefoot's lifetime and alias rules. Arbitrary C code can
still corrupt its own process, so stronger isolation would require a process or
sandbox boundary.

First experiment: export one stateful component through an opaque-handle API.
Generate misuse tests for stale handles, double drop, overlapping buffers,
short outputs, and allocation failure. Require deterministic rejection and no
partial mutation on each failing call.

## A C-to-Whitefoot assumption extractor

A migration tool could ingest a restricted C kernel and identify the
assumptions that make it work: bounds assertions, `restrict`, signed-overflow
expectations, alignment, object lifetime, and unchecked pointer arithmetic.
The tool would translate supported code into Whitefoot and turn each assumption
into an explicit checked obligation. It would reject code whose behavior
depends on an assumption Whitefoot cannot state or prove.

The extractor should favor an incomplete translation over invented semantics.
Its main artifact would be an assumption ledger that a reviewer can inspect
before approving the translated Whitefoot source.

First experiment: select a small, defined-behavior C loop with one bounds
contract and one alias contract. Mutate the contracts one at a time and require
the extractor or Whitefoot checker to reject the corresponding program.

## Effect-derived sandbox policies

Once Whitefoot has production I/O and FFI, the compiler could derive a sandbox
manifest from checked effect rows. A deployment tool could translate that
manifest into a WASI capability set, syscall policy, filesystem allowlist, or
network policy for a named platform.

The effect system would remain the source of authority. Platform policy
generators would consume its output and fail closed when the platform cannot
represent a restriction.

First experiment: define a tiny abstract effect set and one sandbox target.
Generate policies for pure, read-only, and network-using fixtures. Mutation
tests should add one hidden effect at a time and require either a broader
manifest or compiler rejection.

## Narrow semantic domains and automatic niches

AI-written code should choose the narrowest honest type that contains every
legitimate value. It should not default to a broad integer for convenience, and
it should not manually reserve a sentinel value. Existing narrow integer types
already help when the domain fits them. A future language experiment could add
compiler-checked refined integers for domains such as “nonzero `u64`” or
“`u64` from zero through `2^63 - 1`.” Any syntax used to express those domains
would be a separate language decision; these examples describe semantics only.

The compiler would make the refinement invariant unforgeable. Construction and
conversion would require a static proof, a typed fallible constructor for an
expected invalid input, or an explicit named claim for an asserted invariant.
Arithmetic would preserve the refined type only when its result remains in the
declared domain. The layout pass could then derive invalid bit patterns
automatically and use them as enum niches:

- `Option<nonzero u64>` could encode `None` as zero and every nonzero bit
  pattern as `Some`.
- `Option<u64 constrained below 2^63>` could use the high bit to distinguish
  `None`.
- An unconstrained `Option<u64>` still cannot fit in one `u64`; all `2^64` bit
  patterns are valid payloads, so reserving one would lose a legitimate value.

There is no universally best physical layout. A standalone value, a packed
record, and a collection with a separate presence bitmap may need different
representations. The semantic type supplies the valid-value set; the compiler
chooses and measures the context-appropriate representation without changing
that set.

The writer guidance should be: choose the narrowest type that contains every
legitimate value, but never invent a bound from a practical assumption. For
example, an offset remains `u64` unless its specification or enclosing type
actually establishes a smaller range. A compiler diagnostic could suggest a
narrower domain when a declared contract or proof supports it, but the compiler
must not silently narrow program semantics.

The first experiment should use a real bounded identifier or offset domain and
compare an ordinary `Option<u64>` with a refined representation for size, ABI,
code generation, and measured throughput. Boundary tests must show that every
valid value round-trips and every invalid construction is rejected. Stop if
maintaining the refinement costs more complexity or runtime work than the
measured layout benefit justifies.

## Common admission questions

Before an owner promotes one of these ideas, its experiment should answer:

- Which checked proposition does the idea consume?
- Does the consumer affect correctness, performance, or both?
- Which producer and invalidators govern the proposition?
- What safe behavior remains when the target cannot represent the fact?
- Which premise-removal case demonstrates that the consumer fails closed?
- Which correctness, code-shape, and performance observations decide the
  experiment?
- Which result stops the work instead of expanding its scope?

Before one of these notes enters an execution proposal, its Direction Outline
item must name the current producer, consumer, invalidators, safe fallback,
premise-removal case, decision observation, and stop condition that actually apply.
Until then it remains supporting detail rather than compiler work.
