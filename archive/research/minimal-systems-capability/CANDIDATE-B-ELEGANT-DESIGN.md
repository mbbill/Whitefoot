# Candidate B Elegant Design

Date: 2026-07-15

Status: bounded paper comparison complete; mandatory Candidate B Design Gate
stop. No alternative in this file changes Candidate B v0 or authorizes an
implementation.

Candidate B Design Gate disposition: `B-REVISE`.

## 1. Result in plain language

The cross-project evidence supports a substantially more elegant B direction,
but it does not yet support selecting it.

`B-STRATA` is the strongest of the three frozen alternatives. It replaces
container-shaped forms such as `Sparse`, `Ring`, or an epoch-reclamation form
with eight project-independent layers: physical places, structural liveness,
owner transitions, scoped progress, physical-root provenance, executable
disposition, invalidatable facts, and concurrent custody. The same layers give
finite end-to-end paper routes to all five Hashbrown operations and mimalloc
small allocation, including its cold collect, extend, retry, and null outcomes.
That is six of the fourteen frozen operations across two materially different
projects.

Eight routes remain open:

1. mimalloc local free expresses the direct block-role transition, but its rare
   unfull, retire, and page-release path still needs exact final page
   disposition and observer exclusion;
2. mimalloc remote free additionally needs a policy-neutral way to prove that
   page-map readers are gone before the page root is released;
3. SQLite insertion/split needs exact pager, journal, allocation, pointer-map,
   and VFS rows in addition to the closed in-memory page relation;
4. SQLite deletion/balance needs the same exact leaves for its page/freelist
   and rollback-required paths;
5. SQLite rollback still needs exact VFS/WAL event rows and a closed
   reinitializer contract;
6. Crossbeam protected load still needs a nonforgeable, zero-per-load way to
   connect a guard, atomic pointer, and collector domain;
7. Crossbeam retirement needs a safe heterogeneous erased one-shot action with
   the source's inline fast path; and
8. Crossbeam collection needs a finite producer of quiescence that does not
   make epoch reclamation language policy or accept a writer proof.

This is not a Hashbrown-only result. Hashbrown accounts for five of the
fourteen routes. mimalloc exposed layout overlays and atomic owner transfer;
SQLite exposed cross-root byte ranges and transaction-level repair; Crossbeam
exposed the unresolved observation, retirement, type erasure, and quiescence
boundary.

The result is therefore `B-REVISE`, not `B-SELECT` and not `B-NONE`.
`B-STRATA` is a concrete next research hypothesis because most of B compresses
cleanly. It is not selectable because every frozen alternative retains at
least one non-closed row. This is paper derivability and structural accounting,
not formal safety, implementation feasibility, generated-code parity, or
measured performance.

## 2. The question being answered

The purpose of a minimum capability set is not to isolate ordinary code from a
privileged implementation. The purpose is to let checked ordinary libraries
preserve the native representations and machine-event paths needed for
performance, while keeping the language's semantic authority small enough not
to grow without discipline.

The test used here is therefore:

> Can a small closed algebra express the exact storage, owner traffic, partial
> progress, cleanup, provenance, and concurrency relations already required by
> several unrelated high-performance systems, without adding one semantic
> family per project and without accepting arbitrary writer propositions?

A capability is useful only when it removes a named performance blocker. A
capability is minimal only when deleting it forces one of the following:

- eager initialization, zeroing, a sentinel value, or a permanent tag;
- an extra copy, move, allocation, indirection, check, scan, or owner record;
- a stronger synchronization event or a per-load/per-object lifecycle event;
- a rejected native representation or operation; or
- a project-specific form or arbitrary proof authority.

The comparison does not reward a small number of names if each name hides an
open proof system, runtime descriptor, policy catalog, or compiler-recognized
container.

## 3. Why four projects are necessary

| Project and frozen operations | New demand beyond the earlier projects | What a no-tax route must preserve |
|---|---|---|
| Hashbrown: lookup, insert, replace, remove, rehash | Control metadata authorizes sparse payload; one physical byte class has a transition-local live meaning during rehash. | No payload initialization for vacant slots, no universal bitmap, direct owner return/replacement, and no persistent rehash tag. |
| mimalloc: allocate, local free, remote free | One block changes between free-node layout and uninitialized caller bytes; remote CAS transfers ownership; page metadata and payload roots can move independently. | No permanent block tag, no atomic event on the local path, no per-block backpointer or count, and no premature page release. |
| SQLite: insert/split, delete/balance, rollback | Packed variable byte ranges cross cache and scratch roots; local failure can require later pager rollback instead of local inversion. | No whole-page copy, no eager zeroing, no inverse closure per mutation, exact page-pin cleanup, and an unusable state after failed repair. |
| Crossbeam Epoch: protected load, retire, collect | Guard-scoped access, unique retirement, heterogeneous deferred actions, and policy-selected quiescence are distinct relations. | No per-load guard event, no per-retire allocation on the inline path, no fixed epoch policy in the language, and no destruction before a grace period. |

The source audit deliberately stops at these fourteen operations. They are a
falsification set, not an ecosystem-completeness claim.

