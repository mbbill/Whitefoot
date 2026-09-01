# Current Plan: source-carried proof in the compiler

Status: IMPLEMENTATION COMPLETE; v0.40 is activated on `codex/source-proof`.

Active language authority: v0.40, SHA-256
`5079ef2efa7862184f06ccf7dc273ae97eda791679a44f66c86e75afbc46c6e0`.
The conditional merge-time activation record becomes effective when the owner
approves the exact revision containing it for merge into `main`.

This branch implements source proof checking in the ordinary compiler path.
Whitefoot source is the proof-bearing input; parser output, checked syntax,
facts, obligations, and lowering input are ordinary compiler data.

## Objective

Whitefoot is a proof-carrying systems language. The only author-controlled
semantic input is Whitefoot source code. That source contains the program,
contracts, loop invariants, and any explicit proof steps needed when the fixed
automatic rules are insufficient.

The compiler must reject every supported partial operation that it cannot prove
safe before lowering. An accepted program may still implement the wrong
algorithm, but it must not execute memory corruption, a data race, an
uninitialized read, silent overflow, an out-of-bounds access, another undefined
operation, or a writer-reachable language trap.

The same checked facts serve four jobs:

1. admit partial operations;
2. prove `ensures` and publish facts to callers;
3. authorize optimizer transformations and removal of redundant checks; and
4. prove `par` ownership, effects, independence, indexed disjointness, map,
   reduction, and bounded queue/completion conditions.

Proof syntax is erased before lowering. It has no runtime representation and
adds no runtime branch, lock, scheduling edge, metadata lookup, or fallback.

The cost is source-writing difficulty. A valid program may need contracts,
loop invariants, and explicit finite proof steps before the compiler can admit
it. Whitefoot accepts that cost because AI is the intended writer; the compiler
still checks every step, and the human still approves the requirements and
resulting code.

## Acceptance boundary

The normal compiler path is:

```text
.wf source
  -> lexer and parser
  -> resolved, typed program plus typed source proof syntax
  -> semantic fact flow
  -> direct checking of source proof steps and compiler obligations
  -> checked program
  -> proof erasure
  -> ordinary IR, LLVM, and executable
```

The source is the sole author-supplied proof input. If internal compiler
structures disagree, that is a compiler defect to fix in code and tests, not a
new proof obligation or a runtime check.

## Deterministic proof boundary

The official compiler uses no SMT solver for acceptance. Automatic reasoning is
admitted only when all of the following are true:

- the specification fixes the algorithm and its iteration order;
- the result is unique for the given input facts;
- termination does not depend on a timeout;
- worst-case work and memory follow from checked input size; and
- the implementation remains fast on maintained real programs.

The implemented automatic domains are ownership, borrows, effects,
initialization, integer type ranges, ground relations, difference bounds, the
current affine rules, and specification-fixed dataflow. Known bits, modular
congruence, and richer typestate remain candidate finite domains until an
active rule and a real program justify each one. Algorithms such as graph
closure or a finite fixed point are allowed only when their universe, order,
result, and work bound are fixed.

Harder proofs must not expand compiler search. The implemented loop surface has
the author state the invariant explicitly; the compiler checks it but does not
guess one. Implemented `prove`/`use` steps name already available premises and
one fixed coefficient-one combination. No proof form may ask the compiler to guess a
coefficient, induction hypothesis, branch split, intermediate term, rewrite
direction, or path.

## Implemented first vertical slice

The first end-to-end program now passes source checking, proof erasure, LLVM
lowering, linking, and execution:

```whitefoot
fn weigh['w](weights: &'w buffer<u8>, count: own u64) -> total: own u32 reads(weights) contract {
  define capacity = len(deref(weights));
  requires ile(count, capacity);
  requires ile(count, 1000_u64);
  ensures ile(total, 255000_u32);
} {
  let sum = 0_u32;
  for i in 0_u64..count {
    invariant per_byte: ile(sum, 255_u32 * i);
    let w = deref(weights)[i];
    let wide = cvt<u8, u32>(w);
    set sum = sum + wide;
  }
  return sum;
}
```

The checker handles this source in program order and proves seven separate
things:

1. on the first loop header, `sum <= 255*i`;
2. on an arbitrary reachable header, `i < len(weights)` before indexing;
3. `sum + wide` is in the `u32` domain before the addition;
4. the hidden `i + 1` update is in the `u64` domain;
5. after an arbitrary iteration and the hidden update, the next header again
   satisfies `sum <= 255*i`;
