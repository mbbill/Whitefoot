# THE PLAN

Status: CANONICAL ROADMAP, updated 2026-07-20.

This file contains Whitefoot's one current execution order. Other files may
define law, specify behavior, explain a design, or preserve evidence. They do
not set current priority or authorize work.

## Authority and scope

- `CONSTITUTION.md` defines project law.
- `optimizer-language-research/notes/user-directives.md` records owner rulings.
- `spec/kernel-spec-v0.8.md` defines the accepted language.
- `PATTERNS.md` defines the closed set of forms writers use.
- This file orders implementation and records its authorization boundary.
- `optimizer-language-research/implementation/decision-gates.md` preserves the
  append-only evidence log. `mcts_mem/` preserves design reasons and rejected
  alternatives.

If these sources disagree, stop at the conflict and reconcile the governing
source and this roadmap in one change. A research result, handover, follow-up
list, design dossier, archived plan, or MCTS alternative cannot authorize work.

On 2026-07-17 the owner directed the project to consolidate its plan files and
complete phases 1 through 7 below. That direction authorizes each phase after
its predecessor passes its exit gate. Phase 1 completed on 2026-07-17; phase 2
is active. The authorized scope ends with the first production `seq` component.
It excludes every later
container, checked-source library, public I/O or FFI surface, compiler
migration, and concurrency feature.

A failed correctness gate returns work to the first phase that owns the defect.
A phase cannot borrow a future artifact to satisfy its gate.

Every reproducible defect or discrepancy gets the smallest practical automated
regression before its fix closes. Language and specification discrepancies use
conformance cases; implementation defects use focused unit or integration
tests; hostile boundary failures also pin failure atomicity and guard behavior.
If automation is not practical, the decision log records the reproducer and why
the gate cannot execute it yet.

## Measured baseline

Baseline date: 2026-07-17, before plan consolidation.

- The wfc unit contains 32 Whitefoot files, 24,695 lines, and 477 functions.
  Concatenation in `sources.txt` order produces 1,095,342 source bytes, 217,254
  tokens, and 109,235 unique-head AST nodes.
- wfc lexes, parses, validates, indexes, resolves declarations and types, and
  builds function scopes for that complete unit. Two parses produce identical
  token and AST tapes.
- Body semantics classify 15 functions as provisionally clean and report 462
  as Unsupported, which makes no legality claim; zero reach a semantic-reject
  path in the then-implemented subset. The first source-order frontier is
  `lexer_scan_op_suffix`.
- LLVM support emits the same 15 clean functions as one deterministic module.
  Clang accepts it and the native tests pass.
- `wfc_frontend_run` is the current external seam. The complete `wfc_compile`
  path, a stage-1 compiler, and a self-hosting fixpoint do not exist yet.
- wfc uses fixed-capacity structure-of-arrays tapes backed by primitive
  buffers. Source size bounds each tape. Bootstrap needs no growable
  collection, pool, generic container, or thread.
- Stage 0 is `prototype/democ`. It compiles wfc with optimizer facts disabled.
  The current v0.6 implementation and conformance suite provide the independent
  behavior oracle while wfc catches up.
- The systems-capability research package has finished. Its 15-rule loan/freeze
  design passed a 97-program machine corpus and all nine mutation tests. Its
  `seq` operation table and dry run provide landing evidence. None of that work
  has changed the production specification or compiler.
- `make check` and `make -C compiler check` passed at the baseline commit.

## Phase 1: establish one roadmap

**Entry:** the 2026-07-17 owner direction above.

Work:

1. Move the live compiler sequence from `compiler/PLAN.md` into this file.
   Preserve architecture in `compiler/README.md` and `mcts_mem/`; remove the
   duplicate plan.
2. Move `HANDOVER.md` and the completed validation-harness plan into `archive/`.
   Fold unresolved production prerequisites from the capability follow-up list
   into the relevant gates below, then remove that list.
3. Replace dated current-focus material in `AGENTS.md` and `CLAUDE.md` with a
   pointer here. Keep those two files byte-identical.
4. Add a root check that rejects another plan-named file, a second canonical
   roadmap marker, a retired active path, or a duplicate current-focus section.
   Historical plans may remain under `archive/` and rejected alternatives may
   remain under `mcts_mem/**.alt/`.
5. Correct stale counts, decisions, and authorization statements in active
   documentation. Preserve completed protocols and archived evidence unchanged.

**Exit gate:** this file is the sole active roadmap. The structural check
rejects competing roadmap markers and retired paths; review confirms that no
other active document defines execution order. Both project gates pass. The
change has one commit and one decision-log entry.

## Phase 2: self-host the compiler subset with facts off

**Entry:** phase 1 exit.

Execute these steps in order:

1. **Complete semantics and certify the compiler unit.** Grow wfc's declaration,
   type, ownership, loan, control-flow, effect, and whole-unit checks until the
   exact `sources.txt` unit receives one atomically published compiler-subset
   `CheckedUnit`, not just per-function statuses. Semantic `Unsupported` means
   not certified and makes no legality claim. The rejection ABI records one
   rule ID, canonical node path, primary and related nodes, source span, and an
   applicable mechanical fix code.
   Extend one unified, source-independent body checker by one cluster of
   specification rules per slice under the production-compiler contract below.
   A slice is only a delivery and review boundary; it never becomes a
   production profile or whole-function dispatch path. Compare every covered
   conformance case and the whole compiler unit with stage 0. Resolve each
   verdict difference as a wfc defect or a stage-0 defect backed by a new
   conformance case.
2. **Complete whole-unit lowering.** Start only after the compiler-subset
   `CheckedUnit` publishes. Build one generic `CheckedUnit` plus verified
   `FactOverlay`-to-LLVM lowering path, using the canonical empty overlay in
   facts-off mode. Its dispatch is by statement, expression, type, operation,
   checked semantic result, and verified check decision, never by source
   function or whole-body family. Grow that path until `wfc_compile(sources)`
   emits every function. Clang must accept the complete module, and native tests
   must exercise it.
