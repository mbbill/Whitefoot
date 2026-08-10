# Proof-certificate architecture decision packet

Status: **draft pending task 0048 terminal refresh**

Research task: `0049-proof-certificate-architecture`

Date: 2026-08-10

Priority order: architectural clarity, correctness, compile-time performance

## Decision

Do **not** replace the current entailment implementation with an untrusted
complete producer plus a fully independent exact verifier now.

Retain one canonical, deterministic completeness engine for the normative
least-closure result, accepted-program set, claim lifecycle, canonical
residuals, and first diagnostic. The smallest justified next experiment is to
make that engine record a canonical *used-premise witness* for every positive
result that matters: discharged bounds obligations, discharged call goals,
redundant claims, refuted predicates, and contradictions used by those
judgments. This first witness is internal observational metadata. It does not
initially become lowering authority and it does not cause a second closure
pass. Once shadow coverage proves that every checkless site is represented, the
same canonical engine should seal those site/witness pairs into a private
`EntailmentApprovedProgram` that lowering must consume. That intermediate gate
makes authority local and exhaustive, although it does not shrink the TCB: the
complete engine is still the issuer.

The recommended responsibility graph, even if no independent verifier is ever
promoted, is:

```text
checked semantic unit
    -> one canonical proof-flow normalization
    -> one deterministic complete entailment authority
    -> complete EntailmentResult
         -> outcome vector -> canonical acceptance and diagnostics
         -> used-premise witness -> audit and later provenance consumers
         -> accepted-site inventory -> seal all sites
              -> EntailmentApprovedProgram -> lowering
```

This is a target boundary, not the migration order: witness recording and site
inventory come first, and the shared normalization is extracted only after
those measurements show which information is actually needed.

A later split remains promising, but only in this narrower form:

```text
checked semantic unit
    -> trusted, syntax-directed ProofFlow extraction
    -> one deterministic complete entailment producer
         -> complete outcomes -> canonical acceptance and diagnostics
         -> sparse used-premise certificate
    -> small positive verifier
         -> lifetime-bound VerifiedCheckedProgram
    -> lowering
```

The positive verifier may eventually become the sole authority for omitting a
source bounds check and admitting a requirement-bearing ordinary call. It
cannot become the sole authority for exact source acceptance: an absent proof
does not prove non-derivability. The canonical engine therefore remains
responsible for completeness unless a future bounded experiment finds a
non-derivability certificate that covers the complete flow semantics without
recreating the analyzer.

This is a deliberate **defer** decision for the full producer/verifier split,
not an endorsement of the current tangle. It preserves the status-quo language
semantics while selecting a staged route by which a materially smaller verifier
can earn lowering authority. Research alone authorizes no implementation.

## Why this is the decision

Three findings decide the result.

First, the normative result contains negative information. A small checker can
validate a derivation of

```text
F in closure(State at p)
```

but an absent or invalid certificate does not establish

```text
F not in closure(State at p).
```

That missing half controls source behavior. An obligation either discharges or
rejects; a call goal is exactly `discharged`, `refuted`, or `unproved`; a claim
is exactly `redundant`, `refuted`, or `retained`; and refutation additionally
requires non-contradiction and absence of a positive derivation. A verifier
that validates all of those outcomes must validate the complete state after
every source, kill, join, and loop rule. That is a second entailment analyzer,
not a small proof kernel.

Second, most of the implementation complexity is not algebraic closure. In the
v0.26 candidate snapshot, the entailment and goal core is 4,922 lines; the
structured flow walk and source logic alone are 3,649 lines, while the local
`close` routine occupies roughly `state.rs:333-450`. Term and goal identity,
effect-projected kills, origin validity, join coverage, scope exits, and the two
loop rules remain safety-critical even if transitivity is checked from a short
path. Trusting producer-supplied versions of those facts would only rename the
TCB.

Third, duplicating closure is already measurably unacceptable. On the captured
candidate, release profiles of two real compilation units put 96.0% and 99.3%
of samples below `semantic::entailment::state::close`. Hashing ordered term
pairs accounts for most self time. The candidate also performs the ordinary
acceptance walk plus unasserted and S4-blinded counterfactual walks for
observational provenance. A verifier that recomputes state or closure would
duplicate the dominant compile-time cost. A sparse used-premise verifier can
avoid that cost, but it cannot remove the producer's existing cost by itself.

The result is a clean responsibility split rather than the misleading claim
that one small verifier can certify every property of compilation.

| Responsibility | Canonical completeness engine | Small positive verifier |
| --- | --- | --- |
| Exact normative least closure | Yes | No |
| Exact accepted-program set | Yes | No |
| `refuted` versus `unproved` | Yes | Can validate the negative premise, not the complete disposition |
| `redundant` versus `retained` | Yes | Can validate redundancy only |
| Canonical residual and first rejection | Yes | No |
| No invalid bounds-check omission | Producer proposes | Yes, for certified sites |
| No unjustified ordinary-call/S4 admission | Producer proposes | Yes, for certified sites |
| Binding proof to the exact lowering site | Outcome identity today | Yes, after a verified-program gate exists |

