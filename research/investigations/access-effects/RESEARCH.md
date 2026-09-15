# Access effects and association polymorphism

Research date: 2026-09-15. This is a language-design investigation, not an
amendment to the active specification or a claim of compiler support.
The specification baseline is `277a184473875e28da2e1aa9461c9cc99fc9d63f`.

This investigation serves the experiment of expressing independently usable
allocator blocks, individual reclamation and reuse, and shared external state
through one static mechanism. It records literature, semantic witnesses,
candidate rules, and the questions a compiler experiment must discriminate.
Maintain it as that question evolves; supersede it in place if a subsequent
investigation replaces its analysis. There is no implementation work queue here.

## Finding and scope

There is substantial precedent for separating a pointer from the permission
to access its target. There is also precedent for generic, caller-instantiated
tracking of aliasing and effects. Their combination is a credible alternative
to WF's rule that a live usable exclusive reference excludes other overlapping
access paths. No source examined establishes the soundness or practical cost
of the complete WF combination: movable owned aggregates, arbitrary stored
associations, explicit reuse, checked partial operations, and proof-derived
parallelism with specification-fixed, non-exponential checking.

The most promising hypothesis to test is **aliasable references with explicit
association information, access effects, and resource-state contracts**.
Associations describe targets; effects describe accesses; resource contracts
describe which operations preserve or consume storage validity and ownership.
The access effects are also a consumer of those associations when deriving
parallel permission. The hypothesis changes the permission semantics; it is
not a way to duplicate current `&uniq` authority.

Three clarifications from the initiating discussion constrain the research:

- A release that changes allocator metadata and ends the existence of data
  affects both. Its complete write footprint can express the conflict with
  data access. A separate `invalidates` effect category is not required merely
  to discover that conflict.
- The proposed association parameter describes relationships between object
  instances, not allocator classes and not the duration of a borrow.
- The motivation is operation-level access granularity. A percentage of time
  spent accessing a resource is illustrative, not a request for temporal
  annotations, phase contracts, or runtime scheduling protocols.

The delivery is a research analysis. The examples below are pseudocode and
deductions under candidate rules, not WF conformance results. No implementation,
new source acceptance rule, or performance improvement is claimed.

## Requirements and discriminating criteria

These criteria precede any implementation experiment that uses this analysis
to choose a design. They are requirements for comparing candidates, not results.

1. Several blocks retain their allocator association while allocation,
   ordinary data access, and explicit individual release remain expressible.
   The caller need not thread an allocator value through every release.
2. Distinct block payloads can be proved independent even when their control
   pointers reach the same allocator. Releases through the same metadata stay
   ordered; distinct wrapper values do not establish independence.
3. Releasing a block and allocating again may reuse the same physical bytes.
   A stale alias cannot become valid just because the address is reused.
4. Ordinary fields, generic containers, results, and opaque contracts preserve
   associations. No Arena-, Block-, File-, or host-specific permission rule.
5. Calls use checked signatures. Unknown aliasing remains conservative;
   acceptance does not depend on the optimizer inspecting a body.
6. There are no inserted borrow counters, generation checks, locks, alias
   tests, or dependencies needed to repair an unproved source operation.
   Existing allocator metadata and ordinary algorithmic branches are allowed.
7. Every read has live, initialized, suitably typed storage; moves and
   releases cannot duplicate ownership; mutations cannot preserve invalid
   current-state proofs. Callbacks must obey the same rules.
8. Automatic derivations have fixed terminating rules and practical,
   non-exponential cost. Harder spatial facts may use WF's finite explicit
   proof mechanism. Unknown overlap may deny optimization; missing safety
   evidence must reject the operation.

The first decisive program should allocate three blocks, retain aliases,
release the middle block, allocate a replacement into the available space,
and use the two surviving blocks. A second should use two handles connected
to the same external state. These distinguish the target from a scalar-cell
demo, bulk-only arena reclamation, and wrappers that hide interference.