6. normal exhaustion exports `sum <= 255*count` to the continuation; and
7. `count <= 1000` then proves every selected return's written
   `ensures sum <= 255000`.

The word "arbitrary" in step 5 is essential. This is the induction step for
any reachable iteration, not a simulation of the second iteration.

## Implemented

1. `invariant name: ile(left, right);` is part of the ordinary grammar,
   resolver, checked model, diagnostics, and proof-erasing lowering path.
2. The fixed affine checker proves a simultaneous invariant prefix at the first
   header and across every reachable backedge, including the hidden unit binder
   update. At an ordinary control-flow join, incoming affine values with the
   same canonical coefficient vector and different constants are represented
   by that common nonconstant form plus one compiler-owned delta whose interval
   is the exact minimum and maximum incoming constant. This fixed linear
   transfer preserves bounded conditional steps such as `old` versus
   `old + 1` without path enumeration, and every goal consumer uses it through
   the same proof context. It performs no coefficient, invariant, or path
   search.
3. Exact normal exhaustion substitutes the captured upper endpoint into proved
   invariants. A reaching `break` removes facts not shared by every continuation.
4. The loop guard and contract facts discharge `weigh` indexing. Its invariant
   discharges accumulator overflow and, after exact exhaustion, the written
   `ensures`; the counted-loop rules discharge hidden binder overflow. The
   backend test compiles, links, and runs `weigh`, with no invariant runtime
   operation or overflow fallback.
5. Multiple invariants are checked as one source-ordered induction batch. When
   proving a later invariant's backedge, the checker may use an earlier proved
   invariant from the same header prefix.
6. Numeric and logical consumers use one current `ProofContext` and one
   normalized `prove` entry. Fixed ordinary closure and affine reasoning remain
   separate deterministic routes behind that entry; diagnostics are produced
   with the originating decision.
7. `prove name: ile(...) { use ile(...); }` is implemented in grammar,
   resolution, semantic checking, diagnostics, proof flow, and erasure. Each
   premise is checked in the same pre-proof context and the target must equal
   the exact coefficient-one sum.
8. `requires` is discharged before call transfer, and `ensures` is proved at
   selected returns and published atomically across call-graph components.
   Supported callers consume the resulting summaries through the same proof
   context.
9. The v0.40 grammar, semantic compiler, checked model, lowering, backend,
   conformance surface, and ordinary tests no longer contain the released
   writer assertion or its runtime path. Proof failure now has one disposition:
   compile-time rejection.
10. Selected-target layout, allocation-ceiling, and address-domain checks run
    before emission. Parallel permission consumes already-checked operation,
    ownership, effect, and loop facts; proof-only statements add no runtime or
    scheduler edge.
11. One source-derived fixed two-slot bounded-batch completion path is
    implemented for the narrow direct staged counted-loop shape. On native
    POSIX completion targets its runtime window is bounded to `1..2`; qualified
    non-completion targets retain the same generated CFG with a deterministic
    window of one and direct calls. Every issued slot is drained in source
    order before slot zero can be reused. Dynamic per-iteration paths, an odd
    final batch, ordinary result/error arms, and native link/run behavior have
    executable evidence. A function containing two staged loops deliberately
    leaves both on the ordinary path rather than partially transforming one.
12. Counted-loop map permission consumes the exact value image retained beside
    an already-discharged OP-4 result. It admits a direct-owned, write-only
    single-binder affine map `a*i+b` with `a != 0`, requires every write to one
    root to use the same map, and keeps borrowed roots, reads, `replace`, and
    unresolved effects sequential. Copies and checked affine transforms of the
    binder therefore work without parser-shape recognition or a second bounds
    proof.

## Measured compiler cost and runtime behavior

These are bounded development measurements from 2026-09-01 on the local
10-core Apple M4 host running macOS 26.5.2, using the release compiler. They
are evidence for this implementation, not portable performance guarantees.

- Three complete compilations of `tests/programs/wfgrep.wf` took
  0.694-0.750 seconds through LLVM emission and 1.159-1.571 seconds through
  native linking. The same real program currently retains 23,394 diagnostic
  derivation nodes and 45,669 parent edges. Machine-independent tests cap those
  sizes at 25,000 and 50,000, so a return to the former million-node cost shape
  fails deterministically instead of depending on a wall-clock timeout.