## 4. Rules shared by all three alternatives

The three alternatives inherit the unchanged C0 lower bound. In particular,
C0 owns opaque generativity, generic callable contracts, recoverable outcomes,
allocation roots, atomic and thread events, external events, and target events.
The B alternatives may compose those rows but may not invent machine behavior.

The following meanings are also fixed for this comparison:

- A physical root is a generative allocation identity, not an address-shaped
  integer, page number, wrapper, registry slot, or guard field.
- Owning raw storage grants no typed read, move, borrow, or destruction.
- Runtime representation state is permitted only when the selected native
  representation already needs it. Static versions, capabilities, and facts
  must erase.
- A logical commit can contain several physical writes while exclusive checked
  authority prevents observation of an invalid intermediate state.
- A recoverable result must account for every owner. An abort has no later
  state but may not perform an invalid read, duplicate drop, or race first.
- Cleanup and repair are different. Cleanup closes finite ownership
  obligations; repair may execute a library-selected transaction or lifecycle
  policy and can itself fail into a restricted state.
- Protected observation delays disposition. It does not by itself prove
  publication, same-domain association, payload noninterference, unreachability,
  or quiescence.

The analytical names below are not proposed xlang syntax.

## 5. Alternative 1: B-FORMS

### 5.1 Plain-language definition

`B-FORMS` is the original Candidate B. The compiler knows separate forms for
common topologies, footprints, handles, shared lifecycles, cleanup, and facts.
Ordinary libraries may compose those forms, but cannot define a new form or
state relation.

### 5.2 Exact inventory and deletion witnesses

| ID | Capability | Why it exists; what deletion breaks |
|---|---|---|
| `BF-1` | Tag-free affine storage with no implied payload liveness. | Without it, spare capacity and allocator blocks require initialization, tags, sentinels, or per-object roots. |
| `BF-2` | Compiler-known `Full`, `Prefix`, `Ring`, `Sparse`, `Product`, and `Hole` views. | Each registered shape otherwise uses a structurally more expensive view or needs a new built-in. |
| `BF-3` | Fixed initialize, take, replace, swap, relocate, clone, and destroy transitions. | Owner traffic otherwise acquires extra moves, temporary owners, hidden clone/drop, or rejection. |
| `BF-4` | Scoped topology protocol with non-escapable holes and fixed public exits. | Multi-step mutation otherwise needs a snapshot, persistent tag, or general pre/post logic. |
| `BF-5` | Closed root, field, index, split-range, cursor, and finite product/sum footprints. | Disjoint access otherwise needs copies, repeated checks, runtime borrow tables, or rejection. |
| `BF-6` | Borrow-carrying aggregate and finite result-source maps. | Stored or returned borrows otherwise lose their physical source or require callback-only APIs. |
| `BF-7` | Append-only, generation, retired, and owner-scoped logical handles. | Reuse either revives stale handles or taxes append-only identities with generations. |
| `BF-8` | Unique, strong/weak counted, dynamically borrowed, and synchronized shared lifecycle forms. | Shared libraries otherwise need an unbounded proof relation, universal runtime checking, or trusted internals. |
| `BF-9` | Canonical cleanup per topology and protocol. | Partial states otherwise leak, destroy vacant places, or scan an unrelated extent. |
| `BF-10` | Fixed bounds, membership, footprint, freshness, lifecycle, refinement, and code-shape facts. | Repeated checks remain, or facts become writer assertions. |
| `BF-11` | C0 in full. | Storage forms alone cannot create allocation, atomic, external, target, or callable semantics. |

### 5.3 What ordinary libraries can do

A dense vector selects `Prefix`, a ring selects `Ring`, a hash table selects
`Sparse`, and a multi-root structure selects `Product`. Libraries write their
algorithms using the listed transitions and close each scope to an admitted
view. The compiler owns the meaning and cleanup of every selected form.

### 5.4 Why the multiproject audit does not close it

The original inventory names useful destinations but does not define the
relations exposed by the new sources:

- `Sparse` has no public exact grammar for control-to-payload admission or
  transition-local reinterpretation.
- There is no general state-dependent layout overlay for a mimalloc block.
- Fixed local cleanup does not express SQLite's move to a transaction-level
  rollback-required state.
- Shared-lifecycle forms do not define a policy-neutral producer of
  quiescence or same-domain guard association.
- Canonical topology cleanup does not define Crossbeam's heterogeneous erased
  one-shot action.

All fourteen `B-FORMS` rows are therefore `OPEN`. Adding a new flat form for
each missing topology, recovery path, or reclamation policy would reproduce
the growth problem that this alternative was intended to avoid.

## 6. Alternative 2: B-STRATA

### 6.1 Plain-language definition

`B-STRATA` factors B by semantic job instead of by container shape. A library
does not request a `Sparse`, `B-tree`, `allocator-page`, or `Epoch` capability.
It composes physical places, a closed description of which places are live,
fixed owner transitions, structured progress, root-exact references, finite
disposition, fixed facts, and concurrent custody.

