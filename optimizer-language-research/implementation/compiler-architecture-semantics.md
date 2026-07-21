# Production Compiler Architecture — Semantic Kernel

Status: OWNER-APPROVED ARCHITECTURE WITH BLOCKERS. This file is part of the
single [Production Compiler Architecture](compiler-architecture-design.md)
design record. `THE-PLAN.md` is execution authority. This file does not amend a
numbered specification, protected semantic surface, profile, or entrance gate.

This file uniquely owns Decisions 7 through 10. Cross-stage authority, entrance gates,
execution order, owner decisions, and exit status remain in the parent index.

## Decision records

### Decision 7: types, constants, operations, and substitutions

**Decision:** Represent modes, types, regions, and constants as separate exact
terms. Check every function declaration symbolically, pass its finite typed-call
records through Decision 9's pre-instantiation FN-6 gate, then enumerate
concrete semantic instantiations by a deterministic worklist, apply complete
normalized type/const substitutions and typed region environments, and recheck
every admitted instantiation before target layout.

**Problem being solved:** Rust host integers, traits, inference, map order, or
monomorphization shortcuts must not become Whitefoot semantics.

**Specification and project constraints:** TYPE-1 through TYPE-7, CONST-1/2,
FN-1/2/3/6/8, and the closed operation table require exact types, explicit
arguments, nominal identity, no implicit conversion, and concrete rechecking.

**Selected design:**

- `ModeTerm` is `own`, shared region, or unique region; it is never embedded as
  an accidental property of `TypeTerm`.
- `TypeTerm` is a closed structural enum over primitives, nominal declaration
  plus arguments, array, slice, box, arena, and buffer.
- Written call arguments are kinded `TypeArg`, `RegionArg`, or `ConstArg` in
  declared order; partial and inferred forms do not exist. The checker separates
  type/const monomorphization from the callee's lexical region parameters. It
  alpha-normalizes every region leaf nested in a type argument to a canonical
  first-occurrence `RegionSlot`, preserving equality of repeated leaves, and
  records a separate `RegionCallEnvironment` mapping declared region formals and
  nested slots to caller-side `RegionActualRef::Lexical` or
  `RegionActualRef::InstanceSlot`. Thus `f<T> -> g<T>` forwards a normalized slot
  without fabricating a lexical region or embedding the caller environment in
  an identity. Environment composition is validated for every incoming typed
  boundary, never only a worklist parent: lexical actuals must entail the
  callee's A-17 profile, slot forwarding must preserve/entail it, and every
  boundary-graph SCC must be locally profile-preserving. The independent closure
  check quantifies over the complete incoming-edge inventory with explicit
  step/cycle limits. No caller `RegionRef` enters an instance identity. A-17
  must select the exact finite region-fact profile available to a concrete
  generic-body recheck before these schemas land.
- A **parametric function declaration** has at least one type, const, or region
  parameter. A **zero-monomorphization function declaration** has no type or const generic
  parameter, although it may have region parameters, and is an empty-key seed.
  Complete target-independent template semantic coverage applies to every source
  function, including functions with no parameters; this keeps cross-function
  summaries closed before instance enumeration.
  Nominal types, contracts, and const items use separate
  kind/member/storage/dependency coverage and are never treated as function CFGs or call vertices.
- Constants use exact decimal and structural value terms. Integer range checks
  operate over mathematical sign/magnitude or bit vectors, never overflowing
  Rust arithmetic. Float parsing converts an exact bounded decimal rational to
  IEEE bits under round-to-nearest-even and validates canonical emission after
  the FORM-5/FORM-7 conflict closes.
- Operation calls are selected by exact written operation name, mode suffix,
  and explicit type arguments, then checked against the versioned declarative
  table. User calls use resolved function signatures. Neither is overload
  search.
- After a base nominal type, globally resolved constructor, function signature,
  or contract is fixed, typed-label resolution checks each field projection,
  construction/match field label, named argument, conform binding, and contract
  member against that one owner's declared ordered member table. It emits an
  exact label-to-member record. This deterministic dependent lookup is not
  lexical resolution, parser disambiguation, inference, or overload search.
- Symbolic template checking validates every function body against its
  signature and kind/contract environment, including unused parametric function
  bodies. A
  successfully checked user call emits an immutable source-ordered
  `TemplateTypedCallRecord` containing its resolved callee, checked symbolic
  signature, complete explicit type/region/const argument terms, declared-order
  arguments, result type/mode, and source node. It contains no derived graph
  edge, SCC result, body-derived return-provenance claim, or concrete-instance
  receipt. The same check emits a closed template `TypedCallBoundary` whose
  owners are exactly the caller and callee `Template` domains and whose only
  cross-owner entries are the written type/const/region formal-to-actual
  relations. Template
  effects and provenance consume that mapping; no raw foreign owner reference
  is admitted.
- Concrete rechecking emits the corresponding `ConcreteTypedCallRecord` with a
  complete normalized type/const semantic substitution, an explicit complete
  `RegionCallEnvironment`, its validated `TypedCallBoundary`, and concrete
  type/const terms. Lexical region identities and instance slots remain only on
  the caller side of that mapping. A separate
  `CallProvenanceRecord` is added only by Decision 8 after the owner-approved
  A-11/A-12 provenance phase; neither typed-call record is later mutated to add
  that authority.
