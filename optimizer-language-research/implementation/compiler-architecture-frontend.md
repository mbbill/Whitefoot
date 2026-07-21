# Production Compiler Architecture — Frontend and Source

Status: OWNER-APPROVED ARCHITECTURE WITH BLOCKERS. This file is part of the
single [Production Compiler Architecture](compiler-architecture-design.md)
design record. `THE-PLAN.md` is execution authority. This file does not amend a
numbered specification, protected semantic surface, profile, or entrance gate.

This file uniquely owns Decisions 2 through 6 and the proposed successor-specification compilation-unit rule. Cross-stage authority, entrance gates,
execution order, owner decisions, and exit status remain in the parent index.

## Decision records

### Decision 2: exact grammar boundary

**Decision:** Stop the production parser until a successor numbered
specification makes fixed terminals and `IDENT` disjoint. After that, classify
terminals in a specification-versioned grammar layer and use an iterative,
typed LL(2) parser with no backtracking, ordered-choice priority, recovery,
synthetic tokens, or semantic disambiguation.

**Problem being solved:** Under exact v0.8, `deref(p)`, `index<T>(p, q)`,
primitive type arguments, and `unit` have multiple complete derivations.

**Specification and project constraints:** GRAM-1 claims one parse and at most
two-token lookahead. META-2 forbids context-dependent spelling. No existing
test or retired implementation may invent terminal precedence.

**Selected design:** Stop for the successor grammar decision, then use the
version-bound classifier and iterative typed LL(2) parser described above.

**Grammar-change verification tool:** Grammar evolution is checked by one
separately runnable verification entry point outside the production compiler.
It accepts the exact full bytes of the current specification and the exact full
bytes of a non-authoritative proposal. The proposal bytes are a review input
outside the numbered `spec/` surface; the verifier never writes a numbered
specification. They are evidence for review, not a language verdict or
permission to modify a numbered specification or protected expectation.

For the transition governed by this dossier, the verifier rejects unless the
SHA-256 of the current input is the pinned v0.8 identity
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.
It records the SHA-256 of the complete proposal bytes. If the owner approves
that proposal, the new numbered specification must be byte-for-byte identical
to the reviewed proposal bytes, and therefore have the recorded hash, before
this grammar gate can pass. Any proposal edit requires a fresh report and fresh
owner review.

The entry point orchestrates two deliberately independent engines:

1. A static grammar auditor extracts and source-binds every normative grammar
   production, lexical-class definition, and fixed terminal from each exact
   byte input. It fails on unclassified grammar-shaped normative content,
   computes nullable, `FIRST_2`, `FOLLOW_2`, and the exact strong-LL(2)
   predictive relation, and reports every terminal/lexical-class intersection
   and every overlapping alternative with a concrete lookahead witness.
2. A bounded generalized-parser oracle independently extracts the grammar and
   token-membership rules from the same exact byte inputs into its own
   representation. For each explicitly bound entry nonterminal and token
   stream, it returns `zero`, `one`, or `many` complete derivations. It does
   not import the static auditor's extraction, predictive relation, or any
   production-parser table.

The two engines share only the exact input bytes and declared resource profile,
not an extracted grammar table. The report binds both input hashes, each
engine's independently extracted source coverage, grammar and terminal
inventories, both engine revisions, resource profile, entry nonterminals,
registered witnesses, conflicts, and surviving derivations. Any extraction or
source-coverage disagreement is a failure. Output ordering is canonical.
Exhausting an extraction, token, chart, alternative, or work bound is an
explicit inconclusive resource outcome, never evidence that no conflict
exists.

The static claim is precise. A lookahead atom is a predicate over an exact
shape-token kind and spelling, not a priority-selected classification. Two
atoms overlap when at least one concrete shape token satisfies both; for
example, under exact v0.8 `LowerWordForm("deref")` satisfies both fixed
`"deref"` and `IDENT`. For every alternative `A -> alpha`, the auditor computes
`SELECT_2(A -> alpha) = FIRST_2(alpha FOLLOW_2(A))` using concatenation
truncated to two lookahead atoms and intersection by concrete satisfiability.
It expands every legal lexical membership, uses an explicit `SOURCE_END`
analysis sentinel for a complete source grammar entry, and uses a distinct
`WITNESS_END` sentinel for an isolated oracle case; neither sentinel is a lexer
token. The grammar is strong-LL(2)-predictable only when alternative `SELECT_2`
sets are pairwise disjoint and every nullable or repeated
continuation-versus-exit decision is likewise disjoint. A static overlap proves
that this production parser lacks a unique two-token decision; it is labelled a
predictive collision, not automatically an ambiguous complete string. Only the
generalized oracle reports counts of complete derivations.

