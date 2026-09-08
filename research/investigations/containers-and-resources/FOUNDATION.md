# Full values, construction, and checked container composition

The next implementation should make a complete fixed array a general owned value,
let ordinary source types preserve finite invariants over already-valid fields,
and construct fresh results in their eventual storage through the compiler's
general destination machinery. The discriminating program is a reusable pool of
empty fixed-size backing blocks: construct a block, publish its full owner, then
empty and return the same backing, including construction failure at every prefix.
The evidence does not require exposing a partially initialized `T` to source code
for this first workload. It also does not establish that the current circular
`Vector` is a sufficient representation for every future container.

This is a research selection and amendment proposal, not current language syntax
or an implemented compiler capability. The active [specification](../../../spec/kernel-spec.md)
remains authoritative. [REASSESSMENT.md](REASSESSMENT.md) records the preceding
owned-place work; the narrower experiment results live in
[foundation/RESULTS.md](../../experiments/container-representation/foundation/RESULTS.md).
This document owns the selected semantic boundaries until the implementation and
specification supersede them; update the selection in place if its acceptance
experiment falsifies it.

## Ground and evidence

The [constitution](../../../docs/constitution.md) makes performance, mechanically
checked safety, usable efficient source forms, and local reasoning the selection
grounds. A small primitive inventory is not an end. Neither implementation
convenience nor compatibility with the current container table selects semantics.
The research-compiler priority is the next useful end-to-end experiment, with the
required proofs intact; it is not completion of a universal library framework.