- The maintained parallel benchmark generator produced `bal_d12_w192` with
  700 repetitions. Nine fixed-order interleaved runs per cell gave minimum
  execution times of 0.5373 seconds for the sequential build, 0.5381 seconds
  for the `--par` build at one worker, 0.2849 seconds at two workers, and
  0.1615 seconds at four workers. That is 1.00x one-worker cost and measured
  speedups of 1.89x and 3.33x at two and four workers for this admitted shape.
  All 36 runs exited successfully and published identical result bytes. This
  demonstrates that the checked ownership, effect, and independence facts can
  authorize useful parallel code without adding proof work at runtime; it does
  not claim that every permitted workload is profitable.

## Future extensions after v0.40

The v0.40 source-proof implementation and activation evidence are complete.
The items below are possible later projects selected by real-program pressure;
they are not missing work for v0.40 activation.

1. Generalize beyond the fixed same-coefficient join interval across the
   remaining coefficient-changing branch/join, nested-loop, correlated-value,
   and multi-exit shapes without recognizing a function name or source pattern.
2. Exercise the implemented `prove`/`use` surface through the shared goal entry
   for postconditions, calls, allocation/address checks, target-domain checks,
   and `par`. Add another source proof rule only when a selected real program
   cannot express its proof with the fixed coefficient-one form; do not replace
   missing expressivity with compiler search.
3. Extend static discharge to newly implemented active-spec operations as each
   one enters the compiler. Arithmetic, conversion, subscript, initialization,
   projection, allocation, layout, address, target, system-range, and
   queue/completion families retain the same prove-or-reject boundary; an
   unsupported family is not silently treated as valid or invalid source.
4. Complete proof-driven `par` beyond the current sibling-call, write-only
   single-binder affine map, enumerated reduction, and narrow staged-I/O forms.
   The fixed two-slot
   bounded batch now supplies one complete multi-operation driver; widen it
   only with equally explicit ownership, publication, queue, drain, and
   selected-target evidence for additional control-flow shapes, operation
   families, and more than one staged loop in a function.
### Retirement evidence for the old runtime-assertion surface

The active v0.40 branch has removed `claim ... because`, `deny_claims`, the `traps` effect
category, and the CLM/PRV/TRAP rules that existed only to constrain or execute
that surface. Their former conformance cases must be retired rather than
rewritten into vacuous passing programs: once those constructs are no longer
in the language, a test of their old acceptance, rejection, locality,
provenance, or runtime behavior is not evidence for any surviving rule.

The conformance audit identifies 52 such single-purpose cases. Before their
retirement, every surviving rule referenced by those mixed rows was counted
against the remaining corpus; no surviving rule loses all coverage. The
narrowest affected counts remain nonzero (`SYS-2` changes from 19 cases to 3,
and `FN-6` from 8 to 3). Cases that still test a live effect, arithmetic,
ownership, function, or system rule are migrated to ordinary facts,
contracts, invariants, or explicit finite source proofs instead of being
deleted. The active specification, manifest, runner verdict domain, compiler
tests, and runtime path are updated in the same branch so no removed test is
being discarded merely to make a failing check green.

The replacement inventory is concrete rather than inferred from the counts.
Every surviving rule that the retired rows also named has independent live
positive or negative evidence outside that retirement set:

- effects: `EFF-1` by `eff1-neg-wrong-order-row` and
  `x-eff-pure-combined-with-allocation`; `EFF-2` by
  `eff2-neg-undeclared-exhibited`, `eff2-neg-declared-unexhibited`, and the two
  allocation-call rows;
- finite entailment: `ENT-3` by `op4-pos-index-discharged` and
  `ent3-pos-s1-branch-fact`; `ENT-4` by
  `ent4-pos-transitivity-discharges` and
  `ent4-neg-nonstrict-bound-underivable`; `ENT-5` by
  `ent5-pos-element-write-preserves-length` and
  `ent5-neg-kill-on-write`; `ENT-6` by `op9-pos-buffer-new` and
  `op4-neg-index-undischarged`;
- functions and control: `FN-1` by `fn1-pos-signature-driven-call` and
  `own1-neg-bare-affine-call`; `FN-2` by
  `ent1-pos-instantiation-judged-at-value` and
  `ent1-neg-instantiation-judged-at-value`; `FN-6` by
  `fn6-pos-recursion` and `fn6-neg-polymorphic-recursion`; `FN-8` by
  `fn8-pos-requires-run` and `fn8-neg-entry-contract`; `FN-9` by
  `fn9-pos-plain-direct-result` and `fn9-neg-unproved-selected-return`;
  `ERR-3` by `err3-pos-propagate` and `err3-neg-error-type-mismatch`; and
  `GIVE-1` by `run-ex1-value-match` and
  `x-match-give1-wrong-type`;
