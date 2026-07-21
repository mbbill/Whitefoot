# Compiler Architecture Preflight

Status: debate input, not an architecture decision or execution authority.

This document exists because implementation must not silently choose the
compiler architecture. `THE-PLAN.md` remains the sole roadmap and
authorization source. The purpose here is to make the questions that the next
roadmap revision must answer explicit enough for independent and hostile
review.

## Why this must happen before more semantic implementation

A compiler can pass many tests while its architecture is becoming harder to
make correct. The dangerous choices are often made indirectly:

- a convenient parser representation becomes the permanent source model;
- an internal consistency assertion is mistaken for evidence that the grammar
  was implemented correctly;
- a verifier becomes a second checker without anyone deciding whether that
  duplication is justified;
- an identifier chosen for one in-memory pass becomes a serialized authority;
- declared effects, recursion, and call-graph closure acquire accidental
  semantics from traversal order;
- optional optimizer facts become mixed with facts required for correct
  lowering; or
- an LLVM optimization hint is emitted before its exact proposition and
  invalidation rules are understood.

These mistakes are expensive because later stages depend on the earlier
contracts. Tests added after the interfaces have hardened may confirm the
chosen design without answering whether it was the right design.

The next compiler design must therefore be written down before the parser,
canonical tree, checked artifact, proof language, or semantic checker freezes
their public shapes. For every stage, the design must explain:

1. what problem the stage solves;
2. its exact input and output contract;
3. the invariants it establishes;
4. the work it intentionally does not perform;
5. why the work belongs there and not in another stage;
6. which component must trust the result;
7. what could make the result false;
8. how independent evidence will detect those failures; and
9. why the selected design is better than the rejected alternatives.

A diagram or list of stage names is not enough. A stage without this reasoning
is an implementation placeholder, not an architecture.

## What the recent design discussion exposed

The discussion around the permanent-architecture diagram found several places
where short labels hid materially different problems.

| Topic | Correct distinction or current status |
|---|---|
| Required AST pieces | Strong Rust node types and fallible constructors should make missing required fields impossible. Rechecking the same schema afterward does not prove that the schema matches the specification. |
| One production per node | A node can record the production that constructed it. That does not prove that the input has only one possible derivation. Grammar unambiguity needs independent grammar analysis and an oracle that can count derivations. |
| Child spans do not overlap | Parent and child spans necessarily overlap. The meaningful requirements concern parent containment, ordered non-overlapping siblings, and exact token-leaf coverage. |
| No skipped tokens | This is a parser construction invariant and a parser-bug target. It should be enforced by the consumption API, root finalization, tree-based rendering, and hostile tests, not treated as proof that the grammar is correct. |
| Validated AST | Internal tree construction, grammar-conformance testing, and validation of an untrusted serialized artifact are three different jobs. The current label mixes them. |
| Alternative parses | An AST walk cannot discover a second derivation without another grammar implementation. The production parser needs static lookahead analysis plus an independent bounded grammar oracle and hostile generation. |
| Region lifetime | Exact v0.8 uses explicit lexical regions. A borrow lives until the end of its named region, not until its last use. Last-use lifetime inference must not enter the v0.8 checker. |
| Effect calculation | Effects are not simply computed bottom-up over a tree. Signatures are collected first, calls use declared and region-substituted rows, bodies are checked against them, and recursion plus SCC rules require an explicit call-graph design. |
| Checker versus verifier | No semantic check should be postponed to the verifier. The checker must fully check the program. A separate verifier is justified only if it validates explicit certificates through a materially smaller and simpler trusted process. |
| Facts off versus facts on | Core semantic information is never disabled. An empty optional optimization overlay is a correctness control. Once fact families are verified, production compilation should consume them; the empty-overlay path remains a differential and containment tool. |

The lexical-region question is resolved by the v0.8 specification. The other
rows are architecture requirements or open decisions that the revised plan
must address precisely.

## Fixed constraints for the design debate

The debate does not reopen these constraints without a separately approved
change to project authority:

- The exact first target is `spec/kernel-spec-v0.8.md` with its pinned digest.
- The numbered specification is not reinterpreted or edited to fit the
  implementation.
- A specification conflict stops affected implementation and is recorded for
  an owner-gated successor specification.
- The production compiler is safe Rust; production semantic code contains no
  `unsafe`.
- Conformance expectations remain compiler-independent.
- No source, function, corpus, facet identifier, or project-specific dispatch
  may select compiler behavior.
- Every unproved required safety check remains in generated code.
- Explicit checks are not removed.
- Optimizer authority comes only from an exact machine-verified proposition.
- Archived implementations are evidence only and cannot supply active
  semantics or architecture.
- v0.8 regions and borrow liveness are explicit and lexical.
- The compiler must be deterministic, resource-bounded, and failure-atomic at
  publication boundaries.

The debate may challenge an existing plan decision, including the separate
checked-artifact verifier, but it must say explicitly that it is proposing a
roadmap change and must compare the complete alternatives.

## Required design decisions

### 1. Name every representation and its authority

The revised plan must distinguish at least these possible representations:

- original source bytes and source bundle;
- lossless lexical token and trivia tape;
- grammar derivation result;
- canonical or core tree;
- declaration and resolution tables;
- semantic checking state;
- candidate checked artifact;
- verified checked artifact;
- optional verified optimization overlay;
- lowering representation, if one exists between the checked artifact and
  LLVM; and
- LLVM module text and runtime metadata.

For each representation, decide:

- whether it is internal, serializable, or authoritative;
- whether it is lossless or normalized;
- which identities are stable and what bytes they bind;
- whether another representation can be reconstructed from it;
- whether it may contain recovery, error, or placeholder nodes;
- who is permitted to construct it; and
- who must validate it before consumption.

The terms parse tree, derivation tree, concrete syntax tree, AST, canonical
tree, core tree, checked unit, and proof artifact must not be used
interchangeably.

### 2. Resolve the exact-v0.8 grammar boundary before parser design

The current exact-v0.8 terminal-versus-`IDENT` ambiguity is already recorded.
The design review must compare at least:

- stopping until a successor specification defines the terminal partition;
- producing an ambiguity-preserving grammar forest and delaying selection;
  and
- any other claimed exact-v0.8 interpretation, with its normative authority.

Ordered-choice precedence and an invented keyword list are not available
answers.

If an ambiguity-preserving result is considered, the design must explain:

- its worst-case time and memory bounds;
- how ambiguity is represented without exponential materialization;
- which later stage is allowed to select a derivation;
- what normative rule permits that selection;
- whether name resolution or types would become part of parsing;
- how diagnostics identify a location without a canonical node path;
- how source/tree identity is defined before selection; and
- how hostile inputs prevent resource exhaustion.

No parser implementation should begin until this decision has an owner-reviewed
answer.

### 3. Separate parser construction from grammar evidence

The production parser design must state how successful construction makes its
local invariants difficult or impossible to violate. Examples include typed
node variants, required fields, nonempty-list types, source-bound token
handles, monotonic token consumption, and exact root finalization.

This is different from proving that the implemented grammar matches v0.8. The
evidence design must include:

- a mechanical `FIRST`/`FOLLOW` analysis at the lookahead depth claimed by the
  specification;
- an independently implemented bounded grammar oracle that reports zero, one,
  or multiple complete derivations;
- authored cases for every production and every shared prefix;
- generated and fuzzed cases around nullable lists, delimiters, fixed
  terminals, identifiers, type arguments, calls, constructions, places, and
  effect rows;
- differential comparison between the production parser and the oracle; and
- explicit bounds stating what the bounded oracle does and does not prove.

The revised plan must say which results are construction guarantees and which
are independent evidence. It must not use one as a substitute for the other.

### 4. Decide whether a separate structural-validation pass exists

The phrase `validated AST` is too vague. The design must choose among, or
justify a combination of:

- construction-correct internal trees whose Rust types encode required
  structure;
- a cheap internal invariant audit used for defense and debugging;
- a source-to-tree conformance oracle used only in testing; and
- full structural validation when decoding an untrusted serialized artifact.

For every proposed post-parse check, the plan must provide a concrete failure
example that can survive the earlier constructors. If no such example exists,
the check should not become a separate production phase.

The exact source-coverage contract must define:

- containment of child ranges by parents;
- ordering and non-overlap of sibling ranges;
- exact ownership of token leaves;
- treatment of trivia and punctuation;
- prevention of cross-source token references;
- complete root consumption; and
- whether rendering traverses tree-owned leaves or merely consults the
  original token tape.

A renderer that reproduces bytes directly from the original tape does not
prove that the tree corresponds to those bytes.

### 5. Design identities only after the real representations exist

Runtime handles and artifact identities solve different problems. The plan
must define, for source, token, node, declaration, scope, region, type,
function, instantiation, check, proof, and artifact identities:

- the identity domain;
- the bytes or structure being identified;
- equality and ordering rules;
- whether the identity survives serialization or rebuilding;
- collision handling;
- determinism across machines and process order;
- whether identity depends on traversal order; and
- how references are validated against the containing artifact.

Dense indices may be good internal handles without being valid artifact
authority. Stable identities must not be frozen merely because a later schema
is expected to need them.

### 6. Specify declaration collection and name resolution

The design must state:

- when top-level and local declarations are collected;
- the scope tree and separate name spaces;
- declaration-before-use and no-shadowing enforcement;
- how globally unique variant constructors are represented;
- how labels and regions are resolved;
- how uses connect to declarations;
- how malformed or duplicate declarations affect later work; and
- whether any body is checked before the declaration environment is complete.

If the checked artifact records `use U resolves to declaration D`, the checker
must already have validated that result. Any verifier certificate must carry
enough explicit scope evidence to validate visibility, ordering, namespace,
and uniqueness without performing open-ended name search.

### 7. Specify types, constants, operations, and substitutions

The plan must define the canonical representation and checking order for:

- nominal and primitive types;
- modes separate from types;
- region and constant arguments;
- explicit generic substitutions;
- constants and monomorphization-time evaluation;
- operation-table lookup without overloading;
- exact numeric domains and literal validation;
- user calls versus operation calls; and
- instantiated types and functions.

It must explain which values are interned, which are structural, how recursive
nominal declarations are handled, and how cycles are detected. Host-language
integer conversion, overflow, map ordering, or trait behavior must not become
Whitefoot semantics.

### 8. Specify regions, loans, moves, and control-flow state

The parser only records written region declarations and uses. Resolution should
connect each region use to a declaration identity. The semantic design must
then define at least:

- the lexical region tree and caller-supplied region parameters;
- the outlives-or-equals relation;
- binding storage lifetime versus borrow lifetime;
- loan identity, holder identity, and resolved-place identity;
- shared and unique loan state;
- parent suspension and statement-scoped child reborrows;
- moves, binding death, joins, loops, match arms, and unreachable paths;
- region exit on fallthrough, return, break, and trap;
- derived drops and arena releases on every relevant edge; and
- region substitution at calls.

The plan must say whether this is implemented as a structured syntax walk, an
explicit control-flow graph, or another representation. It must demonstrate
that every v0.8 control-flow edge is represented exactly once and that loop
back edges cannot reuse invalid state.

Last-use or non-lexical borrow shortening is outside exact v0.8 semantics and
must not enter as an implementation convenience.

### 9. Specify effects over recursion and call-graph closure

The current spec states declared function effect rows and a syntactic exhibits
relation. The plan must describe the exact algorithm:

- collect all signatures before body checking;
- resolve calls;
- substitute actual regions into callee rows;
- derive local exhibits from syntax and the operation table;
- account for user calls using the rule required by v0.8;
- compare declared and exhibited rows in both directions; and
- analyze SCCs for the separate polymorphic-recursion rule.