The checker understands the eight layers. A compiler may specialize a valid
composition into direct code, but recognition cannot make an invalid
composition valid.

### 6.2 Exact inventory, reasons, and removal witnesses

| ID | Exact capability | Why it is present | What deletion forces |
|---|---|---|---|
| `BS-1 PHYSICAL-PLACE` | A generative affine root may be partitioned into aligned fields, checked indices, and checked byte ranges. Disjoint parts may be split and reunited. Raw places contain no live value. | All four projects store multiple logical objects or byte ranges under a larger physical allocation. | Eager initialization, per-element roots, page copies, runtime backpointers, or wrapper-rooted references. |
| `BS-2 STRUCTURAL-LIVE` | A closed grammar describes vacant places, live atoms, strided spans, finite products, metadata-classified places, and strict owned-child edges. It is tied to one root and layout version. | The same mechanism derives prefixes, holes, sparse control/payload, free-node overlays, packed byte ranges, and active-prefix bags. | A separate topology family, universal liveness bitmap, permanent role tag, full scan, or arbitrary predicate. |
| `BS-3 OWNER-TRANSITION` | Fixed operations are initialize, take, replace-return-old, swap, overlap-safe relocate, destroy, reclassify, root split/join, and C0-8 atomic publish/take/replace. Every branch names offered, retained, returned, moved, and destroyed owners plus one logical commit. | The audited sources use materially different traffic that cannot be decomposed without transient owners or extra movement. | Clone, temporary boxes, extra moves, owner reconstruction, or project-named operations. |
| `BS-4 FOCUS-PROGRESS` | Closed states are stable, exclusive focus, monotone progress with carried owners, repair-required, and poisoned. Intermediate roles cannot be used as stable state. A repair token may move through sealed internal calls but exposes only resume, repair, poison, or disposition operations. | Rehash, detached-list processing, multi-page balance, pager rollback, and bounded collection all make partial progress. | Snapshots, inverse closures, persistent per-operation tags, mandatory local rollback, or a general protocol proof language. |
| `BS-5 ROOT-FOOTPRINT` | Every borrow leaf records physical root, layout version, byte interval, access mode, and invalidators. Source maps are finite products or tagged sums. Wrapper moves and logical rekeying preserve the root. | Each audited project moves metadata, handles, page numbers, scratch owners, or queue nodes independently from the referenced allocation. | Copies, dynamic borrow tables, per-use root checks, per-block backpointers, or wrong-root acceptance. |
| `BS-6 EXECUTABLE-DISPOSITION` | A closed finite fold consumes live spans, metadata-classified live places, bounded detached chains, strict owned children, finite products, exact release rows, and admitted one-shot disposition callables. Every returning step reduces a closed measure or consumes a strict child. | Partial rehash, detached free chains, page pins/scratch, and active deferred prefixes need executable owner closure, not a liveness proposition. | Leaks, full-capacity scans, duplicate destruction, one finalizer per container, or arbitrary writer cleanup programs. |
| `BS-7 INVALIDATABLE-FACT` | Fixed schemas cover bounds, alignment, live role, same-root, disjointness, current version, protocol state, publication, and noninterference. Producers and exact invalidators are closed; facts-off retains checks and semantics. | Existing comparisons and transitions should discharge repeated checks and inform aliasing without becoming assertions. | Retained bounds/alias checks, stale optimizer authority, family fact catalogs, or writer-defined propositions. |
| `BS-8 CONCURRENT-CUSTODY` | Atomic custody transfer, protected observation, payload interference, retirement, and quiescence are distinct resources. Successful atomic unlink may create one `Deferred(domain, root, cutoff)` right; destruction also needs `Quiescent(domain, cutoff)`. Moving either token never re-roots the target. | mimalloc and Crossbeam require local versus remote paths, zero-event protected loads, exact retirement, and delayed destruction without one mandated reclamation product. | Global locks, per-load counts or checks, per-object epochs/hazards, duplicate retirement, premature release, an epoch-specific language form, or a writer assertion. |

`BS-1` through `BS-8` inherit unchanged C0 machine leaves. C0 is not a ninth
topology layer.

### 6.3 The closed structural-live grammar

The intended normalized grammar is:

```text
Place :=
    physical-root
  | field / checked-index / checked-subrange
  | disjoint split / join

Live :=
    vacant(place)
  | atom(place, layout)
  | span(base, count, stride, layout)
  | classify(metadata, scalar-decoder, affine-place-map, role-table)
  | owned-edge(place, child-root)
  | Live x Live
```

The scalar decoder is deliberately narrow: fixed-width metadata loads,
equality, masks, ranges, null tests, checked integer arithmetic, and checked
affine place mapping. It contains no source call, loop, recursion, quantifier,
arbitrary predicate, proof term, or cleanup code.

A classifier declaration does not create liveness. Its certificate must begin
from freshly vacant storage, a fixed C0 adoption or external-event row that
establishes every live layout, or a valid predecessor transition. An ordinary
refinement predicate cannot create liveness. Once sealed, private role metadata
may change only through `BS-3`. This prevents a forged control byte from
granting access.