- partial operations and mutation: `OP-2` by
  `run-invariant-exact-sum`, `op2-pos-div-checked`, and
  `op2-neg-div-wrap`; `OP-4` by `op4-pos-index-discharged` and
  `op4-neg-index-undischarged`; `OP-9` by `op9-pos-buffer-new` and
  `ent3-pos-s6-allocation-length-fact`; `SET-1` by
  `ent3-pos-s5-set-commit-image` and `ent5-neg-kill-on-write`;
- syntax and system contracts: `GRAM-4` by `gram4-pos-stmts` and the mutable
  buffer program rows; `SYS-2` by `accept-sysentry-command-all-inputs` and
  `reject-sys2-args-count-missing-region`; and `SYS-11` by
  `accept-sysrelease-return-unit-declared` and
  `reject-syseff-return-unit-pure`.

The behavior unique to the 52 rows is therefore exactly the removed runtime
assertion, strict-closure, or assertion-provenance surface. The rows above,
plus the new invariant and source-proof cases, remain wired through the same
manifest and runner after that surface is retired.

The retirement boundary for each old row is recorded below. “Removed with the
surface” means the row tested no surviving source form or rule; the named live
rows are the independent evidence for any behavior that still exists.

| Retired row | Exact after-boundary evidence |
| --- | --- |
| `clm1-trap-false-claim-aborts` | Runtime assertion failure is removed. `prf1-neg-unproved-premise` proves that an unproved written premise is rejected and creates no executable fallback. |
| `clm1-pos-passing-claim-establishes-fact` | `prf1-pos-explicit-affine-proof`, `run-invariant-exact-sum`, and `op4-pos-index-discharged` cover the three surviving fact constructors. |
| `clm1-trap-false-claim-not-refutable` | Removed with the runtime-assertion surface; `prf1-neg-unproved-premise` owns the surviving compile-time rejection. |
| `clm1-neg-repeated-claim-name` | Removed with `claim`; proof and invariant name rules are covered by their own PRF-1/INV-1 source cases rather than retaining a CLM verdict. |
| `clm2-pos-redundant-claim-advisory` | Removed with the assertion lifecycle. The compiler may retain an already proved fact, while every `prove` premise is independently checked by `prf1-pos-explicit-affine-proof`. |
| `clm2-neg-refuted-claim` | Removed with the assertion lifecycle. A false or absent proof premise is a PRF-1 rejection in `prf1-neg-unproved-premise`. |
| `scope4-pos-claim-traps` | Runtime proof failure is no longer a source outcome. `op4-pos-index-discharged` covers static admission and the proof-erasure tests cover absence of a runtime branch. |
| `clm1-neg-user-result-claim-locality` | `fn9-pos-plain-direct-result`, `fn9-neg-unproved-selected-return`, and the FN-8 caller cases cover verified cross-function facts. |
| `clm1-neg-system-result-claim-locality` | `op4-neg-external-index-without-fact` and `op4-pos-external-index-after-branch` cover the exact no-fact/real-branch boundary. |
| `accept-clm1-local-claim-after-boundary-exit` | `op4-pos-external-index-after-branch` and the SYS-11 release cases cover the surviving selected-edge and early-return behavior. |
| `accept-clm1-local-claim-after-boundary-join` | Ordinary join behavior remains in ENT-5 cases; no assertion-locality verdict survives. |
| `accept-clm1-local-claim-inside-selected-arm` | `ent3-pos-s1-branch-fact` covers an ordinary selected-arm fact without assertion authority. |
| `reject-clm1-claim-on-selected-payload` | `fn9-neg-unproved-selected-return` and the external-value negative cases require a verified summary or a real fact before use. |
| `reject-clm1-claim-on-delivered-selection` | `x-match-give1-wrong-type` and the external-value negative cases independently cover GIVE-1 and missing proof. |
| `reject-clm1-claim-on-storage-written-under-selection` | `ent5-neg-kill-on-write` covers the reaching-definition kill; `op4-neg-external-index-without-fact` covers the remaining undischarged access. |
| `reject-clm1-claim-on-loop-carried-update` | `run-invariant-exact-sum` and the INV-1 negative cases cover loop-carried facts by induction instead of assertion locality. |
| `eff1-pos-pure-and-traps-rows` | The `traps` row is removed. `eff1-neg-wrong-order-row` and `x-eff-pure-combined-with-allocation` cover the surviving EFF-1 rows. |
| `eff2-pos-declared-traps-exhibited` | The `traps` exhibition is removed. `eff2-neg-undeclared-exhibited`, `eff2-neg-declared-unexhibited`, and the allocation-call rows cover EFF-2. |
| `eff4-pos-trap-aborts` | Writer proof abort is removed. PRF-1/OP-2 negative cases cover static rejection; proof-erasure evidence covers the absent runtime edge. |
| `eff2-neg-propagate-hidden-trap` | `err3-pos-propagate`, `err3-neg-error-type-mismatch`, and the surviving EFF-2 allocation rows cover propagation and effect closure. |
| `eff2-pos-propagate-declared-trap` | Same surviving ERR-3 and EFF-2 rows; there is no proof-only runtime effect to declare. |
| `fn8-neg-requires-missing-traps` | `fn8-pos-requires-run`, `fn8-neg-external-actual-without-fact`, and the EFF-2 rows separate static requirement proof from runtime effects. |
| `fn8-neg-strict-outside-caller-unproved-requirement` | `fn8-neg-external-actual-without-fact` covers the undischarged caller goal; the deleted strict-closure marker has no surviving behavior. |
| `prv3-pos-external-branch` | Replaced exactly by `op4-pos-external-index-after-branch`. |
| `prv3-neg-external-claim-conjunction` | Replaced by `op4-neg-external-index-without-fact` and `prf1-neg-unproved-premise`: neither a conclusion nor an unproved premise creates a fact. |
| `prv3-neg-external-claim` | Replaced exactly by the OP-4 and FN-8 external-value negative rows. |
| `prv3-pos-internal-claim` | `run-invariant-exact-sum` and `prf1-pos-explicit-affine-proof` cover internally proved relations. |
| `prv3-pos-external-bound-only` | `op4-pos-external-index-after-branch` covers the real bound while keeping unrelated values ordinary. |
| `prv2-pos-allocation-equality-call` | `op9-pos-buffer-new`, `ent3-pos-s6-allocation-length-fact`, and `fn8-pos-requires-run` cover allocation length and caller substitution. |
| `prv2-neg-nonexact-goal` | The surviving FN-8 negative rows require the exact instantiated goal; no provenance event remains. |
| `prv2-neg-direct-system-result` | Replaced exactly by `fn8-neg-external-actual-without-fact`. |
| `prv2-neg-entry-system-result-bridge` | `fn8-neg-external-actual-without-fact` proves that a helper cannot bootstrap its caller requirement. |
| `prv2-neg-two-hop-bridge` | The same FN-8 negative plus ordinary transitive call tests cover the two-hop case without a separate demand graph. |
| `prv2-neg-recursive-demand` | `fn6-pos-recursion`, `fn6-neg-polymorphic-recursion`, and failed recursive FN-9 publication tests cover recursion without assertion demand. |
| `prv2-neg-mutual-demand` | The mutual-recursion and failed-component publication tests cover the surviving SCC behavior. |
| `prv2-pos-seedless-mutual` | The seedless mutual postcondition test covers atomic publication without a provenance component. |
| `prv1-pos-payload-sibling-isolation` | Enum payload and borrow tests retain projection isolation; protected use still requires its own OP-4 fact. |
| `prv3-neg-read-offset-taint` | Each subscript now submits its own OP-4 goal; the OP-4 external negative/positive pair covers the exact boundary. |
| `prv1-pos-control-write-address-nontaint` | `ent3-pos-s5-set-commit-image`, `ent5-neg-kill-on-write`, and the OP-4 branch case cover write facts and later access independently. |
| `prv2-neg-complete-only-postcondition` | `fn9-neg-unproved-selected-return` and `fn8-neg-external-actual-without-fact` require an originating verified summary. |
| `prv2-pos-postcondition-b-summary` | `fn9-pos-plain-direct-result` and `fn8-pos-external-actual-after-branch` cover verified summary publication and caller use. |
| `clm3-pos-transitive-value-branch` | Generic, GIVE-1, FN-8, FN-9, and mutual-SCC behavior remains under the independent live rows listed above; `deny_claims` is removed. |
| `clm3-neg-direct-unreachable-claim` | Removed with `claim` and `deny_claims`; no strict-closure event survives. |
| `clm3-neg-generic-first-import` | Generic instantiation tests cover FN-2; assertion import order is removed. |
| `clm3-pos-upward-near-miss` | Ordinary generic calls and OP-4 facts remain independently covered; upward assertion closure is removed. |
| `clm3-neg-mutual-scc-import` | Mutual-recursion and atomic postcondition-publication tests cover the surviving SCC semantics. |
| `clm3-neg-static-conjunction-unproved` | `prf1-pos-explicit-affine-proof` is the explicit composition repair; FN-8 negative rows retain exact-goal rejection. |
| `clm3-neg-body-check-bounds` | OP-4 and INV-1/PRF-1 rows cover the body obligation directly. |
| `clm3-neg-body-check-requires` | FN-8 and PRF-1 rows cover the call requirement directly. |
| `clm3-neg-transitive-check-summary` | FN-9 publication and OP-4 rows cover the callee summary and downstream obligation directly. |
| `x-arith-idiv-trap-signed-two-variable-traps` | `op2-pos-div-checked`, `op2-neg-div-wrap`, and the explicit nonzero-requirement case cover exact division without runtime proof failure. |
| `eff2-neg-division-class-site-pure-row` | OP-2 owns the static division obligation; the surviving EFF-2 rows cover actual runtime effects. |