The following corner must be preserved as an explicit design and specification
question:

> `f` declares `traps` because it calls `g`; `g` declares `traps` because it
> calls `f`; neither SCC contains a primitive trap, check, bounds-checked
> index, or call outside the SCC that grounds `traps`.

Review must determine whether exact v0.8 intentionally allows this circular
syntactic exhibition or whether a grounded or least-fixed-point rule is
missing. The compiler may not silently choose. Additive evidence should cover:

- direct self-recursion with an otherwise ungrounded `traps` row;
- mutual recursion with ungrounded `traps` rows;
- the corresponding `pure` cycles;
- a cycle with one real trapping operation;
- memory effects propagated around cycles; and
- region substitution through recursive calls.

The already-recorded effect-row canonicality and body-local-region effect gaps
remain separate blockers and must not be hidden by this question.

### 10. Prove or reject the separate checked-artifact verifier

The current roadmap selects a checker that constructs proofs and an independent
verifier that validates them. The revised design must not treat that choice as
self-justifying.

The debate must compare at least:

- one checker in the trusted computing base, with no separate semantic
  verifier;
- a complete checker plus a smaller certificate verifier; and
- two substantially independent semantic checkers.

For the certificate-verifier option, define:

- the threat model: corrupted cache, hand-authored artifact, alternative
  producer, memory corruption outside safe Rust, or another stated source;
- which producer outputs are untrusted;
- the closed proof language;
- the proof object for each semantic family;
- how local proof steps compose into whole-unit closure;
- how source, tree, declarations, and proofs are bound together;
- what the verifier checks directly;
- what remains trusted;
- why verification is materially simpler than construction; and
- the maintenance cost of changing checker and verifier together.

No semantic obligation belongs only in the verifier. The checker must reject a
bad program even if the verifier is not run. The verifier is a second trust
gate, not delayed checking.

The design must reject the verifier architecture if its implementation needs
raw-source parsing, name lookup to discover targets, type inference, ownership
proof search, optimizer reasoning, or a duplicate diagnostic system. Merely
placing similar code in another crate is not independence or simplification.

The dependency boundary should make accidental lowering of unverified input a
Rust type error, but that ergonomic property alone does not justify the
verifier's existence.

### 11. Define the checked artifact and proof language before coding either

If a separate verifier survives review, checker and verifier must be designed
against the same explicit artifact contract before either implementation grows
large. The contract must cover:

- canonical tree and source binding;
- complete declaration and resolution tables;
- types, modes, regions, places, and substitutions;
- control-flow and ownership state transitions;
- effects and call-graph closure;
- checks retained and checks eligible for proof-based elimination;
- drops, releases, and failure edges;
- generics and concrete instantiations;
- diagnostics and report references;
- proof-schema versioning and resource ceilings; and
- rejection of unknown, duplicate, missing, cyclic, or unreachable records.

The proof language should make verification local and linear where possible.
It must not simply serialize every intermediate checker data structure and call
that a proof.

### 12. Decide the semantic-to-lowering boundary

The design must say whether `VerifiedCheckedUnit` is directly suitable for
lowering or whether a verified, explicit intermediate representation is
needed. It must account for:

- structured control flow and joins;
- monomorphized functions and layouts;
- explicit drops and releases;
- traps and failure edges;
- ABI decisions;
- bounds and arithmetic checks;
- aggregate construction and movement;
- report attribution; and
- deterministic LLVM naming and publication.

If another IR exists, define whether it is verified, derived, or trusted and
why it does not create an unchecked semantic gap. If there is no additional
IR, show that the checked artifact gives the lowerer every decision it needs
without asking it to redo semantic checking.

### 13. Separate required semantic facts from optional optimizer facts

The revised terminology should distinguish:

- information always required for correct lowering, such as resolved
  declarations, types, layouts, control-flow meaning, region substitutions,
  effects, drops, and mandatory checks; and
- optional propositions that authorize check removal or stronger LLVM facts,
  such as a proven bound, disjointness, no overflow, or a checked algebraic
  law.

