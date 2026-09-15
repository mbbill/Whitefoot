# Two candidate ownership systems for Whitefoot

This is a whole-language proposal, not a change to the active specification.
The [research record](RESEARCH.md) owns the requirements, alternatives, source
assessment, and affected current decisions. The
[executable experiment](../../experiments/access-state/RESULTS.md) tests a
bounded fragment of the first candidate. Maintain this proposal while that
comparison remains active; supersede it when a subsequent design replaces it.
Notation in this document is explanatory, not final source grammar.

## Recommendation and the actual choice

Develop **A: access and current-state checking** as the main candidate, with
**B: validity loans and access effects** as a complete, more restrictive
alternative. Both replace persistent exclusive write loans. The difference is
what a retained reference promises about future storage validity.

- A reference in A is a locator. It says which storage an operation would
  access. Whether that access is legal is proved in the current state.
- A reference in B additionally holds a validity obligation over a declared
  extent. Destruction or a layout change that would violate the obligation is
  forbidden until that obligation ends. Ordinary compatible writes remain
  possible through other aliases.

A fits WF's existing emphasis on explicit, erased proofs and source-order
semantics particularly well: memory validity, initialization, indexing domains,
and object invariants become instances of the same current-state reasoning.
It also admits reclaiming a graph node while unused locator values remain in
other nodes. Its cost is richer state contracts and more proof work for stored
graphs. B makes more references usable by construction and rejects more
programs at destruction rather than at subsequent access. This is an
expressiveness and interface tradeoff, not an established performance ranking.

The recommendation is a research conclusion. Neither candidate is adopted,
implemented by the WF compiler, or proved sound as a complete language here.

The main semantic questions now have explicit proposed answers rather than
unspecified mechanisms:

| Question | Proposed answer |
|---|---|
| Must each alias hold an exclusive permission? | No. Access demands query one checked sequential context; disposal obligations remain affine/linear. |
| How does a caller recover hidden relationships? | Ordinary stored links and explicit abstract association/result contracts; no body-derived ancestry. |
| Does reuse need a runtime generation check? | Fresh logical allocation identities and current-use premises distinguish reuse; the bounded physical model exercises this. |
| Must the compiler enumerate runtime objects or alias cases? | No. First-order substitution, conservative possible-alias updates, and explicit family/invariant witnesses. |
| Can a stored mutable pointer keep changing its static target? | Its slot can change association; a previously loaded locator retains its captured target. |
| Can effects completely replace safety checking? | No. They describe accesses; type, initialization, liveness, ownership and domain premises justify them. |

Choosing A or B remains an owner decision. Proving and measuring the complete
calculus remains research/implementation work; those limits are stated at the
end and are not passed off as consequences of the bounded model.

## Common foundation: separate three responsibilities

| Information | Question it answers | Can it be copied? |
|---|---|---|
| Locator and association | Which storage or abstract state does this value designate? | Yes, subject to ordinary value/visibility rules. |
| Ownership obligation | Who must consume, return, or release this resource? | An affine or linear obligation cannot be duplicated. |
| Current access evidence | Is this target live, suitably typed and initialized, and is this operation authorized now? | Facts can be reused in one sequential context; they are not permission to duplicate that context into racing executions. |

Effects describe actual reads and writes. They neither manufacture an owned
resource nor prove that storage exists. For example, `store` and `free` both
write their target, but only one preserves it. Ordinary state pre/postconditions
express that difference; no additional scheduling effect category is needed.
Ending a storage lifetime counts as a write to that storage even when the
implementation physically changes only allocator metadata.

An owner is not automatically the only pointer to its content. A locator is
not automatically permission to dispose of its content. A read-only locator
restricts operations through that path; it does not promise that other paths
never write. Frozen data, when required, needs an actual invariant or access
contract that excludes the conflicting writes.

An object's association is a compile-time parameter describing runtime values,
not a runtime parameter pretending to be a type. A pointer still contains the
ordinary address. The static parameter connects its type to contracts and
proofs; it need not encode that address or select different machine code.