## Authority and scope

The owner separately approved this bounded research on 2026-08-10. It advances
the questions under Direction Outline items `PROOF-1`, `PROOF-8`, and
`VERIFY-3`, but it does not change their status or authorize implementation.

This packet may recommend internal compiler architecture only. It does not
authorize:

- a language rule, proof rule, fact source, or accepted-program change;
- source-level proof terms or external proof inputs;
- an SMT result as semantic authority;
- a serialized checked-program or certificate format;
- artifact replay, caching, portable identities, or a proof protocol;
- changes to claims, conformance verdicts, the active specification, the
  Current Plan, the Direction Outline, approvals, or MCTS-Mem.

The repository's live design memory fixes two important boundaries:

- check removal requires a deterministic machine proof, and unproved source
  obligations do not silently acquire implicit runtime checks
  (`mcts_mem/whitefoot/checks-and-proofs/obligation-discharge.md`);
- the private checked in-memory program is the sole lowering authority, with no
  current serialized or replayed authority
  (`mcts_mem/whitefoot/toolchain/semantic-artifact-authority.md`).

An earlier independent *production semantic verifier* was rejected because it
duplicated the complete language while still requiring producer-to-artifact
consistency. The possible future verifier in this packet is narrower: it sees
only one trusted normalized proof-flow input, checks only used positive proof
steps, and has no parsing, type, ownership, diagnostic, serialization, or full
closure responsibility. The prototype must demonstrate that this distinction
is real rather than verbal.

## Revision boundary

The research worktree was created from
`b11e22f1901dc9e59cac79a9250d709e4a2082a8` and received an exact file snapshot
of task 0048's uncommitted candidate. At the time of this draft:

- landed language authority is v0.25;
- the copied v0.26 content is candidate evidence, not landed authority;
- task 0048 owns all v0.26 specification, compiler, provenance, plan, outline,
  approval, and design-memory changes;
- task 0049 must refresh against task 0048's terminal revision before this
  packet becomes final.

Accordingly, every caller-side requirement, opaque-goal, entry-goal, and O3
provenance statement below describes the candidate until the terminal refresh
confirms it.

## Current pipelines

### Landed v0.25

```text
syntax, resolution, type, ownership, and effect checking
    -> executable callee-entry requires prologue
    -> one entailment walk per concrete function
    -> OP-4 / CLM-2 rejection when applicable
    -> private CheckedProgram
    -> lowering emits requires then body
    -> index IR with no source bounds branch
    -> backend inbounds address calculation and load/store
```

The ordinary call does not statically prove the callee requirement in v0.25;
the callee executes it. Accepted OP-4 subscripts nevertheless lower without a
runtime bounds branch.

### Candidate v0.26

```text
Phase A: complete concrete function inventory
    -> requires becomes GoalTemplate, not an ordinary callee prologue
    -> calls retain pre-transfer actual images
Phase B: install concrete call requirements
    -> combined L0 + signed-goal entailment
         -> OP-4 obligation outcomes
         -> FN-8 call-goal outcomes
         -> CLM-2 claim outcomes
    -> deterministic first rejection
    -> observational O3 provenance
         -> unasserted entailment rewalk
         -> S4-blinded entailment rewalk
         -> dependency and bridge fixed points
    -> private CheckedProgram
    -> ordinary lowering emits only the function body
    -> program entry evaluates its goal once
    -> index IR still has no source bounds branch
```

This is a meaningful semantic improvement over an executable prologue, but it
raises the importance of explicit proof authority: every ordinary call now
relies on static goal discharge, and each callee body begins with S4 as a
conditional axiom.

## Current TCB and the architectural problem

### Identity

The current safety result depends on all of the following identities being
right:

- concrete function instance;
- exact checked program point, including the phase before or after a transfer,
  set commit, claim establishment, scope exit, or join;
- obligation occurrence `(function instance, psuffix NodePath, 0)`;
- requirement occurrence `(function instance, final-check NodePath, 0)`;
- call occurrence `(caller instance, call NodePath)`;
- L0 terms: zero, mathematical constants, concrete const parameters, resolved
  places with ordered field/deref projections, lengths, and counted captures;
- exact typed goals, including operation row, type and const arguments, result
  type, argument order, and datum identity;
- named constants, which deliberately keep declaration identity inside opaque
  goals but fold to their mathematical value in L0;
- ephemeral actual values, scoped to caller instance, call occurrence,
  argument ordinal, type, and remaining projections.

`NodePath` alone is not a proof point. Actual-expression obligations precede a
call goal; the call goal precedes argument consume and callee writes; set target
and right-hand-side judgments precede commit; a claim is judged before its own
S3 fact; and scope kills precede joins. Any proof representation needs a
compiler-owned phase or event identity in addition to its source occurrence.

### Facts, flow, and closure

The candidate analyzer currently owns, in one broad implementation path:

- term and goal reconstruction and interning;
- S1-S11 source recognition;
- comparison, goal, and outcome origins;
- alias/holder support and effect-projected kills;
- scope exits and consume order;
- closure, contradiction, and subsumption;
- joins and reachability filtering;
- ordinary-loop continuing-kill summaries;
- counted preheader materialization and S11;
- bounds obligations, call goals, claims, and residual rendering.

