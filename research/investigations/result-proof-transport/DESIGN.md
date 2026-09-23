# Result proof transport across source forms

This investigation asks why a verified result relation survives a direct call
match but not naming that result or forwarding its success with `propagate`.
The active specification remains authoritative. The implemented v0.65 rules
carry conditional evidence with local integer-payload Results.

The consumer is ordinary library composition: perform one operation, keep its
outcome while doing unrelated work, then use the established success bound.
Keep this record with its reproducible examples while its comparison informs
that design question; supersede its guidance in place if the candidates change.

## Scope and comparison criteria

The requirements are the current single-owner and reference-validity rules,
verified callable contracts, erased evidence, deterministic terminating
checking without SMT or acceptance budgets, and no added runtime check, data
tag, allocation or scheduling edge. Existing restrictions on which relations
an `ensures` can express are a separate question.

Before running the baseline probes, use these discriminating criteria:

1. Compare a direct call match, an intervening `let`, an ordinary value copy or
   move, and direct/indirect `propagate` for the same verified `Ok` relation.
   Keep the producer, arguments and consuming obligation fixed. Record the
   actual rejection rule, not merely the exit status.
2. Separate publication at the caller from verification of a wrapper's own
   routed postcondition. A transport repair that still requires rebuilding
   the same `Ok` at every boundary must report that remaining limitation.
3. Overwriting a result, changing a supporting value or measure, merging
   unrelated outcomes and crossing loop iterations must not authorize stale
   or mismatched evidence. A candidate must specify capture, transport,
   activation, invalidation and joins together.
4. Compare a direct-`propagate` extension, a finite local result-evidence
   mechanism, and general refinements/dependent result types. State what each
   admits and leaves out; do not call a finite representation a practical
   complexity bound without accounting for joins and loops.

Baseline acceptance checks establish current behavior only. A proposed
extension needs its own implementation and checking-cost evidence before any
claim of performance or complete soundness.

## Baseline boundary and its history

