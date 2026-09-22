# Implementation contract for bounded execution

This proposal makes the [resource objective](README.md) concrete enough to
implement. It is not the active specification or an implemented compiler
capability. Its first consumer is a sequential no-heap calculation with
counted loops or ranked direct recursion, including `bounded-recursion.wf`.
An ordinary ranked loop uses the same progress rule. Mutual recursive
components, parallel execution, asynchronous entry and blocking operations
remain outside the first qualification domain.

The implementation is divided by ownership of evidence, not into independent
source checkers: source progress, structural cost composition, and target
resource qualification. A passed source check, a resource qualification and a
successful native run remain distinct results.

## Supplied stack budget

The deployment supplies a usable stack capacity S in bytes before
qualification. For the fixed entry contract, target configuration and actual
linked image, the required conclusion is that every admitted invocation's
peak stack use is at most S. A source activation count is intermediate
evidence, never the resource limit or a sufficient success criterion.
Even one nonrecursive activation can exceed S; many small activations can fit.

The checker combines proved path/depth bounds with complete machine frame
and edge costs, including entry, exit, helpers, spills, temporaries and
alignment. In a simple uniform-frame case this yields `O + D*C <= S`, where
O accounts for the remaining peak-path bytes and C is the complete incremental
cost of an activation. For `C > 0` and `S >= O`, the budget can be restated as
`D <= floor((S-O)/C)`, but the checker must still prove the actual path fits
that derived bound. It must not impose an independent depth cap, strengthen
the entry contract silently, enlarge the supplied region or rely on stopping
execution when a counter runs out. Nonuniform paths use the full frame/edge
composition below.

The fixture's `remaining <= 32` is an algorithm input domain chosen for a
small exhaustive experiment, not a proposed stack policy. Its rank snapshot
is named `entry_remaining` below to keep that distinction visible. Source
rank checking can be implemented first as a dependency; the first usable
fixed-stack milestone must take S as input and establish the byte inequality.
No source-only result may announce that the supplied stack is sufficient.

## Source proposal

Add one proof-only rank clause. The proposed grammar delta is:

```text
rank_clause    := "decreases" IDENT ":" affine_expr
contract_block := "contract" "{" contract_define* requires_clause*
                  (rank_clause ";")? ensures_clause* "}"
loop_header    := header_invariant | rank_clause
loop_stmt      := "loop" LABEL? ("(" loop_header ("," loop_header)* ")")?
                  "{" stmt* "}"
```

A contract rank belongs to a body-bearing `fn_decl`, not to a `fn_sig` or
callable refinement. A loop has at most one rank. Counted `for` already has
an intrinsic finite counter, so it gains no second rank spelling. The delta
adds two productions and one fixed word, `decreases`; it changes two existing
productions. Canonical rendering, keyword exclusion and declaration resolution
must change with them. A proposed new RES-1 rule owns formation, captures and
progress obligations; INV-1/PRF-1 remain the arithmetic proof authority.

For example, this **proposed**, currently unaccepted fragment adds a rank to
the existing recursive helper's contract:

```text
contract {
  requires remaining <= deref(data).len;
  requires remaining <= 32_u64;
  decreases entry_remaining: remaining;
}
```

`entry_remaining` denotes the mathematical value of the expression at this
activation's entry. It is an immutable, erased proof datum visible only in
affine proof expressions in the function body. It is not a runtime local, argument, effect,
measure, source value or callable promise. It cannot occur in executable
expressions, `requires`, `ensures`, or its own defining expression. A loop's
rank name similarly denotes the current iteration's header value and is
visible only in the loop body, including its nested bodies. Names are fresh
against visible value, invariant and rank names; a rank name is an affine
operand, never the bare proof-premise alternative of `use`.

Rank expressions use INV-1's affine formation and structural ceilings.
At a function boundary their operands are admitted parameter values and
parameter-rooted measures, integer literals and admitted constants. At a
loop they use the live preheader vocabulary of INV-1. Calls, ownership moves
and side effects cannot be rank expressions. The first rule proves
`0 <= R <= 2^64 - 1` at entry; this bounds one scalar progress measure without
adding a runtime integer operation or wraparound. Wider or lexicographic
measures require a later admission change backed by a consumer.