- Nominal layouts are not computed until recursion legality, concrete
  substitution, and target profile are known.
- Storage admissibility is a separate positive recursive `StorableType` judgment
  for each exact STOR-5 obligation site's fully substituted stored type; it is
  never required for a non-storage value or inferred from the outer grammar node.
  Its exact rules remain blocked on A-13, so no field/container/generic
  instantiation involving a region-carrying nested type is accepted or given an
  artifact proof schema before the successor rule closes.
- Decision 9 first derives the finite template call graph/SCC partition and,
  after A-18 fixes the exact FN-6 relation, either rejects a violating cycle or seals
  `RecursionQualifiedTemplates`. Decisions 8 and 9 then complete parametric
  template CFG/region/ownership/provenance and effect judgments for every source
  function and seal
  `TemplateSemanticCoverage`. No instance worklist may start without both
  capabilities. Thus `f<T> -> f<box<T>>` is a normative FN-6 rejection before
  it can become an expanding resource workload, and no unused or cross-kind
  function body can escape non-target semantic checking.
- Semantic and code identity are separate. `SemanticInstanceRef` is
  `(function, complete type/const substitution with nested regions replaced by
  RegionSlots, A-17 region-fact profile)` and contains no `RegionRef`.
  Conservative `CodeInstanceRef { semantic_instance }` is one-to-one; a
  region-erased `EmissionShapeKey` is non-authorizing comparison data only. Merging
  several semantic instances into one emitted body is absent from the baseline.
  A later optimization would need an independently verified
  `CodeInstanceEquivalence` covering every mapped instance's erased signature,
  types, layout, CFG, runtime checks, cleanup, ABI, and selected-overlay
  consequences; one unequal field rejects sharing.
  Every zero-monomorphization function declaration is an empty-key seed, whether
  or not reachable from `main`; this ensures that an unused concrete body still
  receives concrete type, CFG, ownership, and effect checking. A parametric
  function with type or const generics is never given an invented seed: its
  semantic keys arise only from normalized substitutions on
  already checked calls reached by the canonical worklist. A function with
  region parameters but no type/const generics is its one empty-key concrete
  semantic/code instance; each call carries a separate region environment and
  cannot create another instance. After A-18 closes, FN-6 makes the approved
  type-shape-changing recursive expansion illegal before this step; const terms
  have no recursive expression
  form; and A-17 must select a finite fact profile for each bounded normalized
  type shape. These premises make closure finite without caller-owner nesting;
  ordinary finite fan-out remains resource-bounded.
  Every included body is concretely rechecked and emits concrete call records;
  no template proof substitutes for a concrete proof. A parametric function
  with type/const generics and no concrete key still has complete symbolic and
  parametric semantic checking, but no invented concrete instantiation.

**Input contract:** Symbolic checking receives the resolved unit, exact
operation/prelude inventories, resource profile, and approved contract rules.
Concrete closure additionally requires `RecursionQualifiedTemplates` from
Decision 9, `TemplateSemanticCoverage` from the semantic coordinator, and the
closed zero-monomorphization function-seed inventory, plus the A-17-approved
normalization/fact-profile rule; layout is still deferred.

**Output contract and established invariants:** `SymbolicallyTypedUnit` covers
every function declaration and owns complete immutable template call records
and template-owner typed-call boundaries.
`ConcreteTypedUnit` is constructed only from `RecursionQualifiedTemplates` plus
`TemplateSemanticCoverage`; it contains every required seed/reachable semantic
instance, one typed-call and typed-boundary record with normalized type/const
terms and an explicit region environment per concrete user call, canonical
unique semantic/code-instance tables and their one-to-one baseline mapping, and
exact typed-label/constant/substitution coverage. After A-13 closes, every
exact STOR-5 storage-obligation site has one complete positive recursive
`StorableType` judgment for its fully type/const-substituted stored type and a
site-to-proof coverage record. Legal non-storage values such as a transient
`slice` require no positive storage proof. Neither output contains a
call edge, SCC certificate, or return-provenance assertion.

**Explicit non-responsibilities:** This stage does not decide call-graph/FN-6
legality, return provenance, ownership transitions, effects, cleanup, ABI,
LLVM types, or optimizer facts.

**Why this stage owns the work / why adjacent stages do not:** Resolution fixes
names; this stage fixes mathematical terms. Ownership needs those terms. Layout
needs concrete types and a target. The backend must not redo any of them.

**Alternatives considered and rejected:** Rust primitive parsing alone makes
host overflow behavior semantic. Trait-driven operation lookup introduces
implicit overloading. Checking templates only violates FN-2. Hash-consing by
digest alone hides collisions and unstable order.

**Trusted assumptions and threat model:** Exact numeric algorithms and the
declarative operation table are TCB inputs. Hostile literals, type depth,
substitutions, and instance graphs try to overflow counters or explode work.

