# Candidate B-Strata Decisive Research and Landing Plan

Date: 2026-07-15

Status: controlling plan for the owner-selected B-Strata-only research track.
It supersedes the exhausted authorization boundary in
`CANDIDATE-B-ELEGANT-DESIGN-PLAN.md` without rewriting that completed result.

## 1. Owner decision and forced outcome

The owner has selected B-Strata as the only capability architecture to pursue.
The project will not pivot to Candidate C and will not develop B-Graphs as a
competing architecture. Prior alternatives remain historical evidence and
falsifiers only.

The objective is no longer another comparative paper result. It is:

> Determine whether one normalized, project-independent B-Strata core can
> close the fourteen frozen systems-performance demands safely and within
> their performance bands, using either the reference representation or a
> safe substitute, then test the decisive claims far enough to return a forced
> YES or NO verdict.

The final result must be exactly one of:

- `STRATA-YES`: one coherent core closes all fourteen demands through selected
  reference or substitute routes, survives hostile safety and erasure review,
  has an implementable deterministic checker and lowering, and meets every
  frozen performance and resource band in the bounded evidence; or
- `STRATA-NO`: a named irreducible conflict, unavoidable runtime or structural
  tax, unsafe authority cycle, unbounded rule interaction, or infeasible
  checker/lowering prevents B-Strata from meeting the project constraints.

`REVISE`, `OPEN`, and `UNKNOWN` may describe intermediate work. They are not
permitted as the final disposition. A NO verdict must explain why neither
another local B-Strata repair nor an admissible substitute route can remove the
blocker without violating a binding constraint. Failure to reproduce one
source project's exact data structure is not a NO argument. Lack of elapsed
research time alone is not a NO argument.

## 2. What remains fixed

### 2.1 Performance-first objective

The problem remains missing safe expressiveness that would otherwise force
initialization, zeroing, copies, relocation, tags, runtime metadata,
allocation, indirection, retained checks, stronger atomics or fences, larger
scans, or an unavailable native representation. Isolation is not the goal.

Every source-level check remains present unless a machine-verified proof
discharges it. No route may weaken safety to preserve source performance.

### 2.2 Frozen demand corpus and reference baselines

The decisive corpus remains exactly the previously audited fourteen demand
cases. Their source operations define reference baselines and stress cases;
they do not require the final route to preserve the same internal data
structure or algorithm:

1. Hashbrown lookup;
2. Hashbrown vacant insertion;
3. Hashbrown replacement;
4. Hashbrown removal;
5. Hashbrown rehash;
6. mimalloc small allocation, including frozen cold outcomes;
7. mimalloc local free through final page disposition;
8. mimalloc remote free, collection, and page disposition;
9. SQLite insertion and split;
10. SQLite deletion and balance;
11. SQLite rollback;
12. Crossbeam protected load;
13. Crossbeam retirement; and
14. Crossbeam collection.

The pinned source identities and exact source anchors in
`CANDIDATE-B-MULTIPROJECT-AUDIT.md` remain controlling for the reference
behavior, workload, cost, and source-route claims. Definition chasing is
allowed only inside those complete reference routes. No fifth project or
fifteenth demand may be added merely to postpone the verdict.

### 2.3 Demand closure and substitution

The object of coverage is the useful systems demand, not the historical
implementation. For each demand, the cross-family lock must freeze before
evidence is observed:

- the externally observable behavior relevant to the consumer, including
  failure and rare outcomes;
- the safety, ordering, and progress requirements that the consumer depends on;
- the workload, corpus, target, and reference baseline;
- one primary performance endpoint and its non-inferiority margin; and
- hard resource ceilings for the dimensions that can invalidate an apparent
  speed win, including memory, tail latency, code size, scalability,
  synchronization, startup, and asymptotic work where relevant.

A demand may be closed by either route kind:

- `REFERENCE`: preserve the reference algorithm and representation closely
  enough to claim native structural parity; or
- `SUBSTITUTE`: use a different data structure, reclamation strategy, or
  algorithm while preserving the frozen demand contract and passing its
  quantitative performance and resource bands.

The final minimum is the union of semantic rules required by one passing route
for every demand, not the union needed to reproduce every reference topology.
A rule needed only by a reference implementation must be removed when selected
substitute routes make it unnecessary. Conversely, a substitute is not useful
merely because it is different: it enters the bounded route frontier only when
it names the core rule it is expected to delete or merge, or when it closes a
demand whose reference route is blocked without adding project-specific
authority.