Each parser terminal category denotes a set of exact token spellings. The
static auditor computes symbolic intersections between every distinct category
pair; the complete parser gate requires the owner-approved categories to form
a disjoint partition over accepted token spellings, including zero
fixed-terminal/`IDENT` intersections. The generalized oracle receives shape
tokens plus their exact bytes, admits every category whose set contains those
bytes, and never receives a priority-selected classification. Under the
recommended fixed-lowercase-terminal repair, the fixed-terminal set is
mechanically extracted from the grammar and candidate `IDENT` is the FORM-3
language minus that complete set. Reserving only a parser-local subset does not
pass.

The generalized oracle counts source-grammar derivation trees, not chart items,
packed-forest paths, or artifacts of EBNF lowering. `?`, `*`, and `+` have one
canonical source-level meaning: absent-or-one value, one ordered list of any
length, and one ordered nonempty list. Two trees are distinct only when they
differ in a normative production alternative, terminal-category assignment,
or source-level child structure. `many` becomes conclusive as soon as two such
complete trees are proved. `zero` or `one` is conclusive only after exhaustive
chart closure for that case; a resource limit before the applicable condition
is inconclusive.

Every authored oracle case preregisters its entry nonterminal, exact spelling
stream, end sentinel, and expected `zero`/`one`/`many` count. For a candidate
whose complete static audit is strong-LL(2)-conflict-free, the static engine
also supplies a deterministic recognizer over the bounded generated domain.
Each recognized stream must be `one` in the generalized oracle and each
rejected stream must be `zero`. The static engine is never said to compute a
derivation count for the ambiguous baseline. Given faithful extraction and the
verified terminal partition, the complete static audit proves strong-LL(2)
predictability over the whole grammar; bounded oracle results make claims only
about their registered or generated domain.

For the pending fixed-terminal/`IDENT` repair, `deref(x)` is the required
transition case; the current discrepancy registry separately carries the exact
`deref(p)` witness. The new case binds exact bytes `deref(x)`, their exact
shape-token sequence—`LowerWordForm("deref")`, `LeftParen`,
`LowerWordForm("x")`, `RightParen`—entry nonterminal `expr`, and
`WITNESS_END`. The exact-v0.8 run must reproduce these two complete
derivations:

1. `expr -> atom -> place -> pbase -> "deref" "(" place ")"`, with
   the inner `place -> pbase -> IDENT` consuming `x`.
2. `expr -> call`, with `callee -> IDENT` consuming `deref` and the sole
   `atom_list` element ultimately consuming `x` through `place -> pbase ->
   IDENT`.

A proposal that makes fixed `deref` ineligible for `IDENT` must remove exactly
the second path for this case, report `one`, retain the first
`expr/atom/place/pbase` fixed-`deref` alternative, and expose the
lexical-membership change.

The proposal-level result also lists every removed, retained, and introduced
intersection or predictive conflict. It verifies this delta only if the named
conflict is removed and no unreviewed conflict is introduced; unrelated
pre-existing blockers remain visible.

That proposal-level result is not the parser entrance gate. Parser work starts
only after an owner-approved successor specification is rerun from its exact
bytes and its full-byte hash equals the reviewed proposal hash, the static
audit is conflict-free for the complete extracted grammar,
every authored oracle result matches its preregistered count, and every
generated stream matches the static recognizer as described above. Any
full-byte difference from the reviewed proposal, any extraction disagreement,
or any non-pass result stops parser implementation.

Neither engine is linked into or invoked by normal compilation. The production
classifier and parser consume only the owner-approved, specification-versioned
grammar contract. The verification entry point may reject or report on a
candidate, but it cannot select language semantics, generate an approval,
change an expected verdict, or grant a parser capability.

**Input contract:** A complete `LexedBundle`, exact specification identity, an
approved fixed-terminal inventory, and parser resource limits.