The semantic unit-test retirement follows the same boundary. Five historical
modules were audited test by test rather than merely unwired:

- `strict.rs` tested only the deleted CLM-3 marker and the Complete/U/B
  counterfactual partition;
- `claim_locality.rs` and `claim_residuals.rs` tested the deleted
  `ClaimAuthorityAnalysis`, residual-claim lifecycle, and generic-claim schema;
- `provenance.rs` tested the deleted PRV value-taint gate and its diagnostic
  carrier graph; and
- `check_dissolution.rs` combined one live contract test with one deleted S3
  claim test.

The live behavior embedded in those modules remains under its owning compiler
rule. Exact and weak verified postconditions and static contract clauses moved
to `postconditions.rs`; whole-box replacement moved to `boxes.rs`; concrete
generic nominal rebuilding, partial generic descendant discovery, and
source-order diagnostics moved to `generics.rs`. Existing originating-context
tests retain the other independent properties: `postconditions.rs` checks
entry images, writes, holder consumption, call substitution, atomic recursive
publication, failed-component withholding, and seedless cycles;
`system_effects.rs` checks state-formal write substitution; and `borrows.rs`
checks projected ownership paths and write invalidation. None of these tests
reads a Complete/U/B view, a claim ledger, a strict partition, or a PRV carrier
graph to decide acceptance.

