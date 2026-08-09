# 0045 — correct ENT-5 and switch to the stable active specification

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** the ACTIVE obligation-discharge plan derived from Direction
  Outline revision 21; the exact-approved v0.24 ENT-5 delta at
  `governance/spec-evolution/ent5-loop-fix-v024-candidate.md`; the adopted
  stable-filename proposal; and the owner's 2026-08-09 approval of the complete
  digest, named protected changes, and S10 evidence disposition
- **Owner / workspace:** Codex lead /
  `/Users/bytedance/do_not_scan/wf-0045-final-activation`, branch
  `codex/0045-ent5-activation`
- **Base revision:** task claim `82a37af`; atomic integration rebuild
  `19b02ea`
- **Dependencies:** terminal tasks 0040, 0042, 0043, and 0044; O11 and the
  provenance gate remain outside this task

## Goal and direction

Fix loop-head fact invalidation so only kill events on an execution path that
can reach the same loop body's next entry invalidate pre-loop facts. Preserve
all real continuing kills, including nested-loop paths that continue inside an
outer loop. Assemble the complete v0.24 bytes and migrate the one active file
to `spec/kernel-spec.md`, leaving v0.23 immutable at its versioned archive.

The owner has approved the complete v0.24 bytes and named protected changes,
and the atomic tree records that approval with the chained `ACTIVE-SPEC:` line.
The task remains live until the complete gates, independent adapter, frozen
acceptance, and S10 evidence are rerun against that installed authority.

## Method and scope

1. Add focused D1h/D1i and nested-control regressions, then replace the current
   recursive all-events loop summary with structured continuing-reachability
   analysis. Return, propagated-Err, current/enclosing break, and exit-only
   suffixes do not reach the same head; a real fallthrough/backedge or nested
   continuation does.
2. Assemble and independently hash the full v0.24 candidate from immutable
   v0.23 plus the reviewed ENT-5 delta. Verify unchanged grammar counts and a
   deliberate-break negative control.
3. Prepare the stable-path compiler identity, generated grammar header,
   qualification guards, conformance runner/tests, derivation pin, workflow
   law, and live-document changes. Inventory every protected corpus change;
   do not alter a case or expectation without the approval required by the
   workflow.
4. Present the exact digest and impact packet. After exact approval only,
   atomically record the approval and activation, install the approved stable
   bytes and pins, run the stable-layout mutation controls and full gates, then
   rerun frozen utf8parse/SHA-256/deflate acceptance and the shipped SYS-S10
   boundary path.

Expected touch set: `compiler/src/semantic/entailment/flow.rs` and focused
semantic tests; `spec/kernel-spec.md`; compiler specification identity and
generated syntax data; conformance runner identity tests and only approved
protected rewrites; the derivation ledger; `AGENTS.md`, `CLAUDE.md`,
`docs/WORKFLOW.md`, `docs/roadmap.md`, `docs/current-plan.md`, the specification
approval ledger, and this record. The old approved-candidate byte comparison
may be removed only when the stable identity checks replace it.

## Progress

- Completed: v0.23 activation, stable-aware archive integrity, canonical-corpus
  gate repair, ENT-5 re-cut, and plan reset are terminal and the full repository
  gate is green at the recorded base. The complete stable-file candidate is
  assembled from the immutable v0.23 digest `e09b32ed…aace0f5`; its current
  complete SHA-256 is `53495b9c…8cb2dc86`. The candidate now fixes the exact
  history-header assembly rule instead of leaving the v0.23 `Prior:` bytes to
  inference. Protected impact is one existing ENT-5 witness rewrite plus one
  additive positive case; no verdict changes.
- Completed on the review branch: structured continuing-edge analysis and 8
  focused ENT-5 controls pass; with the added S10 UTF-8 control, the complete
  entailment suite is 62/62. Independent semantic and exact-byte reviews found
  no high-confidence defect. The native
  grammar verifier remains 69/84/93 and rejects a deliberate grammar change.
  The corpus differential has 0 moved, 0 removed, and 1 additive case; the
  independent adapter reports 390 pass, 1 pre-existing OWN-3 attribution
  failure, and 13 skipped. The frozen rerun preserves utf8parse at 22/33 and
  SHA-256 at 0/9 proven, and moves deflate from 5/29 to 11/29 proven with no
  previously proven regression.
- Completed: the owner approved the complete v0.24 digest, the two named
  protected-corpus changes, and the honest S10 limitation. The reviewed tree
  was rebuilt from the integration base as one atomic activation transition;
  no pre-approval review commit is being fast-forwarded separately.
- Resolved acceptance mismatch: all four S10 producers have focused
  obligation-consuming controls, and the real boundary path establishes
  `taken <= room`, but that program uses `taken` only in `+wrap` and has no
  entailment obligation that consumes the relation. Treating that as an
  end-to-end boundary consumer would be false. The owner approved the honest
  boundary-producer plus focused-consumer evidence rather than an artificial
  sentinel or copy loop.
- Current: run the complete post-activation gates, adapter, frozen acceptance,
  and S10 confirmation, then record the results and close this task. Until that
  terminal closure, no later planned task is claimable.

## Validation and stop condition