**Output contract and established invariants:** Exactly one complete typed
derivation consuming every token in every source, or one closed failure
outcome. Before the approved pre-tree DIAG repair, a parse issue is a
toolchain/specification-blocked non-normative observation. After that successor
rule is approved, the same boundary returns a normative syntax rejection with
exactly its approved source-coordinate location and rule attribution; resource
and invariant outcomes remain non-normative. No partial tree escapes.

**Explicit non-responsibilities:** No name lookup, type lookup, canonical
formatting verdict, semantic verdict, recovery, or selection among ambiguous
derivations.

**Why this stage owns the work / why adjacent stages do not:** Only the grammar
defines syntactic derivation. Letting resolution or typing select one makes the
grammar contextual and contradicts the claimed boundary.

**Alternatives considered and rejected:** Keyword-first choice, a conventional
lexer keyword table, and rejecting ambiguous inputs lack normative authority.
A production shared packed parse forest preserves ambiguity but cannot grant a
canonical tree or node path. It remains useful only as a test oracle.

**Trusted assumptions and threat model:** The approved grammar inventory and
parser implementation are trusted production code; hostile tokens try to
exhaust lookahead, nesting, and list storage.

**Failure modes:** Before the DIAG-1 repair, parse issues carry deterministic
source/token coordinates but no normative node path or invented rule verdict.
After the repair, syntax failures use only its approved normative location and
rule mapping; resource and parser-invariant failures remain separate.

**Independent evidence required:** The grammar-change verifier described
above, an explicit extraction/source-coverage differential, authored
shared-prefix cases, bounded generated token streams, fuzzing, and
priority-inversion mutants. Mutants must independently break source binding,
extraction, terminal membership, lookahead relations, and generalized
derivation counts; the intended engine must detect each one.

**Resource and determinism bounds:** Production parsing is
`O(tokens + nodes)` time and memory with an explicit work counter and iterative
stack. The test oracle has separate lower ceilings and an honest worst case up
to cubic time and cubic packed-alternative storage; it saturates derivation
count at two and never materializes all trees.

**Dependencies on unresolved specification questions:** Terminal/`IDENT`,
GRAM-1/GRAM-7, FORM-2, float canonical spelling, and DIAG-1.

**Migration or foundation-audit consequences:** Keep the shape lexer. Do not
add keyword classification to it. Defer parser and tree variants.

**Owner ruling (2026-07-21):** The grammar-verifier design, discrepancy
evidence, protected-surface census, and non-authoritative candidate preparation
are approved. Verifier implementation and the terminal repair remain separate;
exact v0.8 remains active.

### Decision 3: parser construction versus grammar evidence

**Decision:** Make malformed local tree shapes unconstructible through typed
builder APIs, then run one linear finalization audit. Prove grammar agreement
through independent analysis and an oracle, not through the finalizer.

**Problem being solved:** Rust constructors can prevent many malformed trees,
but that does not prove that the implemented alternatives match the normative
grammar.

**Specification and project constraints:** One production/node decision must
be approved first. The parser must be deterministic, resource bounded, and
free of recovery nodes.

**Selected design:** Typed postorder construction, one linear whole-tree
finalizer, and independent grammar analysis/oracle evidence.

**Input contract:** A unique classified token stream and a versioned grammar
contract.

**Output contract and established invariants:** Nodes are built postorder in a
private arena; child handles precede parents; required fields are represented
directly; `arm+` and other nonempty forms use nonempty collections; terminal
leaves retain exact source bytes and ranges.

The finalizer checks one root, one parent per non-root, no child reuse, complete
reachability, contained and ordered child intervals, strictly ordered token
leaves, exactly-once token ownership, and complete token consumption under the
separately approved compilation-unit contract.

The program root cannot fabricate a source-local span. If A-10 and the pre-tree
diagnostic contract are later approved, the proposed concrete representation is
`BundleRootExtent`: an ordered vector of exact per-source token coverage, with
item children ordered first by source ordinal and then by source-local token
interval. The corresponding proposed whole-unit diagnostic is root `NodePath`
plus `Location::BundleRoot`. Those names and contracts are examples of the
blocked A-10/diagnostic design, not part of approval of this construction
strategy.