SQLite does not use foreign validation to create typed values. Its input row
initializes a whole page as bytes; `BS-2` then derives checked byte subranges
inside that already-live byte root. The two closed SQLite mutation routes do
not depend on a predicate minting payload authority.

The grammar derives familiar shapes rather than naming them:

- full and prefix storage are one span;
- ring and hole storage are products of disjoint spans;
- Hashbrown is a metadata classifier plus an affine slot map;
- a mimalloc free list uses a role overlay and strict owned edges;
- SQLite cells use checked subranges and finite products; and
- a Crossbeam bag is one active span over a fixed-capacity root.

If an independently required native representation cannot be normalized into
this grammar, that is evidence against `B-STRATA`. It is not permission to add
the project's name to the language.

### 6.4 Owner transitions and logical commit

The transition table distinguishes operations with different source traffic:

- `initialize` consumes a vacant place and one offered owner;
- `take` consumes one live place and returns its owner;
- `replace-return-old` installs one offered owner and returns the displaced
  owner without making both live in the same place;
- `swap` requires proven-disjoint live places;
- `relocate` handles admitted overlap and moves every owner exactly once;
- `destroy` consumes one live owner;
- `reclassify` changes a place between admitted layouts, such as free-node and
  uninitialized caller bytes; and
- atomic publish/take/replace attaches owner transfer to the successful C0-8
  event while a failed or retried compare-exchange retains the offered owner.

These operations remain separate because lowering one through another would
change owner movement, temporary state, or memory traffic in at least one
audited source.

### 6.5 Progress, repair, and poisoned state

`BS-4` does not define one automaton per algorithm. It defines how ordinary
control flow may carry a finite checked state:

- `Stable` permits the ordinary public operation set.
- `Focus` is exclusive and lexical; old stable facts cannot be used.
- `Progress` carries a structural cursor, the remaining obligations, and any
  displaced owner.
- `RepairRequired` may be passed only through sealed repair-capable functions.
  It cannot be interpreted as the former stable owner.
- `Poisoned` exposes only declared close, retry-from-external-source, or
  release operations.

This is why SQLite does not need an inverse closure for every B-tree write.
Insertion or deletion may release its local pins and return a sealed
`RepairRequired` pager state. The library-selected journal or WAL protocol then
either returns a stable reader or moves to `Poisoned`. The language checks the
resource transitions but does not prove B-tree ordering or database crash
consistency.

### 6.6 Finite disposition, not arbitrary cleanup

The smallest credible cleanup mechanism separates the source of outstanding
owners from their terminal action:

```text
Source :=
    live-span
  | metadata-classified-live-set
  | bounded-detached-successor-chain
  | strict-owned-children
  | finite-product(Source...)

Action :=
    destroy
  | release-through-exact-row
  | transfer-to-stable-sink
  | invoke-once

Dispose :=
    drain(Source, Action)
  | case(closed-state, finite-arms)
  | close(stable-or-restricted-outcome)
```

Every step removes one unique obligation before executing the action. The
closed decreasing measure is remaining live places, existing bounded fuel,
strict affine child ownership, or a lexicographic finite product. The grammar
does not accept a writer predicate, callback-selected successor, inverse
program, arbitrary loop, or writer termination proof.

This closes only ownership disposition:

- Hashbrown can destroy exactly phase-local pending owners after recoverable
  partial rehash without scanning them on success.
- mimalloc can consume a detached bounded chain, but page release/reclaim/
  re-advertise remains an explicit lifecycle protocol.
- SQLite can release exactly acquired page pins and scratch roots, but journal
  replay remains an explicit repair protocol.
- Crossbeam can iterate only the active bag prefix, but the current common
  callable capability cannot yet represent its heterogeneous erased one-shot
  actions without changing the source's representation.

A callback that diverges has no returning successor. A callback trap follows
the language abort edge. Normal return must consume the one-shot disposition
exactly once. Eventual reclamation is a policy/progress property, not a memory
safety promise of the disposer.

### 6.7 Concurrent custody: the exact unresolved boundary

`BS-8` deliberately separates five facts:

1. an atomic edge owns or publishes a target;
2. one successful event transfers that custody;
3. an observation delays destruction in one generative domain;
4. publication ordering and payload interference independently authorize the
   actual payload access; and
5. a quiescence certificate permits destruction of a retired target.

The first four relations are finite in principle. The fifth is not yet closed.
For Crossbeam, a producer would have to justify its participant scan, modular
epoch rule, barriers, and two-advance expiry. For mimalloc, it would have to
justify a different page-map reader protocol. Hard-coding those policies makes
B grow toward C. Letting a library state an arbitrary observer predicate,
happens-before invariant, or unreachability proof makes B grow toward A.

The current evidence therefore defines the required output
`Quiescent(domain, cutoff)` but not a finite safe producer. A requirement name
is not a capability. The affected rows remain open.

### 6.8 Ordinary-library derivations

#### Hashbrown

One physical root is partitioned into control and payload regions. A closed
byte classifier produces a live-slot role only for the corresponding FULL
class under the matching root and version. Insert, replace, and remove use
direct owner transitions. Rehash opens an exclusive progress state in which
the physical DELETED byte may mean pending-live; that interpretation cannot
escape. Classified disposition handles only a recoverable partial state.