For a recursive call, instantiate the callee's rank over the **evaluated
actual argument images**, after ordinary argument evaluation and its checks,
before publishing any callee result facts. Prove `0 <= R_actual < entry_remaining`.
The comparison is with the containing activation's entry snapshot, not the
current values of its mutable parameters. Capture an argument at its own
evaluation point; later argument effects cannot retroactively change its
value. For a reference argument, that capture freezes the reference, not its
referent: a reference-rooted rank measure reads the callee-entry state after
all argument effects, using the existing CALL-6 substitution boundary. The
first recursive qualification domain is a one-member component;
its direct self edge includes calls later lowered as FN-10 transfers.

An ordinary loop first proves the rank range in the preheader. At its generic
header, after the existing invariant base batch succeeds, capture the rank
and prove the same range in that header's established context. On every
normal backedge prove `0 <= R_next < rank`. Break, return, give and propagation
follow their existing target edges; only an edge returning to this loop's
header owes this descent. In particular, `give` resumes its value initializer
and can subsequently reach that same backedge. An inner loop creates its own
snapshot and cannot refresh the outer one. A loop with no structural backedge
needs no rank for progress. A proved-contradictory edge retains the contradiction root, rather
than disappearing from the source audit.

The rank name lets a preceding ordinary local invariant expose a hard descent
fact using PRF-1, for example `invariant step: next < entry_remaining;`.
Header ranks need no new inline proof language. Function-entry requirements, preheader
invariants and local backedge/call-site invariants are the existing places
to establish their premises. Merely writing a rank publishes no unproved
inequality.

Each written clause is checked on the ordinary semantic path, in symbolic
templates and concrete instances under FN-2. An invalid written clause is a
source error whether or not a resource report was requested. Absence of a
rank does not reject an otherwise valid ordinary program. It prevents a
requested completion guarantee when a relevant cycle lacks evidence.
Function-kind actuals are selected before concrete resource composition;
their actual bodies must meet progress obligations independently of the
formal's ordinary contracts. No new function-refinement dimension is needed.

## From local facts to complete progress

Local descent is only one input to a completion result. Visit the concrete
root's complete structural call closure, collecting calls in expressions as
well as statement position. Every body loop and callee must have a progress
account. The root must satisfy the ordinary launch/entry contract; an
uninhabited entry is not permission to emit an executable with a zero budget.
Maintain the ordinary source graph and provenance even where a
bound or contradiction makes the resource contribution zero.

Compose components in deterministic callee-before-caller order. Acyclic
functions complete when all their structured statements and callees do.
A self-recursive function additionally needs a proved rank decrease at every
self call; well-founded induction on the entry rank discharges those calls.
The proof also checks all nonrecursive work in the body. A multi-function
recursive component is an explicit unsupported resource capability in the
first implementation, not an ordinary source rejection. Never reuse a sum
of SCC frames as a proof of a number of visits to that component.

The existing FN-9 schedule withholds same-component postconditions; retain
that boundary. A resource proof cannot publish a hypothetical termination
summary and use it to establish itself. Unknown linked bodies remain pending
target obligations even if their signatures are `pure`. A primitive's value
contract alone does not establish progress of its selected native body.

For the source loop/call model, a rank with proved entry maximum B permits
at most B backedges and B+1 header visits, or B+1 nested activations. A counted
loop's body count is at most `max(0, U-L)` for its captured endpoints, with
the header/update work counted separately. Early exits may reduce these
numbers. Use a uniform maximum for nested bodies in the first composition;
tighter dependence on an outer iteration is a precision improvement.