**Explicit non-responsibilities:** Finalization does not prove grammar
unambiguity, canonical formatting, name validity, types, or semantics.

**Why this stage owns the work / why adjacent stages do not:** Builders own
local shape; finalization owns whole-tree topology; test oracles own independent
grammar evidence. Combining them creates correlated evidence.

**Alternatives considered and rejected:** A broad post-parse "validation pass"
with no surviving failure example adds ceremony. Trusting constructors alone
misses child reuse, orphan nodes, and incomplete root consumption. Sharing
production parser tables with the oracle defeats independence.

**Trusted assumptions and threat model:** Parser bugs may misuse otherwise safe
handles. Untrusted serialized trees receive full structural decode validation,
not the cheaper internal audit.

**Failure modes:** Parser bug becomes compiler-invariant failure; malformed
serialized data becomes artifact failure; neither becomes language rejection.

**Independent evidence required:** Every production and alternative, nullable
and repeated boundaries, malformed near misses, parser/oracle differential,
tree-topology mutants, and bounded grammar-generation coverage.

**Resource and determinism bounds:** One node and bounded builder work per
approved production occurrence; finalization is linear in tokens and nodes;
all expected-token sets use grammar order.

**Dependencies on unresolved specification questions:** Terminal partition,
node-kind mapping, A-10 compilation-unit formation, and the pre-tree DIAG-1
location rule.

**Migration or foundation-audit consequences:** No parser code exists to
preserve. Lexer token handles remain runtime-local.

**Owner ruling (2026-07-21):** The construction strategy is approved exactly at
this boundary. It does not authorize implementation or approve an exact-v0.8
parser, program-root extent, diagnostic location, node-kind/path contract,
multi-file behavior, portable identity, or schema.

### Decision 4: structural and canonical-source validation

**Decision:** Do not create a separate copied AST. Pair typed construction with
a cheap internal topology finalizer and a tree-driven canonical-source audit.
Full structural validation exists only when decoding an untrusted serialized
tree or artifact.

**Problem being solved:** Replaying original tape bytes proves only that the
tape is lossless, not that the tree corresponds to those bytes or that the
source uses canonical spelling.

**Specification and project constraints:** FORM-1/FORM-2 demand one byte form;
GRAM-1 demands a canonical tree; DIAG-2 binds artifact to source.

**Selected design:** Keep one derivation tree, finalize its topology, and grant
canonical syntax authority only after the tree-driven audit succeeds and every
separately approved terminal, node-mapping, canonical-format, diagnostic, and
compilation-unit gate is closed.

**Input contract:** Finalized derivation tree, complete token/trivia tape, exact
source inputs under the approved compilation-unit contract, and the approved
canonical-format rules.

**Output contract and established invariants:** The tree's token-leaf
projection equals the tape's complete token subsequence exactly once and in
order. A tree-driven renderer emits fixed grammar tokens from production kinds,
variable token bytes from validated leaves, and canonical trivia from formatting
rules. For each source admitted by the approved compilation-unit contract, its
output equals the input bytes. The eventual opaque
`CanonicalSyntaxUnit` may be issued only after both checks and only after its
terminal, node-mapping, canonical-format, diagnostic, and multi-file entrance
gates are separately closed. The current architecture ruling selects the audit
strategy, not that concrete factory.

**Explicit non-responsibilities:** The audit does not resolve names or validate
types. Input trivia is not reused as proof of canonicality.

**Why this stage owns the work / why adjacent stages do not:** Only after a
complete tree exists can the tool prove tree/source coverage and canonical
format. Semantic stages must consume that result, not repeat it.

**Alternatives considered and rejected:** Tape-only rendering passes when the
tree is mutated. Normalizing input and continuing violates hard rejection.
Maintaining both CST and AST creates an unnecessary binding proof.

**Trusted assumptions and threat model:** Renderer mistakes can accept a
noncanonical spelling or bind the wrong tree. Tree mutation while retaining the
tape is the primary hostile test.

**Failure modes:** Canonical-source rejection is normative only after the
DIAG-1 location rule is repaired; topology disagreement is an invariant or
artifact failure.

**Independent evidence required:** Tree-only mutations, token deletion and
duplication, punctuation substitution, cross-source leaves, independent byte
format cases, render/reparse metamorphics, and the protected-surface audit.