Reference-route failure remains diagnostic evidence. It becomes
`STRATA-NO` only when the frozen demand has no safe B-Strata-expressible route
within its performance and resource bands, or when an independent core
soundness or implementability blocker applies to every route. Substitute
routes, workloads, margins, and ceilings must be frozen before measurement;
results may not redefine the demand. This is a bounded minimization exercise,
not an invitation to enumerate arbitrary containers or algorithms.

### 2.4 Historical evidence

The completed `B-REVISE` comparison is evidence, not an active alternative
contest. In particular:

- the six previously labeled paper-closed B-Strata routes are now only
  `PAPER-ROUTED`; they receive no soundness, erasure, implementation,
  code-shape, or performance credit;
- the eight open routes are not presumed impossible;
- the validator-to-liveness correction remains binding;
- hot-subpath credit remains prohibited;
- B-Forms demonstrates special-form growth risk; and
- B-Graphs demonstrates the boundary at which local protocol descriptions
  become arbitrary writer invariants.

Those controls may falsify a proposed B-Strata rule. They receive no parallel
design or implementation budget.

## 3. What B-Strata must become

The existing eight strata are analytical jobs, not eight preselected language
keywords, runtime objects, or checker subsystems. The first task is to
normalize them into the smallest coherent semantic core.

Every rule must preserve one common resource-conservation invariant:

> Each live physical root owns one affine release authority and one current
> checked partition whose nonoverlapping leaves completely cover the root.
> Every leaf is exactly vacant bytes, initialized bytes, one live typed value,
> one transit/progress obligation, or one disposition obligation. Loans,
> observations, and deferred rights are a separate obligation set that escrows
> incompatible move, reclassification, or release authority while retaining
> the original root and nonwrapping version. Release is legal only after the
> partition is reunited and every subowner and obligation has ended; released
> is a terminal root state, not a footprint state. Metadata, addresses,
> validators, and ordinary predicates create no owner or authority. Abort need
> not run cleanup, but no invalid read, duplicate disposition, race, or
> premature release is permitted before abort.

The normalized design must answer, without circular premises:

1. what physical authority exists over a root and its disjoint places;
2. what establishes a layout or live owner in a place;
3. what checked transition consumes and produces each owner and obligation;
4. how partial progress, repair-required state, and poison restrict later use;
5. how every borrow, result, and fact retains exact physical-root provenance;
6. how outstanding obligations become executable, finite disposition;
7. how checked facts are produced, transferred, invalidated, and erased; and
8. how atomic custody, protected observation, retirement, and final
   disposition compose without embedding one reclamation policy.

The result may have fewer semantic primitives than the eight analytical
strata. A stratum that is derived must be shown as a deterministic composition.
A stratum that remains primitive must have a deletion witness showing why the
other rules cannot derive it without safety loss or a protected structural
delta.

### 3.1 First normalization hypothesis

The first hypothesis to try to falsify is a three-judgment kernel:

- `K1 ROOTED-PLACE`: generative physical roots; exact fields, checked indices,
  and byte ranges; layout versions; split/join; and disjointness. It carries no
  liveness, owner, container identity, or logical handle authority.
- `K2 SEALED-STATE`: sealed certificates with explicit affine, shared, or
  scoped multiplicity, bound to exact roots and versions for vacant/live
  layout, roles, strict children, loans, observations, and deferred
  obligations. Its sum/product/span/classifier descriptions are closed and
  accept no ordinary predicate, proof term, validator, or cleanup program.
- `K3 LINEAR-STEP`: fixed initialize, take, replace, swap, relocate,
  reclassify, borrow, destroy, exact event, commit, outcome, and structured
  close transitions. It accepts no writer-defined state relation, global
  invariant, or termination proof.

This is a falsifiable starting hypothesis, not a primitive-count quota. A
fourth primitive is admissible only with an irreducibility witness and need in
at least two independent frozen projects.

Under this hypothesis, physical-root footprints derive from K1 plus K2 loans;
focus/progress/repair are sealed K2 states consumed by K3; executable
disposition is a structural K2 fold whose terminal actions are K3 leaves; and
optimizer facts are read-only projections from accepted K1-K3 derivations.
Facts never feed back into safety authority. Atomic transfer is a K3 exact
event, while observation, retirement, and quiescence remain the first hard K2
authority-production test.

