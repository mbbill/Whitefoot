# Proof-derived parallelism and the `pal` structural contract

Status: RESEARCH ONLY. This document records a design hypothesis and paper
simulations. It is not part of the Whitefoot specification, does not select a
language syntax or runtime, does not amend the Direction Outline or Current
Plan, and authorizes no implementation, specification, conformance, or gate
change. `pal` is a provisional spelling used to make the hypothesis concrete.

The active specification and implementation remain authoritative. In
particular, Whitefoot currently has no parallel construct, parallel checked IR,
or parallel runtime. The current outline describes writer-declared parallelism;
this investigation deliberately studies a different starting point and cannot
supersede that outline direction by itself.

## 1. Research question

Can Whitefoot treat parallelizability as a proved property of ordinary program
shape, then let an implementation opportunistically execute a proved
decomposition in parallel, without exposing threads or trusting a writer's
parallelism assertion?

The intended result is stronger than a safe thread API but narrower than an
omniscient automatic parallelizer:

- the writer writes ordinary loops and recursion;
- the checker examines those forms whether or not they carry a marker;
- the checker derives only plans belonging to finite, specified proof families;
- a `pal` marker, if retained, demands that the ordinary code expose such a
  plan, but supplies no fact or permission;
- the runtime chooses serial or parallel execution according to input size,
  load, and resources that are not part of the source language; and
- every chosen execution refines one sequential reference semantics.

This does not claim that a finite checker can discover every mathematically
possible parallel algorithm. An algorithm can be parallelizable only after a
non-local reformulation that no bounded checker should attempt to synthesize.
The useful completeness target is instead: inspect every ordinary loop and
recursive site, and classify every opportunity expressible in the selected
plan families, with a precise reason when classification fails.

## 2. Three different subjects

This investigation keeps three often-confused subjects separate.

| Subject | Meaning here | Initial priority |
|---|---|---|
| **Parallelism** | Overlap otherwise independent computation to reduce the latency of one logical operation. | Primary subject. |
| **Concurrency** | Keep several independently progressing interactions alive, especially external or blocking interactions. | Important but deferred from the first design. |
| **Runtime resources** | The physical and operating-system means used to perform work: threads, workers, cores, heterogeneous core classes, cache allocation, NUMA placement, accelerators, bandwidth, and power budgets. | Runtime and target concern, not a source-language model. |

A file search illustrates the distinction. Scanning two already-owned byte
buffers is computation parallelism. Having several file reads outstanding is
external-I/O concurrency. Assigning the scans to two performance cores, four
efficiency cores, or one core with vector units is resource mapping.

The language should not make `thread` the unit of parallel meaning. A thread is
only one runtime implementation technique, and even a thread abstraction says
nothing about the other resources that determine performance. The source
program should describe values, ownership, effects, control flow, and proof
obligations. It should not describe a hardware topology.

## 3. Starting principles

### 3.1 The ordinary program is the authority

The presence or absence of an author marker does not determine whether work may
overlap. Memory accesses, ownership, effects, control dependences, and result
construction determine that.

Consequently, an unmarked loop or recursive call tree is analyzed exactly as a
marked one. If the checker derives a legal plan, the implementation may use it.
If no plan is derived, the sequential program remains the meaning.

### 3.2 Sequential reference semantics comes first

Every accepted program has one ordinary sequential reference semantics `S`.
For a deterministic compute kernel, `S` fixes one execution for a source input.
For a program that interacts with a host, `S` is additionally parameterized by
the same admissible environment trace; it is not a closed function of source
input alone. A parallel plan is legal only after the checker establishes that
all of its language-visible observations refine that same reference. Parallel
execution is not a second semantics and is not an opt-in relaxation.

At minimum, the observations include:

- the returned value and normal/error variant;
- all source-reachable memory results;
- ordered external output and other visible external effects;
- normal versus trapping completion;
- the selected trap's identity and mandatory record;
- ownership transfer and required normal-edge cleanup; and
- every deterministic ordering promise made by the language.

Execution time, worker count, task identity, and target resource placement are
not observations. Resource exhaustion remains governed by the language's
separate TCB boundary; it cannot be used as proof that a transformation
preserves ordinary language behavior.

### 3.3 Permission is not profitability

`a = f(); b = g();` may contain two provably independent computations and still
be slower when packaged as separate tasks. The proof answers whether overlap is
legal. It cannot, by itself, answer whether overlap is worthwhile for this
input on this machine.