3. **Run compiler-subset conformance through stage 1.** Add a wfc adapter to the
   conformance runner. Require zero verdict differences for the language subset
   wfc's source uses.
4. **Reach the facts-off fixpoint.** Build wfc1 with stage 0; compile the unit to
   IR2 and build wfc2; compile the same unit to IR3. Require byte-identical
   `IR2 == IR3`. Freeze stage 0 at this point. Keep it as a differential oracle,
   but add no language feature to it.

Facts remain disabled throughout this phase. Unit counts and clean ordinals are
regression observations, not implementation targets or sources of authority.

### Phase 2 production-compiler contract

The objective is a production implementation of Whitefoot, followed by generic
lowering and a self-hosting fixpoint. Compiler-unit counts are integration
observations. They do not define legality, select work, or confer authority.

Incompleteness is stage-qualified. Body `Unsupported` means only that an
applicable semantic transition in the current facet catalog is not implemented;
it makes no claim that the source is legal. An incomplete declaration,
whole-unit, or artifact obligation prevents `CheckedUnit` publication without
rewriting a body verdict. A backend gap reports `LoweringUnsupported` against an
already certified unit and cannot change semantic legality or certification. A
per-body `CLEAN` result is provisional: it means the one body checker traversed
that body under a validated FN-1 signature and reconciled its local exit
judgments. Whole-unit compiler-subset certification is separate and atomic. It
requires valid declarations, every declared body and every concrete
instantiation named transitively by a resolved call or instantiation edge in any
declared body—treating every declared body, not only `main`-reachable bodies, as
a graph root—every applicable source and whole-unit rule, and generic-cycle
obligations to pass. Neither body `CLEAN` nor unit certification authorizes
optimizer facts. Lowering consumes only the published elaborated artifact
described below.

Phase 2 is facts-off. Checked indexing, arithmetic, allocation, and other
trapping operations compile with their checks intact. A proof that a check
cannot fire is optional evidence for a later facts-on optimizer channel, never
a prerequisite for accepting legal checked code.

#### One semantic pipeline

There is one production semantic pipeline, with these reviewed boundaries:

1. **Validated input.** Structural validation publishes a unit-bound validated
   AST handle tied to the exact source, token, AST, and validator generation.
   Declaration, resolution, and semantic phases consume that handle rather than
   repeatedly treating raw topology as trusted input. A malformed or stale
   handle fails at the boundary.
2. **Declarations and whole-unit rules.** One pass indexes declarations, types,
   constants, contracts/conformances, generic parameters, and complete FN-1
   signatures under stable IDs. It checks declaration-phase whole-unit rules,
   including namespace uniqueness and FN-7 entry status. This table, not a
   callee body or prior body-analysis verdict, is the call-checking authority.
   The body checker later records resolved call and instantiation edges; the
   unit finalizer alone owns FN-6 SCC validation over those edges.
3. **One syntax-directed body checker.** Every concrete body enters the same
   `check_function` driver. That driver folds generic
   `check_place`, `check_atom`/`check_expression`,
   `check_statement`, and `check_block` judgments over arbitrary legal AST
   lists and nesting. Production rule selection depends only on the current AST
   construct and resolved semantic state. Each call records its resolved
   declaration and explicit substitution edge. A delivery slice never becomes
   a profile, function family, or alternative body path.
4. **Staged unit finalization.** Each body result is staged without publishing
   unit acceptance. The unit finalizer verifies that every body reconciles with
   its signature, runs FN-6 SCC validation over the resolved call and
   instantiation graph, and checks all other whole-unit and concrete-
   instantiation obligations. It then publishes one immutable certified unit
   atomically. Failure publishes no partial clean unit, elaboration, effect
   result, lowering token, or optimizer fact.
5. **One elaborated semantic artifact.** Acceptance produces a versioned,
   unit-hash-bound `CheckedUnit` containing resolved types and modes,
   declaration and concrete-instantiation IDs, place/root provenance, per-edge
   owned/loan state, explicit region exits, derived drops and releases,
   instantiated calls and effect rows, retained runtime checks, `try`/`give`
   and other control edges, monomorphizations, diagnostics mappings, and
   canonical source paths. It also embeds verifier-checkable acceptance
   derivation records for every completed facet and per-check instrumentation
   with retained/eliminated status and proof reference, sufficient to decide its
   certified claims without rereading raw source or mutable external state. A
   derivation reference may point only to a record embedded in the same artifact
   or to an immutable rule schema identified by the artifact's exact
   specification and verifier hashes; a bare rule ID, handler ID, external cache
   entry, or compiler assertion is not a proof object. Each record is
   provenance-bound to its rule facet and node/declaration identity. The schema
   contains no profile, family, function-shape, or corpus-membership
   discriminator.
6. **One generic lowerer.** The lowerer consumes `CheckedUnit` plus a verified
   optional `FactOverlay`, both read-only. Facts-off uses one canonical empty
   overlay. A nonempty overlay is a closed, ledgered union of verified
   optimizer-only record kinds: implicit/spec-elidable check elimination,
   region-scoped alias metadata, declaration effect attributes, and checked-law
   rewrite licenses. Each kind has its own proposition, scope, producer,
   provenance, consumer, invalidators, and proof schema; an unknown kind fails
   closed. Check-elimination records may target only checks explicitly marked
   implicit and spec-elidable. OP-5/FN-8 explicit checks, user checks, and every
   non-elidable check class remain retained regardless of proof. A fact verifier
   binds every record to the exact unit, specification, verifier version,
   node/declaration/check identities, and proof objects. No record may alter
   `CheckedUnit` or semantic certification. The overlay is included in the same
   content-addressed canonical artifact bundle, so no mutable external fact
   state is consulted.
   The lowerer dispatches only on elaborated node kind, resolved type/layout,
   operation, control edge, check record, verified overlay decision, and derived
   cleanup. It never re-resolves raw text, rechecks semantics, or selects
   emission by source function, signature profile, body shape, or compiler
   corpus membership. Module publication has one failure-atomic entry.

