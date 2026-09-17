# x0 dependency and gap map

This is the compact navigation map for every non-D x0 matrix cell at the
frozen matrix revision `1c9a66aa`: 67 U and 12 C. It classifies the operation actually
shown in each witness, rather than assigning a cell from either axis name.
Each cell has one primary group; secondary dependencies are named where they
change what a future derivation must establish. Supersede this file in place as
cells are revisited. It is not a rule amendment or a correction to the frozen
verdicts.

## Missing rules (U)

### E — hidden target packages (11)

**Code discriminator:** a value is loaded from a collection, recursive edge,
or other representation that hides which target parameter it contains, and a
later read, write, call, effect comparison, loop step, or recursive call must
open that package and eventually preserve or repack its relation. A field whose
type still exposes `Loc<P,T>` is not in this group.
```text
box: Exists<P>.Holder<Loc<P,Int>>; p = unpack(box); read(p.at); repack(box)
```
**Missing rule:** existential target introduction/elimination, scoped witness
names, relation and duty transfer while open, and checked repacking after
mutation.
**Cells:** 1-8, 8-15, 8-17, 8-19, 8-20, 8-21, 8-22, 8-23, 16-19, 17-19,
19-19.
**Secondary links:** predicate opening is also needed by 8-15, 16-19, 17-19,
and 19-19; loop-family projection is also needed by 8-17 and 8-23; callback or
recursive contracts are also needed by 8-19, 16-19, 17-19, and 19-19.

### P — representation predicates (11)

**Code discriminator:** the body invokes or relies on `close_invariant`,
`Repr`, `VecRep`, `SeqRep`, or an equivalent abstract relation to turn private
representation into concrete live/full/layout/bounds/duty facts and back. A
mere helper or `use` name supplies no derivation.
```text
use open VecRep(v); take(loc(v.data[i])); use close VecRep(v)
```
**Missing rule:** predicate definitions with ownership, finite checked
fold/unfold steps, framing, mutation obligations, and exact resource return on
close.
**Cells:** 1-15, 6-15, 6-20, 7-15, 9-15, 10-15, 12-15, 13-15, 15-15,
15-18, 15-20.
**Secondary links:** dynamic element families affect 1-15 and 15-18; active
layout affects 12-15; 6-20 and 15-20 also require the explicit proof-step
grammar.

### F — quantified resource and selection families (14)

**Code discriminator:** runtime-many occupied elements, owners, allocation
duties, initialized prefixes, shards, or loop iterations must be related by one
symbolic invariant; direct facts for a fixed finite set do not suffice.
```text
for i in 0..n invariant { owns_exactly(items, 0..i) } { items[i] = alloc(i) }
```
**Missing rule:** specification-fixed family constructors, projection/split/
merge/update steps, freshness and exact-duty accounting, bounds-indexed range
families, and terminating certificate checking across loop backedges.
**Cells:** 1-17, 2-15, 2-17, 3-15, 4-15, 4-23, 5-15, 10-17, 14-17,
14-20, 15-17, 15-23, 17-20, 20-20.
**Secondary links:** most container members also need P; 4-23 and 15-23 must
then project cross-iteration separation for R15; 14-17 and 14-20 additionally
need provider-child formation.

### V — provider formation, dependence, and physical extent (16)

**Code discriminator:** `provider_alloc`, provider-backed release/end, or pool
metadata appears, but no checked primitive/body establishes the fresh payload,
its byte extent, its unique disposal duty, its dependence on the provider, and
the provider metadata accesses. Payload separation alone does not separate the
management target.
```text
a = provider_alloc(pool, 8); release(a); close_provider(pool)
```
**Missing rule:** provider/child creation and destruction transitions; sibling
byte separation versus ancestor overlap; provider-live dependency accounting;
complete read/write/allocate/end effects; concrete runtime representation and
synchronization costs.
**Cells:** 1-14, 2-14, 3-14, 4-14, 5-14, 6-14, 7-14, 8-14, 10-14, 11-14,
12-14, 13-14, 14-14, 14-15, 14-21, 14-24.
**Secondary links:** 8-14 may need a hidden-target package only if `block`
actually erases its target parameter; 14-14, 14-15, and 14-20-style provider
sets need F after formation exists; 14-24 also constrains lowering and runtime
metadata.

### Q — conditionally partial aggregate cleanup (5)

**Code discriminator:** a shared exit must clean a runtime-selected empty/full
element or an arbitrary aggregate with conditionally present duties, and the
source supplies no branch-local cleanup sequence valid in every represented
state.
```text
v = take(loc(a[k])); cleanup(v); scope_exit(a) // a[k] alone is empty
```
**Missing rule:** either a source-visible discriminant with a verified cleanup
match, or a defined aggregate partial-state representation and cleanup rule;
no hidden occupancy/drop flag may be inferred from ghost state.
**Cells:** 2-11, 11-13, 11-15, 11-17, 11-20.
**Secondary links:** 11-13 and 11-17 need F to name the selected dynamic part;
11-15 needs P; 11-20 needs the explicit proof-step grammar.

### R — callable contract refinement (2)

**Code discriminator:** one callback must satisfy more than R11's direct
identical-contract substitution: different enum payload types or per-element
effects selected from an abstract container family.
```text
match e { A(x) => f(x), B(y) => f(y) } // one refined contract for f
```
**Missing rule:** checked contravariant/covariant contract implication,
effect/resource-row refinement, and target substitution without inspecting the
callee at each call.
**Cells:** 12-19, 15-19.
**Secondary links:** 15-19 first needs P/F to expose each selected element;
12-19 must state a common operation on both payloads before refinement can be
tested.