This distinction prevents two symmetrical mistakes: rejecting a useful source
shape merely because one target has no spare execution resource, and eagerly
spawning tiny independent operations merely because the checker can prove
them independent.

### 3.4 A marker may create an obligation, never authority

`pal` is not analogous to a trusted `independent`, `noalias`, or race waiver.
It gives the checker no new premise. It says only that failure to derive a
nontrivial parallel decomposition is a source-shape error at this site.

## 4. The four-layer model

For a checked kernel `K`, keep four products distinct:

1. **Decomposition `D(K)`**: one or more candidate, finite plan families that
   divide `K` into work items and reassemble their results.
2. **Permission `P(K, p, g)`**: the proof that exact plan member `p`, under its
   structural guard `g`, may overlap its selected work items without changing
   the sequential observations.
3. **Profitability `C(plan, input, target, state)`**: an implementation decision
   about work, span, task overhead, locality, bandwidth, and current load.
4. **Mapping `M(plan, resources)`**: runtime placement on workers and physical
   resources.

The dependencies are:

```text
ordinary checked program K
        |
        v
candidate decomposition family D(K)
        |
        v
plan-indexed proof P(K, p, g)
        |
        v
verified plan family {(p, g, proof)}
        |
        +------------------------+
        |                        |
        v                        v
serial member             eligible parallel members
        \                        /
         +---- profitability C --+
                      |
                      v
             hidden resource mapping M
```

Candidate decomposition and permission are co-derived and publish atomically:
no kernel-level permission token may be detached and reused for another split,
join, guard, or failure order. Coarse non-interference facts may rule plan
families in or out, but only `P(K, p, g)` makes one exact member legal. That
proof still does not imply profitability. For example, many straight-line
statements commute but expose too little work. Conversely, a familiar parallel
algorithm is not permitted until its actual memory, effect, failure, and join
behavior has been proved for that decomposition.

`C` and `M` must not participate in source acceptance. A target with one worker
and a target with many heterogeneous cores accept the same program and can make
different execution choices.

## 5. Candidate formal shape

Let `Obs_seq(K, x, e)` be the sequential reference observation of kernel `K` on
legal input `x` under admissible environment trace `e`, and let
`Obs_plan(K, p, x, e)` be the observation produced by executing plan member
`p`, excluding only behavior already outside the language contract. For the
initial deterministic, claim-free compute kernels, `e` is empty. Let a plan
family `Pi` contain `serial(K)` and zero or more non-serial plans.

The checker may publish `Pi` only when, for every member `p` and every legal
input admitted by that member's structural guard:

```text
Obs_plan(K, p, x, e) = Obs_seq(K, x, e)
```

This equality is an exact language-observation equality, not merely race
freedom or equality of the final numeric result.

The analysis judgment can be sketched as:

```text
Gamma |- K => Pi
```

where `Gamma` contains only facts already admitted by the normative checker.
`Pi` is compiler-owned checked data, not writer-authored evidence. It records:

- the work-item domain and split operation;
- each work item's read, write, ownership, allocation, external, blocking, and
  trapping footprint;
- the structural eligibility guard, if any;
- the join and its required order;
- a proof reference for each non-interference and refinement fact; and
- the serial fallback.

An optimizer heuristic may discard or coarsen `Pi`. It may not invent a member
that lacks this proof.

### 5.1 Runtime guards and serial fallback

Some plan validity is input-dependent. A checked plan may therefore carry a
hidden structural guard, such as a proved range-disjointness test. The guard
must be total, nontrapping, effect-free, and unable to change a source-visible
result. If it is false, execution uses the original serial member.

This is distinct from a profitability guard:

```text
structural guard: is this non-serial plan legal for these values?
profit guard:     is using it likely to pay for itself now?
```

Both can select the serial member, but only the structural guard belongs to the
plan's proof. Worker availability, core class, cache state, and a grain-size
threshold belong to profitability or mapping.

A plan whose structural guard is identically false is not nontrivial and
cannot satisfy `pal`. An open question is whether `pal` should accept a plan
whose legality is conditional for a nonempty subset of inputs, or demand a
parallel member for every structurally splittable input. An initial experiment
should report this distinction rather than silently choosing it.

## 6. The provisional `pal` contract

The illustrative source form is:

```text
pal for item in items {
    body(item);
}
```

This is not a grammar proposal. Recursion may ultimately need a different
placement or no source marker at all.

The intended laws are:

1. The checker performs the same parallel-plan analysis on `K` and `pal K`.
2. `pal` contributes no proof fact, memory privilege, effect, runtime value,
   trap, worker request, scheduling parameter, or target assumption.
3. `pal K` is accepted only if the checker derives a supported plan family and
   that family satisfies the selected non-vacuity policy. Candidate policies
   include at least one non-serial member with a non-false guard, or the
   stronger requirement that every structurally splittable input has one; this
   document does not yet select between them.
4. If accepted, `K` and `pal K` publish the same checked kernel and the same
   plan family. Erasing `pal` changes only whether failure to derive a family
   satisfying the selected non-vacuity policy is rejecting.
5. Acceptance does not promise that any invocation will use more than one
   worker. Runtime selection may always choose the serial member.
6. A backend without a parallel executor may execute the serial member without
   changing source semantics or invalidating the structural proof.
7. `pal` disposition is a deterministic, target-independent checker judgment
   for one language version. Optional optimizer facts and runtime measurements
   cannot make rejected source pass or accepted source fail.

Thus `pal` is best understood as a checked performance-shape contract. It is
closer to a request for a machine proof than to a parallel command.

It gives an AI writer a hard feedback loop: if parallel form is a requirement,
the source must expose it in a shape the checker can prove. A loop-carried
dependence cannot be wished away by spelling `pal`; the code or algorithm must
change.

## 7. Proof inputs and permission

The first proof families should reuse facts Whitefoot needs independently for
memory safety and ordinary optimization:

- unique ownership and borrow exclusion;
- resolved places and storage origins;
- exact read and write effect projections;
- immutable shared reads;
- disjoint owned values and subobjects;
- statically bounded or proved-disjoint index ranges;
- injective indexed destinations, where a finite proof family can establish
  them;
- value flow and loop-carried identities;
- function requirements and verified normal-result relations; and
- exact trapping and external-effect categories.

At region granularity, two work items `A` and `B` satisfy the familiar
non-interference condition when:

```text
W(A) intersect (R(B) union W(B)) = empty
W(B) intersect (R(A) union W(A)) = empty
```

This is necessary but not sufficient. The checker must also account for moves,
lifetimes, cleanup, traps, external effects, and the join. Region effects are
also too coarse for two iterations writing distinct elements of one buffer.
Useful loop parallelism will require a bounded element/subrange proof family,
not a claim that region-level effects solve the entire problem.

When permission cannot be established, the classification must distinguish:

- **proved dependence**: the checker has a concrete carried or sibling
  dependence witness;
- **unknown non-interference**: the accesses may be disjoint, but the selected
  proof fragment cannot establish it;
- **unsupported decomposition**: independence may hold, but no selected join
  or recursive plan family represents the computation; and
- **not profitable**: a plan was proved, but the implementation selected its
  serial member. This is never a proof failure.

## 8. Candidate loop plan families

The plan vocabulary should start narrow. Each added family needs a verifier,
diagnostics, adversarial soundness work, and a real program that needs it.

### 8.1 Disjoint map

Each iteration reads immutable or shared-read inputs, consumes only its own
owned inputs, and produces a private result or writes a proved-disjoint output
slot. Results are joined in the sequential iteration order.

This is the smallest useful family and should cover ordinary transforms over
owned sequences and many tree-child computations.

### 8.2 Disjoint ranges and injective scatter

Iterations write subranges proved pairwise disjoint, or destinations
`out[index(i)]` under a checked injectivity fact:

```text
i != j  ==>  index(i) != index(j)
```

This family cannot be implemented honestly by treating one mutable buffer
region as disjoint from itself. It needs an exact place/subrange judgment whose
proof lifetime survives calls and iteration lowering.

### 8.3 Private production with stable ordered join

Each iteration builds a private sequence, diagnostic bundle, display-list
segment, or match result. The join concatenates or merges those results in the
reference iteration order. This converts a shared ordered sink into independent
production plus one deterministic publication boundary.

### 8.4 Checked reduction

A parallel reduction may regroup operations. It refines a sequential fold only
when checked algebraic laws over the reachable domain prove the regrouping
equal and the operation's totality/effects preserve failure behavior. Integer
overflow, floating-point non-associativity, claims, and order-sensitive
allocation can invalidate the plan.

No trusted `associative` or `commutative` annotation is acceptable. A first
runtime need not support reduction at all; disjoint map and ordered join are a
more defensible starting slice.