Phase 1 must maintain an authority-origin ledger with no cycles:

- roots originate only in exact allocation or acquisition leaves, which also
  create affine release authority and vacant bytes;
- initialized bytes originate only in checked writes or exact external reads;
- typed state certificates originate only in a fixed generic validity/adoption
  rule over existing bytes or a valid predecessor transition;
- metadata may select a role maintained by a certificate but may not create
  the role, typed validity, ownership, or a strict child edge;
- borrows originate only in a live state over an exact place;
- observations originate only in an exact begin-observe or pin event;
- retirement rights originate only in a successful unlink or custody-transfer
  event;
- quiescence originates only in complete checked domain validation; and
- optimizer facts originate only as projections and create no authority.

## 4. Boundary between generic semantics and exact leaves

B-Strata is not required to manufacture machine or external semantics from
ordinary memory rules. It may consume exact fixed leaves for allocation,
release, atomics, threads, file or device events, target operations, and
foreign callbacks. That boundary does not permit a loophole.

Every exact leaf must state:

- the concrete event and target/platform scope;
- argument and result ownership;
- physical-root and borrow provenance;
- effects, traps, failure, cancellation, and partial completion;
- atomic order or external ordering where relevant;
- facts produced and exact invalidators;
- runtime and code-shape cost; and
- why the row is a machine/external semantic rather than a disguised
  Hashbrown, allocator, B-tree, pager, or reclamation operation.

Leaf outputs are closed by kind:

- allocation or acquisition may produce only a generative root, its affine
  release authority, vacant bytes, and an exact failure result;
- external reads may produce only initialized bytes under the supplied root or
  an exact failure/partial-completion result;
- fixed generic typed adoption may establish only the layout validity that its
  closed rule checks; a trusted foreign adoption is an explicit TCB assumption
  and earns no ordinary-library closure credit;
- atomics and fences may produce exact event witnesses and the declared
  success/failure custody transition, but no unrelated state fact;
- release consumes a reunited root and complete release authority;
- callable packing and invocation may perform only the closed erased-callable
  state transitions; and
- target or device leaves may produce only their enumerated low-level event
  results and effects.

No leaf may directly mint `Quiescent`, `Stable`, `RepairComplete`, a live-role
certificate, a strict child edge, or any fact not implied by its concrete
machine event. A SQLite page reinitializer is ordinary checked byte parsing and
transition code, not a high-level trusted leaf.

A high-level project operation hidden behind an exact-leaf name does not close
a route. Conversely, an irreducible exact machine event is not counted as a
new B-Strata topology rule merely because one frozen project needs it.

## 5. Admission test for every semantic rule

Every primitive or derived rule must have one ledger row with:

1. exact inputs, outputs, and linearity;
2. its sole authority producer;
3. normal, recoverable, abort, and abandonment behavior;
4. physical roots, versions, byte ranges, and invalidators;
5. executable disposition of every outstanding owner;
6. a deterministic local checking procedure;
7. a static-erasure argument;
8. the demand routes that need it;
9. a deletion witness;
10. hostile negative examples;
11. interaction points with every other primitive; and
12. an observable falsifier.

A rule is rejected if any of the following holds:

- its semantics depend on a project, container, algorithm, API, path, or
  reclamation-policy identity;
- it accepts a writer predicate, proof, invariant, termination argument, or
  cleanup program as new safety authority;
- metadata, validation, a guard value, or compiler recognition can forge a
  live owner, borrow, quiescence fact, release right, or optimizer fact;
- it requires runtime state not already selected by the program's native
  representation;
- it adds work or code to an unrelated weaker route;
- its checker requires open-ended theorem proving or unbounded graph search;
- its interactions grow as operation-specific cross-products; or
- it closes only a hot subpath while leaving a rare or failure path undefined.

## 6. Work phases and mandatory gates

There is no arbitrary numerical quota on semantic corrections. Any change to a
primitive, authority producer, rule semantics, leaf contract, route semantics,
or state model reopens the earliest affected gate and invalidates every
downstream proof, prototype, and measurement. A correction is admissible only
when it remains project-independent, passes Section 5, and preserves a finite
closed core with bounded interactions. Repeated core growth is evidence against
minimality and elegance, but is not by itself a `STRATA-NO` proof. A NO verdict
requires the irreducible witness defined in Section 1. Mechanical verifier,
model, checker, or lowerer defects may be fixed when semantics and routes remain
unchanged; affected evidence must still be regenerated.