### L — physical lowering and scoped optimization facts (8)

**Code discriminator:** the abstract transition is available or assumed, but
the witness demands lowering of arbitrary represented owners, ranges,
predicate facts, loop facts, or proof-derived separation into executable
copies/addresses and correctly scoped alias metadata.
```text
use separate(P,Q); write(p, 1); write(q, 2); lower_with_scoped_alias_facts()
```
**Missing rule:** representation-specific move/copy lowering; scope and
invalidation of alias/bounds metadata; ancestor/descendant byte extent; and the
bridge from R14/R15 permission to physical parallel or vector execution.
**Cells:** 3-24, 4-24, 13-24, 15-24, 17-24, 18-24, 20-24, 23-24.
**Secondary links:** 15-24 depends on P; 17-24 and 23-24 depend on admitted R15
families; 3-24 and 4-24 require a concrete owner representation before their
machine copies can be counted.

## Known restrictions (C)

### C1 — source-written cleanup discrimination (4)

**Code discriminator:** branch alternatives reach one exit with different
live/full/empty ownership states. R7 has no single unconditional cleanup, but
moving cleanup into the already written arms is a complete safe rewrite.
```text
if c { release(a) } else { skip }; scope_exit(a)
```
**Cells:** 1-11, 3-11, 5-11, 11-11.
**Cost:** writer-visible branching or duplicated cleanup code; no runtime flag
is introduced. These are restrictions, not missing safety rules.

### C2 — closed counted-parallel body families (7)

**Code discriminator:** each iteration is sequentially derivable, but its body
is a take/put pair, returned-old-value replace, release/cleanup, variant switch,
or arbitrary callback rather than an admitted R15 direct affine form or
adjacent-range helper.
```text
for i in 0..n { v = take(loc(a[i])); put(loc(a[i]), v) }
```
**Cells:** 5-23, 6-23, 9-23, 10-23, 11-23, 12-23, 19-23.
**Cost:** the ordinary loop remains legal but loses overlap, or 19-23 must
inline the set/use a supported view contract. Extending R15 would be a new
bounded rule, not completion of F by itself.

### C3 — non-total or order-sensitive reduction (1)

**Code discriminator:** checked addition can exit at the first overflow, so
arbitrary recombination would change the source-observed failure order.
```text
for i in 0..n { total = add_checked(total, a[i])? }
```
**Cell:** 23-23.
**Cost:** retain source order; wrapping arithmetic is a replacement only when
the program's intended result is modular.

## Verdict/rationale follow-ups

- **8-14 is primarily V, not E as its current reason says.** The displayed
  `block.owner` and `block.at` do not hide a target parameter. Existential
  open/repack becomes necessary only if an omitted type definition erases it.
- **14-21 mixes a derived rejection with a missing facility.** R12 already
  rejects the shown contract because the body reads provider metadata while
  the row lists only writes/allocates. U remains justified only for the separate
  provider-child formation relation. The exact incomplete-contract boundary is
  therefore D-like and should be split from the V witness when revisited.
- **12-19 does not yet demonstrate refinement.** The shown callback has no
  callable contract for either payload and `active_parts(E)` is undefined, so
  R11 can simply reject it. A corrected R witness should give two concrete
  payload contracts and a proposed common refined contract.
- **17-24 and 18-24 overstate what is needed for execution.** Their own
  boundaries say conservative sequential loads/stores are executable. If the
  requested observation is only safe source execution, they should be D; U is
  appropriate only when the witness explicitly requests preservation of an
  optimization or overlap permission. The same distinction should be made
  explicit in 13-24, 20-24, and 23-24.
- No cell demonstrates an x0 safety contradiction. These are classification
  and witness-precision issues, not grounds for silently changing the frozen
  matrix.

## Highest-value bounded refinement

After the selected hidden-package experiment, test one **range-indexed
resource-family certificate fragment**, with
only four operations: create a family from a proved full owned range, project
one bounded member, return that member with its exact duty, and split/join at a
proved adjacent boundary. Give each step fixed syntactic premises and structural
checking; exclude arbitrary user predicates, hidden existential targets,
providers, recursion, and general cleanup from this refinement.

This is the preferred next substantial dependency if the priority is to test
all three evaluation dimensions together rather than to isolate one semantic
boundary. It tests more than cell count.
It can render a growable resource buffer's initialized/owned prefix, justify
sequential same-index take/put or consume in a loop, and carry adjacent shards
to an already admitted R15 view-helper body. The family rule does not itself
extend R15 to transfer, cleanup, or arbitrary callback bodies.
That directly probes safety (exactly one duty per member), sequential
expressiveness (dynamic resource containers), and current parallel performance
(range separation and admitted element maps). It also exposes whether proof
size and checker work scale with symbolic ranges rather than runtime length.
Predicate abstraction, hidden graph edges, and provider dependencies remain
separate later refinements, so a favorable result cannot conceal their costs.

The E1 hidden-package local refinement is selected for the current round. It
has the smaller independent missing-rule footprint and can isolate package
snapshot, invalidation, and call-boundary semantics before predicate, family,
provider, or lowering rules are supplied. That selection does not resolve F or
make the range-family experiment lower value for the next substantial step.

The acceptance discriminator should be one concrete resource-buffer program:
initialize `0..n`, split at runtime `k`, independently process the adjacent
ranges through an existing admitted view helper/body, join them, remove one
element sequentially by duty
transfer, restore or explicitly clean every remaining member, and end the
backing allocation. Reject an out-of-bounds projection, double projection of
one live duty, overlapping split, lost member at join, and parallel bodies with
shared captured writes. Compare its sequential operations and retained R15
permission with the current floor before broadening the certificate language.