**Failure modes:** Normative type/constant/operation rejection is distinct from
resource exhaustion, impossible host representation, and compiler invariant
failure.

**Independent evidence required:** Arbitrary-precision numeric oracles,
operation-table source audit, substitution composition properties, nominal
near misses, field/member wrong-owner and wrong-order mutants, named-argument
coverage, kind mutants, concrete-template disagreement tests, and an independent
instance-closure graph. Required hostile cases include unused bad concrete and
parametric function bodies, an unused region-parameter-only body, a region-only
function omitted from empty-key seeds, missing zero-monomorphization seeds,
generic-to-zero-parameter-to-generic call SCCs and generic-to-zero-parameter
borrow/slice-return summaries before closure,
and self/mutual type-growing generic cycles that must fail FN-6 before the first
expanding instance is enqueued. A recursive region-parametric function that
passes a newly local region to itself must reuse one type/const instance while
recording distinct call-local substitutions; it may not create recursively
nested instance identities.

**Resource and determinism bounds:** Limits on digit bytes, type depth, type
terms, substitution length, constant elements, template and concrete typed-call
records, concrete instances, instance-follow steps, and total symbolic/concrete
rechecked nodes. Worklist order is canonical; all arithmetic is
checked; cycles are classified rather than recursively traversed on the host
stack.

**Dependencies on unresolved specification questions:** Float spelling,
contract-member semantics, law admission, recursive finite layout, recursive
STOR-5 storage well-formedness, TYPEID collisions, operation reservations, and
frame profile.

**Migration or foundation-audit consequences:** Add no generic Rust abstraction
that permits implicit inference. Select any bigint/decimal dependency only
after a dependency, determinism, and independent-oracle review.

**Approval status:** The architecture is adopted; unresolved language behavior
requires separate successor-specification approval.

### Decision 8: control flow, ownership, regions, and cleanup

**Decision:** After the FN-6 gate, complete target-independent template CFG/semantic
coverage for every source function. Then, from `ConcreteTypedUnit`, build an
immutable explicit CFG, attach complete owner-approved provenance without
mutating typed-call records, and only then check a closed ownership state
machine over every edge. Store accepted transitions and exact cleanup on the
checked representation before lowering.

**Problem being solved:** A syntax-only walk obscures joins, backedges,
propagation, and edge-specific cleanup. A generic low-level CFG loses the
structured scopes that define lexical regions and reverse-order drops.

**Specification and project constraints:** OWN-1 through OWN-13, STOR-1 through
STOR-5, GIVE-1, ERR-3, and EFF-4 require lexical borrows, resolved-place
overlap, bounded reborrow suspension, no trap cleanup, and explicit derived
normal-edge cleanup.

**Selected design:**

- After `RecursionQualifiedTemplates`, build a finite
  `TemplateControlFlowUnit` for every source function declaration and run the same
  target-independent region, provenance, control-edge, ownership, and
  cleanup-obligation judgments parametrically under its declared kind/contract
  environment. A symbolic copy/affine or operation claim is usable only when
  structural type form or a closed bound proves it for every admitted
  substitution; an unknown claim fails closed under OWN-8. Target layout,
  concrete const evaluation, and concrete drops remain deferred. Decision 9
  checks the template effect row. The semantic coordinator seals
  `TemplateSemanticCoverage` only when every source function has complete
  symbolic type, call, CFG, region, provenance, ownership, cleanup-obligation,
  and effect coverage. This covers unused templates but never substitutes for
  FN-2's concrete recheck.
- For each symbolic template or concrete function, construct a canonical
  `RegionTree` from the already resolved region declarations. A region record is
  exactly
  `CallerSupplied { declaration }` or
  `Local { declaration, parent_local_or_function_root }`; local parent links
  follow strict lexical block containment. The reflexive
  `outlives_or_equals(a, b)` predicate implements OWN-3 exactly: true when the
  IDs are equal, when two locals have `a` as a strict ancestor of `b`, or when
  `a` is caller-supplied and `b` is local; false for distinct caller-supplied
  regions and for every other pair. Function entry activates all region
  parameters, block entry/exit activates/deactivates its local region, and each
  call first requires every substituted region actual to be active. OWN-4
  storage/pass/return checks and OWN-12 calls cite this same recorded judgment;
  no host nesting or string comparison is an alternative relation. A concrete
  generic recheck also receives its normalized type-argument `RegionSlot`s and
  only the finite owner-independent facts selected by A-17; the
  `TypedCallBoundary` separately proves the caller-side lexical/slot actual
  mapping.
- OWN-10 uses a separate total root-class judgment
  `storage_outlives(resolved_root(place), requested_region)`. For
  `OwnBinding(b)`, the requested region must be local and its declaration scope
  must be lexically contained within `b`'s recorded storage scope; a
  caller-supplied requested region always fails, for locals and own parameters.
  For `Borrow(source_region)`, the source region must outlive-or-equal the
  requested region. For `Arena(arena_region)`, the arena region must
  outlive-or-equal the requested region. For `NamedConst`, the judgment is
  always true. A place reached through an alias-bearing view such as `slice` is
  never classified by the view binding's own storage scope; it requires the
  A-11-approved source-loan/region and all-origin rule. That root class and its
  OWN-10 artifact schema remain blocked with A-11. Any unknown or ambiguous
  resolved-root class fails closed. The
  derivation records the root class, binding storage scope where applicable,
  requested region, and exact lexical-containment or region-order premise.