**Resource and determinism bounds:** Linear in sources, bytes, tokens, and
nodes; one checked output-length calculation; fallible allocation; per-source
comparison; no full duplicate render when a bounded streaming comparator can
be used.

**Dependencies on unresolved specification questions:** FORM-2 conflict,
FORM-5/FORM-7, GRAM-1/GRAM-7, terminal partition, A-10 compilation-unit
formation, and the pre-tree DIAG-1 location rule.

**Migration or foundation-audit consequences:** Retain the lossless tape;
introduce no `ValidatedAst` placeholder.

**Owner ruling (2026-07-21):** The one-tree/audit strategy is approved exactly
at this boundary. Protected/specification bytes, implementation, and the
concrete `CanonicalSyntaxUnit` contract require their separate approvals and
completed entrance gates.

### Decision 5: runtime handles and portable identities

**Decision:** Separate cheap runtime handles from portable artifact references.
Never make a digest or traversal ordinal the sole authority for referenced
content.

| Entity | Runtime handle | Portable reference |
|---|---|---|
| specification | nominal `SpecHash` | exact numbered-spec bytes plus nominal SHA-256 |
| source bundle | private allocation identity | complete ordered source-binding bytes; digest is a locator only |
| source | `SourceId` ordinal in one bundle | source ordinal plus validated logical path in the containing binding |
| token | source-bound dense `TokenId` | source ordinal, byte range, and approved token/terminal kind, revalidated against source |
| tree node | typed dense postorder `NodeId<K>` | child-ordinal `NodePath` from the finalized program root, schema-bound |
| scope | dense `ScopeId` | owner `NodePath` plus closed scope kind and local ordinal where needed |
| declaration | dense `DeclId` | closed `DeclRef`: `Source { node_path, namespace, kind, spelling }` or `Prelude { spec, PreludeKey }` |
| semantic-record owner | tagged owner handle | `SemanticOwnerRef::Template { function: DeclRef }` for source-function symbolic records, or `SemanticOwnerRef::Concrete { instance: SemanticInstanceRef }` for rechecked semantic-instantiation records |
| region | owner-scoped dense `RegionId` | `RegionRef { owner: SemanticOwnerRef, declaration, spelling, class }` |
| normalized instance region slot | owner-scoped dense `InstanceRegionSlotId` | `InstanceRegionSlotRef { instance: SemanticInstanceRef, slot: RegionSlot }`; validates slot bounds/equality/profile and has no declaration or spelling |
| call region actual | tagged local handle | `RegionActualRef::Lexical(RegionRef)` or `RegionActualRef::InstanceSlot(InstanceRegionSlotRef)`; never an untagged or recursively expanded reference |
| borrow holder | owner-scoped dense `HolderId` | `HolderRef { owner, binding: DeclRef }` or `HolderRef { owner, call_temporary: NodePath, ordinal }` |
| loan | owner-scoped dense `LoanId` | immutable `LoanRef { owner, creation: NodePath, creation_holder: HolderRef, region: RegionRef, kind, ordinal }`; current binding-to-claim relations are a separate A-16-gated record family |
| operation entry | dense table index | `OperationRef { spec, OperationKey }`, separate from declarations and source nodes |
| type | interned `TypeId` | full canonical structural term; nominal leaves use source or prelude `DeclRef` |
| function | dense `FunctionId` | function declaration reference |
| semantic instantiation | dense `SemanticInstanceId` | `SemanticInstanceRef` containing function, complete type/const substitution with every nested region leaf replaced by a canonical first-occurrence `RegionSlot`, and the finite owner-independent region-fact profile selected by A-17; it contains no `RegionRef` |
| code instance | dense `CodeInstanceId` | baseline `CodeInstanceRef { semantic_instance: SemanticInstanceRef }`, one-to-one and non-recursive; a region-erased `EmissionShapeKey` is comparison data only and cannot merge bodies without a separately verified whole-class equivalence |
| value | owner-scoped dense `ValueId` | `ValueRef { owner, definition: NodePath or closed derived key, ordinal }` |
| CFG block/edge | owner-scoped dense `BlockId`/`EdgeId` | `BlockRef`/`EdgeRef` containing `owner` plus the complete source-or-derived structural key |
| place | owner-scoped dense `PlaceId` | `PlaceRef { owner, resolved_root: DeclRef, projections }` |
| check | owner-scoped dense `CheckId` | `CheckRef { owner, node: NodePath, class, ordinal }` |
| semantic derivation/proof | owner-scoped dense `DerivationId` | `DerivationRef { owner, closed_family, conclusion_key, ordinal }`; the complete record at that reference repeats the owner and carries all premises and conclusion |
| typed call boundary | owner-pair-scoped dense `TypedCallBoundaryId` | immutable `TypedCallBoundaryRef { caller, call_node, ordinal, callee }`; its record binds one typed-call record plus complete type/const and region-formal/slot environment, but contains no place/origin assertion |
| call provenance boundary | owner-pair-scoped dense `CallProvenanceId` | immutable `CallProvenanceRef { typed_boundary }`; its record is constructed later, repeats/validates the same owner pair, and contains the only admitted directional callee-formal-place/origin-to-caller-actual mappings and result origin |
| binding-to-claim relation | owner-scoped dense handle, shape blocked on A-16 | no portable schema until A-16 fixes direct-borrow holders, shared-borrow copies, unique-borrow moves, parameters, return/`give`, and match-binder projections |
| whole-unit/target derivation | record-family-local dense index | a distinct non-semantic-owner record family with its complete canonical conclusion key; it cannot satisfy a template or concrete semantic premise |
| artifact | internal byte owner | domain-separated digest of the complete canonical bytes, always paired with bytes at trust boundaries |