### 8.5 Checked scan and staged transforms

Some carried dependences are algorithmically expressible as a prefix scan:

```text
parallel measure -> checked scan -> parallel place
```

The scan operator needs the same checked algebra and exact-result constraints
as reduction. More importantly, the writer may need to expose the stages.
Discovering an arbitrary scan-equivalent reformulation from a sequential loop
is outside a bounded checker's expected role.

## 9. Candidate recursive plan families

Loops alone miss tree, divide-and-conquer, and irregular workloads. Recursion
should be an equal first-class analysis target.

### 9.1 Independent child subtrees

A call computes parent-derived immutable context, passes disjoint owned child
state into recursive calls, then combines private child results in child order.
Sibling calls may overlap. The parent phase and ordered join remain explicit
dependencies.

### 9.2 Range divide and conquer

A recursive call partitions one owned or uniquely borrowed range into proved
disjoint children, recursively processes them, and performs a checked join.
The runtime may stop splitting below a dynamic grain threshold and call the
sequential body.

### 9.3 Recursive serial islands

A recursive algorithm may be parallel only at some nodes. A node with a
cross-child cursor, float interaction, counter, or shared ordered sink remains
serial; an independent formatting-context or ownership boundary below it may
resume parallel decomposition. The plan is therefore a conditional task DAG,
not a promise that a whole recursive function is parallel.

Dynamic recursion depth and fan-out are runtime facts. The plan describes how
work may split; it does not prescribe a fixed worker set or eagerly allocate
one task per node.

## 10. Effects, claims, traps, and determinism

### 10.1 Effects

The easiest first plan domain is a worker closure with only shared reads,
proved-disjoint writes, private owned results, and no external or blocking
effect. Allocation may be admitted later when ownership, normal cleanup, and
resource behavior are accounted for; it should not be assumed harmless merely
because allocated addresses are not source values.

`external` and `blocks` identify a concurrency problem, not ordinary compute
parallelism. The first feature should exclude them from parallel workers. A
future concurrency design may use the same effect information but needs its
own ordering, cancellation, resource, and failure model.

### 10.2 Claims and traps

Claim admissibility and redundancy are a separate language question. This
document neither defends nor revises the active claim rules. It assumes only
the conservative boundary required by the active language: any claim that
survives checking is an executed semantic event, and a failing executed claim
selects an exact mandatory trap record and abort behavior.

It follows that `pal` cannot discharge, trust, suppress, or reorder a claim.
The smallest parallel plan family should require claim-free worker closures
and a claim-free transitive call closure. This avoids changing:

- which claim executes first in sequential order;
- which trap record is published;
- effects performed before the trap;
- the active no-unwinding/no-language-cleanup behavior; and
- whether speculative work after the reference trap executes.

A later design could investigate ordered failure as data: private tasks return
indexed outcomes, and an ordered join selects the first reference-order
failure. Under the active trap semantics that is not an implementation detail;
it would require explicit semantic and runtime analysis. Until then, a
claim-bearing region is a serial island rather than an invitation to weaken
trap observability.

### 10.3 Determinism

Scheduler nondeterminism must remain unobservable. A valid plan therefore
needs more than data-race freedom:

- private results are joined in a fixed reference order;
- reductions use proved exact algebra and a fixed admissible result;
- external publication occurs at an ordered boundary;
- no worker observes another worker's partial result without a proved plan
  edge;
- traps are excluded or selected exactly as the reference semantics requires;
  and
- changing worker count or stealing order does not change source results.

Randomized schedule testing is useful evidence, but it cannot replace the
static proof that grants permission.

## 11. Diagnostics and the writer repair loop

The checker should report the smallest blocking witness, not merely
"cannot parallelize." Examples:

```text
PAL-DEPENDENCE: iteration i writes cursor; iteration i+1 reads cursor
carried value: cursor
first write:   layout.wf:41
next read:     layout.wf:38
candidate repair: separate measure from placement, then use a checked scan
```

```text
PAL-UNKNOWN-DISJOINTNESS: iterations write out[perm[i]] and out[perm[j]]
missing goal: i != j ==> perm[i] != perm[j]
candidate repair: produce a checked injectivity relation or use private output
```

```text
PAL-ORDERED-SINK: every iteration writes output
the reference order is file order
candidate repair: return a private per-file result and publish with stable join
```

```text
PAL-TRAP-ORDER: recursive children may execute distinct retained claims
the first reference-order claim is not invariant under this plan
candidate repair: move proof/branching before the split or keep this node serial
```