- Runtime semantic state uses owner-scoped dense `RegionId`, `HolderId`, and
  `LoanId` handles and the owner-discriminated portable reference domain in
  Decision 5. A direct borrow expression initially associates its binding or
  call-scoped temporary with the created loan, region, kind, resolved origin,
  and same-node ordinal. The checker reserves an explicit binding-to-live-claim
  relation; it never infers a holder from the current syntax node. A-16 must
  decide how copying a shared borrow and moving a unique borrow update holder,
  loan, and binding-to-claim identity; how parameters/return/`give` transfer it;
  and how an OWN-13 borrow-mode match binder projects a claim while remaining
  ineligible as an OWN-6 child-reborrow parent. Until then none of those
  transition or artifact schemas is implemented. Replay rejects cross-owner,
  duplicate, stale, or wrong-kind
  references and can never use a template claim as a concrete claim.
- Concrete CFG construction produces `ConcreteControlFlowUnit`: complete
  blocks, operations, places, value-definition/use links, return/give sites,
  calls, and edges, but no ownership or return-provenance acceptance. Decision
  9 separately derives `ConcreteCallGraph` from immutable concrete typed-call
  records. No partially filled CFG or call record is authoritative.
- CFG blocks and edges retain a required `Origin`: one canonical source node
  or one closed derived reason tied to source nodes.
- Every source statement and expression receives structural CFG/coverage
  representation even when no runtime edge reaches it. Such blocks are marked
  `Reachability::UnreachableFromEntry`, never silently dropped or treated as an
  arbitrary artifact orphan. EFF-2 continues to scan the complete declaration.
  Production ownership/type rejection for an unreachable suffix is blocked on
  A-09's approved entry-state rule.
- Terminators cover fallthrough, match dispatch and join, loop entry/backedge,
  labelled break, give, try `Ok`/`Err`, return, retained-check success/trap, and
  call continuation/potential abort. Each source control edge appears exactly
  once.
- `ResolvedPlace` is a root declaration plus typed field, dereference, and index
  projections. OWN-7 is the only overlap relation.
- Every alias-bearing value—borrow mode or region-carrying slice—eventually
  carries a canonical finite `OriginSet` of formal/source `ResolvedPlace`
  expressions. Direct borrow creation is singleton; copies/moves preserve it;
  joins form a deduplicated union; operation and function summaries substitute
  caller origin sets and projections explicitly. Alias/loan checks quantify
  over every possible origin, never choose one traversal-dependent
  representative.
- A-11/A-12 must select one exact provenance rule before this phase or its
  artifact schema lands. Under a **signature-carried** rule, symbolic checking
  validates a canonical `ReturnOriginContract` in each affected signature. It
  also derives the actual origin set on every template and concrete
  return/`give` path and proves that set conforms to the declared contract under
  the one approved rule: at minimum an actual set must be a subset of a
  sound-upper-bound contract; an exact-contract rule additionally requires
  equality. A wrong or underbroad contract is always rejected, and an overbroad
  contract is rejected if the approved rule is exact. Each body-to-contract
  judgment and derivation is immutable artifact data. Only after those body
  judgments close may provenance qualification instantiate the contract at a
  concrete call. Under a
  **body-derived** rule, the CFG's value/return links generate equations over a
  finite formal-origin universe; Decision 9's already complete template call
  graph schedules the parametric template solution and its concrete call graph
  schedules each concrete recheck. A separate canonical provenance-equation SCC
  solver computes the owner-approved fixed point with explicit iteration/work
  limits. A successor rule choosing a single-origin restriction is represented
  as the corresponding checked one-member contract. These are language
  alternatives, not implementation modes; exactly one is selected by the
  successor specification.
- In the template phase, either selected branch produces immutable symbolic
  summary/call tables sealed inside `TemplateSemanticCoverage`. In the concrete
  phase it constructs a new immutable `ProvenanceQualifiedUnit` containing
  complete `FunctionProvenanceSummary` and `CallProvenanceRecord` tables. Each
  concrete call record binds one `ConcreteTypedCallRecord` and one closed
  `TypedCallBoundary`, then adds one immutable `CallProvenanceRecord` referencing
  that exact boundary. The later record repeats the validated caller/callee
  owner pair and lists only callee-formal-place/origin-to-caller-actual
  relations, resulting origin set, derivation, and coverage; it cannot alter the
  typed environment. Each referenced record remains local to its own owner.
  Template provenance uses the same two-stage template-owner forms. Both phases
  cover direct borrow and `slice_of`, copy/move subject to
  A-16, `give`, every return edge,
  branch/join union, ordinary and recursive calls, and formal-to-actual
  substitution. Original typed and CFG records remain unchanged. Ownership
  transfer does not start until the applicable table is complete, canonical,
  and A-11's slice read/write authority is known.
