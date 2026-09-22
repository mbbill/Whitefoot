# Result proof transport across source forms

This investigation asks why a verified result relation survives a direct call
match but not naming that result or forwarding its success with `propagate`.
The active specification remains authoritative. This study changes no language
rule, conformance expectation or compiler implementation, and selects no live
design-tree revision.

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

## Current boundary and its history

The measured baseline is compiler revision
`7127bcb6f48a0664d31a856ef54e21010bb2c238`, with specification v0.63.
The original discussion used v0.62; v0.63 widens owned-descendant measure
placement and retains the result-route restrictions examined here. The sources
of authority are [FN-9, CALL-4](../../../spec/kernel-spec.md#8-functions-generics-contracts),
[CALL-6, ENT-3.S12, ENT-5](../../../spec/kernel-spec.md#15-obligation-discharge-deterministic-facts-invariants-and-local-certificates-normative)
and [GIVE-1](../../../spec/kernel-spec.md#3-grammar).

There are three separate losses:

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

The engineering benefit of the present restriction is identifiable: it avoids
representing conditional evidence that must survive value transfer and later
mutation. The inspected current decisions and historical specification do not
establish that this is necessary for deterministic proof checking, or that its
composition cost is preferable today. That is a design question, not a
compiler defect under the current rules.

## Reproduction and observed results

[probe.rs](probe.rs) is the sole source generator and baseline runner. Its caller
is the explicit command below; it is research, not a daily gate or conformance
adapter. It writes all generated WF programs, LLVM output, diagnostics and its
CSV to scratch. Remove or replace it when these discriminating observations
are superseded; an implemented language change must add its maintained cases
to the formal test system without making that system import this probe.

Run from the repository root, with task-specific scratch paths:

```sh
wf_result_scratch=$(mktemp -d)
make -C compiler build CARGO_TARGET_DIR="$wf_result_scratch/target"
rustc --edition=2024 -O research/investigations/result-proof-transport/probe.rs -o "$wf_result_scratch/probe"
perl .github/run-check.pl result-proof-probe "$wf_result_scratch/probe" "$wf_result_scratch/target/gate/whitefootc" "$wf_result_scratch/results"
cat "$wf_result_scratch/results/results.csv"
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

| Probe | Current result | Observation |
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
required challenges for a future candidate, not evidence that such a candidate
already handles those transitions correctly. The affine probe adds a `nocopy`
error declaration to exercise a legal move; it is not a Copy Result with an
illegal `move` disguising the publication failure.

## Implementation correspondence

The inspected [flow checker](../../../compiler/src/semantic/entailment/flow.rs)
matches the specification:

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
- [The return checker](../../../compiler/src/semantic/check/ensures.rs) reports
  `InvalidPostconditionReturn` for a routed return without the required
  constructor shape.

The [existing negative conformance case](../../../tests/conformance/cases/fn9-neg-named-outcome-no-publication.wf)
deliberately requires this loss. Repairing it requires a specification amendment
and corresponding conformance updates, not silently making that case pass.

## Alternatives

| Direction | Benefit | Remaining cost or uncertainty |
|---|---|---|
| Keep the closed routes | No additional retained state; known implementation. | The measured naming, extraction and wrapper failures remain. |
| Add only direct `propagate` publication | Reuse the selected-success event of the direct call; small semantic extension. | Named outcomes, copies/moves, value-match delivery and forwarded returns still fail. It adds another permitted shape rather than closing the composition question. |
| Retain finite evidence associated with the returned value | One account of capture, transport and success selection can serve named outcomes, propagation and wrappers. | Requires precise support, replacement, join, loop and return judgments, with a bounded term universe and measured checking cost. |
| General refinement or dependent result types | Could express richer payload, storage and cross-boundary relationships. | Changes the type/interface and proof language well beyond this defect; current examples do not select that scope. |

The recommended next experiment is the third direction, confined initially to
the existing single-Result integer-payload relation language. This is a
recommendation for a prototype and rule comparison, not a selected language
amendment or a claim that the following sketch is implementation-complete.
The direct-`propagate` alternative remains useful as a comparison baseline.
No new relation syntax, SMT, arbitrary predicate inference or runtime state is
needed to state the narrower question.

## Candidate: conditional evidence associated with a value

Conceptually retain `Ok(v) implies R(payload(v), captured operands)` for one
particular produced value `v`. This is compiler proof metadata, not a source
reference, a field added to Result, a new user-authored assertion or an
unconditional numeric fact. Selecting the success path makes the relation
available; merely possessing the Result does not prove that it is `Ok`.

The candidate needs these rules together:

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
   overlapping event, including scope exit. The first prototype need not invent
   a general scalar snapshot family. Consequently it may conservatively lose a
   relation after an argument changes even where richer snapshot reasoning
   could preserve a useful old-value relation. This limit must be explicit.
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
   build cross-products of guards over independent outcomes. A conservative
   exact normalized-template intersection is a candidate initial meet; it can
   lose relations that a more expensive consequence meet would retain.
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

For example, after `let saved = outcome; set outcome = replacement;`, saved
must retain only the evidence for its copied value. After changing a buffer's
length between a call and matching its Result, a bound about the old length
must never become a bound about the new length. A genuinely immutable captured
length can still denote the old length, but connecting it to current storage
requires a surviving relation. These are different obligations.

Aggregate fields, indexed storage, recursive payloads, multiple result
ordinals and cross-function body-private facts are outside this initial
prototype's added transport surface. Existing admitted direct routes remain
available. In particular, no callee-local fact crosses a function boundary
without its declared contract. A local-result prototype that succeeds does
not settle general stored refinement types or quantify over container slots.

The retained state should use a finite vocabulary of source sites, result
ordinals, payload placeholders and already-admitted operand terms. Copying
metadata can share immutable relations and derivation nodes. If H is the number
of tracked holders and Q the finite candidate-relation vocabulary, a holder
map with at most Q entries has O(HQ) association storage; this conditional bound
does not prove that a proposed construction of Q is polynomial in source size.
The next design must bound that construction, the join work and the added
ordinary closure cost. Never substitute a timeout, fuel cap or arbitrary path
depth for that argument. No candidate checking-cost measurement was made here.

## Next experiment and affected owners

Before selecting a language change, implement the compared mechanisms only as
research candidates with these held observations:

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

The main language owners are FN-9, CALL-4, CALL-6, ENT-3.S12/S13, ENT-5,
GIVE-1 and ERR-3. If new immutable scalar captures are selected, ENT-2/MSR-3
are affected too. The relevant tree owners are
[requires-entry-contract](../../../design/language/checks-and-proofs/requires-entry-contract.md),
[obligation-discharge](../../../design/language/checks-and-proofs/obligation-discharge.md),
[surface-form/match-form](../../../design/language/surface-form/match-form.md)
and [compiler/checker-facts](../../../design/compiler/checker-facts.md), with
their ancestors. A selected revision needs amendments for the new transport
and return decisions, the active-spec/version update, changed conformance
expectations and cases, and the ordinary checker/derivation changes together.
Ownership, reference access and existing contract vocabulary do not change
merely because evidence travels farther.

Design suitability: a unified local transport mechanism directly addresses the
observed composition failures and has a plausible erased representation.
General stored refinements would expand this study's scope, while a direct-
propagation-only repair leaves most failures intact. Prototype the local
mechanism first; retain invalidation, join, iteration identity, return selection
and vocabulary-size bounds as unresolved design work. No live-tree decision,
specification revision or implementation is approved by this research result.