This makes proof discovery, proof validity, diagnostic policy, and publication
authority difficult to distinguish during review.

Closure itself is an on-demand least fixed point over a `HashMap` of ordered
term pairs. Joins close every input and materialize common results. A crucial
semantic detail follows: a closure-derived fact is normally only a query view.
It cannot be carried past a later kill merely because the derived conclusion's
endpoints survive. Joins and the counted S11 preheader are the two deliberate
materialization boundaries. A proof format that treats every derived relation
as a live flow fact is unsound.

### Outcomes are not certificates

The candidate retains:

- a bounds `ObligationOutcome` with a boolean `discharged`, a contradiction
  flag, and an optional residual;
- call evidence categories such as `OpaquePositive` and
  `ExactL0Projection`;
- claim lifecycle outcomes;
- observational provenance witnesses.

It does not retain a derivation path for each accepted bounds site, and
lowering does not consume a proof or local authorization. The candidate
specification's DIAG-2 text requires the checked program to retain the exact
ENT-4 derivation, so this is also a candidate implementation-evidence gap that
task 0048's terminal review must settle.

### Actual no-check authority is global and implicit

Array, buffer, and slice index lowering assumes semantic success and emits a
plain index operation. The IR variants carry the value and a separate
target-domain representability obligation, but no source-discharge ID or proof
handle. The backend then emits direct `getelementptr inbounds` and load/store
operations without a source bounds branch.

The actual authorization today is therefore:

> Complete semantic checking published a `CheckedProgram`; any index node
> found inside it is assumed to have been discovered and discharged correctly.

That boundary is failure-atomic, but not local. A missing obligation-discovery
case, a stale `discharged = true`, or a mismatch between occurrence identity
and lowering can silently become a checkless address operation. This is the
strongest reason to investigate a future lifetime-bound verified-program gate.

### O3 is observational and should remain separate

Candidate O3 provenance recomputes entailment without S2/S3 and again without
S4, then solves dependency and bridge fixed points. It currently affects
neither acceptance nor lowering. It should not enter a positive safety verifier
until a later approved gate gives it a semantic consequence. Its current
rewalk cost is a performance concern, not a reason to enlarge the proof kernel.

## Three possible verifier envelopes

### Envelope A: algebra-only checker

The producer supplies a relation path or signed-goal witness. The checker
validates endpoints, weights, transitivity, and exact goal projection, while
trusting the producer's source availability, dominance, support, kills, joins,
and loops.

This can be small and linear in certificate size, but it cannot independently
authorize check omission. A forged source fact or omitted kill passes. Reject
this envelope as a safety boundary; it is suitable only for observational
explanations.

### Envelope B: trusted ProofFlow plus sparse positive verifier

One syntax-directed extractor converts the already checked semantic unit into
an internal, lifetime-bound `ProofFlow` containing:

- canonical point and edge identities;
- the exact term and goal universe;
- typed source events and their availability point;
- kill events, scope exits, and resolved overlap classes;
- exact join predecessor inventories;
- ordinary and counted loop structure;
- obligations, call goals, and claim queries.

The producer computes the exact normative result and emits only the used proof
paths. The verifier validates liveness, join coverage, loop carry, algebraic
steps, contradictions, and complete coverage of every accepted safety site. It
does not compute the full closure or choose diagnostics.

This is the only future split not ruled out by the current evidence. The
extractor remains in the safety TCB. It succeeds only if it is one canonical
normalization used by both producer and verifier and is substantially easier
to audit than the current interleaved analyzer. Producer-emitted, unchecked
events do not qualify.

### Envelope C: independent exact verifier

The verifier walks `CheckedProgram`, independently reconstructs identities,
sources, origins, support, kills, CFG reachability, joins, loops, closure,
contradiction, every negative outcome, and deterministic diagnostics.

This duplicates the analyzer and the measured hot path. Reject it now. It is
the same architectural failure that the older complete production-verifier
alternative warned about, even if the interchange object is in memory rather
than serialized.

## Recommended near-term witness

The first experiment should be specialized to the current fragment rather than
introducing a general theorem language.

### Contents

For each positive or refuting outcome, record a canonical root into an arena of
used premises:

- one source or implicit fact identity;
- one path of difference-bound edges for an L0 bound;
- the two paths for equality;
- a zero-bound path plus an exact disequality for strengthening;
- an exact signed opaque source, or an exact comparison-root projection;
- a negative self-bound or opposite signed-goal pair for contradiction;
- explicit predecessor coverage where a fact is materialized by a join;
- an explicit counted-preheader snapshot marker when S11 materializes closure;
- stable query identity for the obligation, call, claim, or contradiction.

The engine should record parent choices while it performs its existing work.
It must not rerun closure merely to reconstruct a proof. Proof nodes should
refer to compiler-owned dense identities in the same checked unit; no hash,
serialization, schema negotiation, or portable name is required.

### What the first witness does not do