Other useful blockers include overlapping unique places, a moved value needed
by a sibling, an external or blocking call, an unsupported reduction law, an
unknown recursive footprint, and a join whose order is not fixed.

Diagnostics should not pretend to synthesize a correct algorithm. A
measure/scan/place suggestion is useful only as a named candidate; the revised
source must still prove that measurement is independent of placement.

## 12. Errors, warnings, and optimization remarks

Not every sequential loop is defective. Parsers, state machines, prefix
computations, ordered publishers, and deliberately tiny loops can be exactly
the right shape. A blanket warning for every carried dependence would train an
AI to perform unsound or unprofitable rewrites.

The proposed feedback tiers are:

- **Hard error:** a `pal` site has no proved nontrivial plan. The diagnostic
  distinguishes dependence, unknown proof, unsupported decomposition, and
  forbidden effect.
- **Ordinary compilation:** every loop and recursive site is analyzed, and any
  proved plan is available without a marker. Failure to prove a plan does not
  by itself reject ordinary code.
- **Optimization remark:** an inspectable report records the plan found, the
  serial island, or the exact blocker for every analyzed site.
- **Performance warning:** a profile, workload contract, or high-confidence
  static work bound identifies a material hot site and the checker has an
  actionable structural blocker. This warning policy must be measured for
  precision before becoming a default W1 gate.

Useful remark categories might include `parallel.plan`,
`parallel.serial-small`, `parallel.blocked.dependence`,
`parallel.blocked.proof-unknown`, `parallel.blocked.effect`, and
`parallel.serial-island`. These are research names, not diagnostic IDs.

Crucially, "proved but not profitable" is never an error under `pal`, because
`pal` constrains structural parallelizability rather than current hardware use.

## 13. Paper simulation: DOM style, layout, and rendering

This simulation asks whether the model can describe a real recursive tree with
both abundant sibling work and unavoidable serial islands. It is not a claim
that the current Whitefoot system surface can implement a browser.

### 13.1 Reference boundary

Take one immutable DOM/style/resource epoch as input. Concurrent DOM mutation,
network fetch, font loading, and image decoding belong to a separate
concurrency/snapshot design. A layout run consumes the frozen epoch and
produces one fragment tree and ordered display list.

The coarse dependency graph is:

```text
frozen DOM/style epoch
        |
        v
top-down parent context
        |
        +--> child subtree 0 --+
        +--> child subtree 1 --+--> stable parent assembly
        +--> child subtree n --+
                                  |
                                  v
                      ordered display-list construction
                                  |
                                  v
                         tile/raster work candidates
```

### 13.2 Independent child recursion

A favorable source shape computes the context each child needs before the
split, gives each recursive call disjoint owned result state, and assembles
children in DOM order:

```text
layout(node, parent_context):
    child_inputs = derive_child_inputs(node, parent_context)
    child_results = for child in node.children:
        layout(child, child_inputs[child])
    return assemble_in_dom_order(node, child_results)
```

The ordinary `for` and recursive calls are analyzed without `pal`. A `pal`
marker could turn failure to derive the sibling plan into a hard shape error,
but would add no permission.

### 13.3 A real carried dependence

Naive vertical placement often has this form:

```text
cursor = content_start
for child in children:
    fragment = layout(child, cursor)
    cursor = fragment.after
```

This loop is not directly parallel: child `i + 1` reads a value produced by
child `i`. If the relevant CSS subset permits it, the writer might expose:

```text
metrics = map(measure, children)
offsets = scan(advance, metrics)
fragments = map(place, children, offsets)
```

That is a source/algorithm restructuring, not a scheduling flag. Floats,
margin collapse, counters, line breaking, fragmentation, or a child's size
depending on its final placement may refute the separation. The checker should
then retain the affected formatting context as a serial island while allowing
parallel recursion across independent boundaries below or beside it.

### 13.4 Ordered rendering products

Display-list construction is not made deterministic merely by writing to
different buffers. CSS painting and stacking order are observable. A plausible
plan lets each proved-independent subtree build a private display-list segment,
then performs a stable join after stacking-context dependencies have been
resolved. Global z-order relationships or shared spatial/clip tree mutation
remain serial until represented by a verified merge.

Raster tiles may expose a later disjoint-map family after the display list is
fixed. GPU queues, cache residency, and heterogeneous placement remain runtime
mapping, not language semantics.