## Candidate A: locators plus a current resource context

### Values, storage, and the typing judgment

Use three kinds of static names: ordinary types `T`, storage/association names
`P`, and footprint/family expressions `F`. A footprint can name fields or
captured ranges within storage, or explicitly related abstract state. It is
not the entire graph reachable through every pointer.

```text
Loc<P, T>                 // locator for a T-shaped slot in P
Owner<P, T>               // ordinary owner descriptor plus disposal obligation
exists P. Owner<P, T>     // package an allocation whose identity is hidden
```

Read/write authorization can be restricted by the locator's interface and
ordinary visibility rules. A writable locator is aliasable. It grants no
exclusive interval and cannot promote a read-only path to writable. The exact
surface spelling of these access restrictions is independent of the mechanism.

The proposed checking judgment is:

```text
names; values; resources; facts |- statement
    => values'; resources'; facts' ! reads(R), writes(W)
```

- `values` describes bindings, types, captured target identities, and owned
  values. This is where an owner move is checked.
- `resources` describes current live storage, layout, initialized fields,
  outstanding disposal obligations, and explicitly folded family predicates.
- `facts` describes equalities, separation, bounds, stored links, and current
  invariants. Every fact about mutable state has its supporting footprint.
- `R` and `W` describe runtime access. Proof-only operations do not add effects.

The resource context is compiler state, not a heap table or a runtime token.
At an ordinary sequential call it is checked against the signature and then
updated by the verified exit contract. No user has to pass this context as a
runtime argument. Opaque code has the same explicit contractual obligations as
ordinary code; declaring a contract does not establish its implementation.

The essential distinction from explicit capability threading is that two
sequential demands for `write(P)` are demands on the same context. Substitution
of `P = Q` in `{write(P), write(Q)}` combines the footprint; it does not demand
two consumable ownership tokens. A call consuming two owners still needs two
legitimate owned values and cannot receive the same linear value twice.