- Focused semantic controls for return, propagated error, current/enclosing
  break, exit-only kill, true continuing kill, else-free continuation, and
  nested-loop reachability.
- Native grammar verification on both compiler and standalone paths, including
  the deliberate-break negative control; expected counts remain 69/84/93.
- Stable-layout archive gate plus its missing/malformed/wrong-identity
  mutations; `make -C compiler check`; `make check`; independent conformance
  adapter; frozen acceptance and S10 revalidation after activation.

Stop on a semantic case not decided by the candidate, any need to change a
protected expectation without owner approval, any complete-byte mismatch, or
any attempt to make a gate green by manufacturing approval or activation.

## Exact review and approval packet

Implementation candidate: commit `00e6ce4`. The complete proposed active file
is `spec/kernel-spec.md`, SHA-256
`53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
Relative to immutable v0.23 it has exactly three diff hunks: title v0.24, the
new Status paragraph plus byte-preserved v0.23 Prior paragraph, and the one
ENT-5 final-paragraph replacement. The historical candidate-document digest at
review commit `7e47130` is
`8c520d868b54ff40332ac2c2475a8e4e32fe217b4ab513279420a0a67818c656`;
the activation disposition later changes that document, so this is not its
current identity.

The approved protected corpus agreement is limited to the exact changes in
review commit `00e6ce4`:

- `ent5-neg-loop-rule-drops-preloop-fact`: preserve `reject OP-4`, but replace
  the obsolete unconditional current-loop break with an else-free branch whose
  false edge carries the real continuing kill; update only its explanatory
  source/manifest text.
- add `ent5-pos-return-does-not-kill-loop-head-fact` with `accept`, pinning the
  non-continuing return case.

There is no existing verdict, cited rejection rule, or runnable-status change;
the full differential is 0 moved, 0 removed, 1 added, 404 total. Grammar is
unchanged at 69/84/93. The compiler library is 584/584, entailment 62/62,
clippy is warning-free, runner tests are 23/23, canonical corpus tests are 3/3,
and the native adapter is 390/1/13 with only the pre-existing OWN-3 issue.
Independent semantic and exact-byte reviews reported no findings.

Before approval, the review tree deliberately remained red at the authority
boundary: `make check` and `whitefoot-spec` both reported the missing v0.24
identity while internal hashes, rules, and derived grammar checks passed. The
owner then approved the packet and the atomic activation tree appends this
exact chain record:

```text
ACTIVE-SPEC: v0.24 53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86 e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5
```

The same tree changes the identity test to the 16-link success assertion and
updates the derivation ledger and live roadmap/plan/proposal status. Complete
post-activation gates still precede task closure.

The owner approved the honest boundary-producer plus focused-consumer evidence
as the S10 item-4 revalidation boundary. The implementation adds no
evidence-shaped program behavior. A feasibility audit found no hidden or
natural consumer in the preregistered algorithm: current entailment judges only
subscript obligations, `taken` feeds only total `+wrap`, and L0 cannot derive
`room = 4097 - filled` through the loop and mutable accumulator. A sentinel
index or extra byte-copy loop would exist only to manufacture evidence. A
future counted-range loop over an actual `[0, taken)` processing path could be
a natural consumer, but that is a later capability, not this revalidation.

## Disposable activation rehearsal

The complete post-approval tree was rehearsed in a detached worktree under
`/Users/bytedance/do_not_scan`; none of its synthetic authority bytes was
committed or copied to this branch. A conspicuously non-approval rehearsal
entry supplied the exact v0.24 chain link, the compiler identity test was
changed to `Ok(16)`, and the activation-only ledger, roadmap, rolling plan,
proposal/candidate status, task closure, and 0041-readiness edits were applied.
That was a rehearsal of the eventual terminal tree, not the required real
commit boundary.

The full `make check` then passed. Its identity evidence was:

- archive integrity: 25 recorded specifications hash as recorded;
- compiler library: 584/584, plus every binary/integration/program suite;
- `whitefoot-spec`: v0.24,
  `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`,
  128 rules, 16 unbroken activations;
- complete compiler and repository gates green.

This proved that no hidden machine-gate dependency remained, but the rehearsal
itself granted no authority: its synthetic bytes were discarded before the
real approval and chain were recorded.

The review commits `00e6ce4` and `7e47130` are not fast-forwarded and then
followed by a third activation commit. WORKFLOW requires the approved stable
bytes and `ACTIVE-SPEC:` record to enter the integration history atomically.
The reviewed tree is therefore rebuilt from integration base `19b02ea` as one
linear activation commit. That commit carries the exact bytes, implementation and pins, real approval and
chain, success identity test, derivation binding, revision-21 active authority,
and adopted stable-model status. It leaves task 0045 `IN PROGRESS`, the current
plan on this milestone, and task 0041 unclaimed. The complete v0.24 digest must
be recomputed after any rebase, and any byte change returns to exact review.

Only after that atomic commit lands are the complete gates, adapter, frozen
acceptance, and S10 evidence rerun against the installed authority. A separate
cohesive closure commit records those post-activation results, moves 0045 to
`docs/done/`, replaces the rolling plan with stage 5a, and marks 0041's
dependency satisfied. This prevents a pre-activation measurement from being
presented as installation evidence and prevents 0041 from being claimed early.

## Closure

Pending.