**Problem being solved:** Dense indices are useful implementation handles but
become stale or misleading when serialized, rebuilt, reordered, or moved
between source bundles.

**Specification and project constraints:** DIAG-1/DIAG-3 require stable node
paths and report references. DIAG-2 requires a canonical artifact. Exact source,
tree, target, and schema identities must remain bound together.

**Selected design:** Use owner-scoped dense runtime handles and the full
structural portable references in the identity table above. `Template` names
the symbolic record domain for a source function declaration; it does not mean
that the declaration necessarily has type or const generics. Every serialized
region, holder, loan, value, CFG, place, check, semantic proof, and every
reference between them carries the same explicit `SemanticOwnerRef` tag.
`SemanticInstanceRef` and `CodeInstanceRef` are deliberately distinct types,
but the conservative baseline maps them one-to-one. A call's
`RegionCallEnvironment` maps the callee's declared region formals and normalized
type-argument `RegionSlot`s to caller-side `RegionActualRef`s. A generic
forwarding call can therefore map callee slot to caller instance slot without
inventing a lexical declaration. Exact bounded environment composition follows
incoming typed boundaries until a lexical actual or a stable slot-to-slot cycle;
it validates the A-17 fact profile and never expands references into either
instance key.

**Input contract:** Finalized containing representation and its exact schema,
source, specification, and target identities.

**Output contract and established invariants:** Every internal handle is scoped
to one owner. Every portable reference validates owner tag, kind, bounds,
content, and containment against the enclosing artifact. A template record may
reference only records with its exact `Template { function }` owner; a concrete
record may reference only records with its exact
`Concrete { semantic instance }` owner. The sole inter-owner channels are the
two closed staged families above: replay first validates a
`TypedCallBoundary`'s caller/callee against its bound typed-call record and
permits only type/const/region relations; a later `CallProvenanceRecord` must
reference that exact boundary and may add only place/origin relations. Records
on each side remain owner-local, and no raw foreign reference may bypass either
record. Replay rejects swapped caller/callee,
cross-function, cross-instance, stale, duplicate, wrong-kind, and undeclared
cross-owner references. No template proof can discharge a concrete premise,
including for an empty-key instance of the same function. Equality and order
are defined by canonical structural bytes, not allocation or insertion order.

`PreludeKey` and `OperationKey` are closed specification-versioned enums with
canonical order and collision checks against source declarations. They never
create synthetic source nodes or source spans. A prelude declaration's
diagnostic location is a closed built-in origin, while a source rule violation
still points to the source use node.

**Explicit non-responsibilities:** A hash does not prove semantic validity. A
`NodePath` does not exist before a canonical tree. Runtime indices need not
survive rebuilding.

**Why this stage owns the work / why adjacent stages do not:** An identity is
defined only after the representation it names is final. Freezing anticipated
IDs before the real tree or artifact exists would make schema accidents
permanent.

