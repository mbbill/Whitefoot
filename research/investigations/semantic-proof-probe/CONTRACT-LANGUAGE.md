# Contracts for Generalized Test Intent

Whitefoot should investigate a specification language with mathematical data,
named laws, abstract state, and observable histories. Its expression boundary
should be broader than its automatic proof fragment. A promising design keeps
proof discovery outside acceptance, checks explicit derivations, and connects
each contract to actual program operations through a specified state model.

This is an exploratory recommendation, not a selected language amendment.
Existing verification systems establish that much of this is possible. They
do not establish that this combination will meet Whitefoot's checking-cost,
source-proof, ownership, and reviewability requirements. The most useful next
experiment is a small streaming decoder with content, failure, and composition
contracts; another scalar arithmetic example would miss the central questions.

## Scope and evidence

The Whitefoot baseline is revision
`ef688430bca69b675237ce7ad94e8d9475668ff6`. The
[active specification](../../../spec/kernel-spec.md), particularly FN-8,
FN-9, ENT-1 through ENT-6, MSR-3, INV-1, and PRF-1, owns language behavior.
The [compiler guide](../../../compiler/README.md#source-proof-checking) owns
implementation coverage. Living external documentation was consulted on
2026-09-10 UTC; historical papers retain their publication dates below.

The test examples are a deliberately varied sample of requirements, not a
measurement of the percentage of software tests that can be replaced.
External verification results are attributed to their authors; no independent
reproduction of those toolchains is claimed. The mathematical counterexamples
below are analytical arguments, not newly machine-checked Whitefoot proofs.

The [existing search probe](DESIGN.md) supplies a narrower executable example
and an explicitly hypothetical proof sketch. Its finite runtime checks are
evidence about those executions. They do not establish the proposed universal
semantic contract, and the hypothetical syntax is not accepted by the compiler.

## 1. The semantic object to preserve

A generalized test intent is a claim about a family of permitted executions.
For a deterministic, terminating function, it may be a relation between an
input and a result. For a service, it may concern an entire interaction. For
a cache, compiler optimization, or confidentiality property, it may compare
several executions. These distinctions determine what the language must be
able to mention.

For a single-execution property, the following mathematical notation, which is
not WF syntax, gives a useful schematic statement:

```text
For every environment E satisfying assumptions A,
and every execution t of implementation P in E:
    contract C holds of the selected observations of t.
```

This is mathematical notation, not proposed WF syntax. An observation might
be a returned tree, writes to a caller-owned buffer, emitted messages, or a
logical cost counter. Selecting observations is part of the specification:
ignoring an error code, intermediate output, or termination changes the promise.
Properties relating several executions require a corresponding multi-execution
statement; the single-execution form is not universal enough by itself.

Partial specification is useful. A sorting contract can initially require
ordered output and preservation of the input multiset without specifying an
algorithm, memory layout, or stability. Adding stability later strengthens
the requirement. Within a fixed environment and observation model, conjoining
a new property reduces the set of permitted implementations.

Three changes do not have that simple interpretation:

- Strengthening a precondition reduces the inputs for which service is owed.
  Changing "all JSON inputs" to "inputs shorter than a kilobyte" can make a
  proof easier while weakening the public commitment.
- Changing the observation model can hide formerly visible behavior. Comparing
  final buffers says nothing about whether incorrect bytes were published
  earlier over a network.
- Changing a helper definition can change every contract using it. Keeping
  the name `ValidJson` while redefining it as "the parser accepts" removes
  the independent standard against which the parser was being judged.

The reviewable unit is consequently the contract's semantic dependency closure:
its definitions, domains, observations, environment assumptions, and imported
laws. Proof scripts and implementation details need not normally be reviewed
for claims already established by a trusted checker. The dependency closure
cannot be replaced by the spelling of the top-level proposition.

This also resolves the apparent regress of reviewing tests. A machine can
establish that an implementation satisfies a formal claim. A person still
decides whether that claim captures a useful obligation. Examples, negative
examples, and derived laws help inspect this decision; they do not need an
independent proof that they capture all human intent.

## 2. Expression, proof, and checking are separate axes

An executable predicate on concrete data is already more general than a list
of input/output fixtures. Property-based testing demonstrates this distinction:
QuickCheck runs generated instances of stated properties. Passing those
instances does not prove the universally quantified property. The reusable
asset is often the property, even when its first checking method is sampling.
[QuickCheck documentation](https://hackage.haskell.org/package/QuickCheck).[^1]

Five separate questions should govern a contract design:

| Question | Failure if neglected |
|---|---|
| Can the intent be stated without choosing an implementation? | The contract becomes a second copy of the algorithm. |
| Can a person understand the statement and its dependencies? | A small top-level clause conceals an enormous or circular definition. |
| Can code be connected to the statement? | Correct mathematics describes a ghost model unrelated to actual writes. |
| Can sufficient evidence be constructed? | A meaningful contract remains an unproved obligation. |
| Can that evidence be checked predictably and affordably? | Rich syntax produces an unusable acceptance path. |

Making a proposition expressible does not require a compiler to decide whether
it is true. A proof of a universally quantified law can introduce an arbitrary
value and derive the result; it need not enumerate all values. Conversely,
executability alone does not make universal validity decidable.

This distinction has an important cost consequence. A small logical definition
may describe an enormous computation. A terminating model function can take
exponential time; a compact proof can request normalization that expands an
exponential term. "There is an explicit proof" and "the checker terminates"
are both weaker than the [constitutional requirement](../../../docs/constitution.md)
for practical checking without exponential growth in program or written-proof
size. There is also no general promise that every useful true claim has a
short proof in a chosen calculus.

Whitefoot can accept incompleteness in proof automation without making
acceptance dependent on solver timeouts. A difficult obligation can require
more written evidence or a different implementation. The compiler must still
finish the specified checking procedure for an admitted source form.

## 3. Test intentions that a contract language must distinguish

The following catalog mixes repository witnesses with explicitly illustrative
requirements. Each row describes a property family; it does not assert that
the present WF contract language expresses or proves that family.

| Intent and witness | Generalized commitment | Required vocabulary and boundary |
|---|---|---|
| First occurrence; [search probe](search.wf), `scan_range` | A returned position is a hit with no earlier hit, or the entire range has no hit. | Sequences, bounded quantification, an absence case, termination if required. Bounds alone are insufficient. |
| Bad stored-block length; [decoder vectors](../../../tests/programs/raw_deflate_vectors.wf), `check_invalid_length` | A complete first stored header with inconsistent LEN/NLEN is rejected without changing destination contents. | Byte interpretation, error alternatives, entry/post-state images, frame conditions. "First" matters. |
| Truncated stored header; same file, `check_truncated` | EOF inside this header yields the specified error and leaves output unchanged. | Input-prefix classification and error precedence. An arbitrary proper prefix of a valid document need not be invalid. |
| Overlapping LZ77 match; same file, `check_fixed_match` and `check_fixed_phrase` | A back-reference repeats the appropriate suffix of already produced history, even when match length exceeds distance. | Content sequences, old history, arithmetic indexing, alias-aware updates. |
| Destination exhaustion; same file, `check_full`, `check_dynamic_full`, `check_fixed_full` | The selected failure route has the promised written prefix and frame. | Result-indexed state relations. These tests do not establish one global all-errors-are-atomic policy. |
| Multiple blocks; same file, `check_mixed_block` | Successive blocks extend the same decompressed history and preserve prior output. | Composition laws over logical state, including cross-block back-references. |
| Replacement invalidates old facts; [replacement tests](../../../compiler/src/semantic/tests/replace.rs), `replace_kills_the_stale_length_fact_at_the_commit` | Replacing a four-element value by a two-element value cannot preserve a current-length justification for index three. | State identity and invalidation. This is a compiler proof-transport obligation, not only an application postcondition. |
| Optimization preserves behavior; [percent-decoder verifier](../../experiments/default-floor/percent-decode/harness/src/bin/verify.rs) | Identical source has matching observations with optimizer facts enabled and disabled. | Relations between executions and compiler modes. Existing finite differential checks are not a universal compiler theorem. |
| Streaming JSON; illustrative | Any finite partition of the same bytes, followed by EOF, produces equivalent terminal observations under the same resource policy. | Histories, partitioning, error offsets, pending versus committed output, termination. |
| Incremental layout/cache; illustrative | For a fixed logical scene snapshot and supported layout fragment, the cached result agrees with the declared layout relation. | Abstract state, dependencies including fonts/viewport/style, an observation projection. Internal cache layout need not be public. |
| Cancellation; illustrative | After cancellation is acknowledged, no further result for that request is published. | Event order, request identity, commit boundary. "Eventually acknowledges" is a separate progress obligation. |
| Concurrent queue; illustrative | Completed operations admit a sequential explanation consistent with their real-time order and the queue model. | Histories, existential witnesses, concurrency semantics. Data-race freedom does not imply this property. |
| Runtime/resource bound; illustrative | Work or allocation is bounded by a function of input size in a stated cost model. | Logical counters or resource accounting; overflow and failure routes. CPU time needs additional hardware/environment assumptions. |
| Confidentiality; illustrative | Public observations do not reveal the selected secret inputs under a declared information-flow model. | Relations among runs or sets of runs. Ordinary output validity is insufficient. |

This sample supports a qualitative conclusion: scalar arithmetic plus
quantifiers would leave important gaps. It does not support a numeric claim
such as "most tests disappear." Some tests intentionally pin a single incident,
an interoperability example, a platform result, or a visual judgment. A
generalization must be justified rather than inferred from extremeness alone.

## 4. Worked contract families

All notation in this section is explanatory pseudocode, invented for this
report. It is not existing Whitefoot syntax and is not a proposed final grammar.
`Seq<T>` means an immutable mathematical sequence; `++` means concatenation;
`length` counts elements; slices use half-open bounds. `before` and `after`
denote logical snapshots, not runtime copies. Quantifiers range over the
declared domains. Program operations named in a contract require a verified
connection to their actual WF implementation.

### 4.1 Failure preserves arbitrary caller data

The existing malformed stored-block fixture initializes one output byte to a
sentinel. Its intended guarantee can cover every destination size and every
initial byte pattern. Define `BadFirstStoredHeader(input)` independently:

- At least five bytes are present, starting at the first DEFLATE block.
- The first byte's BTYPE bits select a stored block; after alignment, bytes
  one and two encode LEN in little-endian order, and bytes three and four
  encode NLEN in the same order.
- The 16-bit complement relation between LEN and NLEN fails.

These fields and alignment are defined by
[RFC 1951, section 3.2.4](https://www.rfc-editor.org/rfc/rfc1951.html#section-3.2.4).[^2]
The proposed application commitment is:

```text
For every input and writable destination:
    if BadFirstStoredHeader(input), then
        inflate terminates with InvalidStoredLength;
        destination.after = destination.before;
        input.after = input.before.
```

This deliberately specifies error priority as well as data preservation: even
a zero-capacity destination must report this malformed header before attempting
output. It is a candidate contract generalizing the fixture, not a claim that
the fixture proves that priority universally. Every indexed header access in
the definition has a domain obligation established by the five-byte premise.

Two tempting generalizations are incorrect. A malformed later block may follow
valid blocks that already produced output. Also, the current
[`copy_distance`](../../../tests/programs/raw_deflate_dynamic.wf) calls
`emit_byte` repeatedly and can return an output-capacity error after earlier
bytes were written. A rule that every decoder error restores the entry buffer
would silently demand different semantics.

The crucial new capability is a relation over arbitrary old and new contents,
not another numeric measure such as length. The bridge must establish that an
actual store changes the post-state image and that an untouched region retains
its previous image. Merely possessing `uniq` authorizes exclusive access; it
does not assert content equality across that access.

### 4.2 Compression requires a relation, not a canonical encoder

For a lossless codec, `decode(encode(x)) = x` is a valuable law, but identity
functions satisfy it too. It cannot independently establish DEFLATE format
compatibility. Nor should a compressor normally promise one exact byte string:
many different block choices and matches represent the same content.

An independent relation `DeflateStream(bytes, content)` can define legal
encodings. The encoder must produce some bytes related to its input; the
decoder must return the related content for the promised input/resource domain.
Invalid-input handling and termination are additional clauses. The relation
may be decomposed into bits, block grammar, Huffman symbols, and history
updates, with separately checkable composition rules.

A useful small part is the back-reference operation. Let `h` be the produced
history, `m = length(h)`, and let the distance `d` and match length `n` be
mathematical natural numbers. For `1 <= d <= m`, define the appended sequence by:

```text
copy_suffix(h, d, n) =
    [ h[m - d + (k mod d)] for k in 0 .. n ]

append_match(h, d, n) = h ++ copy_suffix(h, d, n)
```

The range excludes `n`. The modulo divisor is nonzero, and each index lies
inside the final `d` elements of `h`. For `h = [a,b,c]`, `d = 2`, `n = 5`,
the appended bytes are `[b,c,b,c,b]`. This specifies overlap without encoding
a loop or copy instruction. It is consistent with the backward-distance
semantics in [RFC 1951, section 3.2.3](https://www.rfc-editor.org/rfc/rfc1951.html#section-3.2.3).[^2]

A byte loop, repeated-pattern fill, or vectorized routine can satisfy this
same contract. Proofs must connect the chosen loads and stores to the sequence
equation. A naive fixed-source slice of `n` old bytes is invalid when `n > d`;
the desired bytes partly originate from output generated during the copy.

This is a small semantic definition with substantial implementation freedom.
Its success route also needs destination capacity and preservation outside
the appended interval. A resource-limited route can instead describe an exact
committed prefix. DEFLATE's distance/length encoding limits belong to its
format layer; the generic repeated-suffix law need not bake them in.

### 4.3 Streaming JSON exposes temporal and relational requirements

A streaming parser has more observable behavior than a function from bytes to
a tree. A finite chunk can end before a token is decided. After receiving `1`
without EOF, the parser must allow the next chunk `2` to complete the number
`12`; irrevocably publishing a complete number `1` would need a different
protocol. An append-only input buffer does not justify append-only committed
tokens at every byte boundary.

Define a logical service state containing received bytes, unconsumed or pending
input, committed observations, and whether EOF has occurred. A transition
relation describes `feed(chunk)` and `finish()`. Concrete buffering, tokenizer
states, allocation, and tree construction can remain private.

For terminal results, a useful law is:

```text
For every finite byte sequence b and finite partition chunks of b:
    run feed over chunks, then finish;
    compare its terminal observation with one-shot processing of b.

The observations agree on:
    accepted semantic value or declared error category;
    absolute error/consumption position, if part of the API;
    committed output visible to the caller.
```

This law requires a fixed resource policy, successful delivery of the bytes,
and termination of the finite interaction. If event timing itself is public,
terminal equality is too weak: a relation between event traces is needed.
Conversely, requiring identical intermediate `NeedMoreInput` events across
different partitions would freeze an inappropriate observation.

One-shot and streaming execution can both be wrong in the same way. A parser
that always returns the same value satisfies partition invariance. An
independent JSON relation must therefore anchor accepted results; partition
invariance adds a different guarantee. The model must also choose policies for
duplicate object names, numerical representation, supported limits, and
errors. RFC 8259 leaves implementation latitude and documents interoperability
problems around some of these choices; it does not supply a unique application
object model. [RFC 8259, sections 4, 6, and 9](https://www.rfc-editor.org/rfc/rfc8259).[^3]

The expression burden is manageable if histories, partitions, grammar
relations, and tree models are reusable libraries. Requiring each application
to restate their mathematical foundations would make the language technically
expressive but practically unsuitable for review.

### 4.4 Rendering permits useful component contracts

A browser need not start with a proof from arbitrary HTML to exact GPU pixels.
A restricted layout component can accept a logical scene snapshot and promise
a geometry relation for its supported CSS fragment. That snapshot must include
all modeled dependencies, such as viewport, style, fonts and their metrics.
An optimized incremental implementation can then prove that its visible
geometry satisfies the same relation after each supported update.

Other components can have different promises: display-list order, clipping,
texture bounds, alpha-composition equations under a stated numeric model, or
preservation of unchanged tiles. A theorem connecting layout to painting is
required only for a cross-component claim that actually depends on that
connection. Separate component proofs do not silently establish that theorem.

There are two practical traps. First, saying "the cache equals a fresh run"
does not anchor correctness if both runs use the same wrong layout semantics.
Second, approximate equality is usually not transitive at a fixed tolerance:
two successive errors of at most epsilon may accumulate to twice epsilon.
An optimization pipeline needs error accounting or a shared reference model,
not repeated appeals to "close enough."

The Cassius research project makes the scope lesson concrete: it formalizes a
CSS 2.1 layout subset using constraints and Z3. It supports reasoning about
that modeled fragment; it is not a full browser specification or a direct
fit for WF's acceptance mechanism.
[Cassius project and model](https://github.com/uwplse/Cassius).[^4]

## 5. Established approaches and their limits

### Value specifications and modular interfaces

Dafny already offers mathematical sequences, sets, multisets, maps,
quantifiers, ghost definitions, state snapshots, frame specifications, and
termination measures. Its contract language is a useful reference for the
vocabulary of generalized tests. Its solver-driven verification workflow is
not Whitefoot's chosen acceptance architecture.
[Dafny Reference Manual](https://dafny.org/dafny/DafnyRef/DafnyRef).[^5]

Verus explicitly separates specification, proof, and executable code. This
separation fits the distinction between a statement, its evidence, and its
implementation. Some specification operations deliberately return unspecified
values outside their useful domain: the guide discusses division by zero and
out-of-range sequence indexing. This is a logical modeling convention, not an
executable memory-safety escape. WF should make a deliberate choice here;
requiring domain proofs or returning an option would make malformed
specifications harder to mistake for useful claims.
[Verus modes](https://verus-lang.github.io/verus/guide/modes.html)[^6]
and [ghost semantics](https://verus-lang.github.io/verus/guide/ghost_vs_exec.html).[^7]

Why3 provides a particularly relevant modularity example: a set interface can
expose mathematical contents while an implementation maintains a sorted list
and a representation invariant. Clients verify against the interface. Theory
cloning also supports reusable algebraic requirements; by default, cloned
axioms become obligations rather than silently remaining assumptions. These
are established mechanisms for giving reusable contracts an independently
checked implementation.
[WhyML reference, sections 6.5.6–6.5.7](https://why3.org/doc/syntaxref.html).[^8]

The implication for WF is architectural: a public contract should be reusable
across representations, while a private representation invariant is discharged
by each implementation. A person need not review the invariant's proof merely
because the implementation changes. The public meaning and the connection to
observable operations must remain fixed.

### Ownership-aware state models

There is no single established answer to translating borrow-based source
programs into mathematical state. Creusot represents a mutable borrow using
its current value, its eventual value, and identity information; resolving a
borrow connects the eventual value to the final contents. Its guide explains
why a simpler two-value account is insufficient for soundness.
[Creusot mutable borrows](https://guide.creusot.rs/representation_of_types/mutable_borrows.html).[^9]

Aeneas follows a different route: it translates a supported Rust fragment to
a functional representation for verification. Its documented language coverage
and backend limitations matter; it does not establish a translation theorem
for arbitrary WF programs. It does show why ownership information can reduce
the need for explicit heap reasoning in appropriate programs.
[Aeneas project](https://github.com/AeneasVerif/aeneas).[^10]

Pulse exposes separation-logic specifications, erased ghost computations, and
abstract ghost state for imperative programs. These provide another way to
describe resources and their evolution.
[Pulse ghost computations](https://fstar-lang.org/tutorial/book/pulse/pulse_ghost.html).[^11]
For WF, a content-image model aligned with existing unique ownership is the
smallest plausible starting point. Prophecy mechanisms and a general separation
logic should be compared when an actual borrow or concurrency example needs
them, rather than adopted because they are more general.

### Histories, progress, and relational properties

TLA+ describes systems using states, transitions, and temporal properties.
Its treatment of stuttering allows implementations to perform internal steps
without making every such step part of the public protocol. Fairness and
liveness add obligations beyond permitted state transitions.
[Lamport's advanced topics](https://lamport.azurewebsites.net/tla/advanced.html).[^12]
A TLA+ model alone does not prove that WF code implements that model.

Connecting realistic code to temporal models is active research with concrete
results, not an unexplored idea. Trillium develops trace-refinement reasoning
in separation logic, including fair liveness-preserving refinement and an
instantiation connecting distributed programs with TLA+ models. These results
require explicit semantics and scheduling assumptions; they do not make
liveness follow from ownership or from an invariant alone.
[Trillium, POPL 2024](https://iris-project.org/pdfs/2024-popl-trillium.pdf).[^13]

Relational properties have an additional subtlety. Ordinary refinement often
means removing permitted behaviors. This preserves single-trace requirements,
but not all requirements about sets of executions. Clarkson and Schneider's
example lets a system nondeterministically output either bit, independently
of a secret. Restricting it to always output the secret selects an originally
permitted behavior while destroying the independence property. Their analysis
identifies the subset-closed class for which this refinement principle works;
not every relational property has this problem.
[Hyperproperties, section 2.6](https://www.cs.cornell.edu/fbs/publications/Hyperproperties.pdf).[^14]

The WF consequence is narrow but important: "any implementation within the
allowed behaviors" must use the refinement relation appropriate to the declared
property. Optimizer correctness for public I/O does not automatically preserve
a timing or confidentiality contract that observes more.

### Evidence from substantial verified systems

EverParse3D is strong evidence for domain-specific specifications with practical
implementations. Its PLDI 2022 paper reports deployment of generated, proved C
parsers in the Windows kernel, covering nearly 100 message formats across four
protocols. The stated guarantees include functional and memory safety and
double-fetch freedom. This supports reusable format languages; it does not
show that one format DSL covers services, rendering, or all error policies.
[EverParse3D paper](https://www.microsoft.com/en-us/research/publication/hardening-attack-surfaces-with-formally-proven-binary-format-parsers/).[^15]

There is now a directly relevant agentic example. F*'s living tutorial reports
an agent-generated TLS 1.3 client/server for a restricted profile, with over
130,000 lines beneath a state-machine refinement interface. The model still
occupies thousands of lines. The reported result is functional correctness;
the chapter explicitly excludes cryptographic security proofs and production
readiness. It assumes properties of cryptographic primitives, links external
certificate validation, retains some unverified C setup, and uses OpenSSL
interoperability tests. These are authors' reported results, not an independent
reproduction. The evidence supports the proposed division of human semantic
review and machine implementation checking, without establishing a general
cost or scalability guarantee.
[F* TLS case study](https://fstar-lang.org/tutorial/book/agentic/agentic_tls.html).[^16]

The preceding F* chapter is especially useful for contract design: a reusable
state-machine implementation interface ties concrete event handling to a model
and tracks received/sent history. It separately addresses the need to preserve
historical commitments. A small interface is useful because its meaning was
established once, not because a short interface is automatically trustworthy.
[F* stateful-service interface](https://fstar-lang.org/tutorial/book/agentic/agentic_state_machines.html).[^17]

CompCert supplies established evidence that pass-level semantic-preservation
proofs can compose. Its notion of observation includes externally visible
effects and termination, while excluding time and memory consumption from
that particular guarantee. The theorem's domains matter more than a percentage
of "verified code."
[CompCert semantic preservation](https://compcert.org/motivations.html).[^18]
A 2022 analysis documents bugs outside CompCert's proved transformations,
including erroneous assembly printing. That is evidence for explicitly tracking
the trusted translation boundary, not evidence that the proved transformations
provide no guarantee.
[Monniaux and Boulme, trusted-computing-base analysis](https://arxiv.org/html/2201.10280v2).[^19]

Collectively, these examples establish viable mechanisms and useful scoped
outcomes. They do not establish that arbitrary contracts are easy to write,
easy to prove, or cheap to check. SMT-based tools are valuable expression and
engineering precedents even where their acceptance workflow is unsuitable for
Whitefoot.

## 6. Candidate shape for a WF contract language

The recommended prototype has three parts: a mathematical specification layer,
checked bindings to program behavior, and an explicit proof layer. The public
spelling can stay close to WF, but these roles should remain semantically
distinct. The following is a capability proposal, not grammar or implemented
coverage.

| Capability | Purpose | Initial restriction worth testing |
|---|---|---|
| Mathematical values | Integers, finite sequences, records, variants, trees, finite maps and multisets. | Separate mathematical integers from machine words; define byte/float interpretation explicitly. |
| Named propositions and quantification | State reusable laws over arbitrary values and positions. | Explicit arguments and quantifier instantiations; no guessed triggers in acceptance. |
| Total model definitions | Define tree traversal, byte interpretation, and reference operations. | Structural recursion first; checked well-founded definitions only with explicit evidence. |
| Inductively defined relations | Describe parsing and permitted transitions without a unique reference algorithm. | Strict positivity and specified induction rules; no circular declarations masquerading as proofs. |
| Abstract content and state views | Relate containers and module state to mathematical values. | Immutable snapshots and compiler-governed load/store/ownership bindings. |
| Result-specific contracts | Describe success, errors, consumed bytes, and partial publication. | Cover each declared route explicitly; distinguish absent promises from proved preservation. |
| Frame conditions | State what may change and what must remain equal. | Start with exclusive regions/subranges supported by the ownership model. |
| Model parameters and checked laws | Reuse specifications for collections, codecs, and protocols. | Explicit module instantiation; instances discharge laws, never introduce writer axioms. |
| Histories and simulation relations | Connect sequences of operations to an abstract service. | Finite histories and safety first; leave room for explicit progress and relational extensions. |
| Resource models | Express bounded work, storage, or proof-relevant accounting. | Logical units first; no claim that a counter is wall-clock time. |

Model definitions and mathematical values are erased. This does not mean that
ordinary effectful WF functions can be executed inside propositions. A call
such as `decode(input)` in a mathematical law must refer to a total model,
a relational description of execution, or a checked abstraction of a program
call. The language must make the choice visible.

Nor does current `pure` provide the required logical totality: the absence of
effects does not prohibit divergence. General recursion imported into logic
without a sound discipline can compromise reasoning. An inductive relation
can describe finite evaluations of a partial program without asserting that
an evaluation always exists; termination is then a separate theorem.

### The code-to-model bridge is part of the design

A proposed `contents` operator needs rules for the following cases:

1. A successful load reads the current content image at its checked index.
2. A store updates the selected element and preserves the proved frame.
3. Moving or replacing an owner transfers or replaces the appropriate image.
4. A call imports only the callee's proved state relation for that actual call.
5. A loop relates each current image to an immutable entry image through its
   invariant; old snapshots do not silently become descriptions of new storage.
6. Parallel tasks can use an image only under the ownership, disjointness, or
   synchronization rule that justifies its stability.

These are proposed proof obligations for operation schemas, not six magic
library axioms. They must be justified against WF semantics. A library can
then implement a ring buffer, chunked sequence, or tree using a private
representation invariant and expose a simpler abstract model to clients.
Otherwise, an agent could prove a ghost sequence is sorted while returning
unrelated real bytes.

### Compositional contracts need explicit discharge points

Sequential composition requires the first component's guarantee to establish
the next component's precondition, with compatible state ownership and
observations. Shared definitions must mean the same thing on both sides.
Internal assumptions must be discharged at calls or by initialization, not
silently reclassified as assumptions about the outside world.

Cycles require an induction principle, an invariant, or an appropriate
rely/guarantee argument. Two modules each assuming the other's eventual response
do not establish progress. Likewise, a proof that a transition preserves a
state invariant does not prove that any transition occurs.

The most useful initial public abstractions are likely a collection with
content laws, a codec with a format relation, and a stateful service with an
event model. Their implementation refinements can share underlying logic.
They need not share one overloaded `ensures` syntax that conceals their
different meanings.

## 7. Rich logic without solver-dependent acceptance

There is an established precedent for separating logical strength from proof
search. HOL Light uses a small kernel for classical higher-order logic; its
programmable proof procedures build results through primitive inference rules.
The kernel includes explicit equality reasoning rather than a demand to decide
every mathematical statement. This is a useful architectural precedent, not
evidence that running arbitrary HOL tactics during WF compilation would meet
WF's requirements.
[Harrison, HOL Light overview](https://www.cl.cam.ac.uk/~jrh13/papers/hollight.pdf).[^20]

Importing an entire modern dependent type theory would require more care than
"its kernel checks proofs." Lean's current reference explicitly documents rare
constructed cases in which its type checker does not terminate, while stating
that this does not undermine logical soundness. Even ordinary terminating
normalization can be costly. Lean is strong evidence for expressive proofs,
but its complete kernel behavior is not a ready-made fit for WF's stricter
termination policy.
[Lean kernel reference](https://lean-lang.org/doc/reference/latest/Elaboration-and-Compilation/#the-kernel).[^21]

Two foundations merit comparison:

| Foundation | Benefit | Main cost to test |
|---|---|---|
| Typed first-order logic, algebraic data, induction rules, explicitly parameterized modules | Direct fit for many sequence, parser, and state-machine laws; relatively simple formation and substitution. | Reusable predicates, callbacks, and richer resource abstractions may require awkward encodings or additional rules. |
| A small simply typed higher-order logic with explicit equality proofs and conservative definitions | Natural predicate/function parameters and reusable mathematical libraries without full dependent type conversion. | Binding, substitution, extensionality, induction libraries, and the source-program semantics still require careful design. |

The present examples justify a rich mathematical layer but do not yet choose
between these foundations. A full dependent runtime type system is not a
prerequisite for either. WF could retain its systems types and give proof-only
terms a separate, more expressive language.

A candidate acceptance path would:

- Retain the present principle of fixed, terminating automatic derivations
  within their specified families.
- Check additional written proof steps for conjunction, cases, explicit
  quantifier introduction/elimination, equality rewriting, induction, and
  instantiation of already proved lemmas.
- Require a particular definition and occurrence for unfolding, with a
  specified amount of reduction per step, instead of implicit normalization
  until a goal happens to close.
- Keep hypotheses explicit and discharge them before publishing an
  unconditional result. A writer-declared assumption cannot become an axiom.
- Preserve sharing of terms and lemmas, with an explicit cost account for
  substitution, binder traversal, arithmetic bit lengths, and context handling.

An untrusted authoring tool can search for proofs, propose invariants, or change
the implementation. It must ultimately emit source evidence the official
checker understands. No external solver verdict or unvalidated generated
certificate becomes acceptance authority. Under current WF rules, source is
the writer-controlled proof input; changing that boundary is a separate choice.

There is no checking-cost theorem for this proposal yet. Explicitly naming an
unfolding does not bound its result size, and retaining a DAG does not by itself
bound repeated traversal. The algorithm must identify a size measure tied to
written source and evidence, explain expansion costs, and survive adversarial
scaling tests. Those tests support the cost argument; they cannot replace it.
The existing [source-checking investigation](../proof-certificate-architecture/SOURCE-CHECKING.md)
already records this distinction for the narrower current calculus.

## 8. Reviewing and strengthening contracts

Contract review needs tools for finding missing requirements, even when proofs
are sound. The following countermodels distinguish frequently confused claims.
Each is a concrete logical counterexample, not a claim that an automated audit
will discover every weakness.

| Incomplete claim | Incorrect implementation still permitted | Additional commitment |
|---|---|---|
| Output is sorted. | Always return an empty sequence. | Preserve the input multiset; add stability if required. |
| Decode after encode returns the original input. | Use identity for both functions. | Conformance to an independently defined format relation. |
| All chunk partitions agree. | Always return the same result. | Accepted values and errors correspond to an independent semantic model. |
| If a parser returns success, its result is valid. | Always reject. | Accept the promised valid-input domain, with explicit resource conditions. |
| If a function returns, its output is correct. | Loop forever. | Termination or an appropriate service progress property. |
| An error returns an error code. | Overwrite the caller's buffer first. | Route-specific state relation and preservation of the promised frame. |
| The public theorem name is unchanged. | Weaken a transitive helper or strengthen its input assumption. | Review semantic dependencies and assumption changes. |
| Every visible transition is legal. | Stop taking transitions. | Progress under stated scheduling/environment conditions. |

For a concrete regression, a disciplined generalization has four steps. First,
identify the varying dimensions: input structure, lengths, contents, operation
history, errors, schedules, or environment. Second, state the property over
those dimensions, including observations and domain restrictions. Third,
exhibit an intended example and a plausible wrong implementation distinguished
by the property. Fourth, connect the contract to the actual code and discharge
the resulting proof obligations.

The original fixture remains useful as an intent witness. It can expose an
accidental change in number representation, error precedence, or output policy
that a terse law hides. Where a property has an executable fragment, the same
definition can support sampled tests and formal proofs, provided the sampling
route is labeled as sampling. A declaration must never gain proof status just
because a generated test harness passed.

Non-vacuity checks deserve explicit support. A requirement with an impossible
precondition is easy to satisfy. A witness shows that one case is possible,
but does not establish that the whole intended domain remains supported.
Useful derived obligations include constructor coverage, acceptance of the
declared valid-input language, and proofs that ordinary clients can establish
the requirements. These audits are themselves partial commitments, not a
complete decision procedure for specification quality.

Some tests remain valuable even with strong contracts:

- Interoperability checks whether independently produced specifications agree
  about real protocols and corner cases.
- Compiler and platform checks investigate the trusted implementation of the
  proof checker, lowering, runtime environment, and hardware model.
- Performance measurements connect logical work bounds to actual machines.
- Visual and product evaluation supplies judgments not yet captured by a
  chosen mathematical observation model.

This is not a binary choice between proof and testing. A contract can become
more precise after a new failure while old verified commitments remain useful.
What must stay binary is whether a particular universal claim has the required
proof under its declared assumptions.

## 9. Discriminating experiments

These are proposed experiments and criteria for future evidence. No score,
verification result, authoring time, or scaling measurement is claimed here.
The comparison should hold the public contract constant rather than tailor
the contract to the implementation that was easiest to prove.

### Expression and semantic review

Start with three linked examples: a malformed stored-header checker, the
overlapping back-reference operation, and a small stateful decoder assembled
from them. Restrict the decoder format explicitly; it need not be a complete
DEFLATE or JSON implementation. Include successful decoding, truncation,
output exhaustion, at least two block/record transitions, and chunk boundaries.

Compare three contract styles over the same requirements:

1. A pure reference function and equality of observations.
2. A declarative relation plus independent laws.
3. A domain-specific format/state-machine surface translated to those laws.

The decisive criterion is whether each style states the intended domains,
outputs, errors, frames, and history commitments without fixing buffer layout,
loop structure, or tokenizer representation. Record every imported definition
needed to understand the promise. Count and inspect missing policies, not just
lines of specification. A shorter statement with a hidden algorithm-sized
dependency is not an improvement.

Use the countermodels above as challenges to the specification. In particular,
require that the contract exclude constant results, unconditional rejection,
incorrect overlap copying, incorrect output before failure, and premature
publication across chunk boundaries. If it permits one, either strengthen the
contract or state that this behavior is deliberately allowed.

### Implementation freedom and composition

Supply two materially different implementations of the same content contract,
such as a byte loop and a repeated-pattern implementation. For the stateful
example, compare contiguous and segmented buffering where the declared public
resource policy permits both. Each should have a distinct private invariant
but the same public meaning.

The composition criterion is that callers use only proved component contracts,
not callee bodies or hidden assumptions. A mismatch in error priority, consumed
position, or output publication must create a proof obligation that cannot be
silently bypassed. Termination of the finite driver should be proved separately
from correctness of its completed transitions.

This phase distinguishes an expression sketch from a verification mechanism.
Before calling it successful, a checker must validate the operation-to-model
bindings and the written derivations for the actual WF code. Manually erasing
annotations and passing runtime tests is insufficient.

### Deterministic checking and maintenance

Test nested model definitions, repeated substitutions, growing induction
proofs, and long chains of reused lemmas. Measure total checking time, memory,
expanded term size, and repeated visits to shared terms against source and
proof size. Record the algorithmic size argument before using a benchmark to
choose a representation. No timeout or measurement threshold may decide
language acceptance.

Then change an implementation without changing its contract, and change a
contract helper without changing its top-level name. The first should require
updated implementation evidence while preserving caller reasoning. The second
should expose the affected semantic dependents. The experiment need not build
a new review infrastructure; an explicit dependency report is enough to test
whether the distinction can be made.

## 10. Recommendation and unresolved decisions

Proceed with the specification-layer investigation. Existing work supports
mathematical models, modular refinement, reusable interfaces, and generated
implementations. A restricted automatic proof language should not also be the
ceiling on what a person can commit to. The most promising WF contribution is
their combination with source-bound, deterministic proof checking and the
existing systems-language ownership rules.

The next language experiment should prioritize content snapshots, named
relations, quantified laws, explicit induction, result-specific frames, and
finite service histories. Select a narrow decoder scenario broad enough to
exercise all of them. Preserve the current prohibition on accepting unresolved
semantic obligations through runtime fallbacks or writer escape hatches.

Three decisions remain open. First, choose a logical foundation only after
comparing library reuse and checking costs. Second, determine how far existing
ownership and value-image machinery can support content relations before a
richer heap/resource logic is necessary. Third, decide which history and
relational judgments belong in an initial prototype, without promising
concurrency liveness, cryptographic security, or whole-browser rendering from
single-call contracts.

Success would be evidence that useful test intentions can be reviewed once as
stable semantic commitments, then preserved across substantially different WF
implementations. It would not mean that arbitrary human intent has been
formalized, that every contract has a tractable proof, or that testing no longer
serves a purpose.

## Sources

Numbered notes identify primary sources and their scope. Undated documentation
is a living source accessed on 2026-09-10 UTC, not a pinned toolchain result.
Repository links identify source witnesses at the baseline stated above.

[^1]: QuickCheck contributors. [QuickCheck package documentation](https://hackage.haskell.org/package/QuickCheck). Living documentation. Property definitions and randomized testing.
[^2]: L. Peter Deutsch. [RFC 1951: DEFLATE Compressed Data Format Specification version 1.3](https://www.rfc-editor.org/rfc/rfc1951.html), May 1996. Sections 3.2.3–3.2.4: backward references and stored blocks.
[^3]: Tim Bray, editor. [RFC 8259: The JavaScript Object Notation Data Interchange Format](https://www.rfc-editor.org/rfc/rfc8259), December 2017. Object names, numbers, and implementation limits.
[^4]: University of Washington PLSE contributors. [Cassius](https://github.com/uwplse/Cassius). Project documentation. CSS layout fragment, constraints, and tool scope.
[^5]: Dafny contributors. [Dafny Reference Manual](https://dafny.org/dafny/DafnyRef/DafnyRef). Living documentation. Mathematical collections, ghost definitions, framing, and termination specifications.
[^6]: Verus contributors. [Specification code, proof code, executable code](https://verus-lang.github.io/verus/guide/modes.html). Living tutorial. Separation of the three semantic roles.
[^7]: Verus contributors. [Ghost code vs. exec code](https://verus-lang.github.io/verus/guide/ghost_vs_exec.html). Living tutorial. Erasure boundaries and unspecified out-of-domain logical values.
[^8]: Why3 contributors. [The WhyML Language Reference](https://why3.org/doc/syntaxref.html), documentation labeled 1.8.2. Sections 6.5.6–6.5.7: cloning, interfaces, and representation invariants.
[^9]: Creusot contributors. [Mutable borrows](https://guide.creusot.rs/representation_of_types/mutable_borrows.html). Living guide. Current/final-value and identity treatment of mutable borrows.
[^10]: Aeneas contributors. [Aeneas: a verification toolchain for Rust programs](https://github.com/AeneasVerif/aeneas). Living project documentation. Functional translation and supported scope.
[^11]: F* and Pulse contributors. [Ghost Computations](https://fstar-lang.org/tutorial/book/pulse/pulse_ghost.html). Living book. Erased computations and ghost resource state.
[^12]: Leslie Lamport. [Advanced Topics](https://lamport.azurewebsites.net/tla/advanced.html). Living TLA+ notes. Stuttering, safety, liveness, and fairness.
[^13]: Amin Timany et al. [Trillium: Higher-Order Concurrent and Distributed Separation Logic for Intensional Refinement](https://iris-project.org/pdfs/2024-popl-trillium.pdf), POPL 2024. Trace refinement and fair liveness reasoning.
[^14]: Michael R. Clarkson and Fred B. Schneider. [Hyperproperties](https://www.cs.cornell.edu/fbs/publications/Hyperproperties.pdf), Journal of Computer Security 18, 2010. Section 2.6: refinement and subset closure.
[^15]: Nikhil Swamy et al. [Hardening Attack Surfaces with Formally Proven Binary Format Parsers](https://www.microsoft.com/en-us/research/publication/hardening-attack-surfaces-with-formally-proven-binary-format-parsers/), PLDI, June 2022. EverParse3D guarantees and reported Windows deployment.
[^16]: F* contributors. [A Verified TLS-1.3 Client and Server](https://fstar-lang.org/tutorial/book/agentic/agentic_tls.html). Living book. Authors' reported agentic case study, assumptions, and explicit exclusions.
[^17]: F* contributors. [Stateful Services](https://fstar-lang.org/tutorial/book/agentic/agentic_state_machines.html). Living book. Reusable protocol refinement and historical commitments.
[^18]: CompCert contributors. [Context and motivations](https://compcert.org/motivations.html). Project explanation of semantic preservation and observations. Historical pass counts and coverage percentages on this page are not used as current measurements.
[^19]: David Monniaux and Sylvain Boulme. [The Trusted Computing Base of the CompCert Verified Compiler](https://arxiv.org/html/2201.10280v2), 2022. Trust boundaries and defects outside verified transformations.
[^20]: John Harrison. [HOL Light: An Overview](https://www.cl.cam.ac.uk/~jrh13/papers/hollight.pdf), TPHOLs 2009. Simply typed higher-order logic and primitive inference kernel.
[^21]: Lean contributors. [Elaboration and Compilation: The Kernel](https://lean-lang.org/doc/reference/latest/Elaboration-and-Compilation/#the-kernel). Living reference. Kernel architecture and documented termination caveat.