- `OwnershipState` contains active lexical regions; every binding's live/dead
  state, mode, type, storage scope, and holder role; and canonical live loans
  with region, origin set, shared/unique kind, and usable/suspended status.
- Binding storage lifetime and borrow lifetime are different fields. Ending a
  borrow or child suspension never implicitly drops its holder.
- Ordinary argument atoms follow the owner-approved A-03 evaluation order and
  thread ownership/effect state. This is required to reject two moves of one
  affine binding and to order traps or other state changes; no common snapshot
  shortcut applies to ordinary atoms.
- Child reborrow candidates that will coexist for the call receive an
  additional sibling-compatibility judgment against one explicit
  `CallBorrowSnapshot`, then atomically commit their child-loan and
  parent-suspension transitions under the approved evaluation model. The parent
  resumes after all call-scoped children end. The exact placement of that
  snapshot/commit is part of A-03 and remains blocked; it is not inferred by the
  checker.
- Each transfer computes one post-state. Joins compare all reachable incoming
  states under one approved finite join rule. Production join checking is
  wholly blocked until A-08 is approved; no provisional rejection, meet, or
  path-state rule is implemented as language behavior.
- A loop header carries an explicit invariant state. Entry and every backedge
  must agree with it; loop-local regions and loans are ended before a backedge;
  OWN-11 is checked directly.
- Every normal edge leaving owner or region scopes carries an `ExitPlan` with
  exact live drops, frees, and arena releases in approved reverse declaration
  order. Return, break, try-Err, give, match exit, loop iteration, and ordinary
  fallthrough are represented. Trap/abort edges carry no cleanup.

**Input contract:** Template checking receives `RecursionQualifiedTemplates`,
resolved lexical scopes/regions, the one owner-approved A-11/A-12 contract, and
approved target-independent semantic rules. Concrete CFG construction receives
`ConcreteTypedUnit` and target-independent storage classes. Concrete provenance
qualification additionally receives Decision 9's complete
`ConcreteCallGraph` and the provenance resource profile. Ownership receives
only the corresponding complete provenance-qualified capability, plus approved
evaluation, join, unreachable-suffix, and cleanup rules.

**Output contract and established invariants:** `TemplateControlFlowUnit` and
the Decision 8 contribution to `TemplateSemanticCoverage` completely cover
every source function under its declared environment. `ConcreteControlFlowUnit` has
complete structured CFG/source-node coverage. `ProvenanceQualifiedUnit` has one
complete immutable provenance record for every alias-bearing definition and
call. `OwnershipCheckedUnit` adds one validated state transition per reachable
semantic operation and edge plus the approved A-09 judgment for unreachable
operations; exact region/outlives, join, and backedge records; no access
violating live loans; no use after move; complete normal-exit cleanup; and no
trap cleanup.

**Explicit non-responsibilities:** The CFG builder does not infer types, assert
provenance, or invent drop semantics. Provenance qualification does not perform
ownership transitions. The backend does not discover control edges, regions,
liveness, loans, joins, or cleanup. Last-use/NLL shortening is absent.

**Why this stage owns the work / why adjacent stages do not:** Only the semantic
checker knows resolved places, lexical regions, source control structure, and
ownership state together. Delaying any of these decisions to lowering creates
an unchecked semantic path.

**Alternatives considered and rejected:** A pure structured walk makes edge
coverage difficult to state. An unstructured optimized IR discards lexical
evidence. Implicit backend drops cannot satisfy DIAG-2. Path explosion from
unbounded disjunctive states is rejected; the approved join must be finite.

**Trusted assumptions and threat model:** The checker state machine is TCB
code. Same-kernel replay receives complete states and transitions, reapplies
the closed judgments, and compares full states; it may not trust state digests
or producer completeness flags. The distinct bounded operational model and
cleanup interpreter provide independent semantic evidence.

**Failure modes:** Ownership and storage violations cite their exact rule and
node. Resource exhaustion, missing CFG coverage, inconsistent state, and bad
cleanup are distinct resource/invariant/artifact outcomes.

**Independent evidence required:** Bounded operational ownership model,
Featherweight-Rust reconciliation cases, exhaustive small paths, join and loop
mutants, reborrow sibling permutations, single/multiple returned-origin and
slice read/write mutants, unreachable-suffix type/ownership/effect cases,
independent cleanup path interpreter, and failure injection on every exit kind.
Region cases exhaust equal, local ancestor/descendant/sibling, parameter/local,
and distinct-parameter pairs. OWN-10 cases independently cover own bindings at
function/block/match-arm scopes, caller-supplied rejection, borrow roots, arena
roots, const roots, alias-view roots after A-11, cross-scope escapes, and
slice-copy/move/join with expired-source mutants. Holder cases after A-16 include
copied shared borrows, moved unique borrows, parameter/return/`give` transfer,
borrow-mode match-binder projection, stale creation holders, and several live
holders linked to one shared-loan origin. Mutants swap region classes/parents,
storage scopes/root classes, caller/callee owners, call-boundary functions,
owner tags, holder/loan IDs, wrong-origin/underbroad contracts, overbroad
contracts under an exact rule, body-to-contract derivations, equations, fixed
points, call substitutions, and phase capabilities.