### Phase 0: durability and baseline

Produce and commit this plan, the owner ruling, synchronized active status,
the MCTS-Mem decision, the status verifier, and one decision-log entry. Preserve
all pre-existing user worktree changes. Both repository verification gates
must be green before and after the step.

Gate: `STRATA-PLAN-LOCKED`.

### Phase 1: normalize the semantic core

Front-load four verdict-forcing definitions before expanding the rest of the
document:

1. a finite liveness-authority judgment in which only vacant creation, exact
   adoption leaves, and sealed owner transitions can establish or change live
   layout; metadata decoders can select maintained authority but cannot mint
   owners, typed validity, or strict child edges; and
2. a finite quiescence-producer judgment that must derive safe final release
   for both audited mimalloc and Crossbeam observer protocols from their native
   events without a per-load or per-object tax. One theorem schema must prove
   complete pre-cutoff observer coverage, observer exit or cutoff advance,
   registration/scan race closure, exclusion of new access to the retired
   target, required ordering, generative-root protection against ABA/reuse, and
   stalled-observer blocking. It accepts no project/policy identity, callback
   predicate, writer proof, or quiescence-producing leaf;
3. an erased affine one-shot disposition judgment that preserves Crossbeam's
   inline representation, exact effects, environment provenance, cross-thread
   use, and consume-before-invoke behavior without a hidden allocation, count,
   or second owner box; and
4. an exact external-event and repair boundary showing that SQLite's pager,
   VFS/WAL, reinitializer, and poison route decomposes into generic sealed
   repair/poison authority plus a finite low-level leaf interface. Phase 1 must
   expose a minimal falsifying witness and prohibit hidden database operations
   or arbitrary foreign contracts; Phase 2 performs the complete leaf
   enumeration and route closure.

If a reference boundary requires a project/policy identity, an arbitrary writer
invariant, a hidden trusted assertion, or an extra protected-path event, reject
that reference route before spending time completing lower-risk exposition.
This is an immediate `STRATA-NO` only when the underlying frozen demand has no
admissible substitute route or the same blocker follows from the generic core
and therefore applies to every route.

Each of the four boundaries must already provide, for every relevant reference
stress case, one machine-checkable positive derivation, one single-fault
authority-forgery rejection, and the exact native event manifest. These are
early reference-pressure falsifiers, not proof that the same topology belongs
in the final minimum and not substitutes for the general proof and complete
demand work in later phases.

Produce:

- `CANDIDATE-B-STRATA-CORE.md`, defining the exact judgments, authority flow,
  primitive inventory, derived strata, and illustrative reductions;
- `CANDIDATE-B-STRATA-RULES.tsv`, one complete admission row per primitive and
  derived rule;
- `CANDIDATE-B-STRATA-NORMALIZATION.tsv`, mapping every old BS-1 through BS-8
  statement to exactly one primitive or one acyclic derivation;
- `CANDIDATE-B-STRATA-AUTHORITY-ORIGINS.tsv`, recording every authority kind,
  sole producer class, consumers, transfers, and invalidators;
- `CANDIDATE-B-STRATA-LINEAGE.tsv`, giving every primitive output a consumed
  input lineage or one enumerated true origin, with exact failure equations;
- an interaction matrix showing which rule pairs can exchange authority and
  why no unchecked cross-product exists; and
- `CANDIDATE-B-STRATA-BOUNDARY-PROOFS.tsv`, containing the eight required
  positive derivations, single-fault rejections, and native event manifests;
- a deterministic verifier for inventory, required fields, authority-source
  uniqueness, deletion witnesses, and interaction coverage.

The core document must use one representation-independent state model for
vacant/live layouts, owner obligations, progress, repair, facts, and concurrent
custody. It must not add source syntax or choose a production encoding yet.
At this gate the normalized core is a finite working upper bound derived under
reference-route pressure. It does not earn the word *minimal* until the
substitution frontier has challenged reference-only rules and the selected
passing routes have allowed those rules to be deleted.

Gate: `STRATA-CORE-PASS` or `STRATA-NO`.