Obtain B by reusing the existing `project_numeric_upper_bound` path after
proving the rank's u64 ceiling: the admitted ceiling, tighter closed L0 bound,
then the fixed coefficient-one affine interval projection, each retaining
its derivation. This procedure is already used to hand allocation ceilings
to lowering. It does not infer a rank. A preceding checked invariant can
expose a tighter endpoint without extending AUTO. A maximum of `u64::MAX`
is valid evidence but will normally be too coarse for a stack budget.

## Cost algebra and its limits

Keep total work and maximum simultaneous storage separate. A work unit in the
first source report is an executable checked statement/expression node visit,
counted with its component summary; erased proof constructs contribute zero.
Aggregate operations also report their bounded element
or byte extent. It is not a CPU cycle or a final machine instruction. Native
helper progress and resource coverage are separate obligations. A future
hardware time budget must introduce and validate its own cost interpretation.

For each function, calculate a nonnegative upper bound A for one activation's
nonrecursive work, including completed callees, and M for the number of self
calls one activation can perform. Sequence adds, alternatives take a maximum,
and bounded loops multiply their body's upper bound plus their own overhead.
Use the maximum of A and M across branches even when they cannot occur on
the same path. With rank maximum B:

```text
W(-1) = 0
W(r) <= A + M * W(r-1), for 0 <= r <= B
```

The deliberately conservative base includes A even when the zero-rank arm
does less. M=0 is acyclic work; M=1 is at most `(B+1)*A`; M=2 already grows
exponentially with B. Evaluate the affine map `x -> M*x + A` to power B+1
by repeated squaring. Do not recursively unfold B levels in the checker.

For a requested numeric budget Q, use nonnegative arithmetic clipped at Q+1.
Addition, multiplication and maximum commute with this clipping, including
`0 * (Q+1) = 0`. Checked u128 intermediates suffice when Q is a u64 request;
an overflowing nonnegative product is certainly above Q. B+1 is computed
in u128, so B=`u64::MAX` is represented. At most 65 exponent bits are visited.
This is exact evaluation of a clipped bound, not a compiler work budget.
All fixed proof families still run to completion. A clipped result says only
that **this conservative upper bound does not certify Q**, not that actual
execution necessarily exceeds Q. Keep missing evidence distinct from that
result. Do not silently saturate an unknown value to zero or to a supposedly
proved finite maximum.