The target production architecture has exactly one constructor/publication site
for provisional body `CLEAN`, one for certified `CheckedUnit`, and one for a
completed LLVM module.
The body site is reachable only after the generic driver visits every applicable
node and performs cited function-exit rules such as return reconciliation and
EFF-2 equality. Syntax helpers return local typed transitions, flow states,
exhibited effects, elaboration records, or diagnostics; they never return a
whole-function admission verdict. CI rejects an alternative status constructor,
a publication-path call to a whole-body or subtree-profile validator, and any
admission or lowering branch that compares a resolved identifier with a
hard-coded compiler-corpus name or function ordinal. Ordinary symbol-table
equality, declaration-before-use, reserved/prelude recognition, and callee
resolution remain mandatory.

During recovery, every remaining legacy profile path is explicitly quarantined
and may emit only a `LegacyProfileResult`; it cannot feed `CheckedUnit`, generic
lowering, or optimizer facts. The legacy call-graph and authority baseline may
only shrink, and new edges into legacy validators are forbidden. The generic
`CLEAN` site accepts only syntax-directed body results, and `CheckedUnit`
publication remains disabled until every legacy admission path is removed.
`LegacyProfileResult` may feed only an explicitly named isolated regression
harness. It may not feed `wfc_compile`, completed-module publication, a
production artifact or coverage claim, any certification gate, or any new
consumer. The existing 15-function profile emitter may continue only as a
frozen historical regression seam until removal; its output is explicitly
non-production, carries no semantic, lowering-completeness, or optimizer
authority, and its call graph may only shrink.

In Phase 2, `CheckedUnit` is compiler-subset certification bound to the exact
completed-facet catalog; it does not claim that arbitrary v0.8 input is fully
supported. Its permanent DIAG-2-shaped schema is already the generic lowerer's
semantic input; facts-off supplies the canonical empty overlay. Phase 2 executes
the embedded verifier for the compiler-subset claims. Phase 3 closes every
remaining normative facet and executable DIAG-2/3 obligation before the same
artifact schema and verifier can publish full-v0.8 certification.

#### Calls, effects, ownership, and flow

Per FN-1, a call uses only the structurally and semantically validated callee
declaration. The checker verifies the named formal-to-actual mapping, exact type
and written-mode equality, explicit type/const/region substitution, ordered
owned-value consumption, loan compatibility, and the instantiated declared
effect row. The caller path never inspects or re-proves the callee body.
Separately, every concrete callee body is checked once against its own signature.
Unit-finalization SCC analysis consumes the body-produced call and instantiation
edges to check FN-6 and deterministic monomorphization obligations. It is not
effect inference and creates no caller-specific callee proof.

Every declared body is checked; no invalid declaration escapes checking merely
because no caller reaches it. Every concrete generic instantiation named
transitively by a resolved call or instantiation edge in any declared body is
also validated under its explicit substitution and deterministic
monomorphization identity, treating every declared body—not only
`main`-reachable bodies—as a graph root. Cached semantic data may be reused only
when the analyzer version, complete unit hash, validated-AST identity,
symbol/type-table generations, and every referenced declaration identity match;
otherwise reuse fails closed.

EFF-2 reconciliation is a syntax-complete, control-flow-insensitive fold over
the body and `requires` block. It includes unreachable syntax, instantiated
declared callee rows, implicit checks, and checks later proved redundant. At
function exit, the complete exhibited reads, writes, allocates, and traps are
compared in both directions with the declaration.

The reviewed ownership design must state, at minimum:

- each binding's uninitialized, live, consumed, and dead states, its resolved
  type and written mode, and its copy or affine class;
- every place's immutable root and projection path and the OWN-7 overlap
  relation;
- live loan lineage, shared/unique capability, usable or suspended state, and
  statement-scoped child boundaries;
- lexical region/outlives and scope stacks;
- fallthrough, return, labeled break, give, region-exit, and derived
  drop/release edges;
- a proof-preserving join retaining only propositions true on every predecessor
  and OWN-8 rejection for incompatible affine or loan states;
- convergent loop fixed points and OWN-11 back-edge checks; and
- call-time consumption, sibling-overlap checks, parent suspension/resumption,
  instantiated effects, and failure-atomic state updates.

Diagnostics use canonical source/node order plus a fixed rule priority where one
node violates multiple rules. Their result is independent of body-analysis
scheduling, SCC traversal, worklist order, or parallel execution for one fixed
canonical source and declaration order. Tests hold that semantic order fixed,
permute only analysis scheduling, and require byte-identical diagnostics.

#### Generalization and completeness

Grammar and judgment constructor schemas are finite; program sizes,
environments, place paths, loan sets, region nesting, generic instances, and
CFG-state instances are not. Generality rests on total transition schemas
parameterized by those unbounded finite structures, with reviewed induction
invariants for list traversal, scope and loan stacks, place resolution,
control-flow composition, and fixed-point iteration. It does not come from
enumerating compiler functions.

A machine-checked implementation ledger is indexed by atomic facets: every
grammar production; type, mode, and storage constructor; operation-table row,
domain, and edge case; ownership premise and transition; effect clause; control
transfer; diagnostic/artifact obligation; and applicable whole-program rule.
Each facet records a validated owning stage (`lexer/tokenizer`, `validator`,
`parser`, `declarations`, `resolver`, `checker`, `unit-finalizer`, `artifact`,
`diagnostic/report`, `runtime`, `lowerer`, `fact-verifier/optimizer`, `policy`,
`spec-CI`, or justified `N/A`), handler, implementation status, positive and
negative witnesses, composition witnesses, diagnostic path, malformed-input
behavior, elaboration field, lowering obligation or structured none, and
normative/provisional/deferred status. CI rejects an unowned facet, an
unreasoned none, a blank/default-accept handler, or a source-emittable rule
marked policy-only.