`STRATA-CORE-PASS` requires a finite deterministic working core with no
circular authority, no hidden runtime state, and a syntax-directed checking
algorithm for every rule. The plan must state a termination measure and worst-case bound
in program size, monomorphized instance count, and state arity; acceptance may
not depend on solver timeout, heuristic success, backtracking, or unbounded
search. The verifier must reject a cycle in the complete authority-origin graph,
not merely uncovered pairwise interactions, and every K2 constructor must fix
its affine, shared, or scoped multiplicity.

Hostile review may repair omitted or inconsistent definitions before the gate,
but every correction must update the full rule, lineage, authority-origin, and
interaction ledgers. At the gate, every required authority must have either a
complete derivation or a minimal missing-authority witness over the
exhaustively enumerated frozen grammar; it may not remain an open expository
task.

After `STRATA-CORE-PASS`, no primitive or stratum may be silently added. A
proposed addition reopens the core gate, must pass the full admission ledger
and interaction matrix, invalidates downstream evidence, and counts explicitly
against the design's minimality and interaction-growth claims. Deleting or
merging a rule after a substitute succeeds also reopens the core gate, but it
is the intended path from this working upper bound to the final minimum.

### Phase 2: freeze demands and close the bounded route frontier

Produce `CANDIDATE-B-STRATA-CROSS-FAMILY-LOCK.md` before constructing a
candidate model or prototype. Its demand table has exactly fourteen rows. Each
row freezes the consumer-visible contract, outcome partition, required safety,
ordering and progress properties, reference source/function/corpus hashes,
correctness digest, workload, target triple, compiler and flags, allocator,
endpoint and aggregation, ratio direction, sample or sequential-stop rule,
balanced run order, exclusions, confidence method and level, non-inferiority
margin, multiplicity treatment, hard resource ceilings, and any preselected
structural/event-count endpoint for a cold or rare path. Results may not rewrite
this lock.

The reference algorithm, owner/drop account, roots, synchronization and
external events remain frozen as the comparison baseline and as one diagnostic
route. They are not a mandatory implementation contract. For every admitted
`REFERENCE` or `SUBSTITUTE` route, record:

- the demand IDs and complete normal, recoverable, abort, abandonment, and
  rare-path contract behavior it covers;
- the chosen data structure, algorithm, progress strategy, and exact mapping
  to the frozen consumer contract;
- every owner and obligation before and after each logical commit;
- every root, range, borrow, fact, invalidator, and re-root prohibition;
- exact cleanup or repair progression;
- exact atomic or external events;
- every required core rule and exact leaf;
- static state expected to erase;
- its preregistered structural, performance, and resource falsifiers; and
- one executable or mechanically checkable semantic falsifier.

Maintain two separate ledgers:

- `CANDIDATE-B-STRATA-DEMANDS.tsv` has exactly the fourteen frozen demand rows
  and stable `outcome_id` rows for every required normal, precommit failure,
  retry, partial-progress, abandonment, abort, and rare outcome; and
- `CANDIDATE-B-STRATA-ROUTES.tsv` has a finite set of route rows, with
  `route_kind=REFERENCE|SUBSTITUTE`, a declared core-rule set, a paper status,
  and the specific capability-deletion or blocked-demand hypothesis that
  justified admitting the route.

A demand is paper-closed when at least one route closes all of its frozen
outcomes. An open reference route does not block a closed substitute route. A
verifier must prove that every demand and outcome has at least one complete
route, every reference source anchor appears in its baseline exactly once, and
every route maps to frozen demands without silently weakening their contracts.

Keep the route frontier bounded. A substitute may enter only to delete or merge
a named core rule, or to solve a demand whose reference route is blocked. It
must use the same project-independent core and must not hide a project operation
inside an exact leaf. Stop adding routes once every demand has a paper-closed
route and every retained core rule has either a route-level necessity witness
or a concrete deletion challenger. Freeze all surviving routes before Phase 3;
adding a post-result route invalidates downstream evidence and reopens this
gate.

A blocked route may reopen normalization only when the missing relation remains
necessary after considering substitutes, is required by at least two
independent projects, and still passes Section 5. Multiple demands in one
project may support a derived composition or a genuine exact machine,
external, or callable leaf, but cannot justify a new core authority. A
project-specific patch is prohibited. A blocked reference route alone never
yields `STRATA-NO`.