The [external study](EXTERNAL-WORKLOADS.md#focused-revalidation-contracts-and-host-mechanisms)
recovers actual contracts from pinned ripgrep, DuckDB, Kubernetes, and Rust source.
These are qualitative, deliberately discriminating samples. There is no measured
industry distribution, upstream execution profile, or claim that a source use-site
is hot. Rust's `MaybeUninit` is a host mechanism; the recovered need is construction
before publication without reading raw storage. DuckDB's NULL bitmap and grouped
aggregation reservation demonstrate different logical states, not a mandate that
all containers have a bitmap or a three-way tag.

| Evidence | Consequence | Limit |
| --- | --- | --- |
| Full-array consumption, allocation/fill/publication, fixed-block pool traces | The full owner needs a full-array type; the free pool element separately needs persistent empty-state and capacity facts | The pool is a controlled selection workload, not a reported prevalence estimate |
| 165 finite construction/failure cases, 42 relocation cases, nine retirement cases | Empty release, failure ownership, allocation-before-move, and backing retirement are separate obligations from range coverage | Concrete bounded protocols do not verify symbolic source contracts or runtime execution |
| Matched full/prefix/ring native control | A full fixed value needs no occupancy words; prefix and ring have their own necessary state | Prefix stayed close to full; wrapped ring was about 14% slower in this host's copy-plus-observation test. Different semantics prevent a universal representation choice |
| Matched nullable native control | A per-element tag can have a material layout cost; arbitrary occupancy cannot be dismissed as a prefix | The measured C tagged layout is not Rust `Option`; synthetic clustered traversal is not a production profile |
| Current large `Result<Record, E>` construction probe | The previous fresh-binding destination work does not itself eliminate an intermediate whole result/payload at a retained producer boundary | LLVM shape is not a portable timing claim or proof that every producer needs a temporary |

Safe Rust already provides `Vec<T>` to `Box<[T;N]>` and boxed-slice to boxed-array
conversions; full-value push followed by conversion does not require a writer's
`assume_init` or `set_len`. The matched baseline must include that safe route,
with its exact length/capacity and allocation behavior, alongside a raw-construction
control. See the [Box conversions](https://doc.rust-lang.org/alloc/boxed/struct.Box.html#impl-TryFrom%3CVec%3CT%3E%3E-for-Box%3C%5BT;+N%5D%3E)
and [Vec guarantees](https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees).

The intended Whitefoot delta is narrower: machine-checked static fullness/capacity
relations across ordinary helpers, with no unproved partial-operation fallback,
and efficient fresh nested construction under the same fixed resource contract.
The latter remains an implementation experiment, not measured superiority over
optimized safe Rust. A generic source abstraction which preserves those relations
would also improve W1/W2 compared with maintaining an unenforced library convention.
An extra public focus protocol needs evidence that these simpler routes fail.

## Keep five questions separate

1. **Logical value:** exactly `N` initialized elements, a variable prefix, a ring,
   nullable cells, or keyed membership.
2. **Placement:** inline in its owner, or in backing obtained from a provider.
   Inline does not mean permanently tied to one stack frame; an inline field of a
   boxed object is physically in that object's allocation.
3. **Construction responsibility:** which subvalues already exist and who must
   return or release them if construction stops.
4. **Access:** which initialized places a shared or exclusive view authorizes and
   when that authority returns.
5. **Abstraction:** which relations a caller can rely on without reopening a helper
   body. A pointer or an initialization interval proves neither map correspondence
   nor pool conservation.

A fixed array must not carry a ring head merely because both use contiguous memory.
A heap must not become a type of array element state. Conversely, a stable identity
does not imply a stable address or a mandatory handle lookup. These distinctions
are the foundation that subsequent library policies compose.

## Selected public value and transition boundary

Generalize the existing full-array concept, provisionally retaining
`array<T, N>`. Its standing facts are `len = cap = N`, `head = room = 0`.
It contains exactly `N` valid `T` values and no dynamic occupancy metadata.
Its permitted element types follow the same stored-content and ownership rules
as other owned containers. Copy-fill construction remains restricted to copy
elements; generalizing the type does not authorize duplication of affine values.
Derived cleanup visits the actual elements. Linear elements still require an
explicit checked consuming path; a destructor is not invented from a proof.

The following rows specify the proposed transition shapes. Names are provisional;
the preconditions, transferred responsibilities, and resource effects are the
selection. None is available through these spellings today.

| Proposed operation | Required state | Result and work |
| --- | --- | --- |
| `array_from_fixed(own FixedVector<T,N>)` | `len=N`, `head=0`, no conflicting live loans | `own array<T,N>`; transfers all element responsibilities, with no element construction or allocation |
| `boxed_array_from_run<T,N>(own Vector<'s,T>)` | `len=cap=N`, `head=0`, compatible typed layout and provider, no conflicting live loans | `own Box<'s,array<T,N>>`; adopts the same backing and release duty, with no new allocation or payload copy |
| `array_into_fixed(own array<T,N>)` | Ordinary consuming ownership | `own FixedVector<T,N>` initially full at head zero; enables written element extraction |
| `boxed_array_into_run(own Box<'s,array<T,N>>)` | Ordinary consuming ownership | Same allocation as a full `Vector<'s,T>`; no payload move or allocation |
| `dispose_empty(own run)` | `len=0`, no outstanding loans, ordinary provider authority | Ends the empty run even when `T` is linear; executes only its actual backing release, never a fictional element destructor |

These are full-state transitions, not general reinterpretation or raw-pointer
casts. The compiler checks exact element layout, extent, store brand, initialization,
ownership, and loan conditions. A full rotated ring cannot be sealed by ignoring
its head: that would change its logical element order.

For inline conversion, value transfer alone does not promise address preservation:
moving a pre-existing inline array can require moving its payload. Fresh construction
should share final payload storage with the builder and keep construction metadata
separate in the storage plan. The store-backed conversion has the stronger
allocation-adoption contract. No implicit heap allocation may rescue inline storage
which cannot outlive its frame.

The existing runs are adequate vehicles for the mutable construction phase:
build with head zero, append checked valid values, then seal. This does not select
their circular state as the permanent public builder abstraction. A new prefix
nominal has no demonstrated advantage merely because its coordinates are `lo/hi`
instead of `head/len`; both can carry comparable metadata. Add a distinct form only
when a source, proof, or generated-code comparison establishes the benefit.

Ordinary runtime array literals should provide the same full-value guarantee as
constant literals. A proposed writer shape is:

```text
let a = make_point(...);
let b = make_point(...);
let points: array<Point, 2> = [move a, move b];
```

This is proposed notation, not a currently compilable fixture. The final grammar
must preserve written evaluation order and move each affine input exactly once.
It must not add a separate array safety model or require a permanently redundant
length proof at every use.

## Allocation and the zero-extent decision

For nonzero extent, a Vector's slot allocation and a Box of a full array can have
the same element stride, alignment, and release class. The descriptor is outside
the slot backing. In the current specification, zero extent breaks the equivalence:
a zero-count Vector charges zero arena bytes, whereas a Box of an empty array
charges a cell stride of at least one, rounded to the provider alignment.

That distinct byte has no current source-observable address-identity benefit:
Whitefoot exposes no pointer equality, a zero array admits no element access, and
loans refer to source places. Its allocation refusal and arena-capacity effects
are observable, however. Silently converting under the existing law is wrong.

Select an explicit zero-size resource amendment: a typed owner with statically
zero payload extent requires no dynamic backing bytes or backing free; zero-size
formation succeeds without querying host availability. This applies consistently
to the qualifying Box and Vector formation routes, rather than special
handling inside the seal operation. Logical ownership and store confinement remain
checked even if a target uses a shared non-dereferenced sentinel. Empty reads,
projections, moves, and cleanup must lower without accessing that sentinel.
Use the following bounded structural judgment after type substitution:
`storage_empty(array<T,0>)` holds; for positive `N`,
`storage_empty(array<T,N>)` holds exactly when `storage_empty(T)` holds; a struct
is storage-empty exactly when every field is. The judgment is false for scalars
(including the currently byte-represented `unit`), enums, Box/run descriptors,
providers, and opaque types. It follows finite structural recursion on well-formed
stored types. This is a zero-stored-extent rule, not erasure of every type with only
one semantic value.

Box formation has zero backing charge when its content is storage-empty. Vector
formation has zero backing charge when its count is zero **or** its element type
is storage-empty. Apply the same condition to host queries, arena cursor advance,
and backing free. This also covers a positive-length array of empty array elements,
not only an outer `N=0`. Logical length, capacity, store confinement, and each of
the `N` element responsibilities remain intact: zero physical bytes never prove
zero logical elements or discharge linear ownership. Keep ordinary type/alignment
qualification, and leave predicate-false layouts unchanged. Update refusal and
arena-cursor contracts together. The current conservative stride ceilings must not
be mistaken for a need to allocate a byte for a storage-empty value.

This is a proposed normative resource change. If an observable identity or resource
contract requires preserving the current law, the coherent alternative is an
array-specific Box reservation builder allocated under that law, including the
honest zero-case charge. A second store-branded full-array type duplicates Box for
the nonzero cases and is not selected. Nor does this issue justify generic public
`Reserve<T>` authority for arbitrary partially initialized structures.

## Construction without a public hole in `T`

Ordinary `&uniq T` always points to a valid `T`; `MutSlice<T>` covers initialized
elements, not spare capacity. Neither becomes an initialization permission.
The first full-block workload can keep every incomplete aggregate inside compiler
construction state while source code moves only complete values and valid builders.

The selected implementation experiment is a general result-destination tree:
separate destinations for a result tag, active payload, and constructed fields,
consumed by the normal call/storage path. For an immediately routed
`Result<T,E>`, the success payload can be built in the vacant final slot while the
error payload uses separate storage. The builder boundary advances only when a
complete successful `T` is delivered. A failure cannot publish that slot, and all
completed field owners retain their checked cleanup/return responsibilities.
This is not a `Result` spelling exception: route selection follows checked nominal
variants and typed projections.
Selecting a vacant slot also requires the producer's admitted effects to preserve
that destination's address and vacant state until commit. Capturing an address
before the call supplies no alias authority and does not freeze a mutable run
descriptor. Use the same checked occurrence, loan, and write-footprint information
as ordinary calls; do not infer eligibility from immediate source adjacency.

| Producer/use | Honest destination result |
| --- | --- |
| Fresh total result, built from fields/subresults | Construct into final field destinations |
| Fresh fallible result, immediately matched and placed | Separate success/error destinations; commit the initialized boundary only on success |
| A path returning an existing owned inline `T` | Transfer that existing value once; do not claim it was constructed at the destination |
| A whole-result observation or materialized storage use before routing | Materialize as required by that use; do not discard the observation |
| Conflicting input loan, escaping address, or opaque producer without a placement contract | Use ordinary valid storage or report the unavailable performance capability; placement grants no alias permission |

Allocation remains where written. Reserving an entire array before invoking its
element producers avoids starting construction when that reservation is refused.
It does not authorize moving a later `heap_box(value)` allocation before an
observable producer. Likewise, “failure leaves the builder boundary unchanged”
does not erase producer effects or restore already consumed external resources.

No public `VacantFocus` or new `init` parameter mode is selected for this slice.
Those forms would introduce source-visible partial-object responsibility, failure
protocols, and additional writer obligations for a workload that does not yet need
to express them. Reopen them if a real program must suspend, pass, inspect, or
return an incomplete object through source-level abstraction and complete-value
construction cannot preserve its contract.

Destination choice is ordinarily compiler quality, not a new source safety law.
If a future hard source resource/location contract requires no full intermediate,
it needs an explicit checked qualification with a precise eligibility rule.
Forwarded existing owners and whole-result observers are counterexamples to an
unconditional zero-move promise. Failure to meet a performance qualification must
not misreport an otherwise valid program as unsafe, or silently change its resource
contract by materializing a forbidden temporary.

## Empty backing across library and pool boundaries

A pool of complete arrays and a pool of empty reusable backing satisfy different
contracts. The frozen workload needs the second: if production fails after `k`
elements, those elements are consumed or returned and the **same empty backing**
returns to the free pool. Replacing it with a pool of initialized default values,
releasing and allocating again, or checking its capacity at runtime changes that
contract. Full-array types solve only the successful publication side.

`FixedVector<Vector<'s,T>,M>` preserves the outer pool's capacity `M`, but the
current MSR-3 measure transport does not preserve arbitrary nested element facts
`cap=N`, `len=0`, and `head=0`. The existing
[pool boundary probes](../../experiments/container-representation/lifecycle/RESULTS.md)
expose this distinction. Finite result projection alone cannot recover facts that
were never part of the stored element's type.

Select **finite nominal value invariants** as the next source abstraction. The
following declaration is proposed notation, not current Whitefoot syntax:

```text
struct RawBlock<'s, T, N> {
    run: Vector<'s,T>
    invariant capacity: cap_of(run) == N;
    invariant empty: len_of(run) == 0;
    invariant origin: head_of(run) == 0;
}
```

Here `N` is a `u64` const generic. The fields are ordinary valid values. A RawBlock
contains a valid empty Vector; it contains no invalid `T` and grants no ability to
read, borrow, or initialize raw payload. A free pool can be ordinary
`FixedVector<RawBlock<'s,T,N>,M>`. Every stored element is a valid RawBlock, so
extracting any element reintroduces its declared facts without quantified proof
about all pool indices. This induction is ordinary type validity, not a trusted
library name or a special pool operation.

The invariant mechanism is deliberately bounded:

1. For this slice, declarations attach to source structs; enum payloads transport
   these valid structs without a new enum-invariant grammar. A declaration
   contains a finite conjunction in the existing deterministic
   arithmetic proof fragment, over its scalar fields, static field chains,
   existing measures of those fields, and const parameters. No dynamic indices,
   dereference traversal, arbitrary function calls, quantifiers, ownership
   predicates, or initializedness predicates are introduced. Well-formed fields
   and types are checked before the predicates are admitted.
2. Whole construction proves the instantiated predicates from entering facts
   about the evaluated inputs, before consuming those inputs. It cannot assume
   the new object's invariant to prove itself. Copying or transferring a valid
   nominal preserves validity; a selected complete field, result, or pool element
   introduces its type facts at its new place. Ordinary fact support/invalidation
   still applies to these live-place facts.
3. Existing whole-consuming destructure, `let RawBlock(run: r) = move block;`
   [GRAM-4, PROV-6], kills the wrapper and exposes its valid Vector field. Reroot
   the invariant facts onto the new field bindings at that transfer. This does
   not add partial field moves or a period during which a live RawBlock is invalid.
   Mutate the ordinary Vector, then prove the invariant again when reconstructing
   the wrapper.
4. Whole replacement by an already valid nominal is allowed under ordinary
   ownership rules. For the first slice, a projected mutation or unique loan
   whose possible writes overlap a containing nominal's invariant support is
   rejected: consume and reconstruct that nominal. Unknown write footprints
   overlap conservatively. A statically disjoint write may proceed. Check every
   affected ancestor; do not lose its obligation by passing a plain field type
   to a helper. This rule adds no exception to BLK-4 or existing loan admission.
5. Shared access sees only valid objects. Whole consumption and derived release
   end the nominal's validity obligation; they preserve each field's ordinary
   ownership and release duties. No opening under a borrow, temporary
   broken-invariant state, new loan endpoint, or destructor assumption is added.
   Declared numeric truth cannot create ownership, backing identity, provider
   authority, or permission to access an uninitialized slot. The proof erases;
   the ordinary Vector fields do not thereby disappear from its representation.

For example, a helper cannot borrow `block.run` uniquely, append an element, and
return while leaving the enclosing RawBlock live. It may consume the RawBlock,
append into `r`, and return that valid Vector as the partial construction result.
The required checks follow syntax, types, and write support, not API names.

The pool lifecycle is then:

```text
allocate once -> Vector(cap=N,len=0,head=0) -> checked RawBlock -> free pool
checkout -> RawBlock -> consuming destructure -> Vector(len=0)
fill -> Vector(len=k) -> success at k=N -> Box<array<T,N>>
                    -> failure at k   -> explicit take_back/consume -> RawBlock
full-owner return -> open to Vector(len=N) -> take_back/consume -> RawBlock
RawBlock -> free pool -> repeat; final empty drain -> dispose_empty
```

Rollback uses `take_back`, preserving `head=0`, and discharges elements in reverse
construction order. This is written program behavior, not an assumed order of the
compiler's derived release walk. A helper which cannot consume a linear element
returns the valid partial Vector and remaining inputs; its caller supplies the
actual consuming operations. A generic destructor or callback is not invented.
Normal full-owner return uses the same written emptying path. Allocation identity
comes from the checked ownership transfers and full/run backing adoption, not
from equality of `cap` or `len`. Block backing is allocated before reuse begins;
any resources acquired by element producers have their own written contract and
cleanup. Type invariants alone are not a proof of a global resource bound.

### Specialized slot versus a checked source wrapper

The smallest specialized rival is **one** builtin `ArraySlot<'s,T,N>` representing
empty backing; opening it can recreate an ordinary Vector. A second
`ArrayBuilder<'s,T,N>` with only pointer and prefix length is an optional metadata
optimization, not necessary to make the free pool sound. A builder alone still
has dynamic length and does not preserve empty-state facts through arbitrary pool
storage. `Box<FixedVector<T,N>>` similarly does not encode emptiness, and its
in-allocation length/head words prevent direct adoption as a plain boxed array.

| Choice | Free-slot descriptor on a 64-bit target | Mutable builder | Semantic cost |
| --- | ---: | --- | --- |
| Selected checked RawBlock over current Vector | 32 bytes | Existing four-word Vector | General finite nominal formation, fact transport, and mutation restrictions; no new runtime type |
| Builtin ArraySlot plus Vector | 8 bytes, proposed | Existing four-word Vector | One kernel nominal with reservation/open/close/release and provider/zero-size rules |
| ArraySlot plus ArrayBuilder | 8 bytes, proposed | Proposed pointer plus length: 16 bytes | Two kernel nominals and their transitions, ABI, cleanup, and proofs |

The 32-byte figure follows the current
[Vector descriptor](../../../compiler/src/lowering/builder/runs.rs); the specialized
figures are prospective layouts, not measured Whitefoot implementations. At 512
free blocks the wrapper costs 12,288 extra descriptor bytes relative to a one-word
slot. Excluding allocator and common pool metadata, 16-byte backing per block gives
24,576 versus 12,288 total bytes; 512-byte backing gives 278,528 versus 266,240;
65,536-byte backing gives 33,570,816 versus 33,558,528. Thus the difference is large
for tiny blocks and small for the wide-record construction case. No throughput or
workload-frequency conclusion follows from this arithmetic.

Choose the source invariant for the first wide-record pool because it preserves
the exact behavior/backing contract and addresses a reusable abstraction failure:
ordinary types cannot currently retain checked relations over their valid fields.
The same capability can describe a bounded cursor or related counters without
adding a nominal kernel family for each. This is not a claim that all future
containers should pay Vector metadata. Reopen the specialized slot if a matched
small-block program needs its descriptor footprint or if consume/reconstruct
source imposes material W1 cost. Do not claim that proof erasure automatically
specializes away constant fields; any such representation optimization needs its
own general correctness and cost evidence.

## Allocation and helper result contracts

The allocation-first heap form has this proposed shape:

```text
match heap_vector::<Point>(store: &uniq heap, count: 16) {
  None => allocation_refused(),
  Some(builder) => {
    // Each helper returns a valid builder or an explicit failure responsibility.
    // len(builder)=i, head(builder)=0 is the loop invariant.
    repeat i in 0..16: builder = append_generated(move builder, i);
    return boxed_array_from_run::<Point,16>(move builder);
  }
}
```

This is operation-level pseudocode: it assumes successful element production to
show the core transition. A fallible producer's error must carry the current valid
builder and any still-owned inputs, or the caller must explicitly drain/release
them before returning. Affine cleanup and linear discharge are different cases.
No partially constructed value may disappear in an `Err` branch.

The accompanying contract extension is finite result projection: static field chains and
named variant payload routes through returned structs and enums. It must preserve
the existing entering snapshots, arithmetic proof rules, ownership checks, and
write-footprint invalidation. This enables a failure wrapper to expose the length
of its returned builder and a pool helper to expose free-count conservation.
It does not add arbitrary element quantification or grant a bitmap initialization
authority. Same capacity/length is also not proof of identical contents or object
identity; promises about those require their own justified relation.

Full arrays do not solve dynamic column validity, index-map correspondence, or
graph reachability. A caller-written allocate-before-move block already preserves
the old owner on reservation failure without inventing a new general equality
predicate merely to describe the local code. A helper's stronger advertised
relation must be checked, not inferred from its name or from matching counts.

## Why the broader alternatives are not selected first

| Alternative | What it could solve | Selection and reopening condition |
| --- | --- | --- |
| Add full/prefix/ring/sparse families as needed, all compiler-owned | Small closed initialization judgments | Full state earns its place now. No evidence makes every future layout a new builtin; one hard contract can reopen the boundary |
| New prefix/span owner plus existing operations | Different source coordinates, possibly simpler proofs | Unselected until it beats the existing head-zero construction vehicle on a matched use; a new name is not a foundation improvement |
| Builtin empty ArraySlot, optionally with a prefix builder | Persistent pool state and smaller descriptors | Sound rival; the checked ordinary wrapper is selected for the first wide-block workload. Reopen on a matched metadata or writer-cost requirement |
| Public vacant focus / initialization mode | Passing incomplete construction responsibility through a helper | Unselected for complete-value producers; reopen on an actual source-level incomplete-object lifetime |
| General checked library representation predicates | Custom sparse/packed layout with a verified abstraction boundary | Remains the D17 direction. It needs deterministic body proof, ownership/initialization framing, abstract contract preservation, and erasure evidence; current finite models do not supply them |
| Ordinary `Option<T>` everywhere | Safe, explicit runtime occupancy using existing value semantics | Valid baseline, not a universal performance answer. Nullable bitmap evidence requires a matched checked alternative where its cost matters |
| Mandatory handles/stable slots for all containers | Some identity/recycling workloads | Unselected for dense values; introduce only where retained identity/address lifetime requires it |

Finite nominal value invariants are not general representation authority: they
relate already-valid fields and cannot make an absent `T` a present one. The next
pressure case for broader library representation authority is a nullable or
reserved-slot implementation with the exact external contract frozen first.
Ordinary runtime metadata cannot manufacture an initialized owner. A bitmap's
relationship to payload initializedness must be maintained by checked operations
or by a verified representation predicate. Finite arithmetic projections alone do
not prove arbitrary sparse membership or many-to-one target independence.
One demonstrated hard-contract failure is enough to revisit the selected small
core; no fixed number of workloads or document status is an additional gate.

## First implementation slice and falsifiers

The first vertical slice is a reusable pool of empty fixed-size backing, with
inline full-array controls, heap-backed full owners, non-copy elements, a fallible
nested element producer, ordinary helper boundaries, and complete release. It
includes the generalized full-array value, checked full/run transitions and
empty-linear discharge, finite nominal value invariants, the finite result
contracts actually needed by its helpers, and generic fresh result placement. These parts
are one end-to-end capability, not independent API additions declared complete
before they compose.

Executable acceptance must cover:

- Zero and nonzero extents; aligned/nested affine elements; source literals and
  copy-fill with their correct ownership domains; inline and provider-backed forms.
- Pool exhaustion, out-of-order block return, and reuse with the same backing
  budget; empty-capacity RawBlock facts and full-array facts across field, helper,
  and pool boundaries. Test a second ordinary invariant-bearing type such as a
  bounded cursor, so no RawBlock-specific checker behavior can satisfy the slice.
- Failure before any element, after a prefix, and inside nested construction;
  exact affine cleanup or explicit linear return/discharge, with no lost inputs.
- Fresh total and fallible producers at retained call boundaries; forwarded-owner
  and whole-observation controls that honestly retain necessary transfers.
- Missing initialization, duplicate ownership, wrong extent/order/provider,
  unsupported projected claims, stale facts after mutation, and premature reuse as
  nearby negative cases, with source-rule diagnostics rather than internal errors.
  Invariant negatives include self-justifying construction, an invalid wrapper
  returned through another reachable branch, overlapping projected unique mutation,
  and an inner write that violates an enclosing nominal's relation.
- Ordinary and staged calls through the same typed ABI rules, with two independent
  result destinations and storage retained through join return, result consumption,
  and retirement. Worker identity and DONE are not lifetime premises.
- Generated storage, copies, initialization work, allocation count, and peak backing
  against matched native controls. Run the normal facts-off path as well; an
  optimizer fact cannot authorize source acceptance or change behavior.

The source-checker extension must use fixed deterministic derivations or checked
finite written steps, with no SMT acceptance, timeout, cumulative fuel, or runtime
proof fallback. The backend may consume established facts but cannot grant source
authority. A temporary implementation limitation remains a capability limitation.

If the result-tree experiment cannot preserve alias, effect, failure, or retirement
boundaries without a special call route, stop expanding that lowering design and
reconsider explicit checked destination qualification. If the full-block program
requires pervasive layout exposure or repeated reconstructed proofs, reconsider the
contract abstraction before adding more containers. Completing this slice does not
claim a verified hashmap, general sparse layout, or a resource-envelope checker.