- It does not shrink the TCB.
- It does not authorize lowering.
- It does not certify `unproved` or `retained` by absence.
- It does not replace the canonical result.
- It does not become optimizer evidence.
- It does not include O3's seedless fixed-point absence claims.

Its purpose is to establish whether every landed positive judgment has a
compact, stable, correctly invalidated explanation and to measure the real
cost. If that fails, the independent-verifier direction stops early.

## Worked certificate

Consider the proof obligation `i < len(values)` under established facts
`i < n` and `n = len(values)`.

Let the exact query point immediately before the subscript be `q`.

```text
ti = Place(binding_i, [], u64)
tn = Place(binding_n, [], u64)
tl = Length(Place(binding_values, []))

O  = (concrete function instance, subscript psuffix NodePath, conjunct 0)
goal(O) = ti - tl <= -1
```

The source facts are:

```text
F1: ti - tn <= -1
    source: S1 true edge of ilt(i, n)
    support: {i, n}

F2a: tn - tl <= 0
F2b: tl - tn <= 0
     source: S6 length equality
     support: {n, values root, holders used to reach values}
```

The difference graph orients `a - b <= c` as `b -> a` with weight `c`:

```text
E1: tn -> ti, weight -1, from F1
E2: tl -> tn, weight  0, from F2a

path: tl -> tn -> ti
sum:  0 + (-1) = -1
root O: ti - tl <= -1
```

A future Envelope-B verifier must check more than the arithmetic:

1. `ti`, `tn`, and `tl` are the exact normalized terms of this checked unit.
2. F1 is available only on the true edge of the exact condition.
3. F2a is available only after the exact S6 source.
4. Both premises are live at `q`.
5. Any intervening join covers every reaching noncontradictory predecessor.
6. No intervening write, projected callee write, consume, or scope exit kills
   either premise.
7. Edge endpoints compose and the mathematical sum proves the exact relation.
8. `O` names the exact checked subscript and no other occurrence.
9. Every checkless source subscript in the accepted unit has exactly one
   verified authorization before lowering begins.

### Required invalidations

`set n = ...` judges its target and right-hand-side obligations before commit.
After commit it kills both F1 and F2a; a later certificate cannot cite either.

A user call that writes through an actual projected from `&uniq n` judges its
own actual obligations and call goal before the call effects. On normal return,
the projected `writes` kill invalidates F1 and F2a.

Consuming `n` kills both facts. Consuming `values` kills the length premise.
Leaving the scope of `n`, `values`, or a holder read through by the length term
kills the corresponding premise before any join.

An element write `values[k] = ...` is the necessary counterexample: it does not
kill a fixed `len(values)` observation. A write replacing the whole root, or a
callee write whose projection can replace that root, does. The verifier must
use the same resolved-place overlap and fixed-length boundary as ENT-5, not a
string prefix or a blanket "any write" rule.

Finally, the derived result at `q` is query-local. It cannot itself flow past a
later kill. Only a join result or the counted S11 preheader snapshot may
materialize a closure consequence as a new live fact.

## Hard semantic cases

### Claims

A claim is judged before its own S3 establishment, so it cannot prove itself.
`redundant` requires a positive proof; `refuted` requires a negative proof plus
non-contradiction; `retained` is an absence result owned by the completeness
engine. Every accepted claim still executes in all build modes, even when a
positive witness proves it redundant. A proof is never an elision token for a
claim.

### Joins

A used fact at a join must account for the exact reaching predecessor set.
Contradictory predecessors impose no constraint. For bounds, individual
predecessors may prove stronger constants and the join keeps the weakest. Exact
disequalities and opaque signed facts must be present on every applicable
input; a goal derivable through an L0 projection is not interchangeable with a
live opaque fact. An empty join is contradictory under the current rules.

### Contradiction

Contradiction is either an L0 negative self-bound or both signs of one exact
goal. It makes every query derivable and prevents refutation. A certificate
must cite the contradiction and its liveness; producer assertion of an
`all_derivable` bit is not evidence. The unreachable/contradictory state is
absorbing across later kills, so the proof-flow representation must preserve
that rule deliberately.

### Ordinary loops

There is no induction. A loop head receives only pre-loop live facts whose
support survives every continuing kill. A future verifier must derive the
continuing set from the normalized control graph, not trust an unchecked
producer summary. Facts established in one iteration do not carry to the next.

### Counted loops

The order is semantically load-bearing:

1. capture both endpoints and establish S11 preheader facts;
2. close and deliberately materialize that preheader state;
3. subtract continuing kills;
4. establish the two true-header S11 bounds at body entry.

A certificate language without a distinct counted snapshot/materialization
step cannot represent already accepted cases faithfully.

### Calls, recursion, and entry

Each ordinary call proves one concrete instantiated goal before transfer and
callee effects. The callee body then checks under its own S4 axiom. Direct and
mutual recursion therefore use ordinary local contract reasoning at each edge;
they do not need a cyclic entailment proof summary. Every real dynamic entry
must still execute its required entry-goal evaluation. A verified-program gate
must cover ordinary call sites and separately confirm the retained entry path.

### Generics and constants