The facet catalog is versioned and hash-bound to the exact kernel specification.
Grammar productions, rule IDs, and operation-table rows are extracted
mechanically where possible; manually decomposed prose facets carry exact
rule/text anchors and hostile-review approval. CI displays and rejects an
unexplained facet addition, deletion, merge, split, or status change. Each
semantic handler exit links to exactly one of: a successful facet transition;
one exact numbered-rule rejection premise; or a stage-qualified `Unsupported`
linked to one exact pending facet. An unledgered wildcard, catch-all `Reject`,
catch-all or default `Unsupported`, or fallback diagnostic over a completed
normative facet is forbidden. Mutation gates delete or redirect each
handler/facet edge and must fail.

If new evidence exposes a soundness gap in a completed facet, certification and
all dependent publication stop immediately. Reopening requires a reviewed,
versioned catalog revision of that exact anchored facet; the revision
invalidates every dependent staged body result, `CheckedUnit`, `FactOverlay`,
module, and cache entry before the facet may return stage-qualified
`Unsupported`. Do not invent a source-shaped facet, cite an unrelated rejection
premise, or use a broad rule such as OWN-8 to camouflage an implementation gap.

Phase 2 closes the facets exercised by the exact compiler subset while using the
permanent architecture above. Phase 3 closes every remaining normative
source-emittable v0.8 facet without replacing that architecture. It makes
`Unsupported` unreachable for canonical parseable v0.8 input: valid programs
are accepted and invalid programs receive their required diagnostic. Evidence
combines static exhaustive dispatch over every finite syntax, type/mode,
operation, and analysis-state discriminator; reviewed structural-induction and
invariant-preservation arguments parameterized by arbitrary finite state
instances; and bounded generative/model tests as validation rather than proof.
Those tests compose legal alpha-renaming, multiple list lengths and nesting,
all operation domains and boundary values, ownership/control-flow combinations,
region substitutions, effects, and acyclic and recursive call graphs. Zero
conformance skips is necessary, not sufficient. Non-program rules require
executable artifact, policy, or spec-CI gates. This is not an exhaustive
enumeration of programs.

Metamorphic requirements are explicit rather than "semantic equivalence":
alpha-renaming applies only to non-reserved identifiers under TYPE-6 and legal
scope; independently legal copy-only statements may be inserted where they do
not change flow/effects; blocks and nesting are tested at zero, one, multiple,
capacity-boundary, and recursively composed lengths while implementation remains
parameterized over every representable finite length; and permutations are
tested only when the applicable ownership, flow, and effect premises are
unchanged.

The Phase 2 architecture design maps all v0.8 categories now, including
generics/contracts, `requires`, `try`/`give`, consts, allocation and
storage, drops/releases, aggregate values and returns, and DIAG-2/3, even when a
handler is explicitly pending until Phase 3. Deferred text is not promoted and
provisional status is not silently treated as ratification.

#### Required exactness and prohibited profiles

Exact local checks remain mandatory wherever a numbered rule requires them:
grammar-fixed direct-child arity, well-formed bounded variable child chains,
canonical operation and literal form, literal type/range/constant validity,
resolved nominal declaration identity, named argument order, explicit
substitution, sequential ownership flow, callee signatures and effects,
loop/region structure, source anchoring, and bounded topology. These local
judgments are not profiles.

The following routes are prohibited:

1. **No whole-function or subtree admission profile.** No production entry may
   select, short-circuit, or publish a function or subtree verdict from a
   preselected conjunction of source name/ordinal, hard-coded project nominal
   spelling, signature/effect shape, fixed block/region/loop/match-arm child
   count, statement/literal/callee sequence, or body/subtree fingerprint. Exact
   arity applies only where the grammar fixes a node's direct arity; variable
   statement and item lists are traversed generically. Rule-defined recursive
   subjudgments—such as block flow, exhaustive-match traversal, and
   GRAM-7/GIVE-1 delivery completeness—are allowed when parameterized by
   arbitrary resolved input and analysis state. They return the cited local
   rule fact, never a source-shaped subtree admission status.
2. **No profile laundering.** A parser, resolver, cached flag, generated table,
   stage-0 verdict, host shim, test manifest, or lowerer may not turn such a
   fingerprint into a "resolved semantic property." Resolved nominal identity
   remains mandatory; selecting an analyzer because that identity has a
   repository-specific spelling, ordinal, or enumerated compiler membership is
   prohibited.
3. **No special-case stacking.** Do not add a signature/body fallback, wrapper
   or probe recognizer, exact call-sequence path, or parallel emitter for a newly
   encountered source shape. A general transition replaces or subsumes the
   source-specific authority it covers; it is not stacked beside it.
4. **No census authority.** Hard-coded compiler function names, corpus ordinals,
   dependency-frontier lists, clean ordinals, and desired counts are
   observations only. They never select an alternative rule, checker, lowering,
   test-expectation, or slice path. Ordinary source-name resolution to a
   declaration is not census authority.
5. **No callee-body re-proof.** A caller relies on the validated signature as
   FN-1 requires. Re-reading a callee body from the caller path, including via a
   cached profile or "independent proof," is prohibited.
6. **No proof-for-admission.** Source-specific proofs of bounds, overflow
   freedom, termination, literal values, or algebraic properties are not
   prerequisites for compiling checked operations. This does not waive
   canonical literal parsing, type/range validity, constant-expression rules,
   effect reconciliation, or any other specification check. Proof may remove a
   check only through a separately reviewed fact channel with facts-off
   semantic identity.
7. **No analyzer-driven source distortion.** Investigate every discrepancy
   against the accepted specification. Repair the compiler or source to match
   it; a specification change requires its owner-gated evidence protocol. Do
   not rewrite wfc into artificial shapes merely to satisfy an incomplete
   analyzer. Spec-required canonical migration is not distortion.
8. **No raw-AST backend semantics.** Lowering consumes only `CheckedUnit`.
   Source-specific LLVM emitters, raw-text semantic rediscovery, caller-forged
   facts, and emission selected by function/profile are prohibited.

#### Slice and maintainability gates