For stack, retain the [frame/edge equations](README.md#target-storage-and-correspondence).
Serial sibling calls take a maximum, not a sum. A loop reuses its activation's
storage; its iteration count multiplies work but not stack. The source bound
on activations alone cannot license a machine-frame product until the target
mapping described below is established.

## Compiler responsibilities and interfaces

These are internal responsibilities, not a stable certificate/file protocol.
The root compiler decision rejects such infrastructure before a consumer.

| Owner | Proposed change | Preserved boundary |
|---|---|---|
| Grammar, resolver, `semantic/check/control/proofs.rs` | Form rank clauses and proof-only names; check affine vocabulary | One syntax and semantic path, no expression recognizer |
| `semantic/model.rs` | Retain the checked no-heap declaration and add rank declarations/capture identities to functions and loops | Concrete function and source-node identities remain authoritative |
| `semantic/entailment/flow.rs` | Issue entry/header/actual/backedge queries at their existing flow events; retain snapshots, bounds and roots | Reuse ProofContext and current effects/kills; no second affine solver |
| `semantic/entailment.rs` | Retain rank outcomes in FunctionEntailment and remap their live derivation roots at finalization | Source evidence remains in the existing DAG |
| `semantic/check.rs` and a focused `semantic/resources.rs` | Build complete call/loop coverage and compose source summaries after accepted ordinary proofs | Reuse call traversal/SCC primitives, not the conditional FN-9 schedule as a complete graph |
| `lowering.rs` and its builder | Carry only proved maxima and origin identities on functions, loops and calls, including FN-10 loop origins | Lowering never re-proves source arithmetic or reads rendered diagnostics |
| `backend/stack_ledger.rs`, target and driver | Read a structured complete machine inventory; compose target bounds; render the existing ledger separately | Developer text is not evidence consumed by qualification |
| Driver/runtime packaging | Link and account for the actual entry adapter and native closure | No source operation-name classification chooses a different semantic rule |

The checked result needs explicit progress coverage, B and its proof, and a
work-bound expression or evaluated bound for every relevant construct.
Coverage identities include the containing concrete function and exact call
or loop node. Snapshots are owner-tagged finite identities, like current
capture identities: one entry image or one arbitrary current header image,
never a fresh check-time identity on each checker replay. Loop exit removes
the proof-only name from scope without equating two runtime iterations.
Bodyless functions carry pending native obligations, not zero-cost summaries.

Keep the new resource consumer focused; do not turn the 17,000-line flow
implementation into a second graph/cost engine. The flow owns program-point
facts. The resource module consumes retained conclusions and structured
control, and the target module consumes lowering conclusions. Generalizing
all existing proof consumers into a plugin framework has no current benefit.

## Target and runtime boundary

The first complete qualification target is one synchronous no-argument WF
entry on a supplied fixed stack, returning its ordinary status to the host.
Its precondition names the fixed stack's writable extent and alignment,
executable/read-only regions, single invocation, and absence of asynchronous
callbacks on that stack. Host loading, allocation of the supplied regions and
the suspended caller's stack are outside this invocation claim and must be
accounted for by a larger deployment. It is not a claim about total macOS
process memory or bare-metal startup. This boundary is explicit because the
current ordinary runtime creates threads and mappings.

The compiler's eventual supplied-stack packaging must use the same checked
body, ABI and result conversion. A research assembly adapter can establish
whether that boundary is practical; it is not the production runtime choice.
Native bodies, ABI shims, aggregate-copy helpers and target stack-probe helpers
must all be accounted for by their actual linked definitions. Missing native
progress, stack or allocation evidence means no complete qualification.
No user-written numeric assertion qualifies a native body.

Produce assembly and stack usage once, assemble that exact text to an object,
and link that object. Include every selected native object in the same
inventory. Retain all direct calls and tail edges, unresolved targets,
indirect transfers, frame qualifiers and arithmetic failures. An unsupported
dynamic stack adjustment or indirect target remains an explicit gap. The
current ledger silently skips unmeasured targets and malformed frame rows,
uses saturating sums and excludes cycles from its acyclic chain figures;
none of those diagnostic conveniences can decide qualification.

Post-link checks must resolve actual symbols and record introduced stubs,
helpers, executable/static sections, alignment and reserved regions. Disable
LTO for the first qualified target, so linking does not replace the analyzed
machine bodies. Verify the linked code against the analyzed objects apart
from understood relocations. A digest binds the report to those inputs and
output; it is not the proof. A changed image invalidates the association.

Source progress remains valid under the compiler's semantic-correctness trust
boundary. A numeric resource bound needs the stronger call/loop correspondence
as well. For the first implementation, retain source call/loop origins through
Whitefoot lowering and qualify only a backend configuration whose relevant
activation mapping is accounted for. Inlining can put several source frames
inside one measured frame; newly outlined helpers or cloned recursive cycles
cannot simply inherit a bound by spelling. A missing mapping is unsupported
resource qualification, not a guessed limit. A preserved recursive boundary
can use `noinline` as an implementation control, but that attribute alone
does not prove resource correspondence or bound optimizer-created work.

Start source implementation without promising a certified arbitrary `-O2`
image. The next target implementation must first make that correspondence
observable and testable; ordinary optimized execution remains the comparison.
The source work report is never relabeled as a machine-instruction budget.
This separates an implementable source consumer from an unsupported whole
toolchain certification claim.

## Implementation admission and evidence

Implementation can start with RES-1 and source summaries now: formation,
capture semantics, proof timing, graph coverage, bound algebra and ownership
of each interface are specified above. Do not postpone those to design a
stable artifact protocol. A first source implementation is complete only when
the following cases run on the ordinary checker/lowering path:

| Case | Required observation |
|---|---|
| Existing unannotated programs | Same ordinary acceptance and result |
| Ranked self recursion, including FN-10 | Proved decrease and a retained finite source bound |
| Unchanged actual; reset then decrement | Descent fails against the activation-entry snapshot |
| Ranked loop; unchanged backedge; nested reset | Every continuing edge checked against its own header snapshot |
| Counted loop with nonterminating callee | Finite counter does not establish completion |
| Argument effects and alias writes | Captured actuals and rank snapshots retain the intended time identity |
| Branch-only proof; missing edge | No function-global memo supplies another edge's proof |
| Function-kind actual, generic instance | Resource closure follows concrete selected bodies |
| Binary recursive calls; early exit; zero-trip loop | Work and simultaneous depth compose differently |
| Maximum u64 rank and zero-cost algebra operands | No numeric-depth unrolling or wraparound in the bound evaluator |

Normative syntax/proof cases belong in conformance when RES-1 is specified;
source-summary and target-inventory obligations belong in compiler tests;
complete result checks belong in `tests/programs`. Extract useful fixtures
there rather than making the gate depend on this investigation. No research
model becomes a second production checker or a corpus-selected proof path.

A complete resource-qualified deployment is a later, explicitly stronger
completion point: same-object target evidence, complete native closure,
mapped recursive depth, fixed-stack adapter, all reserved memory regions,
and budget comparison. Its negative controls must include an unaccounted
callee, dynamic frame, unproved new cycle, insufficient budget and changed
image. Passing the source milestone alone must not print that guarantee.

Budget-directed target cases must also distinguish the following outcomes.
The arithmetic examples are test criteria, not additional native measurements:

| Case | Required observation |
|---|---|
| One unavoidable 8 KiB frame, S = 4 KiB | Cannot qualify even without recursion |
| 33 activations at 32 B each plus 96 B complete path overhead, S = 4 KiB | The 1,152 B bound fits; depth alone imposes no rejection |
| Same input/depth proof, changed machine frame sizes | Recompute bytes for the new image; do not reuse the previous budget result |
| All else fixed, S equal to the proved byte bound or one byte below it | The bound certifies the former but does not certify the latter |
| Proven tail transfer with many source calls | Account for reused machine storage; do not charge retained frames that do not exist |

## Alternatives and selection grounds

- A recursion ban reduces initial obligations but excludes the bounded
  recursive fixture even when its demand fits. Select scalar ranked recursion
  and counted loops; defer mutual components and richer measures until needed.
- Inferring rank expressions adds a new search/admission policy. Select written
  ranks and fixed proof queries; reuse the existing numeric-bound projection
  for precision. A writer can expose harder facts with current local proofs.
- A runtime counter or a larger stack gives an execution limit but does not
  prove normal completion. Neither supplies missing source evidence.
- A second source interpreter or resource-only parser duplicates semantics.
  Select a consumer of the existing checked model, with graph/cost work outside
  the program-point proof flow.
- Certifying arbitrary optimized binaries immediately needs more origin and
  native coverage than the current ledger contains. Begin with source evidence
  and explicit target qualification boundaries; do not present that deferral
  as a solved correspondence problem.
- A stable serialized certificate, general abstract interpreter and full WCET
  system would outgrow the first consumer. Use private typed evidence and one
  image-bound developer report; revisit only for an independent consumer.

The word `decreases` labels strict descent, as in
[Dafny's termination clauses](https://dafny.org/dafny/DafnyRef/DafnyRef#sec-loop-termination),
but the proposed surface has one explicit integer rank and an erased named
snapshot: no inferred tuple, collection order, wildcard exemption or imported
solver policy. [LLVM's function attributes](https://llvm.org/docs/LangRef.html#function-attributes)
define `noinline` as an inliner restriction, not a cost theorem; that is why
target correspondence remains an explicit compiler responsibility.

This document owns the proposed implementation contract until the selected
rules and compiler decisions supersede it. Replace its proposals with links
to their owners as they land; retain only useful design evidence and rejected
alternatives. Deferred precision and target capabilities are tracked in the
maintained TODO, not inferred as current language rejections.