Produce a separate `CANDIDATE-B-STRATA-LEAVES.tsv` for every allocation,
release, atomic, external, target, and callable leaf. No route may receive
closure credit merely because a missing high-level operation was moved into
that ledger.

Map the working core back to all fifteen existing performance-demand families
as a non-regression check. This reuses the existing ledger and opens no new
source-audit scope. Compute the core union for every surviving combination of
one route per demand and expose which rules become removable under each
combination; final route selection waits for measured evidence.

Gate: `STRATA-PAPER-YES` or `STRATA-NO`.

`STRATA-PAPER-YES` requires fourteen frozen demand contracts, at least one
complete paper-closed route for every demand and outcome, a frozen finite route
frontier, and all fifteen existing demand families mapped to an exact K1/K2/K3
derivation or legitimate exact leaf with no new semantic gap or unaccounted
runtime tax. This mapping earns routing/non-regression credit only; it does not
close the 340 exact dense obligations or exact D-2/P-1. An `OPEN` reference
route is permitted when a substitute closes the demand. An uncovered demand,
arbitrary-authority dependency, hidden high-level exact leaf, or unbounded
route/core interaction blocks the gate.

### Phase 3: hostile safety, erasure, and implementability model

After `STRATA-PAPER-YES`, define a general operational semantics and the
concurrent memory model for every admitted authority-bearing rule and leaf.
Give independently reviewed proofs of:

- type/state preservation and the common resource-conservation invariant;
- absence of uninitialized typed reads;
- exact-once owner traffic, no double disposition, and overlay exclusivity;
- non-escapable progress, repair-required, and poisoned states;
- physical-root, nonwrapping-version, and footprint provenance;
- safe disposition across callbacks, divergence, abort, and nested resources;
- fact production, invalidation, speculation, and facts-off semantic identity;
- race freedom under the admitted publication/interference rules;
- custody transfer only on the declared successful atomic event; and
- no retired-root release before the shared quiescence theorem applies.

Exact leaves appear as explicit, enumerated TCB assumptions in conditional
theorems; the semantics may not assume a high-level leaf conclusion for free.
A mechanically generated coverage table must map every rule and authority-
producing leaf in the ledgers to its formation, preservation, progress or
termination measure, erasure, and hostile-proof obligations.

Build a separate executable checker and independent byte/owner/root/observer
oracle for counterexample search. Bounded execution and negative corpora
support the general proofs but never substitute for them. Freeze state-space
bounds, seeds, and input hashes, with at least four Hashbrown slots; four
mimalloc blocks, two threads, and one reader; three SQLite page roots with
failure after every external event; and three Crossbeam participants including
one stalled participant, a two-entry bag, and two distinct deferred payloads.
Every producer, transfer, consumer, and invalidator needs a positive case and a
single-fault mutation; concurrent leaves need fixed interleaving/litmus cases.
Accepted oracle violations must be zero, every preregistered negative must be
rejected, and facts-on/off program semantics must match.

A Phase 3 correction that changes any frozen semantics reopens the earliest
affected gate and invalidates all dependent evidence. A proof, checker, oracle,
or generator defect that leaves semantics unchanged may be fixed, but all
affected evidence must be regenerated. An accepted counterexample yields
`STRATA-NO` only when no finite project-independent correction can pass Section
5 and the reopened gates; otherwise the corrected design must repeat them.

Gate: `STRATA-MODEL-YES` or `STRATA-NO`.

### Phase 4: decisive cross-project vertical evidence

Only after `STRATA-MODEL-YES`, preregister and build the smallest prototypes
that collectively exercise every authority class used by the surviving route
frontier. The four source projects remain workload and comparison suites, but
their fixture internals may use either reference or substitute routes:

1. a Hashbrown-shaped rehash route for classified liveness, direct owner
   traffic, progress, invalidation, and disposition;
2. a mimalloc-shaped allocation/free/page-disposition route for overlays,
   suballocation, local versus atomic custody, observation, and release;
3. a SQLite-shaped mutation/rollback route for checked byte subranges,
   multiple roots, repair-required state, exact external events, and poison;
4. a Crossbeam-shaped load/retire/collect route for zero-extra-event protected
   loads, unique retirement, erased one-shot disposition, and quiescence.