### 13.5 What this case tests

The DOM case requires all of the following to work together:

- recursive, dynamically sized fan-out;
- parent-before-child and join dependencies;
- serial islands inside an otherwise parallel tree;
- ordered private-result merging;
- disjoint ownership across sibling work;
- snapshot boundaries separating concurrency from compute parallelism; and
- runtime grain decisions that avoid one task per small DOM node.

If the design can express only a flat array map and cannot describe these
boundaries, it is not yet a credible recursive parallelism design.

## 14. Paper simulation: wfgrep

The current wfgrep program searches an explicit ordered file list and performs
real blocking system I/O; it does not recursively enumerate directories. The
existing [traversal reconnaissance](../wfgrep-traversal/RECON.md) records that
boundary. This simulation therefore separates the existing search core from a
possible future traversal workload.

### 14.1 Whole-file decomposition

The sequential reference order is the ordered input path list. Directly
sharing `out` and `err` across workers is both an effect conflict and an output
ordering problem. The parallel-friendly compute shape is:

```text
ordered paths
    |
    v
acquire owned file snapshots          external I/O; concurrency question
    |
    v
scan snapshot 0 -> private result 0 --+
scan snapshot 1 -> private result 1 --+--> stable ordered join
scan snapshot n -> private result n --+             |
                                                    v
                                      one ordered output publisher
```

Scanning already-owned snapshots is computation parallelism. Overlapping
`open_read`/`read_once` operations is concurrency and should not be smuggled
into the first compute feature. Errors should be returned as values in private
per-file results. One ordered publisher reproduces stdout, stderr, and exit
status in reference file order.

Recursive traversal, when the system surface eventually supports it, can
enumerate and sort entries at a deterministic boundary, recursively produce
private subtree results, and concatenate them in path order. Directory streams
and reads remain external effects even if independent directory capabilities
make them safe to overlap.

### 14.2 Within-file decomposition

Large files offer another loop family, but arbitrary byte chunks are not
independent grep units. A match or line can cross a chunk boundary. A valid
plan needs one of:

- a first pass that proves line-aligned chunk boundaries;
- bounded halos with an exact ownership rule for boundary matches; or
- per-chunk summaries followed by a deterministic stitching pass.

Each chunk produces private match records tagged by byte position; the join
sorts or concatenates them in increasing reference position. A shared append
buffer is not accepted as parallel merely because individual writes are
short.

The matching worker should initially be claim-free and free of external or
blocking effects. The output publisher remains serial. Runtime profitability
can choose whole-file, within-file, nested, or entirely serial execution based
on file sizes and available resources without changing the source program.

### 14.3 What this case tests

The wfgrep case tests:

- private production plus stable ordered publication;
- an honest boundary between blocking-I/O concurrency and compute parallelism;
- dynamic work sizes and nested decomposition;
- chunk seam correctness;
- error-as-value aggregation rather than shared diagnostic effects; and
- runtime choice among file-level, chunk-level, and serial plans.

## 15. Candidate research sequence, not an execution plan

If this direction is selected through project governance, the smallest useful
experiments appear to be:

1. Define one analysis-only disjoint-map and recursive-child plan judgment on
   paper, including exact blockers and serial refinement.
2. Apply it manually to one frozen DOM-layout subset and one frozen wfgrep
   compute shape. Count proved plans, real dependences, proof-unknown cases,
   and required source restructurings.
3. Prototype a compiler report that analyzes ordinary loops/recursion but does
   not change lowering. Test whether its witnesses lead to bounded mechanical
   repairs.
4. Add a checked plan IR and a serial plan interpreter. Differentially verify
   it against the existing sequential lowering before adding workers.
5. Execute only the smallest claim-free disjoint-map/recursive-child family
   with a runtime threshold and serial fallback. Measure work, span, allocation,
   wall time, and deterministic results across worker counts.
6. Consider ordered private joins, subrange proofs, reduction, scan, or
   concurrency only when one of the two programs demonstrates the need.

These steps do not authorize work. They show how to falsify the hypothesis
before committing to a broad language or runtime framework.

## 16. Open questions

### Proof and language boundary

- What is the smallest normative plan vocabulary that covers useful loops and
  tree recursion without becoming an optimizer specification?
- Is plan discovery a source-acceptance judgment, a verified optimization
  fact, or two layers with different stability promises?
- What completeness statement can be made relative to the finite plan
  families without implying discovery of arbitrary algorithmic equivalence?