Certificates bind to concrete instances after type, const, and region
substitution. A raw const-parameter declaration identity cannot authorize an
executable judgment across instances. Goal identity retains named-constant
declarations, while L0 projection uses their mathematical values; conversion
between those domains is trusted normalization and must be tested explicitly.

### Ephemeral actuals

An actual value with no ordinary source datum is identified by its caller,
call occurrence, argument ordinal, type, and projections. It exists only for
that immediate call judgment and has no source place support. A certificate
must not equate it with a later repetition of the source expression.

### Facts on and off

Acceptance-bearing entailment and certificate generation run identically in
facts-on and facts-off compilation. No optional optimizer fact may repair a
missing source proof. Explicit checks and claims remain retained. Call/S4 proof
does not become `llvm.assume` or an alternate lowering path.

### Provenance fixed points

O3's dependency and bridge summaries are observational today. A positive path
can explain membership, but absence in a seedless recursive component requires
the complete fixed point. Keep this analysis out of the verifier unless a
future approved gate makes it acceptance- or lowering-bearing.

## Failure behavior

For an internal certificate produced by the compiler:

- an invalid positive certificate is a compiler-internal failure;
- a missing certificate for an outcome the producer marked positive is a
  compiler-internal failure;
- neither condition becomes a source rejection;
- neither condition silently changes `discharged` to `unproved`;
- neither condition inserts an implicit source runtime check;
- no partially verified checked program is published.

If the complete producer simply misses a real derivation and reports
`unproved`, a small positive verifier cannot detect the false rejection. That
is a completeness defect in the canonical engine. It remains subject to exact
conformance tests, differential migration tests, and the specification's
determinism requirement. Calling the producer "untrusted" without this
qualification would be misleading: it may leave the safety-elision TCB after
Envelope B, but it remains conformance-critical.

## Lowering gate if Envelope B succeeds

The future gate should be an unforgeable internal capability such as a
lifetime-bound `VerifiedCheckedProgram<'unit>`, not a serialized proof.

Before it can be constructed, the verifier must establish coverage of:

- every checkless source bounds occurrence;
- every ordinary call to a requirement-bearing callee;
- the global entry invariant that every actual path into a body using S4 comes
  from either a verified ordinary call or a retained dynamic-entry check;
- every required retained entry-goal evaluation.

Each lowering-visible index or required call carries a private dense occurrence
ID tied to the same checked unit. The verifier is the only module able to turn
that occurrence into a `VerifiedDischarge` or `VerifiedCallGoal`. Lowering has
no default branch for a missing token. The backend need not understand the
proof; it consumes IR that could only have been built from the verified unit.

This local binding addresses the current global-authority problem without
inventing portable identities or making proof data a backend protocol.

## Diagnostics and determinism

The canonical engine continues to choose outcomes and first rejection in its
specified stable occurrence/rule order. Proof choice does not select a
diagnostic. Residual text is rendered from the normalized checked obligation,
never copied from a certificate.

Canonical parent selection is still desirable for stable testing and future
`proof_ref` diagnostics, but proof byte identity is not a language property.
The producer may not use randomness, a timeout, allocation identity, unordered
iteration, or worker scheduling to change an outcome. Verification failure is
an internal failure and does not cause the engine to try a different
source-language disposition.

## Performance evidence

All measurements in this section are exploratory candidate-snapshot evidence.
They must be rerun on task 0048's landed terminal revision. They measure the
whole compiler unless a sample attribution is stated.

### Build

```text
CARGO_TARGET_DIR=/Users/bytedance/do_not_scan/proof-cert-root-target \
  cargo build --release --bin whitefootc --locked --offline
```

The first release build completed in 8.54 seconds. Build time is not used as an
entailment comparison.

### Whole-compiler timings

```text
whitefootc --emit-llvm -o /dev/null tests/programs/sha256_abc.wf
```

- 0.81 s real, 0.34 s user, approximately 6.5 MB maximum RSS.

```text
whitefootc --emit-llvm -o /dev/null tests/programs/utf8parse.wf
```

- 2.50 s real, 2.48 s user, approximately 8.6 MB maximum RSS.

```text
whitefootc --emit-llvm -o /dev/null \
  tests/programs/raw_deflate.wf \
  tests/programs/raw_deflate_dynamic.wf \
  tests/programs/raw_deflate_dynamic_decode.wf \
  tests/programs/raw_deflate_boundary.wf
```

- 1.43 s real, 1.42 s user, approximately 16.7 MB maximum RSS;
- five required redundant-claim advisories were emitted.

An independent repetition of the four-file unit observed 1.98 s and 1.47 s
release runs and approximately 16.9 MB maximum RSS. Debug runs were about
31.7 s, so architecture measurements must use release builds.

### Sampling profiles

The profile command shape was:

```text
samply-for-ai record --save-only --main-thread-only --rate 1000 \
  --output /Users/bytedance/do_not_scan/PROFILE.json.gz -- \
  /Users/bytedance/do_not_scan/proof-cert-root-target/release/whitefootc \
  --emit-llvm -o /dev/null INPUTS...
```

`utf8parse.wf`, 2,288 samples:

- `entailment::state::close`: 99.26% inclusive, 32.60% self;
- SipHash `Hasher::write`: 36.49% self;
- `RandomState::hash_one` for a `(TermId, TermId)` pair: 29.94% self;
- the two observational `rewalk_unasserted` executions account for 67.00% of
  total samples on their call path.

The four-file deflate unit, 1,401 samples:

- `entailment::state::close`: 96.00% inclusive, 28.62% self;
- SipHash `Hasher::write`: 37.33% self;
- ordered-term-pair `hash_one`: 30.41% self.

These profiles strongly identify repeated hash-based closure as the candidate
hot path. They do not measure certificate construction or verification, and
they do not prove that a particular replacement algorithm is faster.

### Cost implications

- A sparse used-premise witness can reuse closure predecessors and should not
  need a second closure. Its time and memory cost remain **not measured**.
- A positive verifier can be linear in the used proof and normalized-flow
  coverage it checks. Its actual cost remains **not measured**.
- A complete state certificate can approach `O(points * (terms^2 + goals))`
  memory before transition validation; it is disproportionate without contrary
  evidence.
- A query-local non-derivability certificate may use shortest-path dual
  potentials for one L0 state, but full flow completeness, opaque goals, joins,
  and loops remain unsolved. Its size and cost are **not measured**.
- Certificate architecture does not fix the producer hotspot. Closure algorithm
  work is a separate, measured performance project and should remain decoupled
  from the witness format.

## Alternatives under the same criteria

| Alternative | Architectural clarity | Soundness / correctness | Determinism and completeness | Compile/artifact cost | Decision |
| --- | --- | --- | --- | --- | --- |
| Unified engine as implemented | One result path, but normalization, flow, closure, diagnostics, and authority are tangled | Full TCB includes all discovery and lowering publication; current proof-to-site binding is implicit | One engine can implement exact negative outcomes | No certificate cost; current closure and rewalk cost is severe | Retain semantics, not structure |
| Unified engine plus used-premise witness | Makes positive derivations inspectable without another semantic path | Does not shrink TCB; catches no producer bug by itself | Exact behavior remains with the canonical engine | Likely small if parents are recorded in-line; must measure | Recommend as first experiment |
| Trusted ProofFlow plus positive verifier | Clean separation of normalization, completeness, validation, diagnostics, and lowering authority | Can remove closure/search from the safety TCB; extractor remains trusted | Engine still owns absence and exact accepted set | Avoids a second closure; proof and verifier cost unknown | Best future split candidate, conditional |
| Full proof-carrying checked IR / exact state verifier | Local authority is explicit | Sound only if complete state transitions and source coverage are checked | Can in principle certify exact outcomes, but recreates analyzer | Large state artifact and duplicate hot work | Reject now |
| Untrusted SMT producer | Search and validation can be separated in theory | SMT result alone is never authority; proof checker and exact encoding still required | Risks timeout/version-dependent false rejection and violates the current search-free architecture unless constrained to the exact calculus | External dependency and proof-format complexity with no measured need | Reject |
| Source/language proof objects | Makes writer evidence explicit | Could be sound with a language change | Changes accepted forms and author obligations | Large language, tooling, and review cost | Out of scope and reject for this problem |
| Missing-proof runtime fallback | Simple operational fallback | Safe only by retaining a check, but changes current source semantics and hides producer incompleteness | Changes accepted set, traps/effects, and diagnostics | Runtime and audit cost; repeats a rejected design | Reject |

### Status quo

The status quo wins on having one exact semantic path, but loses on reviewable
authority and current performance. Keeping it unchanged would leave both the
candidate DIAG-2 derivation gap and the global implicit no-check boundary.

### Derived witness only

This is the smallest change that yields new evidence. It can fulfill exact
derivation retention, improve explanations, and reveal whether certificates
stay compact. It must be described as observability, not verification.

### Producer plus verifier

This is selected only as Envelope B. "Untrusted producer" means the verifier
does not trust its positive safety conclusions; it does not mean the producer
may be incomplete while remaining conforming. Exact negative outcomes stay in
the canonical engine.

### Proof-carrying checked IR

Local verified tokens are a good *downstream gate* after Envelope B succeeds.
A full state at every point is not a good certificate. Do not conflate these
two meanings of proof-carrying IR.

### SMT

The current fragment is closed, deterministic, and search-free. An SMT engine
would still need to emit a proof in the exact internal calculus, while adding
encoding, process, resource, and determinism boundaries. There is no current
expressiveness or performance evidence that requires it.

### Language proofs

Writer-authored proof objects would change the language and its review model.
The present problem is compiler-internal confidence in already normative
derivations. Exposing proofs cannot repair an internal authority boundary.

### Runtime fallback

The source language deliberately replaced implicit hidden checks with explicit
claims or rejection. Falling back when the producer or verifier fails would
make a compiler defect change traps, effects, diagnostics, and acceptance. It
is not an implementation-only choice.

## Adversarial review matrix

