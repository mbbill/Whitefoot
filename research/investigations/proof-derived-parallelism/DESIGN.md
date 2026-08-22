# Proof-derived parallelism v1 — design contract (batch 0074)

Status: batch contract for branch `par/proof-derived-parallelism`. This
document is the executors' brief and the merge packet's design summary. It
synthesizes PAL.md (same directory), the three research rounds
(`do_not_scan/wf-parallelism-research/`), and the owner's rulings of
2026-08-20/21. Nothing here is approved until the branch merges.

> **Superseded in two places by later landings on this branch, and corrected in
> place on 2026-08-22 after the 0075/0076 batch audit.** The runtime protocol
> named in section 5 (`wf_par_try_fork` / `wf_par_join`) was replaced during
> batch 0075 by `wf__par_claim` / `wf__par_publish` / `wf__par_join` /
> `wf__par_release`, and the `WF_WORKERS` semantics changed during batch 0076:
> an unset variable now asks for one lane per logical CPU instead of meaning
> "no pool". Both corrections are made at the paragraphs that stated them. Read
> the rest as the design contract it is — the current behavior of anything it
> describes lives in `docs/ongoing/0075-par-optimization-digs.md` and
> `docs/ongoing/0076-night-par-ceiling.md`.

## 0. Charter

Owner direction (2026-08-21, verbatim, chartering this branch under the
merge-boundary process):

> 你研究完了就开始实现吧。像我之前让你改的流程一样,开一个worktree然后直接
> 开始修订和实现。不要block。明天我起来我要看到所有结果。再说一遍,你在自己
> 的worktree和branch上实现这个并行化能力,所以任何情况都不要block住。另外
> 记得用Opus5来写代码,只有最最复杂的事情才可以交给Fable。

Plus the two design rulings of the same night:

1. **Claim doctrine.** A claim is an always-true lemma bridging checker
   incompleteness; it is not an assert and cannot fail on an admissible
   input; a fully reviewed program cannot trap. Therefore traps under
   overlap are an *audit* problem, not a semantics problem: "如果程序的trap
   只可能由审计失败的程序产生……我们应该不需要考虑这种情况。"
2. **Frame.** Permission from proofs on ordinary code; actualization is the
   runtime's; resources (threads, cores) are never language concepts;
   markers, if any, never gate legality (PAL §3.4/§6).

## 1. Doctrine consequences (why v1 is small)

- **Eligibility = transitively claim-free.** An overlappable region whose
  transitive call closure reaches zero `claim` sites has zero trap sites
  (v0.33: claims are the only writer-reachable runtime checks, spec
  1875–1880). No trap sites ⇒ no trap-selection question ⇒ **no
  arbitration machinery, no parked lanes, no coordinator** in v1.
- **Divergence dissolves.** Eligible lanes carry no `external`, no
  `blocks`, no trap sites. The join waits for all its lanes. If any lane
  diverges, the overlapped execution hangs at the join exactly where the
  elision hangs; nothing observable can be emitted mid-lane. **No
  termination judgment, no weakened law, no EFF-4 ruling is needed for
  v1.** (The EFF-4 two-half ruling and elision-rank arbitration remain on
  file — `debate/d1-defense.md` — for the day claim-bearing regions are
  worth actualizing. Deferred, not rejected.)
- **Claim-bearing regions stay sequential and that is principled**: claims
  mark exactly the checker's incompleteness gaps; each checker improvement
  converts claims to proofs and mechanically widens the eligible set
  (ENT-1 version monotonicity). The ledger makes the denial visible, so
  the sequentialization is never hidden (PAR-4).

## 2. Scope

**In:** the permission judgment P (compiler-internal); the non-normative
permission ledger; runtime actualization of eligible sibling-call pairs
(fork/join, default-off via env); one minimal spec rule stating the law
(CANDIDATE v0.34); compiler tests incl. the named counterexample shapes; a
determinism repeat test (in-crate, non-protected); one real demo workload +
measured results; durable landing of the deciding research probes; batch
record 0074; PROPOSED current-plan revision + roadmap PAR update.