The measured baseline is compiler revision
`7127bcb6f48a0664d31a856ef54e21010bb2c238`, with specification v0.63.
The original discussion used v0.62; v0.63 widens owned-descendant measure
placement and retains the result-route restrictions examined here. The sources
of authority are [FN-9, CALL-4](../../../spec/kernel-spec-v0.63.md#8-functions-generics-contracts),
[CALL-6, ENT-3.S12, ENT-5](../../../spec/kernel-spec-v0.63.md#15-obligation-discharge-deterministic-facts-invariants-and-local-certificates-normative)
and [GIVE-1](../../../spec/kernel-spec-v0.63.md#3-grammar).

The baseline has three separate losses:

1. **Capturing an outcome.** FN-9 limits an `Ok` relation to a call used
   directly as a match scrutinee. A named, copied, moved, stored or propagated
   whole outcome carries no pending summary. The call has a verified contract,
   but no persistent association connects that relation to this returned value.
2. **Delivering the payload.** GIVE-1/ENT-5 supply bounded numeric relation
   delivery for `value_if`, not `value_match`. Even a direct call match can
   prove something inside its arm and lose it when `give` delivers that scalar
   to the enclosing initializer's binding.
3. **Forwarding the outcome.** A function with a routed `ensures` must return
   a direct canonical `Ok` or `Err`; forwarding a call or a named Result is
   rejected by FN-9 before it can establish the same success contract. Fixing
   caller publication alone would leave this wrapper boundary unchanged.

Plain integer postconditions already survive an ordinary subsequent scalar
copy. This is not a general reset of the fact state after every statement.
Likewise, unrelated statements inside a direct match do not discard the
established payload relation.

The restrictions predate references-as-paths. The
[v0.28 specification](../../../spec/kernel-spec-v0.28.md) introduced verified
normal-return summaries, direct receiver routes, the absence of a pending
summary for a named Result, and direct-constructor return selection. Git commit
`1d0633e8` installed that specification. It also carried the historical
complete/unasserted/blinded proof views and provenance gates; those are not
current requirements. Later contract-surface changes replaced executable-looking
contract blocks, and x1 replaced ownership and reference machinery, while
these closed publication routes survived.

The engineering benefit of the baseline restriction is identifiable: it avoids
representing conditional evidence that must survive value transfer and later
mutation. The inspected current decisions and historical specification do not
establish that this is necessary for deterministic proof checking, or that its
composition cost is preferable today. That is a design question, not a
compiler defect under the baseline rules.

## Baseline reproduction and observed results

[probe.rs](probe.rs) is the sole source generator and baseline runner. Its caller
is the explicit command below; it is research, not a daily gate or conformance
adapter. It writes all generated WF programs, LLVM output, diagnostics and its
CSV to scratch. Remove or replace it when these discriminating observations
are superseded; an implemented language change must add its maintained cases
to the formal test system without making that system import this probe.

Run from this branch, which contains the current runner. Create a detached
checkout of baseline `7127bcb6` in scratch and build its compiler separately;
the baseline checkout does not contain the runner:

```sh
wf_result_scratch=$(mktemp -d)
git worktree add --detach "$wf_result_scratch/baseline" 7127bcb6f48a0664d31a856ef54e21010bb2c238
make -C "$wf_result_scratch/baseline/compiler" build CARGO_TARGET_DIR="$wf_result_scratch/target"
rustc --edition=2024 -O research/investigations/result-proof-transport/probe.rs -o "$wf_result_scratch/probe"
perl .github/run-check.pl result-proof-probe "$wf_result_scratch/probe" "$wf_result_scratch/target/gate/whitefootc" "$wf_result_scratch/results"
cat "$wf_result_scratch/results/results.csv"
git worktree remove "$wf_result_scratch/baseline"
```

The actual run on 2026-09-22 built the unchanged compiler with `make -C compiler
build CARGO_TARGET_DIR=<scratch-target>` (exit 0), compiled the generator with
the `rustc` command above (exit 0), and ran it through the guard (exit 0).
The compiler invocation for each generated input was `whitefootc --emit-llvm
-o <case.ll> <case.wf>`. Six inputs were accepted and thirteen rejected with
the expected primary rule. All nineteen expectations matched. LLVM emission
checks acceptance and lowering; these results do not measure runtime behavior
or the performance of a proposed replacement.

The first seventeen probes were the predeclared call/transfer/wrapper controls.
Reading GIVE-1 then motivated the two additional `value-match-delivery` and
`value-if-delivery-control` probes; these are follow-up exploration.

| Probe | Baseline result | Observation |
|---|---|---|
| `direct-match` | accept | A verified `payload == input` is available in the arm. |
| `named-match` | FN-8 | One intervening `let` loses that equality. |
| `copied-match` | FN-8 | Copying the named Result does not recover it. |
| `moved-affine-outcome` | FN-8 | Moving a Result with a `nocopy` error does not carry it. |
| `direct-propagate`, `named-propagate` | FN-8 | Neither propagation form supplies it. |
| `value-match-delivery` | FN-8 | Extracting a direct-match payload with `give` loses it. |
| `direct-unrelated-statement` | accept | A harmless local between arm entry and use retains it. |
| `plain-result-copy-control` | accept | Ordinary scalar-result publication and copying compose. |
| `value-if-delivery-control` | accept | Scalar delivery across an `if` preserves the relation. |
| `wrapper-direct-return`, `wrapper-named-return` | FN-9 | Direct forwarding cannot verify the wrapper's same routed contract. |
| `wrapper-rebuild` | accept | Matching and rebuilding both variants verifies it. |
| `index-direct-match` | accept | The producer's bound discharges an actual array subscript. |
| `index-named-match`, `index-direct-propagate` | OP-4 | The same bound is unavailable at the same subscript. |
| `direct-stale-scalar` | FN-8 | Changing the scalar supporting an established equality kills it. |
| `overwritten-outcome` | FN-8 | Replacing the outcome must not lend it the old result's equality. |
| `mixed-join` | FN-8 | One branch returning an unrelated value cannot borrow another branch's evidence. |

The last two negative controls are rejected today even without invalidation or
join handling for stored evidence: the initial evidence is absent. They are
required challenges for a candidate, not evidence that the baseline
handles those transitions correctly. The affine probe adds a `nocopy`
error declaration to exercise a legal move; it is not a Copy Result with an
illegal `move` disguising the publication failure.

## Baseline implementation correspondence

At the measured baseline revision, the flow checker implemented those rules:

- `PreparedCall` is transient evidence for an exact root call and explicitly
  does not enter the checked expression tree.
- `establish_direct_result` skips variant-routed clauses. An ordinary `let`
  holding the Result therefore has no deferred success relation.
- `walk_arm` calls `establish_direct_match` only with the direct call's
  `PreparedCall` and the selected arm.
- `PropagateLet` applies initializer effects, declares its binding and supplies
  a fresh unknown integer image. Its comment explicitly withholds a new fact.
- The give path and `establish_value_if_delivery_join` enable relation delivery
  only for `ValueInitializerKind::ValueIf`.
- [The baseline return checker](https://github.com/mbbill/Whitefoot/blob/7127bcb6f48a0664d31a856ef54e21010bb2c238/compiler/src/semantic/check/ensures.rs) reports
  `InvalidPostconditionReturn` for a routed return without the required
  constructor shape.

The [baseline negative conformance case](https://github.com/mbbill/Whitefoot/blob/7127bcb6f48a0664d31a856ef54e21010bb2c238/tests/conformance/cases/fn9-neg-named-outcome-no-publication.wf)
deliberately required this loss. The v0.65 specification amendment changes
that expectation explicitly; the unchanged WF source now lives in the
[positive transport case](../../../tests/conformance/cases/fn9-pos-named-outcome-publication.wf).

## Alternatives

| Direction | Benefit | Remaining cost or uncertainty |
|---|---|---|
| Keep the closed routes | No additional retained state; known implementation. | The measured naming, extraction and wrapper failures remain. |
| Add only direct `propagate` publication | Reuse the selected-success event of the direct call; small semantic extension. | Named outcomes, copies/moves, value-match delivery and forwarded returns still fail. It adds another permitted shape rather than closing the composition question. |
| Retain finite evidence associated with the returned value | One account of capture, transport and success selection can serve named outcomes, propagation and wrappers. | Requires precise support, replacement, join, loop and return judgments, with a bounded term universe and measured checking cost. |
| General refinement or dependent result types | Could express richer payload, storage and cross-boundary relationships. | Changes the type/interface and proof language well beyond this defect; current examples do not select that scope. |

The owner selected the third direction for implementation, confined to local
integer-payload Results and the existing declared relation language. The
comparison rejects a direct-propagate-only repair because it leaves naming,
copying and forwarding failures intact. It does not select general refinement
types or make that larger design impossible.

## Implemented conditional value evidence

Conceptually retain `Ok(v) implies R(payload(v), captured operands)` for one
particular produced value `v`. This is compiler proof metadata, not a source
reference, a field added to Result, a new user-authored assertion or an
unconditional numeric fact. Selecting the success path makes the relation
available; merely possessing the Result does not prove that it is `Ok`.

The implementation applies these rules together:

1. **Capture once, after a successful call judgment.** Obtain authority from
   the verified callable contract and its actual arguments at that call.
   Preserve parameter/result ordinals and route identity. FN-4's formal
   contract remains the authority for a function parameter; do not inspect a
   selected implementation to strengthen it. Failed calls produce no evidence.
2. **Establish at the call and restrict by the route.** Retain CALL-6's order:
   evaluate arguments, perform ordinary call transfers/effect kills, then
   establish the appropriate relations. A later `match` or successful
   `propagate` selects existing conditional evidence; it must not reinstantiate
   the contract against the arguments' later values. Dead support never revives.
3. **Associate evidence with the value actually transferred.** A local copy
   gives the destination its own association with the same value evidence;
   a move transfers that association under ordinary ownership. Overwriting the
   source after a copy invalidates only its old association, not the unchanged
   copy. Replacing a holder or mutating a selected payload retires that holder's
   affected evidence; an alias write uses the same resolved-place overlap
   machinery as other invalidation. Metadata never grants a storage access.
4. **Keep value identity distinct from external support.** Existing immutable
   call/placement datums stay immutable. A substituted live scalar or reference
   measure remains supported by its current place and is killed by the ordinary
   overlapping event, including scope exit. Existing immutable call datums
   retain old argument values. Closing before a kill preserves consequences
   whose own support survives; it never equates an old datum with a new value.
5. **Use one success-delivery judgment.** Direct match, named match, successful
   propagation and value-initializer delivery must map the same selected
   payload evidence to their receiving binding after ordinary kills. Carrying
   only the whole-Result association would not fix `value-match-delivery`.
   The error edge publishes no success relation and retains ordinary cleanup.
6. **Meet at joins, rather than enumerate branch histories.** After renaming
   each incoming result to the destination's payload placeholder, retain only
   relations established on every reaching alternative that can supply that
   success value. Retain the proof parent from each alternative. Do not union
   branch-specific claims, pair a relation from one outcome with another, or
   build cross-products of guards over independent outcomes. The selected
   representation reuses the ordinary weakest-bound join: payload < 8 and
   payload < 10 retain payload < 10. The initial exact-template-intersection
   sketch is rejected because that needless loss buys no simpler inference
   family once the existing fact-state machinery is reused.
7. **Keep loops finite without cross-iteration identity reuse.** Start with
   the existing conservative head discipline: evidence whose holder or support
   changes on a continuing backedge is not carried to the next head. A fresh
   call in the body can establish evidence for that iteration. The identity of
   a written call site is not evidence that different iterations returned the
   same value. Zero-trip and exit joins obey the same meet. This admits local
   body composition but makes no promise of inductive guarded Result facts.
8. **Verify forwarded returns with the same conditional relation.** A routed
   wrapper contract must be proved for the forwarded value's success payload
   from available evidence or ordinary facts. It cannot be accepted merely
   because both function signatures write the same clause. The rule must also
   replace direct-constructor selection and define the nonempty selected-return
   judgment for forwarded values. Same-SCC callee summaries remain unavailable;
   otherwise an unchecked recursive forwarding cycle could prove itself.
   Direct Err and values whose constructor-tag information is definitely Err
   supply no selected success exit, including through copies and joins. A
   merely contradictory numeric context does not change structural selection.
   Each clause reads only its own returned ordinal's conditional context.

For example, after `let saved = outcome; set outcome = replacement;`, saved
must retain only the evidence for its copied value. After changing a buffer's
length between a call and matching its Result, a bound about the old length
must never become a bound about the new length. A genuinely immutable captured
length can still denote the old length, but connecting it to current storage
requires a surviving relation. These are different obligations.

Aggregate fields, indexed storage, recursive payloads, multi-result call
destinations and cross-function body-private facts are outside this initial
prototype's added transport surface. Existing admitted direct routes remain
available. In particular, no callee-local fact crosses a function boundary
without its declared contract. A local-result prototype that succeeds does
not settle general stored refinement types or quantify over container slots.

The representation is a deterministic map from live local bindings to separate
`ResultEvidence` values. Its private integer parameter is a formal name scoped
to each context, not a globally shared value. Interning one parameter per
integer type adds at most eight terms; equal TermIds in different contexts
never justify combining their guards. Copies receive independent associations;
whole-holder writes remove only associations whose resolved storage overlaps.
The `definitely_err` bit records constructor information, not a numeric proof
search. It meets by conjunction at joins and is reset by unknown replacement.

For a concrete checked function, let H be its maximum live local Result count,
T its ordinary term inventory including call-substituted operands plus at most
eight parameters, E its finite number of walked events and incoming join edges,
and Q its instantiated clause occurrences. Terms remain bounded by the existing
source/path substitution inventory; copying a Result creates no new term.
Each conditional matrix has O(T^2) selected cells. Independently live proof
candidates are additional storage: they are not a constant-size second layer.
Each refresh imports at most two closed ordinary candidates per pair, and a
materialization or join reduces its output to the strongest full and ordinary
fallback. Between such reductions, at most O(E*T^2 + Q) candidates accumulate
per context. Thus O(H*(E*T^2 + Q)) conservatively bounds stored numeric cells
and candidate references per flow snapshot, excluding the shared proof DAG.

Transport performs O(E*H) invocations of the existing terminating closure/kill
machinery plus corresponding candidate projections and joins. Substitution
maps candidates individually; joins retain one joined parent per output and
layer, never products of incoming proof alternatives. It introduces no new
predicate, term per path, iteration unrolling or recursive summary search.
These bounds describe added storage and closure invocations relative to a
concrete function and the existing closure cost, not generic expansion, proof
DAG byte size or all compiler work. The existing deterministic ledger retains
and shares derivation parents. Measurements below account for whole-process
memory as well, rather than treating the matrix-cell bound as a practical
memory guarantee.

The old `PostconditionDirectMatch` and first-arm
`PostconditionSelectedReceiver` implementation and required roots are removed.
Every own selection uses the same context-to-binding substitution; subsequent
scalar assignments use ordinary S5 commit images. Tests retain exact callable
clauses, source call identities and numeric actuals. They retire unused special
assignment-root counts because those roots are no longer a language rule;
actual downstream requirements still test assignment correctness.

## Implementation criteria and affected owners

### Authorized implementation investigation

The owner selected the unified transport direction and authorized implementation
after clarifying that an initially conservative prototype does not select a
permanent language restriction. Ordinary local binding, copy, move, selection,
propagation, value delivery and verified forwarding are the delivery scope.

The implementation experiment uses one independent conditional numeric context
per live local Result. Its private typed payload parameter is interpreted only
inside that context; selecting success substitutes it into the receiving value,
and no conditional relation is published into ordinary flow before selection. Copies share the
value's evidence while holder replacement removes only the replaced association.
Each context uses the existing difference-bound closure and weakest-bound join,
including common weaker bounds such as `x < 8` and `x < 10` yielding `x < 10`.
It never combines the guards of distinct outcomes. Ordinary events and lexical
exits use the existing resolved-place support kills; continuing-backedge kills
also remove changed associations before entering an arbitrary loop iteration.

Before judging this candidate, require all ordinary transfer probes to recover,
common weaker bounds to survive joins, and all stale-support, replaced-value,
mixed-producer and cross-iteration controls to reject. Include a surviving-copy
control after overwriting its source, constructor delivery, an Err-only incoming
alternative, and a wrapper whose declared relation is stronger than its callee's.
Runtime validation must execute both success and error paths. Inspect emitted
code for the absence of runtime evidence and independently scale transfer depth,
live outcomes and joins when measuring checking time and peak memory.

Representation assessment: keep this mechanism in a private child of the
existing entailment flow module, which already owns events, joins and call
authority. Extending the large walker inline would obscure those responsibilities;
a second analysis pass or sibling solver would duplicate them. The child serves
this experiment and the maintained rule if selected; remove it if the candidate
is rejected. The existing fact matrix bounds one context by the finite term
inventory rather than by path histories. The implementation must account for
the extra contexts, term construction and closure calls before claiming a
practical polynomial overhead. General aggregate and indexed-storage transport
remains a separate research question, not an inferred impossibility.

The implementation is judged against these held observations:

- Recover the named, copied, moved, propagated and extracted success bounds
  and the forwarded wrapper contracts from this probe, with unchanged producer
  contracts and downstream operations. Preserve existing accepted forms.
- Reject stale scalar/measure dependencies, overwritten payloads, alias writes,
  mixed-outcome joins, evidence reused from a previous iteration, and recursive
  summaries that depend on themselves. Also retain the valid copied-outcome
  control after overwriting its source and the unrelated-write controls.
- Account for joins with separate producers that establish the same relation,
  versus producers establishing different relations. Conservative loss must be
  reported rather than hidden by weakening the consuming obligation.
- Compare compiler checking time and peak memory while independently scaling
  local transfer depth, live outcomes and branch joins; inspect the generated
  code for unchanged value transfers, branches and allocations. A candidate
  requiring extra runtime proof state fails the held requirement.

The affected language owners are FN-9, CALL-4, CALL-6, ENT-2, ENT-3.S12,
ENT-5, GIVE-1 and DIAG-2; ERR-3 retains its ordinary value/control semantics.
S13 captures, MSR-2 descriptor support, ownership, callable refinement and the
same-component summary schedule retain their existing rules. The outgoing
v0.64 bytes from main are archived unchanged; no grammar production or generated
syntax data changes. The approved decisions supplement
[automatic-facts](../../../design/language/checks-and-proofs/automatic-facts.md)
and [checker-facts](../../../design/compiler/checker-facts.md); they replace no
live-tree line. Their ancestors and the existing contract, obligation-discharge,
match and propagation decisions remain applicable.

META-5 delta: numbered rules +0/-0, tokens +0/-0, spellings +0/-0,
source-shape exceptions -3 (direct-match-only Result publication,
direct-constructor-only routed return, and value-if-only scalar delivery),
plus retirement of the redundant first-arm payload-assignment special route.
Selection ground: evidence-selected. The matched baseline programs expose
these losses; the common transfer rule recovers them while the stale-support,
guard-isolation, join and loop controls preserve their rejection. The bounded
term and context argument selects reuse of L0 closure over guard products.

## Compilation cost and erased execution

On 2026-09-22, compare baseline `7127bcb6` with candidate `da55db81`, using
Rust 1.98.1, the repository gate profile, macOS 26.6.2 and Apple M1 Pro with
32 GiB RAM. Both compilers are built before measurement. `probe --scale`
generates identical accepted inputs for three axes, each with 4, 8, 16 and 32
steps after one initial outcome: copying the last outcome, adding independent
call outcomes, and chaining value-if joins. Thus 32 additions retain 33 local
outcomes. Each input has three separate invocations, with no warmup exclusion.
The runner measures wall time around `whitefootc --emit-llvm`, and macOS
`/usr/bin/time -l` records peak RSS in bytes. This includes parsing, checking,
LLVM emission and startup; it excludes compiler construction, linking and
program execution. It is not an isolated proof-checker measurement. Baseline
runs precede candidate runs, so the small samples do not isolate every cause
of their timing difference.

[scaling.csv](scaling.csv) retains all 72 samples and their source sizes. It
serves this representation comparison and can be retired when that comparison
is superseded. Neither the data nor the research runner enters correctness CI.
Times below are medians; peak memory takes the maximum of three samples.

| Axis | Steps | Baseline ms | Candidate ms | Baseline MiB | Candidate MiB |
|---|---:|---:|---:|---:|---:|
| Copies | 4 | 19.278 | 24.706 | 10.03 | 10.22 |
| Copies | 8 | 19.501 | 20.250 | 10.05 | 10.48 |
| Copies | 16 | 19.700 | 23.143 | 10.28 | 11.36 |
| Copies | 32 | 20.330 | 23.897 | 10.47 | 12.86 |
| Outcome additions | 4 | 19.407 | 21.258 | 10.13 | 10.67 |
| Outcome additions | 8 | 19.622 | 27.183 | 10.08 | 12.58 |
| Outcome additions | 16 | 20.359 | 64.728 | 10.69 | 19.83 |
| Outcome additions | 32 | 22.988 | 585.006 | 11.66 | 56.42 |
| Joins | 4 | 20.798 | 22.210 | 10.17 | 10.77 |
| Joins | 8 | 20.348 | 28.773 | 10.36 | 13.53 |
| Joins | 16 | 21.686 | 78.119 | 11.00 | 33.62 |
| Joins | 32 | 25.652 | 720.838 | 13.30 | 213.56 |

Copying stays near startup cost. Independent calls add immutable call datums;
joins retain several conditional contexts. Both expose steep growth: at 32
steps, candidate compilation is about 25 times baseline for outcome additions
and 28 times for joins. This supports an executable local transport experiment,
not a claim of cheap general refinement checking. Matrix sharing/projection
was deferred pending a matched precision/cost experiment or a real-program
cost signal. The maintained-program comparison below supplies that signal and
reopens implementation cost. Changing the stored fact set still needs a
correspondence argument. No acceptance limit hides the measured cost.

Reproduce on macOS from this branch. Set `wf_result_compiler` to the prebuilt
baseline or candidate executable, and create `wf_result_scratch` as above.
Build the current runner even when measuring the older compiler:

```sh
rustc --edition=2024 -O research/investigations/result-proof-transport/probe.rs -o "$wf_result_scratch/probe"
perl .github/run-check.pl result-proof-scale "$wf_result_scratch/probe" \
  "$wf_result_compiler" "$wf_result_scratch/scaling" --scale
```

The scale inputs impose no requirement available only to the candidate, so
both compilers successfully compile the same source. The 35 correctness probes
separately require the recovered facts. All twelve matched LLVM files are
byte-for-byte identical, while the runtime transport case executes the newly
accepted success and error paths. This is evidence of erased transport on
these inputs; runtime representations and lowering were not changed.

### Maintained-program cost and optimization criterion

A second comparison on the same host uses the exact pre-merge parent
`e8e1c411` and the merged implementation `a1aa1aa7` (the same tree as merge
`8d6da723`). The six maintained workloads' source files are unchanged. Both
compilers use the gate profile; one warmup per compiler/workload precedes five
alternating paired samples. The timed operation remains the complete
`--emit-llvm` invocation, with no compiler build, native linking or execution.
[program-cost.csv](program-cost.csv) retains the samples; like the scale data,
it serves this representation comparison and can be retired when superseded.

| Workload | Before transport ms | With transport ms | Before MiB | With MiB |
|---|---:|---:|---:|---:|
| Dense container control | 33.407 | 33.626 | 15.95 | 15.53 |
| Grayscale conversion | 24.089 | 25.164 | 12.14 | 11.92 |
| Telemetry packet | 25.816 | 27.263 | 12.86 | 12.56 |
| Prefix expression | 23.593 | 23.953 | 11.25 | 11.34 |
| wfgrep | 824.066 | 847.021 | 249.52 | 252.27 |
| Raw DEFLATE vectors | 227.915 | 697.128 | 78.22 | 115.53 |

The DEFLATE ranges are 227.076-233.065 ms before and 691.773-697.451 ms after;
all six final LLVM pairs are byte-identical. This exposes a real-program
regression, rather than establishing a whole-suite slowdown or attributing
cost to a specific checker function. The DEFLATE workload has many direct
matches and separate checking functions; it is not the synthetic workload's
33 simultaneous local outcomes.

Before selecting an optimization, require unchanged acceptance and retained
proof validity, including ordinary fallback candidates, conditional guard
isolation, replacement, support kills, joins and loop boundaries. Compare
against the unoptimized compiler on identical maintained and scaled inputs,
with prebuilt binaries and alternating samples. The practical target is to
bring DEFLATE within 25% of the pre-transport cost and cut both 32-step outcome
and join costs by at least half, without a reproducible regression above 10%
on the other maintained workloads. These are experiment selection criteria,
not compiler acceptance limits. A missed criterion requires investigating the
remaining cost; it does not justify weakening proof rules. Profile first;
prefer preserving the existing closure's completed work over introducing
another solver or omitting facts. Any representation choice must state its
correspondence argument before it is selected by timing.

The paired comparison is available through the existing research runner:

```sh
rustc --edition=2024 -O research/investigations/result-proof-transport/probe.rs -o "$wf_result_scratch/probe"
perl .github/run-check.pl result-proof-cost "$wf_result_scratch/probe" \
  "$wf_result_before" "$wf_result_scratch/comparison" \
  --compare "$wf_result_after" "$PWD"
```

It emits every sample to `comparison.csv`, includes the six maintained
workloads and all twelve existing scale inputs, and compares the emitted LLVM
after every paired round. Compiler construction remains a separate command;
both paths must name prebuilt gate-profile executables.

The first attribution used macOS `sample` on the unoptimized `a1aa1aa7`
compiler during the DEFLATE invocation: 316 of 470 sampled driver stacks had
`close_with_row_pruning` at their leaf. Major callers included ordinary
pre-kill closure following Result selection. This single short sample locates
an investigation target, not a precise phase-time partition. Code inspection
then found that importing an equal or weaker bound marked a closed cell fresh
even when its selected numeric value did not change. Reimporting a closed
Result snapshot therefore scheduled ordinary matrix cells for closure again.

The isolated first experiment preserves the full layer's closure record when
a new candidate cannot improve its selected bound, and separately preserves
the ordinary layer's record when no non-call candidate improves that layer.
A new disequality matters only to a layer that did not already contain it.
Every distinct proof candidate is still stored: a weaker or equal candidate
can become necessary after another proof's support is killed. This is an
implementation correction under the existing incremental-closure decision;
it does not select a different witness or numeric fact family. The matched
timing comparison tests whether these unnecessary fresh marks explain the
observed regression before changing conditional-state representation.

That isolated change (`b9e11c81`, with test/research additions at `f2c22f52`)
reduces DEFLATE from 700.756 to 371.629 ms against current unoptimized main
`1b916975`; its five-sample ranges are 692.687-710.006 and 368.197-373.060 ms.
This supports the attribution but misses the practical target, so it does not
end the investigation.

The selected implementation preserves the closed numeric core when starting and
substituting a conditional context. A fresh context takes the already
materialized ordinary numeric snapshot, drops writer-origin and opaque-goal
metadata, and uses the existing term kill to remove private payload parameters.
Its numeric bound and disequality candidates, including ordinary fallbacks,
are exactly the previous filtered import. Removing metadata cannot remove a
numeric consequence because materialization precedes that projection; opaque
goals themselves have never been part of Result transport.

Substitution similarly starts with the materialized numeric snapshot, kills
the substituted source term, and reinstalls the mapped incident candidates
with their ResultTransport parents. Nonincident candidates are unchanged. This
produces exactly the old candidate substitution set while preserving the closed
core among unaffected terms. A destination already present may introduce new
paths; the existing insertion and kill records require those paths to close
normally. No claim that arbitrary substitution preserves a complete closure is
needed. Both layers retain their separate candidates and closure records.

It prepares one ordinary snapshot per predecessor at a join, kill or scope
exit, then imports that same snapshot into each independent Result.
Previously each import rematerialized an unchanged ordinary state with a new
snapshot event. The facts and support are identical at this shared flow point;
the materialized witness can be shared. No conditional context is shared
between distinct guards.

Refreshing also reuses an ordinary core covering more registered terms than
the conditional core: repeated calls add immutable datums after earlier
contexts were captured. It filters private payload parameters from the ordinary
snapshot, then imports every original conditional candidate. This is exactly
the previous candidate union, including all ordinary fallbacks. Core size only
chooses a reuse opportunity; all fresh and weakened cells still undergo the
existing closure. Candidate/provenance sets are preserved, while a different
valid equal-bound witness may win when import order reverses. Contradictory
contexts remain absorbing. The direct correspondence test compares both an
unseeded context and one with an older closed core, including S12 removal.

The narrower variant at `16fed4e5` reused the ordinary core only when the
conditional context had none. It reduced the 32-join case to about 264 ms and
106 MiB but left independent outcomes at about 374 ms: existing smaller cores
still repeated closure over later call datums. Reusing the larger core addresses
that measured case without adding a lifetime analysis or omitting any fact.

This private representation change stays inside FactState and the Result flow
child. The walker still owns event order, and no additional solver, acceptance
budget or lifetime analysis is introduced. The proposed supplement is kept in
`design/amendments/result-closure-reuse.md` pending the owner's ruling. Smaller
predicate sets and last-use tracking are declined for this change because they
need broader correspondence arguments; the measured question is whether reuse
of already completed numeric work suffices.

### Selected cost result

The final compiler code is `f16eea6b`, compared with unoptimized `1b916975` in
the same environment and gate profile as the maintained-program measurement.
[optimization-cost.csv](optimization-cost.csv) retains every warmup and measured
sample, including the isolated first experiment and the pre-feature comparison.
The table gives medians of five alternating pairs and maximum measured RSS;
the CSV retains each range. All eighteen workloads produce identical LLVM in
every paired round.

| Workload | Unoptimized ms | Optimized ms | Peak MiB before / after |
|---|---:|---:|---:|
| Dense container control | 33.812 | 34.370 | 16.62 / 16.78 |
| Grayscale | 24.751 | 24.500 | 11.95 / 12.45 |
| Telemetry | 28.538 | 27.336 | 12.59 / 12.58 |
| Prefix expression | 24.038 | 24.122 | 11.17 / 11.30 |
| Wfgrep | 839.589 | 833.489 | 247.44 / 244.06 |
| Raw DEFLATE vectors | 691.728 | 279.112 | 111.45 / 86.58 |
| Copies 4 | 19.929 | 19.967 | 10.12 / 10.22 |
| Copies 8 | 20.644 | 20.453 | 10.38 / 10.36 |
| Copies 16 | 21.507 | 21.606 | 11.14 / 11.05 |
| Copies 32 | 23.873 | 25.577 | 12.61 / 12.59 |
| Independent outcomes 4 | 21.353 | 20.391 | 10.53 / 10.53 |
| Independent outcomes 8 | 25.268 | 21.949 | 12.25 / 11.45 |
| Independent outcomes 16 | 61.832 | 28.451 | 19.41 / 17.08 |
| Independent outcomes 32 | 591.040 | 61.515 | 57.75 / 36.23 |
| Joins 4 | 21.827 | 21.469 | 10.73 / 10.75 |
| Joins 8 | 28.364 | 25.689 | 13.19 / 12.80 |
| Joins 16 | 79.097 | 50.528 | 33.59 / 24.14 |
| Joins 32 | 728.315 | 267.276 | 214.42 / 105.83 |

DEFLATE falls by 59.6%, independent outcomes by 89.6%, and joins by 63.3%.
The DEFLATE ranges are 687.905-698.689 and 276.630-280.973 ms; the final
median is 22.5% above the original pre-feature 227.915 ms, meeting the stated
25% target. Both large scale cases exceed the 50% reduction criterion. Other
maintained workloads show no regression above 10%; the copying scale control
at 32 steps increases by 1.704 ms (7.1%). These whole-CLI observations do not
establish zero transport overhead or a whole-suite speedup.

A fresh paired comparison against pre-feature `e8e1c411` confirms the target:
DEFLATE is 225.091 ms (222.413-228.038) before transport and 275.585 ms
(274.754-279.796) after this optimization, a 22.4% increase. The final code
still costs 62.147 ms versus 22.377 ms for 32 independent outcomes, and
265.133 ms versus 26.281 ms for 32 joins. The feature's proof work has not
become free, even though the selected reduction criteria are met.

The representation still stores a dense matrix per live conditional value,
and a join still processes every surviving context. The remaining scaling
opportunity and its reopening criterion are kept in TODO; broader storage or
last-use analysis is not necessary to meet this measured target.

## Candidate evidence and remaining validation

With the candidate compiler, the same current runner accepts `--candidate`
for the semantic comparison (omit it only when reproducing the old baseline):

```sh
perl .github/run-check.pl result-proof-probe "$wf_result_scratch/probe" \
  "$wf_result_compiler" "$wf_result_scratch/results" --candidate
```

The original unified path, after removing the old direct-match and selected-receiver
code, passed all 35 candidate probes: 25 accepts and 10 expected rejections.
Its 950 semantic unit tests passed, including independent retained-DAG
arithmetic/substitution checks and the maintained real-program proof inventory.
A first complete library run exposed eight expectations for the retired rules;
those expectations were revised against the amendment and the semantic suite
was rerun. Complete gate results and their tested revision are reported in
[PR #87](https://github.com/mbbill/Whitefoot/pull/87).

The cost revision repeats all 35 probes and adds implementation-specific
observations to the maintained tests: exact candidate-set agreement for
substitution into an existing destination and refresh from a larger core,
before and after S12 removal; equal-value witness replacement and fallback;
and eager complete-closure comparison at every proof point in the existing
Result value-transport and conditional-join cases. These protect candidate
provenance and closure records without duplicating normative case ownership.

The formal corpus adds a runtime value-transport case and a conditional-join
case, and negative cases for replacement, mutable support, guard isolation,
stronger join bounds, loop replacement, strengthened forwarding, an empty
success-exit set, cyclic forwarding and descriptor mutation. The two existing
named-outcome/value-match negatives become positives for the same source.
All thirteen added or changed expectations pass the ordinary compiler CLI.
The runtime case also links and exits zero after checking both success and
error, copy/move, value delivery, loop retention and forwarding paths.
No formal test imports the research probe.

The scalar `give` extension keeps the existing bare-atom carrier rule, while
Result-valued `give` carries its whole conditional context. General scalar
computed/projection delivery, borrowed Result selection, aggregate/indexed
storage and multiple Result destinations from one call remain outside this
amendment. They are recorded together with validation/reopening criteria in
[TODO](../../../docs/todo.md), rather than being treated as impossible or as a
reason to retain the direct-call restriction for ordinary locals.

Design suitability: a private child of the existing entailment flow owns
conditional evidence construction, selection and joins. The parent retains
ordinary event order and callable authority; no second solver, runtime state
or body-private interprocedural summary is introduced. Dense per-local contexts
reuse the ordinary numeric core under the measured criteria above; further
storage sharing remains a follow-up with explicit precision and cost criteria.
This evidence does not constitute a general
soundness certificate; the independent review and current validation status
belong to the PR.