**Resource and determinism bounds:** Iterative CFG construction; explicit
limits on blocks, edges, place depth, bindings, loans, state cells,
region-tree depth, outlives queries, holders, per-value origin members, total
live origin members, origin-term depth, provenance summaries/equations/edges,
formal-to-actual substitution expansion,
SCC iterations, `edges * live_state_size`, `state_cells * origin_members`,
cleanup records, loop checks, provenance work, and total work. All products are
checked before allocation/work. IDs, deduplicated origin members, SCC worklists,
and CFG worklists follow source and canonical edge order.

**Dependencies on unresolved specification questions:** Affine dereference
lifecycle, affine `set`, evaluation order, ordinary-scope cleanup, join rule,
unreachable-suffix checking, slice mutability/provenance, cross-function return
provenance, recursive storage well-formedness, holder identity after borrow
copy/move, and target frame policy.

**Migration or foundation-audit consequences:** Freeze no CFG opcode, ownership
state, or cleanup schema until the relevant language questions and artifact
record obligations close.

**Approval status:** The architecture is adopted. Required ownership and
control-flow transitions remain blocked on the named successor-specification
decisions.

### Decision 9: effects, recursion, and call-graph closure

**Decision:** Run the finite template-call/FN-6 legality gate after symbolic
template checking and before any concrete-instance enumeration. Separately
apply the owner-approved declared-row exhibition rule to every symbolic
template and concrete recheck. This remains conditional on owner disposition of
recursive exhibition and the recorded effect discrepancies; no production
effect acceptance or artifact schema may land before that disposition. Do not
infer optimizer authority from an effect row.

**Problem being solved:** Effects combine direct operations, user-call declared
rows, region substitution, bidirectional exactness, and recursion. A transitive
fixed point silently changes EFF-2.

**Specification and project constraints:** EFF-1 fixes row syntax/order; EFF-2
defines syntactic exhibition in both directions; FN-6 permits recursion but
restricts generic call cycles; EFF-3 gives limited pure-call transformations.

**Selected design:**

1. Consume `SymbolicallyTypedUnit` and derive one finite `TemplateCallGraph`
   from its `TemplateTypedCallRecord`s. Vertices are source function
   declarations, never concrete instance keys; each edge retains the exact
   explicit symbolic substitution, source call node, and validated template
   `TypedCallBoundary`.
2. Compute the deterministic template SCC partition. The FN-6 candidate domain
   is every cyclic call edge in an SCC whose participating functions carry the
   grammar's `generics`; no type-only narrowing is permitted. A-18 must define
   the exact type/const kind-and-argument vector relation before the gate can
   accept or issue its normative violation. Until then it constructs no
   `RecursionQualifiedTemplates` capability.
3. After A-18 closes, order vertices by portable function `DeclRef` and edges by
   caller, source `NodePath`, ordinal, then callee. Select the first violating
   edge and append the canonical shortest path from its callee back to its
   caller within the SCC, breaking equal-length paths lexicographically by that
   edge order. The resulting `Fn6CycleWitness` records every function/call node,
   expected and actual kinded argument vectors, FN-6 rule ID, primary source
   node, and closed restructuring diagnostic payload. The rejecting stage
   outcome owns and deterministically serializes that witness; it is not an
   accepted base-artifact record. On success, artifact projection records every
   accepted cyclic-edge judgment and replay re-derives the SCC and complete
   acceptance inventory. Only then does `RecursionQualifiedTemplates` seal; no
   semantic instance key is enqueued first.
4. Record each template's declared canonical row, scan its complete
   body/requires block for direct exhibits, substitute symbolic region actuals from
   template typed-call boundaries, and compare the resulting row exactly in both
   directions. This uses only facts guaranteed by the declared kind/contract
   environment; an unproved template-wide judgment fails closed under OWN-8 and
   cannot be deferred because the template is unused. Together with Decision
   8's parametric result, this is the effect contribution required before the
   coordinator seals `TemplateSemanticCoverage`.
5. Only after that coverage lets Decision 7 produce `ConcreteTypedUnit`, derive
   a separate finite `ConcreteCallGraph` from its immutable
   `ConcreteTypedCallRecord`s. This graph and its SCC partition may schedule
   Decision 8 provenance equations, but do not supply or assert their solution.
6. Re-scan every complete semantic-instantiation body/requires block under its
   declared symbolic region parameters. Every concrete call record carries its
   complete `RegionCallEnvironment`, which is checked at that call but never
   changes the caller or callee instance key. Compare declared and exhibited
   rows exactly in both directions. Every concrete effect judgment is a recheck,
   never a reuse of the template receipt.
7. Separately compute a grounded-effects view only as evidence until the owner
   decides whether it should become normative.

If the approved rule differs from literal declared-row exhibition, this
algorithm and every effect derivation/artifact record are revised on paper
before implementation; no earlier experiment or schema has authority.