These are four demand fixtures, not four credited subpaths. Together they must
exercise every one of the fourteen frozen demand contracts. The named
reference-shaped routes above remain high-pressure lanes for detailed hostile
and code-shape inspection only when they survive the route frontier; a
contract-equivalent substitute may replace one of them.

The demand-to-evidence-scenario map is exact:

| Fixture | Required independently credited demand scenarios |
|---|---|
| Hashbrown | `H-LOOKUP`, `H-INSERT`, `H-REPLACE`, `H-REMOVE`, `H-REHASH` |
| mimalloc | `M-ALLOC`, `M-LOCAL-FREE`, `M-REMOTE-FREE` |
| SQLite | `S-INSERT-SPLIT`, `S-DELETE-BALANCE`, `S-ROLLBACK` |
| Crossbeam | `X-PROTECTED-LOAD`, `X-RETIRE`, `X-COLLECT` |

The verifier requires every demand ID to receive independent evidence. Code
may be shared and one substitute implementation may serve several scenarios.
A reference route must be trace- and contract-equivalent to its frozen source
operation. A substitute route must be contract-equivalent at the frozen
consumer boundary; it need not reproduce the source's internal trace. Every
scenario carries its own normal, failure, and rare semantic differential,
adversarial rejection set, structural manifest, resource report, and
measurement disposition. A combined workload may time several demands, but it
cannot replace any demand's independent acceptance result.

Each prototype must pass semantic differentials and adversarial rejection
tests before code-shape inspection. Freeze the source baseline, every candidate
route, compiler revision, target, allocator, event counts, instruction-body
comparison where applicable, resource ceilings, and measurement protocol
before observing performance.

Use one shared non-production checker, oracle, normalizer, and lowerer. Project
or operation identities may label fixtures and reports but may not enter the
accepted semantic input or lowering dispatch. Keep this prototype isolated
from the production specification, stage-0 compiler, and xlc so bootstrap
coverage cannot masquerade as B-Strata feasibility or failure.

Static erasure must hold in the canonical checked artifact and generic pre-
optimization lowering; fixture-specific backend dead-code elimination receives
no erasure credit. Lowering is local and syntax-directed by verified primitive,
never by recognizing a whole operation graph or fixture-shaped pattern. Freeze
per-scenario code-size, instruction-body, call/event, resource, and rare-path
limits before inspecting generated artifacts.

For a `REFERENCE` route, fail on any language-required extra:

- initialization or zeroing;
- payload copy, clone, relocation, or owner movement;
- tag, descriptor, backpointer, counter, hazard record, or dynamic borrow
  table;
- allocation, indirection, dynamic dispatch beyond the frozen reference,
  atomic, fence, synchronization, scan, or asymptotic work;
- success-path cleanup traversal absent from the reference; or
- mandatory code-size expansion caused by unused strata.

For a `SUBSTITUTE` route, these differences are recorded rather than rejected
merely for differing from the source topology. The route passes only when its
end-to-end demand endpoint meets the frozen non-inferiority margin and every
resource ceiling. Bookkeeping required only by B-Strata, rather than by the
chosen substitute algorithm, remains a language cost and cannot be hidden by
the substitution.

Benchmark tuning may diagnose a failure but may not change a frozen semantic
route after results are seen.

An endpoint earns performance credit in exactly one preregistered way:

- for a `REFERENCE` route, the optimized instruction body plus transitive
  call/event manifest is identical to the reference, with timing reported as
  confirmation; or
- its own quantitative non-inferiority test passes under the frozen sampling,
  confidence, margin, multiplicity, and resource-ceiling protocol. Every
  `SUBSTITUTE` route must use this quantitative path against the frozen demand
  baseline.

Results may not be pooled to hide a failing demand. Rare failure and release
paths may use exact event-count and structural limits only when that endpoint
was frozen in the cross-family lock. `INCONCLUSIVE` grants no route performance
credit and is not a third final state. A preregistered sample extension may run
once. If the maximum campaign remains inconclusive and structural identity is
absent, mandatory root-cause analysis must reduce the endpoint to its finite
instruction, call, event, resource, and workload differences. The one
nonsemantic implementation-correction round may then run without changing the
lock, semantics, or route. After regeneration, that route must either earn
performance credit or fail. A demand yields `STRATA-NO` only when every frozen
safe route fails or an independent deterministic-lowering blocker applies to
all routes. The goal may not stop at unexplained evidence insufficiency.