## What current WF provides and what this reopens

The [active specification](../../../spec/kernel-spec.md) is authoritative.

| Current rule | Relevant behavior | Candidate question |
|---|---|---|
| OWN-5, OWN-7 | Usable overlapping exclusive loans conflict; fields and proved view ranges establish separation. | Can exclusivity be checked over operations while references remain aliasable? |
| OWN-3, OWN-4, OWN-10 | Regions and backing-storage requirements control borrow validity. | Retain lexical validity initially, or express more validity through resource-state transitions? |
| PROV-1 | The store identity is a region and appears as a store brand. | Give object/footprint associations a distinct static kind from lifetime regions. |
| PROV-6, BLK-2 | Release authority and linearity are controlled; the current Arena is not an individual-free allocator. | Express individually reclaimable blocks and control-state access through general contracts. |
| STOR-5, FN-1 | Stored loan provenance is deferred; direct returns have bounded origin rules. | Export per-component retained associations without recovering caller contracts from callee bodies. |
| EFF-1, EFF-2 | Formal-rooted paths describe exact structural effects; effects do not grant write authority; owned-value history is not followed. | Let ordinary reference fields and explicit abstract associations resolve effect targets after moves and returns. |
| ENT-5 | Writes invalidate the connection between current storage and dependent facts. | Apply this consistently through all possible aliases, including structural-state changes. |
| FN-1, FN-2 | Signatures contain caller obligations; current generic instances have concrete rechecking. | Add relation substitution without making safety rely on unbounded specialization. |
| PAR-1 | Parallel permission is optional; full-call exclusive loans can prohibit overlap despite narrow effects. | Use checked access footprints and resource/data dependencies without the persistent-loan veto. |

The relevant grounds are in [ownership](../../../design/language/ownership.md),
[slice provenance](../../../design/language/ownership/slice-result-provenance.md),
[linearity](../../../design/language/ownership/linearity.md),
[effects](../../../design/language/effects.md),
[parallelism](../../../design/language/parallelism.md),
[permission judgment](../../../design/language/parallelism/permission-judgment.md),
and [HandleFactory](../../../design/language/system-interface/handle-factory.md),
under [language](../../../design/language.md) and the
[constitution](../../../docs/constitution.md).