- How are generic functions classified when different concrete instances
  expose different plan families?
- Does `pal` accept conditional structural guards, and how is non-vacuity
  established?
- Where could a recursion marker live without making recursion mechanics part
  of a public API?

### Memory and effects

- Which existing place/origin facts suffice for child ownership, and which
  real cases require element/subrange disjointness?
- Can injectivity be proved with a small reusable relation vocabulary, or does
  it recreate the annotation burden of region/effect parallel languages?
- Can private allocation enter the first plan family without observable
  cleanup or resource anomalies?
- Is a claim-free transitive worker closure an adequate first boundary?
- Can any claim-bearing plan preserve the active exact trap and abort rules
  without turning traps into recoverable task results?

### Runtime and performance

- What checked plan IR admits serial interpretation, work stealing, coarsening,
  and target-specific lowering without exposing workers in source?
- Which runtime quantities choose between whole-file, chunk, subtree, and
  serial execution?
- How should nested parallelism avoid oversubscription and unbounded task
  allocation?
- Can locality and heterogeneous-core mapping remain entirely outside the
  language on DOM and wfgrep, or does real evidence force a source-visible data
  placement concept?
- What profiling evidence is reliable enough to turn a blocker remark into a
  W1 warning?

### Determinism

- What exact observation relation covers ordered output, traps, and cleanup?
- Which reductions are bit-exact under proved regrouping? Floating-point
  addition is not one by default.
- How are multiple possible child failures ordered without adding cancellation
  or recovery semantics prematurely?
- How can schedule fuzzing and a serial oracle test a plan independently of one
  scheduler implementation?

## 17. Falsifiers

This direction should be rejected or sharply narrowed if any of the following
survives a bounded attempt to fix it:

1. **No useful coverage.** The selected proof families find only tiny maps,
   while DOM and wfgrep require pervasive author decomposition or trusted
   annotations.
2. **No actionable feedback.** Most blocked sites end in "unknown alias" or
   "unsupported" rather than a small witness an AI can repair.
3. **Proof machinery dominates the feature.** Element/subrange, recursion, and
   join proofs require a generalized dependence framework larger than the
   programs and experiments they enable.
4. **Determinism collapses parallel coverage.** Preserving ordered output and
   exact trap selection serializes nearly all measured work.
5. **Runtime overhead erases the win.** A scheduler and private-result joins do
   not beat a strong sequential Whitefoot/Rust baseline on real inputs, or
   produce frequent regressions the cost policy cannot avoid.
6. **Resource abstraction leaks into semantics.** Competitive performance on
   the selected programs requires the writer to name threads, core classes,
   cache partitions, or target topology rather than letting mapping remain a
   runtime concern.
7. **`pal` changes authority.** Removing an accepted `pal` changes the derived
   plan or executable result, or adding it lets otherwise unproved overlap
   pass.
8. **Conditional plans become vacuous.** `pal` can be satisfied by a guard that
   almost never permits non-serial execution.
9. **Source-shape churn is unstable.** Small semantics-preserving refactors
   unpredictably lose plans and diagnostics cannot identify the invariant
   shape.
10. **The cases are really concurrency.** Measured wfgrep gains come primarily
    from overlapping I/O and cannot validate a compute-parallel feature.

Positive evidence would be narrower and concrete: unmarked ordinary code
produces a verified nontrivial plan; `pal` reliably rejects and explains a
deliberately introduced structural dependence; randomized schedules preserve
the serial oracle; and runtime selection yields a measured wall-time win
without source-visible resource configuration.

## 18. Prior art and lessons

These references are evidence and counterexamples, not authorities for
Whitefoot semantics.