| Attack | Required response |
| --- | --- |
| Producer invents a term or goal ID | Verifier resolves only IDs owned by the same lifetime-bound `ProofFlow`; out-of-range or mismatched IDs fail internally |
| Proof is rebound to another subscript | Root includes exact function instance and occurrence; verified coverage maps one root to that occurrence only |
| Source fact appears before execution | Point phase and source-event class reject it |
| Claim cites its own result | Claim disposition point precedes S3 establishment |
| Set commit is ignored | Carry across the post-commit edge fails on overlapping support |
| Callee write or consume is ignored | ProofFlow derives kill edges from checked modes/effects and actual projections; verifier checks carry against them |
| Scope exit is applied after join | Canonical graph places exit kills on predecessor edges before the join |
| Join omits one path | Join proof enumerates the extractor-owned predecessor inventory exactly once |
| Contradictory predecessor blocks a valid join | Verified contradiction marks that predecessor neutral under the normative rule |
| False contradiction authorizes everything | Explosion requires a checked negative self-bound or opposite exact signs at the same live point |
| Derived fact survives a normal kill | Only live facts carry; query-only closure results require rederivation after the kill |
| Counted preheader consequence disappears or survives incorrectly | Dedicated snapshot step precedes continuing-kill subtraction; no other materialization is admitted |
| Ordinary loop acquires induction | Loop carry accepts only surviving pre-loop facts; no back-edge establishment rule exists |
| Recursive proof becomes circular | Acceptance roots are local per concrete call edge; certificate IDs must be acyclic/topologically earlier |
| Generic proof crosses instances | Every identity is under one concrete instance after substitution |
| Named constants collapse in opaque goals | Goal datum retains declaration identity; only checked L0 projection folds value |
| Ephemeral actual is reused later | Its identity is scoped to one call occurrence and has no source-origin rule |
| Facts-off skips validation | Acceptance and certificate pipeline are identical in both modes |
| Backend receives an uncovered no-check operation | Future lowering input is constructible only after complete verified-site coverage |
| Extractor forgets a subscript or kill | This remains a safety-TCB defect; exhaustive checked-node construction, mutation tests, and independent behavioral cases are required |
| Certificate arithmetic overflows | Kernel uses checked exact arithmetic under an explicit representational bound; overflow is an internal resource/invariant failure, never saturation-based evidence |
| Invalid proof changes source diagnostic | Verification failure is internal and publishes no semantic result |

## Migration plan

Every step below requires separate owner approval and an independently claimed
task.

### Stage 0: terminal refresh

- Wait for task 0048 to become terminal.
- Refresh from its landed revision.
- Re-read the active specification, outline, plan, task record, relevant design
  memory, checked model, entailment, provenance, lowering, backend, and tests.
- Rerun the profiles and replace every candidate-only observation that changed.

### Stage 1: observational used-premise witness

- Keep the existing engine as the sole acceptance authority.
- Record canonical parents during existing closure and flow work.
- Retain roots for every positive/refuting outcome and contradiction used.
- Do not alter lowering authority or runtime behavior.
- Measure witness nodes, bytes, peak memory, compile-time delta, and proof depth
  on protected programs.
- Mutation-test identities, invalidations, joins, contradiction, loop ordering,
  substitutions, and proof-root coverage.

If a landed case cannot be represented compactly and exactly, stop the split
and retain the unified engine with better internal factoring only.

### Stage 2: one canonical ProofFlow

- Extract one structured, private proof-flow input from the checked semantic
  unit.
- Make program points, sources, kills, joins, loops, and queries explicit.
- Keep diagnostic text and policy outside it.
- Make both canonical engine and any shadow verifier consume the same input.
- Do not create a second source/AST semantic walk under a different name.

The extractor is accepted only if its trusted responsibility is syntax-directed
and reviewably smaller than the current flow analyzer. In particular, the
verifier must contain no parser, type checker, source goal builder, full closure
algorithm, or diagnostic selector.

After the query inventory and witness coverage have run in shadow mode without
drift, the canonical engine may seal each exact bounds/call occurrence and
proof root into a private `EntailmentApprovedProgram`. Lowering then has no
default for an uncovered checkless site. This is an architectural binding gate,
not independent validation; it closes occurrence-omission and proof-rebinding
paths while leaving derivation soundness in the engine's TCB. A future positive
verifier can replace the issuer without changing the lowering interface.

### Stage 3: shadow positive verifier

- Verify only used positive paths and complete safety-site coverage.
- Run after the canonical engine in shadow mode.
- Treat every failure as internal.
- Differentially compare the complete outcome bundle, diagnostics, and emitted
  behavior with the pre-migration path.
- Measure verification time and proof memory separately from producer time.

### Stage 4: verified lowering authority

Only if Stage 3 remains materially smaller, clear, exact on every landed case,
and acceptably cheap:

- introduce the lifetime-bound verified-program gate;
- require local verified tokens for checkless bounds operations and required
  ordinary calls;
- retain the canonical engine for exact negative outcomes;
- retain dynamic entry checks and every explicit check/claim;
- remove transitional shadow-only paths once equivalence evidence is complete.

### Separate performance work