Earlier [ordinary-host-value reasoning](../ordinary-host-values/DESIGN.md#effects-and-cleanup)
removed implementation-derived ancestry and required explicit shared state.
That objection still applies to a hidden origin table. The new possibility is
an ordinary, checked association contract plus a different access rule.
Renaming the old ancestry table, or adding only a brand while retaining all
old permission rules, does not answer the new question.

## Primary-source research

The entries distinguish what a source establishes from its usefulness here.
Search-engine crawl dates were not treated as publication dates. Formalization
and implementation claims below are the authors' claims; their proof builds
and performance results were not independently reproduced in this study.

| Source and inspected material | Contribution relevant to WF | Boundary of the evidence |
|---|---|---|
| [Alias Types](https://www.cs.princeton.edu/~dpw/papers/alias.pdf), ESOP 2000, abstract and the L3 comparison | Represents sharing explicitly while admitting destructive operations; function interfaces carry store information. A direct ancestor of association-indexed references and state contracts. | A low-level calculus, not a demonstration of ergonomic inference or WF's parallel semantics. The original PDF's extracted body text was unusable in this inspection. |
| [Typed Memory Management in a Calculus of Capabilities](https://www.cs.cornell.edu/talc/papers/capabilities.pdf), POPL 1999, sections 1–2 | Static capabilities admit non-lexical region reclamation; bounded quantification tracks capability aliasing. Capabilities erase. | Region-level memory management in a CPS intermediate language; not payload-level effects over WF objects. |
| [L3: A Linear Language with Locations](https://www.cs.cornell.edu/people/fluet/research/lin-loc/TLCA05/tlca05.pdf), TLCA 2005, section 2 | Freely duplicable location pointers are separate from linear capabilities describing current cell contents. Location polymorphism and existential packaging support abstraction. | Importing explicit capabilities literally would recreate token plumbing. Its strong-update and `thaw` extensions need separate evaluation. |
| [Checking and Inferring Local Non-Aliasing](https://theory.stanford.edu/~aiken/publications/papers/pldi03a.pdf), PLDI 2003, sections 1–4 | Checks local exclusivity among accesses even when aliases exist elsewhere. Gives constraint-based checking and inference algorithms for its `restrict`/`confine` system. | Does not establish complete memory safety for arbitrary C. Ordinary C `restrict` alone is unchecked and is not an acceptable WF mechanism. |
| [DPJ](https://dpj.cs.illinois.edu/DPJ/Publications_files/DPJ-OOPSLA-2009.pdf), OOPSLA 2009, overview and region/effect system | Hierarchical region names, disjointness, and method effects support modular deterministic parallelism. Regions here describe heap partitions, not Rust borrow lifetimes. | A Java extension; its parallel theorem is not a theorem about WF manual reclamation or exact rows. |
| [Reachability Types](https://www.cs.purdue.edu/homes/rompf/papers/bao-oopsla21.pdf), OOPSLA 2021, sections 4–5 | Combines tracked aliases with effect summaries and call-site substitution, including read/write and destructive-effect variants. | Its complete framework and proof scope must not be replaced by the slogan that disjoint variable names imply disjoint storage. |
| [Polymorphic Reachability Types](https://arxiv.org/html/2307.13844v1), POPL 2024, sections 2.1.4–2.2.6 | Precisely the association-polymorphism problem: preserve one-step links, substitute at calls, and saturate where overlap matters. Distinguishes fresh resources from untracked values. | Naive polymorphic extensions are shown unsound. Copying only set syntax omits essential freshness, scope, and variance conditions. |
| [Complete the Cycle](https://arxiv.org/abs/2503.07328), OOPSLA 2025 | Extends reachability tracking to cyclic references and separates a reference cell from its indirect referents. | The inspected abstract supports relevance, not a verified mapping of its full rules to WF mutable graphs. |
| [Free to Move](https://arxiv.org/html/2510.08939v1), technical report, 2025-10-10, sections 2.2, 3.4–4 | Static use/kill sequencing combines alias tracking, deallocation, and transfer. Reported mechanization is strong evidence that access-oriented lifetime reasoning is possible. | Free is idempotent in its model; move/swap allocate fresh locations. True in-place swapping and richer indirect invalidation remain open there. The semantic logical-relations variant excludes subtyping and cycles. It is not a ready WF allocator model. |
| [When Lifetimes Liberate](https://arxiv.org/pdf/2509.04253v3), revision 2026-03-28, sections 1–3 | Uniform reference types combine sharing, scoped reclamation, and arena allocation; arena topology permits flexible internal graphs. | Coarse arena reachability intentionally merges objects. Used alone, it loses the distinction between independent payloads sharing one allocator. |
| [Spegion](https://drops.dagstuhl.de/storage/00lipics/lipics-vol333-ecoop2025/LIPIcs.ECOOP.2025.15/LIPIcs.ECOOP.2025.15.pdf), ECOOP 2025, abstract and conclusion | Effect-based non-lexical, splittable regions and sized allocations are directly relevant to capacity and subdivision. | This inspection does not establish a general mutable-object association model or an acceptable WF checking-cost bound. The paper describes its implementation as under development. |
| [Escape with Your Self](https://sweetsinpackets.github.io/file/escape-paper.pdf), PLDI 2026, introduction and concluding algorithm discussion | A sound decidable bidirectional algorithm, verified in Lean, addresses escaping names and self-references. This closes an important gap between declarative reachability rules and a checker. | Polymorphic type/qualifier instantiations still remain explicit. Decidability alone does not establish WF's non-exponential cost requirement or full automatic caller inference. |
| [Typestate via Revocable Capabilities](https://arxiv.org/pdf/2510.08889v2), revision 2026-06-10, sections 2 and 4 | Object-associated capabilities, revocation, and implicit capability supply reduce explicit state-token threading. It is a useful interface-design comparison. | Its full soundness proof is out of scope. The implementation approximates use by reachability and limits destructive effects on mutable variables/fields. It does not establish erased, fully general WF permissions. |
| [Graph IRs for Impure Higher-Order Languages](https://arxiv.org/abs/2309.08118), OOPSLA 2023 companion report | Describes deriving effect dependencies from reachability and preserving them through optimization. | Useful for the lowering proof obligation; not permission to add new WF runtime dependencies or import its complete scheduling model. |
| [GhostCell](https://plv.mpi-sws.org/rustbelt/ghostcell/paper.pdf), ICFP 2021 | A Rust library API separates data aliases from a statically checked permission token and has a mechanized soundness argument. | It does not change Rust's compiler. A shared brand and explicit token do not by themselves deliver WF's desired per-payload effect granularity. |
| [Scala separation checking](https://docs.scala-lang.org/scala3/reference/experimental/capture-checking/separation-checking.html), documentation retrieved 2026-09-15 | Captured relationships are part of interfaces; signatures can explicitly admit aliasing between arguments. | Experimental and explicitly less mature than capture checking. Capture separation and precise field/range effects are different levels of precision. |

The [reachability research repository](https://github.com/TiarkRompf/reachability)
links the related calculi, proofs, and prototypes. It was inspected for the
artifact map, not built. The published PLDI 2026 paper was preferred over the
older arXiv abstract for its algorithm and proof-assistant claims.

## Candidate mechanisms

Four alternatives merit comparison. None is selected as a language decision.

| Candidate | What it changes | Strength | Main risk or cost |
|---|---|---|---|
| Short checked access scopes | Keep existing ownership, but let long-lived handles request narrowly scoped access. | Small conceptual step; established local non-aliasing techniques. | Can reintroduce explicit token/scope plumbing and may retain coarse provider exclusivity. |
| Association parameters + effects + resource contracts | Aliasable references retain target relationships; calls request access and transform checked resource state. | Fits WF's explicit contracts, existing effects, and sequential semantics; targets both Arena and IO. | Requires stored provenance, alias-aware fact updates, freshness, and a sound physical-storage model. |
| General reachability types + destructive effects | Track arbitrary retained dependencies and use/kill composition in types. | Most ambitious route to replacing lifetime-centered borrowing broadly. | Freshness, escaping names, variance, mutable links, and practical checking require a substantial calculus; existing memory models do not settle reuse. |
| Explicit permission tokens | Keep multiple locators and one separately threaded authority. | Clear resource accounting with strong precedent in L3 and GhostCell. | Does not meet the desired calling interface unless authority flow can be inferred and erased. |

The second candidate is the leading **prototype hypothesis** because it can
test the exact allocator and IO requirements without first importing the
entire higher-order subtyping problem. This is not evidence that it is uniquely
minimal, sound as sketched, or superior in runtime performance. A failure on
stored references, reuse, or bounded checking would reopen that ranking.

## A concrete association model to test

All following notation is explanatory, not proposed final WF grammar.

```text
Ref<'life, P, T>      // locator for T in footprint P, valid under 'life
Block<'life, A, D, T> // allocator association A, owned payload footprint D
```

`'life` is an extent of validity. `A` and `D` are static parameters of an
association/footprint kind. They are not lifetimes, runtime pointer arguments,
allocator classes, or permission tokens. The runtime Block can contain the
usual payload pointer, allocator pointer, and size. Static indices do not
require additional runtime fields. Erasure would need a proved lowering.

It is legitimate for a compile-time parameter to describe relationships
between runtime values: the compiler proves a universally quantified relation
between those values; it need not know their addresses. But a single source
allocation site cannot be treated as one runtime object. Loops, recursion,
and repeated calls require generative names and abstract collections.

Keep four relations distinct:

1. Same target: two paths refer to the same state.
2. Spatial containment/disjointness: a range is part of a larger footprint,
   or two ranges do not overlap. Unequal target names alone prove neither.
3. Stored association: a Block contains a pointer to its allocator. This
   does not make its payload alias allocator metadata.
4. Validity dependency: reclaiming an enclosing allocation can invalidate
   a child. This does not mean every metadata write touches that child.

For an arena, let `whole(A)` include `meta(A)` and its managed data space.
Let `D1` and `D2` be live disjoint payload ranges. `meta(A)` is disjoint from
both under the allocator representation invariant. `writes(whole(A))`
conflicts with either payload. `writes(meta(A))` alone does not.

An effect must follow the selected association, not automatically include the
entire transitive object graph. Otherwise every Block's backpointer merges
all blocks into one footprint, destroying the intended granularity. A path
used for effect projection must also refer to the captured target, not be
silently re-evaluated after a mutable field changes.

Opaque objects need the same explicit abstract associations as ordinary
reference fields. An allocating contract states that the returned Block's
allocator association equals the input allocator, and that its payload is a
fresh live subrange disjoint from existing live payloads. The implementation
must establish these facts. Neither a type parameter nor a native declaration
can assert them without the language's ordinary checking/trust obligations.

### How a call instantiates relationships

```text
fn overwrite_pair<P, Q>(x: Ref<P, u8>, y: Ref<Q, u8>)
    writes(P, Q)
{
    store(x, 1)
    store(y, 2)
}
```

This body is safe for both equal and distinct targets if they are live,
initialized as required, and writable. It never assumes `P` and `Q` are
different. At `overwrite_pair(r, r)`, substitution yields `P = Q = R` and
the second store wins. At a call with proved disjoint targets, the two stores
may also admit a different optimization. The same sequential machine code
can serve both calls.

There are three different uses of instantiation:

- **Contract substitution:** replace formal association symbols with actual
  symbolic footprints and verify stated obligations. This is necessary.
- **Optimization specialization:** use proved actual relationships to derive
  additional independence or valid alias metadata. This is optional.
- **Acceptance by enumerating body variants:** recheck an implementation for
  every possible alias configuration. This is not necessary and is a poor
  default: many parameters and recursive calls create a combinatorial problem.

An alias-sensitive body instead needs an interface condition:

```text
fn first_after_clear<P, Q>(x: Ref<P, Vector>, y: Ref<Q, Vector>)
    requires disjoint(P, Q)
    reads(P, Q), writes(Q)
{
    if len(x) > 0 {
        clear(y)
        return element(x, 0)
    }
    return none
}
```

Here `clear` is assumed to read and write Q. Without the disjointness condition
or a fresh length check, the body cannot
retain the current nonempty fact for x after clearing a possibly aliased y.
The condition is proved at the caller. Caller knowledge cannot retrospectively
justify an unstated assumption in a supposedly universal definition.

A contract can be parametric in relationships without cloning runtime code.
Whether WF should infer simple association arguments, require them explicitly,
or provide constrained overloads is an open interface choice. Existing FN-2
does not make one of those future choices automatic.

### Unknown targets, freshness, and packaging

For `r = if condition { a } else { b }`, an interface may carry the finite
possible-target set `{A, B}`. Unknown does not mean fresh, unrelated, or
disjoint. Safe sequential stores can remain legal; unproved independence
prevents parallel permission. A caller-authored branch may refine the relation,
but the checker does not insert an alias test to rescue acceptance.

Fresh allocation should introduce an existential resource name together with
its live-storage and separation facts. Repeated calls do not reuse one static
identity. A container of many blocks needs an abstract family of payloads and
its separation/ownership invariant, rather than a compiler enumeration of all
runtime allocations. This is a central unimplemented part of the hypothesis.

A nominal brand alone is insufficient: it can express a common association,
but different brands require a checked freshness/separation guarantee before
they imply different storage. Location indices give more exact identities;
footprint indices also need ranges and containment; reachability qualifiers
describe possible retained targets. These are alternative representations
with different precision. The new information need not force explicit generic
syntax everywhere: dependent parameter paths can carry some of it implicitly.

Mutable association fields need either an invariant possible-target envelope
or a checked state transition that updates all dependent facts. A value already
loaded through a field retains its original target. Treating a mutable field
path as a permanently stable identity is unsound.

Moving an owner or descriptor preserves a backing target when the storage
itself does not move. Physically relocating an inline referent is a different
transition: it must establish that old locators cannot subsequently access the
old storage, or preserve them by an independently justified representation.
Renaming a binding is neither fresh allocation nor proof of relocation safety.

## Access effects and resource state

The complete effect row can retain ordinary reads/writes for scheduling.
Reclamation is a write to affected storage as well as provider state. What
read/write sets alone do not say is whether the target remains live afterward.
That information can be an ordinary resource-state pre/postcondition; it need
not become a separate scheduling effect category.

The candidate's ordinary write rule requires a mutable target, suitable live
and initialized storage, a well-typed new value, and the operation's domain
proof. It does **not** require that only one locator exists. This is the
actual change to write authority. Parallel execution separately requires
noninterference; ownership still controls consuming or moving out resources.
An explicitly read-only access path would restrict operations through that
path, not silently promise that other aliases never change the target.

```text
fill(block<A, D>)
    requires Live(D), Initialized(D)
    writes(D)
    preserves Live(D), Initialized(D)

release(own block<A, D>)
    requires Live(A), Live(D), ReleaseRight(D)
    reads(meta(A)), writes(meta(A), D)
    consumes ReleaseRight(D), Live(D)
    preserves Live(A), all other live payloads
```

`Live` and `ReleaseRight` above denote obligations of the candidate resource
logic, not trusted user axioms or new runtime objects. Liveness knowledge is
not the same as disposal authority: knowing that a target exists does not
permit consuming another owner's resource. The semantics must define which
assertions duplicate, which resources consume, and how frame preservation is
checked. Merely declaring `writes` cannot forge a release right.

Each Block owns its release obligation. Its allocator pointer is aliasable;
it is not one of several unique allocator pointers. The call's access demand
is satisfied against the current checked resource context. Sequential calls
can access the same metadata in turn. If calls execute in parallel, the
checker must establish compatible demands and resource transfers.

For complete runtime read/write footprints `R1, W1, R2, W2`, the spatial part
of an independence check is:

```text
disjoint(W1, R2 union W2)
and disjoint(W2, R1 union W1)
```

This is only the spatial part. Dataflow, control flow, consumption, initialized
state, operation domains, and observable behavior must also survive the
overlap, as in the intent of PAR-1. A failed independence proof preserves
source order. It does not authorize a new lock, branch, or dependency to make
an otherwise invalid operation acceptable.

| Operations, under the stated contracts | Candidate result |
|---|---|
| Two reads of D1 | May overlap. |
| Fill D1 and fill D2 | May overlap when D1 and D2 are proved disjoint. |
| Fill D2 and release D1 | May overlap if the full release footprint and state transition preserve D2 and no other dependency interferes. |
| Release D1 and release D2 in one allocator | Remain sequential because both access shared metadata. |
| Fill D1 followed by release D1 | Valid sequentially; not independent. |
| Release D1 followed by reading D1 | Reject: the later access lacks live-storage evidence. |
| Reset whole(A) while a later use needs a child | Reject unless the child's validity is preserved; broad writes already expose the interference. |

Two validity variants should be compared, rather than assuming one is forced:

- **Valid references:** retain a region-style guarantee that stored references
  stay usable throughout their declared validity. Destruction is checked
  against those dependencies, but ordinary content mutation is permitted.
- **Locators with checked use:** permit retained inert addresses after a
  resource transition, but every use needs current live/typed storage evidence.
  Reclamation removes that evidence through every alias. This is more radical
  and closer to alias-type/effect calculi; even what counts as a valid reference
  value changes, so it needs its own semantic and lowering proof.

Both separate persistent address relationships from ordinary write exclusion.
The first can preserve more of WF's current lifetime model. The second might
replace more of it. Neither has been chosen or implemented here.

## Semantic witnesses and failure cases

These are proposed experiment cases. Outcomes are derived requirements,
not results from an implemented checker.

| Witness | Required observation | Unsound or inadequate shortcut it detects |
|---|---|---|
| Allocate b1, b2, b3 from one allocator while all remain usable | All three can coexist; data writes need no allocator token threaded through the caller. | Replacing a lifetime brand but retaining long-held provider exclusivity. |
| Move b1 into a struct, return it through a helper, release it | Release still resolves the original allocator target. | Losing associations at owner moves or function boundaries. |
| Release b2; allocate b4 into reclaimed space; retain b1/b3 | No corruption or loss of surviving payloads; capacity is actually reusable. | Bulk-only reclamation, logical deletion without reuse, or fabricated fresh physical storage. |
| Read a saved alias to b2 after release | Rejection independent of whether its bytes are reused. | Relying only on sequential scheduling. |
| Reuse b2's address for b4, then read the old alias | Still reject; b4's liveness cannot revive b2. | Identifying resource generations solely by address. |
| Release the same block twice through aliases | Reject second disposal; no idempotent-free runtime convention. | Treating duplicable liveness facts as release authority. |
| Two same-target scalar arguments, sequential overwrites | Accept with the source's last-write behavior. | Assuming distinct generic parameter names imply disjointness. |
| Two dynamically indexed array ranges | Preserve sequential legality where domains hold; derive overlap only from actual range facts. | Equating unequal base/index names with non-overlapping extents. |
| Read an element after vector backing replacement | Reject or prove backing stability. | Modeling descriptor writes but losing element-storage dependencies. |
| Clear a possibly aliased vector after proving nonempty | Recheck the indexing domain or demand disjointness. | Keeping stale current-state facts across alias writes. |
| Save a target loaded from a field, then retarget the field | Saved reference still points to the old target. | Re-evaluating the field path as the reference's identity. |
| A callback captures an alias and closes/frees or changes it | Its latent effects/state transitions propagate to the caller and any later access. | Checking only explicit reference parameters. |
| A function temporarily moves out a field, then invokes a callback | Callback cannot observe uninitialized or invariant-broken state through another path. | Treating ordinary sequential aliasing as sufficient for safety. |
| A loop or recursion retains a growing collection of blocks | Abstract family invariant handles freshness and separation. | Enumerating dynamic objects or alias partitions during checking. |
| Two different IO handles refer to the same underlying state | Their related effects interfere despite distinct wrappers. | Mistaking handle identity or pathname inequality for external-state independence. |
| Two blocks share a control pointer but not their payload | Payload effects stay separate. | Using complete reachability closure as every access footprint. |

For the IO witness, the relevant identity is the state actually observed or
changed: wrapper, open-file description, file contents, namespace, or provider
accounting may be different subjects. An interface must expose the subjects
its operations need. If external aliasing cannot be disproved, use a sound
shared footprint or withhold independence. This study does not infer physical
file independence from two different path strings, nor control external agents.

## Candidate checking process

The following is a proposed algorithm outline, not a proved checker.

1. Check declarations under symbolic association parameters. Keep a finite
   environment of target relations, stored links, spatial facts, and resource
   state. Do not assume different formal symbols are disjoint.
2. Give every primitive a typed access footprint and checked resource
   transition. Reads cannot duplicate affine contents; destructive operations
   need the appropriate ownership and initialized-state conditions.
3. Check the body against its declared row and pre/postconditions. Preserve
   current-state facts only where their supports survive possibly aliased
   writes. Mathematical facts about immutable old value images remain true.
4. At a call, substitute actual association information into the entire
   signature, including retained result relationships and resource transitions.
   Prove its requirements, apply its state changes, and open fresh result
   packages. No callee-body reconstruction is needed at this boundary.
5. At control-flow joins, retain only justified state and conservative target
   information. Loops and recursive functions use written invariant/contracts,
   rather than unbounded symbolic execution or interprocedural discovery.
6. Derive parallel permission from the instantiated complete footprints and
   all relevant state/control/data obligations. Preserve sequential acceptance
   when only an optimization proof is missing.
7. Erase association and proof arguments before lowering. Emit alias metadata
   only for a proved spatial and temporal scope. Aliasable mutable references
   cannot receive blanket `noalias` attributes.

Exact rows require care under substitution. A symbolic row mentioning both
P and Q can resolve to one target when actuals alias; deduplicating that target
does not make either declared formal access fictitious. Effect attribution
must preserve explicit backing associations across owner moves. Fresh local
effects may be hidden only with a checked non-observability argument; moving
an incoming object into a local cannot hide writes from its surviving aliases.

Finite graph traversal and normalized footprint comparisons offer plausible
bounded building blocks. They do not prove a polynomial bound for the complete
type system. In particular, eagerly expanding nested generic shapes or
enumerating all alias partitions is unacceptable. A prototype must use shared
representations and explicit bounds/invariants and measure actual checking
growth before a language commitment.

The backend must also model generic aliasable memory correctly. A pointer may
remain physically unchanged while logical ownership moves. Reusing bytes
requires a relation between fresh logical resource identities and actual
storage, without runtime generation checks. This is a proof obligation, not
an invitation to insert a virtual-memory indirection layer.

## What would select or reject the hypothesis

The useful experiment is a small general checker and executable storage model,
not an Arena-name special case. Compare the same witnesses under the current
WF rule, checked short access scopes, and association/effect contracts.

Before implementation, fix these observations:

- **Expressibility:** the three-block reuse program and common-state IO
  witness fit ordinary signatures, including movement through stored fields.
  Excluding those forms would change the question.
- **Safety discrimination:** negative cases above fail for the missing
  resource/alias/domain fact; useful safe alias cases remain accepted.
  A prototype accepting only statically disjoint arguments fails the goal.
- **Physical behavior:** freed capacity is reused, addresses need not change
  on ownership transfer, and no added borrow/generation counter, lock, or
  alias branch appears. Allocator bookkeeping is part of the program.
- **Modularity:** callers check signatures and certificates. Changing a body
  while preserving a checked contract cannot change caller acceptance.
- **Checking cost:** vary parameter count, association depth, result nesting,
  and loop invariant size separately. Record normalized graph size, relation
  queries, proof work, compile time, and memory. A terminating exponential
  checker fails the requirements.
- **Optimization:** on equal workloads, inspect the proved independent
  payload calls and resulting IR, including lost or recovered alias facts.
  Speedups and compile-time costs are not measured in this investigation.

A successful prototype would justify developing a preservation/progress
argument, substitution and frame lemmas, erasure/refinement to reused physical
storage, and a parallel observational-equivalence argument. Passing witness
tests would not substitute for these proofs or for a specification-fixed
acceptance algorithm.

No live design-tree decision or specification rule changes in this research
delivery. A subsequent proposal adopting a mechanism must revisit the affected
rules and nodes above together, especially stored provenance, the permission
meaning of references, provider access, exact effect projection, state support,
and the full-call loan condition. No amendment approval is requested by this
report's prototype ranking.