**Input contract:** The FN-6 gate receives Decision 7's complete
`SymbolicallyTypedUnit`, validated signatures, and resource profile. Concrete
graph construction receives `ConcreteTypedUnit`. Effect checking receives the
corresponding complete symbolic/concrete body records, substitutions, validated
   operation inventory, declared rows, and approved EFF-1
   canonicality/body-local-region rules. Decision 9 is the sole constructor of template/concrete **call**
   edges, call-graph closure, and call SCC partitions; Decision 8's
   provenance-equation dependency SCC is a distinct record family and cannot redefine calls.

**Output contract and established invariants:** `TemplateCallGraph` and
`RecursionQualifiedTemplates` completely cover all symbolic calls before
instantiation. `ConcreteCallGraph` completely covers all admitted instances.
Every exhibited effect has one source record; every declared effect is
exhibited under the selected rule in both the symbolic template and each
concrete recheck; every call substitution is explicit; each call SCC partition
and cyclic edge set is complete.

**Explicit non-responsibilities:** Call graphs do not enumerate instances or
solve provenance equations. Effects do not prove termination, no-alias,
absence of memory access at the ABI level, or LLVM attributes. In particular,
Whitefoot `pure` does not imply LLVM `memory(none)`: an ABI-indirect aggregate
argument or immutable rodata access can still be memory-backed.

**Why this stage owns the work / why adjacent stages do not:** Resolution and
typing establish calls and regions. Effect checking consumes them. LLVM cannot
repair a missing row or use a row as stronger target information.

**Alternatives considered and rejected:** Least-fixed-point inference is not
the written rule. Trusting declared rows without body comparison violates
EFF-2. Combining effects and FN-6 SCC checks obscures their different purposes.

**Trusted assumptions and threat model:** Exact operation effects and row
rules are semantic TCB inputs. Forged call edges, substitutions, SCCs, or local
exhibit lists are hostile artifact mutations.

**Failure modes:** Row mismatch is a normative rejection. An ungrounded cycle
follows the owner-confirmed rule. Bad graph certificates are artifact failures,
not source diagnostics.

**Independent evidence required:** Independent effect/call-graph model;
self/mutual pure and ungrounded-trap cycles; one grounded operation in a cycle;
reads/writes/allocations around cycles; region substitution; SCC membership and
edge mutants; template/concrete graph substitution; and type-growing
self/mutual generic cycles that must reject before instance-work counters advance.

**Resource and determinism bounds:** Canonical row storage is bounded by region
parameters and effect kinds. Body folding is linear in symbolic/concrete
operations and calls. Template vertices/edges/SCC work are source-sized and
complete before instance work; concrete vertices/edges are instance-bounded.
Both SCC constructions, artifact replay, and independent graph evidence are
linear/log-linear under separate explicit ceilings and iterative traversal.

**Dependencies on unresolved specification questions:** EFF-1 row
canonicality, body-local regions, recursive exhibition disposition, contract
members, and top-level call visibility.

**Migration or foundation-audit consequences:** Keep effect algorithms out of
the parser and backend. Do not add grounded-effect authority to the artifact.

**Approval status:** The architecture is adopted. Recursive-effect disposition
and the existing effect discrepancies remain separately gated.

### Decision 10: artifact replay and semantic authority

**Decision:** Reject a separate production semantic certificate verifier. Use
one trusted semantic kernel and mandatory complete replay of the canonical
artifact through that same kernel before constructing `AcceptedCompilation`.
Replay returns an artifact acceptance decision but can never turn later
serialized bytes into lowering authority. Every optional optimizer fact keeps
its independent verifier because it grants additional authority. This decision
removes only a second production semantic certificate verifier; independent
grammar, source/tree, conformance, model, target, backend, guard, publication,
and hostile-mutant evidence remain mandatory.

**Problem being solved:** D22 selected an independent verifier, but a second
copy of resolution, ownership, cleanup, and effects increases complexity
without necessarily reducing trust. DIAG-2 still requires proof records and an
artifact-only acceptance decision.

**Specification and project constraints:** DIAG-2 requires proof objects and
artifact-decidable acceptance, not implementation independence. SCOPE-3 already
places the checker/compiler in the TCB. The current roadmap choice can be
changed only explicitly by the owner.

**Selected design:** The checker/target qualifier constructs a private complete
draft, projects one canonical artifact, and invokes a replay entry point whose
only semantic input is those bytes. Replay uses the same trusted local
judgments as construction but traverses the explicit derivation records and
coverage tables instead of relying on producer memory. It validates:

```text
source/tree binding -> declarations/resolutions -> symbolic template typing
-> template call graph/SCC/FN-6 -> template semantic/effect coverage
-> concrete instance closure/rechecking + types/storage/substitutions
-> concrete call graph
-> CFG/provenance/regions/ownership/joins/cleanup -> concrete effects
-> target layouts/ABI/frame
-> checks/reports -> whole-artifact coverage
```