Classify every regression before a verdict. A semantic requirement that forces
an extra field, event, instruction class, or unit of work rejects the affected
route; it becomes a NO witness only if it is generic or every substitute also
fails the frozen demand band. A nonsemantic checker/lowerer defect permits one
implementation-correction round without changing the lock, semantics, or
route, followed by a fully regenerated campaign. An unexplained slow
measurement alone cannot be presented as an irreducible semantic NO.
Post-result semantic, route, leaf, algorithm, threshold, workload, margin, or
resource-ceiling changes invalidate the campaign and reopen the earliest
affected gate under the global repair rule.

After evidence, select one passing route per demand so that the union of core
rules is minimal among the frozen passing combinations. Delete every rule not
used by that selected bundle, regenerate all ledgers, and rerun the core, paper,
and model gates on the reduced core. This final pruning may remove a complex
reference-only capability; it may not add a new post-result route.

Gate: `STRATA-EVIDENCE-YES` or `STRATA-NO`.

### Phase 5: final verdict and landing boundary

`STRATA-YES` requires all previous YES gates. The final report must name the
exact minimal core, derived strata, exact-leaf boundary, checker/lowering
shape, fourteen demand contracts, selected reference or substitute routes,
rejected reference-only capabilities, hostile-review result, structural and
performance evidence, known limitations, and remaining non-gating ecosystem
work.

The YES scope is the frozen fourteen demands and the admitted semantic rules
actually proved and exercised. It does not claim that B-Strata can reproduce
every data structure in the four source projects, much less every systems data
structure. It is not general-purpose systems completeness, exact dense D-2,
P-1, or closure of the 340 unresolved dense obligations. The fifteen-family map
receives routing/non-regression credit only.

`STRATA-NO` must name the first irreducible failed constraint, the complete
repair and substitution attempts, the evidence that the failure is not a local
omission or a reference-topology artifact, and the demand-level performance or
safety consequence. Exact source-route failure alone cannot support NO. The
report must not recommend Candidate C as part of this goal.

A final YES selects B-Strata for owner review and must include
`CANDIDATE-B-STRATA-PRODUCTION-LANDING-PROPOSAL.md`. That proposal freezes the
first minimal production slice, affected specification rules, stage-0 and xlc
checker/lowering surfaces, conformance and derivation updates, code-shape pins,
migration/compatibility conditions, hostile review, both repository gates, and
rollback conditions. The verdict commit does not silently rewrite the kernel
specification or ship a production feature. Production specification, checker,
compiler, runtime, pattern-doctrine, standard-library, and xlc migration
changes remain separate reviewed landing slices.

## 7. Required deliverables

The complete track must leave durable English artifacts for:

1. this controlling plan;
2. the cross-family demand and evidence lock;
3. the normalized core and rule ledger;
4. the normalization and authority-origin ledgers;
5. the exact-leaf ledger;
6. the fourteen-demand ledger, bounded reference/substitute route frontier,
   and capability-deletion matrix;
7. deterministic verifiers for rules, interactions, routes, and status;
8. the hostile safety and erasure model plus negative corpus;
9. the lowering and structural-budget contract;
10. preregistered cross-project prototype and measurement protocols;
11. prototype results and generated-code evidence; and
12. the final `STRATA-YES` or `STRATA-NO` report; and
13. on YES, the exact production landing proposal.

Every completed phase receives its own commit and one append-only
`decision-gates.md` entry. Active status files advance only after the phase
gate passes or fails.

## 8. Research discipline

- Do not use a broad brainstorming or mind-expansion workflow.
- Do not reopen Candidate C or develop another full candidate.
- Do not add a capability because its name makes an open route look closed.
- Do not require source-topology fidelity when a safe substitute closes the
  frozen demand within its performance and resource bands.
- Do not add substitute routes that name no capability deletion or blocked
  demand they are intended to resolve.
- Do not confuse a token type with a safe producer of that token.
- Do not confuse memory safety with abstract container correctness, database
  crash consistency, eventual reclamation, or application progress.
- Do not claim zero cost from paper erasure alone; inspect generated artifacts.
- Do not claim measured performance from code-shape parity alone.
- Do not weaken a check or accept writer trust for performance.
- Do not continue refining after a proven irreducible NO condition.

The work is complete only at `STRATA-YES` or `STRATA-NO`.