Before coding, every slice records the atomic ledger facets and local transitions
it implements in the one pipeline; arbitrary-source positive, negative,
composition, and malformed-input witnesses; state/diagnostic behavior; expected
elaboration and generic lowering consequences; complexity and capacity bounds;
new-helper self-classification; superseded authority and duplication to remove;
old-to-new hostile-test mapping; and explicit stop conditions.

No new production helper may remain `Unsupported` when its completed cleanup
tranche exits. A necessary temporary helper tranche is preregistered, can confer
no acceptance/lowering/fact authority, and closes before that tranche exits.
Coverage counts are recorded and every delta is explained, but their sign is
not a gate. Removing false profile authority may honestly increase
`Unsupported`; infrastructure may leave it unchanged; a general transition
may clean an unpredicted function. Any unexplained movement pauses integration
for independent derivation, differential testing, and hostile review.

A slice succeeds by completing its ledger facets, preserving or replacing every
valuable defect regression, removing the source-specific authority in scope,
reducing duplicated logic, passing arbitrary-source composition tests, and
passing both project gates. Template-positive expectations may be removed, but a
reproduced topology, source-redirection, cycle, ownership, effect, diagnostic,
capacity, or failure-atomicity defect remains pinned at the corresponding
general boundary. Protected semantic tests remain owner-gated.

The semantic architecture performs one structural validation and declaration
indexing pass per unit, never rescans a callee body per call, and bounds SCC work
by functions plus resolved call edges. Each body node and CFG edge is visited a
bounded number of times per dataflow iteration; fixed capacities derive from
source/token/AST/call/instantiation counts. Every architecture packet includes a
whole-compiler time/memory regression budget. New modules have one semantic
responsibility. Do not grow the existing monolithic `semantic_reader.wf`;
split by judgment, never by compiler function or source family.

Cleanup is architectural, not file-count churn. Retain proven general
mechanisms, remove dead profile branches and duplicate walkers once their
general replacement is green, and keep data ownership and transition authority
obvious at module boundaries. Split a file when one review can no longer hold
its responsibility and invariants together, using judgment or artifact
boundaries; do not replace the monolith with one-use forwarding facades or
profile-shaped micro-modules. Record file-size and dependency deltas for review,
but no arbitrary line target substitutes for cohesion, readable names, focused
tests, and deletion of obsolete paths.

**No-reborrow decision — RESOLVED (2026-07-18): bounded statement-scoped reborrow,
landed in v0.7.** Growing body semantics surfaced that wfc's own source reborrows
at ~1,062 verified sites — it borrows a sub-place of an exclusive `&uniq` holder
and passes it onward as a call argument (e.g. `frontend_unit_reset` forms
`&uniq 'frontend_reset_tokens deref(analyzed).tokens`), which v0.6 forbade. The
owner-directed investigation (`optimizer-language-research/implementation/
reborrow-investigation/`) weighed keeping the rule and rewriting the code against
relaxing it, on written pros and cons, a Featherweight-Rust reconciliation, a
1,000,000-program model-check (zero aliasing), and a hostile no-alias fact-channel
review (PASS-WITH-CONDITIONS; performance preserved). The owner ratified the narrow
relaxation, and kernel-spec **v0.7** admits the bounded, non-escaping,
statement-scoped child reborrow (OWN-5/6/9/12 + new STOR-5) while deferring
result-transfer and the harder forms. What remains in Phase 2 is production
implementation of an already-landed spec rule, not a new spec decision: implement
the rule compositionally in the unified production checker and generic lowerer.
The rule's actual prerequisites in type/mode classification, affine state,
resolved places, ordinary loans, regions, calls, and flow determine its landing
order; the historical F4 frontier does not.

**Exit gate:** the exact compiler unit has complete body semantics and lowering;
stage-1 wfc matches stage 0 on compiler-subset conformance; the facts-off IR
fixpoint is byte-identical; both project gates pass.

## Phase 3: implement all of v0.8 with facts off

**Entry:** phase 2 exit and a frozen stage 0.

Implement every accepted v0.8 construct that wfc's own source did not require.
The work includes `requires`: AST representation, parsing, entry-check
normalization, body-derived obligation accounting, deterministic diagnostics,
lowering, and report/no-report byte identity. Complete effect checking before
any optimizer fact can affect output.

Run the full v0.8 conformance suite through wfc. Use frozen stage 0 as the
behavior oracle only for its frozen subset. Use specification-derived expected
artifacts or purpose-built reference checkers for constructs stage 0 lacks.
Resolve all fourteen pending source cases so the runner reports zero skips.
Implement executable DIAG-2 elaborated-artifact and DIAG-3 report-schema gates;
annotations do not close those rules. Preserve a conformance case for each
discrepancy. Keep facts disabled and rerun the facts-off self-hosting fixpoint
after the final language slice.

Close every normative source-emittable atomic ledger facet. Require exhaustive
dispatch over every finite grammar/AST, type/mode, operation, and analysis-state
discriminator; reviewed structural-induction and invariant-preservation
arguments parameterized by arbitrary finite state instances; and bounded
generative/model composition as validation rather than proof. No normative
program case may remain `pending`, `xfail`, `Unsupported`, or skipped.
Non-program rules require their executable artifact, runtime, policy, or spec-CI
gate. Zero conformance skips is necessary but not sufficient.

**Exit gate:** wfc accepts and rejects every source-emittable v0.8 case with
zero skips, supports `requires` end to end, emits the DIAG-2 artifact and DIAG-3
reports, completes effect checking, and retains the facts-off byte-identical
fixpoint. Both project gates pass.

## Phase 4: enable fact channels and finish the compiler baseline

**Entry:** phase 3 exit.

For each fact family, freeze its proposition, provenance, producer, consumer,
invalidators, and per-site accounting before enabling it. Add observational
reports first. Require report/no-report output identity. Then enable one family,
run its facts-off control, inspect generated IR and assembly, and obtain hostile
review before shipping that family.

Every family publishes only through the Phase-2 `FactOverlay` verifier and the
same generic lowerer, using one of the closed ledgered record kinds. It may not
mutate `CheckedUnit`, create a second lowering path, pass an unverified fact
directly to emission, or reinterpret a non-elidable check as elidable.