No Hashbrown name, hash-table operation, vacant payload initialization,
universal bitmap, or permanent phase tag enters the language.

#### mimalloc

A page root is partitioned into block subowners. A block is reclassified from
an intrusive free-node prefix plus vacant remainder to caller-owned
uninitialized bytes and back. Allocation's hot pop is one direct transition.
Its cold path uses structured progress to collect a bounded detached remote
list, extend a page by initializing only intrusive link words, acquire a fresh
root through C0, retry once, or return null without a block owner. These cold
steps add no event to the protected hot pop.

Local free uses direct non-atomic custody for the ordinary relink, then may
move the page through unfull, retired, or released state. Remote free attaches
custody transfer to successful compare-exchange and later moves the detached
chain through ordinary progress. The physical page allocation, not its
metadata object or page-map slot, remains the root.

The complete allocation route closes. Local and remote free remain open at
final page disposition: the frozen rules do not yet justify safe page release
against every page-map observation or supply every exact release event.

#### SQLite

Cache pages, scratch pages, overflow pages, journal scratch, and metadata are
distinct roots. Checked byte subranges and exact source maps permit in-place
dependency-ordered movement without whole-page exclusivity. A bounded balance
scope carries its acquired page pins and scratch owners. Failure releases local
resources and returns a repair-required pager owner rather than allocating an
inverse action for every mutation.

The in-memory packed-page, owner, provenance, local cleanup, and
rollback-required parts of insertion and deletion factor through `B-STRATA`.
The complete frozen operations remain open, however, because journal-before-
write, pager allocation/free, pointer-map/file-policy work, and the path to
rollback require exact C0 rows that this bounded work did not enumerate. Full
rollback additionally needs complete VFS/WAL and reinitializer contracts.

#### Crossbeam Epoch

A protected pointer borrow must retain the target allocation root and one
static collector domain; moving a deferred disposition through local bags and
the global queue does not re-root it. A successful checked unlink, not a copied
address, creates the one retirement right. An active-prefix disposition would
execute only initialized deferred slots.

The exact guard constructor, heterogeneous erased one-shot callable, and
quiescence producer remain open. Epoch values, bag capacity, scan frequency,
and queue-pop limits stay ordinary library policy.

### 6.9 Expected implementation shape

- The checker normalizes the closed liveness grammar, validates fixed
  transitions, tracks structured progress, and maintains root/footprint facts.
- The compiler lowers transitions to ordinary moves, loads, stores, drops, and
  selected C0 events. Static roles, versions, phases, and source maps erase.
- The backend sees only representation-selected metadata and exact C0 rows; it
  has no Hashbrown, allocator, B-tree, or epoch opcode.
- Generated disposition is a specialized loop or strict-child walk, not a
  runtime descriptor and not capacity-unrolled code.
- Diagnostics name the physical root, role, transition, outstanding owner,
  invalidator, or missing quiescence producer rather than a container family.
- An AI writer chooses from eight semantic jobs and fixed transitions, not from
  an expanding list of nearly identical container operations.

## 7. Alternative 3: B-GRAPHS

### 7.1 Plain-language definition

`B-GRAPHS` exposes a lower-level finite state-machine construction kit.
Ordinary libraries declare a finite protocol graph whose nodes carry closed
place, owner, borrow, progress, and disposition tokens. Edges may use only
fixed primitive transitions and C0 events. The graph itself is checked and
then specialized.

This is the extensibility control. It is still B only while graph validity is
local and finite. Arbitrary invariants, quantified heap or participant state,
writer cleanup, or theorem proving changes it into A.

### 7.2 Exact inventory and deletion witnesses

| ID | Capability | Why it is present; deletion witness |
|---|---|---|
| `BG-1` | Physical roots, checked places, and finite root maps. | Without them, graph tokens can describe state but cannot authorize exact memory or preserve provenance. |
| `BG-2` | Closed primitive tokens for vacant/live places, owners, borrows, atomic custody, observations, retirement, and disposition. | Without them, graph nodes become untyped names or writer assertions. |
| `BG-3` | Library-declared finite graph nodes containing finite products or sums of primitive tokens. | Without nodes, multi-step protocols fall back to built-ins or cannot cross ordinary function boundaries. |
| `BG-4` | Closed edges for the fixed owner transitions, root maps, exact C0 events, and logical commit. | Without edges, graphs cannot change real storage or ownership. Arbitrary edge relations would converge to A. |
| `BG-5` | Closed guards and structural cursors using equality, ranges, role tables, finite classification, and checked decreasing progress. | Without guards, the graph cannot branch on representation state. Arbitrary predicates or loop invariants converge to A. |
| `BG-6` | Fixed fact invalidation and the same finite terminal disposition grammar as `BS-6`. | Without it, graphs can strand owners or mint stale facts. Writer cleanup graphs are forbidden. |
| `BG-7` | C0 in full. | Graphs cannot invent allocation, atomics, external events, or target semantics. |