**Out (recorded, each with its trigger):** the `pal` marker (next packet —
grammar + FORM tables + teaching text deserve their own review; PAL §6 is
the agreed semantics: non-authoritative structural obligation); indexed-loop
permission (Tier A) and buffer-view splitting (Tier B) — future growth with
their recorded hazards (OWN-9 granularity, c2-F4); the I/O concurrency lane
(completion-based family — sequenced FIRST at the plan level, separate
packet; w3: 2.83x vs 0.15%); claim-bearing actualization (arbitration);
heartbeat profitability policy (v1 ships the simple policy below);
`reduce`-clause regrouping (never needed while joins are source-fixed).

## 3. The permission judgment P

For an ordered statement pair (s1, s2) in one block, both of the shape
`let x = f(args);` (calls of named functions; recursion included),
P(s1,s2) holds iff ALL of:

1. **No dataflow:** no argument of s2 uses a binding s1 defines (ordinary
   def-use).
2. **Disjoint footprints:** project each callee's written row onto its
   argument resolved places (the EFF-2 boundary projection the compiler
   already computes for ENT-5 kills); W(s) = written-through places plus
   consumed `own` argument places; R(s) = read-through places. Require
   W(s1) ∩ (W(s2) ∪ R(s2)) = ∅ and W(s2) ∩ R(s1) = ∅ under OWN-7's
   overlap relation, fail closed. (Affine amplifiers make this land: OWN-13
   sibling binders, STOR-5 no stored borrows, OWN-5 shared-read
   immutability.)
3. **Row gate:** neither callee's row carries `external` or `blocks`.
   Rows gate; places prove; no disjointness is ever derived from a row
   (EFF-5 respected).