Require wfc to match the reference proof accounting on the bounds corpus and
all codegen-parity pins. Re-run full conformance in both fact modes and
establish a byte-identical facts-on self-hosting fixpoint.

Finish the compiler boundary in this phase: remove intentional compiler-owned
allocation leaks, retain deterministic diagnostics, harden `wfc_compile`, and
provide the trusted launcher shim. The launcher shim does not define public
file-I/O language semantics.

**Exit gate:** wfc passes full v0.8 conformance, codegen parity, per-site proof
accounting, resource cleanup, and byte-identical self-hosting with facts on.
Every fact family has hostile-review evidence. Both project gates pass. wfc is
the production compiler baseline; frozen stage 0 remains an oracle.

## Phase 5: freeze the production landing contract

**Entry:** phase 4 exit. The owner authorization at the top of this file opens
this phase without another scope decision.

Convert the sequential projection of the ratified loan/freeze research and all
24 `seq` rows into one implementation packet. Freeze exact grammar, judgments,
diagnostics, lowering, facts and invalidators, operation rows, trap behavior,
drop order, failure behavior, target mapping, and artifact hashes. Resolve every
open flag that loan/freeze or `seq` consumes.
Honor the selected explicit `copy struct` design; infer no structural copy. Add
that feature only if the frozen `seq` rows require composite Copy in phase 7.
Resolve ALLOC-ERR through a recorded owner ruling before freezing any allocating
row. The recommended rule makes environmental allocation failure a `Result`,
preserves every owned input on the error arm, and keeps capacity overflow as a
programmer-error trap.

Freeze a corpus adapter before production checking. Classify each research AST
case as source-translated, verdict-only abstract semantics, or deferred
concurrency. Define sealed signature stubs with no runtime authority for the
abstract cases, map research rule IDs to production IDs, and exclude trusted
`body=None` entries from source-conformance credit.

Define the five acceptance legs against one frozen artifact and a named Linux
x86-64 host, compiler, allocator, reference-library version, and workload.
Preregister model bounds, differential and fuzz seeds, run budgets, sanitizers,
fault schedules, review severities, sample counts, statistical bands, teardown,
and resource ceilings. Provision that runner before phase 5 exits; Apple M4 dry
runs provide design evidence but do not satisfy the deploy gate.

Keep `prototype/democ` frozen. Use the 15-rule research checker as the
independent loan/freeze oracle; wfc owns production semantics. Record this
division in the design tree and specification derivation ledger. Hostile review
must attack the packet before implementation begins.

**Exit gate:** the packet contains no open semantic flag, no dependency on a
later component, an owner-ratified ALLOC-ERR policy, a complete corpus adapter,
all 24 `seq` rows, executable pass and fail bands, a provisioned deploy runner,
and a passing hostile landing review. Both project gates pass.

## Phase 6: land sequential loan/freeze without a container

**Entry:** phase 5 exit.

Land rules R1 through R13 and the sequential part of R15, plus confined
borrow-carrying values, across the production specification, wfc parser and AST,
checker, diagnostics, lowering, conformance suite, derivation ledger, and
teaching material. Defer R14 and R15's concurrent-invocation clause with the
concurrency layer. Keep this change free of container implementation.

Run the 88 non-parallel research cases through the frozen adapter and all nine
mutation tests against the independent checker and wfc. Route source-translated
cases through the parser; route abstract cases through the verdict-only semantic
entry. Add production conformance cases for rule boundaries, invalidations,
early exits, effect interactions, and malformed internal state. Pin
unchanged-source IR and diagnostics. Obtain hostile review of the facts the new
judgment makes available to lowering. Preserve the nine parallel cases for the
later R14 landing.

**Exit gate:** wfc and the independent checker agree on all 88 sequential cases;
all applicable mutants fail as intended; full conformance and codegen parity
pass; no safety check has weakened; both project gates pass.

## Phase 7: land `seq` as the first sealed component

**Entry:** phase 6 exit and a phase-5 packet with no unresolved `seq` flag.

Implement `seq<T, N>` with inline capacity `N` as one vertical production
slice. Cover affine elements, uninitialized spare capacity, spill and growth,
take and replace, insert and remove, drain, slices, fact production and
invalidation, capacity overflow, allocation failure, drop order, and teardown.
Land its specification rows, wfc recognition and lowering, sealed
implementation, conformance cases, diagnostics, code-shape pins, and teaching
entry in the same phase.

Run all five acceptance legs across the exact 24-row surface on the same frozen
bytes:

1. differential operation testing against the reference implementation;
2. bounded state and ownership modeling;
3. sanitizer-backed fuzz and fault-injection soak;
4. hostile safety, ABI, drop, and proof-fact review;
5. Linux x86-64 performance and assembly checks, including the preregistered
   wfc-shaped tiny-vector band against SmallVec. A separate layout fixture must
   embed inline `seq` in an ordinary struct and exercise insert, remove, and
   drain. Phase 5 must define the future pool-slot fact-flow check without
   requiring a production pool implementation in this phase.

Do not migrate wfc from its structure-of-arrays tapes during this phase. Those
tapes provide the control for any later migration experiment.

**Exit gate:** all five bounded legs pass on the frozen artifact with zero
observed uninitialized reads, leaks, double drops, stale loans, missed traps, or
failure-atomicity divergences within the preregistered runs. The performance and
code-shape bands pass; both project gates pass. Stop and report completion of
the authorized seven-phase scope.

## Execution cursor

Phase 2 is active, but production implementation is frozen at commit
`c5ef95a` pending the recovery reviews below. The main worktree is clean and
both project gates were green at that commit. This cursor authorizes
documentation, read-only inventory, architecture design, and hostile review
only. It does not authorize compiler implementation, compiler-test changes,
source migration, or lowering work before the explicit implementation entrance
gate.

