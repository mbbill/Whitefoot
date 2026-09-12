# Pull requests: mbbill/whitefoot

Fetched via the GitHub MCP tools (list_pull_requests, state=all, paginated) on 2026-09-12. 47 pull requests exist (#1-#47), all listed below in ascending number order. `Merged` is derived from the API's `merged_at` field, not its `merged` boolean, because the list endpoint's `merged` boolean was observed false on every row regardless of actual merge status (verified against `pull_request_read` on PR #1, which correctly reports `merged: true`); `merged_at` agreed with `pull_request_read` wherever cross-checked.

---

## #1 Integration 2026-08-28c: batches 0103, 0102 (spec v0.39), 0105, 0095

State: closed | Merged: yes, 2026-08-29 | Head: integration/2026-08-28c | Base: main | Created: 2026-08-29 | Closed: 2026-08-29

## Summary

`integration/2026-08-28c` = main (`10b76c66`) + four verified batches, merged in order. Local canonical `make check` is green on the tip (`ddf358a1`, "ALL TESTS GREEN", exit 0); CI is running on this head. Two `static (macos-14)` rounds happened here, both the same defect in the interleave probe's platform guards, both fixed and verified locally with clang object compilation under `-U__linux__` plus a sweep of every completion source: first the eight locals declared before the non-Linux early return (first exposed by round 10's new `miscounted` counter), then the three Linux-only helper functions that early return left unused — the whole helper block now sits inside one `#if defined(__linux__)` region.

## Batch 0103 — quiet notes under `--no-overlap`, and patterns P18

- `whitefootc` prints no denied-I/O-loop notes under `--no-overlap` (the flag already declares the writer wants no overlap); `--par-ledger` still prints the full report. Decided by the flag, not by text: `io_notice_report` returns no lines under it.
- `docs/patterns.md` P18 teaches the explicit per-iteration buffer form for a `&uniq` resource inside a loop (owner decisions 1 and 5 of 2026-08-28).
- Verified: 630 sources × 3 flag ways against the base compiler — 0 acceptance differences, 0 IR differences; stderr differs on exactly 9 no-overlap rows (notes withheld, no error silenced). Record: `docs/done/0103-quiet-no-overlap.md`.

## Batch 0102 — [CLM-1] narrowed, spec v0.38 → v0.39

- The claim-authority paragraph now counts exactly the control dependence a selection carries: a post-join definition built only from Local operands stays Local; delivered values, joined definitions, arm-written storage and loop-carried updates stay refused.
- v0.38 archived byte-exact; digest chain, `APPROVALS.md` line, transcribed literals, `REVIEWED_FOR`, META-5 all updated. 7 conformance cases added (3 accept, 4 reject), no pre-existing verdict changed.
- Verified by an adversarial skeptic (44 attack programs; every world-dependent claim still refused with the rule cited) and a gate verifier (all 120 program/codegen sources plus 510 pre-existing conformance cases byte-identical against the base compiler apart from the version banner). Fuzzer re-run: base 5/208 rejections, all CLM-1; narrowed 0 rejections, 203/203 agreed across the overlap matrix. Record: `docs/done/0102-clm1-narrow.md`.
- Note: the claim/entailment area is chartered for a first-principles redesign (the proof-replaces-claim design on `batch/0106-claim-model-design`); this narrowing is the verified interim state it will supersede in place.

## Batch 0105 — 0096 contract-verification follow-ups

- The `wf_bridge_spin_for_completion` comment rewritten to state only measured, mechanism-level facts (its previous liveness attribution was refuted against the code and its route contrast did not survive measurement; counts are labeled as draws).
- New regression test `test_shutdown_refuses_every_later_entry`: post-shutdown entry points refused at the guard, second shutdown returns EINVAL (red when the `initialized` store is removed).
- Test-only `WF_IO_NO_NATIVE_RING` knob plus a gate arm so the default-route probe's adapter branch is exercised on Linux CI (previously dead code wherever io_uring works).
- Honest shutdown-precondition wording in `file_adapter.h`. Record: `docs/done/0105-bridge-comment-and-shutdown-coverage.md`.

## Batch 0095 — completion-runtime retirement accounting (Stage A), and the ring-storage prerequisite

- **Stage A, complete and confirmed** (ten adversarial fix/verify rounds). The final defect was the award mark moving *backwards*: `wait_begin` reset `awarded = seen` from a stale pre-attempt snapshot, so one returned descriptor could be promised to three waiters in turn, and source order lost an owed `Ok` (~1 in 4,050 repetitions under contention). Fix: the mark is forward-only and every host-satisfied open is charged one unit under `returns > awarded`, at all five host-success sites on both routes. Measured at the tip: 0 losses in 1,920,000 repetitions with the pre-fix control interleaved in the same sweep still losing (237/960,000). Independently re-verified (A10 CONFIRMED: own 300k+ sweep, charge-site enumeration, double-charge and outside-free attack schedules, full prior battery, tsan/asan). Four scripted regression cases shipped, each pinned by a mutation control.
- **Stage B (the pipeline driver) is deliberately not in this PR** — shelved by owner sequencing 2026-08-29. What landed is its one prerequisite: slot-indexed (ring) completion storage in the emitter, with three refusals and mutation-pinned tests. Nothing constructs a pipeline outside `#[cfg(test)]`, and both Stage B verifiers CONFIRMED byte-identity: 1,917 compilations / 834 modules across three flag modes against the pre-ring compiler, differ=0; 13 adversarial programs × 9 worker/helper cells × overlap modes, zero byte differences.
- The verifiers' findings beyond the range are recorded in `docs/ongoing/0095-loop-pipeline.md` as obligations the driver work inherits: the staged test modules fail `llvm-as` verification (a pre-existing staged-join dominance defect the ring probe is the first to expose); the slot index is trusted, unbounded against the ring width; per-operation SSA facts are not ringed; the submitted open's staged-component ring is untested. The record stays in `docs/ongoing/` because Stage B is not done.
- Runner benchmark tables: not applicable to this range — no emitted production byte changes, so there is nothing to measure; the measurement plan transfers to the driver work.
- Merge conflict resolved: the `file_adapter.h` shutdown paragraph kept 0095's wording because the merged code is 0095's (`wf_file_adapter_queued` is a plain atomic load, no lock).

## Approval classes

- Spec: yes (0102 activates v0.39) — `governance/APPROVALS.md` carries the merge-time record and chain line; the conformance boundary is recorded in the 0102 record against main tip `10b76c66`.
- Conformance: yes (0102 adds 7 cases; none modified/deleted/renamed). 0095 adds compiler/harness tests only, no conformance content.

## Deferred (not in this PR, by owner sequencing 2026-08-29)

- Stage B pipeline driver (obligations recorded in the 0095 ongoing record).
- The [PAR-3] follow-up spec sentence (D1): deferred rather than spending a v0.40 activation right before the proof-replaces-claim redesign rewrites that area.

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---

## #2 Proof replaces claim: the design corpus (research only, no compiler change)

State: closed | Merged: no (closed without merging) | Head: batch/0111-proof-replaces-claim | Base: main | Created: 2026-08-29 | Closed: 2026-09-05

## What this PR is

The complete proof-replaces-claim design corpus under `research/investigations/claim-model/`, carried onto the post-merge main. **Research documents only** — no spec byte, no compiler code, no conformance content changes. It is here so the owner can read and comment in PR view; the design keeps evolving on this branch and the PR updates with it.

## Where to start reading

`DESIGN.md` (6,961 lines) is the single design; the four files beside it (`TERRAIN.md`, `CENSUS.md`, `F2-REVIEW-TRIAL.md`, `CLAIM-DISSOLUTION-AUDIT.md`) are unchanged evidence it cites.

- **Section 0** — what changed and the result in one page; §0.1 is the supersession table against the previous DESIGN.md.
- **Section 1** — the principle: premise ownership kept, the third publisher (reviewed claim) replaced by guards + retention + the verified bound statement + contracts.
- **Section 2** — the adjudication of the judges' findings; **§2.10 is the owner's redundancy ruling** as the load-bearing asymmetry.
- **Section 3** — the construct catalog with draft spec text: the deletion (§3.1: CLM-1/2/3, DIAG-3, TRAP-1 go; 138→133 rules; [ENT-1] monotonicity becomes an unconditional theorem), guard publication (§3.2, unchanged rule), loop retention P-LOOP (§3.6), **the bound statement `[IND-1..10]` (§3.8–3.9)** including the certificate check, its four laws L1–L4, the 25-row reachable-error sweep, and the five-part monotonicity argument, the contract overhaul (§3.10: `len` atom, widened ensures datums, `[FN-10]` write postconditions), the world boundary (§3.11), the if/else residue and the impossible-else pricing (§3.12).
- **Section 4** — the complete case walk: 50 audit scenarios, the seven irreducibles (I1 closed via `[IND-10]`, I4 via the counter trace), the six claim customers, the vocabulary price list (§4.4), and all 135 corpus claims dispositioned (§4.5).
- **Sections 5–7** — T3/W3 re-derived; diagnostics (the computed seven-token gap channel and the mechanical fix table); conformance plan and META-5 delta.
- **Section 8** — the implementation plan in batches.
- **Section 9 — the flagged decisions awaiting the owner** (each adopted-and-flagged by the lead, none owner-approved): D1 the certificate check's [ENT-1] status, D2 `[IND-10]`'s straight-line restriction, D6 the `[FN-10]` backward-cursor refusal, D7 the demoted backward wrap rows, D8 the S10 widening condition.
- **Sections 10–12** — open questions with recommendations, honest limits (red ink), and the evidence ledger.

## How it was verified

Three designers → two adversarial judges → synthesis; then the design's own two falsifiers were run before freezing text: **F-D4** rewrote the three flagship corpus programs fully claim-free at today's v0.39, byte-identical over 1,195 differential cases (the dissolution is stronger than first claimed: eleven of eleven sites close today), and **F-I1** hand-executed the certificate check and refuted it — twice soundness, then monotonicity three more times — driving five substantive repair rounds. Round 5 rewrote the measurement-and-monotonicity story as one derivation on four stated laws and the full attack battery **CONFIRMED** it ("the first round where the argument is not one step behind the text"), followed by two wording rounds, each verified. Every attack, trace, and repair is recorded in §3.9.7 and §12.4; the remaining risk is empirical (cap sizes and reach costs on real programs), owned by the implementation batches.

## In flight, landing on this branch

- The **guarded-facts extension** (owner direction 2026-08-29: multi-flag two-pass programs must compile with zero workarounds) — drafted and adversarially audited separately, integrated here when it survives.

Merging this PR is the owner's call under rule 2 and is not needed for the work to continue.

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #3 Complete the Windows IOCP runtime backend

State: closed | Merged: yes, 2026-09-03 | Head: codex/windows-iocp-runtime | Base: main | Created: 2026-08-30 | Closed: 2026-09-03

## Summary

- Complete the `x86_64-pc-windows-msvc` command and runtime path: UTF-16 bootstrap, cwd-relative namespace operations, native error mapping, bounded blocking work, IOCP completion, writer scheduling, and a mandatory native compute pool for `--par`.
- Fail closed instead of degrading. Invalid worker configuration, partial pool startup, missing runtime ABI, IOCP startup or association failure, and exhausted completion capacity cannot silently become sequential execution or direct I/O. Under capacity pressure, the emitter retires the oldest addressable one-slot owned generation; when only ring-backed owners remain, the unified scheduler makes runtime progress before the exact operation is retried.
- Harden the hot path. Completion drain is generation-safe and token-exact; empty ready queues return without a slot scan. IOCP handles use `FILE_SKIP_COMPLETION_PORT_ON_SUCCESS | FILE_SKIP_SET_EVENT_ON_HANDLE`. Synchronous `ReadFile` success publishes through the normal terminal path on the submitting thread, while `ERROR_IO_PENDING` remains a real IOCP dequeue path.
- Integrate compute, writer, completion, and capacity progress through one work-first scheduler. The production bridge, blocking pool, and writer-ready queue share one capacity contract with compile-time consistency checks.
- Reconcile the Windows backend with the v0.40 proof-based source model and current `wf__par_acquire_lane(i64)` ABI without restoring retired claim/trap machinery.
- Run the compiler driver on a named 8 MiB thread so normal Windows launcher stack limits do not fail during compiler recursion.
- Add deterministic normal, pending, synchronous-success, immediate-error, notification-failure, capacity, fault, identity, corpus, native Windows, and paired performance gates.

## Exact revision and v0.40 base

PR head:

`631961ae95a6a7eea108a4ed57dcbbba92ee594c`

v0.40 `main` incorporated by this head:

`29d523f6eae8c54fd6ef5077c68ea92f15415c6f`

The remote `codex/windows-iocp-runtime` branch and this PR head both point to the exact head above. Relative to that v0.40 base, the PR changes 71 files with 16,008 insertions and 547 deletions, covering the Windows runtime, completion backend, compiler emission and driver integration, qualification gates, benchmarks, and supporting documentation.

## Correctness and build evidence