- **Cilk and work stealing.** Cilk separates a computation DAG from dynamic
  worker scheduling and motivates cheap serial fallback, but the writer still
  expresses spawn structure. See [Cilk: An Efficient Multithreaded Runtime
  System](https://pages.cs.wisc.edu/~david/papers/ppopp1995_cilk.pdf) and the
  [MIT Cilk publication index](https://cilk.mit.edu/publications/).
- **Rayon.** `join` may execute two closures in parallel or sequentially under
  a work-stealing runtime, illustrating why a structural fork need not promise
  multiple workers. Rust's `Send`/`Sync` boundary and explicit parallel APIs are
  not Whitefoot's proposed source authority. See the [Rayon `join`
  documentation](https://docs.rs/rayon/latest/rayon/fn.join.html).
- **Tapir.** A task-parallel IR can preserve parallel structure below a source
  language and give ordinary compiler transformations a task-aware form. See
  [Tapir: Embedding Fork-Join Parallelism into LLVM's Intermediate
  Representation](https://dl.acm.org/doi/10.1145/3062341.3062342).
- **Deterministic Parallel Java.** DPJ demonstrates checked region/effect
  non-interference and deterministic parallel constructs, while also exposing
  the cost of fine-grained region/effect annotation and difficult patterns such
  as permutation. Its parallel constructs remain writer-expressed. See
  [Bocchino et al., OOPSLA 2009](https://rob-bocchino.net/Professional/Bocchino-OOPSLA-2009.pdf)
  and the [DPJ language specification](https://dpj.cs.illinois.edu/DPJ/Download_files/DPJSpecification.html).
- **Legion and Regent.** Logical regions and privileges separate logical
  independence from physical instance placement; the runtime can extract
  task-level parallelism and map it onto heterogeneous resources. Tasks,
  privileges, and partitions are nevertheless explicit program structure. See
  the [Legion publications](https://legion.stanford.edu/publications/) and
  [Regent overview paper](http://regent-lang.org/images/regent2015.pdf).
- **Halide.** Halide makes an algorithm's data-parallel semantics clear but
  shows that profitable scheduling remains a distinct and difficult search
  problem even when independence is known. See [Halide's CACM
  article](https://andrew.adams.pub/halide_cacm.pdf) and [Learning to Optimize
  Halide with Tree Search and Random
  Programs](https://halide-lang.org/papers/halide_autoscheduler_2019.pdf).
- **LLVM loop versioning.** Runtime alias checks used by vectorization are a
  precedent for a total hidden guard selecting an optimized path with a scalar
  fallback. They do not by themselves provide task decomposition or language
  determinism. See the [LLVM loop vectorizer
  documentation](https://llvm.org/docs/Vectorizers.html#runtime-checks-of-pointers).
- **Futhark and data-parallel combinators.** `map`, `reduce`, and `scan` make
  decomposition explicit and enable fusion and flattening, but they are not
  discovery from arbitrary sequential code. See the [Futhark PLDI 2017
  paper](https://futhark-lang.org/publications/pldi17.pdf).
- **Servo layout and style traversal.** Servo is a concrete recursive-tree
  witness: current Layout 2020 uses the same layout code for serial and
  parallel execution, permits parallel subtree work, and encounters serial
  regions around features such as floats and counters. Servo's style traversal
  also documents the remaining unsafe thread-safety bridge around DOM nodes,
  which a stronger proof system should aim to remove rather than imitate. See
  [The Servo Book: Layout](https://book.servo.org/design-documentation/layout.html),
  [Layout 2013 and Layout 2020](https://servo.org/blog/2023/04/13/layout-2013-vs-2020/),
  and [Servo's parallel style module](https://doc.servo.org/style/parallel/index.html).
- **Limits of general automatic parallelization.** Even ideal dependence
  information does not manufacture parallelism when true algorithmic
  dependences dominate. This bounds the claim Whitefoot should make about
  automatic discovery. See [Murphy et al., Limits of Dependence Analysis
  for Automatic Parallelization](https://users.cs.northwestern.edu/~simonec/files/Research/papers/HELIX_CPC_2015.pdf).

The repository's earlier adversarial survey remains useful evidence, but its
strong-versus-weak framing combines discovery, permission, profitability, and
mapping. This investigation replaces that combined question with the four
separate layers above. See [auto-parallelism feasibility
results](../../experiments/auto-parallelism-feasibility/RESULTS.md).

## 19. Current non-decisions

This document intentionally does not choose:

- final syntax or whether `pal` survives at all;
- a specification rule number or diagnostic ID;
- whether failure at an unmarked hot site is ever a default warning;
- a parallel checked-IR representation;
- a scheduler, worker count, thread model, or target mapping policy;
- a concurrency, cancellation, or asynchronous I/O model;
- reduction or floating-point regrouping semantics;
- a way to execute claim-bearing tasks in parallel;
- a DOM/browser system API; or
- an implementation batch.

The design hypothesis is only that ordinary code should be the source of
parallel permission, a finite checker should expose and explain supported
decompositions, `pal` should be a non-authoritative structural obligation, and
the runtime should retain complete freedom to choose serial execution and map
work onto resources hidden from the language.