For an artifact within the supported schema and hard resource profile, replay
returns exactly `ArtifactDecision::Accepted` or
`ArtifactDecision::Rejected(ArtifactReason)`. Resource inability is a separate
outcome. The compiler constructs the opaque `AcceptedCompilation` only after
replay accepts the bytes it just projected. The public later-audit entry point
returns the same decision and may produce an opaque borrowed
`ArtifactAuditContext` reconstructed from those bytes for nested
fact/report/final-envelope audit. That context has no constructor, conversion, trait, or
dependency path to `AcceptedCompilation`, `FinalizedCompilation`, or
CodegenPlan authority.

**Input contract:** Complete canonical checked-artifact bytes. They embed exact
source, specification, schemas, target profile, semantic records, target
qualification, derivations, reports, and coverage. Replay consults no producer
memory, cache, ambient source, or capability metadata.

**Output contract and established invariants:** One complete artifact decision;
on `Accepted`, every byte and record is canonical, every derivation replays,
every reference and coverage obligation closes, and target acceptance is
reproducible from the artifact. Public audit may lend the non-lowerable
read-only fact/report views as a lifetime-scoped `ArtifactAuditContext`; it
cannot expose `ReplayedAcceptedPayload`. The private compilation entry point additionally
returns `ReplayedAcceptedPayload` reconstructed entirely from those bytes. It
compares the embedded source binding, specification, schemas, and target
profile byte-for-byte with the originating invocation, then pairs the replayed
payload—not the producer draft—with the bytes to seal `AcceptedCompilation`.
The producer draft is discarded before lowering.

**Explicit non-responsibilities:** Replay is not independent semantic evidence,
does not reduce the SCOPE-3 compiler/checker TCB, does not optimize or lower,
does not reproduce source diagnostics, and cannot authorize cached, third-party,
or hand-authored artifacts for code generation. `ArtifactAuditContext` exists
only to let downstream auditors inspect replayed facts without creating an
authority conversion.

**Why this stage owns the work / why adjacent stages do not:** The trusted
semantic kernel owns all acceptance judgments. Mandatory replay is the only
point that can prove the published artifact completely represents those
judgments. The backend is too late, and a second semantic implementation is not
proportionate to the product threat model.

**Alternatives considered and rejected:**

- One checker with no artifact replay fails DIAG-2's artifact-only decision and
  misses projection/codec omissions.
- A complete checker plus independent certificate verifier was developed to its
  strongest form: explicit scope witnesses, local type judgments, complete
  ownership states and transitions, join/loop invariants, effect equations,
  SCC certificates, cleanup proofs, instance closure, target layouts, and
  coverage. Validating those still duplicates the soundness-bearing visibility,
  substitution, overlap, ownership, join, cleanup, effect, layout, and closure
  rules. Demonstrating the real schema would itself build a shadow compiler.
- Two full semantic checkers double implementation and change cost, share the
  same specification gaps, and add a disagreement authority path.

If a future product requirement allows cached or third-party checked artifacts
to produce code, the threat model changes and the independent-verifier design
may be reconsidered through a new owner-approved architecture decision. It is
not built speculatively now.

**Trusted assumptions and threat model:** The semantic kernel, replay walker,
codec/schema, target qualifier, and private constructor are trusted. Replay
targets projection, omission, corruption, stale references, malformed bytes,
and cache/audit inspection. Independent models, conformance, mutants, and
backend defenses target semantic-kernel logic bugs.

**Failure modes:** A source rejection occurs before projection. Projection,
encode, or replay resource failure publishes nothing. Replay rejection of
checker-produced bytes is a compiler invariant failure. Later untrusted bytes
may be accepted as an audit statement but still cannot be lowered. Unknown,
duplicate, missing, cyclic, unreachable, noncanonical, or inconsistent records
reject.

**Independent evidence required:** Artifact omission/duplication/reordering and
reference mutants; source/tree/target mutation; coherent substitutions of a
different but internally valid type, layout, CFG, or source binding; one mutant
for every semantic derivation and coverage family; exact projection/replay
round trip; attempts to forge or decode `AcceptedCompilation`; plus independent
semantic models and conformance that do not count same-kernel replay as
independence.

**Resource and determinism bounds:** Replay has the artifact ceilings in
Decision 15, uses iterative traversal and checked arithmetic, and performs a
bounded number of passes over canonical records. Any semantic family requiring
more work records that work explicitly. Artifact decision and failure ordering
are byte-stable. Publication waits for replay completion.

**Dependencies on unresolved specification questions:** Exact replay records
remain deferred until the grammar, semantic, target, artifact, and report
contracts close. This does not reopen the rejected independent-verifier route.

**Migration or foundation-audit consequences:** Recommend through the
separately approved Decision 18 migration that the current broad
`whitefoot-verifier` be narrowed to the exact source-binding audit
responsibility. The proposed later architecture puts artifact replay and the
private lowering-authority factory in the semantic component; no name, crate,
factory, API, or dependency change is authorized here.

**Owner ruling (2026-07-21):** The narrow future trust and authority topology
is selected as the proposed replacement for D22's independent production
semantic certificate verifier. It does not immediately change D22 or
`THE-PLAN.md`, and it approves no artifact schema, replay-record family,
`AcceptedCompilation` contract or factory, crate/dependency/API change,
resource profile, roadmap/design-tree edit, migration, or implementation.