- Independent canonical `make check` on the exact head passed: 1,481 library tests (`1,411` fast plus `70` sampled), 53/53 program tests, and conformance `Pass=490 Skip=1`.
- Local `make -C compiler completion-windows-cross` passed the strict Windows x86-64 compile/link boundary, including the real `wf_floor_windows.c` runtime object. This is a cross-link check; native execution is covered separately below.
- [Canonical gate run 33724707805](https://github.com/mbbill/Whitefoot/actions/runs/33724707805): 12/12 Ubuntu and macOS jobs passed. It independently exercised the fast, sampled, corpus, conformance, static, and research rows on both hosts.
- [Native host run 33724707724](https://github.com/mbbill/Whitefoot/actions/runs/33724707724): 2/2 Linux and Windows jobs passed. The [Windows job](https://github.com/mbbill/Whitefoot/actions/runs/33724707724/job/100551007773) executed the completion, native adapter, bounded blocking, resource interleave, namespace, strict-runtime, bridge fail-stop, bridge capacity, real compiler IOCP, compiler capacity recovery, HostString, component-open, and native `--par` worker gates.
- The native capacity gate observed `accepted=3 publications=3 consumes=3 capacity_waits>0 fallback=0`, proving pressure recovery at the real compiler/runtime boundary without direct fallback.

## Windows performance qualification

[io-bench run 33724707786](https://github.com/mbbill/Whitefoot/actions/runs/33724707786) completed with all four jobs green. The [Windows qualification job](https://github.com/mbbill/Whitefoot/actions/runs/33724707786/job/100551008346) ran on the exact head.

It used Windows Server 2025 on an AMD EPYC 7763 allocation with 4 visible logical processors, 4 workers, affinity `0xf`, High performance power mode, clang 20.1.8, and rustc 1.98.0. Each cohort used 2 warm-up pairs followed by 15 recorded alternating pairs on the same host. Every cohort passed the stability gate on attempt 1.

| cohort | reference median ms | candidate median ms | paired candidate/reference | MAD | p10..p90 | production bound |
|---|---:|---:|---:|---:|---:|---:|
| compute | 4717.706 | 1314.381 | 0.2789 | 0.49% | 0.2742..0.2802 | <= 0.90 |
| io-warm | 229.406 | 211.343 | 0.9210 | 1.41% | 0.8948..0.9507 | <= 1.10 |
| mixed-iocp | 275.209 | 277.538 | 1.0075 | 0.26% | 1.0044..1.0146 | stability control |
| mixed-full | 277.065 | 165.706 | 0.5965 | 0.75% | 0.5915..0.6111 | <= 0.95 |
| mixed-total | 275.270 | 166.600 | 0.6033 | 0.57% | 0.5995..0.6164 | <= 0.95 |

Backend identity from the same untimed observed link:

```text
windows-native-mixed-probe status=pass started=3 executed=1024 grants=1024 publishes=1024 outstanding_publishes=1024 kernel_overlap_publishes=1024 inline_completions=0 dequeued_completions=1024 submissions=1025 publications=1025 consumes=1025 helpers=1 fallback=0
```

All 1,024 overlapped reads completed through IOCP dequeue, directly exercising the queued kernel path with zero eligible fallback. The deterministic Win32 seam separately covers synchronous-success inline publication, synchronous result failure, immediate failure, pending dequeue, notification-mode refusal, announced-waiter wakeup, and the absence of a ghost packet after inline completion.

## Scope boundary

- Positioned `read_at` is the current IOCP-eligible source operation. Other operation families follow their selected direct or bounded-blocking contracts; backend failure is not permission to reroute an eligible read.
- This is a same-host paired qualification with complete host identity and raw samples, not a claim that GitHub supplies a persistent cross-revision physical machine.
- The Windows branch adds no independent change to `spec/kernel-spec.md` or the compiler-independent conformance cases or manifest relative to current v0.40 `main`; the merge commit incorporates the v0.40 base bytes unchanged.

---

## #4 Implement source-carried proof compiler

State: closed | Merged: yes, 2026-09-03 | Head: codex/source-proof | Base: main | Created: 2026-09-01 | Closed: 2026-09-03

## What changed

- Positions Whitefoot explicitly as a proof-carrying systems language: contracts, invariants, and finite proof guidance live in ordinary `.wf` source, are checked during semantic compilation, and are erased before typed IR.
- Removes `claim`, the `traps` effect, and every writer-reachable language trap. Every supported partial operation must be proved before lowering or compilation is rejected; there is no hidden runtime fallback.
- Gives counted and ordinary loops a closed induction header. The first `for` item is the binding, every remaining item is an `invariant`; `loop` headers contain invariants only. A counted header with no invariant renders on one line (`for (i in a..b) {`); a header with invariants keeps the multi-line form. Commas are separators and a trailing comma is rejected.
- Implements simultaneous loop-invariant base and arbitrary-backedge checking, body-only theorem names, counted exhaustion substitution, and no exhaustion export through `break`.
- Uses the same `invariant` keyword for one-time program-point facts. A local invariant may carry explicit `use` steps; loop-header invariants may not.
- Makes AUTO's author-visible boundary exact and deterministic: DIRECT, every coefficient-one single published affine premise, every unordered coefficient-one pair including self-pairs, then the fixed L0-image route, each candidate followed by the fixed integer tightening (divide an accumulated candidate by the target-multiple factor and by the coefficient divisor, flooring the bound). Three-or-more-premise combinations, explicit multipliers, and special routes are directed by `use`.
- Checks each `use` premise against the entering snapshot, combines written steps linearly, permits target weakening, rejects duplicate premises and redundant `use` blocks, and publishes only the outer invariant.
- Gives a `set` commit the same value image its `let` spelling has: the right-hand side is evaluated to a compiler-owned commit value before the [SET-1] kill (image, pre-kill closure, kill, copy equality), so `set i = i + 1_u64` and `let j = i + 1_u64; set i = j;` reach the same verdict. This is what makes an ordinary `loop` header invariant over a guarded cursor provable.
- Routes contracts, control-flow facts, invariants, and operation obligations through one `ProofContext + ProofGoal + prove` compiler interface backed by fixed equality, difference-bound, affine, ownership/effect, layout/address, and target-domain rules. It introduces no SMT solver, timeout, fuel, proof-work budget, random seed, proof cache, external certificate, or compiler self-validation layer.
- Preserves derived facts before writes, derives fixed unsigned-literal division images, keeps complete expression identities stable across aliases, and retains canonical facts through control-flow joins.
- Lets `par` consume the same checked ownership, effect, affine-map, layout, target, queue, and completion facts. A missing optional parallel permission produces sequential lowering rather than source rejection.
- Updates README, compiler documentation, real programs, grammar data, specification, diagnostics, and compiler-independent conformance evidence together.

## Specification and scope boundary

This revision **activates Whitefoot v0.40** on the branch: `spec/kernel-spec.md` reads `Status: ACTIVE v0.40`, SHA-256 `15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168`, superseding `main`'s active v0.39 (`b4d8e01e…`), whose bytes are archived byte-for-byte as `spec/kernel-spec-v0.39.md`. The intermediate draft that carried `prove`/`use` and the earlier v0.41 numbering were collapsed into this one version. `governance/APPROVALS.md` appends the merge-time record for v0.40 (exact specification bytes, META-5 delta from v0.39: rules +2/-9 with 131 remaining, productions +8/-1 with 82 remaining, and the exact conformance boundary: 34 added, 102 modified, 60 deleted, 5 renamed case files; 39 added and 65 removed case ids; no retained id changes its declared verdict; the runner, adapter, collection, manifest, and gate-wiring digests before and after) followed by `ACTIVE-SPEC: v0.40 15ec2f6f… b4d8e01e…`. The record states that it becomes effective only when the owner approves this exact revision for merge (rules 2 and 4); no approval is claimed by the branch.

Only external resource availability remains deferred in this cycle: heap exhaustion, stack exhaustion, operating-system quotas, and runtime-start resources. Layout, address, allocation ceiling, target qualification/domain, parallel independence, and bounded queue/completion proof are not deferred.

`docs/constitution.md` is unchanged.

## Verification

Canonical `make check` on this exact revision (`0b847a3a`, Linux, run under a wrapper that drops root's permission bypass so the unreadable-path fixtures behave): `== WHITEFOOT ALL TESTS GREEN ==`. All root gates pass, including spec archive integrity (41 recorded specifications hash as recorded) and spec digest sync (live prose quotes v0.40). The compiler stage, unit tests, sampling, corpus, research tests, native conformance (490 Pass, 1 existing Skip, 0 Fail), and the compiler-independent conformance run are green. Locally the serial compiler stage takes longer than five minutes because it runs the CI unit, sampling, corpus, and static jobs one after another; each CI job stays under five minutes.

An adversarial probe set of 68 programs (guarded cursors, wrap/sat/checked right-hand sides, struct and buffer targets, joins, midpoint) shows every newly accepted program holds for concrete values and every must-reject program still rejects. No compile-time or runtime numbers are recorded; performance is compared on the GitHub runner stages only.

## Known boundaries kept out of scope

AUTO never pairs an affine premise with an L0 relation, so the certificate-free binary-search midpoint still needs one `use` block; non-affine addressing (`r * cols` with two nonliteral operands) is outside the affine fragment. Both are recorded in `docs/roadmap.md`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---

## #5 Let the gate name main on a push to main

State: closed | Merged: yes, 2026-09-03 | Head: batch/0113-gate-main-ref | Base: main | Created: 2026-09-03 | Closed: 2026-09-03

## What changed

Every `gate` job on a push to `main` has failed at its first step, "Name main as a local ref", for as long as the workflow has run on main (runs 45, 104, 151, 192): `git branch -f main origin/main` refuses to move the branch the checkout has out, so the step exits 128 before any gate target runs. Pull-request runs check out the PR branch and were never affected, which is why every PR gate has been green while every main gate has been red.

The step now creates the local `main` ref only when the checkout is not already on `main`; on a push to main it just prints the commit. Nothing else in the workflow changes, and no Makefile, gate target, or test changes.

## Verification

The pull-request path is exercised by this PR's own gate run. The push-to-main path cannot run before the merge; it is a two-branch shell conditional on `git symbolic-ref -q HEAD`, and the first push to `main` after merging will show it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #6 Sweep fixes: re-close after kills, causal diagnostics, join-image pattern

State: closed | Merged: yes, 2026-09-03 | Head: batch/0114-sweep-fixes | Base: main | Created: 2026-09-03 | Closed: 2026-09-03

## What changed

Three findings from the scenario sweep over the v0.40 compiler (48 finder agents, 470 programs, every mismatch judged against the specification text); the fourth finding, the callee `&uniq buffer<T>` length-fact hole, is tracked here as a conformance xfail and left for the container/ownership redesign.

- **Re-close a fact state after a kill; re-emit [ENT-2] implicit bounds at every closure entry.** `FactState::kill` and `kill_goals` removed bounds without clearing the closure memo, so the next query took the fast path and returned the post-kill map verbatim; the materialized `len(arr) = N` pair for an `array<T, N>` was the only copy after a join, and a root `replace` killed it. Acceptance of a 13-line program depended on how many terms the function happened to intern (an unrelated `let` after the `replace` flipped the verdict). The [ENT-2] rule table is now one routine used by all three closure entry points, and a missing implicit bound is restored on the fast path. Tests pin the verdict's invariance under an unrelated binding, and `ent5-neg-kill-on-write` still rejects. New conformance case `ent2-pos-array-length-survives-root-replace`.
- **Report the body failure that breaks a loop backedge, not the backedge.** The single [DIAG-1] rejection was chosen by the failing outcome's written node path, so a loop-header invariant's backedge failure always beat the body [OP-2]/[OP-4]/[FN-8] failure that demoted the value and caused it. The selection key is now the order in which the checker decides obligations: an INV-1 backedge failure is positioned after the enclosing loop's subtree; base failures stay at the header. No diagnostic shape changes; nine sweep programs now cite their real cause; no conformance verdict moved.
- **Docs and conformance for the join-image rule.** `docs/patterns.md` gains P19 (a conditionally advanced loop binding: every arm of a body join must leave the tracked binding with the same affine image or images differing only by a constant; three accepted shapes and two repairs, each compiled), a sentence in P8 (a counter-relative accumulator bound proves nothing unless the counter is bounded), and two sentences in P14 ([DIAG-1] reports one rule and one location, so a silent probe invariant is not a proof; AUTO's pair family includes the self-pair). Conformance cases `ent6-neg-join-one-arm-advances-accumulator`, `ent6-pos-join-value-if-lifted-addend`, and the tracked gap `ent5-neg-callee-uniq-buffer-replace-kills-length` (status xfail with its reason).

## Verification

Canonical `make check` on the merged head `239879aa` (Linux, run under a wrapper that drops root's permission bypass): `== WHITEFOOT ALL TESTS GREEN ==`; every stage under five minutes. Conformance adapter: Pass=492, Xfail=1, Skip=1. Library suite 1484 tests. Across the 665-program sweep corpus, the D2 fix flips exactly its reproducer from reject to accept and unmasks one genuine [OP-2] obligation in its parent program; the D3 fix changes the cited rule on nine rejected programs and flips no accept/reject verdict.

The specification is unchanged (active v0.40); the conformance manifest gains four cases.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #7 Land the snapshot corpus as its own gate stage

State: closed | Merged: yes, 2026-09-03 | Head: batch/0115-snapshot-corpus | Base: main | Created: 2026-09-03 | Closed: 2026-09-03

## What changed

The scenario sweep that found the D1 to D4 defects also produced several hundred realistic programs with a stated expected verdict. This lands the ones whose expectation was stated as a snapshot corpus so the sweep's reach is kept as a regression net.

- `tests/snapshot/`: `README.md` (first paragraph: this records the current compiler's verdicts, is not specification evidence, and lives outside the conformance boundary; the columns; the accept/reject-only gate; the flip rule; the removal condition), `index.tsv` (491 rows, the single source of truth: id, family, verdict, cited rule as reference only, finder expectation, agreement, one-line doc), and `cases/<family>/<id>.wf` (491 programs across 12 families; 303 accept, 188 reject).
- `compiler/tests/snapshot.rs`: reads the index, compiles each case through the ordinary compiler path (no linking, no execution), compares accept/reject only, prints `Pass=N Flip=M`, and fails on any flip with id, expected, actual, and the first diagnostic line. Index order; deterministic.
- Wiring: `snapshot-run` after `conformance-run` in the root Makefile's `CHECK_STAGES` with its own timing line; added to the CI `conformance` job; `snapshot` added to the compiler's integration-target list.

Every verdict was re-derived with the compiler that carries the sweep fixes (PR #6). Thirteen rows disagree with the finder's expectation; each doc line states the finder's expectation, the compiler's verdict, and the rule that decides it (four are the join-image rule documented as P19, two are redundant `use` blocks, the rest are unbounded counters or ordinary type errors). The D1 rows carry no stated expectation and are therefore not in the corpus; that gap is tracked by the conformance xfail case `ent5-neg-callee-uniq-buffer-replace-kills-length`. One sweep file was dropped because it fails in the parser and records nothing about the checker.

No specification or conformance content changes. A future verdict flip must be explained in the commit message; when it is a known defect being fixed, the flip is the expected outcome.

## Verification

`snapshot corpus: Pass=491 Flip=0`. Canonical `make check` on `e336db77` (Linux, under the permission-bypass-dropping wrapper): `== WHITEFOOT ALL TESTS GREEN ==`; the new stage takes about one second of wall time, so the CI `conformance` job stays far under five minutes. `cargo fmt --check` and `clippy -D warnings` clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #8 Judge SET-2 and STOR-5 targets by the region-bearing relation

State: closed | Merged: yes, 2026-09-03 | Head: batch/0117-set2-region-bearing-targets | Base: main | Created: 2026-09-03 | Closed: 2026-09-03

## Defect

[SET-2] makes a region-bearing `replace` target a hard error, and [STOR-5] defines a type as region-bearing when its complete type contains `slice<'r, T>` or `arena<'r, T>` at any depth. The checker tested only the `Slice` constructor, so two programs the spec rejects were accepted at `main` (exit 0):

```
region 'r {
  let first = arena_new<'r, u64>(1_u64);
  let second = arena_new<'r, u64>(2_u64);
  let previous = replace first = move second;   // SET-2 requires rejection
}
```

```
region 'r { let a = arena_new<'r, u64>(1_u64); let b = box_new(move a); }   // STOR-5 requires rejection
```

Found by a design falsifier while probing the container redesign; it is a defect against the active v0.40 spec, independent of that design.

## Fix

One structural predicate, `checked_type_is_region_bearing` in `compiler/src/semantic/check/types.rs`, computes [STOR-5]'s relation from the checked type: a slice, an arena instance, or anything reaching one through array or buffer elements, box referents, struct fields, or enum payloads, with a visited set for nominal cycles. It is used at the [SET-2] replace target, at `box_new`'s derived-referent [STOR-5] check, and in the generic-substitution arm of the written-type predicate, replacing three constructor-name tests. The SET-2 diagnostic now carries the spec's restructuring text, which names the arena case.

## Evidence

- Conformance cases added: `set2-neg-arena-replace-target` (reject SET-2), `set2-pos-box-descriptor-replace` (positive control, run exit 0), `stor5-neg-box-new-arena-content` (reject STOR-5). Adapter tally 496 pass, 1 xfail (the recorded D1 case), 1 skip.
- Unit tests added in `semantic/tests/replace.rs` and `semantic/tests/boxes.rs`.
- Snapshot corpus: 491 pass, 0 flips (it contains no arena program).
- Canonical `make check` green on this revision (`4faf6d23`). One earlier run failed only in `backend::tests::stackless::may_suspend_tail_wrappers_release_the_writer_stack_and_resume_on_a_scheduler_lane` while the host was under heavy load from other work; that test passes repeatedly alone on this branch and on `main`, and this change touches no backend code. Its load sensitivity is noted as a separate follow-up.

## Merge record

This PR adds conformance content, so the merge needs the rule-4 conformance record in `governance/APPROVALS.md` (three added cases and their manifest rows; no case modified or removed).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #9 Activate kernel specification v0.41: integer comparison symbols and the call-site `::` delimiter

State: closed | Merged: yes, 2026-09-03 | Head: claude/ecstatic-archimedes-t3mzp9 | Base: main | Created: 2026-09-03 | Closed: 2026-09-03

## What changed

The owner rulings of 2026-09-03 on the FLOOR-5 comparison row, carried through the specification, the compiler, and every derived artifact in one batch, and activated as v0.41 on this branch after the owner's approval of the candidate. The three rulings: the six integer comparisons become symbols; a call's type and region arguments are written after a `::` delimiter; the `use` multiplier stays and a multiplied relation-form use step is parenthesized. Migration cost was not a criterion for any of them.

**Specification.** `spec/kernel-spec.md` is `ACTIVE v0.41`, SHA-256 `899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761`, superseding v0.40 at `15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168`, whose outgoing bytes are archived byte-for-byte at `spec/kernel-spec-v0.40.md`. The candidate hashed to `55ee571e…`; activation flipped its status line and changed no other byte.

- `ieq ine ilt ile igt ige` are respelled `== != < <= > >=` as one `compare_op` class of `infix_tail`, integer-only rows exactly as the arithmetic symbols are. Float and enum comparisons, `band`/`bor`/`bxor`/`bnot`, the shifts, and the unary operations stay named.
- A call writes its type and region arguments after `::` (`cvt::<u8, u32>(w)`), at call sites only; constructors and type positions are unchanged. `IDENT "<"` therefore begins only a comparison, and every grammar decision keeps its two-token bound, so the grammar stays strong LL(2).
- The four ordered symbols replace `ile ilt ige igt` as the relation of a header invariant, a local invariant, and a relation-form use step; a multiplied relation-form use step is parenthesized, `use 3 * (a <= b);`. Equality and disequality are not invariant relations.
- Compound tokens `== != <= >= ::`; the `!` byte enters the token alphabet inside `!=` only.
- Twenty-one rules and the header move at forty-one verbatim-anchored sites. `spec/derivation/derivation-ledger.md` and `governance/spec-evolution/comparison-symbols-v041-candidate.md` record the delta, the acceptance-set delta, the rulings, and the rejected alternatives (an undelimited LL(4) call, a delimiter on every type application, dropping the use multiplier).

**Merge-time record (rule 4).** `governance/APPROVALS.md` gains the v0.41 record: the specification record names the exact bytes and the archived v0.40 bytes; the conformance record names the five added and 275 respelled cases against `main` tip a97edf9, the manifest before/after digests, the verdict and coverage totals, and the runner and adapter files that are byte-identical to `main`; the `ACTIVE-SPEC: v0.41` line extends the chain. The record becomes effective with the owner's merge approval of the exact revision containing it.

**Compiler.**

- Lexer: five compound tokens and the `!` byte; five fixed terminals; grammar tables regenerated from the v0.41 EBNF (83 productions, 109 decisions, 4,305 select rows, 106 terminal predicates); spec identity regenerated (chain length 33); the embedded active-specification constants name v0.41 and its measured digest.
- Parser diagnostics: the DIAG-1 row-1 and row-2 attributions are keyed on `(IDENT, "::")`, and the forbidden-atom override recognizes a delimited call start.
- FORM-2 renderer and audit: `::` in both attachment sets, a `compare_op` `<`/`>` in neither, and the stated space between a use multiplier and its `(`.
- Resolver: the invariant-carrier role is gone; the operation catalog spells the six families as symbols, and the retired names are free identifiers.
- Checker: infix comparisons, contract clauses, invariant relations, and use steps are read from the `compare_op` node; goal and invariant renderings and the entailment flow use the new spellings.
- Backend: the per-activation qualification review pin is bumped to v0.41 with its dated review note (the respelled rows lower to the same `icmp` predicates; no system operation, resource, release row, result shape, entry form, or host ABI mapping changes).

**Corpus, conformance, and docs (rule 4 content).**

- Every `tests/**/*.wf` case, `tests/snapshot/index.tsv`, `tests/conformance/manifest.jsonl`, the embedded test programs, and the current documentation examples were respelled by a one-shot token rewriter that is not shipped. Verdicts, rule citations, statuses, and coverage rows are unchanged; the historical kernels in `docs/why-whitefoot.md` Part II keep their historical spelling.
- Five conformance cases pin the new rules: `gram5-pos-comparison-operators`, `gram5-neg-type-application-without-delimiter` (a bare `cvt<u8, u32>(narrow)` parses as a comparison and fails at `u8`, which DIAG-1 row 3 attributes to FORM-3), `gram4-neg-multiplied-use-relation-bare`, `prf1-pos-multiplied-relation-use`, and `inv1-neg-equality-relation`.
- Research programs a maintained runner still compiles are respelled and were compiled through the branch compiler afterwards: the ten `research/experiments/io-completion-bench/programs/*.wf` benchmarks that the `io-bench` and `io-hosts` workflows build, and `research/experiments/buffer-initialization-cost/drain.wf`. Four programs that already fail to parse on `main` for an unrelated `for @label` form (`many_files_loop.wf` and the three `wfgrep-double-walk` shapes) and the dated evidence under `research/investigations/` and `research/experiments/blind-writer/` keep the spelling of the specification they were written against.
- `main` at a97edf9 (PR #8) is merged in; the two conformance cases and the one semantic test it added with the old spelling are respelled, with their verdicts unchanged.
- `README.md`, `compiler/README.md`, `docs/roadmap.md` (revision 63; FLOOR-5 closed), `docs/current-plan.md`, `docs/patterns.md`, the derivation ledger, the design memory (`mcts_mem/whitefoot/surface-form/operation-spelling.md` and its `.alt/named-comparisons.md`), and the SWEEP record name v0.41 and its digest.

## Verification

On the activated head 710d8ce (Linux, gate profile):

- `make spec-archive-integrity`: 42 recorded specifications hash as recorded; `spec-digest-sync`: live prose quotes the chain tail v0.41; `spec-append-only`, `approval-history-integrity`, and `repository-invariants` green. `whitefoot-spec` bin tests 18/18.
- `make conformance` green (131/131 rules; 113 by case, 32 by annotation); `make conformance-run`: `Pass=501 Xfail=1 Skip=1` (the xfail is the pre-existing `ent5-neg-callee-uniq-buffer-replace-kills-length`); `make snapshot-run`: `Pass=491 Flip=0`.
- `make -C compiler format lint docs spec completion-test` green. Full library suite: 1,489 passed, 1 failed, the stackless sanitizer probe that needs the `libclang_rt.asan` runtime this container lacks (CI's `unit` and `sampling` jobs pass it). `test-corpus` green apart from the three `programs` traversal tests that rely on `chmod 000` and cannot fail under root (CI's `corpus` job passes them). `make research-tests` fails locally only because the offline registry here lacks the `quote` crate; CI's `research` job passes.
- CI on the pre-activation head a392e6c: every job green except `static`, which stopped at the candidate-stage activation guard that this activation retires.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01W5Gx4bfx6tiozYwhVSTQBC

---

## #10 Bring the research programs and wfgrep bundles to spec v0.41

State: closed | Merged: yes, 2026-09-03 | Head: claude/kind-hamilton-d81821 | Base: main | Created: 2026-09-03 | Closed: 2026-09-03

## Summary

- **Four research programs respelled to spec v0.41.** `io-completion-bench/programs/many_files_loop.wf` and the three `wfgrep-double-walk/shapes/*.wf` still carried the v0.25 counted-loop header `for @label binder in lo..hi {` plus the pre-v0.41 comparison calls (`ieq` … `ige`) and bare call-site type arguments. The labelled `for` was never retired: v0.40 parenthesized the binding and made the label optional (`for_stmt := "for" LABEL? "(" for_binding … ")"`), and v0.41 respelled the six comparisons and added the `::` delimiter. The rewrite is exactly those three rules (4 headers, 184 comparisons, 108 call sites; no other line kinds in the diff), and the v0.41 respell commit `b046cfe8` had simply omitted these four files.
- **The wfgrep bundle family is made honest about HEAD** (`wfgrep-baseline/`, `wfgrep-double-walk/`, `wide-scan-lowering/`). Their subject was the `tests/programs/wfgrep.wf` of 2026-08-06; since `238ba7ce` (2026-08-18) that file is a recursive search printing `PATH:LINE:TEXT` lines, so their `verify` phases cannot match the pinned outputs from HEAD. Commit `c4e82fba` (2026-08-25) also rewrote path strings inside the committed raw evidence after each RESULTS.md pinned its digest. Each RESULTS.md gains a dated *Replay status* section stating both facts (no sample values changed). The double-walk driver and runner are reduced to what HEAD can still assert: the three shape sources, kept on the active specification, are compiled with the gate-profile compiler and verified byte for byte against the inherited manifest; `make check` there now runs `gen` when the corpus is missing and never appends to the committed raw file. The baseline and wide-scan drivers stay as the frozen runs' record with a boundary header, and their replay logs go to the scratch root instead of the committed raw files. The experiments index gains the two bundles it did not list.
- **Two v0.17-era floor studies retired as drivers.** `literal-line-floor/` and `wfgrep-scan-floor/` had Makefiles that fed kernels the current compiler rejects (unnamed result bindings, and the `traps` effect retired in v0.40). Those Makefiles are removed, each RESULTS.md records why and points at the freeze commit, and the index moves both bundles into a *Frozen v0.17 floor studies* section. `port-study/wc-chunk-summary/` is left as is: its Makefile invokes the archived democ toolchain, which the index already classifies as historical and not replayable.

No specification, conformance, or compiler code changes. `PROTOCOL.md` files are untouched; the frozen commits they name remain the protocol identities.

## Verification

- `compiler/target/gate/whitefootc --emit-llvm -o /dev/null <file>` on the four respelled programs: exit 0.
- `make -C research/experiments/io-completion-bench verify`: `all lines publish 17098009301725298919 00000000000071024640`.
- `make -C research/experiments/wfgrep-double-walk check`: with an existing corpus and from an empty corpus (the stamp triggers `gen` once); all three shapes and the pinned `grep -h -F` match all five output digests and exit codes; the committed raw files stay byte-identical.
- `make research-tests`: exit 0.
- `make check`: `== WHITEFOOT ALL TESTS GREEN ==` (compiler 149 s, research-tests 5 s, conformance-run 43 s, snapshot-run 2 s).
- Every `.wf` named by a Makefile under `research/experiments/` that invokes the current compiler now compiles (the two v0.17 drivers are gone; democ-era drivers are unchanged).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

---

## #11 Force region elision: kernel specification v0.42 (activated)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0118-region-elision | Base: main | Created: 2026-09-03 | Closed: 2026-09-04

## The rule

[FORM-8] Canonical region spelling (spec/kernel-spec.md lines 120 to 140). A region name is written exactly where the document does not otherwise fix the region denoted, and is absent everywhere else, so each region position has one legal spelling under [FORM-1]. Every clause is decided from the owning declaration's own text; a writer never needs a checker verdict to know which spelling is legal.

- Declarations: a region name is written only to relate two positions of the same declaration, or to name a result region that no input determines (the one region a caller must choose). Every other position is unnamed and distinct. The region parameter list exists only when a name is written and lists exactly the written names.
- Borrows: `&x` / `&uniq x` write a region only when it is not the innermost enclosing region block's.
- Region blocks: `region 'r {` only when the name occurs inside; otherwise `region {`.
- Calls: the `::` application carries only the region arguments no actual determines, as its leading members; when none remain, no `::` at all. Determined regions are inferred from the actuals; a set of actuals with no common outlives-or-equal region is an [OWN-4] error.

Before and after, from `tests/programs/byte_string.wf`:

```
fn bs_concat['d, 's](destination: &uniq 'd ByteString, source: &'s ByteString) -> result: own unit reads(...) ...
fn bs_concat(destination: &uniq ByteString, source: &ByteString) -> result: own unit reads(...) ...

    region 'measure { set count = bs_len::<'measure>(s: &'measure deref(source)); }
    region { set count = bs_len(s: &deref(source)); }
```

## The three commits

1. `5333a0cb` the amendment as a v0.42 CANDIDATE (digest `8cf0b914…`), 132 rules: [FORM-8] added; [GRAM-3/4/5] gain five optional REGIONID decisions and no production; [TYPE-5], [TYPE-6], [OWN-2/3/11], [FN-1/2], the 17 [SYS-2] signatures and the EX-1 example restated. Compiler: resolution mints unspellable synthetic regions for unnamed positions; the checker infers determined call regions from the actuals; a written-but-determined or missing-but-undetermined region is rejected under FORM-8 with the spec's restructuring text; grammar tables regenerated. Corpus: every program under tests, docs and live research respelled mechanically (region tokens across the four test corpora: 2623 before, 260 after; all 26 declarations that relate two positions remain written). Conformance: eight FORM-8 cases added; three "wrong region-argument count" cases retire with reasons, because a rule with exactly one legal argument list cannot express that fault.
2. `2e8a96fe` merge of `main` at PR #10; the four research programs both sides touched take main's v0.41 text and are re-elided.
3. `c168ab2a` activation, after the owner's review: the status line flips to `ACTIVE v0.42` (digest `6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26`) and no other spec byte changes; the outgoing v0.41 bytes are archived as `spec/kernel-spec-v0.41.md`; `governance/APPROVALS.md` carries the activation record and the `ACTIVE-SPEC:` chain line; `spec_identity.rs` regenerated; README, plan, roadmap, patterns and the derivation ledger name v0.42 as active.

## Evidence on the head

- Canonical `make check` on `c168ab2a`: all ten stages green, including `spec-archive-integrity` (43 recorded specifications hash as recorded) and `spec-digest-sync`.
- `cargo test --profile gate --lib`: 1490 passed. clippy `-D warnings` and `cargo fmt --check` clean.
- `make conformance`: coverage 132/132 rules. `make conformance-run`: Pass=506 Xfail=1 Skip=1 (the xfail is the recorded D1 case). `make snapshot-run`: Pass=491 Flip=0.
- CI: all 14 checks green on this head.

## Merge record

The merge needs the rule-4 specification record (exact bytes at digest `6b935d2e…`) and conformance record (8 cases added, 3 retired, 123 respelled) in `governance/APPROVALS.md`; the activation record already in the branch carries that content.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---

## #12 Design: containers and resources (eighth draft, for owner review)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0116-containers-and-resources | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

## What this is

The integrated design for containers, stores, and the resource-closed judgment, under `research/investigations/containers-and-resources/`. Research documents only: no compiler, spec, or test change. It is the eighth draft after seven adversarial falsifier rounds (memory soundness, resource-closed, spec consistency, writer usability, and a dedicated linearity round); the last three rounds' remaining findings were about the precise semantics of the fact machinery, which only an implementation can pin, so this is the last paper draft before implementation.

## How to read it

- `DESIGN.md` (6161 lines, 51 kernel rules). Read in this order: the settled decisions at the top; §1 (the two goals and D1 as the concrete failure); §2 (laws L1 to L18 and the notion-closure table in §2.1); §3.S (the decision record of every language-surface item, with the three still-PROPOSED items in a table at its top); §3.K (kernel rules, families [MSR], [PROV], [BLK], [VIEW], [LIV], [CALL], [RES], [STK], [RUN]); §4 (two worked programs: a heap-free cooperative run queue with a pool, and a hosted line collector over `Heap`); §7 (implementation order, first batch = the fact machinery the compiler must pin). §3.L is the library written in wf on top of the kernel, the partition test for the minimal-kernel ruling. §6 separates verified from reasoned and carries every falsifier finding and the rule that refuses it now.
- `CONTAINERS.md` and `RESOURCES.md` are the reduced companions (rationale and migrations). `EVIDENCE-owner-discussion-2026-08-31.md` and `EVIDENCE-sweep-D1.md` are the evidence.

## Decisions already made by the owner (recorded as decided)

Heap as an explicit capability value; `resource-closed` over an envelope E; the minimal kernel (only what wf cannot express; the rest is library or user code); no `&uniq` container or view parameters (parameters are inputs, results are outputs); one `set` commit rule with value lists (swap and rotate fall out; no exchange operation); `linear` modifier beside derived linearity; linearity read against whether the scope holds the capability (a scope holding `heap` derives release on every leaving edge, charged to `writes(heap)`); `dispose p;` resolving the capability from the brand; the shared `slice` is a copy value and `mut_slice` is affine; names `Vector`/`FixedVector`, `slice`/`mut_slice`, the `seq_` operations; `reserve_file` fallible; system range operations over views; region elision (v0.42, merged separately) and loop bodies as region blocks with the associative [ENT-6] join (v0.43 candidate, separate PR).

## Still PROPOSED, for the owner

- S31 `seq_reslice` (a helper handed a `&uniq mut_slice<u8>` publishing its fill).
- S32 a linearity bound on a generic parameter (so one generic body can serve affine and linear instances).
- S33 `reserve_file -> own ReserveOutcome` (a typed refusal for the handle table).

## Open questions worth the owner's time

Section 5, especially Q13 (release closure at region end for containers of linear elements) and Q17 (what the `linear` modifier buys beyond visibility).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #13 The I/O model under T4 (v0.50): the backed file permit, park-on-miss on the scheduler core, streams and TCP

State: closed | Merged: yes, 2026-09-06 | Head: io/t4-resource-relations | Base: main | Created: 2026-09-04 | Closed: 2026-09-06

The I/O model under constitution T4, three batches on one branch: the backed file permit, park-on-miss on the scheduler core, and streams and TCP. The specification amendment lands as **v0.50** over main's v0.49 (the branch's own v0.45 and v0.46 were renumbered at the merge of `043a721`; main's v0.45 through v0.49 are front-end and touch no rule this amendment touches).

**Answered at: `b21c0a3`**

## Derivation

### A. Language change

- Constitutional premise: T4 (resource dependencies are API relations, owner ruling 2026-09-04), with R1 to R3 for the spellings. Derivation: every native descriptor a program can hold is one credit of the entry's `HandleFactory`, consumed by an open, a listen, an accept or a connect, handed back inside the failed variant and by the explicit closes, so the factory's count is never raised and no operation can wait on a credit another holds; a connection's two directions are two fields of one returned struct, so full duplex is two `&uniq` loans on disjoint places under OWN-5 and no split operation is invented. Design: `research/investigations/io-model/NETWORK.md` §2 to §4.
- No contradiction with the constitution; no amendment to it.
- [x] Walked the rule table in `spec/derivation/derivation-ledger.md`: 140 current rules, every row read. The amendment adds SYS-15, SYS-16, SYS-17 and SYS-18 and amends SYS-2, SYS-4, SYS-5, SYS-6, SYS-8, SYS-10, SYS-11, SYS-14 and FN-7; main's v0.45 through v0.49 amended ENT-3, ENT-6, FN-8, FN-9, INV-1, GRAM-4 and PRF-1, and none of those names a resource, a permit, an entry input or a system operation, so no row conflicts. One stale sentence was found and corrected in the same amendment: SYS-6's `propagate` operation list still named the opens, whose outcome stopped being a `Result` when the permit was backed.
- [x] Rows whose status is `existence-only`, listed by ID rather than counted (55 in the current rule set; the ledger's statistics line says 56 and the table rows say 55, which the ledger's next sweep should settle): CALL-4 CAP-1 EFF-1 ENT-2 EX-1 FN-3 FN-5 FN-6 FN-7 FN-8 FN-9 FORM-1 FORM-2 FORM-3 FORM-4 FORM-5 FORM-8 GIVE-1 GRAM-10 GRAM-11 GRAM-2 GRAM-4 GRAM-5 GRAM-6 GRAM-7 GRAM-8 GRAM-9 INV-1 LEX-1 MSR-5 OP-1 OP-3 OP-5 OWN-1 OWN-11 OWN-12 OWN-13 OWN-14 OWN-3 OWN-7 OWN-8 PAR-1 PAR-2 PRE-1 PRF-1 STOR-3 STOR-4 SYS-1 SYS-14 SYS-16 SYS-2 SYS-7 TYPE-2 TYPE-5 TYPE-6

## Specification

- [x] Outgoing bytes archived as `spec/kernel-spec-v0.49.md`; v0.50 was free.
- [x] Conformance: eleven cases added (`sysin-*`, `systcp-*`), thirty-one respelled with their verdicts unchanged, five `systcp-*` verdicts moved from `unsupported` to `accept` when the operations became supported; generated syntax data unchanged (no production changes); tests and docs moved with the spelling changes.
- [x] The specification carries no commentary about its own versions.
- [x] [META-5] delta declaration and selection ground, stated here:
  - Delta of the first half (the backed file permit): numbered rules +0/-0; SYS-10, SYS-11, SYS-14 and the operation, target-contract and outcome tables modified; writer operation spellings +3 (`close_read`, `close_directory`, `close_directory_source`), system operations +3, declaration records +24; `reserve_file` answers `Result<FilePermit, IoError>` and the four opens answer `FileOpenOutcome`, `DirectoryOpenOutcome`, `SourceOpenOutcome`, whose failed variant carries the permit back.
  - Delta of the second half (streams and TCP): numbered rules +4/-0 (140: SYS-15, SYS-16, SYS-17, SYS-18); grammar productions +0/-0 (84); fixed lowercase grammar atoms +0/-0 (54); compound punctuation tokens +0/-0 (8); writer operation spellings +10/-0 and one respelled (`read_next`, `socket_address_v4`, `socket_address_v6`, `tcp_listen`, `tcp_accept`, `tcp_connect`, `receive_next`, `send_once`, `close_connection`, `close_listener` added; `reserve_file` respelled `reserve_handle`); opaque system nominals +5/-0 and three respelled (`InputStream`, `SocketAddress`, `TcpListener`, `TcpReceive`, `TcpSend` added; `Output`, `FileFactory`, `FilePermit` respelled `OutputStream`, `HandleFactory`, `HandlePermit`); struct system nominals +1/-0 (`TcpConnection`, a new category beside the opaque and enum ones: one nominal-type entry, two owner-local field records, no constructor entry); enum system nominals +3/-0 (`ListenOutcome`, `AcceptOutcome`, `ConnectOutcome`, six constructors with ten variant fields); runtime-trap families 0; entry forms 1; entry standard-input rows +1/-0 (ordinal 5 is `command.stdin`, ordinal 4 respelled `command.handles`); system operations +10/-0 (29) and declaration records +80/-0 (307). Changed rows: the six respellings wherever written; SYS-2's inventory, preorder and counts; SYS-4's no-split sentence; SYS-5's release table (+5 rows, a system struct takes no row); SYS-6's outcome table and `propagate` sentence; SYS-8's range-bearing set (+`read_next`, `receive_next`, `send_once`); SYS-10's permit accounting (+3 consuming operations, +2 returning closes); FN-7's standard-input table and canonical entry header. No rule id retired.
  - Selection ground: evidence-selected under T4, applied to every socket resource in `NETWORK.md` §2: a descriptor is one credit of the factory; a local port is the `SocketAddress` value the program binds, so two binds of one port are the program's own source-order conflict; an ephemeral port, the accept queue and the socket buffers are outside the program and each answers with honest target exhaustion or with the partial progress the sequential program already produces, so overlap invents no outcome. The pairing is on the API by the owner's decision of 2026-09-05 (`NETWORK.md` §3, §4, §8) rather than in a later `split`. The first half's ground is T4 applied to the file descriptor as a credit, with the cost of the missing relation measured at `compiler/src/backend/completion/contract.h`.

## Written at landing

- [x] `mcts_mem/whitefoot/system-interface.md` and `mcts_mem/whitefoot/parallelism.md` carry what was decided and why, including the three ring designs built and rejected on the control test (a ring per thread, a kernel submission thread, an armed multishot receive).
- [ ] n/a for archiving: `research/investigations/io-model/` stays, because the readiness-driven adapter for ring-less hosts, the data-terminated server loop and the scheduler shape the profile points at are its open designs, not landed capabilities.

## Citations

- [x] Support is the specification (SYS-10, SYS-15 to SYS-18, FN-7), the `sysin-*`/`systcp-*` conformance cases, the measured results in `research/experiments/io-completion-bench/`, `research/experiments/park-on-miss-measurements/`, `research/experiments/park-on-miss-switch-cost/` and `research/investigations/io-model/RESULTS.md`, the design in `research/investigations/io-model/NETWORK.md`, and the two `mcts_mem/` entries. The batch records under `archive/done/` are history, not evidence.
- [x] `research/` is cited for its studies and measurements, not as a description of the implementation.

## Gate

Reported stage by stage on `b21c0a3`, each stage run separately on the development host (Linux, root):

| stage | result |
| --- | --- |
| repository-invariants | pass (6 s) |
| spec-append-only | pass |
| spec-prose-integrity | pass |
| conformance | pass |
| compiler/format | pass |
| compiler/lint | pass (22 s) |
| compiler/test-partition | pass |
| compiler/test-unit | pass, 1500 (210 s) |
| compiler/test-sampling | pass (55 s) |
| compiler/test-corpus | bins, canonical corpus and programs 64 of 67: the three failures are the root-permission cases (`chmod 000` is no barrier to root), and the `snapshot` target the failing invocation did not reach was run on its own and passes |
| compiler/docs | pass (11 s) |
| compiler/spec | pass (19 s) |
| compiler/completion-test | pass (172 s), the TCP round trip on both routes included |
| research-tests | pass (8 s) |
| conformance-run | Pass=541, Xfail=1 (`ent5-neg-callee-uniq-buffer-replace-kills-length`, main's own), Skip=1 (51 s) |
| snapshot-run | Pass=491, Flip=0 |

- [x] The one stage that cannot run here in full is `compiler/test-corpus`, for the three root-permission cases above; CI's `corpus` job runs it as a non-root user and passed on this head (`gate` run 364, every job green on ubuntu-24.04 and macos-14; `io-hosts` run 364 green on Linux and the real Windows host; `io-bench` run 127 green).
- [x] No test was deleted, disabled, narrowed or unwired to reach green. Two tests changed in the last commits with their reasons in the commit: the reset test's peer now peeks one echoed byte before it closes so the close carries a reset on every host, and the sampling case reads the started-worker count from the core's own report instead of the grant count.

## What this branch carries

- Batch 1: `reserve_handle -> Result<HandlePermit, IoError>` from a factory with a real capacity; a refused open hands its permit back; three explicit closes return it; the descriptor retirement ledger and its probes deleted on every route and target.
- Batch 2: the scheduler core (`compiler/src/backend/sched/`) replaces both parallel runtimes and both writer schedulers; a join on I/O parks the stack; one submit-then-join lowering for every operation; the design's schedules enumerated under `completion-test`.
- Batch 3: `InputStream`/`OutputStream`, `HandleFactory`/`HandlePermit`, `SocketAddress`, `TcpListener`, `TcpConnection` and the seven TCP operations on io_uring, on the bounded POSIX adapter (Darwin) and on IOCP and Winsock in shared code; the staged hand-out of a may-suspend user call; the TCP echo control test against io_uring and epoll references with one load generator and one protocol, wired into `io-bench.yml`.

**Performance, development host, at this head.** TCP echo against the io_uring reference (medians of three recorded passes): 1.26 at one connection, 0.76 at 64, 0.64 at 1024, 0.82 on the 64 KiB payload; from 0.54, 0.11, 0.08 and 0.31 at the first reading. Compute, `par_layout` against `main` at 82f6d6a with each tree's own compiler: within two percent at one and two workers, eight to nine percent behind at four and eight (the compute-miss item of `research/experiments/park-on-miss-measurements/README.md` §2, still open). What the profile says the remaining network margin is, and the design that answers it, are in `mcts_mem/whitefoot/system-interface.md` and `RESULTS.md`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---

## #14 Loop bodies are region blocks; associative image joins: v0.43 candidate

State: closed | Merged: yes, 2026-09-04 | Head: batch/0120-loop-body-region | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

## Two amendments over v0.42 (both owner decisions of 2026-09-03)

**1. Every loop body is itself a region block ([OWN-11], [FORM-8], [OWN-3]).** A borrow of an outer binding inside a `for` or `loop` body is written bare and lives in the body's own unnamed per-iteration region, so [OWN-11]'s guarantee (a borrow formed in the body ends with the iteration; outer bindings are written again between iterations) is unchanged. A `region_stmt` that is a loop body's only statement restates what the loop already says and is a [FORM-8] rejection with the restructuring text "remove the region block", unless a type argument inside it must write its name (an implicit region has no name to put there). A block beside other statements is narrower than the body, carries [OWN-6]'s statement scope, and stays legal. A writer decides by reading the loop body alone.

```
for (i in 0_u64..3_u64) {
  region {                      // before: required by OWN-11
    let got = peek(x: &n);
  }
}
for (i in 0_u64..3_u64) {
  let got = peek(x: &n);        // after: the body is the region
}
```

**2. [ENT-6]'s control-flow join is associative.** Before comparing images at a join, every delta atom an earlier join minted is folded back into the interval it stands for; inputs sharing one non-delta form join to that form plus one fresh delta over the hull of their intervals. Nested conditionals now reach exactly the image one flat `match` over the same branches reaches, so acceptance no longer depends on the shape of the control join; delta atoms stay ordinary shared atoms between joins, so correlations formed after a join survive. Root cause and controls: a three-way demux with six header invariants was accepted as a flat `match` and rejected at [INV-1] as nested `if/else`, because the outer join compared `{h}` against `{h, d1}` and gave the binding a fresh atom.

## What changed

- Specification: v0.43 CANDIDATE over v0.42 (digest `1708dd2b…`), 132 rules, no rule added or retired; [OWN-3], [OWN-11], [FORM-8], [ENT-6] amended; META-5 delta names both amendments. No archive of v0.42 and no `ACTIVE-SPEC:` line; activation is a separate step. `governance/APPROVALS.md` carries the candidate record; the derivation ledger two rows.
- Compiler: the implicit body region is minted at `loop`/`for` in resolution; the checker's OWN-6 statement scope and OWN-11 containment generalised; the FORM-8 rejection of the sole-statement block; body-region slice loans ended on backedge, give and break edges; `join_affine_states` folds earlier deltas and takes the hull; `REVIEWED_FOR = "v0.43"`.
- Corpus: the four sole-statement loop region blocks in the test corpora removed; embedded Rust test sources respelled; research programs byte-identical.
- Conformance: six cases added (bare loop-body borrow accepted; sole-statement block rejected; a narrower named block inside a loop accepted; a borrow escaping the iteration still refused; the nested-conditional demux accepted; a genuinely non-preserving nested body still rejected); three cases modified with reasons in the manifest. Coverage 132/132.

## Evidence

- `cargo test --profile gate --lib`: 1490 passed. clippy `-D warnings` and `cargo fmt --check` clean.
- `make conformance-run`: Pass=512 Xfail=1 Skip=1 (the xfail is the recorded D1 case). `make snapshot-run`: Pass=491 Flip=0. `make compiler`, `make research-tests` green under the canonical `setpriv` wrapper.
- Canonical `make check` stops only at `spec-archive-integrity` with the expected CANDIDATE message; the `static` CI job is red for that reason until activation, as for every candidate before it.

## Merge record

The merge needs the rule-4 specification record (candidate bytes at `1708dd2b…`, or the activation digest if activated first) and conformance record (6 cases added, 3 modified, 4 respelled) in `governance/APPROVALS.md`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #15 Design: record the rulings on S31 to S33 (docs only)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0123-design-s31-s33 | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

Documentation only, under `research/investigations/containers-and-resources/`. Records the owner's decisions of 2026-09-04 on the three items the eighth draft left proposed:

- **S31 not adopted as an operation.** Forming a shared `slice` from a `mut_slice` is the ordinary shared child reborrow of a unique loan that [OWN-6] already admits for places. Verified on the v0.42 build: `peek(x: &deref(x))` with `x: &uniq u64` inside a region block is accepted. [VIEW-6] now states that rule applied to views (parent's origin set and range, shared child loan, parent frozen while the child lives); the `seq_reslice` row is gone from the kernel vocabulary, so the fill-and-publish helper is writable without a new row.
- **S32 adopted.** A linearity bound on a generic parameter (`fn f<T: affine>`, `fn f<T: linear>`, `fn f['s: affine]`), read by [PROV-6] and [BLK-4]; one declaration keeps one verdict.
- **S33 adopted.** `reserve_file` returns a three-variant `ReserveOutcome` (`Reserved`, `Exhausted`, `Failed`) in place of the `Result` form, so the handle table's exhaustion publishes `room(factory) = 0` on the `Exhausted` arm only.

The proposed list in §3.S is now empty. §7's test lists for B5, B7, B8 and B10 are updated; B1 (in progress) is unaffected.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #16 Activate kernel specification v0.43 (restores main's static gate)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0122-activate-v043 | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

## Why

PR #14 merged the v0.43 amendments as a CANDIDATE specification, so `main`'s `static` gate stops at `spec-archive-integrity` ("CANDIDATE status is valid branch work but is not a merge-ready ACTIVE identity"). This PR is the activation, done exactly as the v0.42 activation (commit `c168ab2a`) was.

## What changed

- `spec/kernel-spec.md`: line 3 flips to `Status: ACTIVE v0.43`; no other byte changes. Identity `037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`.
- `spec/kernel-spec-v0.42.md`: the outgoing ACTIVE v0.42 bytes (`6b935d2e…`, the spec at `285594ce`), archived byte-for-byte.
- `governance/APPROVALS.md`: the v0.43 record becomes the activation record (candidate digest `1708dd2b…`, activation digest, superseded digest, archive path, the META-5 delta as the header declares: two amendments, no rule added or retired, 132 remain) and the chain gains `ACTIVE-SPEC: v0.43 037c9e69… 6b935d2e…`.
- `compiler/src/spec_identity.rs` regenerated by `whitefoot-spec --emit-identity` (chain length 35); `compiler/src/spec.rs` digest updated.
- README, compiler README, `docs/current-plan.md`, `docs/roadmap.md`, `docs/patterns.md`, `spec/derivation/derivation-ledger.md`: v0.43 named as active, v0.42 as archived.

## Evidence

- `make spec-append-only spec-archive-integrity spec-digest-sync approval-history-integrity repository-invariants`: all pass (44 recorded specifications hash as recorded; chain tail v0.43).
- `cargo test --profile gate --lib`: 1490 passed.
- Canonical `setpriv … make check` on this revision: `== WHITEFOOT ALL TESTS GREEN ==` (spec stage reports v0.43, 132 rules, 35 unbroken activations; conformance coverage 132/132).

## Merge record

The activation record already in the branch is the rule-4 specification record for this merge (exact bytes at `037c9e69…`).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #17 B1 fact machinery: v0.44 CANDIDATE (needs activation before merge)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0121-fact-machinery | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

## What this is

Batch B1 of the merged container-and-resource design (`DESIGN.md` §7): the four fact-machinery rules, as a **v0.44 CANDIDATE** over v0.43. The `static` CI job is red by design until activation; please do not merge before I activate it (activation is a separate commit on this branch once you say the content is fine), so `main` does not carry a candidate again.

## The four rules

- **[MSR-5] contract operands.** `requires` and `ensures` take a `clause_expr` whose operands may be calls, so `len(P)` derives directly on either side of a comparison for every place [ENT-2] admits a length term for. Before this, `requires len(short) <= len(long);` was a [GRAM-5] parse error and the only spelling was one `define` per measure. Bool-rooted clauses (`requires ok;`, `.defined`, predicate calls) keep working. Header and local invariants keep their v0.43 atom set (widening them is B2's [MSR-4]).
- **[MSR-3] denotation by mode.** An `own` operand of an established relation denotes the call datum: a compiler-owned immutable term minted at the transfer point, so a callee's `ensures` over a consumed operand reaches the caller. A `&uniq` parameter's measure is inadmissible in a source-declared `ensures` (this is the door D1 walked through; the diagnostic names the restructuring).
- **[CALL-4] routes.** The single-result route as the language has it today; the measured-result, result-measure and any-variant routes are DEFERRED with their deltas (they need multi-return and B7's runs).
- **[CALL-6] publication, with [ENT-3.S13] call datums.** One statement of how a declared relation is instantiated at the call, established on the normal continuation, restricted to its routed arm; and a refusal of a contract block whose published relations are contradictory at their establishment point.

## Evidence

- Six conformance cases (each verified on the built binary): `msr5-pos-two-measure-clause`, `msr3-pos-own-operand-call-datum`, `msr3-neg-uniq-state-measure-in-ensures`, `call6-neg-contradictory-published-relations`, `call6-pos-routed-relation-over-a-call-datum`, `call4-neg-measured-result-not-admitted` (the design's probe q7, now a semantic refusal instead of a parse error).
- `cargo test --profile gate --lib`: 1490 passed; clippy `-D warnings` and fmt clean; `make conformance`: coverage 136/136; `make conformance-run`: Pass=518 Xfail=1 Skip=1; `make snapshot-run`: Pass=491 Flip=0; `make compiler` and `make research-tests` green; canonical `make check` stops only at the candidate's expected `spec-archive-integrity` stop.

## Corpus consequence worth knowing

`wfgrep.wf` and `raw_deflate_boundary.wf` published a bound through a `&uniq` measure (`ensures result <= capacity` with `capacity = len(deref(destination))`), exactly the shape [MSR-3] now refuses. Both now take `capacity: own u64` with `requires capacity == len(deref(destination));`. No behaviour, diagnostic, or exit code changes. The entailment proof-cost ceilings rise from 30,000 to 40,000 nodes and 58,000 to 75,000 edges (wfgrep measures 36,760 and 71,153), linear in the number of calls publishing a relation over an `own` operand.

## Design defects this batch exposed (to fix in DESIGN.md, not here)

1. The design's [MSR-5] production (`affine_expr compare_op affine_expr`) would drop every Bool-rooted clause the corpus writes; the implementation widens the operand set instead.
2. [MSR-3]'s table row says any `&uniq` operand is inadmissible in an `ensures`; its Judgment line says the `&uniq` parameter's *measure*. The measure reading is implemented.
3. §7's B1 test list assumes multi-return and B7's runs for four of its tests.
4. S13's population is call datums until B7's declared rows exist.

## Merge record

Specification and conformance content change; the candidate record is appended in `governance/APPROVALS.md` (no existing record edited). Activation will add the archive of v0.43 and the chain line.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #18 Record B1's landed form in the containers design (docs only)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0124-design-b1-defects | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

Docs only: `research/investigations/containers-and-resources/DESIGN.md` (+206/-72) and `CONTAINERS.md` (+5/-2). No spec, compiler, test, or conformance content changes.

B1 (the v0.44 candidate, [PR #17](https://github.com/mbbill/Whitefoot/pull/17)) implemented [MSR-3], [MSR-5], [CALL-4] and [CALL-6] in a narrower form than the eighth draft wrote, and exposed four defects in the design text. This PR brings the design to what landed, each as a dated correction at its rule:

- **[MSR-5]'s production.** The draft's `clause_expr := affine_expr compare_op affine_expr` has a comparison at the root and nothing else, so it would drop every Bool-rooted clause the corpus writes today (`requires ok;`, `requires band(nonzero, not_neg1);`, `requires buffer_fits::<T>(length);`, `requires total /defined steps;`) and contradicts [FN-8]'s retained `.defined` admission. The text now carries the production that landed, with the Bool root carried by the [OP-5] judgment. The affine widening stays [MSR-4]'s in B2.
- **[MSR-3]'s table row versus its Judgment line.** The row said "naming a `&uniq` parameter"; the Judgment said "a `&uniq` parameter's measure". The measure is what landed; the row now agrees, and a non-measure `&uniq` operand such as `deref(p).count` is recorded as still admissible, with the L11 reason.
- **[CALL-4]'s deferred admissions.** A result of measured type, a measure over a result place, and a route over any variant of any returned enum are recorded as B7's. [S16]'s ordered result list and the destinations that read it go to a new **B1b** entry in section 7.
- **[ENT-3.S13]'s population.** No v0.44 compiler-owned row carries a declared relation set, so the draft's population was empty at the tip. What landed is the call-datum substitution half at ordinary source calls; B7 extends the population to [BLK-0] rows without reusing the label.

Section 6 gains **6.0 B1 landed** listing the six conformance cases with their verdicts, and the corpus consequence: `wfgrep.wf` and `raw_deflate_boundary.wf` now take `capacity: own u64` with `requires capacity == len(deref(destination));`.

Verified before opening: the six case names and their three rejection rules (MSR-3, CALL-6, FN-9) match `tests/conformance/manifest.jsonl` on the B1 branch; the four corpus clauses quoted above exist verbatim in the tree.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #19 Activate kernel specification v0.44 (turns main's static gate green)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0125-activate-v0.44 | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

## Why

[PR #17](https://github.com/mbbill/Whitefoot/pull/17) merged the v0.44 **candidate** to `main`, so main's `static` job now stops at `spec-archive-integrity` ("CANDIDATE status is valid branch work but is not a merge-ready ACTIVE identity"), exactly as after #14. This PR is the activation, the same shape as [PR #16](https://github.com/mbbill/Whitefoot/pull/16) for v0.43. It changes no language content.

## What changes

- **`spec/kernel-spec.md`**: the status line flips from `CANDIDATE v0.44 supersedes v0.43 037c9e69…` to `ACTIVE v0.44`. No other byte changes. Active SHA-256: `5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049`.
- **`spec/kernel-spec-v0.43.md`**: the outgoing v0.43 bytes, archived byte-for-byte (`037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`, verified with `sha256sum` against the pre-#17 main head).
- **`governance/APPROVALS.md`**: one **appended** activation record plus the `ACTIVE-SPEC: v0.44 5ef144bf… 037c9e69…` chain line. The v0.44 candidate record and every earlier record are untouched (the gate's "existing main records are an exact prefix" check passes).
- **`compiler/src/spec.rs`, `compiler/src/spec_identity.rs`**: embedded identity and chain length (36), regenerated with `whitefoot-spec --emit-identity`.
- **Docs**: `README.md`, `compiler/README.md`, `docs/current-plan.md`, `docs/roadmap.md` (revision 68), `docs/patterns.md` (P16, P21 no longer "candidate"), `spec/derivation/derivation-ledger.md` name v0.44 as the active authority. The ledger's v0.44 section is bound to the activated bytes; its candidate-time preamble said "one derived and three existence-only" while its own table has two derived (MSR-3, CALL-6) and two existence-only (MSR-5, CALL-4), so the preamble is corrected and the statistics line reads 81 derived · 55 existence-only · 0 underived over 136 rules.

## Merge record

Specification content changes (the status line) and no conformance content changes; the appended record in `governance/APPROVALS.md` is the rule-4 record for this merge.

## Verification

Canonical `make check` on this exact revision, private cargo target dir, passes end to end:

```text
approval history integrity: existing main records are an exact prefix
spec append-only: no released kernel specification was modified or removed
spec archive integrity: 45 recorded specifications hash as recorded
spec digest sync: live prose quotes the chain tail (v0.44 5ef144bf…)
conformance adapter: Pass=518  Xfail=1  Skip=1
snapshot corpus: Pass=491 Flip=0
```

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #20 Retire the candidate spec state and stop quoting the identity in prose

State: closed | Merged: yes, 2026-09-04 | Head: batch/0126-gate-simplify | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

## Why

The v0.43 and v0.44 candidates each merged to `main` as candidates and each needed a zero-content activation PR ([#16](https://github.com/mbbill/Whitefoot/pull/16), [#19](https://github.com/mbbill/Whitefoot/pull/19)) to turn main green, and every activation forced six documents to re-quote a digest. The owner asked for both to go. No language content changes.

## What changes

**1. No candidate state.** `spec/kernel-spec.md` always declares `Status: ACTIVE vN` and hashes to the chain tail. An amendment lands as one change on a branch: the amended file, the archive of the outgoing bytes, the appended approval record with its `ACTIVE-SPEC:` line, and the regenerated `compiler/src/spec_identity.rs`. The owner's merge approval of that revision is the activation.

- `Makefile`: `spec-archive-integrity` requires the ACTIVE status line; its candidate branch and the `spec-candidate-integrity` alias are removed.
- `compiler/src/bin/spec.rs`: parses only `Status: ACTIVE vN`; the candidate variant, the successor-version arithmetic, and their five tests go with the state they tested.

**2. Prose stops quoting the identity.** The chain tail in `governance/APPROVALS.md` and the generated `spec_identity.rs` are the two machine-checked homes of the version and digest.

- `spec-digest-sync` (which required six files to quote the digest and an "active vN" sentence) is replaced by `spec-prose-integrity`, a negative check: live prose may not quote a 64-hex digest or name a version as the active authority. The derivation ledger's per-version amendment bindings are frozen history, so the ledger is held to the phrase check only. The retirement reason is in the target's comment.
- README, compiler/README, WORKFLOW, current-plan, roadmap (revision 69), patterns, and the ledger header now point at the chain. WORKFLOW's "Specification identity" section states the one-change procedure.
- `.github/workflows/gate.yml` static stage runs the new target.

**Also:** `compiler/src/spec.rs` derives `ACTIVE_KERNEL_SPEC_VERSION` and `ACTIVE_KERNEL_SPEC_HASH` from `spec_identity.rs` through a compile-time hex decode, so no identity is hand-typed anywhere in the crate. The hand literal's test is retired with an explanation in place: the independent `shasum` measurement still enters through the approval record, which the chain check compares against the computed digest.

## Verification

Canonical `make check` on this revision, private cargo target dir, passes end to end; clippy `-D warnings` clean.

```text
approval history integrity: existing main records are an exact prefix
spec append-only: no released kernel specification was modified or removed
spec archive integrity: 45 recorded specifications hash as recorded
spec prose integrity: live prose quotes no specification digest and names no version as the active authority
conformance adapter: Pass=518  Xfail=1  Skip=1
snapshot corpus: Pass=491 Flip=0
```

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #21 Design: retire array (S34) and capitalize the view names (S35)

State: closed | Merged: yes, 2026-09-04 | Head: batch/0128-design-array-views | Base: main | Created: 2026-09-04 | Closed: 2026-09-04

Docs only: `research/investigations/containers-and-resources/DESIGN.md` (+110/-79). No spec, compiler, or test content changes; the implementation lands with B7/B8 on the containers branch.

Two surface decisions the owner took on 2026-09-04 after asking why `array<T, n>` had survived the container redesign:

- **S34: `array<T, n>` retires**, with its `array_new` row. It was exactly the `len = cap = n`, `head = Z` case of a run, which appendix A.1 already tabulated as four exact constants. A `FixedVector<T, n>` whose four measures are standing facts is that case with no runtime descriptor word, so a `const` of `FixedVector<T, n>` type with exactly `n` literal entries is the const-eligible form, lowers to element storage only, and materializes its descriptor at each use. One fixed run, one spelling.
- **S35: every compiler-owned container, store and view nominal is capitalized**: `Vector`, `FixedVector`, `Heap`, `Arena`, `Slice`, `MutSlice`. Only the primitive types stay lowercase. Supersedes S5's kept name and S6's spelling; `seq_slice` / `seq_mut_slice` are unchanged.

Updated: the 3.S table and grounds (with a note on why the omission survived seven falsifier rounds: none asked whether the old surface was fully replaced), 3.K.10's naming table, [BLK-1], the A.1 measure and layout tables, and the amendment register ([TYPE-2], [OP-7], [CONST-1]). Every normative section now writes `Slice` / `MutSlice`; section 6 keeps the old spelling in its probe quotes, and the one v0.41 [ENT-2] quotation in [MSR-1] keeps `slice` as quoted.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---
_Generated by [Claude Code](https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5)_

---

## #22 Binary arithmetic in the proof surface (v0.45–v0.49), and the workflow that carried it

State: closed | Merged: yes, 2026-09-06 | Head: batch/0129-binary-arithmetic | Base: main | Created: 2026-09-05 | Closed: 2026-09-06

Two separable bodies of work. **v0.45–v0.49** widen the proof surface so a
written certificate can scale a premise by a value; the commits after
`519a228` change no language at all and retire the process artifacts this
branch kept tripping over.

**Answered at: `76adab5`** — CI green, 14/14 on `ubuntu-24.04`, `macos-14` and
`windows-latest`.

---

## A. Language change — v0.45 through v0.49

| version | change |
| --- | --- |
| **v0.45** | `[ENT-3.S14]` publishes the interval an admitted product already proved |
| **v0.46** | a clause states an affine relation, `len(P)` is an affine atom, and the affine route needs no L0 projection |
| **v0.47** | an integer-typed named const is an affine atom |
| **v0.48** | a `use` cites one premise, and its multiplicity may name an unsigned value |
| **v0.49** | the fold names the declaration a writer wrote, not what it expands to |

`use stride times (row < height);` now compiles with a **computed** stride,
which is what every real stride is. Held to one shape with only the derivation
varying:

| `stride` | v0.48 | v0.49 |
| --- | --- | --- |
| `let stride = stride_src;` | accept | accept |
| `let stride = stride_src + padding;` | reject | **accept** |
| `let stride = stride_src + 4_u64;` | reject | **accept** |
| `let stride = 2_u64 * stride_src;` | reject | **accept** |

An operand and a multiplicity name the same value when they name the **same
declaration**, not when their images coincide — the rule PRF-1 already applies
to a named premise one sentence away. Each such binding contributes one opaque
handle that lives only between the fold and the residual.

Two other designs were built first and both are the obvious first idea.
Publishing the handle's defining equality as a fact is invisible to the
residual, which is the direct L0 route by rule. Replacing the binding's image
with the handle makes every reader agree and costs the other side: an ordinary
premise about that binding then needs the equality, and two of the four rows
above moved from a fold failure to a premise failure. Reading handles at the
*domain* site was tried too and costs the interval, failing the
multiplication's own OP-2.

### Derivation

**Premise: R3** — one way to say anything, and the survivor is chosen by
evidence. The `*` spelling claimed the multiplicity was a multiplication whose
right operand is a relation; `times` is evidence-selected, with zero identifier
uses in the corpus and four of its fifteen doc-string appearances being the
corpus explaining this very construct in prose. Mandatory parentheses remove
the second spelling split from the same ambiguity. **Premise: R4**
(shift-left) for the surface itself — the interval S14 publishes was already
computed and thrown away, so a writer was made to restate by hand what the
checker had proved.

### Ledger sweep — and what it actually found

Not a clean "walked all 143 rows". The honest result, because the checklist
that demands this sweep was written *after* this language work landed:

- **Checked**: the rows that can interact with an affine layer gaining atoms
  and a certificate gaining a term multiplicity — `ENT-2`, `ENT-5`, `OP-2`,
  `OP-4`, `MSR-3`, `MSR-5`, `CALL-6`. No conflict. `ENT-5`, `OP-2` and `OP-4`
  are `derived`, and the amendments were built *against* their stated premises
  rather than around them: the measure atom's stability is argued from ENT-5's
  kill rule, and v0.49 deliberately keeps transparent images at the domain site
  **because** handles there fail OP-2.
- **Learned nothing**: `ENT-2` and `MSR-5` are `existence-only`. So are all
  four rules this batch amended most — `GRAM-4`, `FN-8`, `INV-1`, `PRF-1`. A
  row that states no premise cannot be contradicted, so the sweep is silent
  exactly where this change is loudest.
- **Not done**: the remaining rows were not read one by one against these
  amendments.

The ledger holds 86 `derived` rows and 57 `existence-only`. The sweep is worth
what the ledger is worth, and re-deriving those 57 is registered as the next
piece of work, not this one.

---

## B. Everything else — no language change

The branch kept hitting the same defect in its own process, so the process
changed. None of this touches `spec/`, the grammar, or what the checker
accepts; it is decided against the project goal and priority order in
`CLAUDE.md`.

**A real defect hid behind a failing gate stage for fifteen commits.** `make
check` stops at the first failure, `test-sampling` was failing on a missing
ASan runtime, and `test-corpus` — the stage after it — never ran here. Behind
it: `times` is the ninety-ninth fixed terminal, and `ALL_TERMINAL_PREDICATES`
was sized 106 with its external predicates at hardcoded slots 98..=105, so
`predicates[98] = Identifier` overwrote `Fixed(Times)`. CI had been red on both
hosts the whole time and nobody was reading it. Fixed, made structurally
impossible, and pinned by two tests verified against an injected off-by-one.

Then the artifacts that caused the rest of it:

| retired | why |
| --- | --- |
| `docs/current-plan.md` | its plan was delivered at v0.40 and never replaced, so it stayed "current" by having each later version appended to it |
| `docs/done/` (100 records) | a finished task is not evidence; every live citation of one is removed rather than repointed |
| `governance/APPROVALS.md` (3.1k lines) | the async approve-then-execute loop is gone and its prose is read by nothing |
| the `ACTIVE-SPEC:` chain | verified a total order no decision consumed, and made two concurrent specification branches structurally impossible |
| `spec_identity.rs` + its freshness test | a committed second copy of a fact the crate already embeds; `build.rs` derives it now |
| version commentary in `spec/kernel-spec.md` | the v0.30 rework evicted twenty-one such paragraphs; this branch had put five back |

Added: `.github/pull_request_template.md`, `compiler/build.rs`,
`docs/practice.md` (the non-workflow half of the old `WORKFLOW.md`), and two
facts plus a move in `mcts_mem/`.

Net ≈ −3800 lines of process. The gate goes from ten stages to eight.
**Concurrent specification branches are now possible**: two branches archive
the same outgoing bytes under the same name, so only the version number
collides and the second to merge retitles two lines and rebuilds.

One theme runs through all of it: **the same fact kept having two homes**. The
version prose lived in the specification header, the roadmap, the plan document
and the approval record at once. The terminal inventory was written twice and
the two copies collided. The specification identity was a committed file
policing a copy of bytes the crate already had. Each fix removes the copy
rather than synchronizing it.

---

## Specification

- [x] Outgoing bytes archived through `spec/kernel-spec-v0.48.md`; the identity
      needs no action, `build.rs` derives it from the bytes. Verified: appending
      one byte changes the reported digest with no other step.
- [x] Derived material moved with it — conformance cases and verdicts,
      generated syntax data, tests, docs.
- [x] The specification carries no commentary about its own versions.
- [x] The delta declaration and selection ground are stated above, per the
      amended `[META-5]`.

## Written at landing

- [ ] **`mcts_mem/` is not updated for the five amendments.** `checks-and-proofs.md`
      newest entry is 2026-08-05 and `surface-form.md` is 2026-08-09. This is a
      real gap and it is the reason the rule now exists; it is queued, not done.
- [x] `docs/roadmap.md` is out of the working loop, so nothing was owed to it.
- [ ] `research/investigations/binary-arithmetic/` not archived — the capability
      lands with this merge, so the archive move belongs to the merge.

## Citations

- [x] Nothing cites a finished task as evidence.
- [x] Nothing cites `research/` as a description of the current implementation.

## Gate

**CI is the answer: 14/14 green at `76adab5`** — `static`, `unit`, `sampling`,
`corpus`, `conformance`, `research` on both `ubuntu-24.04` and `macos-14`, plus
`completion-linux` and `completion-windows`.

Locally, `make check` stage by stage:

| stage | result |
| --- | --- |
| `repository-invariants`, `spec-append-only`, `spec-prose-integrity` | pass |
| `conformance` | pass — 136/136 rules |
| `research-tests` | pass |
| `conformance-run` | pass — 528 / 1 pre-existing xfail / 1 skip |
| `snapshot-run` | pass — 491 / 0 flips |
| `compiler/format`, `lint`, `test-partition` | pass |
| `compiler/test-unit`, `test-sampling` | pass |
| `compiler/test-corpus` | **3 failures**, one cause, below |
| `compiler/docs`, `spec`, `completion-test` | pass |

- [x] The three failures are this container, not the tree: each sets a file or
      directory to mode `000` and asserts `permission denied`, and the container
      runs as UID 0, which the kernel exempts. They fail identically on a build
      from before this batch, and they pass in CI, which is not root.
- [x] No test was deleted, disabled, narrowed, or unwired to reach green. Six
      activation-chain tests and four conformance-runner tests were removed with
      the mechanism they exercised, and the commits say so.

**Running the local stages individually is not equivalent to CI, and this
branch proved it.** Retiring two gate stages left `.github/workflows/gate.yml`
naming them in its own hardcoded copy of the list, so `static` failed on both
hosts while every local stage was green — my gate script was a third copy of
that list and had been updated, which is precisely why running it reproduced
the oversight instead of catching it. `76adab5` gives the list one home: a root
`static` target the workflow calls.

## Adversarial validation

Eight parallel sweeps ran against the landed work. Every finding was re-derived
before being acted on, and **four of the agents' conclusions were wrong**: a
CONST-1 mis-citation reported as inherited was introduced by this batch; a
stack overflow reported as new is inherited; a reported `+`/`−` asymmetry
dissolved once three probes were held to one target; one report described a
defect already fixed.

**No unsoundness was found.** A genuine out-of-bounds chain was built to check
the hunt's proxy assumption, and the sharpest attack — a target naming a
product formed under an operand's old value — rejects correctly.

**Four things this batch itself had written were false and are corrected**,
including one specification sentence withdrawn because measurement could not
reach the case it described.

Two pre-existing defects are recorded in `compiler/README.md` rather than fixed
here, both dated against a pre-v0.48 build.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP

---

## #23 Record v0.45 through v0.49, and correct four artifacts that read as current and are not

State: closed | Merged: yes, 2026-09-06 | Head: batch/0130-decision-memory | Base: main | Created: 2026-09-06 | Closed: 2026-09-06

Five commits, one shape. Each touches an artifact a reader takes as a
description of the present, and each was describing something else.

**`mcts_mem/` had no entry for v0.45 through v0.49 at all.** The merged batch
flagged that against itself: `checks-and-proofs.md` stopped at 2026-08-05 and
`surface-form.md` at 2026-08-09 while the whole proof surface moved underneath
them. `certificate-fold` lands as its own node rather than a bullet, because
that decision has three alternatives that were actually built and a measured
table separating them; each rejected one keeps its own node under
`certificate-fold.alt/` with the measurement that killed it.

**The pull-request template stated something false, and I am the one who put it
there.** It said an `existence-only` ledger row "states no premise, so nothing
can conflict with it and the sweep learns nothing there", and told every future
change to list such rows by ID instead of reading them. Measured against this
tree:

```
 58  rows carry the existence-only marker (the ledger header's 56 excludes
     CLM-1 and CLM-3, retired at v0.40)
  0  have a Notes field shorter than 60 characters
337  characters is the median Notes length
 31  name an explicit condition under which the row would be promoted, 29 of
     them rules live in v0.50
```

The status separates a derived need from a minimality-selected form. It does
not mark an empty row. `[PRF-1]` holds its form "until wider source compares
proof length, diagnostic quality, and missing expressivity"; `[INV-1]` "until
broader real programs compare their writing cost". v0.45 through v0.49 produced
wider source, measured proof length, and a documented missing-expressivity case
in the derived stride, and recorded none of it against either row. The template
now asks two questions at such a row instead of one.

**`mcts_mem/whitefoot/development-workflow.md` had the same defect one level
down.** The 2026-09-06 ruling landed there as a Fact while the standing rules
above it still routed what landed to `governance/APPROVALS.md` and language
change to a guarded branch in `docs/WORKFLOW.md`, both retired by that same
ruling. The rules now state the loop that runs, and the drift is recorded as
its own pitfall: appending a Fact is easy and rewriting a rule is not.

**Two Moves named a node that did not exist.** `operation-spelling` records
replacing `multiplied-use-star`, `development-workflow` records replacing
`rolling-current-plan`, and neither replaced design had a node, so the reason
each was rejected lived only inside the sentence that retired it. A Move
pointing at nothing records that something was rejected without recording what.
Both are written from evidence that already existed rather than recalled. All
117 `[[link]]` targets in `mcts_mem/` now resolve.

**Three corpus cases were failing for a reason they did not state.** They set a
fixture path to mode 000 and assert the walking program reports it unreadable.
Mode bits do not deny a process holding `CAP_DAC_OVERRIDE`, which uid 0 carries
by default, so under a container that runs the gate as root the scenario is
unconstructible: the walk reads the path it was told was closed and the case
fails on its output comparison. That failure reads like a traversal defect and
is not one, which is how three permanently red cases become background noise.
`close_path` now confirms the denial actually reaches this process, so the
failure names its cause and its remedy. The same check covers the opposite
risk: without it a case could report success in an environment where the path
it describes was never closed.

This is not a skip and it does not make the gate green here. Verified both ways
on this tree: as uid 0 all three fail at the precondition with the new message,
and the same test binary under `setpriv --reuid=65534` passes all three.

**Answered at: `f8965cb3`**

## Derivation

### B. Everything else — derivation is required, completeness is not

Nothing here touches `spec/`, the grammar, or what the checker accepts.
`git diff --name-only origin/main...HEAD` returns no path under `spec/`, so
class A and its ledger sweep do not apply.

Why this: priority 3 in `CLAUDE.md`, keep the implementation understandable and
easy to change. An artifact read as description is where a false statement
costs the most, because no gate covers prose and the next reader spends the
error rather than finding it. Every repair here is the same move — make the
artifact say what is true now — and none adds a mechanism, a script, or a file
that has to be maintained afterwards.

The template change specifically: a checklist item that instructs a reader to
skip is worse than no item at all, because it converts a real check into
recorded ceremony. That is the failure mode the template's own preamble names,
so the item was working against the document it lives in.

## Specification

- [x] `spec/kernel-spec.md` unchanged. No path under `spec/` is touched; the
      merge of `origin/main` brings v0.50 in from #13 and this branch adds
      nothing to it.
- [x] n/a — nothing derived from the specification moved, because the
      specification did not move here.
- [x] n/a — same reason.
- [x] n/a — [META-5] declares a spec delta and this change has none.

## Written at landing

- [x] `mcts_mem/` updated. `checks-and-proofs.md` gains a standing rule and
      five Facts, two of them method rather than semantics because both cost
      the last batch real work; `surface-form.md` gains the `times` rule and
      three Facts; `certificate-fold` and its three built-and-rejected
      alternatives land as their own nodes; `development-workflow.md` gains the
      correction above and the drift pitfall; `multiplied-use-star` and
      `rolling-current-plan` recover the two nodes that were only referenced.
- [ ] **Not done, and this is an honest "no" rather than an "n/a".**
      `research/investigations/binary-arithmetic/` studied the capability that
      landed as v0.45 through v0.49, so by this line it should now be archived.
      It is not, because `spec/kernel-spec-v0.45.md` through `-v0.48.md` cite
      that path and released archives are immutable. Archiving the
      investigation would leave four immutable files pointing at nothing. How
      an archived specification's citations survive the archiving of what they
      cite is a decision this change should not make on its own.

## Citations

- [x] Nothing here cites a finished task as evidence. The support is the
      specification, the measured tables under
      `research/investigations/binary-arithmetic/evidence/`, and the ledger
      rows quoted above.
- [x] Nothing cites `research/` as a description of the current implementation.

## Gate

`make check` on `f8965cb3`. Each stage was invoked separately rather than
through `check`, because `check` stops at the first failure and would have left
the six stages after `test-corpus` unrun and unreported.

| stage | result |
| --- | --- |
| `repository-invariants` | pass |
| `spec-append-only` | pass |
| `spec-prose-integrity` | pass |
| `conformance` | pass |
| `compiler/format` | pass |
| `compiler/lint` | pass |
| `compiler/test-partition` | pass |
| `compiler/test-unit` | pass |
| `compiler/test-sampling` | pass |
| `compiler/test-corpus` | **fail** — 64 passed, 3 failed |
| `compiler/docs` | pass |
| `compiler/spec` | pass |
| `compiler/completion-test` | pass |
| `research-tests` | pass |
| `conformance-run` | pass |
| `snapshot-run` | pass |

- [x] The one failing stage is named, with the reason and where it passes.
      `compiler/test-corpus` fails on exactly the three denied-path cases, at
      the precondition this change added, because this container runs as uid 0
      and mode bits do not deny a process holding `CAP_DAC_OVERRIDE`.
      Established three independent ways rather than assumed: the mechanism
      demonstrated directly here (a mode-000 file is still readable), the same
      test binary passing all three under `setpriv --reuid=65534`, and CI green
      on this same SHA — `gate` and `io-hosts` both succeed on `f8965cb3`.
- [x] No test was deleted, disabled, narrowed, or unwired to reach green. The
      denied-path change moves three cases' failure from an output comparison
      to a named precondition; they still fail here, and they still verify the
      same behavior wherever the precondition can hold.

---
_Generated by [Claude Code](https://claude.ai/code/session_0162KgsKK2YdTw6XqEEcgNSP)_

---

## #24 Batch 0108: streams, TCP, and scheduler core refactor

State: closed | Merged: no (closed without merging) | Head: main | Base: handoff/2026-08-28-cloud | Created: 2026-09-06 | Closed: 2026-09-06

Implement streams and TCP operations (specification v0.46), refactor the completion I/O runtime to use park-on-miss scheduling, and reorganize the backend to separate scheduler concerns from completion adapters.

**Answered at: draft**

## Derivation

### A. Language change

This change amends `spec/kernel-spec.md` to v0.46, adding standard streams as `InputStream` and `OutputStream`, the descriptor factory as `HandleFactory`, `SocketAddress`, `TcpListener`, `TcpConnection`, and seven TCP operations under T4. This is the first API that waits on completion.

- Constitutional premise: T1 (target operations), T2 (target semantics), T3 (target proofs). The amendment derives from the need to expose network I/O as a first-class capability with the same proof-carrying completion model as file I/O.
- No constitutional contradiction. The amendment extends T4 operations without changing existing rules.
- [x] Walked the rule table in `spec/derivation/derivation-ledger.md` and found no conflict. Rows 0106–0108 are new; existing rows remain consistent with the TCP operation semantics.
- [x] No `existence-only` rows apply to this amendment.

### B. Everything else

**Why this change:**

Batch 0108 completes the park-on-miss I/O model (batches 0106–0107) by adding TCP operations and refactoring the runtime to eliminate the writer scheduler and completion slot pool. The record is now a block of the submitting frame; the engine finds it by address and publishes completion directly into it. This simplifies the runtime, removes a source of complexity, and enables the first waiting API.

The backend reorganization separates scheduler concerns (`compiler/src/backend/sched/`) from completion adapters (`compiler/src/backend/completion/`), making the architecture clearer and the code easier to evolve.

**Rationale:** The project goal is to reach meaningful end-to-end language experiments. TCP operations and the simplified scheduler core unblock real network programs and reduce the runtime surface that must be verified.

## Specification

- [x] `spec/kernel-spec.md` archived as `spec/kernel-spec-v0.45.md`; new version is v0.46. Identity derived by `compiler/build.rs` from the bytes.
- [x] Conformance cases and verdicts moved with the specification in `tests/conformance/`.
- [x] The specification carries no commentary about its own versions.
- [x] Delta declaration and selection ground stated in `archive/done/0108-streams-and-tcp.md`.

## Written at landing

- [x] `mcts_mem/` updated with decisions on the park-on-miss model, TCP operation semantics, and the scheduler/adapter split.
- [x] `research/investigations/io-model/PARK-ON-MISS.md` archived; the capability has landed.

## Citations

- [x] Support comes from the specification (v0.46), conformance cases, and design decisions in `mcts_mem/`.
- [x] No research artifacts cited as descriptions of the current implementation.

## Gate

| stage | result |
| --- | --- |
| compiler build | green |
| format and lint | green |
| semantic check | green |
| backend tests | green |
| conformance adapter | green |
| native I/O tests | green |
| specification checks | green |

- [x] All stages passed on the commit named above.
- [x] No test was deleted, disabled, or narrowed. Tests for the old writer scheduler and Windows completion slot pool were removed as those subsystems were replaced; their functionality is covered by the new scheduler core tests.

https://claude.ai/code/session_01TgfzFwvfhmCPhXgdhD2Mc5

---

## #25 Implement store-branded containers and typed owned storage

State: closed | Merged: yes, 2026-09-08 | Head: codex/container-continuation | Base: main | Created: 2026-09-07 | Closed: 2026-09-08

This PR implements store-branded containers and the typed storage foundation needed to execute their values efficiently. It also closes the ownership gap between capturing a mutation target and actually committing its right-hand side.

## Complete change

- Amend the active specification from v0.50 to v0.51, preserving the exact outgoing v0.50 bytes in `spec/kernel-spec-v0.50.md`. The amendment covers initialized-window measures, container operations, ordered results and commits, liveness, linearity and provider release, view origins, and call-fact transport. The five container/provider nominals are `FixedVector`, `Vector`, `Heap`, `Arena`, and `Box`.
- Implement the corresponding checking and executable lowering, including generic and store-region instantiation, initialization-window operations, owned-value transfer and release, shared/exclusive views, and constant fixed-vector construction. Existing range I/O operations use views; this PR does not introduce the separate I/O agent's scheduling/runtime model.
- Separate value snapshots from physical storage. Typed field/index addresses, explicit aggregate result destinations, CFG liveness, and conservative storage reuse prevent element updates from rebuilding whole aggregates. Eligible fresh bindings construct directly in their planned backing. Definitions, ordinary/refused/granted/staged calls, thunks, and split calls use the typed internal ABI; exposed addresses and deferred uses keep their backing alive.
- Preserve evaluation and cleanup order: capture targets before RHS evaluation, revalidate the complete post-RHS loan state before commit, then read the displaced owner for replacement. Cleanup snapshots its required values before release effects and preserves field/binding release order. Granted task results are consumed before their frame is released.
- Implement the owner-selected OWN-4/OWN-6 boundary: a completed `own` enum match header or exact `own Bool` condition ends only temporary argument loans created by that header, before its arm/branch begins, in statement and value forms. Bound holders, surviving views, borrowed-result candidates, borrowed matches, and older enclosing temporaries retain their loans. A unique parent resumes only after its last suspending child ends. Ordinary RHS loans still cover later RHS expressions and the final commit.

## Selection grounds and conformance

The [assessment](https://github.com/mbbill/Whitefoot/blob/codex/container-continuation/research/investigations/containers-and-resources/REASSESSMENT.md) selects general owned places and result destinations using executable dense scalar, wide-record, inline-view, failure/cleanup, and lifetime witnesses. Six pinned subsystem traces from Rust, C++, and Go recover behavior, storage, and lifetime requirements; they are qualitative evidence, not production prevalence measurements or a distribution inferred from Whitefoot tests. [Recorded dense measurements](https://github.com/mbbill/Whitefoot/blob/codex/container-continuation/research/experiments/container-representation/dense/RESULTS.md) retain their specific workloads, host, and comparison limits.

The complete amendment adds 21 rule IDs and retires none; the [derivation ledger](https://github.com/mbbill/Whitefoot/blob/codex/container-continuation/spec/derivation/derivation-ledger.md#v051-amendment--multi-return-and-the-proof-surface-2026-09-04) records the per-family grounds. The final header change adds no rule, token, or operation spelling and adds one temporary-loan endpoint. Its ground is a concrete composition obstruction: whole-match loans prevent repeated typed acquisition through a borrowed provider, while a preceding `let` is not generally equivalent because child-region admission and direct-match result facts differ. STOR-5's prohibition on borrow/view payloads and completed argument accesses justify the type-selected endpoint. This is not last-use inference or a special allocator rule. Future stored-borrow support must revisit the non-escape premise.

Conformance now contains **749 cases**, with **206 additions and zero removals** relative to main `6cc00984`. The new loan cases distinguish disjoint captured-index mutation from overlapping RHS/commit access, cover all owned control forms, and retain bound/view/result/borrowed-match exclusions. Six legacy cases remain because their `array`, `buffer`, and `arena` rules still exist; migrating to the new types would not preserve those exact obligations. Their expected verdicts are preserved. The legacy borrowed-buffer replacement case remains xfail with expected OP-4 because compilation explicitly stops at `BorrowedBufferDescriptorMutation`; unsupported capability is not relabeled as normative rejection.

## Boundaries

Legacy `array`, `buffer`, lowercase `box`, and `arena` remain; broad retirement is not part of this PR. Public partial-initialization authority, a dedicated fully initialized array API, general destination/alias-directed placement, stronger field/result contracts, and later container/API design remain subsequent research. This PR does not claim to finish those capabilities.

The native regression separately executes a direct user-call match header and the bound-`let` form that the current optimizer actually stages. Deferred execution checks result 48, real overlap, and exact source/frame release; it does not claim direct user-call match actualization. Physical storage remains valid through join return, result consumption, and retirement, including suspension or worker migration; an internal DONE flag alone is insufficient.

## Review and validation

- **Scope:** GPT-5.6 Sol reviewers inspected the complete inherited specification/conformance scope and bounded owned-storage ABI, address planning, mutation, cleanup, parallel retirement, header-loan, and affected guidance paths; base `6cc00984`, delivered head `9e9edb5a`. These targeted reviews do not certify every line of the large inherited PR. Unrelated runtime internals and malformed-IR hardening were outside the code review.
- **Checks:** Canonical root `make check` **passed on clean `9e9edb5af6e7d12e6205fcbb7d046efef415a15b`**, with all scratch confined to the isolated worktree: 1,506 unit cases, 71 sampling/parallel cases, 73 program cases, format/lint/build/docs and runtime checks, maintained research tests, specification/archive checks, full native conformance **747 passed / 1 expected failure / 1 declared skip**, and snapshots **484 passed / 0 flips**. The ignored conformance and snapshot adapters were explicitly run by the canonical gate. All 161 specification rules are covered. Remote [gate](https://github.com/mbbill/Whitefoot/actions/runs/34181684983) passed all 12 jobs and [Linux/Windows host tests](https://github.com/mbbill/Whitefoot/actions/runs/34181685086) passed both jobs, each on attempt 1 at this exact head. The final commits do not match the I/O benchmark workflow's existing path filter, so that workflow did not run for this head. Archive-byte comparison and patch checks pass.
- **Findings:** The reported semantic, legacy-coverage, and current-documentation findings were fixed and rechecked within the reviewed scope. Separate memory lint still reports the same 24 format/provenance diagnostics from earlier committed history in this PR, rather than main, with no new violations; committed Facts/Moves remain unchanged under the required append-only history rule. There are no remaining implementation or delivery findings within the reviewed scope. Validation is complete; this PR is ready for owner review, with no main merge requested.

---

## #26 [Paused] Research sequential-source concurrency and native baselines

State: open (draft) | Merged: no | Head: codex/io-runtime-followup | Base: main | Created: 2026-09-07 | Closed: -

> **Paused — compute research continues in [draft PR #28](https://github.com/mbbill/Whitefoot/pull/28), branch `codex/compute-runtime`, from `main`.**
>
> This PR remains an open draft preserving I/O/runtime implementations and measurements at `6de4557c1c7259084b134c4b51894f386265b657`. Neither stackful nor stackless execution has been established as universally optimal or impossible. The stackless experiment does not establish issue/continue/join equivalence or a separately compilable public ABI. The new investigation recovers pre-I/O current-stack join/help/steal, expands WF's own programs beyond the small layout example, and rebuilds native references without assuming Rayon is the performance ceiling. Implementation and timing on this I/O line are paused.

## Change

Sequential WF calls can expose independent I/O through staged execution and experimental LLVM continuations. The opt-in `--continuation-compute --emit-llvm --par` mode also suspends an I/O continuation while an expensive checked pure callee runs on the existing CPU pool, so the sole I/O owner can service other connections.

**Recorded research at `6de4557c1c7259084b134c4b51894f386265b657`; no general performance lead is established.** The active specification and conformance corpus are unchanged relative to main `6cc00984`. Source has no `yield` construct: earlier yield measurements concern the runtime's OS `sched_yield()`.

Arguments and results remain in the suspended caller's typed frame. Admission includes queued, executing and completed-unretired jobs, capped at twice the actual started CPU-worker count. The owner retires the lane after core DONE before resuming the parent. Zero-worker startup takes the ordinary sequential path; a bounded scalar entry probe can bypass admission for qualifying cheap returns. Worker-side ordinary compute outlining remains enabled.

The internal entries are `wf__par_compute_workers`, `wf__par_publish_async` and `wf__par_frame_done`; core completion-record and container layouts stay unchanged. Direct owner-side `LoopSplit`, effectful compute offload, multi-owner continuations, general cancellation and asynchronous cleanup remain open. This is not a language-wide fairness guarantee.

- [Comparison matrix and evidence levels](https://github.com/mbbill/Whitefoot/blob/codex/io-runtime-followup/research/investigations/io-model/SCHEDULER-EXPERIMENT.md#baseline-matrix-and-evidence-levels)
- [Designs, rejected alternatives and measured artifacts](https://github.com/mbbill/Whitefoot/blob/codex/io-runtime-followup/research/investigations/io-model/SCHEDULER-EXPERIMENT.md)
- [Reproduction commands](https://github.com/mbbill/Whitefoot/blob/codex/io-runtime-followup/research/experiments/io-completion-bench/README.md) and [compiler capabilities and limits](https://github.com/mbbill/Whitefoot/blob/codex/io-runtime-followup/compiler/README.md)

## Recorded results and controls

| Experiment | Qualified evidence and interpretation |
| --- | --- |
| Stronger client, 60 | With fixed server CPU, increasing client width changes large-message WF/epoll from 1.0351 to 0.7905; every paired comparison reverses. Earlier close rates were sensitive to client capacity. |
| Native epoll/io_uring, 62 | At large-message client width two, epoll beats all seven alternatives in every pass. Pure-ring 128-buffer uring/epoll throughput is 0.894937 and WF/epoll is 0.820019. A missing summary denominator failed the original workflow after its complete cohort; the repaired summary uses the original artifact without retiming. |
| CPU controls, 63–64 | Lower publication count gives no useful gain. Stack-capacity effects overlap duplicate-baseline variation; retain twelve stacks and the original builder/granularity. Against the qualified same-host Rayon anchor, the short S12 workload takes 8.88% more wall time; the long ratio is 1.003446. |
| Storage, 65 | Private/shared manual-epoll large-message throughput is 0.847483, CPU/trip 1.160714 and p99 1.279192; all five pairs worsen. WF/private retains a throughput gap at 0.924538. |
| Owned chunks, 67 | [Frozen ARM run](https://github.com/mbbill/Whitefoot/actions/runs/34138346137) passes. Lease/private-C++ throughput is 1.1431, CPU/trip 0.8730 and max RSS 0.5150, with all five pairs improving. Lease/epoll rate is only 0.9959; every p99 pair worsens, including the adverse 3,753 us sample and ratio range 1.0087..3.2050. Shared epoll remains the competitive reference. |

These ratios are paired within each experiment's recorded host/cohort, not pooled across hosts. The owned-chunk fixture independently qualifies simultaneous loans, reuse, short sends and EAGAIN. Six separate ordinary-load observations have only one live node and no send wait; timed loan occupancy and backpressure are not inferred from them.

Experiment 66 now has **36 M1 ASan/UBSan + 36 M1 TSan** qualifying invocations and **72 successful Linux CI invocations** at `03d3e05d`. Held-worker cases demonstrate a light reply while three heavy requests cannot finish, then verify 64 changing-input frame reuses. Recursive cases balance 75,024 inner publications/joins under one outer job. Default and forced-helper policies pass; the new compute reports alone do not count each operation's native route. No mixed-performance result is claimed.

Experiment 68 retains the original Tokio/Rayon reference and adds optional `mixed-retain`. Its actual semaphore permit travels with the completed result until consumption or discard, exposing the effect of counting completed-unconsumed results against Q. An independent closure-tail counter makes drain wait for the producer's final source-controlled cleanup marker. This is not Rayon's internal epilogue or WF's physical core-DONE boundary. The guard Arc, atomic counter, Notify, larger oneshot value and drain cost belong to the diagnostic's ordinary executable. It is not an optimized replacement or a claim that Tokio/Rayon is the fastest available mixed implementation.

## CI corrections and remaining failure

The exact [`03d3e05d` canonical run](https://github.com/mbbill/Whitefoot/actions/runs/34142452216) has **13 successful jobs, two failures and one cancellation during package download**. Linux scheduler-streams, both sampling jobs and both conformance jobs pass. This failed run remains recorded.

- macOS scheduler-streams used Apple clang 15, which cannot parse `coro_elide_safe`. That stage now installs/selects LLVM 20 explicitly. Other stages keep the host compiler; the Linux scheduler job no longer downloads the continuation-only package that stalled before any test ran.
- Linux static's wake-count test confused a provisional sleeper announcement with a completed epoch recheck. Correct early notification can cancel the park with no eventfd write. Test-only gates now select and check both this zero-write cancellation and the committed park's exact one-write wake/drain. The two-ring and four-record-waiter scenarios remain. Both Linux probe build callers enable the hooks; ordinary runtime code has no gate calls.

The previous `03d3e05d` correction also replaces finite steal sampling with a controlled foreign-thread thunk handoff and direct acquisition/publication counters. The weak-refusal negative case keeps correct output but fails that evidence requirement. Ordinary byte matrices remain; no runtime scheduling policy changed.

## Review and validation

- **Scope:** Bounded independent reviews cover the prior runtime/lowering and frozen measurements; this update does not re-review the entire historical PR. `readiness_review`, `native_baselines`, `workflow_review` and root reviewed the current toolchain/evidence, five-file native-wake correction, three-file retained control and two-file documentation scopes. The missed direct Linux CI probe caller was fixed and rechecked. No findings remain within these scopes. Integration preserves the reviewed changes.
- **Checks:** The complete local compiler gate at `b3b5d8e2` passes 1,436 unit and 71 sampling tests, corpus, format/lint/docs/spec and native completion/core checks; it is not the root all-repository gate. Experiment 66 has the sanitizer and Linux qualification above; prior-mode WF IR remains byte-identical, with macro-off host differences limited to module paths/assertion source locations. The retained reference passes seven original and eight retained Rust tests, four original and six retained external protocol cases, plus six separate release observation reports. Two deliberately broken permit/tail variants fail their distinguishing assertions. Root replays the canonical mixed target and the qualified ordinary-release IR comparison; only ten verified source-location records/references and one empty-barrier source cookie are normalized. Native wake qualification passes strict Linux cross-builds and an ASan/UBSan replay of actual park/notify bodies using POSIX pipe/poll substitutes, with four distinguishing negative controls. That replay is explicitly not native io_uring execution.
- **Gate status at pause:** The exact [`6de4557c` canonical run](https://github.com/mbbill/Whitefoot/actions/runs/34145082109) completed with 15 successful jobs and one macOS scheduler-streams failure: upstream LLVM 20 rejects the Darwin stack-probing method. Both [native Linux and Windows host jobs](https://github.com/mbbill/Whitefoot/actions/runs/34145082062) passed. This head is not fully green. Earlier gates, frozen failures and adverse timing cohorts remain separate evidence; no merge is requested.
- **Unfinished at pause:** The controlled mixed comparison, residual native-I/O gap, ready-work availability, cleanup and multi-owner limits remain unresolved. I/O implementation and timing are paused. Pure-compute recovery and a rebuilt WF workload/reference suite continue separately in [PR #28](https://github.com/mbbill/Whitefoot/pull/28), based on main rather than this branch. Hosted topology does not promise dedicated physical cores; ordinary timings and instrumented observations remain separate.

---

## #27 docs: clarify project guidance, add completion review, and repair memory

State: closed | Merged: yes, 2026-09-07 | Head: codex/docs-ai-workflow | Base: main | Created: 2026-09-07 | Closed: 2026-09-07

## Change

Clarify Whitefoot's goals and document ownership; add a [22-item completion checklist](https://github.com/mbbill/Whitefoot/blob/8830c07d926adee83b6f3c6f9a750e82ae9be72a/docs/review-checklist.md) for fast-agent review and automatic publication to existing PRs. Reduce the PR template to a change summary and three review bullets, with detailed obligations kept in the checklist. Remove the retired merge-hook target; compiler source edits are comments only. The active specification, conformance evidence, immutable specification archives, and container implementation are unchanged.

Repair MCTS-Mem integrity through explicitly authorized normalization of historical metadata: correct provenance and replacement endpoints, add matching rationales, and remove construction-only or duplicate records. Technical evidence and rejected alternatives remain; no lint checks were disabled.

## Agent review

- **Scope:** gpt-5.6-luna, medium; incremental reviews of `7868e5f8..8830c07d`. Checked A/D/M/V and preservation of the template's language-evidence obligations; implementation/spec changes were inapplicable in that range. Earlier prose has not received the complete purpose/citation audit.
- **Checks:** current `make static`, patch whitespace, and 11 local links/anchors passed. Memory lint passed with 126 nodes at `30246127`; memory is unchanged since. Root `make check` last passed at `7868e5f8`, **not the current head**. Remote PR head and description are synchronized.
- **Findings:** no remaining findings in the reviewed follow-ups. Existing **D1/D2** issue at [constitution lines 9–12](https://github.com/mbbill/Whitefoot/blob/8830c07d926adee83b6f3c6f9a750e82ae9be72a/docs/constitution.md#L9-L12): “The owner clarified…” is editorial history, and its memory citation violates the document's purpose/reference boundary. Cleanup of this and similar earlier prose remains pending.

---

## #28 Integrate current-stack compute experiments, native baselines and scheduling controls

State: open | Merged: no | Head: codex/compute-runtime | Base: main | Created: 2026-09-07 | Closed: -

## Change

Integrate the measured pure-compute research foundation before resuming [I/O design, PR #26](https://github.com/mbbill/Whitefoot/pull/26). This includes the recovered current-stack compute runtime, executable WF workloads (FIR, UTF-8 records, Mandelbrot and adaptive quadrature), native references and measurement/checking tools. The [experiment README](https://github.com/mbbill/Whitefoot/blob/424a99bc9522815128daa7c7ff70d0d182512632/research/experiments/compute-runtime/README.md) owns commands, artifacts and dated evidence.

The recovered research runtime uses persistent ordinary worker stacks, owner-local slots/deques, inline execution of an unstolen join target and nested helping before its yield/park slow path. It repairs deque-cell races and initialization ordering. Atomic publication/ownership and slow-path locks remain; no lock-free optimum is claimed. Diagnostic counters are excluded from ordinary timing images. **Normal whitefootc linking still uses the shared compute/I/O scheduler; this PR does not promote the research runtime to the default.**

Existing compiler controls also merge: scalar-leaf offer suppression, sequential fallback after refused offers and private recursive-frontier specialization (depths 1–32). Under `--par`, scalar-leaf suppression now defaults to 16 nonconstant IR operations. `--par-scalar-leaf-limit N` overrides it and `off` restores unfiltered offers; numeric 0 retains its previous meaning. All six historical unfiltered compute experiment recipes explicitly pin `off`. The choice follows consistent gains across the measured M1/Linux quadrature inputs, not a universal or shared-runtime performance guarantee. Refusal and recursive frontier remain opt-in. These controls preserve acceptance, joins and public function signatures; compilation without `--par` still leaves compute outlining off. Further policy tuning is deferred so future work can start from main. Native controls include oneTBB, Parlay, native WF and pinned Rayon, reciprocal fork directions and serial kernel controls, with SIMD/FMA/LTO off in the scheduling panels.

Generated WF quadrature depths 4 and 8 and their sequential controls share the ordinary image, retain separate IR/ledger/assembly and undergo exact work and owner-slot exhaustion checks.

**Final repair:** [05649749's macOS research job](https://github.com/mbbill/Whitefoot/actions/runs/34274301809/job/102223392979) exposed an invalid assumption that every short Rayon process must migrate work. All-local execution is legal. The same kernel join/migration helper now has a controlled two-worker test in both fork directions; the gate accepts coherent all-local reports and still rejects helper work without migration. Numerical, fork/work conservation, diagnostic, sanitizer and exhaustion checks remain. Older failures without forwarded stderr remain unclassified. Helper extraction can affect code layout, so the historical timings below remain bound to their original images.

## Performance evidence

The independently audited a62b98f2 [Intel Linux job](https://github.com/mbbill/Whitefoot/actions/runs/34269691191/job/102207830095) includes 1,280 processes/5,253,120 checked outputs. This is a different host from the prior EPYC cohort: Xeon8370C VM, two reported cores/four SMT CPUs, mask0–3. No host/source causal delta is inferred. All peaked-input W4 WF/Rayon wall/CPU comparisons win five pairs under both observers, but balanced cap is near parity:

| W4 configuration | Center wall us | Left wall us | Right wall us | Cap wall us |
|---|---:|---:|---:|---:|
| Generated WF leaf |31.502|25.898|25.258|64.576|
| Generated WF frontier8 |19.527|16.735|15.327|37.681|
| Native WF value8 |20.028|17.319|16.036|38.529|
| Rayon4 |22.479|21.025|30.716|37.648|
| Rayon-left4 |22.598|30.779|20.718|37.559|
| Parlay-left4 |49.915|79.545|22.625|37.462|
| Parlay-left8 |21.635|18.340|18.441|39.826|
| oneTBB8 |37.524|31.051|28.484|54.600|

Values are medians of 4096-call process means, not latency percentiles. Paired cap WF/Rayon4 ratios are 0.999(3/5 lower wall pairs), WF/Rayon-left4 1.012(2/5), and WF/Parlay4 1.006(2/5). Parlay4 uses less CPU on cap in all five pairs, WF CPU ratio1.040. Hardware counters remain unavailable and CPU accounting discrepancies persist.

The independent unit-node tree model already gives balanced cap enough four-worker parallelism at depth4, while skewed depth4 has model parallelism only1.668. This motivates the matched generated-depth experiment, not a default cutoff.

**Dated M1 ordinary cohort (05649749):** 720 processes/2,954,880 checked outputs,18 forms×2 widths×4 inputs×5 alternating passes, same scalar settings. Unpinned M1, no frequency or PMU qualification; separate from all earlier cohorts. Four-worker paired medians:

| Input | Generated WF4 wall us | WF4/WF8 wall | WF4/Parlay-left4 wall | WF4/Rayon-left4 wall |
|---|---:|---:|---:|---:|
| Center peak |10.734|1.073|0.654|0.678|
| Left peak |12.961|1.500|0.499|0.443|
| Right peak |10.806|1.355|0.945|0.743|
| Depth cap |15.448|0.944|0.971|0.916|

WF4 loses all five WF8 wall pairs on peaked inputs and wins all five on cap. Its CPU ratios versus WF8 are1.018/1.553/1.301/0.951. On cap it wins all five wall pairs against Parlay4 and both Rayon4 directions. Cap CPU versus Parlay4 retains one adverse pair (maximum1.040), despite a0.937 median. CPU losses remain on other cases, including WF4/Rayon4 left1.078 and WF4/Parlay4 right1.120. W1 WF4/WF8 ratios1.021/0.997/1.018/1.002 and sequential-clone4/8 ratios1.002/0.999/1.000/0.998 retain mixed/adverse controls; they are not normalized away. Grain, specialization/inlining and code layout change together; these ratios do not isolate per-task scheduler instructions.

Earlier diagnostic worker-node attribution remains available through `quadrature-worker-profile` in CI. It identifies fork-direction-dependent work distribution but does not establish why ordinary timings differ; task-handoff timing remains unfinished. These finite panels do not establish universal fastest performance, fully tuned references, general composition/scaling/tails or hardware limits.


## Agent review

- **Scope:** workflow_review and stats_review (inherited models) reviewed the eight-file migration fix, 05649749..d981d03c; workflow_review additionally reviewed the six-file CLI default, baseline recipe and documentation delta d981d03c..424a99bc9522815128daa7c7ff70d0d182512632. Prior rounds reviewed the compiler/runtime and experiments separately; these final scoped reviews do not re-certify the entire PR. No specification or conformance changes relative to current main f2a29866.
- **Checks:** canonical root `make check` passed on final tree `27747a16adba91dae19a6f6199a5f0f0174660b0`, exit 0, ending `WHITEFOOT ALL TESTS GREEN`. This includes the 1,584 library/backend tests, 15 CLI tests (new defaults/overrides included), 73 program tests, all maintained research experiments, native conformance (747 Pass / 1 existing Xfail / 1 Skip), and snapshot (484 Pass / 0 Flip). Both Rayon tests pass; quadrature batch qualification passes 284 processes / 3,124 outputs. Independent parser replay accepts coherent all-local work and rejects inconsistent migration for the expected reason. `git diff --check` passes and the worktree is clean.
- **CI:** exact head 424a99bc passes all 12 [gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34279474433), [compute-bench](https://github.com/mbbill/Whitefoot/actions/runs/34279474382), [io-hosts](https://github.com/mbbill/Whitefoot/actions/runs/34279474464) and [io-bench](https://github.com/mbbill/Whitefoot/actions/runs/34279474381). Workflow success does not add new audited performance claims to the dated tables above.
- **Findings:** no remaining findings within the final reviewed scope. The earlier local gate was deliberately interrupted before the authorized default change; only the subsequent complete gate certifies this tree. Finite benchmark panels do not establish universal fastest performance or theoretical limits. Further scheduler policy research is deferred; I/O design resumes after integration.

This candidate contains current main f2a29866. It is ready for the owner's exact-revision merge approval under AGENTS.md; it has not yet been merged.

---

## #29 Rewrite the constitution and make decision grounds maintainable

State: closed | Merged: yes, 2026-09-09 | Head: codex/agent-harness-constitution | Base: main | Created: 2026-09-07 | Closed: 2026-09-09

Rewrite the constitution around Whitefoot as a programming language designed as a harness for AI agents, preserving human control over objectives and tradeoffs.

Define when to read, decide, update affected records, and review. Assess all 161 active rules: 129 have current provisional grounds; 32 retain concrete revisit questions. Correct stale memory summaries and the host-qualification comment, with historical evidence preserved.

Specification v0.51 → v0.52: amend META-6 to require the current grounds index and remove obsolete constitutional references and unsupported rationale. Rules/tokens/spellings/exceptions +0/−0. Archive the outgoing specification unchanged. Writer acceptance and runtime behavior remain unchanged. The workflow is a provisional choice addressing observed traceability failures.

- **Scope:** incremental agent reviews; final round 6f5a4e83 → 868354f0 reviewed by two GPT-5.6-Luna agents and an independent workflow reviewer. No executable compiler changes in this round.
- **Checks:** full make check passed on 868354f0 in 11m 28s. Conformance: 747 pass, 1 expected failure, 1 skip. Snapshots: 484 pass, 0 flips. Skill-directed memory lint passes; coverage, references, history preservation, and diff checks pass.
- **Findings:** assessment does not establish optimality or soundness. Allocation/effect inconsistencies, ambiguous meta/trust-boundary wording, proof expressivity, and compilation costs remain explicitly open. Workflow efficacy remains unmeasured.

---

## #30 Container storage foundations and exclusive two-state run contracts

State: open (draft) | Merged: no | Head: codex/container-foundation-research | Base: main | Created: 2026-09-08 | Closed: -

## Change

This PR contains container storage foundations, owning-array support and the v0.53–v0.56 amendments. Head `6614c0310549ea62dae164166cbbca8e7e718def`. It does not claim a complete generic container library or universal native performance.

- **Research and experiments:** [family witnesses and retained data](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/research/experiments/container-representation/families/RESULTS.md) cover dense storage, heap operations, owning sparse mutation/growth/refusal, byte growth and layout controls. External source studies recover requirements, not workload prevalence.
- **Spec-neutral repairs:** owner-slot/address writeback, nested-region substitution, result-ordinal facts and release-sensitive physical call specialization; D5 additionally fixes loop-origin set comparison (`245f932b`) and direct/held Box-content storage paths (`415ff306`), each with a prior-spec regression.
- **Storage placement:** eligible consumed inputs reuse complete results or a complete struct-result field. This is bounded compiler optimization, not a general same-place owning-call guarantee.

## Amendments and selection grounds

| Amendment | Changed behavior and discriminating witness | Selection ground |
| --- | --- | --- |
| v0.53 | Owning full arrays/conversions; [owning round trip](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/blk3-pos-full-array-owning-round-trip.wf). Invariant brands remain distinct from loans; [multiple brands](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/form8-pos-multiple-brand-positions.wf). | Dense complete owning storage and actual store identity must survive ordinary value transfer without invented provider arguments. |
| v0.54 | Normal-exit owner routing through results/exclusive referents, replacement identity and dead-binding initialization; [Box exchange](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/fn1-pos-exclusive-owner-exit-state.wf), [omitted read rejection](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/eff2-neg-installed-owner-read-omitted.wf). | Ordinary-object direct/helper equivalence and correct attribution of the owner actually read or replaced. |
| v0.55 | Temporary-child endpoints are independent of enclosing region extent; [longer local region](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/own6-pos-statement-children-use-a-longer-local-region.wf), [suspended-parent rejection](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/own5-neg-later-argument-uses-suspended-parent.wf). | Sequential reuse needs no one-statement region restriction; same-statement loans must still exclude conflicting accesses. |
| v0.56 | `entry(parameter)` selects the entry measure in ensures; bare exclusive measures select exit. Four boundary rows mutate through `&uniq`. The whole BLK-4 recursive unique-parameter refusal is removed, including opaque generics; confinement/loan-bearing positions stay. [Counted push/pop](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/run-exclusive-push-pop-counted.wf), [generic append](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/run-exclusive-generic-append.wf), [generic stale-fact rejection](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/tests/conformance/cases/exclusive-neg-generic-element-replacement-kills.wf). | D1 identifies owning transfer cost; D2 exposes the missing mutable post-state contract. Exact effect-overlap kills followed by verified ensures handle all referents uniformly. Whole replacement and same-time increment negatives exclude unsound shortcuts. |

The v0.56 change archives the exact outgoing v0.55 bytes, updates grammar/kernel rows, FN-9/MSR-3/CALL-6, equality proofs, rule grounds, compiler guide and writer patterns. [Migration inventory](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/research/experiments/container-representation/families/exclusive-migrations.csv) lists 444 existing WF files. Four former refusal witnesses become positives under the amendment; other expected outcomes remain. No verdict was relaxed to mask an implementation failure.

## Measured D5 scope

[Retained-helper measurements and limitations](https://github.com/mbbill/Whitefoot/blob/85fce15f3d1d2b3700502e7e088b77b49e60119f/research/experiments/container-representation/families/RESULTS.md#exclusive-two-state-contracts): the inline scalar heap uses an ordinary proved contiguous view; WF/C medians are 4.380/4.282 microseconds per sixteen-round trace, with zero run-transfer bytes and no heap allocation. The D1 same-place contract is dropped.
At map capacity 4096, retained WF/interleaved-C lookup is 2.857/2.683 ns, insertion 3.271/2.843 ns and doubling 15.667/13.427 microseconds. About 93% of the prior retained insertion gap is absent; receiver descriptor writeback is zero. Stride remains 24 bytes, with 32 requested bytes per capacity unit. Initialization, circular addressing and exhausted-source cleanup remain; small-table growth is still 34% slower. No projected sparse layout is selected.
Each of three lowering modes checks 1,808 growing-map executions with every allocation-refusal position and exact owner/release accounting. Generic hash/equality behavior, stable-row/concurrent containers and general recursive origin routing remain outside this result.

## D6b behavior design study

[Three-page comparison and three complete source appendices](https://github.com/mbbill/Whitefoot/blob/cd3e946817af5eaea9a80efcefd828e39b94d015/research/investigations/containers-and-resources/BEHAVIOR.md) supersede the rejected D6 type-conformance/open-row proposal. Seven primary-source language surveys compare the owner's function-kind + shape/group baseline with nominal-as-shape and expanded function-kind-only interfaces, on identical owning map find/put and priority push/pop operations. Syntax counts include top-level declarations, binder forms, productions and keywords.
The conditional recommendation is shape/group over one function-substitution core. Fixed formal rows with narrower actuals require an explicit EFF-2 abstraction boundary; same-row identity/fresh-return functions also require a proposed FN-1 routing ceiling. Those rules are unimplemented and unvalidated. The note covers v0.56 contracts/regions, hygienic expansion, branded-type limitations and finite function/nominal instantiation; non-exponential checking remains an explicit shared prerequisite. Native parity and universal library acceptance are not claimed. D6b changes no specification, compiler, tests or memory nodes.

## D7 in progress

The owner selected option A with `formal`/`actual`, authoritative formal rows, prefix-covered actual effects, structurally equal contracts, and fresh non-copy owned function-formal results. The proposed FN-1 routing ceiling is dropped; compile-cost remains recorded. Behavior parameterization is not implemented at this head.
The prerequisite backend repair (`6614c031`) omits entry-unreachable CFG and excludes its phi predecessors, fixing return-only loop continuations in the expanded priority queue. The new regression runs across all three lowering modes; the counted-ranges suite passed 2/2, and the full concrete priority source compiled and ran in all three modes. Independent review of the two-file repair found no issue and independently passed the existing no-native-completion regression. No specification change belongs to this repair.

## Boundaries retained

`state_origins.rs` and `check/result_state_origin.rs` change only the redeclared row result/referent mapping. `lowering/specialize.rs` is unchanged. General v0.54 routing remains for real owning transfers; a unit-returning boundary row has no run-result reuse edge. Earlier parked prototypes stay excluded. Work remains on this PR; no split, new PR or I/O/runtime workspace change.

## Validation and review

- **Scope:** independent `loop_regression_review` agent, D5 base `d45ad27c` through `85fce15f3d1d2b3700502e7e088b77b49e60119f`, core/spec/conformance, docs, memory, measurements and representative bulk migrations. The earlier PR history was not independently re-reviewed in this round. No remaining finding in this scope.
- **Checks:** full canonical root `make check` passed on the D5 tree `85fce15f`: 1,728 unit tests, 73 sampling tests, 73 program integration tests, all maintained research checks, native conformance 773 pass / 1 expected failure / 1 skip, and 484 snapshot cases with zero flips. Static/archive checks and memory lint (153 nodes) passed. Independent review has no remaining finding in the stated scope.
- **D5 CI baseline:** all 18 checks passed on `85fce15f`: [12 gate jobs](https://github.com/mbbill/Whitefoot/actions/runs/34569177219), [2 host completion jobs](https://github.com/mbbill/Whitefoot/actions/runs/34569177240), and [4 benchmark jobs](https://github.com/mbbill/Whitefoot/actions/runs/34569177221). D6b changes research proposal artifacts only; this PR remains draft and no merge is requested.
- **D6b review:** independent `d6_completion_review` agent, base `0a7167f2` through `cd3e946817af5eaea9a80efcefd828e39b94d015`, reviewed the proposal, three full-source variants and affected current rules; earlier PR work was not re-reviewed. No remaining blocking findings within the research scope; routing soundness, compiler acceptance and native parity remain unestablished.
- **D6b checks:** `make static`, staged `git diff --check`, all five local document links, and one-shot comparison of all 17 complete function bodies across three variants passed. Three-page layout checked at 10pt body. Proposed syntax is not compiled or represented as a maintained executable test; no new native measurement or full local `make check` is claimed. Prior D5 gate/measurements retain their exact scope.
- **Current CI:** Checks for prerequisite head `6614c0310549ea62dae164166cbbca8e7e718def` are running. D6b head `cd3e9468` passed all 14 checks; D7 full-gate verification will be reported on its delivered head.

---

## #31 Bring the I/O bench programs to the current language and let the gate see them

State: closed | Merged: yes, 2026-09-11 | Head: io/bench-programs-current-language | Base: main | Created: 2026-09-10 | Closed: 2026-09-11

## Change

Since f2a29866 (the containers batch) ten of the twelve programs in `research/experiments/io-completion-bench/programs` have not compiled: `len` became `len_of`, the ambient heap stopped being an effect root so `allocates(heap)` no longer resolves, and a callee's unique borrow of a whole buffer now kills the caller's length fact, so a name buffer handed to `name_at(&uniq name)` could no longer discharge the `0..10` range of the `open_file` after it. The io-bench workflow stayed green through this: its steps run `sh ... | tee`, so a step's status was tee's, and the summary step printed "the network table did not run" into a green job. Every io-bench run on main since then finished in six to eight minutes with empty read and network tables (the merge of #13 took 41 minutes with full tables).

This migrates the programs the way `read_heavy_wide8_4k.wf` already was: `len_of`, no allocation clause, and the two unique-borrow helpers (`name_at`, `render_u64`) take `&uniq MutSlice<u8>` lent through `mut_slice_of` in a nested region, so the buffer keeps its length. Sizes, offsets, checksums, the printed line and the doc strings are unchanged. Two guards follow: the bundle gains `programs-check` (compile every program, nothing more) and the root gate runs it as a new `bench-programs` stage in its own CI job (about fifty seconds of compilation on four processors, so no existing stage grows); and `io-bench.yml` sets `shell: bash` as its default, which GitHub runs as `bash -eo pipefail`, so a protocol that fails behind `tee` now fails its step.

No specification or conformance change. The programs are research material, not conformance evidence; the gate change adds a stage and narrows nothing.

## Agent review

- Scope: Claude (this session); 3016842..6c281c87; all fourteen files. Read: every program diff. Run: see below.
- Checks, on 6c281c87: `make static` passes; `make bench-programs` compiles all twelve programs (50 s with the compiler already built); `make -C research/experiments/io-completion-bench verify` publishes `17098009301725298919 00000000000071024640` on every line; `read-verify` publishes the same bytes with the cache policy off and on; the network protocol's correctness pass (every server echoes what netload sent at 4 connections) passes and the table runs to completion for all three lines. The full root gate was not run locally (three program cases need a non-root user here); the gate workflow on this head is the full-gate result, including the new stage on both hosts. io-bench on this push is the first run with real tables since #13.
- Findings: none within the reviewed scope. One thing to know rather than a finding: `tcp_echo_server.wf` still describes one parked callee per connection, which is true on main today and will stop being true when the current-stack runtime lands; the network table will show that change instead of hiding it.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #32 Restore current-stack compute execution with independent I/O waits

State: closed | Merged: yes, 2026-09-11 | Head: compute/current-stack-runtime | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

Compute joins currently participate in the unified managed-stack scheduler. This restores ordinary native-stack join/help/steal with lane-owned task storage and atomic deque ownership, while retaining typed submit-then-join I/O through an independent completion wait. It includes the required lowering, storage/floor, native linking, tests and documentation integration. Compiler scalar-leaf defaults, refusal and recursive-frontier controls remain outside this change.

Connection-level concurrency through suspended user calls is temporarily unsupported. A source-order server finishes its current handler before entering the next, so a silent peer can hold up later connections. No restoration mechanism is chosen, and `WF_STACKS` is inert. The compiler guide and system-interface/parallelism memory state these boundaries. The TCP benchmark docstrings now describe ordinary source-order execution. Its concurrent four-peer protocol cannot complete with this WF server, so the CI network step explicitly selects `NET_LINES="uring epoll"`; both reference lines still undergo correctness and timing checks, the WF program still compiles under `bench-programs`, and `bash -eo pipefail` remains effective. The network table supplies no WF execution or performance evidence.

The managed-stack enumerator and its configuration/state-search checks are retired with their state machine, as explained in 6816e9bd. Native deque/TSan checks cover observed concurrent reuse, task completion and counter accesses, not exhaustive schedules, weak-memory ordering or liveness proofs. Separate startup, completion and exhaustion tests remain. Reverse-order connection fanout and staged lane-layout assertions are retired with that temporary unsupported capability; source-order network, result, mutation, error and cleanup checks remain. The conformance adapter only removes the deleted switch header's import/staging: no specification, conformance source, verdict or case-selection change.

The local M1 Pro `par_layout.wf --par` comparison at 6816e9bd used five alternating runs at W=1/2/4/8, with medians main 1.585030/0.835393/0.480445/0.444162 s versus branch 1.580739/0.814455/0.442291/0.374019 s; all 40 executions and 59 `cmp` checks passed. Compiler/runtime and this program's bytes are unchanged by the main merge and follow-up. This is retained evidence from that revision, not a new measurement or a general performance claim.

## Agent review

- Scope: independent Codex reviewer `runtime_split_review` (exact model identifier unavailable). The PR is `8498c774..731836dd`. The prior runtime review at `3016842..6816e9bd` covers the unchanged compiler/runtime; the three follow-up files were separately reviewed against merge commit `8f0460ec` and committed unchanged as `731836dd`. Checked A/D/C/T/R/M and available V evidence across those reviews; the follow-up did not repeat the runtime audit or review unrelated inherited main changes.
- Checks: exact pushed head `731836dd9e61f13cdec9af11a378121585e03b1c`. Local root `make check` passed all stages, including the new `bench-programs`; all locally invoked Cargo tests used `--profile gate`, retaining debug assertions and overflow checks. Native conformance: `Pass=747 Xfail=1 Skip=1`; snapshot: `Pass=484 Flip=0`. [CI gate](https://github.com/mbbill/Whitefoot/actions/runs/34546727335) passed all 14 Linux/macOS jobs, and [io-hosts](https://github.com/mbbill/Whitefoot/actions/runs/34546727261) passed Linux and Windows. `git diff --check` and `npx mcts-mem lint` passed (131 nodes, 0 fact files). The [io-bench Linux file/network job](https://github.com/mbbill/Whitefoot/actions/runs/34546727283/job/103100865096) passed: the log records `NET_LINES: uring epoll`, both selected servers pass the four-connection correctness check, and both publish all four workload rows.
- Findings: no remaining source-review findings within the reviewed scope. **io-bench is not fully qualified on this head.** Its [Windows job](https://github.com/mbbill/Whitefoot/actions/runs/34546727283/job/103100865014) failed at `windows-bench.ps1:612` with `mixed-total remained unstable after two complete cohorts`; the previous 6816e9bd run failed identically, and neither the compiler nor that protocol script changed in this follow-up. The log establishes failure of the existing measurement-stability criterion, not its cause. Linux/macOS read-heavy jobs are still running at PR creation; their results are unverified here and available in the [same exact-head run](https://github.com/mbbill/Whitefoot/actions/runs/34546727283). No Windows qualification threshold or protocol was changed.

---

## #33 Add compiler compute scheduling controls

State: closed | Merged: yes, 2026-09-11 | Head: codex/compiler-scheduling-controls | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

Short scalar calls can cost more to offer as compute tasks than to execute directly. This extracts the compiler scheduling controls from PR #28 at `70aa8e5` onto main after #32 (`33ed2c00`), with fresh evidence on main's current-stack runtime.

- `--par` suppresses offers of eligible scalar leaves with at most 16 nonconstant IR operations. `--par-scalar-leaf-limit N|off` overrides it; `0` filters only zero-operation leaves.
- `--par-sequential-refusal` optionally enters an existing same-ABI sequential clone at a refused call's original join.
- `--par-recursive-frontier N` optionally generates private parallel call layers before entering sequential clones; CLI range `1..32`. Both additional policies remain off by default.

The shared graph helper is the existing iterative SCC algorithm extracted for the frontier and stack report. Runtime code, host optimization/vectorization flags, public source signatures, joins, acceptance, non-`--par` lowering, and I/O submit-then-join are unchanged. No specification or conformance-evidence change, retired test, compute-bench workflow, native baseline, or wider research bundle is included. Current connection-level concurrency limits and `NET_LINES="uring epoll"` remain in force.

Only the quadrature kernel is brought forward, as `tests/programs/adaptive_quadrature.wf`, with an ordinary WF command entry and README reproduction commands. It integrates 2048 Lorentz profiles (center `1/4 + i/4096`, width `1/64`, tolerance `2^-42`, numerical depth cap 24) and emits the sum's full 64-bit representation as ASCII. The corpus test independently checks the analytic integral under sequential, unfiltered parallel and filtered parallel compilation. CLI boundaries, mixed-group join preservation, self/mutual recursion, callbacks, destination results, exclusive tree borrows, and excluded I/O components are covered by maintained tests.

The owner-reported Windows mixed-total tail from #32 is recorded as an open measurement in `mcts_mem/whitefoot/parallelism.md`, preserving the distinction from the stable cohort that excludes process startup/exit. It is not a diagnosis or a change to qualification thresholds.

Default requalification at **`cce8256a84ad17ce0625803886d4a9f4945db605`**:

Same-host whole-process timing, Apple M1 Pro (8 CPUs, 32 GiB), macOS 26.6.2, rustc 1.98.1, Apple clang 21.0.0, ordinary `whitefootc` gate-profile build and its unchanged `-O2` link. `WF_WORKERS=4`, other inherited WF/WHITEFOOT overrides cleared; no build/gate work overlapped timing. Five alternating pairs, all samples retained, no warmup or exclusions. Default and `off` differ only in leaf-offer selection; neither optional recursive policy is enabled.

| Pair | Order | Default 16 (ms) | Off (ms) |
|---|---|---:|---:|
| 1 | default, off | 310.450 | 330.962 |
| 2 | off, default | 33.836 | 82.386 |
| 3 | default, off | 33.591 | 83.320 |
| 4 | off, default | 33.761 | 82.634 |
| 5 | default, off | 34.063 | 83.419 |
| **Median** | | **33.836** | **83.320** |

Default/off median ratio **0.4061** (59.4% less elapsed time). The first pair is retained despite its higher process times. All five output pairs passed actual `/usr/bin/cmp`; every 64-byte bit string passed the analytic check within `1e-8`. The compiler report shows two omitted offers in `adaptive` and two in `integrate`; the off report has none. These are static offer-site counts, not measured dynamic task counts. Non-`--par` and `--no-overlap` LLVM also compare byte-for-byte with main's compiler on quadrature and par_layout; unfiltered main `--par` and the new default produce identical par_layout LLVM on this host.

The comparison checks changed offers, identical outputs and a default median no higher than `off`. This is an exploratory qualification of the provisional default on one workload/host, not a universal optimum or evidence for enabling refusal/frontier by default. The separate selection observation in memory is labeled exploratory and is not substituted for these exact-head samples.

Validation on that exact head:

| Root `make check` stage | Result | Seconds |
|---|---|---:|
| repository-invariants | PASS | 1 |
| spec-append-only | PASS | 0 |
| spec-prose-integrity | PASS | 0 |
| conformance structure | PASS | 0 |
| compiler | PASS | 344 |
| research-tests | PASS | 9 |
| bench-programs | PASS | 0 (up to date) |
| conformance-run | PASS | 98 |
| snapshot-run | PASS | 20 |

Root `make check` completed with `WHITEFOOT ALL TESTS GREEN` at the pushed head. Compiler sub-stages passed: format (1 s), lint (4 s), test-partition (34 s), test-unit (118 s), test-sampling (37 s), test-corpus (145 s), docs (0 s), spec (1 s), completion-test (4 s). Native conformance: **Pass=747, Xfail=1, Skip=1**, preserving the existing expectations; snapshot: **Pass=484, Flip=0**. All local Cargo test invocations used `--profile gate`; a temporary PATH wrapper added the profile, with debug assertions and overflow checks retained, to the standalone research packages whose unchanged Makefile calls do not specify one. No acceptance limits or repository gate changes were added.

| Exact-head CI | Result |
|---|---|
| [gate](https://github.com/mbbill/Whitefoot/actions/runs/34549854852) | **PASS, 14/14 jobs**, Linux and macOS |
| [io-hosts](https://github.com/mbbill/Whitefoot/actions/runs/34549854679) | **PASS**, Linux and Windows native qualification |
| [io-bench](https://github.com/mbbill/Whitefoot/actions/runs/34549854655), Linux file/network | **PASS**; network lines are uring and epoll only |
| io-bench, Windows | **FAIL**, compute cohort stability (details below) |
| io-bench, macOS read-heavy | **PASS** |
| io-bench, Linux read-heavy | **IN PROGRESS** when this PR was opened; no completed result claimed |

The Windows io-bench failure is **`compute remained unstable after two complete cohorts`** at `windows-bench.ps1:612`. From its published raw samples (15 alternating pairs per cohort), the paired-ratio relative MAD is 0.69% / 0.46%, but the normalized p90-minus-p10 spreads are **14.04% / 19.77%**, above the unchanged 10% bound. Reference/parallel wall medians are 4766.774/1693.422 ms and 4774.984/1694.614 ms. It stops in the par_layout compute cohort before a qualified table; this run does not qualify the later I/O/mixed cohorts. The cause is unisolated. This is distinct from the earlier #32 mixed-total observation and is not evidence of a default/off regression or of a host-only cause. No thresholds were loosened and no workflow rerun was used to hide it.

## Agent review

- Scope: `runtime_split_review` (Codex reviewer; exact model identifier unavailable), all 18 initial files at `33ed2c00..4df2456a`, followed by the documentation correction delta `4df2456a..cce8256a`. Applied the relevant A/D/C/T/R/M/V completion checklist groups. This is scoped control/consumer review, not a whole-compiler soundness or universal-performance claim.
- Checks: Reviewed pruning, original joins, refusal ABI, SCC/frontier callback routing, graph reuse, oracle/corpus collection, scope exclusions, and append-only memory. Independently checked `git diff --check` and initial CSV medians. The reviewer ran no tests or benchmarks; execution evidence above was run by the implementing agent and CI. Memory lint on the delivered head: 131 nodes, 0 fact files, clean.
- Findings: All reported documentation issues were fixed and independently rechecked at `cce8256a`: policy-dependent actualization reports, optional clone/frontier routing, unfiltered test-helper descriptions, current startup behavior, and exploratory measurement wording. No remaining finding within the review scope. CI limitations are stated above.

---

## #34 Stabilize Windows qualification with W=3 and native controls

State: closed | Merged: yes, 2026-09-11 | Head: codex/windows-bench-stability | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

Hosted Windows compute and mixed cohorts intermittently exceed the former
absolute spread limit. The qualification now uses W=3 on the four logical
processors, leaving capacity for other VM work, and interleaves the existing
scalar Rust/Rayon layout twin as a multithread control in every cohort. The
full affinity mask and normal process priority remain. This is benchmark
resource allocation; production worker defaults and runtime code are unchanged.

The numerical stability limits remain **0.05 MAD and 0.10 spread**, now as
percentage-point excess of the WF/reference distribution over the
native/reference distribution. This changes the criterion from absolute to
relative; it does not claim the old absolute limits passed. The summary
prints unadjusted paired and raw wall distributions alongside the excess.
Individual invalid timings, failed outputs, and missing controls fail.
All five cohorts, fifteen Whitefoot pairs, two warmups, one possible retry,
and the independent speed ceilings **0.90 / 1.10 / 0.95 / 0.95** remain.
Native samples are additional controls, not added Whitefoot rounds or pairs.

The optional `compare_windows_workers=true` dispatch runs Windows only.
Fifteen balanced rounds share one sequential reference and one W=3 native
control between W=4 and W=3 candidates, with no retries. W=4 is diagnostic;
W=3 must qualify. IO-only rows repeat the same configuration because those
children do not use `WF_WORKERS`.

Head: `145c0f771f9acb175ebe126462abc9c9027a9c47`; base: `33ed2c000f7ae2f097bb9c3deea9f8d90267be2a`.

The exact-head [same-VM comparison](https://github.com/mbbill/Whitefoot/actions/runs/34561778413/job/103145828302)
passes W=3 qualification on an **EPYC 9V74**, with **4 visible logical
processors / 2 guest-reported cores**, mask `0xf`. It records 300 samples
across all five cohorts, with fifteen observations per child configuration.
Raw wall MAD is MAD/median; spread is (p90-p10)/median, both in percent:

| Cohort | Before W=4 MAD | Before W=4 spread | After W=3 MAD | After W=3 spread |
|---|---:|---:|---:|---:|
| compute | 3.02% | 29.39% | 2.49% | 14.13% |
| io-warm | 1.45% | 8.56% | 3.32% | 7.99% |
| mixed-iocp | 0.47% | 1.69% | 0.87% | 2.63% |
| mixed-full | 1.57% | 12.81% | 1.19% | 6.19% |
| mixed-total | 1.83% | 14.42% | 1.34% | 4.12% |

These raw widths are not the paired qualification statistics. The paired
MAD/spread cells below are percentages; excess cells are percentage points:

| Cohort | W=4 paired MAD / spread | W=3 paired MAD / spread | Native paired MAD / spread | W=3 excess MAD / spread |
|---|---:|---:|---:|---:|
| compute | 2.79 / 25.08% | 1.15 / 14.04% | 0.44 / 4.17% | 0.70 / 9.87 points |
| io-warm | 2.87 / 12.34% | 2.06 / 12.43% | 1.95 / 10.10% | 0.11 / 2.33 points |
| mixed-iocp | 0.26 / 1.62% | 0.33 / 1.76% | 0.71 / 9.83% | 0.00 / 0.00 points |
| mixed-full | 1.42 / 11.97% | 1.09 / 5.34% | 2.41 / 11.75% | 0.00 / 0.00 points |
| mixed-total | 2.38 / 14.31% | 1.19 / 3.47% | 1.53 / 8.51% | 0.00 / 0.00 points |

Compute's 9.87-point excess is close to the 10-point limit, not evidence that
its variation disappeared. Its median rises **13.7%**, from 2024.194 ms at
W=4 to 2301.111 ms at W=3. Mixed-full and mixed-total medians decrease from
210.471 / 206.555 ms to 180.979 / 176.740 ms. All unchanged speed ceilings
pass. The diagnostic includes an extra W=4 candidate set and takes **8m48s**;
it is not the default job's duration measurement.

The exact-head [default W=3 qualification](https://github.com/mbbill/Whitefoot/actions/runs/34562388812/job/103147616643)
passes every cohort on attempt 1 in **6m15s including build**, satisfying the
default job's approximately eight-minute envelope. This is a separate EPYC
7763 guest, again reporting 2 cores / 4 logical processors, not the VM used
for the before/after table. Its paired MAD/spread percentages are compute
0.41/2.24, io-warm 1.61/9.32, mixed-iocp 0.29/1.38, mixed-full 0.46/2.04,
and mixed-total 0.49/1.40. All five satisfy even the former absolute bounds
in this run. Every raw output check and positive-grant assertion passes.
Only this completed Windows job is claimed here; unrelated Linux/macOS
measurement jobs in the general workflow are outside this qualification.

The prior ordinary protocol at `a93e4c1c` passed all five first attempts in
**6m18s including build** on an EPYC 7763 guest reporting 2 cores / 4 logical
processors. Its native median CPU/wall ratios were 2.72–2.79, with a 957 ms
compute control and 172 ms short controls. Mixed-total still had **13.54% WF
paired spread**, against **10.02% native paired spread**: **3.52 points of
excess**. Its raw wall spreads were 17.41% and 5.84%; those are different
statistics. [Raw artifact and Windows job](https://github.com/mbbill/Whitefoot/actions/runs/34560763765/job/103142831746).

An independent native slowdown supports variable CPU availability during
one interval. Another interval has a quiet control followed by two slow WF
children; WF/runtime-specific or I/O effects are not excluded there. The
data supports the stated relative-margin qualification, not a proof that
all variability is host noise or that finite tests guarantee future passes.
W=3 alone had already failed a subsequent comparison; that failure, the
predeclared native-control criterion, complete first-run distributions and
selection limits are retained in
[RESULTS.md](https://github.com/mbbill/Whitefoot/blob/145c0f771f9acb175ebe126462abc9c9027a9c47/research/investigations/io-model/RESULTS.md#windows-hosted-worker-comparison-criterion-2026-09-11).
The earlier rendered excess columns had an integer-overload display bug;
this head fixes it. The gate arithmetic and raw samples were unaffected.

## Agent review

- Scope: Codex subagent `runtime_split_review` (model identifier not exposed),
  full `33ed2c0..145c0f77` seven-file diff, applicable A/D/C/T/R/M/V groups.
  No compiler, runtime, specification or conformance change. The reviewer
  independently recomputed the native trial's 225 raw samples and the final
  comparison's 300 samples, inspected
  duration/topology metadata, and checked the connected native control,
  sample validation, ordering, unchanged speed ceilings and documentation.
- Checks: exact-head root `make check` **PASS**;
  [CI gate](https://github.com/mbbill/Whitefoot/actions/runs/34561658916)
  **14/14 PASS**;
  [Linux/Windows native runtime checks](https://github.com/mbbill/Whitefoot/actions/runs/34561658912)
  **2/2 PASS**. Local Cargo tests all used `--profile gate`; a temporary Cargo
  wrapper supplied that profile to maintained research commands that omit it,
  retaining debug assertions and overflow checks. PowerShell 7.6.6 parser and actual-function mock checks
  passed for order/counts, missing controls, individual nonfinite timings,
  relative boundaries, exact two-attempt failure, unchanged compute ceiling,
  and fractional summary rendering. The old rendering fails the latter
  check. Native scalar sequential/parallel controls match both existing fold
  oracles; native `cargo test --profile gate` builds successfully (zero unit
  cases), and formatting passes. Final `mcts-mem lint` passes (131 nodes);
  `git diff --check` passes. No test or cohort retired.
- Findings: early ordering, individual-value validation and reporting defects
  were fixed and reviewed. No remaining content findings in the reviewed
  scope. Causal and finite-sample limits are stated above.

| Root `make check` stage | Result | Wall |
|---|---|---:|
| repository-invariants | PASS | 1 s |
| spec-append-only | PASS | 0 s |
| spec-prose-integrity | PASS | 0 s |
| conformance structure | PASS | 1 s |
| compiler (build, format, lint, tests, docs, spec, completion) | PASS | 317 s |
| research-tests | PASS | 12 s |
| bench-programs | PASS | 0 s (cached) |
| full native conformance adapter | PASS | 103 s |
| snapshot corpus | PASS: 484 / 0 flips | 22 s |

---

## #35 Add the design trees and their procedure, migrate mcts_mem, and retire the compiler README

State: open (draft) | Merged: no | Head: claude/code-space-constraints-mdagkl | Base: main | Created: 2026-09-11 | Closed: -

## Change

`mcts_mem/` recorded decisions with their full history and had no consumer that could fail, so it drifted. This PR adds `design/` as its successor, completes the move, reviews every node with the owner, and retires the derivation ledger the specification used to require.

**The trees and the procedure.** `design/language` (checked against the specification) and `design/compiler` (checked against the code) are concept-organized trees of live decisions. A node is a file named for the decision it owns and holds only `Decision:` lines, each with its reason after `because` or its refused alternative after `instead of`, plus an optional `Rejected:` list. `design/log.md` has one entry per approved tree change. `design/skill/SKILL.md` owns the procedure (discuss, plan the tree diff, design gate, implement, PR-time correspondence check) and `design/skill/lint.py` checks the form; `make design-lint` is a `make check` and `make static` stage, and with `--base` it requires every changed node to be named in an added log entry. The owner's rule for the whole skill: anything with no rule that maintains it will rot and does not exist.

**Migration, then the owner's review.** All thirteen `mcts_mem/` subtrees were migrated (47 nodes, 136 decisions, 83 rejected alternatives), and the owner then went through both trees node by node with the agent, root to leaves. The result is 37 nodes (29 language, 8 compiler), 112 decisions, and 53 rejected alternatives. What went: the optimizer facts-on/facts-off decisions in both roots (the owner's intent was never a pair of builds but the absence of any behavior-changing mode; what the compiler hands LLVM is an implementation detail), tautologies and history (the checker as trusted computing base, serialization and replay, a Python reference model, claim-era trap alternatives), designs for things that do not exist (a totality fact, library-chosen representations, a future foreign boundary), restatements of root rules in children, rejected lists that only repeated their decision, and one out-of-date decision (automatic parallelization is not a direction). What changed: the language root now states one behavior per accepted program, that a language choice is made on its merits and never on migration cost, corpus frequency, change effort, or how a model writes today, and that the specification and the tree are the two records with no ledger between them; `pure` is recorded as promising nothing about termination; the contracts law decision says what it meant; the system-interface root says that every system object is an ordinary owned object, that no language mechanism is invented to express system state, and that a system object lacking a capability is redesigned as a better ownership relation rather than patched with interior mutability. Ten single-decision or restating nodes were folded into their parents. Every round is a `design/log.md` entry.

**Specification amendment: v0.53 (rule 4 of the branch-and-main boundary).** The active specification is redeclared v0.53 and the outgoing v0.52 bytes are archived unchanged as `spec/kernel-spec-v0.52.md`. Delta: rules -1 (`[META-6]`, which required a rule-to-ground index at `spec/derivation/derivation-ledger.md`, is removed, and META-1's cross-reference to it goes with it); tokens, spellings, and exceptions unchanged. Selection ground: the owner ruled that the specification and the design tree are the two records and must be consistent with each other, so a third record that nothing consumes only drifts. Conformance evidence changes with it: the META-6 row of `tests/conformance/manifest.jsonl` (an annotation row, not a case) is removed, and coverage stays complete at 160 of 160 rules. `spec/derivation/` is deleted (the ledger and its v0.2 predecessor); the Featherweight-Rust reconciliation memo that shared the directory moves to `research/notes/`. The `whitefoot-spec` gate keeps its identity, unique-rule-id, and cross-reference checks and drops the index coverage checks and their tests; the compiler's qualification tripwire moves to v0.53. Guidance that pointed at the index now points at the design trees.

**Compiler rules move to the tree.** The owner ruled that compiler implementation rules belong to the tree that is checked, not to the agent instructions: `CLAUDE.md`/`AGENTS.md`'s compiler-rules section is now a pointer to `design/compiler`, and its two repository rules moved to the hygiene section. The ripgrep umbrella target moved from the compiler root to `research/experiments/README.md` as a project direction.

**Compiler README retired.** Its run instructions are in the root README (including the three `--par` grain controls and the runtime settings), its known defects and gaps in `docs/todo.md`, its decisions in `design/compiler`, and its implemented-surface inventory is replaced by the conformance report. Every live reference was repointed; the review checklist's memory section is now the design-tree section (M1 tree before code, M2 correspondence, M3 form), and memory-era wording is gone from the guidance files.

**Merged with main.** PRs #31 to #34 landed while this branch was open. The merge keeps both gate additions (`design-lint` and `bench-programs`) and carries the paragraphs main had added to the compiler README into the new homes: the current-stack runtime, the grain controls, and the startup fallback in `design/compiler/parallel-lowering`, the connection-concurrency gap and inert `WF_STACKS` in `docs/todo.md`, and the scalar-leaf remeasurement protocol in `research/investigations/proof-derived-parallelism/bench/PROTOCOL.md`.

**PR-time checks.** Before the owner's review, two agents ran the procedure's design-gate and correspondence checks over both trees (the language tree against the specification, the compiler tree against the code, C1 to C3 over this PR's own code diff). They found one real drift, `design/compiler/cleanup-traversal` migrated from a memory record the owner's 2026-09-04 ruling had superseded, which was rewritten from the emitter's recorded ruling and then confirmed by the owner; their wording findings were applied. The owner's node-by-node review is the review of record for the final tree.

**For the owner.** `design/zh-tmp/` is the temporary Chinese rendering with a `> 通俗解释：` line under every decision, rejected alternative, and log entry; it was kept in step with every round. Delete it when done with `git rm -r design/zh-tmp`.

## Not done, and why

- `mcts_mem/` is not deleted; the owner chose to keep it for a while now that the trees are reviewed.
- Research investigations and governance evidence keep their historical mentions of `mcts_mem/`, the compiler README, and the ledger as dated evidence.

## Validation

- GitHub gate on the head `2c1bd39`: static, unit, sampling, corpus, conformance, research, bench-programs (Linux and macOS), completion-linux, and completion-windows all green.
- Local `make check` on `2c1bd39`: every stage green with two environment notes. Three corpus tests (`programs::traversal` and `programs::wfgrep` denied-path cases) refuse to run as root by design and pass when re-run as an unprivileged user; `research-tests` needs the experiments' dependencies fetched once in this offline container and then passes. `whitefoot-spec`: v0.53, 160 rules. Conformance structure: 160 of 160 rules covered. Conformance run: Pass=747, Xfail=1, Skip=1. Snapshot run: Pass=484, Flip=0.
- `make design-lint` on the head: 37 nodes, depth 3, 112 decisions, 53 rejected; the log-per-change check ran against `origin/main`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01DmvaM2XFNArVZGTAk9hNHr

---

## #36 Add the compute scoreboard: four kernels against Rayon, Parlay, oneTBB and native references

State: closed | Merged: yes, 2026-09-11 | Head: compute/scoreboard | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

The target is that a Whitefoot program built by this tree's `whitefootc` with plain `--par` is the fastest thing in its row. Nothing in `main` could say whether that holds; the comparisons lived in the research bundle on `codex/compute-runtime` (76 files, C adapters labeled as Whitefoot, calibrations and diagnostics mixed with the kernels). This adds `research/experiments/compute-bench/`, the scoreboard re-cut small enough to read in one sitting: 27 files, one harness, one scheduler boundary (`map` and `fork2`), four kernels, one independent oracle each, and references on oneTBB 3046c8b0, ParlayLib 51017699, Rayon 1.12.0 (join, and the parallel iterator where the kernel has one), a static pthread pool and a serial loop. Outputs are checked bit for bit against the oracle after every timed call.

The `wf` row is the compiler-emitted module linked with the runtime sources `whitefootc` itself links, through a host adapter of at most 18 lines of IR. The emitted module carries weak no-op stubs for every `wf__par_*` symbol, so a link that lost the scheduler would run correct and silently sequential; the harness references `wf__par_grants` (no stub) and the Makefile asserts strong `wf__par_publish` and `wf__par_split_budget` after every link, on ELF and Mach-O. The sequential control is the same source under `--no-overlap`. Widths 1, 2, 4, 8 are filtered by the online processor count; a width above it is emitted but marked oversubscribed and gets no verdict. Sizes come from the chunk count the split admission rule gives at the highest recorded width (chunks = 2^floor(log2(min(16·lanes, span/ceil(1200000/weight))))), inside a 5 to 60 ms sequential window; the `wf` row prints the chunk count it was given. Five alternating passes, medians, MAD and p10..p90 as information; the ratio is the median of within-pass paired ratios against the best width-matched parallel reference. Nothing fails on a ratio, a spread or a time. `compare` never overwrites an earlier table. The compiler binary is a normal prerequisite of every emitted module: a rebuilt `whitefootc` re-emits them (the first cut had it order-only, and the defect that caused is on the record below).

References are built at `-O3`, `-march=x86-64-v3` on x86_64, loop alignment, vectorization off; the WF module and runtime are built exactly as `whitefootc` builds them (`-O2`, no `-march`). The README states that asymmetry and discloses each reference's grain as the old bundle's panel setting, not a per-host optimum. The gate runs only `programs-check` (both modules of each kernel with publish-site greps) through the existing `bench-programs` stage; a record-only `compute-bench` workflow runs deps, build, verify and compare on ubuntu-24.04 and macos-14 and fails only on deps, build or verify.

Four tables are in `research/investigations/compute-runtime/RESULTS.md`, each with its host block, run id and a reading. On the development host (Linux x86_64, 4 logical CPUs): the baseline at 33ed2c00, and the merged tree re-emitted at 11d1e4a2. Recorded W=4 block, WF over the best reference, merged tree: FIR 0.978 (fastest, 3/5 passes), Mandelbrot 1.034 (oneTBB, 16 chunks), records 1.138 (oneTBB), quadrature 1.581 (rayon-join; plain `--par` offers every recursion node while every reference stops at depth 8; W=1 `wf` 17.206 ms against `wf-seq` 16.305 ms). The first merged-tree table, recorded at ed7eb0a7, is replaced: it had timed quadrature on the module the 33ed2c00 compiler emitted (three acquisition sites per node, no scalar leaf pruned), because the Makefile named the compiler only as an order-only prerequisite; that module read 24.870 ms and 4.930 at W=4 and 21.710 ms at W=1, so PR #33's scalar-leaf default is worth about 3x on this kernel and the earlier table could not see it. From the first hosted run of the `compute-bench` workflow, run 34574271919 at 5dd1eb7b, both legs green, on the same eight module hashes: `ubuntu-24.04` (4 CPUs, recorded W=4): no WF row is fastest — FIR 1.117 (static), records 1.113 (oneTBB), Mandelbrot 1.178 (oneTBB), quadrature 1.432 (rayon-join). `macos-14` (3 CPUs, recorded W=2): records 0.866 (fastest, 5/5), FIR 0.954 (fastest, 3/5), Mandelbrot 0.984 (fastest, 3/5, interval straddles one), quadrature 1.565 (static). That is the work the next changes have to remove; this change only makes it visible.

No specification or conformance change. The three carried Whitefoot sources are byte-identical to the bundle's; `fir.wf` is new (the bundle's tile-recursion FIR does not split as an independent map). One dated measurement is appended to `mcts_mem/whitefoot/parallelism.md`.

## Agent review

- Scope: Claude (this session) with Opus subagents; 395042f4..c0ff6d4e; every file. The bundle was specified from seven structured maps of the old bundle, the specification was reviewed by a critic who reproduced its load-bearing claims on this host (17 defects fixed before implementation), the implementation was reviewed by three independent reviewers (clean-scratch run, specification conformance, fairness and honesty) and their findings repaired and rechecked to zero, then ten remaining notes were fixed. The runtime design review that followed the first tables found the order-only prerequisite by comparing module hashes across the two local manifests; the fix and the re-recorded table are the last commit.
- Checks: `cargo build --profile gate` of the compiler; `make deps build verify programs-check` in the bundle (verify: 87 cells, every `VERIFY PASS`, split fixtures confirmed chunked at widths 2 and above); root `make static` and `make bench-programs` pass; after the prerequisite fix, `make build` re-emitted all eight modules (dry run and real run), the quadrature ledger reports `scalar leaf limit 16: omitted 2 compute offers` for `adaptive` and `integrate`, `@wf_adaptive` carries one `wf__par_acquire_lane` site, `make verify` passed on the re-emitted images, and `make compare PASSES=5 CALLS=5` produced the recorded table with module hashes equal to the hosted ubuntu-24.04 run's. After the reducer change, `compare PASSES=2` passes and the re-reduction of the same raw samples is number-for-number identical to the old table. Both link assertions were demonstrated to fail on an image linked without the scheduler sources, including the case where only the first of the two strength greps fails. The full root gate was not run locally (three program cases need a non-root user here); the gate workflow on this head is the full-gate result. The `compute-bench` workflow ran on 5dd1eb7b (run 34574271919) with deps, build and verify passing on both legs; the Makefile change re-triggers it on this head.
- Findings: none open within the reviewed scope. Two things to know: the `static` reference is a regular-work reference and not a dynamic-scheduling ceiling (its oversubscribed W=8 cells are extreme by construction and get no verdict), and the Mandelbrot chunk count of 16 at 98,304 points is the compiler's choice under the admission rule, recorded in the table's note column rather than repaired by any knob.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #37 Give user recursion the grain the loop splitter already has: a runtime-derived recursion budget, on by default under --par

State: closed | Merged: yes, 2026-09-11 | Head: compute/frontier-default | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

The compiler side of the quadrature result, on top of the merged scoreboard (#36).

Under plain `--par`, three of the scoreboard's four kernels reach the runtime through the loop splitter, which asks `wf__par_split_budget` once at loop entry and stops offering at a budget; user recursion had no budget at all and offered at every node of a ~150,000-node tree while every reference stops forking at depth 8. That is why quadrature read 1.4 to 4.9 times the best reference and slower than its own sequential build at one lane.

This gives every ordinary cyclic call-graph component one synthesized budget-carrying clone family, the splitter's shape applied to user recursion: each member gets a variant with one hidden trailing `i64` budget (legal because the variant is synthesized, never a source function); intra-component calls and published callbacks carry `budget - 1` and go to the existing sequential clone at zero (no scheduler test, no null branch, no phi below the cut); the component's ordinary entry is a trampoline that obtains the initial budget from the runtime once per entry. `wf__par_recursion_budget()` in the scheduler core returns `floor(log2(64 * lanes))`, clamped to 24 — 7 at two lanes, 8 at four, 9 at eight — with the sweep that selected 64 leaves per lane in the constant's comment. Component discovery is unchanged: may-suspend members, completion pipelines and synthesized functions exclude their component, and `frontier.rs`'s synthesized-function exclusion is what keeps the three map kernels' modules byte-identical (verified by hash in every run). The old N-private-layer emission is deleted; `--par-recursive-frontier auto|N|off` is one mechanism with three starting values (`auto` is the default under `--par`, `N` pins the budget, `off` is the previous behaviour and the A/B control). The `--par-ledger` output names, per component, the members, the budget mechanism, and the components excluded with the reason. This selects actualization only, never acceptance; nothing in it is a timeout, fuel or work budget on a proof path.

Measured, plain `--par`, quadrature `wf` over the best per-pass reference (paired passes lower):

| host | before (every node offers) | after |
|---|---|---|
| hosted ubuntu-24.04 W=4 (runs 34592005664 → 34597909514) | 1.386 (0/5) | **0.974 (5/5), fastest form at W=2, W=4 and W=8** |
| hosted macos-14 W=2 | 1.515 (0/5), W=1 84 % over sequential | 1.004 (2/5) fastest median, W=1 3.4 % over |
| development host W=4 | 1.564 (0/5) | 0.987 (3/5); three readings of the same bytes 0.966–1.024 |

W=1 goes from 5.6 % above the sequential control to 0.3 % below (the pool-off world pays nothing); W=4 process CPU from 1.81 to 1.17 times the sequential control. Mandelbrot, records and fir modules unchanged. The fixed-depth sweep that preceded this (depths 4/6/8/12, on the record) showed the best depth moves with the width, which is why the depth is a runtime value and not a compile-time constant; module size stays at two copies (quadrature-par.ll 21.6 KB → 23.1 KB) where depth 8 as layers cost 82.8 KB.

No specification or conformance change. Records: seventeen dated sections in `research/investigations/compute-runtime/RESULTS.md` (the sweep, the A/B runs, the plain tables, the four hosted tables) and four dated facts in `mcts_mem/whitefoot/parallelism.md`. The bundle gains `WF_PAR_CONTROL_FLAGS`, an A/B-only handle that re-emits when it changes and is never set in a recorded table.

## Agent review

- Scope: Opus subagents under this session's direction; every commit on `compute/frontier-default` beyond the merged scoreboard (3c35c6ca..78867abd); every file. Design from a written brief that read the runtime, the emitter and the references first; each step one variable with its own tables.
- Checks: `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean; `cargo test --profile gate` in `compiler/`: 1585 lib tests and all suites pass, the only failures being the three root-only `programs::traversal`/`programs::wfgrep` unreadable-path cases (they refuse to run as root by design); new coverage: emitter matrix over `off`/pinned/runtime-derived × self- and mutually-recursive components × scalar and destination results, exclusions, ledger text, the scheduler probe for the budget (lanes 1/2/4/8/16 and the clamp), and `tests/programs/adaptive_quadrature.wf` built `--par` and `--no-overlap` at `WF_WORKERS` 1/2/4 with byte-identical output; the module-wide `alloca` count test (`handing_a_call_out_adds_no_stack_slot`) untouched and passing. Root `make static` and `make bench-programs` pass; the gate workflow is green on the head; the compute-bench workflow ran on both legs (runs 34592005664 and 34597909514).
- Findings: none open. Two things to know: the development host's run-to-run spread on identical bytes is up to 14 percent on the quadrature W=4 row, so the hosted runners are the quiet reading; and the ubuntu runner pool is heterogeneous (two machine classes seen), so tables are compared only within a run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #38 Record process CPU in the scoreboard and size the compute spin bound to a measured park-and-wake

State: closed | Merged: yes, 2026-09-11 | Head: compute/wait-path | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

Two things on the wait path, each one variable with its own tables, on top of the merged recursion budget (#37).

**The scoreboard records process CPU.** `raw.tsv` carried wall, outputs and steals only, so the owner's process-CPU target (WF at most 1.2 times the best reference) could not be read from any table. The harness's `time` subcommand now reads `CLOCK_PROCESS_CPUTIME_ID` (every thread summed; `getrusage(RUSAGE_SELF)` where a host lacks the clock) nested inside the same wall interval, and records it as `cpu_ns` beside `wall_ns`; `reduce.awk` prints a `cpu_us` median per row and, on the `wf` row, a `cpu_r` beside the wall `ratio`, paired the same way (within each pass, WF's CPU against the CPU of the reference that was fastest by wall in that pass). No runtime, compiler or emitted module changes; a `PASSES=2` before/after pair on one tree agrees within host spread (median absolute change 3.07 percent over 87 rows, signed mean +1.04 percent), and the two nested reads cost 757 ns, 0.016 percent of the smallest median.

**The spin bound is sized to a measured park-and-wake.** `WF_PAR_SPIN_ROUNDS` was 4,096 and had never been measured: a count of misses standing in for a length of time, with a number on neither side. `compiler/src/backend/sched/wake_probe.c` parks and wakes through the core's own `wf__par_signal`, `posted` flag and `wf_prim_wait_sleep` with two threads alternating, and separately times uncontended `wf__par_find` rounds over a four-lane pool: on the four-CPU Linux development host a park-and-wake of 16.3 us (median of seven runs of 2,000 parks) and a spin round of 18.9 ns, so 256 rounds is a 4.8 us window, 1,024 is 19.4 us and 4,096 is 77.4 us. The scoreboard swept all three at `PASSES=5`: 256 is excluded on the wall (quadrature W=4 from 1.013/0.925 to 1.182 with none of five paired passes lower); 1,024 regresses no kernel's ratio beyond its own MAD at any width and improves fir in every reading at every width (W=4 0.983/0.925 to 0.905/0.907), records at W=4 (1.180/1.223 to 1.175/1.174) and mandelbrot at W=8; its cost is mandelbrot's W=4 process CPU, 24.6/23.7 ms to 27.3/27.4 with the wall flat, which points at a time-shaped idle residency that the brief's next step needs a cadence column for. The constant is now 1,024 with both measured numbers in its comment; yields stay at 16. The probe is a measurement, never a timed row.

Also on the record and not landed: replacing the per-round full deque scan with one pseudo-randomly drawn victim. A full-scan round is 18.9 ns and a probe round 3.2 ns, so an unchanged round budget is a different residency (77 to 13 us), not one variable; the tables and the reason are in the record, and the code is kept outside the tree.

Hosted reading on the flipped-default runners (compute-bench runs 34602028663 at 4,096 rounds and 34605951274 at 1,024, ubuntu-24.04 W=4, both with the CPU column): quadrature 0.998 (3/5, `cpu_r` 1.013) and 0.978 (5/5, 0.971); the two runs landed on different runner classes, so their other rows are read within each run only (run 34602028663: records 0.965 (5/5) `cpu_r` 0.954, fir 0.891 (5/5) 0.801, mandelbrot 1.060 (1/5) 0.993).

No specification or conformance change; `prim_windows.c` untouched (the CPU clock is in the bench harness, not the runtime). Records: three dated sections in `research/investigations/compute-runtime/RESULTS.md`, three dated facts in `mcts_mem/whitefoot/parallelism.md`.

## Agent review

- Scope: one Opus subagent under this session's direction; d47223c0..ecd86a6c; every file.
- Checks: `cargo fmt --check` and `cargo clippy --all-targets` clean; `cargo test --profile gate` in `compiler/`: all suites pass with only the three root-only `programs::traversal`/`programs::wfgrep` unreadable-path failures; the new `sched.rs` wake-probe test passes; root `make static` and `make bench-programs` pass; the gate workflow is green on the head (run 34605951288) and the compute-bench workflow ran on both legs (run 34605951274).
- Findings: none open. The development host has one thread per core, so the SMT half of the idle-spin hypothesis is untested here; the hosted ubuntu runners (two cores, four threads) read the same quadrature standing at 1,024 as at 4,096.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #39 Digest the io experiment branches and the prior compute bundle into the investigations before their branches go

State: closed | Merged: yes, 2026-09-11 | Head: research/branch-salvage | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

Fifty-nine `codex/io-*` branches form one linear investigation stack whose notebook (68 numbered experiments, 10,318 lines on `codex/io-runtime-followup`) and 33 dated facts existed only on branches, while `main` retired the runtime they measured on 2026-09-10 and kept almost none of the findings. `codex/compute-runtime` (PR #28) likewise carried the justification for decisions `main` integrated. This puts what survives into the tree, as digests with numbers and verdicts rather than a copy of the notebook, so nobody has to open a branch again:

- `research/investigations/io-model/SCHEDULER-FINDINGS.md` (new): one paragraph per experiment — question, setup, the notebook's own medians and paired ratios, conclusion, and a verdict: **holds** (48, about something `main` still has), **superseded** (18, about the retired runtime; kept as a closed door with the reason), **unresolved** (5, with what would settle each); ten "not carried" lines with reasons; continuation lowering (experiments 26, 33, 39) in its own section. Every paragraph cites `branch@hash` and the notebook heading.
- `research/investigations/compute-runtime/PRIOR-BUNDLE.md` (new): what the PR #28 bundle decided and measured — the grain panel, the recursion-frontier evidence, the current-stack join decision, the calibrations the scoreboard superseded — and what the scoreboard replaced and why it is not carried.
- Corrections in place: `research/investigations/io-model/RESULTS.md` still predicted reaper-local scheduling, a per-thread ready list and steal-from-idleness as "the next performance work on this line"; experiment 1 built and measured exactly that (paired throughput 0.851/0.809 at four connections, 1.000 at 1024, cross-worker resumes 55.3 to 38.1 percent, CPU up) and it did not pay. The prediction is retracted where it stands, historical sentences kept, following the file's own convention; the memory-node twin gets a dated correction line.
- Memory: 17 dated, sourced facts for the findings that hold — 12 in `mcts_mem/whitefoot/system-interface.md`, 4 in `parallelism.md`, 1 in `data-model/container-representation.md` — append-only in each node's convention. No facts for superseded findings.
- `research/experiments/README.md` and `research/README.md` reach both new files in one line each.

The branches themselves: the sixty-one `codex/io-*` heads and `codex/compute-runtime` are the parents of one archive commit on `archive/codex-io-2026-09-11` (main's tree, 62 parents), so every cited revision stays reachable after the branch names are deleted; the digests say so. Twenty-five of the fifty-nine in-window heads are not ancestors of `io-runtime-followup` because the stack was rebased mid-flight, which is why one archive ref holds all of them rather than the superset alone. No code, script or harness file is carried in; nothing is deleted by this PR.

No specification, conformance, compiler or test change. Merged with `main` at aeaad8d8 and later (the three compute PRs); the one conflict was both sides appending to `parallelism.md`'s facts, resolved by keeping both lists.

## Agent review

- Scope: one Opus subagent under this session's direction over the 61 `codex/io-*` branches and `codex/compute-runtime`, read through `git show` without checkout; 176bb16f..fdfda256 plus the merge; every file.
- Checks: root `make static` passes on the head (repository invariants, spec append-only, spec prose integrity); the gate workflow ran green on 49db3776 (run 34583512816) and runs on the merged head. Numbers in the digests are the branches' own recorded medians and paired ratios; the survey's claim that git retains unreferenced revisions by hash was checked and found wrong, which is what the archive commit answers.
- Findings: none open. Could not verify, and stated as open in the digests: whether the Windows IOCP interleaving experiment 24 repaired is still reachable in `main`'s current-stack runtime; the io_uring SQPOLL barrier (the path is optional and never enabled); which of the compute bundle's screens survived PRs #32/#33.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #40 A/B twin for the compute scoreboard, control handles for the runtime constants, and three sweeps that changed nothing

State: closed | Merged: yes, 2026-09-11 | Head: compute/split-grain | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

No shipped behaviour changes: every compiled program, module and runtime constant is what `main` has (the four emitted `--par` modules and the plain images are byte-identical to the merged tables). What lands is the instrument the remaining scoreboard gaps need, and the record of three experiments that were run with it and selected nothing.

**The instrument.** The bundle can build a second Whitefoot image per kernel, `wf-b`, from the same sources with control flags applied only to it — `WF_PAR_CONTROL_FLAGS` (emission, already in `main`), `WF_RUNTIME_CONTROL_FLAGS` (the runtime objects) and `WF_MODULE_CONTROL_FLAGS` (module and runtime compile) — and run it as its own process inside every pass next to `wf`, so the reducer prints one `A/B wf-b/wf` line per block: the median of within-pass paired ratios, its range, the `lower` count and the paired CPU ratio. The plain image is always built with every control empty; a table with a `wf-b` row is an experiment and never a recorded plain table (the README's rules say so, and `programs-check` reads none of the controls). Each control has its own stamp so a changed value rebuilds exactly what depends on it. The workflow's manual dispatch takes the three as inputs, so an A/B pair can be read on a hosted runner, whose reference rows sit under one percent MAD; pushes stay plain. `WF_PAR_SPLIT_WORK_UNIT` (1,200,000; its origin is `f7127c03`'s crossing measurement, now in the comment) and `WF_PAR_SPLIT_OVERSUBSCRIBE` (16) are `#ifndef`-guarded so a control can reach them; a 587-byte never-called function under `#ifdef WF_PLACEMENT_PAD` in `core.c` moves the runtime's placement for the null-shift measurement and is not compiled otherwise. `wfb_split_chunks` now asks the linked runtime for the chunk count instead of re-deriving the rule.

**What was measured, all on the four-CPU development host, each `PASSES=5 CALLS=5`:**

- Null check, arms byte-identical: eleven of sixteen `wf-b/wf` lines within 1.6 percent of 1.000 with mixed `lower`; the instrument's resolution at two passes.
- Work-unit sweep (1,200,000 / 600,000 / 300,000 / 150,000): mandelbrot's chunks 16 → 32 → 64 → 64 at W=4, its ratio 1.037/1.015/1.030 (three controls) against 1.057/1.067/1.017 — no value below the control band; quadrature, which emits no split call, spread 13 percent across identical-work runs, so separate runs cannot select a five-percent effect. Unchanged.
- Oversubscription cap with the twin ((64, 300k), (256, 75k), (1024, 20k)): mandelbrot W=4 `wf-b/wf` 0.999 (3/5), 0.992 (3/5), 0.970 (5/5), while the null arm quadrature read 0.936 (3/5), 1.086 (0/5), 0.940 (5/5) beside them: the twin removes between-run spread but not the difference between two binaries whose runtime objects differ in placement. Unchanged.
- Placement and alignment: a 587-byte shift of identical code moves a block median up to 11.7 percent (fir W=8) and quadrature's five paired readings span 0.68–1.30 at W=1; `-falign-functions=64 -falign-loops=32` on both arms does not remove it (null W=4 reads 1.057 with 0/5 lower), and the aligned-versus-plain pair sits in [0.954, 1.010] everywhere — harmless and unresolvable. No driver flag changed. One unexplained observation left open on the record: records' `wf-seq` at W=1 reads 28.2 ms aligned against 36.1/35.8 ms plain.

The conclusion the record draws: the gaps left on the scoreboard (mandelbrot 1.02–1.06 against oneTBB at W=4, records about 1.10 on one hosted runner class, fir about 1.02) are below what this host resolves whenever the arms differ in layout; the hosted dispatch is the way to read them.

No specification, conformance or test change beyond the bundle. Records: eleven dated sections in `research/investigations/compute-runtime/RESULTS.md` and seven dated facts in `mcts_mem/whitefoot/parallelism.md`.

## Agent review

- Scope: three Opus subagents under this session's direction, plus the workflow commit by this session; b4c8e747..9c37ffd8; every file.
- Checks: `cargo fmt --check` and `cargo clippy --all-targets` clean; `cargo test --profile gate` in `compiler/`: all suites pass with only the three root-only `programs::traversal`/`programs::wfgrep` unreadable-path failures; `core.c` and `prim_windows.c` compile clean under `x86_64-w64-mingw32-gcc -Wall -Wextra -Werror`, `core.c` also with `-DWF_PAR_SPLIT_OVERSUBSCRIBE=256`; root `make static` and `make bench-programs` pass and the latter builds no B image; re-reducing an earlier plain `raw.tsv` through the new reducer reproduces its table row for row; the gate workflow is green on 297ff377 and runs on the head; the compute-bench workflow runs on the head, and its manual dispatch with inputs has not yet been exercised (validation of the new `inputs` block is the push's run).
- Findings: none open. The one measurement-side item to know: two arms that differ only in layout can read up to twelve percent apart on this host, so a twin verdict on a runtime change is credited only when the null kernel's line sits near 1.000 in the same run.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #41 Hosted A/B instrument: the twin's null check, CPU topology in the manifest, guarded spin constants, and five hosted runs recorded

State: closed | Merged: yes, 2026-09-12 | Head: compute/hosted-ab | Base: main | Created: 2026-09-11 | Closed: 2026-09-12

## Change

Four commits, all measurement and instrument; the shipped runtime's bytes are unchanged (the manifests of every run here record the plain images byte-identical to the null check's).

1. **The A/B twin's hosted null check** (run 34626670962 on main `7c18d3e6`, twin built with the value the runtime already compiles): on the ubuntu leg every twin image is byte-identical to its plain image and fifteen of the sixteen `A/B wf-b/wf` lines sit within 0.8 percent of 1.000, with a one-percent lean toward the twin at W=4 on mandelbrot and fir in all five passes over identical bytes. That is the hosted instrument's resolution at five passes, and it is what the remaining few-percent gaps are read against. The macos leg cannot resolve small effects (null lines 0.862–1.263).
2. **CPU topology in the manifest and guarded spin constants.** `compare` now writes one `topology:` line per online CPU (core, package, SMT sibling list from sysfs; `physicalcpu`/`logicalcpu` from sysctl on macOS) and an `smt:` line, so a hosted table can be classified by runner after the fact. `WF_PAR_SPIN_ROUNDS` and `WF_PAR_YIELD_ROUNDS` gain the `#ifndef` guard the split constants already have, so the twin can be built at another bound through `WF_RUNTIME_CONTROL_FLAGS`; values unchanged.
3. **Five hosted A/B runs recorded** in `research/investigations/compute-runtime/RESULTS.md` (tables verbatim, ubuntu and macos legs) with facts in `mcts_mem/whitefoot/parallelism.md`:
   - Grain (`WF_PAR_SPLIT_WORK_UNIT` 150,000 + `OVERSUBSCRIBE` 32; and 300,000 alone): finer chunks buy mandelbrot W=4 6.4 percent and fir W=4 2.7 percent on a runner whose lanes were co-located on SMT siblings, and nothing on a runner where they were not. No constant is moved yet.
   - Wait path (spin bound 10^9 as a probe, 16,384 and 131,072 as candidates): the per-call raw data shows the plain runtime's two lanes at width 2 running on the two SMT siblings of one core for the first 2–6 calls of a process (wall 1.7–2.2×, CPU doubled) until the kernel migrates one, while the `static` reference — whose helpers never sleep — never does. A longer bounded spin never costs wall at the recorded widths (the sixteen W=2/W=4 lines of the two bounded runs span 0.957–1.005) and gains up to eight percent where lanes are otherwise woken per call, at a runner-dependent CPU cost; the unbounded probe shows one recorded-width cost (records W=4 1.026), and any fixed larger constant collapses when oversubscribed (W=8 on four CPUs: 1.7–6.7×). That selects the rule being implemented next on `compute/idle-window`: spin until a clock-measured idle window when the lane count does not exceed the online CPUs, else today's bound.
4. **Review fixes** (`ce9e9ece`): ten prose-accuracy corrections from the completion review below; no table, host block or recorded number changed.

No spec or conformance content changes.

## Agent review

- Scope: Claude Opus 5 reviewer; `origin/main..compute/hosted-ab` at `aa315c23` (findings fixed in `ce9e9ece`); checked table fidelity against every artifact, ~120 quoted numbers against table.txt/raw.tsv, the status paragraph, the Makefile manifest block under `sh` and `dash` with `set -e` in three sysfs states, the core.c guards against the sched test build, and the repository rules (English, no home paths, no archive/ dependency, no test weakened, no new top-level entries, memory append-only). Skipped: the `mcts-mem-use` lint (skill unavailable here) and the macOS debug-map explanation, which the text itself marks unverified.
- Checks: `make static` green (repository invariants, spec append-only, spec prose integrity); in `compiler/` `cargo fmt --all -- --check` and `cargo clippy --all-targets --locked --offline -- -D warnings` clean; `cargo test --profile gate --lib --locked --offline backend::tests::sched` 4 passed at `119940be`. Full `make check` not run on this branch; the runtime source change is two `#ifndef` guards with unchanged values, and the plain images' hashes on the three runs after it match the null check's.
- Findings: ten prose-accuracy defects (a summary contradicted by its own bullet, a MAD attributed to the wrong row, stale ordinal references in the status paragraph, a lean count, an over-broad "every line", two over-tight quantifiers on the per-call excerpt, a misworded null-pair sentence, an incomplete runner list, and an "each time" false for the probe), all fixed in `ce9e9ece`. All twelve quoted tables byte-identical to their artifacts. No remaining findings within the reviewed scope.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---

## #42 Remove a local machine directory name from the tree and make the invariant reject it

State: closed | Merged: yes, 2026-09-11 | Head: hygiene/scratch-root | Base: main | Created: 2026-09-11 | Closed: 2026-09-11

## Change

The name of one developer machine's antivirus skip directory had been encoded as the default scratch root since the root Makefile landed (`b2e2e267`, 2026-08-27) and was copied into every experiment bundle, two CI workflows and several documents: 20 tracked files outside the frozen `archive/`. The rule is that no directory of any developer's own machine belongs in the repository, and the existing `repository-invariants` check only rejected absolute personal home paths, so this relative name passed it.

- The default scratch root is now the system temporary directory: `WHITEFOOT_SCRATCH_ROOT ?= $(patsubst %/,%,$(if $(TMPDIR),$(TMPDIR),/tmp))/whitefoot` in the eleven Makefiles (the `patsubst` strips the trailing slash macOS puts on `TMPDIR`), `${WHITEFOOT_SCRATCH_ROOT:-${TMPDIR:-/tmp}/whitefoot}` in the two shell scripts, `${TMPDIR:-/tmp}/whitefoot` in documentation recipes. The variable name is unchanged, so an explicit override keeps working.
- `gate.yml` and `io-hosts.yml` export `WHITEFOOT_SCRATCH_ROOT=$RUNNER_TEMP/whitefoot` to `GITHUB_ENV` from a step right after checkout (the `runner` context is not available to a job-level `env:` block; the compute-bench workflow already sets the variable this way) and read their cache and harness paths from it; `compute-bench.yml` and `io-bench.yml` already set their own explicit temp path and are untouched. `compiler/Makefile`'s `COMPLETION_TMP` now derives from `WHITEFOOT_SCRATCH_ROOT` instead of a hard-coded home path, so the io-hosts harness path and the build agree.
- Prose in `ripgrep/PROTOCOL.md`, `proof-derived-parallelism/DESIGN.md` and `spelling-relief-candidate.md` now says "the scratch root" / "the local scratch directory outside the repository". `RESULTS.md` changes only its reproduce-recipe line; no recorded table or host block is touched. `archive/` is frozen and keeps the historical text.
- `repository-invariants` gains a rule of the same shape as the home-path rule: the literal name in any tracked file or filename outside `archive/` fails `make static` (and so `make check`) with the message that a local machine directory name is encoded in the repository. The rule builds the name at run time so it does not match itself. Against `origin/main` content it fails listing the nineteen other offending files; on this branch it passes.
- The compute-bench README notes that the temp directory may be cleared on reboot, so dependencies and results there are rebuilt or copied out.

No spec or conformance content changes.

## Agent review

- Scope: Claude Opus 5 reviewer, read-only, `origin/main..e572ce01`; checked the changed set against the offender set (20 = 20), Makefile expansion under eight `TMPDIR` values, the shell forms under `sh`/`dash`/`bash`, both workflows under actionlint 1.7.7 and a YAML parse with path tracing through `RESEARCH_CARGO_TARGET` and `COMPLETION_TMP`, the new invariant rule (self-match, `archive/` exclusion counts 4287 = 3165 + 1122, status handling for exit 1/128/2, failure against an `origin/main` snapshot listing 19 files, four mutation probes), documentation diffs, and that no test or archive file changed. Skipped: a real Actions run (not possible from the review host).
- Checks: `make static` green; `make -C compiler static` green; `cargo fmt --all -- --check` clean; `make -C compiler completion-test` from an empty `/tmp/whitefoot` passes in 9 s creating the new default; `git diff --check` clean; actionlint clean on all four workflows after the fix below.
- Findings: one blocking — the job-level `env: WHITEFOOT_SCRATCH_ROOT: ${{ runner.temp }}/whitefoot` in both workflows is invalid (the `runner` context is not available there), fixed in `ce33b0f9` by exporting from a step after checkout; actionlint now clean. Informational, not fixed: a `TMPDIR` containing whitespace is not handled by the make default (the common macOS and Linux values are fine; the old `$(HOME)` default had the same exposure); the make and shell spellings differ by a doubled separator for a trailing-slash `TMPDIR` (disclosed in the Makefile comment); the I/O benches' default work root is now the temp filesystem, which their own protocols verify and the README tells the reader to override with a durable root.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---

## #43 Compute runtime: an idle window before a lane parks, a publish epoch, a spin hint, and the split work unit at 150,000

State: closed | Merged: yes, 2026-09-12 | Head: compute/idle-window | Base: main | Created: 2026-09-12 | Closed: 2026-09-12

## Change

The compute scheduler parked an idle lane after 1,024 spin rounds (about 20 µs) and woke it by condvar at the next publish. On the hosted ubuntu runners (two cores × two SMT siblings) the per-call raw data showed what that costs: a woken helper is placed next to its waker and the two lanes run on one core's siblings for the first two to six calls of a process at 1.7–2.2× the wall with CPU doubled, while the `static` reference — whose helpers never sleep — never does. Four runtime changes and one bench change:

1. **Idle window** (`core.c`): a lane with no work stays hot until it has been idle for `WF_PAR_IDLE_WINDOW_US` (1,000 µs, clock-sampled every 1,024 rounds) when the pool's lanes at start do not exceed the CPUs the process may run on; an oversubscribed pool keeps the old bound. Answered once in `wf__par_start`; both wait loops share one helper; zero under `WF_SCHED_TEST` so the enumerator probes see the same states. New primitives `wf_prim_monotonic_us` on both hosts; `wf_prim_online_cpus` now honours the affinity mask (Linux `sched_getaffinity`, Windows `GetProcessAffinityMask` falling through on multi-group processes).
2. **Publish epoch** (`core.c`): one cache-line-aligned counter bumped with release at push, winning steal and signal; an idle lane past the first spin bound spins on one acquire load and rescans only when it moved (55× fewer deque scans per idle gap locally). The ordering argument is in the comment.
3. **Spin hint** (`prim.h`): `wf_prim_spin_hint` was `YieldProcessor()` on Windows and empty elsewhere; POSIX x86 now issues `pause`, aarch64 `yield`. Reasoned, not isolated by a measurement; the comment says so.
4. **Split work unit** (`entry.c`): `WF_PAR_SPLIT_WORK_UNIT` 1,200,000 → 150,000. At the old value the work term afforded mandelbrot 16 chunks at every width, so on an eight-CPU M1 Pro its W=8 wall equalled its W=4 wall while tbb ran 1.6× faster; at 150,000 the count follows the 16-per-lane cap (64 at W=4, 128 at W=8). The one test that pinned the old constant's budgets moved with it; the bench README's rule, chunk table, floors and three kernel comments were re-derived.
5. **Bench** (`compute-bench`): width set gains 16 and 32 with `WFB_MAX_WIDTH` 32 (and the rayon adapter's matching assertion); `deps.sh` stamps the kept rayon staticlib with its source hash so a moved ceiling reaches a host whose build is otherwise kept.

**Evidence** (all recorded in `research/investigations/compute-runtime/RESULTS.md`, tables verbatim, with six facts in `mcts_mem/whitefoot/parallelism.md`):
- Hosted plain run 34639809658 on the runner class where the shipped runtime co-located its lanes (run B: fir W=2 1.290, records W=2 1.109, mandelbrot W=4 1.198): fir W=2 **0.994** fastest (5/5), records W=2 **0.928** and W=4 **0.938** fastest, quadrature W=2/W=4 0.960/0.974 fastest; per-call the 2× slow start is gone.
- Hosted twins against a zero-window control (34638747514, 34640833113, 34642559273) on runners whose lanes ran evenly: neutral within about one percent wall at W=2/W=4, 3–7 percent more CPU at W=4 in the tail spin (cpu_r against the best reference at most 1.062), W=8 never collapses.
- Grain twins: hosted 34644579271 neutral at four CPUs; the owner's M1 Pro (eight CPUs) mandelbrot W=8 **0.692 [0.65-0.75]** 5/5 and records W=8 0.965 at the finer grain, which is what moved the work unit.

**Open, recorded as such:** on the M1 Pro at W=8 the window itself is a loss against the zero-window control (quadrature 0.847, records 0.915, fir 0.963) while neutral-to-positive at W=2/W=4 — the opposite sign from the Linux SMT runners; the likely mechanism is the asymmetric cores (six performance + two efficiency), which the "lanes at most the online CPUs" test does not see; a decision waits for a heterogeneous x86 host. And on Darwin the harness's process-CPU clock does not count every thread (a Whitefoot W=4 row reports one lane's CPU), so no macOS `cpu_r` is read in these sections; fixing the Darwin CPU accounting is a bundle task.

No spec or conformance content changes. No acceptance path is touched; the window, the epoch and the work unit select how an admitted program waits and is decomposed.

## Agent review

- Scope: Claude Opus 5 reviewer, read-only, `origin/main..fe4487de` for the runtime and bench code (the hosted-ab commits merged in were reviewed under PR #41): both wait loops, the epoch's ordering argument and cache-line layout (compiled on gcc and clang: sizeof = alignof = 128), the prim-layer primitives on both hosts including the Windows multi-group case, the spin hint's codegen (`pause`/`rep nop`/`yield` confirmed per target), the width set against every per-width array, the split rule recomputed at 150,000 for all three maps at W=1..32, the cited numbers against the artifacts, and the repository rules.
- Checks: `make static` green after merging current `main` (the new local-directory-name rule included); `cargo fmt --all -- --check` clean; `cargo clippy --all-targets --locked --offline -- -D warnings` clean; `cargo test --profile gate --lib` filters `backend::tests::sched` (4 passed) and `backend::tests::loop_split` (17 passed); `make -C compiler sched-smoke sched-deque-test` green (8 smoke scenarios, deque probe 151,898 steals); mingw cross-compile of `prim_windows.c`, `core.c`, `entry.c` clean at `-std=c11 -Wpedantic -Werror`; compute-bench `programs-check` and full `verify` (87 forms × widths) green. Full `make check` not run on the branch.
- Findings: one blocking (a `loop_split` test pinned the old work unit's budgets; expectation moved with the constant, stated in the commit), three major (the bench README's chunk table, the `core.c` cap rationale and three bench kernel comments still carried the 1,200,000 arithmetic; the `entry.c` claim overstated the cap-bound widths), six minor (a cross-table number pair, a stale width-set comment, `deps.sh`'s lost executable bit, the round-cost figure re-measured after the epoch and the hint — 10.3 µs park-and-wake, 11.6 ns per round on the development host — and the spin-hint prose now saying it is reasoned rather than measured), all fixed in `1f2ffe35`. The cited RESULTS.md sections that were forward references at review time landed in `21867151`/`be9fdc90`. No remaining findings within the reviewed scope.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #44 compute-bench: read Darwin process CPU from task_info

State: closed | Merged: yes, 2026-09-12 | Head: bench/darwin-cpu | Base: main | Created: 2026-09-12 | Closed: 2026-09-12

## Change

**Problem.** The compute bundle's `cpu_us` column was defective on Darwin. macOS
answers `CLOCK_PROCESS_CPUTIME_ID` from the task's accounting for threads that
have *already exited*, so a reference pool whose workers are still alive at the
read contributes nothing and the column read about one lane's worth however many
lanes ran. The recorded Apple M1 Pro tables show it: `tbb` on records W=8 at
5,896 us of CPU for a 2,441 us wall on eight threads, `static` on quadrature W=4
at 2,904 us against a 2,866 us wall, and a Whitefoot quadrature W=4 row at 3,081
us against a 3,033 us wall while stealing a thousand chunks. `cpu_r` was
therefore read nowhere in either macOS section.

**Resulting behavior.** `harness.c` now selects the process-CPU source per host,
the same shape it already used for its wall clock:

- **Darwin:** `task_info(mach_task_self(), TASK_THREAD_TIMES_INFO)` for the
  threads that still exist plus `task_info(TASK_BASIC_INFO)` for the ones that
  have exited, both `time_value_t` seconds and microseconds, summed to
  nanoseconds. A thread's time moves from the first to the second when it exits,
  so it is counted exactly once either way.
- **Elsewhere:** `CLOCK_PROCESS_CPUTIME_ID` where the host defines it.
- **Fallback:** `getrusage(RUSAGE_SELF)`, for a host with neither and for a host
  whose primary source refuses at run time.

The source is pinned by the run's first reading and never changes after it, so a
`before` and an `after` bracketing one timed call can never come from two
different sources; a primary that answers once and later fails ends the run
rather than silently switching. The new `wfb_cpu_clock_name()` resolves the name
by taking a reading, and `do_time` calls it while printing the header, before the
first timed call — so the `cpu_clock=` field of every driver line in `raw.tsv`
names the source that actually answered, not the one the build preferred.

**Rejected alternative, on evidence.** `proc_pid_rusage(RUSAGE_INFO_V0)` was
tried first (`f602d621`). Its `ri_user_time + ri_system_time` are documented as
process-wide nanoseconds, but on the hosted `macos-14` leg of run **34667394566**
every row read 0.02 to 0.07 times its own wall and barely moved with the work —
mandelbrot `wf` at W=2 read 540 to 570 us for walls from 11.7 to 36.2 ms, and
`static` at W=4 read 2,363 us against a 35,089 us wall. Not a unit error: a
24 MHz timebase tick would have put that `static` row near 1.6x its wall. A
figure that does not scale with the work is not a CPU figure, and the standing
rule taken from it is that a per-host CPU source is accepted only on a reading
that scales with the work, never on its documentation.

**Confirmation.** The hosted `macos-14` leg of run **34668036736** at `e51c1ff8`,
a three-CPU runner, printed `cpu_clock=task_info` on all 310 driver lines, and
the column grows with the lanes and stops where the CPUs do:

| row | cpu_us | wall_us | cpu/wall |
|---|---|---|---|
| mandelbrot `static` W=4 | 107,878 | 36,495 | 2.96 |
| mandelbrot `tbb` W=4 | 26,072 | 8,887 | 2.93 |
| quadrature `static` W=4 | 374,802 | 126,768 | 2.96 |
| mandelbrot `static` W=2 | 49,213 | 25,141 | 1.96 |
| the four W=1 `serial` rows | — | — | inside 0.5% of wall |

The wall columns are in line with earlier runs, so the extra Mach calls per timed
call did not move them.

**Limits.** The eight-CPU Apple M1 Pro whose tables carry the defect was not
rerun: both recorded sections keep the readings they were made with, no macOS
`cpu_r` becomes readable, and the first macOS table recorded after this fix is
the one to read `cpu_r` from. The `getrusage` fallback has not been read on
Darwin, so a fallback taken there is a figure of unknown standing — which is why
the printed name says which source answered. No runtime or language rule reads
this column; this is an instrument change only.

Docs and memory follow the code: the `cpu_us` bullet in the bundle README, the
two defect notes and the summary in `research/investigations/compute-runtime/RESULTS.md`,
and the parallelism decision memory.

No spec or conformance evidence is touched.

## Agent review

- **Scope:** agent completion review against `docs/review-checklist.md`; base
  `b4833c98` (origin/main) .. head `6ed227c1`. Groups checked: A (scope/layout),
  D (documentation), C (code), R/M (decision memory), V (validation). T
  (specification/conformance) not applicable — no spec, conformance case,
  manifest, adapter or gate wiring is touched.
- **Checks:** on `6ed227c1`, `make -C research/experiments/compute-bench
  programs-check` passed from cleared stamps (all four kernels re-emitted and
  re-grepped for `wf__par_publish`); `clang -std=c11 -O3 -g -Wall -Wextra -Werror
  -Wpedantic -pthread` plus the bundle's scalar flags compiled `harness.c`
  cleanly; `make repository-invariants` passed; `git diff --check` clean. Not a
  full gate: root `make check` was not run here.
- **Findings:** three were found and fixed in `6ed227c1`, all claims wider than
  their evidence. (1) The confirming hosted run was not cited anywhere — the
  memory entry still read "stands only until the hosted `macos-14` leg prints
  `cpu_clock=task_info`" though run 34668036736 had already printed it; the run
  id and its readings are now in `harness.c`, the README, `RESULTS.md` and a new
  memory Fact. (2) The fallback paragraph asserted that `getrusage` shares the
  POSIX clock's blindness on Darwin, which nothing here has read — the exact
  mistake the rejected first attempt was made of; it now states the standing is
  unknown. (3) The nested-read cost was given as "tens of nanoseconds" in both
  the driver comment and the README against this bundle's own 757 ns measurement
  for the pair; both now quote that figure and name the host. The `harness.h`
  summary, which listed `CLOCK_PROCESS_CPUTIME_ID` twice with a property the code
  does not test, was rewritten. Every fix in `harness.c`/`harness.h` is a comment;
  no executable line changed after `e51c1ff8`, so the hosted run remains a
  reading of the delivered code. Remaining unverified: the `__APPLE__` branch
  cannot be compiled or exercised on this Linux tree — its evidence is the hosted
  `macos-14` build and run at `e51c1ff8`; and the decision-memory lint could not
  be run, since the `mcts-mem-use` skill is unavailable in this session (the
  appended Fact follows the file's existing append-only form and the entry it
  supersedes is left intact).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #45 compute-bench: a sparse call cadence, and what it says about a parked lane

State: open | Merged: no | Head: bench/gap-cadence | Base: main | Created: 2026-09-12 | Closed: -

## Change

**The problem.** Every `compute-bench` table so far was taken at one cadence: five calls per process, back to back, with microseconds between them. That is the cadence a runtime which keeps an idle lane hot for a window before it parks is measured at its best in, because the next call always arrives while the lanes are still hot. Real programs have gaps between their parallel regions, and the bundle could not ask what survives one.

**The mode.** `WFB_GAP_US` is a per-process gap, in microseconds, between consecutive timed calls. It is zero by default — unset is exactly what the bundle did before — and it reaches **every form identically, references included**; a gap that reached only the `wf` row would compare one scheduler's idle policy against another scheduler's warm one, which is not a comparison. The wait is a monotonic-clock busy-wait and deliberately not a sleep, so the calling thread stays running as a program doing its own sequential work would while the helper lanes go idle and park, and it sits **outside every measured interval**: both clocks start after it returns, so no part of a gap is in any reported wall or CPU figure.

It is a run-time setting and nothing else — no stamp, no emission, no compile, no link — so a changed gap rebuilds nothing and the images a gapped run times are byte-identical to the ones a gap-free run times. That is what lets it compose with the existing A/B twin, which is the run the mode was added for. `verify` is handed the variable, reports it and waits nothing, because it drives no timed call loop; it is there so a malformed value is refused by the cheap sweep instead of only by a long `compare`. `programs-check`, the one target the repository's `make check` runs, links nothing and runs no image and reads it no more than it reads the three control flags.

**A gapped table can never be read as a plain one.** `manifest.txt` records `WFB_GAP_US` on every run, empty or not. Every process writes the gap it ran at into its own header and trailer in `raw.tsv`, and the reducer refuses to put two cadences in one table — a disagreement is a malformed stream, refused exactly as a header/trailer mismatch is, never a judgement about a measurement. A non-zero gap puts `gap_us=` on the table's `passes=` line with a disclosure line above it. A hosted run takes the gap as the `gap_us` workflow input, beside the three control inputs; a push sets none of the four.

**What the runs say, and what they do not.** Eight sections are recorded in `research/investigations/compute-runtime/RESULTS.md` — three hosted dispatches with both legs each and two local Apple M1 Pro tables. **None is a candidate record**, twice over: each carries a `wf-b` row and each was taken at a non-zero gap. At a 500 us gap nothing is lost. At a 2 ms gap the window goes inert and the compiled forms lose 8 to 18 percent at W=4 on one hosted runner class, while every reference crosses the same gap unchanged. A third twin with a 3,000 us window, whose lanes never park across the gap, reads back among the references — so **the loss is caused by the lanes having parked across the gap**.

**What the park costs is open, and this PR says so.** A later trace instrument on `bench/wake-trace` reproduced the loss on the same runner class and refuted the mechanism the first draft of these sections read into it: at W=4 the wake from the root's first publish is a median 20 to 22 us, helper absence inside a call is 0.3 to 0.4 percent of helper time, and the four lanes sit one per logical CPU and never move, in 75 of 75 traced calls. It is not the wake, not the lanes' absence and not their placement; the cost is paid inside the working portion of the call, with the lanes present on distinct CPUs and each doing the same work slower. The readings, the status paragraph and the decision memory now claim exactly that, and the next work they select is to find why a lane that parked across a gap runs its chunks slower for the rest of the call, with the per-chunk trace as the instrument — not a longer window, whose cost is spent between calls where no column of the table reports it. Every measured number in the sections is unchanged; only the interpretation moved. The decision memory keeps its append-only history: the earlier fact stands as written and a new dated fact corrects it, naming what it supersedes.

No language rule, specification text, conformance evidence, compiler constant or runtime behavior changes here.

## Agent review

- **Scope:** reviewing agent, `b4833c98..ddc15701`, full task diff (workflow, bundle `Makefile`/`harness.c`/`reduce.awk`/`README.md`, `RESULTS.md`, decision memory). Groups A, C, D, T, V checked; R/M checked against observable artifacts only. Skipped as absent: specification amendment, conformance cases, compiler sources.
- **Checks:**
  - `make repository-invariants` (repo root) — pass; `make static` — pass (spec append-only, spec prose integrity).
  - `make -C research/experiments/compute-bench programs-check` — pass, after clearing the stamp directory so all four kernels actually recompiled and both assertions re-ran.
  - `clang -std=c11 -O3 -Wall -Wextra -Werror -Wpedantic -pthread -c harness.c`, and again with the full flag set the bundle Makefile uses — clean both times.
  - `actionlint` on `.github/workflows/compute-bench.yml` — clean.
  - Reducer, run against a captured hosted `raw.tsv`: it **reproduces the recorded ubuntu 2 ms table byte for byte**; a single process retagged to another cadence is refused with `a process at gap_us=0 in a table at gap_us=2000`; a trailer-only mismatch is refused as a header/trailer mismatch; and a stream with the `gap_us` fields stripped, which is every table this bundle wrote before the mode existed, still reduces cleanly and prints no gap line. `awk --lint` clean under `mawk` (no `gawk` on this host, so the stricter lint is unverified).
  - All **eight** recorded table blocks compared byte for byte against the run artifacts and the two local tables — identical. Manifest gap values, control flags, twin flag and all twenty image hashes in the host blocks agree with the artifacts. `git diff --check` clean.
  - Not run here: the full root `make check`, and no benchmark was re-measured.
- **Findings:** three, all fixed in this revision. (1) The readings, the file's status paragraph and the M1 section asserted a placement mechanism the later trace refutes — rewritten to claim the trigger the twin proves and to record the mechanism as open. (2) The fir W=2 per-call series kept its numbers but its "two lanes sharing one core" reading was wrong; corrected, with the one placement effect the trace did confirm (two threads starting on one core for the first warm call at W=2) kept as such. (3) `harness.c` said the gap is read "in every mode"; `list` reads nothing, so the comment now names `verify` and `time`. Unverified: the decision-memory integrity checker is not installed in this environment, so the appended fact follows the node's observable conventions and was checked by inspection only — the earlier fact is byte-identical to its committed form and the file grew by exactly one line.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #46 A compute performance regression check on pull requests

State: closed | Merged: yes, 2026-09-12 | Head: ci/compute-regression | Base: main | Created: 2026-09-12 | Closed: 2026-09-12

## Change

compute-bench could answer "is this tree's `--par` program the fastest thing in the row" and, through its A/B twin, "what would this flag be worth here". It could not answer the question a branch has to answer: **is the program this branch produces slower than the one the branch started from?** This adds that, as a separate required pull-request check.

**The baseline twin.** The twin is the same twin, handed different inputs. Three new variables name where its Whitefoot side comes from, each defaulting to this tree's own:

| variable | names |
|---|---|
| `WF_B_SCHED_DIR` | the twin's `core.c`, `prim_host.c`, `entry.c` and their headers |
| `WF_B_FLOOR` | the twin's `wf_floor.c` |
| `WF_B_WFC` | the compiler that emits the twin's `--par` module |
| `WF_B_SOURCE` | a label for `manifest.txt` (the merge-base sha on a gate run) |

One build path: every rule, link line and post-link assertion is the one the control-flag twin already used, only the inputs move, and the plain image reads none of them. Naming any of the first three is by itself a request for the twin, because a regression run sets no control flag. A changed baseline has to rebuild the twin and two checkouts can carry different bytes under one path, so the existing stamp mechanism is extended with the **content hash** of the baseline sources rather than their paths or timestamps: same bytes under another path rebuild nothing, different bytes re-emit the module and recompile the four `-b` runtime objects and nothing else. `manifest.txt` records `WF_B_SOURCE` always and the paths plus per-source hashes on a baseline run; the table header gains a `WF A/B twin source=` line, without which the reducer would have gone on printing the `WF_AB=1` null-check line that claims the two arms are identical.

**The rule** (`verdict.awk`, the one place in this bundle where a number decides a pass/fail):

```
FAIL when a block's  A/B  wf-b/wf  wall  median < 0.97
     and the baseline was lower in >= 4 of the 5 pairs,
     at W in {1, 2, 4}, oversubscribed blocks excluded.
```

Both halves are required. CPU is a **report**: a paired `cpu` ratio below 0.90 under the same count is printed and marked `*` and changes no exit status, because the reducer prints only the wall `lower` count. W=8 and above are excluded (four-CPU runners oversubscribe there, and oversubscription changes which scheduler wins); macOS is excluded entirely (that runner cannot resolve below about 20%); a block the reducer marked oversubscribed is skipped even at a recorded width. It **refuses rather than passing vacuously**: no A/B line, only unrecorded widths, or fewer than five pairs each exit 2 with `REFUSED`.

**The workflow** (`.github/workflows/compute-regression.yml`): ubuntu-24.04 only, full-history checkout, merge base with the PR base branch exported as a worktree under `$RUNNER_TEMP` (never a repo path), that revision's `whitefootc` built beside it, then one `deps`/`build`/`verify`/`compare` with both arms in the same five passes, artifact upload, then the verdict. Paths are compute-bench.yml's own automatic-run set plus this file; `compiler/src/**` wholesale was rejected because the two arms differ only in the runtime and the emitter, so a parser or diagnostics change emits byte-identical modules and the job would read this host's own spread on most compiler PRs. Concurrency is per PR with cancel-in-progress, and there is no automatic re-run: a job that retried until it agreed would select its own result.

**Not part of `make check`**, which must stay a property of the tree rather than of a runner's load. `verdict-test.sh` is in `make check` (root `research-tests`): eleven crafted table fragments covering pass, regression, each half of the rule alone, no twin, oversubscribed, unrecorded width, empty table, under-powered run and the cpu report. Nothing here changes what the record-only compute-bench workflow measures: it sets none of the new variables, builds no twin, and reads the same numbers.

Material tradeoffs: the check costs a runner slot on PRs that touch the listed paths and reads a 1%-resolution instrument, so it can only catch regressions larger than that; the alternative — a band on a single absolute time — is what an earlier bundle's gating jobs did and is exactly what this avoids. Two standing README claims that the bundle decides nothing are narrowed rather than deleted: every table still decides nothing, and `verdict` is named as the one exception.

## Agent review

- Scope: agent-authored and agent-reviewed; `origin/main`..`d7bf4df1`; every changed file read in full before and after the change (bundle `Makefile`, `reduce.awk`, `README.md`, root `Makefile`, new `verdict.awk`, `verdict-test.sh`, new workflow). No Rust touched.
- Checks: `make repository-invariants` pass; `make research-tests` pass (includes the new `verdict-test`, 11/11); `make -C research/experiments/compute-bench programs-check` pass; `actionlint` clean on all four workflows. Local mechanics proof on a four-CPU Linux host with a fabricated baseline (`WF_PAR_SPLIT_WORK_UNIT=1200000`, a copied sched tree, floor and compiler binary): the twin built from the baseline sources while the plain image built from the tree; `verify` reported the twin's own split floor of 16000 against the plain image's 2000, and the W=8 twin emitted 64 chunks against the plain image's 128, so the baseline constant reached the `-b` side alone; `manifest.txt` recorded `WF_B_SOURCE`, the three paths and the per-source hashes; `compare KERNELS=fir PASSES=5 CALLS=3` produced A/B lines at W=1/2/4/8 and the verdict read `VERDICT: PASS` over W in {1,2,4} with W=8 skipped; a crafted table produced `VERDICT: FAIL` with a non-zero exit and its cpu report; a plain no-twin table produced `REFUSED`. Stamp behaviour proved directly: an mtime-only touch of a baseline header rebuilt nothing, a content change to it rebuilt the twin's module, its four runtime objects and its image and nothing on the plain side. A plain `compare` with no baseline reproduced the old output: no `wf-b` row, no new header line, one added manifest line.
- Findings: the hosted verdict on this PR is itself the remaining unverified item — a PR that changes no runtime code should read all-pass at W<=4, and that run is the resolution check on the rule's band. The stamp hashes every `.c`/`.h` in the scheduler directory, including probe units that compile into nothing, so a probe-only change rebuilds the twin unnecessarily; that is the deliberate side to be wrong on and is stated in the Makefile. `make verdict` reports a refusal and a regression with different text but the same non-zero status through `make`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

## #47 compute-runtime: trace the gapped call, and close the line it opened

State: open | Merged: no | Head: bench/wake-trace | Base: bench/gap-cadence | Created: 2026-09-12 | Closed: -

Stacked on #45 (base branch `bench/gap-cadence`): this PR contains only the trace work, and #45's commit `ddc15701` is merged into it so the sections below are written against the reading that stands. **Retarget this to `main` once #45 merges.**

## Change

The three `bench/gap-cadence` sections read a sparse-cadence loss on the hosted ubuntu SMT runner class, credited it to a woken lane coming back onto its waker's SMT sibling, and selected "find why a parked lane comes back slow" as the next work. This PR builds the instrument that answers that, runs it three times on the hosted class, and closes the line without a runtime change.

**The instrument.** `WF_PAR_TRACE`, gated in `compiler/src/backend/sched/core.c`, with a nanosecond clock added to `prim.h` and both host primitives behind the same guard. Defined, it records per lane a fixed 4,096-entry ring — no allocation, no lock, no shared line between lanes, nothing on the spin path — of call heads with the idle mask, first publishes finding parked lanes, condvar park and wake with rounds and epoch, first steals, every executed chunk with its nanosecond duration and index, a calibration probe, and the release that closes the call, each carrying a monotonic microsecond and `sched_getcpu`. The probe is a fixed 20,000-step latency-bound dependent chain with a compiler barrier per step: not a measure of work done but of how fast the core running it is running. It is dumped to stderr at exit, which `compare` already keeps per cell, so the parsed stdout stream and `raw.tsv` carry none of it and no harness or Makefile change was needed. It reaches the A/B twin only, through `WF_RUNTIME_CONTROL_FLAGS`.

**Undefined — every ordinary build — nothing changes.** The scheduler core, the POSIX primitives and the Windows primitives compiled without the flag are byte-identical to this branch's merge base, `cmp` clean.

**What the three runs settle.** It is not the wake (median 20–22 µs at W=4, 13–18 at W=2, first steal 1–2 µs later), not lane absence (0.15–0.42 % of helper time), and not placement (four lanes on exactly `{0,1,2,3}`, one per logical CPU, no migration, 150 of 150 calls). A core is **not** slow for a period after a 1.4–2 ms park: the first probe out of the park lies within +1.3 to −3.5 % of the probes that follow it on the same CPU. The work is **not** front-loaded and does not decay: a fixed unit chunk takes 255–260 µs in every millisecond bin of the call, on every lane, with lanes parking and with lanes hot. The one real effect the probe finds is **SMT sharing, constant through the call and machine-dependent** — 1.5 to 7 % over a baseline core speed that differs by **18 %** between two runners printing identical `topology:` and `smt:` lines.

**What they do not settle, and cannot here.** The loss appeared on two of the three machines (runs `34667725821` and `34671025894`, both 1.11 at fir W=4) and not on the third (run `34672061881`, **0.852**, with the compiled form fastest in its block), so the machines that lose were never probed and the mechanism there stays open. On real hardware it does not occur at any cadence. The hot window, priced on one machine in one set of passes, is worth 1.6 % of wall at fir W=4 and 6.5 % at mandelbrot W=4 for 7–11 % more process CPU, against 15–27 % for the same lever on the machine of run `34668796717` — machine-dependent too.

**Tradeoff and consequence.** **No runtime change is selected from any of this.** The hosted runner class is not a fixed instrument for a ten percent question, and the record says so and says what a future reading needs: the probe's baseline recorded beside the table, so a result can be attributed to a machine before it is attributed to the runtime. `WF_PAR_TRACE` stays in the tree as a gated instrument that nothing shipping compiles; whether it earns its place long-term is a separate decision, and it is removable in one commit. The probe is the one deliberate cost where the flag is on — 3 to 4 % of a `wf-b` cell — and it lengthens every latency measured from the call head, which the README says and which is why wake and steal latency are read off the un-probed run.

Also here: three sections appended to `RESULTS.md`, one per run, each with its manifest, its `table.txt` verbatim and the trace-derived tables; a dated memory fact superseding the causal half of the two entries above it; and the status paragraph updated where it named the per-chunk trace as the next work. No table count changes — all three carry a `wf-b` row and a non-zero gap, so none is a candidate record. No spec or conformance evidence is touched.

## Agent review

- **Scope:** self-review against `docs/review-checklist.md`; base `origin/bench/gap-cadence` (`ddc15701`) .. head `1b3120b7`. Checked: the gated runtime change (no effect when off, no allocation, no data race in the ring writes, platform guards), README accuracy against what the code does, the three `RESULTS.md` sections and the status paragraph against the artifacts they quote, the memory fact against its file's convention and the entries it supersedes, and citation boundaries. Not reviewed by a second party; the trace-derived tables were computed by scripts kept outside the repository, so their arithmetic is unverified by a second implementation.
- **Checks:** `make repository-invariants` clean; `make static` clean (spec append-only, spec prose integrity); `make -C research/experiments/compute-bench programs-check` pass; `make -C compiler completion-test` pass (compute smoke, sched deque probe, completion core harness at four helper settings, pure-compute link boundary, native adapter probe); `cargo fmt --check` clean (no Rust in this diff). Object identity against the merge base with the flag off: `core.o`, `prim_host.o` and — via `x86_64-w64-mingw32-gcc` — `prim_windows.o` all `cmp`-identical. With the flag on, all three compile clean under `-Wall -Wextra -Werror -Wpedantic`. The full `make check` was **not** run: three corpus cases that require a non-root user (`an_unreadable_*`) cannot pass in this environment.
- **Findings:** five found and fixed in the head commit — (1) `wf__par_trace_chunk` incremented `ring->chunks` with a plain read-modify-write while the call-opening sweep stores zero to the same field from another lane, a data race by the letter of the model; both sides are relaxed-atomic now, which is all the value needs since nothing orders on it. (2) The instrument's header comment still described only the wake path and reasoned about ring wrapping from the old event count. (3) `README.md` gave the probe's cost as 2 % where the hosted runs measured 3 to 4. (4) `README.md` did not say that the offering lane's own call-head probe lengthens every latency measured from the head, and credited `seq` 0 to post-park probes alone when the call-head probe carries it too. (5) `README.md` opened by asserting the loss is at the head of the call, which these runs refute. Remaining **unverified**: the `mcts_mem` integrity lint was not run — its skill is not available in this environment — so the appended fact was checked against the file's convention by inspection only; and the three `RESULTS.md` sections quote figures derived from the raw trace by scratch scripts that ship nowhere, so a reader cannot re-derive them from the repository alone (the artifacts they came from expire).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU

---
_Generated by [Claude Code](https://claude.ai/code/session_01FkkJ7PzdrURF5JQvLAWNTU)_

---

