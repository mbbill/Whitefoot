# Discriminating programs for the ownership redesign

Research date: 2026-09-16, evaluation framing updated 2026-09-17. These programs
are stable witnesses used by the earlier debates; verdicts in
[OPTIONS.md](OPTIONS.md)'s successor files cite their IDs. They are pseudocode,
not an adopted syntax or a requirement to preserve each candidate's mechanisms.
Under the [current evaluation method](DESIGN.md#current-evaluation-method-owner-discussion-2026-09-17),
compare the same engineering task under alternative forms, stating changes in
capability, safety evidence and cost. Earlier `Required` paragraphs below retain
their witness-specific expectations; requirements such as literal locator-based
holes or a particular borrow discipline are candidate hypotheses, not additional
goals overriding the three evaluation criteria. Supersede in place; remove with
the investigation.

The full preservation of existing programs and their overlap permissions has
not been evaluated under any candidate.

The [x0 draft](DESIGN.md#candidate-x0-current-state-access-with-separate-resource-accounting)
now gives a concrete 24-axis choice vector and shared pseudocode. The
[interaction matrix](MATRIX-X0.md) records hand derivations against it; these
engineering tasks remain the broader comparison surface, not automatically
established capabilities just because pairwise cells are filled.

## Current capability floor

On 2026-09-17 the owner required a candidate covering the critical capabilities
from the outset, with at least WF's current level retained. Sequential fragments
remain useful witnesses but are not an adequate candidate boundary. IO and FFI
may be deferred in this discussion. Losing an existing capability is a failure
of this floor, not merely a price that may be noted and ignored.

The initial anchors below were checked against kernel specification v0.59 at
[revision 3f205ff6](https://github.com/mbbill/Whitefoot/blob/3f205ff64b0d881ad9cdb69e8b29bcd86ea5d11c/spec/kernel-spec.md).
The research worktree still carries v0.57; its older copy must not silently
define the current floor. These are specification capabilities, not a new
claim about measured compiler coverage, and the list is not exhaustive.

| Capability anchor | Source ground | Required comparison |
|---|---|---|
| Independent declared calls, including recursive callees | PAR-1, EFF-2 | Preserve permitted overlap using caller-resolved access information, including argument evaluation and consumed resources; keep source-order observables. |
| Element-wise loops with same-index read/modify/write | PAR-2's single-binder affine element family | Retain admitted maps and their already-checked bounds; arbitrary injectivity is not presumed. |
| Runtime-width adjacent ranges passed to a helper | PAR-2's adjacent-range family, VIEW-2 | Preserve origin/range information across the signature and separate iterations; no descriptor-as-fresh-backing shortcut. |
| Accumulator recombination | PAR-2's fixed operation table | Retain admitted wrap/bitwise/min/max/Boolean reductions; do not generalize the guarantee to ordinary checked addition or floating point. |
| Owned composite values and storage | TYPE-2, SET-2, PROV-6 | Cover structs/enums, arrays, buffers, heap/store-backed cells and runs, ownership transfer and disposal obligations. |
| Definition-side and call-side contracts | FN-1, CALL-6, ENT-5 | State access, result relations and invalidated facts across calls; retain the facts unrelated to the call's effects. |

Current parallelism permits overlapping otherwise sequential computation; it is
not a writer-visible thread facility (CAP-1). Denying an overlap need not reject
a sequential program. Arbitrary threads are therefore a further question, not
a reason to omit current parallel computation.

P1-P19 also contain requested extensions, not just this floor. In particular,
P7's locator-based holes and P4's stored cursor are exploration witnesses;
they are not assertions that v0.59 already admits those forms. A full
current-program coverage mapping remains outstanding.

One immediate cross-feature discriminator separates boundary state from access
during a call. In the permissive candidate, consider these pseudocode contracts:

```text
fn inspect(p)
    requires live(p), full(p)
    accesses reads(target(p))
    ensures live(p), full(p)

fn extract_and_restore(p)
    requires live(p), full(p)
    accesses reads(target(p)), writes(target(p))
    ensures live(p), full(p)
{
    v = take(p)
    put(p, v)
}
```

Two `inspect` calls may overlap on the same target, absent other effects or
dependencies. `inspect(p)` and `extract_and_restore(q)` on the same target
cannot overlap merely because both calls preserve the boundary state: the
second exposes an intermediate hole. They can execute in source order; on
proved-disjoint live initialized scalar targets their memory accesses can
overlap. Accesses here summarize the whole call, not only the final state delta.
The notation does not settle syntax or the general contract proof algorithm.

## Representative engineering tasks

These tasks define the initial comparison surface approved for preparation on
2026-09-17, not a completed capability audit or a fixed implementation strategy.
The first row anchors existing computation forms. The remaining rows test the
broader engineering objective; existing WF need not already support each one.
IO, FFI and device interaction are excluded. Resource examples can use ordinary
memory and explicitly linear program values. Each retained candidate needs a
concrete rendering, safe and dangerous variants, and its own derivation.

| Task | Observable behavior to preserve | Couplings to exercise | Costs to record |
|---|---|---|---|
| Array/column compute kernels | Same-index updates, independent columns, adjacent-range helpers and admitted reductions preserve source results and current overlap permissions. | Layout, bounds, function effects, alias relations, iterations and recombination. | Sequential instructions and memory traffic; available parallelism; guards, copies and temporary allocation. |
| Growable container of resources | Insert, grow, replace, remove and transfer elements without duplicating or losing their obligations. | Storage relocation, copy/affine/linear, element access, branches and helper contracts. | Element movement, allocation, retained views/cursors under the chosen semantics, and proof work. |
| Mutable compiler graph | Add/remove nodes and edges, traverse cycles and reclaim nodes with bounded reuse; a relation to a deleted node must not silently become a relation to a replacement node. | Stored associations, dynamic selection, identity/reuse, invariants and independent node-data work. | Per-node/edge metadata, traversal indirection, update work and parallel map opportunities. |
| Allocator over reserved memory | Allocate several independently usable blocks, release one and reuse its bytes without invalidating the others. | Parent metadata, disjoint payloads, allocation/release effects, identities and storage obligations. | Metadata work, synchronization if chosen, block fill overlap and fragmentation policy costs. |
| Recursive tree transformation | Visit and rewrite an owned tree, transfer subtrees across helpers and return a new root while disposing of removed resources. | Recursive layout/functions, ownership transfer, generics, replacement and cleanup. | Deep copies, temporary allocation, address stability and proof annotations. |
| Shared object registry and indexes | Reach one logical object through several indexes, update it coherently, remove it and repair or invalidate those associations. | Multiple access paths, stored relations, abstraction and selective fact invalidation. | Duplicated state, lookup/update complexity, mandatory scans and index maintenance. |
| Resource-processing pipeline | Transfer a non-copy resource between stages, finish or return it on every typed outcome and loop exit; independent work remains eligible to overlap. | Copy/affine/linear, returns, branches, loops, early exits and parallel consumption conflicts. | Cleanup data/branches, copies, per-stage allocation and usable overlap. |
| Generic algorithm through helpers | Express representative container/tree/range tasks over user types and passed functions without inspecting callees at every call site. | Parametric contracts, target and ownership relations, callback effects and proofs. | Interface/proof size, required specialization, code duplication and lost optimization facts. |

Safety is non-negotiable in every row. A "derived" result must explain how its
facts are established and maintained, including those attributed to a library.
Changing a representation is permitted but not free: state the same-task
translation and its costs. During prose design, cost entries are predictions
with explicit grounds, not runtime measurements. The task set does not yet
establish whole-compiler/browser/kernel expressiveness; concrete workload sizes,
baseline programs and further counterexamples refine it.

The task matrix carries derived/refuted/unresolved status per candidate. A task
rendering also keeps at least one unsafe variant and one cross-feature variant
where applicable. Unchanged derivations are linked, not copied into every round.
Existing P1-P19 and CASES.md provide useful source witnesses, but their original
syntax or implementation-specific expectations do not override these task goals.

## P1 Container split with a runtime index

```text
v: growable vector of u64, len n
(part_i, part_j) = take two elements i and j out of v for exclusive writing
write through part_i; write through part_j          // must be allowed, and may run in parallel
read v[k] for a runtime k                            // legal iff k is provably not i and not j; else the writer must branch
put both parts back; push(v, x)                      // must be allowed only after both are back
```
Required: sequential legality by proof, not by borrow scope; parallel permission
for the two writes; a diagnostic at the read of `v[k]` that names the missing
fact; no runtime check.

## P2 Arena with independently released blocks

```text
arena A with metadata M and byte storage S
b1 = alloc(A, 64); b2 = alloc(A, 64); b3 = alloc(A, 64)
fill(b1) ∥ fill(b2) ∥ fill(b3)                        // must be permitted to overlap
free(A, b2); b4 = alloc(A, 32)                        // reuse of b2's bytes; every old pointer into b2 must be refused afterwards
read(b1); free(A, b1); free(A, b3); free(A, b4)
```
Required: blocks are exclusively owned; metadata is written only at alloc and
free; nobody holds A exclusively between operations; a pointer into freed b2
is refused even though its address is reused by b4.

## P3 Graph with back edges in a pool

```text
nodes: pool of Node { next: handle-or-pointer, prev: handle-or-pointer, data: u64 }
insert node n between a and b (four link writes)
remove node m; the storage of m is reused for a later insert
traverse from any node following next, reading data
parallel map over all nodes writing data              // must be permitted: nodes are distinct
```
Required: link updates through several paths to one node are legal
sequentially; removal invalidates every path to m; parallel data writes are
permitted by distinctness of nodes; the invariant "next/prev point to live
nodes" is stated once and reused.

## P4 Cursor over a growing vector

```text
c = Cursor { at: pointer-or-handle to v, i: 0 }
a = c.read()                                          // needs v initialized and i < len
push(v, 5)                                            // may or may not reallocate
b = c.read()                                          // legal iff the candidate can carry "still valid" across push
free(v); c.read()                                     // must be refused
```
Required: a stored pointer or handle to a container in a struct; validity
decided by state and proof, not by a borrow that blocks push.

## P5 Struct-of-arrays kernel

```text
struct Cols { a, b, c, d, e, f, g, h: buffer<u64> }
for i in 0..n { a[i] = f(a[i], c[i], d[i]); b[i] = g(b[i], e[i], f[i], g[i], h[i]) }
```
Required: the eight columns are provably distinct storages inside the loop so
the backend can be told so (per-load alias facts, no runtime guards); PAR-2
iteration overlap permitted; obvious shape is the fast shape.

## P6 Range partition helper

```text
out: buffer of n bytes; k workers
for w in 0..k { helper(slice_of(out, w*s .. (w+1)*s), input) }   // helper writes its slice only
```
Required: iteration overlap permitted from the arithmetic of the ranges; the
helper's signature alone tells the caller what it writes; no worker count in
the source beyond k.

## P7 Take, put, replace through aliases

```text
p, q point to the same slot A holding an affine value
v = take(p); read(q)                                  // refused: hole
put(q, move v); read(p)                               // allowed
old = replace(p, new); read(q)                        // allowed, reads new
free A; read(p)                                       // refused forever
```
Required: state is a property of the storage, not of the pointer; take leaves
a hole that another alias may fill; free is permanent.

## P8 Conditional release and loop exits (CASES.md B10-B13, L04-L06)

```text
if c { free(a) } else { free(b) }; then use the survivor; then free it
loop { if stop { break }; take(p); ...; put(p) }; free(p) after the loop
```
Required: no runtime drop flag; the writer's own branches carry the state;
rejection names the path whose state differs.

## P9 A callee that changes storage state

```text
fn drain(x) leaves x's slot empty
fn reserve(v, m) may replace v's backing when cap < m
fn pick(c, x, y) returns one of two pointers with its permission
```
Required: each effect is visible in the signature; the caller judges by the
signature alone; the callee body is checked once against it.

## P10 Two holders, occasional writes, across threads (future)

```text
counter shared by two threads; each increments occasionally; a third reads
```
Required: synchronization cost only at the write; what the checker knows
before and after acquire; whether determinism is promised.

## Existing corpus programs to preserve

wfgrep, zlib-core-kernels, compute-bench, the accumulators snapshot cases:
whatever the current design permits to overlap under PAR-1 and PAR-2 must
remain permitted or the loss must be stated.

## Pending programs named by VERDICT-D0

Rows whose acceptance test names only one of these are provisional until the
program is written out and derived against.

### P11 Write-once-then-frozen cache

```text
cache: slot per key, initially empty
get(k): if cache[k] empty { cache[k] = compute(k) }; return read(cache[k])    // many readers, one lazy writer per slot
```
Required: state the pass or fail under R1 alone (a slot goes Uninit to Init
once and never changes); whether a reader may hold a pointer to a slot across
another slot's fill; why-whitefoot section 5's stance that the absence of such
cells is a performance argument is recorded, so admission is a capability
question.

### P12 DMA escrow

```text
buf = alloc(n); map_for_device(buf)         // device may write buf; the program may not touch it
wait_completion()                           // a program-observed event
unmap(buf); read(buf)                       // legal only after the completion
```
Required: the loan to a non-program agent is an R2 obligation discharged by
the observed completion; which contents facts survive between map and unmap
(R14(iv)).

### P13 Relocation by a compacting third party

```text
h = handle into a compacting pool; p = pointer_of(h)
pool.compact()                              // may move the object
read(p)                                     // refused unless a fixup form exists
read(pool[h])                               // legal: the handle survives compaction
```
Required: interior pointers and third-party relocation are mutually exclusive
without a contract-visible fixup form (R8a).

### P14 A descriptor as three identities

```text
f1 = open(path); f2 = dup(f1)               // two wrappers, one open-file description, one foreign contents
n = read(f1, buf, 100)                      // may return a short read: a typed outcome
seek(f2, 0)                                 // moves the shared cursor
close(f1); read(f2, ...)                    // still legal; close(f2) discharges the second obligation
```
Required: the wrapper carries the close obligation, the description the
cursor, the contents are foreign; the short-read arm discharges every
obligation (R10, R12, R2).

### P15 Reductions under a law and under a level

```text
s = 0; for i in 0..n { s = s +wrap a[i] }               // associative and commutative: any tree
x = 0.0; for i in 0..n { x = x fadd a[i] }              // no law: named weaker level or sequential
```
Required: ground (ii) admits the first under a fixed-table law; the second is
admitted only under a named level from R14(ii) or stays sequential.

### P16 Abstraction over identities

```text
fn map_nodes<T>(g: pool of Node<T>, f: fn(&T) -> T)     // P3 generic over the node type
fn kernel<E>(c: Cols<E>)                                // P5 over the column type
let h = |slice| helper(slice, input); P6 with h         // P6's helper as a closure
fn pick<N: Nominal>(c: Bool, x: N, y: N) -> N           // P9 through a nominal
```
Required: identity, state and effect facts cross the abstraction boundary;
parameters per interface bounded by the storages the callee touches (R15).

### P17 Partial operations

```text
q = a / d                                   // d a runtime value: proof d != 0 or a written outcome
b: u8 = narrow(x)                           // proof x < 256 or a written outcome
y = v[i]                                    // proof i < len(v) or a written outcome
```
Required: each accepted only with a proof or an outcome arm; the diagnostic
names the missing fact (R11, M7).

### P18 Allocation failure and a byte budget

```text
arena A with budget B bytes
b1 = alloc(A, 64)?; b2 = alloc(A, 64)?     // each may fail: the arm must free nothing twice and leave earlier blocks owned
b4 = alloc(A, 32)?                          // fits within B or the program is refused
```
Required: R2 holds on every failure arm; the declared budget is proved or
refused (R12, R13; owner decision 5).

### P19 A declared depth bound

```text
fn walk(n: Node, depth: u64) requires depth <= 64 { ... walk(child, depth + 1) ... }
```
Required: the declared count is proved on every recursive call; a termination
measure is supplied by the writer (R13).

### P10 trigger

P10 is derived only once a thread construct and R14(i)'s ordering vocabulary
exist; until then it is a pending program.