4. **No skipping exit:** no exit edge of s1 bypasses s2 — s1's only normal
   continuation reaches s2. This condition is REQUIRED: without it a
   compiling, terminating program breaks the law
   (`debate/probes/g2_propagate.wf` — a `propagate` Err-edge exits the
   function while s2's write would still run under overlap). Early exit
   and divergence are the same property ("s1 may not reach s2's
   continuation"); condition 4 handles the exit half, the join's
   wait-for-all handles the divergence half.

**Eligibility (actualization) additionally requires:** the transitive call
closure of both callees reaches zero `claim` sites. Chains of ≥2 adjacent
eligible calls generalize pairwise.

**Invariants P must keep:** P consults typing, rows, resolved places, CFG,
and the call graph — never the entailment fact state. Acceptance is
untouched; facts-on/facts-off behavior is identical by construction.
Permission never licenses reordering of anything `external` (EFF-5
untouched).

## 4. Ledger (non-normative)

`whitefootc --par-ledger` prints one line per analyzed site on the
developer channel (spec 1969 permits exactly this; never on stderr's
mandatory record path):

```
PAR permitted   file.wf:LINE  pair(f, g)  eligible
PAR permitted   file.wf:LINE  pair(f, g)  not-actualizable: N claim sites via h
PAR denied      file.wf:LINE  pair(f, g)  condition 2: writes overlap at <place> vs <place>
PAR denied      file.wf:LINE  pair(f, g)  condition 4: Err edge of s1 skips s2
```

Deterministic order (source order). This is the legality ledger (round-1
salvage), the AI writer's gradient, and the measurement instrument.

## 5. Runtime and lowering

- **Default off, at compile time.** Actualization happens only when the
  compilation asked for it (`whitefootc --par`). **Corrected 2026-08-22:** this
  paragraph also required `WF_WORKERS` set to an integer ≥ 2, and batch 0076's
  L1 landing (`62e30831`) made an unset or empty setting ask for one lane per
  logical CPU instead. `WF_WORKERS=0`, `=1`, and any unparsable value keep the
  original meaning and start no pool, so opting out is still one setting away —
  but it is now an opt-out and not an opt-in. The compile-time half below is
  unchanged and is what keeps a build that did not ask byte-identical.
  Without `--par` the backend reads no permission group at
  all: no thunk is outlined, no module names a runtime symbol, and the
  emitted module is byte-identical to a compiler that has no overlap
  lowering. **Amended after the batch audit** (2026-08-21), which made
  `WF_WORKERS` the only switch and was falsified by measurement: the outlining
  alone, with no runtime linked and no worker requested, cost about 1.2x on the
  layout demo and 2.1x on `fib(38)`, so a byte-identical default has to be a
  compile-time choice rather than a runtime one. (Both of those numbers were
  themselves superseded by Dig 7 of batch 0075, which re-measured the outlining
  tax at 1.00x once the sequential clone landed; the conclusion — that the
  switch is a compile-time one — stands on the stronger ground that a build
  which did not ask now emits no overlap lowering at all.) `WF_WORKERS` remains
  the runtime knob for a build that asked, with the sense of its absent value
  changed as noted above.
- **Runtime:** one small C file (pthreads), linked only when the module
  contains at least one eligible site: lazy pool init on first fork
  (`WF_WORKERS` capped at a sane max), `wf_par_try_fork(fn, arg) ->
  handle|0` (forks only if an idle worker is available — the lane-budget
  policy; else returns 0 and the caller inlines), `wf_par_join(handle)`.
  **Corrected 2026-08-22:** that two-call protocol was replaced during batch
  0075 (`b251382f`, `826cea41`) by a four-call one — `wf__par_claim`,
  `wf__par_publish`, `wf__par_join`, `wf__par_release`, with
  `wf__par_pool_active` selecting the world at the bootstrap. The lane-budget
  intent survives in `wf__par_claim`, which still refuses rather than queueing;
  what changed is that the claim precedes the publish, so a refused hand-out
  builds nothing. `wf_par_try_fork` appears nowhere under `compiler/`.
  No global visible to WF source; the pool is TCB like malloc's internals.
  Policy rationale recorded: lane-budget is the only measured policy that
  is never catastrophic (worst 0.69x on tiny trees, wins on heavy bodies;
  `debate/d2-defense.md` §objection-3 table); static-weight constants are
  falsified (`a2-rebuttal`); heartbeat is the principled successor, out of
  scope v1.
- **Lowering:** for an eligible pair, the backend outlines s1's call into
  an internal thunk `void @wf_par_thunk_N(ptr)` over an args+result frame
  (IR-level function pointers are backend-internal; the language still has
  none), emits `try_fork` (today: `claim` then `publish`); s2 runs inline;
  `join` (or inline fallback call)
  completes before the first use of s1's result. Join precedes any exit.
  Worker stacks: request ≥ the main thread's stack size at pool creation
  (the 512KB-default hazard, g2 revive item 4).
- **No spec-acceptance interaction:** lowering choice is invisible;
  DIAG-2's one-lowering discipline is respected because the fork/inline
  fallback computes bit-identical results on the same single semantic path
  (the thunk calls the same monomorphized function).

## 6. Spec candidate (v0.34, CANDIDATE on this branch)

One new rule family, minimal text (~1 rule + law paragraph), drafted to the
spec's register, activation at merge:

> **[PAR-1]** A conforming implementation may overlap the execution of two
> statements only when the permission judgment of this rule holds: (their
> four conditions, stated normatively) and every reachable checked site of
> both call closures is empty of claims. Under such overlap every
> observable — result values, the trap-or-normal outcome, the [DIAG-3]
> record bytes, and the per-resource order of external effects — is
> identical in every execution to the sequential execution's. Worker
> count, schedule, and execution overlap are not observable, and no rule
> of this specification may consult them. Resource exhaustion remains
> governed by [SCOPE-3] and is not an observable of this rule. Every
> construct of this specification defines a total sequential order over
> its operand evaluations; this rule is a consumer of that order.

Include the protected-premise sentence (DIAG-3 records carry no worker or
call-stack identity — already non-normative at spec 1968 — and must not
grow one while this rule stands). CAP-1 stays a stub (v1 shares only
immutable borrows and moves nothing between threads that the thunk frame
does not own). Grammar unchanged ⇒ the two-path grammar verifier is run
and reports no delta; impact inventory via `whitefoot-spec --index`.
Conformance corpus delta: **zero cases** — the rule changes no acceptance
and no verdict; the record states this explicitly with the rationale.
Protected gates and wiring: untouched.

## 7. Tests

- **Unit (semantic):** P grants the compiled shapes (a2_bubble two-child
  uniq fold; A1-style read-only recursion; B1 bisection `reads`-only) and
  denies each counterexample by the right condition: g2_propagate (cond 4),
  g3_dep (cond 1), shared-`&uniq`-argument overlap (cond 2), external row
  (cond 3), claim-bearing closure (eligibility). Negative controls follow
  the m/n discipline: each denial asserts the cited condition, not just
  "denied".
- **Codegen:** eligible site emits thunk+try_fork/join; ineligible sites emit
  today's code. **Corrected 2026-08-22:** this line also said "WF_WORKERS unset
  ⇒ no runtime linked/no behavior change", which is false in both halves at the
  tip. What is linked is decided at compile time by `--par`, not by the
  variable, and an unset variable now starts a pool rather than suppressing
  one. The acceptance criterion that survives is the compile-time one: a build
  without `--par` links no runtime and is byte-identical.
- **Determinism (in-crate, non-protected):** run the demo N times at
  WF_WORKERS ∈ {1,2,4,8}, byte-compare full stdout; deliberately break the
  lowering in one direction (skip join) in a negative control to prove the
  test can fail.
- Gate: `make -C compiler check` green after every executor stage; full
  `make check` green before audit.

## 8. Demo and measurement

Port the debate's realistic-body workload (`a2r_layout*` shape: box-tree
recursive fold, per-node float cascade + word loop) as
`tests/programs/par_layout.wf` (claim-free variant for eligibility; a
claim-bearing sibling variant to show the ledger's not-actualizable line).
Measure per the protocol (interleaved A/B in one process, best-of-N,
differences under 20% reported as unresolved): WF_WORKERS=1 vs 2/4/8,
wall time + byte-compare. Report the Amdahl share observed. Results into
`research/investigations/proof-derived-parallelism/RESULTS.md`, alongside
the deciding debate probes copied into `probes/` here (evidence-mortality
repair; sources under do_not_scan are dying artifacts).

## 9. PAL.md alignment (the requested gap-check, condensed)

Adopted from PAL: the four-layer separation (permission / decomposition /
profitability / mapping); marker-never-authority (marker itself deferred to
next packet); claim-free worker closures (§10.2 — vindicated by the claim
doctrine); serial fallback as the universal member; diagnostics taxonomy
(proved dependence / unknown / unsupported / not-profitable-is-never-an-
error) — the ledger uses it. Corrected against PAL: its observation list
omits termination — v1's wait-for-all join makes that moot for eligible
lanes; its scan-family and injective-scatter families are future growth.
Residual gaps carried to the record: the checker's band-vs-derived-index
discharge asymmetry (compiler defect candidate, zero spec bytes — fix if
the night allows, else record); the Amdahl share question (the demo
measures it); constraint-18 note (this design declares nothing and changes
no acceptance — the record carries the one-paragraph ruling request that
permission attribution is not "undeclared parallelism").

## 10. Merge packet checklist (what the owner sees in the morning)

Batch record 0074 (authority quote, scope, outcomes, audit dispositions);
spec candidate diff + SHA-256 + verifier output + impact inventory + zero-
conformance-delta rationale; compiler diff with tests green (`make check`
on branch tip); ledger sample output; RESULTS.md measurements; PROPOSED
current-plan + roadmap updates; deferred-items register (§2 Out). Nothing
merges without the owner.