**Alternatives considered and rejected:** Globally hashed nodes make collision
handling and source/tree binding implicit. Raw dense indices are not portable.
Source byte spans alone do not distinguish nested grammar nodes with equal or
zero-width boundaries.

**Trusted assumptions and threat model:** SHA-256 is collision-resistant for
cache lookup, but authority still compares and validates complete bytes. Hostile
artifacts may alias indices, change kinds, create cycles, or cross containers.

**Failure modes:** Invalid references are artifact failures; stale internal
handles are compiler invariant failures; neither is a language rejection.

**Independent evidence required:** Cross-bundle substitution, type confusion,
index aliasing, path mutation, hash-domain substitution, reordered tables,
duplicate keys, and rebuild-determinism cases.

**Resource and determinism bounds:** Reference validation is linear or
log-linear in artifact size; node paths are bounded by syntax depth; canonical
tables use explicit stable sorting and reject duplicate keys.

**Dependencies on unresolved specification questions:** Canonical node kinds,
multi-file program root, proof-reference identity, and final artifact schema.

**Migration or foundation-audit consequences:** Keep current source/token
handles as local only. Do not advertise them as stable artifact IDs.

**Approval status:** The architecture is adopted. Canonical node kinds,
multi-file composition, proof-reference identity, and the final artifact schema
remain separately gated.

### Decision 6: declaration inventory and resolution

**Decision:** Separate complete declaration inventory from visibility-governed
resolution. Build scopes and headers deterministically, stop on malformed or
duplicate declarations, then resolve every use to one explicit declaration
before any body enters type or ownership checking.

**Problem being solved:** Whole-unit signatures, recursion, constructor
uniqueness, no-shadowing, and declaration order need different information.
Collecting a declaration is not permission to use it.

**Specification and project constraints:** TYPE-6 defines disjoint `IDENT`,
`REGIONID`, and `LABEL` namespaces, declaration-before-use, and no live
shadowing. Variant `TYPEID`s are globally unique. FN-1/FN-6 require complete
call signatures and cycles. PROG-1 is closed world.

**Selected design:**

1. Insert the exact prelude as versioned built-in declarations, not synthetic
   source nodes.
2. Walk top-level items in canonical source order and inventory nominal types,
   variants, contracts, functions, conformances, and consts.
3. Validate header-local generics, region parameters, duplicates, reserved
   names, and declaration kinds.
4. Construct the lexical scope tree and an ordered event stream for parameters,
   locals, match binders, regions, labels, and requires-block locals.
5. Resolve every lexical type, value, region, label, function, variant,
   contract, named-const, and written operation use under the approved
   visibility/reservation predicate.
6. Emit exact lexical use-to-declaration/operation records and complete lexical
   node coverage.
7. Leave field projections, construction/match field labels, named call
   arguments, conform bindings, and contract members as explicit typed-label
   uses. Decision 7 resolves each only after its base nominal type, constructor,
   function signature, or contract is fixed.
8. Emit no call graph or SCC. Decision 7 turns each lexical callee resolution
   plus typed labels and explicit arguments into immutable
   `TemplateTypedCallRecord` and later `ConcreteTypedCallRecord` families;
   Decision 9 alone derives the corresponding template/concrete call edges and
   SCCs from those records.

**Input contract:** `CanonicalSyntaxUnit`, exact prelude and reserved-operation
inventories, approved multi-file order, and the approved top-level visibility
rule.

**Output contract and established invariants:** One rooted acyclic scope tree;
one declaration record per declaration node; one resolution per lexical use
node; exact spelling, namespace, kind, ancestry, visibility, and order; explicit
unresolved typed-label uses for Decision 7; no missing, duplicate, poison, or
ambiguous lexical binding.

**Explicit non-responsibilities:** Lexical resolution does not infer types,
select an operation by operand type, resolve a field/member without its checked
owner, instantiate generics, build call graphs/SCCs or CFGs, or continue after
a declaration error.

**Why this stage owns the work / why adjacent stages do not:** The parser knows
spelling but not binding. Lexical declarations must be fixed before type
checking. Field/member labels are different: their owner declaration exists
only after the base type/signature is checked, so Decision 7 owns that
deterministic lookup. Neither lookup is parser disambiguation or overloading.