The plan must explain why the empty-optimization-overlay path exists. Its
purposes may include correctness isolation, differential testing, per-family
measurement, and containment of a fact or backend bug. It is not the desired
final performance mode.

Once an optional fact family is accepted, normal production compilation should
consume its verified facts through the same lowerer. The plan must specify:

- exact proposition and scope;
- producer and proof object;
- verifier and trusted assumptions;
- consumers;
- invalidation and transfer rules;
- LLVM attributes, metadata, or control-flow changes it authorizes;
- facts-on versus empty-overlay semantic identity;
- hostile near misses and mutants; and
- the behavior when proof production or verification fails.

The terms `facts-off` and `facts-on` should be replaced or precisely defined so
they cannot be mistaken for disabling core semantic information.

### 14. Define diagnostic authority at every failure stage

The design must separate:

- lexical observations;
- parse failures before a canonical tree exists;
- source-form rejection;
- semantic rule rejection;
- artifact-verifier failure;
- resource failure;
- compiler invariant failure;
- backend and toolchain failure; and
- runtime traps.

For each, define its location form, rule attribution, determinism contract,
serialization status, and whether it is normative. The known v0.8 requirement
for a canonical node path on pre-tree failures remains a specification gap; an
implementation must not invent a normative path.

Diagnostics must be produced by the stage that understands the failure. A
verifier should not reproduce frontend diagnostics, and a backend failure must
not become a language rejection.

### 15. Define resource, determinism, and failure-atomicity contracts

Every stage must have explicit limits for input size, nesting, collection
growth, graph traversal, recursion, proof size, and output size. The design
must state:

- which limits are user-selectable or implementation-fixed;
- inclusive boundary behavior;
- checked arithmetic used to compute capacities and sizes;
- how allocation failure is classified;
- publication points and rollback behavior;
- deterministic traversal and ordering;
- cycle detection and worklist bounds; and
- how hostile artifacts avoid quadratic or exponential work.

Resource limits must not change a normative rejection into `Unsupported` or
permit partial publication.

### 16. Design the independent evidence system with the architecture

Tests should not be appended only after implementation. Each architectural
decision must name its evidence before code is written. The complete plan
should combine:

- specification-derived positive and negative conformance cases;
- parser grammar analysis and an independent derivation oracle;
- source/tree rendering and binding tests;
- independent models for ownership, effects, drops, numerics, and call graphs;
- checker/verifier agreement and verifier mutation tests;
- malformed and adversarial artifact corpora;
- property, metamorphic, and coverage-guided source fuzzing;
- facts-on versus empty-overlay differentials;
- runtime and ABI differentials against independent oracles;
- LLVM code-shape and poison/undefined-behavior review;
- deterministic rebuilds and resource-limit tests; and
- complete dogfood compatibility profiles.

For every independent component, state what makes it independent: different
algorithm, different implementation, different authority, hand-authored
oracle, or another exact reason. A second program that shares the same flawed
rule table is not automatically independent.

### 17. Design Rust responsibility and dependency boundaries

Crates and modules should follow invariant-bearing responsibilities, not a
fixed file or crate count. The plan must nevertheless define allowed dependency
directions so that:

- source contracts do not depend on parser or semantic code;
- parser code cannot issue semantic verdicts;
- the checker cannot depend on lowering;
- the verifier cannot call checker search or inference implementations;
- lowering cannot accept raw or merely candidate artifacts;
- optional fact metadata cannot influence source acceptance; and
- compiler-independent tests and catalogs remain outside production semantic
  dispatch.

The review should identify where shared data types are legitimate and where
sharing algorithms would destroy claimed independence. Files should be split
by cohesive invariant, with no arbitrary size target, forwarding-only layers,
duplicate generic walkers, or broad utility drawers.

### 18. Audit the current Rust foundation against the selected design

After the architecture is approved and before implementation resumes, perform
a foundation audit. Classify every existing permanent compiler component as:

- keep unchanged because it is independent of disputed architecture;
- keep but narrow or rename because its claimed responsibility is too broad;
- refactor before dependents grow;
- delete because it freezes an unsupported design; or
- defer because its contract depends on an unresolved representation.

Do not preserve a component merely because it is tested or already committed.
Do not discard the lossless lexer, source contract, toolchain gates, or other
work merely because later design was previously unclear. The classification
must follow explicit dependencies and invariants, not sunk cost or distrust by
association.

The audit must pay special attention to names and APIs that imply more
authority than they actually provide, including `validated`, `verified`,
`canonical`, `complete`, `artifact`, and `proof`.

## Required form of each final architecture decision

Every decision entering the revised roadmap should use this record:

```text
Decision:
Problem being solved:
Specification and project constraints:
Selected design:
Input contract:
Output contract:
Established invariants:
Explicit non-responsibilities:
Why this stage owns the work:
Why adjacent stages must not own it:
Alternatives considered:
Why each alternative was rejected:
Trusted assumptions and threat model:
Failure modes:
Independent evidence required:
Resource and determinism bounds:
Dependencies on unresolved specification questions:
Migration or foundation-audit consequences:
Owner approval required:
```

An answer such as “for safety,” “for clean architecture,” or “to catch bugs” is
not sufficient. It must identify the concrete failure, the consumer that would
otherwise trust a false result, and why the selected boundary improves the
situation.

## Suggested hostile-review roles

The architecture debate should assign reviewers distinct jobs rather than ask
all reviewers for a general opinion:

- A specification reviewer checks every claimed behavior against exact v0.8
  and marks every place where the design would invent semantics.
- A parser reviewer attacks ambiguity, lookahead, losslessness, recovery,
  source coverage, grammar-oracle independence, and resource bounds.
- A semantic-soundness reviewer attacks types, ownership, regions, effects,
  recursion, drops, calls, and joins.
- A verifier skeptic argues that the separate verifier should be removed and
  requires the selected design to prove its value and simplification.
- A proof-verifier advocate presents the strongest concrete certificate design
  and its threat model rather than defending the word `verifier` in the
  abstract.
- A backend reviewer attacks every path from a false fact to LLVM poison,
  undefined behavior, a removed trap, wrong cleanup, or ABI mismatch.
- A Rust architecture reviewer attacks dependency cycles, authority-bearing
  shared code, accidental public APIs, allocation behavior, and
  maintainability.
- A test reviewer asks how each failure could survive the main implementation
  and whether the proposed oracle is genuinely independent.
- A hostile integrator searches for contradictions between individually sound
  stage designs, especially at parser/resolver, type/ownership,
  checker/verifier, verifier/lowerer, and fact/LLVM seams.

Each reviewer should return `GO`, `NO-GO`, or `GO WITH BLOCKERS`, with exact
blocking questions and counterexamples. The final synthesis must preserve
dissent and rejected alternatives instead of reducing review to a vote.

## Exit criteria for the design-preflight phase

Semantic compiler implementation should resume only when:

1. every required design decision above has a concrete answer or an explicit
   blocking dependency;
2. the grammar boundary has an owner-approved route;
3. checker and verifier responsibilities have been justified separately, or
   the verifier has been removed from the proposed architecture;
4. the recursive-effect corner has a specification-grounded disposition;
5. the checked-artifact and lowering boundaries are concrete enough to expose
   all semantic decisions and failure edges;
6. the empty optimization overlay and normal verified-fact path have precise
   purposes;
7. independent evidence and resource bounds are designed with each stage;
8. hostile reviewers have attacked the seams and all `NO-GO` findings are
   resolved or recorded as blockers;
9. the current Rust foundation has a keep/refactor/delete/defer audit; and
10. the owner approves the resulting roadmap revision before implementation
    treats it as authority.

Until then, more code may produce evidence or inspect existing behavior, but it
must not freeze disputed parser, tree, proof, artifact, verifier, or lowering
contracts.