### 7.3 What it can and cannot derive

A finite graph can spell the five Hashbrown protocols, complete mimalloc small
allocation, the ordinary local-free relink, and the in-memory SQLite balance
scopes. It can make every owner-carrying state explicit and specialize away
graph identity. Only the first six are complete frozen operations; local page
disposition and SQLite's exact external leaves keep the other three open.

Its weakness appears when a local finite graph must justify a global fact.
Neither a participant registry of unbounded runtime size nor page-map readers
can be summarized as quiescent merely because a writer drew a graph edge. The
frozen graph alternative has no safe producer for those facts. The current
evidence does not prove that every possible finite project-independent B
producer fails, so these routes are `OPEN`, not `CONVERGES-A`. A future writer
invariant would converge to A; a policy-specific compiler rule would converge
to C. SQLite rollback also retains the shared external-event and callback gap.
Crossbeam retirement retains the erased one-shot action gap even before
collection.

The graph vocabulary has fewer top-level names than `B-STRATA`, but ordinary
libraries must repeatedly spell larger protocol graphs. Small semantic name
count therefore does not imply smaller checker state, diagnostics, generated
code, or AI construction space.

## 8. Exact 42-route result

The companion TSV contains exactly three alternatives times fourteen
operations. Its used route states have the following meaning:

- `CLOSED` means the frozen paper rules identify no structural event beyond
  the pinned source contract;
- `OPEN` means a named semantic rule or machine leaf is missing.

No route is labeled `TAXED`, `CONVERGES-C`, `CONVERGES-A`, or `UNKNOWN`. This
does not prove that no implementation tax, convergence, or unknown source
behavior exists. It means the bounded evidence establishes definition gaps but
does not establish that one particular convergence path is unavoidable, before
any candidate execution or measurement.

| Alternative | `CLOSED` | `OPEN` | `CONVERGES-A` | Interpretation |
|---|---:|---:|---:|---|
| `B-FORMS` | 0 | 14 | 0 | The original flat vocabulary names relevant shapes but leaves every audited route's new admission, overlay, repair, provenance, callback, or concurrency relation underdefined. |
| `B-STRATA` | 6 | 8 | 0 | Project-independent layers close Hashbrown and complete mimalloc small allocation; page disposition, exact SQLite external leaves, and concurrent reclamation remain open. |
| `B-GRAPHS` | 6 | 8 | 0 | Finite local protocols close the same six operations, but full local-free, SQLite, and shared-observer routes exceed the frozen graph and C0 leaves. |

The six routes closed by both `B-STRATA` and `B-GRAPHS` are:

- all five Hashbrown operations;
- mimalloc small allocation, including its frozen cold outcomes.

`B-STRATA` and `B-GRAPHS` both leave mimalloc local/remote free, all three
SQLite operations, and all three Crossbeam operations open. They reach some of
those open states through different boundaries: the strata design names
missing project-independent producers, while the graph design cannot authorize
the corresponding global edge with its frozen local vocabulary.

## 9. Structural-cost comparison

All entries are paper expectations, not measurements.

| Dimension | B-FORMS | B-STRATA | B-GRAPHS |
|---|---|---|---|
| Extra initialization or zeroing | Intended zero when the selected topology is exact; underdefined overlays block the claim. | None identified for the six closed routes; vacant places and role overlays are explicit. | None identified for the six closed routes; graph tokens erase. |
| Extra payload copy or relocation | Topology-specific rules can avoid it, but SQLite subrange and rehash relations are incomplete. | Exact transitions and root footprints preserve source copies/moves on closed routes. | Primitive edges can preserve them, at the cost of larger repeated graphs. |
| Owner movement | Fixed transition table is promising but does not account new partial states. | Every closed edge names offered, displaced, returned, carried, and destroyed owners. | Explicit in graph nodes and edges; state size grows with the protocol. |
| Runtime metadata fields | Each selected form intends to carry only native counters/metadata; flat-form growth risks accidental headers. | Static roles, roots, versions, and progress erase; only representation-selected state remains. | Graph identity erases, but implementations must show that node/state encoding does not materialize. |
| Indirection and dynamic dispatch | None intended for ordinary forms; erased disposition is missing. | None on closed routes. Crossbeam's existing one indirect call is not yet safely representable. | None on closed routes; graph dispatch must specialize statically. |
| Checks and branches | Closed facts intend to reuse checks but exact invalidation is incomplete. | Existing representation checks may mint scoped facts; unsupported facts retain checks. | Closed guards map closely to branches, but repeated graph boundaries may duplicate checks. |
| Allocation | No generic descriptor intended. Missing forms may tempt wrapper allocation. | No extra allocation identified on closed routes; roots come only from selected C0 rows. | No graph allocation is permitted; static specialization is required. |
| Scans and asymptotics | Canonical cleanup risks wrong-extent or full-capacity scans. | Disposition follows live spans, classified sets, or existing bounded chains. | Graphs may use structural cursors, but global quiescence cannot be asserted by a local edge. |
| Atomics, fences, and synchronization | B-8 is too underdefined to claim exact events. | Only selected C0 events execute; the unresolved quiescence producer blocks stronger claims. | Local event edges are exact, but global proof edges converge to A. |
| Required code and code size | Topology x protocol x cleanup interaction growth is high. | Expected proportional to selected forms and action bodies; cross-products remain a falsifier. | Repeated explicit graphs create the greatest specialization and diagnostic-volume risk. |
| Optimized code shape | Family-like forms permit canonical lowering but can accrete backend paths. | Generic compositions must normalize to direct operations; not yet inspected. | Arbitrary finite graph specialization is the hardest of the three to keep predictable. |