The current closure hotspot warrants its own measured task. Candidate
directions include deterministic dense storage, sparse shortest-path queries,
incremental closure, and shared multi-lane computation for full/unasserted/
blinded states. This packet selects none of them without a benchmark. The
certificate format should describe semantic paths and not freeze the producer's
algorithm.

## Promotion and stop gates

Do not promote Envelope B unless all of the following are true:

- every landed positive obligation, call, claim, contradiction, join, ordinary
  loop, counted loop, generic, recursion, and identity case is expressible;
- source availability, support, kills, predecessor coverage, and loop carry are
  checked rather than trusted from the producer;
- verifier failure cannot become a source rejection or runtime fallback;
- no acceptance, claim outcome, residual, first diagnostic, runtime check,
  entry behavior, or facts-on/off behavior changes;
- the verifier contains no full closure and no duplicate source semantic walk;
- proof size, peak memory, producer overhead, and verifier time are measured on
  real programs;
- the new trusted extractor plus verifier is materially easier to audit than
  the current combined path;
- no serialization, cache, replay, solver, portable identity, or generalized
  theorem framework appears without a real approved consumer.

Stop and keep the unified engine if any gate fails. That is a successful
research result, not a reason to weaken the gate.

## Owner decisions after final refresh

This research supports the following decisions once task 0048 is terminal:

1. Whether to authorize Stage 1, the observational used-premise witness and
   measurements.
2. Whether the candidate DIAG-2 exact-derivation gap requires a separate bounded
   conformance fix before other proof work.
3. Whether an explicit local lowering token is desired even if Envelope B is
   ultimately rejected; without an independent verifier it improves binding
   clarity but does not reduce the TCB.
4. Whether the measured closure/re-walk hotspot should precede certificate
   prototyping as the next compiler-performance experiment.

The recommended answer to (1) is yes, subject to the landed refresh. The full
producer/verifier split remains deferred until that evidence exists.

## External research context

External work is analogy, not Whitefoot authority.

- Necula's original proof-carrying-code architecture separates an untrusted
  producer from a comparatively simple proof validator. That supports the
  positive-certificate direction, but Whitefoot additionally has normative
  negative outcomes that PCC-style positive validation does not settle:
  <https://doi.org/10.1145/263699.263712>.
- Oracle-based checking shows that a compact producer hint can guide a trusted
  checker and exposes proof-size/check-time tradeoffs. This supports measuring
  used premises before inventing a full proof object:
  <https://people.eecs.berkeley.edu/~necula/Papers/oracle_popl01.pdf>.
- Alethe illustrates the substantial proof-format and independent-checker work
  needed for general SMT proofs. Whitefoot has no current reason to import that
  machinery for a closed difference-bound fragment:
  <https://www.verit-solver.org/papers/pxtp2021.pdf>.
- CompCert TCB analysis is a useful warning that a proof kernel does not remove
  trust from language modeling, input translation, or unverified integration
  boundaries. Whitefoot must inventory `ProofFlow` extraction and lowering
  binding explicitly rather than counting only verifier lines:
  <https://arxiv.org/abs/2201.10280>.

## Evidence inventory

Primary local evidence inspected for this draft includes:

- `spec/kernel-spec.md`, especially FN-8, DIAG-1/2, CLM-1/2, and ENT-1..6;
- `compiler/src/semantic/goal.rs`;
- `compiler/src/semantic/entailment.rs`;
- `compiler/src/semantic/entailment/flow.rs`;
- `compiler/src/semantic/entailment/flow/sources.rs`;
- `compiler/src/semantic/entailment/state.rs`;
- `compiler/src/semantic/entailment/term.rs`;
- `compiler/src/semantic/provenance.rs`;
- `compiler/src/semantic/check.rs` and requires/call/generic helpers;
- `compiler/src/semantic/model.rs`;
- lowering index, entry-goal, and storage builders;
- backend array, buffer, slice, system-entry, and target validation;
- focused entailment, requires, provenance, lowering, and backend tests;
- `research/investigations/obligation-discharge/ACCEPTANCE.md`;
- live proof, obligation-discharge, requires-entry, fact-channel, compiler,
  artifact-authority, and effect MCTS nodes and their real alternatives;
- the task 0048 and 0049 coordination records;
- two release sampling profiles and whole-compiler timing probes under
  `/Users/bytedance/do_not_scan`.

Three independent read-only investigations covered the current TCB and metadata
flow, minimal certificate models, and adversarial determinism/completeness/cost.
All three converged on the same boundary: a sparse positive verifier can be
small, but an exact independent acceptance verifier currently becomes a second
analyzer.

## Finalization checklist

Before replacing `draft` with `final`:

- [ ] task 0048 is terminal and its integration revision is recorded;
- [ ] this work is refreshed onto that exact revision;
- [ ] active/candidate terminology is corrected to the landed state;
- [ ] relevant code and specification line references are rechecked;
- [ ] performance probes are rerun on the landed bytes;
- [ ] the adversarial reviewer challenges the final recommendation;
- [ ] repository checks required by `docs/WORKFLOW.md` pass;
- [ ] task 0049 moves to `docs/done/` with terminal evidence.