The current pipeline reports 655 functions: 166 provisionally clean, 489
Unsupported, and zero through its currently implemented semantic-reject paths.
`Unsupported` makes no legality claim. Self-parse is deterministic at
1,783,808 source bytes, 360,726 tokens, and 179,036 unique-head AST nodes. The
parser census is 5,065 regionful calls: 497 explicit and 4,568 staged omissions.
LLVM profile emission still covers 15 functions.

The exact compiler unit is already known not to satisfy all accepted source and
whole-unit rules:

- TYPE-5 requires explicit type, region, and const arguments, so the 4,568
  staged omissions are source-conformance debt, not legal calls and not
  permission for inference or normalization.
- `sources.txt` contains no `fn main`, while FN-7 requires exactly one.
  Stage 0 currently checks only at most one and is not authoritative for that
  discrepancy. The recovery must either make the unit a canonical program or
  bring an exact language/toolchain alternative through the owner-gated
  specification protocol. There is no inferred compiler-only exemption.

The durable implementation also contains useful foundations: lexing and parsing
for the current compiler subset, structural AST validation, transactional
diagnostics, exact byte-name handling, atomic global/type/member/function symbol
indexing, type-resolution pieces, explicit call-region AST retention, generic
pieces of block/match/loop walking, place/field/index resolution, output
capacity handling, checked-operation LLVM primitives, codegen corpus gates, and
the repaired entry-block placement of fixed stack slots. These pieces are not
presumed correct merely because they predate this correction; the inventory
classifies and tests them before retention.

The later body-semantics route became source-shaped. Ten bounded F4 slices added
exact signature and whole-body/subtree recognizers for particular reborrow
writers, byte emitters, fixed-literal wrappers, numeric composition, probes, and
the guarded span loop. Hostile review found real topology, source-anchoring,
effect, and failure-atomicity defects, and some local helpers may be reusable.
But the final authority remained tied to current source shapes. The last span
slice made one existing function provisionally clean while adding eleven
Unsupported compiler helpers, changing 644 / 165 / 479 into
655 / 166 / 489. It also proved source-specific bounds, overflow, and
termination properties while facts were disabled and checked traps could remain.
That negative leverage and conflation of checked acceptance with source-specific
proof are unacceptable.

The exact-profile route is suspended. The uncommitted
`byte_tape_emit_span_probe` implementation remains isolated outside main and
must not be merged as another profile. Completed slice narratives remain in the
append-only decision log as evidence; they no longer select work. Cleanup is not
a blind rollback: it separates general mechanisms and valuable defect
regressions from source-specific authority, then replaces that authority through
the reviewed pipeline.

### Phase 2 recovery sequence

0. **Land the governing correction only.** Reconcile `THE-PLAN.md`, the owner
   directive, toolchain design memory and rejected alternative, and active
   compiler architecture documentation. Change no compiler implementation,
   compiler test, compiler source unit, numbered specification, conformance
   verdict, oracle digest, or protected reference test. Run both project gates
   and stop.
1. **Produce a read-only semantic, whole-unit, source, and lowering debt
   inventory.** Trace every semantic state structure, phase-validity token,
   status constructor/publication site, elaboration/fact record, body or subtree
   validator, exact profile, LLVM kind/emitter, `sources.txt` ordering
   dependency, build/test integration, and call path into body `CLEAN`, unit
   acceptance, or LLVM publication. For every module/helper/test, record
   responsibility, callers, line count, current status, complexity, and
   dependence on names, ordinals, hard-coded project nominal spellings,
   signatures as fingerprints, constants, fixed child/list counts, statement
   order, callee sequences, or complete bodies/subtrees. Classify each item as
   general machinery to retain, general machinery to repair, source-specific
   authority to remove, or fixture/measurement code with no authority. Map every
   reproduced defect test to the general invariant that must preserve it.
   Separately enumerate known and newly discovered source/whole-unit
   discrepancies, including TYPE-5 omissions and FN-7.
2. **Write the complete architecture packet, still without compiler edits.**
   Instantiate every boundary and state required by the production contract:
   validated AST handle, declaration/signature table, concrete-instantiation
   graph, one syntax-directed body checker, ownership/loan transfer and joins,
   EFF-2 fold, staged body results, atomic `CheckedUnit`, deterministic
   diagnostics, facet-ledger generator, verified optional `FactOverlay` with a
   canonical facts-off empty form, and unique generic lowerer. Map every
   v0.8 category, including constructs pending for Phase 3. State fixed-capacity
   formulas, time/memory budgets, dataflow convergence, analysis-schedule
   independence for a fixed canonical source/declaration order, cache identity,
   publication tokens, and failure atomicity.
   Build a reviewed specification-dependency graph for implementation order.
   Historical F labels and the compiler dependency frontier have no ordering
   force. The ownership foundation—type/mode and copy/affine classification,
   resolved places, live/moved state, lexical regions, overlap, and ordinary
   loans—precedes general calls and statement-scoped reborrow.
3. **Pre-register the first cleanup tranche.** Choose the smallest coherent set
   of atomic facets and pipeline infrastructure justified by the dependency
   graph, not a target function. State arbitrary-source and malformed witnesses,
   bounded/generative composition, diagnostic scheduling tests, expected
   elaboration, source migrations if any, capacity/performance budgets, helpers
   and their closure schedule, exact production branches to remove, old-to-new
   defect-test mappings, census observations, and stop conditions. Its first
   authority change must quarantine every remaining legacy profile behind
   `LegacyProfileResult`, disable `CheckedUnit` publication until those paths are
   removed, and forbid new callers; subsequent removals may then proceed in
   reviewed tranches. If FN-7 resolution would change the specification or
   toolchain contract, present the exact alternatives and owner-gated delta
   instead of assuming one.
4. **Implement only the explicitly approved tranche; replace, never stack.**
   Extend the one pipeline and remove the source-specific whole-function and
   subtree authority in scope. A temporary helper tranche is allowed only when
   preregistered, confers no semantic/lowering/fact authority, and ends with all
   its helpers clean. Coverage movement is recorded but never a pass criterion.
   Removing false authority may increase Unsupported; infrastructure may leave
   it unchanged; a general transition may clean an unpredicted function.
   Unexplained movement stops for independent derivation and hostile review.