## 10. Broader design comparison

| Dimension | B-FORMS | B-STRATA | B-GRAPHS |
|---|---|---|---|
| Conceptual semantic inventory | 11 entries, including six named topology views and four named shared-lifecycle forms. | Eight orthogonal layers plus inherited C0. | Six graph-specific layers plus inherited C0, but each library supplies a protocol graph. |
| Independent operations covered by one rule | Low to medium; topology rules are reused locally. | Highest in the audit: place, transition, progress, provenance, and disposition rules repeat across three or four projects. | Primitive edges repeat widely, while complete graph definitions repeat per protocol. |
| Representation freedom | Limited to admitted topology forms. | Libraries choose representation metadata inside one closed structural grammar. | Highest for finite local protocols; global invariants exceed the boundary. |
| Cleanup generality | One canonical traversal per topology/protocol. | Orthogonal source x terminal-action fold. | Same terminal fold plus graph-directed state, with greater interaction risk. |
| Provenance precision | Separate footprint and aggregate forms, currently incomplete across moving roots. | One physical-root leaf model shared by storage, repair, and deferred disposition. | Precise if every graph token carries the root map; verbose. |
| Concurrency-policy freedom | Fixed shared-lifecycle forms risk policy lock-in. | Safety relations are separated, but quiescence production remains deliberately unresolved. | Writer graphs offer apparent freedom but require A-like invariants for global safety. |
| Checker state | Many independent form tables and interaction rules. | Normalized grammar plus flow-sensitive structured states. | General finite graph validation and specialization. |
| Compiler/backend specialization | Per-form canonical paths. | Normalize generic compositions into ordinary operations. | Specialize arbitrary admitted graphs; greatest code-shape uncertainty. |
| Diagnostic locality | Good when one form is missing; poor at cross-form interactions. | Errors can name one semantic layer and exact root/owner/state. | Errors may span several nodes and edges. |
| Writer-visible complexity | Many named forms and conversions. | A small repeated pattern by semantic job. | Fewer primitive names but much larger per-operation declarations. |
| AI-writing stability | Familiar closed forms, with pressure to choose near-matching forms. | Best current hypothesis: fewer semantic jobs and local repair targets. | Largest construction and error-repair space. |
| Extension pressure | Add topology, footprint, lifecycle, cleanup, and fact forms. | Add a rule only for a repeated irreducible relation that does not fit the grammar. | Add primitive tokens/guards or silently enlarge graph logic. |
| Interaction growth | Form cross-products. | Layer interactions, intended to normalize through one root/owner/state account. | Node x edge x guard x token x cleanup graph space. |
| C convergence | Highest if every held-out gets a named form. | Occurs if project/topology/reclamation identities enter a layer. | Occurs if hard global cases receive compiler graph templates. |
| A convergence | Low until a form accepts predicates. | Occurs if classifiers, progress, cleanup, or quiescence accept writer propositions. | Highest: arbitrary node invariants turn the graph kit directly into a proof system. |

## 11. Pros and cons

### B-FORMS

Pros:

- Concrete local checker tables and recognizable diagnostics.
- Canonical lowering is plausible for each admitted form.
- Simple dense and ring structures need little explicit protocol state.

Cons:

- The multiproject relations fall between the existing flat forms.
- Each repair tends to add a topology, phase, cleanup, provenance, or lifecycle
  case and its cross-product interactions.
- It is the B alternative most likely to evolve toward C while retaining less
  complete family contracts than C.

### B-STRATA

Pros:

- Eight project-independent jobs close six complete operations across
  Hashbrown and mimalloc and factor the unresolved SQLite/Crossbeam relations
  without project or container identity.
- State-dependent representation, owner traffic, repair, provenance, and
  cleanup share one account instead of separate family tables.
- The finite disposition fold cleanly separates local resource closure from
  library-selected rollback and reclamation policy.
- Static authority can erase while retaining only native metadata and events.
- The missing concurrency boundary is explicit instead of hidden inside a
  generic shared-lifecycle form.

Cons:

- The structural classifier grammar and overlay rules still need a precise
  safety model and held-out validation.
- A policy-neutral quiescence producer is absent.
- Crossbeam exposes an erased one-shot callable gap outside the current direct
  callable contract.
- Generic strict-tree cleanup with guaranteed O(1) machine stack is not closed;
  lists work, while branching trees need representation-selected support.
- Normalization, code-size bounds, diagnostics, AI behavior, and exact code
  shape are unmeasured.

### B-GRAPHS

Pros:

- Finite local state machines are highly flexible without naming containers.
- Owner disposition and partial progress are explicit at every edge.
- It is a useful control for testing whether fixed strata reject an otherwise
  local finite protocol.

Cons:

- Graph declarations repeat protocol detail and enlarge writer/AI search space.
- Local graphs cannot justify global observer, reachability, or quiescence
  invariants.
- Adding writer invariants converges directly to A; adding compiler templates
  converges to C.
- Specialization, diagnostics, compile time, and code-size predictability are
  weaker than `B-STRATA`.

## 12. Hostile review of B-STRATA

1. **Project-name laundering:** no layer admits a project, container, API,
   algorithm, symbol, or path identity.
2. **Predicate laundering:** the structural classifier is closed scalar and
   affine-place grammar. A source call, recursive decoder, quantifier, or proof
   term is rejected.
3. **Metadata or a writer validator forges liveness:** rejected after hostile
   correction. A classifier must be paired with vacant creation, a fixed C0
   row that establishes every live layout, or a checked transition. C0-4
   refinement validation cannot create liveness or ownership. SQLite starts
   from already-live bytes and derives only checked byte subranges.
4. **Role overlay creates two layouts:** rejected. Exclusive focus consumes the
   predecessor role before reclassification and cannot expose both layouts.
5. **Logical identity becomes provenance:** rejected. Every leaf retains its
   physical allocation root across page-number, wrapper, heap, registry, bag,
   and queue moves.
6. **Failure silently restores state:** rejected. `RepairRequired` and
   `Poisoned` are distinct from stable state; local cleanup need not promise an
   inverse database or lifecycle operation.
7. **Cleanup becomes arbitrary code:** rejected by the closed source/action
   fold. Policy loops remain explicit protocols.
8. **Callback termination is assumed:** rejected. Divergence has no successor
   and trap aborts. Exact normal-return disposition is the safety promise.
9. **Strict recursion hides a worklist:** rejected for branching structures.
   Only linear strict-child descent has a current O(1)-stack lowering; a
   general branching route remains unclaimed.
10. **Guard presence grants payload access:** rejected. Observation,
    publication, same-domain association, noninterference, and quiescence are
    separate.
11. **Quiescence is merely renamed:** confirmed as an open blocker. No route is
    closed by the `Quiescent` output name alone.
12. **Static tokens hide runtime metadata:** prohibited. Root IDs, versions,
    roles, progress, and facts must erase unless the selected representation
    already stores corresponding state.
13. **Compiler recognition grants safety:** prohibited. Specialization may
    change code shape only after the generic composition is valid.
14. **Success-path cleanup scan:** prohibited. A disposer exists only on the
    outcome that owns outstanding obligations and may enumerate only their
    exact source.
15. **External effects are hand-waved:** rejected after independent review.
    SQLite insertion, deletion, and rollback all remain open until their exact
    pager, VFS/WAL, pointer-map, release, and callback rows exist.
16. **A hot subpath stands for a whole operation:** rejected after independent
    review. mimalloc allocation now accounts for cold collect/extend/retry/null;
    local free is open because its rare page transition is not closed.

The independent reviews found no new closed route. They changed three
overclaimed `B-GRAPHS` convergence rows to `OPEN`, removed the validator-to-
liveness authority hole, required complete rather than hot-subpath accounting,
downgraded local free and four SQLite mutation routes, and preserved the
`B-REVISE` gate.

## 13. What would change the result

`B-STRATA` can become a `B-SELECT` research hypothesis only if a separately
authorized revision supplies all of the following without changing the six
closed routes:

1. a finite, deterministic, policy-neutral quiescence producer that covers the
   two audited reader protocols without a per-load/per-object tax;
2. a safe erased one-shot disposition capability with exact inline/boxed
   representation, call cardinality, effects, provenance, cross-thread
   behavior, and abort semantics;
3. exact SQLite VFS/WAL and page-reinitializer C0 contracts; and
4. hostile safety and structural validation showing that the classifier,
   overlay, progress, root, disposer, and fact rules neither accept an invalid
   route nor materialize hidden runtime state.

If quiescence repeatedly requires policy-specific forms, the evidence moves
toward C. If it requires ordinary libraries to state and prove arbitrary global
invariants, the evidence moves toward A. If neither direction can preserve the
native event paths safely, Candidate B may move from `B-REVISE` to `B-NONE`.

## 14. Candidate B Design Gate

The exact disposition is `B-REVISE`.

- `B-FORMS` has fourteen open rows.
- `B-STRATA` has six closed and eight open rows.
- `B-GRAPHS` has six closed and eight open rows.
- Therefore no alternative closes all fourteen operations, and `B-SELECT` is
  prohibited by the frozen gate.
- The storage, owner, progress, provenance, cleanup, and fact compression is
  strong enough that `B-NONE` would also overstate the evidence.

Retain `B-STRATA` only as the best-defined B revision hypothesis. Work stops at
this gate. No safety model, additional audit, language or specification change,
checker/compiler/runtime work, prototype, candidate execution, generated-code
inspection, benchmark, AI trial, standard-library work, or production decision
is authorized.