This distinction has a precise precedent and a caution. [Alias Types](https://www.cs.cornell.edu/talc/papers/alias.pdf)
uses freely copied pointers and a separate store description. Its linear
store join forces distinct entries to be disjoint; simply reusing that join
for WF's alias-permitting access demands would reject useful calls. The
[SL/implicit-dynamic-frames comparison](https://lmcs.episciences.org/802/pdf)
also distinguishes ordinary conjunction from permission addition. The proposed
access-demand union is not a new way to duplicate linear resources.

### Primitive rules

These rules define the proposed first-order core. They are not trusted
writer-supplied assertions. A library contract must be derived from the rules
and from checked representation invariants.

| Operation | Required evidence | State after the operation |
|---|---|---|
| Create storage | The allocating operation's capacity, layout, alignment and representation conditions, or its success branch | Fresh logical allocation identity, live storage and one accounted-for owner; initialization follows the actual operation. |
| Copy a locator | A well-formed locator value and permission to read its containing slot if loaded from memory | Same target; no new owner, liveness fact, or separation fact. |
| Read data | Live enclosing storage, active layout, initialized selected data, readable path, and all operation-domain proofs | Preserved storage; copying is allowed only for copy values. |
| Write copy data | Live compatible slot, writable path, well-typed value and operation-domain proofs | Selected slot initialized; overlapping current-value facts forgotten before exit facts are added. |
| Replace affine data | Checked transfer of old and new ownership, plus compatible live writable storage | Old value becomes the returned owner; new value occupies the same slot. No release obligation disappears. |
| Take data out | Initialized live writable slot and permission to transfer its content | Value ownership moves out; selected slot is uninitialized. Other aliases cannot read it until initialization is established again. |
| Move an owner descriptor | The source owns the value, and destination formation is valid | Source binding is consumed; backing identity is unchanged if backing storage does not relocate. |
| Reclaim storage | Its disposal authority, current liveness, and discharge of contained/dependent resource obligations required by its representation | Allocation identity permanently ceases to be live; every possibly affected current-state fact is removed. Writes include the reclaimed storage and actual management state. |
| End a stack scope | Ordinary cleanup/linearity obligations | Local storage ceases to be live before function exit postconditions are checked. |

An operation can lose proof information through possible aliasing without
proving that every possible target changed. If writing Q might change P,
forget P's old value fact; do not assert that P definitely has Q's new value.
Only must-alias information permits that strong conclusion. Liveness and
initialization use the same distinction.

Structural state matters as much as byte values. Changing an enum variant
removes the former active-layout facts. Truncating a vector can remove element
initialization; replacing its backing allocation ends the old allocation's
validity. A reference into an old field or buffer needs the corresponding
current layout and live-storage evidence before another read.

There is a deliberate difference between reinitializing a still-live slot and
reallocating storage. After `take(p); put(p, new_value)`, an alias of that slot
may be usable again. It designates a slot, not an immutable value identity.
After `free(P)`, allocating another object at the same physical address does
not restore P. Suballocation release similarly ends that reservation's logical
identity even if its enclosing physical allocation remains.

Existing deterministic affine cleanup can be preserved, with effects and
state transitions made explicit to checking. Linear values still require the
language's explicit consumption discipline. This proposal adds no hidden
user-defined finalizer. A parent owner cannot silently erase separately held
linear child obligations: a bulk-release contract must account for them, or
require an empty outstanding-child family.

### What a reference means after reclamation

An inert locator can remain in a register, field, collection, or capture. It
may be copied or discarded without dereferencing its target. Reading the slot
that contains it still requires that containing slot to be live and initialized.
Using it to access the former target requires evidence that cannot be obtained
from a new allocation's liveness.

The core should permit no pointer-to-integer operation that reconstructs access
authority, and no assertion that distinct logical generations have distinct
observable addresses. For the initial candidate, pointer identity comparisons
require current valid operands and a defined comparison domain. A later raw
address-observation API would need its own explicit semantics; it could not
turn address equality into ownership or resurrect a dead allocation.

This is a concrete policy choice, not an unresolved need for runtime generation
checks. [LLVM's object-lifetime rules](https://llvm.org/docs/LangRef.html#object-lifetime)
distinguish dereferencing from non-dereferencing pointer operations, so inert
pointer values are not inherently incompatible with a low-level backend.
That observation does not prove a particular WF lowering correct.

If an API promises that a result is immediately usable, its exit contract
includes the relevant live/initialized state. Returning a locator to a local
variable cannot satisfy that contract: scope exit destroys the local first.
An interface deliberately returning only an inert locator makes a weaker
promise. This distinction belongs in the interface, not an optimizer heuristic.

### Function boundaries and generics

```text
fn write_pair<P, Q>(x: writable Loc<P, u8>, y: writable Loc<Q, u8>)
    requires live(P), live(Q), initialized(P), initialized(Q)
    writes(P, Q)
    ensures live(P), live(Q), initialized(P), initialized(Q)
{
    store(x, 1)
    store(y, 2)
}
```

P and Q may be equal. The definition is checked once under precisely that
possibility. At `write_pair(r, r)`, the caller substitutes one actual target
for both. The signature requests one writable footprint twice; the second
store wins in source order. No runtime alias dispatch or body cloning is
necessary.

By contrast, `take(y); read(x)` cannot be verified for arbitrary P and Q.
Taking from Q may uninitialize P. A contract requiring `disjoint(P, Q)` makes
the body valid, and the caller must prove that condition. Another valid body
can restore initialization before reading. Caller knowledge does not justify
an undeclared assumption retroactively.

The call algorithm is fixed:

1. Resolve actual values and their captured associations. Infer association
   arguments by first-order matching where an argument position determines
   them. Otherwise require explicit arguments or witnesses. Do not search
   over alias partitions, higher-order qualifier solutions, or callee bodies.
2. Substitute those associations throughout types, preconditions, effects,
   state transitions, and result relationships. Preserve DAG sharing and
   normalize equal effect subjects idempotently.
3. Check required facts and ownership transfers. A pure liveness requirement
   and a consumable owner requirement remain different judgments.
4. Apply the declared changes conservatively through possible aliases, then
   establish verified exit facts. Preserve the unaffected frame.
5. Open result packages. A returned existing target retains its association;
   only a contract for actual creation introduces freshness/separation.

Ordinary type/const code specialization can remain a separate implementation
choice. An association parameter is still a genuine generic parameter even
when it erases and produces no additional code instance.

Higher-order function arguments or captures carry latent effects, target
relationships and state contracts. Capturing a locator does not capture an
unconditional promise that it will remain live. Capturing an owner follows
ordinary affine/linear function-value rules. A callback that clears, moves,
reclaims or retargets state exposes the same transitions as a direct call.

The expanded signatures above expose every premise for clarity. A concrete
surface language can derive ordinary shallow type/readiness obligations from
parameter forms and selected accesses by fixed rules. It must still export
state-changing contracts and additional domain conditions. Inferring a simple
target argument is not permission to infer arbitrary graph invariants or
strengthen a public precondition by inspecting callers.

### Stored associations and mutable fields

The identity attached to a locator is an immutable value image. The heap fact
that a field currently contains that locator is mutable and has support.

```text
holder.next = p
saved = holder.next       // saved targets P
set_next(holder, q)       // writes holder.next; exit contract says target Q
read(saved)               // still accesses P, never Q
```

Writing the field kills its former link fact through every possible alias of
the field. It cannot rewrite the already captured identity of `saved`.
The call contract of `set_next` names the entry holder and new value and
publishes the replacement link. Effect paths are resolved against their
declared entry or result images, never silently re-evaluated later.

For a fixed `Cell<Loc<P, T>>`, P is invariant: storing Q requires P = Q.
For a retargetable cell, use an existential target or a declared possible-target
family. Loading opens a target witness; an update changes the cell's current
association, not every outstanding witness. This handles ordinary structs,
opaque objects and containers through the same rule. No hidden host-origin
table is involved.

At an unknown branch result, use a finite target envelope such as `{P, Q}`
and a captured result identity X. X is not fresh storage. A write through X
can establish a fact about X while conservatively forgetting facts about
possibly aliased P or Q. Joins intersect justified state and retain sound
target envelopes. Correlations not retained automatically need explicit proof;
the checker does not enumerate all branch combinations.

Moving a box or vector descriptor can preserve its heap target. Moving the
inline storage of a self-referential struct cannot simply rename that target.
It must end the old storage identity and establish the new representation,
including any internal links. An invariant requiring intact self-links then
forces backing stability or a checked relocation procedure. Pointer fixups
are program behavior, not an unannounced runtime repair by the checker.

### Dynamic collections, graphs, and loops

A checker must not make one source allocation site stand for one live object,
nor enumerate all future runtime allocations. Use an abstract family F with
a checked ownership/representation predicate. For example:

```text
OwnedFamily(F) = separating collection of the owned resources named by F
LiveFamily(F)  = every member currently has the required live storage
```

These are defined predicates over resources. They are not automatically true
because a programmer wrote their names. Positive inductive definitions,
introduction/elimination, and named finite fold/unfold witnesses supply the
proposed proof mechanism. This extends WF's proof kernel; the current integer
fact system alone does not already implement it.

The generic resource transformations are:

```text
create:  Family(F) -> exists D. Family(F union {D})
         with a fresh live D and checked separation from current members

extract: OwnedFamily(F), D in F
         -> Owner(D) * OwnedFamily(F minus {D})

insert:  Owner(D) * OwnedFamily(F), D not in F, required separation
         -> OwnedFamily(F union {D})

dispose: Owner(D), release conditions -> consumes D's disposal obligation
```

The star separates consumable resource obligations; the union in an access
demand does not. Extracting ownership does not itself deallocate or relocate
storage. It changes where the accounting is held. An actual container
operation has ordinary runtime effects in addition to this proof accounting.

A loop's invariant describes F, the owned collection and its remaining
obligations. Check the body once under that invariant, prove the next invariant,
and check the exit state. Recursive functions use declared contracts in the
same way. A repeated allocation introduces a fresh witness per logical
execution; the static rule is quantified, not a runtime counter.

Two independently opened existential packages are not automatically distinct.
They might hide the same target. Only checked allocation, membership and
separation evidence establishes distinct live storage. Returning a member of
F is not fresh allocation merely because its result uses `exists D`.

For a graph, pointer edges and ownership edges need not coincide. A registry
can own a family of nodes while node fields contain freely copied locators,
including cycles. Removing D consumes the relevant owner and changes the
family to `F minus {D}`. A saved locator P is usable only if current evidence
still establishes its required state; `P in F` does not alone imply
`P in F minus {D}`.

An abstraction promising that every edge remains traversable must restore that
invariant when deleting a node, for example by repairing incoming edges. A
weaker graph abstraction can retain inert locators but cannot dereference them
without proof. This is the graph's chosen contract, not a graph-specific
ownership escape. Arbitrary graph algorithms are not automatically proved by
the compiler; their finite invariants and certificates carry the hard facts.

### Fields, slices, and parallel execution

Under A, several views and the original container locator can coexist.
Sequential access through any of them is allowed when its current domain holds.
There is no intermediate state in which a nominally usable parent is prevented
from touching elements solely because a view exists.

```text
left  = view(data, 0, k)
right = view(data, k, n)
read(data[i])              // sequentially legal if i is in bounds
fill(left); fill(right)   // potentially independent if their ranges are disjoint
```

Range endpoints are captured values. A later assignment to k does not retarget
a view. Unknown indices are not automatically distinct. Non-overlap is proved
from fields, current representation invariants, or captured ranges using the
fixed arithmetic/proof rules. A whole-object write includes every affected
subrange.

Parallel permission requires both complete read/write noninterference and
compatible resource transitions, operation domains, control/data flow, and
observables. Each operation's required current-state facts must remain valid
under the other operation. This is a proof about a proposed overlap of the
sequential program, not runtime permission acquisition.

If independence is unproved, keep source order. If a later operation lacks
memory-safety evidence even in source order, reject it. Neither failure inserts
a lock, generation test, alias test, dependency or scheduling protocol.
Read/read sharing needs no writer-visible fractional-token bookkeeping in
this sequential-source model. Independent overlapping writes still require
disjoint actual footprints, not two copies of the same context.

A call's aggregate row remains conservative over that call. Two long-running
calls that each briefly write the same metadata do not automatically become
independent. Smaller source operations or optional body-based optimization can
expose more opportunities; signature-only checking does not know an internal
access schedule. The proposal removes the reference-lifetime monopoly without
claiming that an unordered effect row captures every possible interleaving.

For two blocks with payloads D1/D2 and shared metadata M:

| Calls | Required result |
|---|---|
| Write D1; write D2 | Can overlap with proved separation. |
| Write D2; release D1, touching M and D1 | Can overlap if the full contracts preserve D2 and no other dependency interferes. |
| Release D1; release D2, both touching M | Source order remains. |
| Release D1; read a saved locator to D1 | Rejected for missing current liveness. |

For IO, distinguish the state operations actually share: handle fields,
open-description offset, file contents, directory namespace, device state,
provider metadata. An ordinary interface exports these associations. Two
wrappers do not prove different files; external aliasing that cannot be
disproved must use a conservative shared footprint. The language's declared
external-environment model still bounds any guarantee about external agents.

### Automatic checking and explicit proofs

The automatic fragment should contain these fixed operations:

1. First-order kind/type checking and argument matching; no impredicative
   qualifier inference or alias-partition enumeration.
2. Shared DAGs for paths, immutable value images, sets and instantiated
   contracts. Equality and supplied relation closure are finite graph tasks.
3. Structural field/range overlap queries plus the existing bounded arithmetic
   derivations. Unknown overlap remains possible overlap.
4. A forward body walk with support-based forgetting and declared join/loop
   invariants. No unbounded whole-program points-to fixpoint is required.
5. Checking written resource introduction/elimination, fold/unfold and family
   witnesses. Search does not recursively unfold arbitrary predicates looking
   for a missing proof. Witnesses name the predicate instance and substitution.

For a straightforward implementation with K retained facts and U declared
write footprints at a call, at most K times U support comparisons suffice
before adding its verified postconditions. This bounds that operation, not
all proof normalization. Each comparison's arithmetic/proof work must obey
its own specification-fixed completion rule. Certificate size and shared term
size must be counted; compressed notation is not permission for exponential
expansion hidden in one checking step.

The experiment's all-pairs symbolic case performs N(N-1) may-alias queries.
That supports a simple non-exponential implementation of this fragment. It
does not establish the complexity of recursive predicates, nested generics,
the complete WF proof system, or a production implementation.

## Candidate B: validity loans, with mutation governed by effects

B uses A's associations, ownership accounting, effect projection, alias-aware
current-value facts, signatures and parallel checks. It changes the reference
formation and destruction rules:

```text
Ref<'a, P, T> = a locator for P plus a validity obligation until 'a ends
```

`'a` means only the validity extent. P names the storage association. Neither
uses the other's syntax or kind. Overlapping writable references are legal;
they do not suspend each other for ordinary compatible accesses.

A borrow formation records a validity obligation against the selected
storage/layout. Copying the reference preserves that obligation. A returned or
stored reference retains it through the type's lifetime parameter. An explicit
end or the declared region boundary discharges it only after all dependent
holders have ended. Destruction, backing replacement and layout changes must
prove that they preserve every still-active obligation, or wait until its
declared endpoint. This does not require inference of last-use endpoints.

This is a general type rule for all objects. Blocks may hold ordinary aliased
references to shared control state and release independently after payload
view obligations end. Graphs can contain cycles within a common validity
scope. A node cannot be individually freed while references to it remain
valid under outstanding scopes; remove those references, shorten the explicit
scope, or retain the node until the scope ends.

The guarantee is storage and layout validity, not the truth of arbitrary
mutable-value facts. Clearing a possibly aliased vector still invalidates a
nonempty proof. If an operation temporarily uninitializes or changes layout
that a valid reference promises, B rejects it while that reference remains
active. This makes B meaningfully simpler and more restrictive than A,
rather than silently falling back to checked-use semantics for the hard cases.

The former `&uniq` exclusion rule is still removed. Whole-container and slice
locators can perform sequential operations normally; effects decide whether
those operations can overlap. B therefore fixes the long-lived write monopoly
without admitting inert references to reclaimed storage.

## A third option and why it is not the recommendation

Explicit location capabilities, as in [L3](https://www.cs.cornell.edu/people/fluet/research/lin-loc/TLCA05/tlca05.pdf),
can also form a whole-language discipline. Aliasable locators travel freely,
while a separate linear capability supplies access to current contents.
Functions explicitly consume and return capabilities; an effect system can
help split disjoint capability fragments for overlap.

This is a useful alternative if explicit authority flow is preferred over an
implicit resource context. It has a clear accounting model, but demanding the
capability through every helper recreates the interface problem motivating
this work. If the compiler infers that flow at each operation and abstracts it
in contracts, the proposal converges toward A; it is not an independent fourth
breakthrough. A family-wide token alone also loses per-payload independence.

Mezzo's [permission system](https://cambium.inria.fr/~fpottier/publis/pottier-protzenko-mezzo.pdf)
is another useful comparison, including implicit permission flow. Its dynamic
adoption/abandon route is not a substitute for the requested erased static
solution. None of these distinctions argues for adding a runtime borrow check
as the default fallback.

## Comparison on the same programs

| Program or property | A: current-state access | B: validity loans | Explicit capabilities |
|---|---|---|---|
| Several long-lived writable aliases, sequential writes | Accepted with current access evidence. | Accepted while validity obligations hold. | Accepted while the separate capability is supplied. |
| Two identical arguments to a sequential writer | Access demand is combined; accepted. | Same. | Needs an alias-permitting capability interface, not two disjoint tokens. |
| Delete graph node while unused locators remain | Allowed; later invalid access fails. | Delete fails until validity obligations end. | Allowed if aliases do not retain the consumed capability. |
| Take a field, do unrelated work, then put it back | Allowed; intervening reads of the hole fail. | Rejected if an outstanding reference promises the affected validity. | Requires a state-changing capability interface. |
| Return/store a reference | Association plus exported current-state contract. | Association plus retained lifetime obligation. | Pointer plus appropriate packaged or separately supplied capability. |
| A growing runtime collection | Inductive resource/family contracts. | Also needs family/ownership accounting; lifetime grouping can simplify validity. | Inductive capability packages; explicit splitting/recombination. |
| Independent views of a common backing object | Proved range effects. | Same, plus validity constraints on structural mutation. | Split capability fragments plus range effects. |
| Shared hidden IO state | Ordinary association and effect contract. | Same. | Shared association plus explicit capability flow. |
| Default reference mental model | Address relationship; each use must be justified now. | Address relationship with a declared validity promise. | Address relationship; bring its separate authority to use it. |

The old WF system remains the baseline for implementation comparison, but it
does not admit these aliasable writable references. Neither A nor B can be
implemented by merely renaming the current store brand or adding another
generic parameter while preserving OWN-5.

## Safety argument and what remains to prove

The proposed invariant relates a checked symbolic state to a physical heap:

1. Every retained current-state assertion holds in every physical state allowed
   by the target-relation environment. Unknown aliases remain included.
2. Every usable storage identity maps to its actual live allocation/range and
   layout. Proved separation implies disjoint relevant physical bytes in the
   same state. Parent/child containment is represented explicitly.
3. Every disposal obligation is accounted for once. Splitting or combining a
   family preserves that accounting; aliasing a locator cannot create an owner.
4. Every runtime access belongs to the instantiated declared footprint and
   has the required live, initialized, typed and domain evidence.

Preservation follows the intended primitive cases: creation establishes a
fresh logical identity for valid storage; writes forget overlapping facts;
takes remove initialization; reclaim removes validity; copies preserve target
identity; owner transfer preserves the backing map; field replacement changes
only the stored link and its dependent facts. Call substitution needs a frame
lemma and universally checked contracts. Loops need invariant induction;
resource packaging needs introduction/elimination and substitution lemmas.

For reuse, retain a historical interpretation of locator values while removing
the reclaimed identity from the live-storage map. A new identity can map to
the former physical range when that range is available. An old locator cannot
pass the live-state premise. No runtime generation check is needed by this
argument. Allocation capacity, overlap, layout, provenance and all actual
allocator actions still require a verified representation refinement.

For parallelism, the proof must combine footprint noninterference with
resource/state stability and the existing observational-equivalence
requirements. Separately, lowering must erase proof/association arguments,
preserve logical-to-physical storage relations, and avoid blanket `noalias`
or invariant-load attributes on aliasable mutable references.

These are proof obligations with proposed rules and invariants, not completed
theorems. The experiment tests the first-order alias/state cases, real slot
reuse, ordinary shared links, and signature substitution. It does not verify
inductive resource predicates, arbitrary higher-order callbacks, full array
layout, native lowering, or a WF-wide soundness theorem. Source-level and
backend performance are unmeasured.

The remaining work is therefore distinguishable from a missing mechanism:
formalize and prove this calculus; implement/check the family certificate
rules and recursive/generic cases; and measure representative graph,
container, IO and parallel workloads through a real backend. A counterexample
or impractical certificate/checking growth would reopen the recommendation.
Neither a green bounded model nor this design authorizes adoption or a claim
that the existing borrow checker can already be deleted.