The unified entry chooses a fixed proof route and returns its derivation in the
same call. That derivation explains the decision for diagnostics and grants no
independent proof authority. A consumer defines the exact proposition it needs
but does not choose between the ordinary and affine engines.

Runtime-origin data receives no special assertion authority. A parameter,
system result, loaded byte, or other run-time value may appear symbolically in
the proof context, but it establishes a proposition only through an ordinary
typed operation, a selected control-flow edge, a proved callee `ensures`, a
proved loop invariant, or a source `prove` step whose named premises and fixed
rule the checker verifies. Source proof syntax cannot inject a proposition,
mark a run-time value trusted, or turn a failed proof into an executable
fallback. This direct fact-admission invariant replaces the old assertion
provenance partition: without a writer assertion, there is no separate
assertion-laundering channel to classify.

The existing checker already combines one affine invariant with ordinary range
facts. In `weigh`, `sum <= 255 * count` and `count <= 1000` prove
`sum <= 255000`. The current automatic boundary is instead visible when two
independent affine premises must be selected together:

```text
x <= y
z <= w
------
x + z <= y + w
```

A later explicit proof step may name those two premises and coefficient one for
each. The checker verifies that written linear combination in source order; it
does not search for the premise set or coefficients.

## Deferred resource-availability boundary

For v0.40, heap OOM, stack exhaustion,
operating-system quotas, and runtime-start resource availability are outside
the work scope. This does not defer layout or address proofs, target
qualification, target-domain membership, parallel proof, or bounded
queue/completion protocols. Existing termination paths for the scoped-out cases
are documented gaps, not the final Whitefoot safety guarantee.

## Evidence rules

Every proof feature ships with a positive source program, a nearby source case
whose stated premises are insufficient, a deterministic work bound, and tests
for proof erasure. Optimizer facts additionally require facts-off correctness.
Parallel facts additionally require sequential/parallel result equivalence and
evidence that more than one worker is actually granted when the proof permits
it.

No test, conformance verdict, source rule, partial operation, or diagnostic is
deleted or weakened merely to make a gate pass. A deliberately retired test
must state which retired compiler architecture it exercised and why no live
language behavior was removed.