**Alternatives considered and rejected:** A single map filled while checking
bodies cannot support cycles and conflates collection with visibility. Poison
declarations create cascades and incomplete proofs. Type-directed constructor
or operation resolution violates explicit spelling.

**Trusted assumptions and threat model:** The checker resolver is production
TCB code. Same-kernel replay revalidates sorted scope records, ancestry,
declaration events, target spelling/kind, and exact coverage from artifact
bytes. The bounded scope model in Decision 16 is the independent semantic
evidence and must not call the resolver.

**Failure modes:** The resolver emits the first deterministic normative
rejection after a canonical tree exists. Resource exhaustion and internal table
inconsistency are separate outcomes.

**Independent evidence required:** A bounded independent scope model; generated
nested scopes; all namespace pairs; earlier/later uses; no-shadowing near
misses; variant and nominal collisions; reserved names; mutual cycles; and
wrong-target/missing-use replay mutants. Typed-label cases belong to Decision 7.

**Resource and determinism bounds:** Canonical source-order inventory and
diagnostics; sorted scope tables; `O((declarations + uses) log declarations)`
or better; explicit limits on scopes, events, declarations, uses, spelling
bytes, ancestry depth, and work.

**Dependencies on specification work:** A-01's semantics are owner-selected but
still require exact successor-specification encoding. TYPEID constructor
collisions, OP-1 reservations, contract members, and multi-file order remain
unresolved.

**Migration or foundation-audit consequences:** No resolver schema or stable
`DeclId` should be implemented before these questions close.

**Owner ruling (2026-07-21):** All top-level function signatures are visible
throughout the closed compilation unit. Locals, regions, labels, and named
constants remain declaration-before-use. This selects A-01's semantics; the
exact version-bumped successor-specification bytes remain separately guarded
before resolver implementation.

## Proposed successor-specification compilation-unit rule

The source foundation already models an explicitly ordered bundle, while v0.8
defines only `program := item*`. Because bundle order can change normative
visibility and acceptance, the following is a proposed numbered specification
rule, not a lower-authority toolchain convention. It requires the
normal exact owner approval, governance entry, version bump, filename/title
change, live-reference update, and protected additive evidence before any
multi-file parser or resolver implementation:

- A compilation unit is an ordered **nonempty** sequence of source files. Zero
  source files is an input-envelope failure: no Whitefoot program exists to
  judge. The generic `SourceBundle` carrier may represent zero files for tests
  or transport, but the compilation-unit constructor must reject it.
- Each source is a FORM-2 file, not a new category below "file." Therefore a
  source containing zero items has canonical bytes of exactly one LF; zero
  bytes is noncanonical. Every nonempty item sequence also ends in exactly one
  LF under FORM-2.
- One invocation has one ordered compilation-unit `SourceBundle`. Logical paths and file
  partition are part of source/artifact identity, not language names.
- Each source contains zero or more complete top-level items. No token,
  grammar production below `program`, or source span crosses a source boundary.
- The grammar parser consumes each source's complete item sequence. A single
  toolchain program root owns all item nodes flattened by bundle order and then
  source item order. It carries `BundleRootExtent` (ordered complete coverage
  for each source), not a source-local byte span. All descendants are
  source-local. Source-container records remain outside the grammar tree.
- The parser inserts no whitespace, delimiter, separator, declaration, or
  synthetic token between files.
- Canonical byte formatting and rendering are checked independently for each
  FORM-2 source file, including the one-LF zero-item case.
- Declaration order is bundle order followed by item order. Global uniqueness,
  the exact prelude, `main`, conformances, call graphs, SCCs, instances, and
  reports range over the whole closed unit.
- Empty sources and source boundaries remain in the source binding even when
  they contribute no item. Repartitioning the same token sequence changes
  source and artifact identity, even if later semantic behavior is equivalent.
- There is no include, import, module, separate compilation, incremental
  semantic cache, internal ABI, or source-path lookup in the language.

A single-source-only compiler is the only exact-v0.8 envelope that does not add
this missing composition rule, but the terminal and node-kind blockers already
prevent a complete exact-v0.8 frontend. Carrying single-source composition into
the successor specification would postpone real ordering and identity
questions while preserving no executable v0.8 milestone, so it is rejected.