5. **Make the compiler source unit canonical before claiming semantic
   completion.** Migrate every TYPE-5-required call argument explicitly from
   resolved signatures and enforce a mechanically checked zero-omission gate.
   Resolve FN-7 under the accepted specification or a separately approved
   spec/toolchain decision. Audit the whole unit for every already-implemented
   source and whole-unit rule. Stage-0 disagreement is recorded as oracle debt,
   never used to relax wfc or source.
6. **Complete Phase 2 semantic facets in specification-dependency order.**
   Repeat reviewed tranches through one body checker and one atomic unit
   finalizer. Every completed tranche removes the superseded authority in its
   scope and preserves each valuable defect regression at a general boundary.
   Body-semantic recovery exits only when the compiler unit is canonical,
   every required facet is complete, every body and whole-unit rule passes, and
   one compiler-subset-certified `CheckedUnit` publishes atomically.
7. **Replace LLVM profile emission with generic lowering.** Do not extend a
   source-specific emitter during semantic recovery. Before lowering resumes,
   prove each existing primitive emitter generic at node/type/operation level or
   classify it as removal debt. The first lowering tranche establishes the
   unique `CheckedUnit` consumer and removes the whole-function emitter
   branches it supersedes. Subsequent tranches extend only that path, covering
   layouts/ABI/mangling, constants and instantiations, generic CFG construction,
   retained arithmetic/bounds/allocation checks without poison or undefined
   behavior, traps/reports, ownership-derived cleanup, strict operation
   semantics, deterministic ordering/capacity, and failure atomicity.
8. **Resume conformance and self-hosting gates.** After the generic lowerer emits
   the entire compiler-subset-certified unit, run stage-1 subset conformance and the
   facts-off byte-identical fixpoint exactly as specified above. Body `CLEAN`,
   unit acceptance, lowering authority, and optimizer facts remain four
   separate gates.

**Inventory checkpoint:** after recovery step 1, present the debt and
source-conformance inventory to the owner. No compiler or compiler-test edit is
authorized by that review.

**Implementation entrance gate:** after read-only recovery steps 2 and 3,
present the complete architecture packet, generated facet-ledger design,
specification-dependency graph, source-conformance resolution, and
preregistered first tranche to the owner. No compiler implementation, compiler
test, or source-unit file may change until the owner explicitly approves that
packet in writing. Approval of this plan correction is not approval to begin
cleanup. A red gate, profile or subtree fingerprint, caller-side callee-body
inspection, authority outside the unique publication paths, unexplained verdict,
unclosed helper tranche, source-specific lowering condition, or need to weaken a
contract stops recovery.

## Work outside the seven-phase scope

- `table`, pool, arena, other sealed components, checked-source libraries,
  catalog cards, and public I/O or FFI wait for a later owner-directed roadmap.
- Concurrency waits for a later roadmap. Its memory model, synchronization
  effect, sharing rules, target lowering, per-form models, and acceptance rows
  must land before any thread, queue, scheduler, or reclamation component.
  Retain the fixed-fan-out v1 choice. Reconsider the dynamic loop-spawn capture
  carve-out after the concurrency layer exists or when a named scenario needs
  runtime-count workers with shared outer borrows; require its own hostile
  review. Fold AMD-7 and AMD-8 into the ratified rules at that landing. Place
  MM-1 through MM-6 and MM-10 in the kernel specification, and keep the writer
  manual limited to MM-0 and MM-7.
- Fold AMD-6 into the ratified rules when a production phase first consumes
  branded endpoints. That obligation does not wait for concurrency if an
  earlier branded component needs it.
- A wfc migration to `seq` remains a measured experiment after phase 7. The
  current structure-of-arrays compiler stays as its control.
- The two completed shipped-library protocols stay frozen. Do not tune them or
  add a third target to rescue or enlarge their claims.
- Surface ergonomics and deferred proof-accounting promotion work require their
  recorded triggers and a later plan amendment.

## Process gates

- Run `make check` and `make -C compiler check` before and after each completed
  slice. Do not weaken a check to restore green.
- Give each completed step one commit and one appended decision-log line.
- Keep semantics, lowering, conformance, and measurement artifacts tied to the
  same frozen source hashes. Treat an unexplained difference as a defect.
- Require hostile review before shipping any new fact channel or production
  safety judgment. A green automated gate does not substitute for that review.
- Keep all repository artifacts in English. Keep `AGENTS.md` and `CLAUDE.md`
  byte-identical.
- Update the relevant MCTS node and its `.alt/` in the same change as any design
  redecision. Preserve rejected designs with paired reasons.
- Stop a phase on a red gate, an unresolved semantic flag, an unsound model, a
  failed mutation, or a missed performance band. Repair or reject the design
  before expanding scope.

## Evidence pointers

- Compiler state and architecture: `compiler/README.md`, `compiler/sources.txt`,
  `compiler/test_self_parse.py`, `compiler/test_semantic_unit.py`, and
  `compiler/test_llvm_supported.py`.
- Toolchain decision: `mcts_mem/whitefoot/toolchain.md` and
  its rejected alternatives under `mcts_mem/whitefoot/toolchain.alt/`.
- Proof contracts and reviews:
  `optimizer-language-research/implementation/requires-check-accounting-REVIEW.md`
  and `mcts_mem/whitefoot/fact-channels.md`.
- Capability landing evidence:
  `optimizer-language-research/implementation/systems-performance-coverage/DESIGN-DOSSIER.md`,
  `m1-loan-judgment/RULES-RATIFIED.md`, `m1-loan-judgment/programs.json`,
  `m2-spec-mass/KERNEL-DELTAS-DRAFT.md`, `m2-spec-mass/optables.md`, and
  `m3a-kernel-dryrun/RESULTS.md` under that directory.
- Completed shipped-library evidence: `experiments/default-floor/RESULTS.md`.

Evidence supports a gate. It does not advance the execution cursor.
