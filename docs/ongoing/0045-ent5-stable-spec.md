# 0045 — correct ENT-5 and switch to the stable active specification

This is a temporary live coordination record, not execution authority.

- **Status:** `IN PROGRESS`
- **Authority:** the ACTIVE obligation-discharge plan derived from Direction
  Outline revision 20; the v0.23-based ENT-5 candidate at
  `governance/spec-evolution/ent5-loop-fix-v024-candidate.md`; the approved
  stable-filename proposal; and the owner's 2026-08-09 delegation for
  branch-local specification drafting, implementation, and lead review
- **Owner / workspace:** Codex lead / `/Users/bytedance/code/Whitefoot`, branch
  `codex/0045-ent5-stable-spec`
- **Base revision:** `82a37af`
- **Dependencies:** terminal tasks 0040, 0042, 0043, and 0044; O11 and the
  provenance gate remain outside this task

## Goal and direction

Fix loop-head fact invalidation so only kill events on an execution path that
can reach the same loop body's next entry invalidate pre-loop facts. Preserve
all real continuing kills, including nested-loop paths that continue inside an
outer loop. Assemble the complete v0.24 bytes and migrate the one active file
to `spec/kernel-spec.md`, leaving v0.23 immutable at its versioned archive.

The branch may contain a complete reviewable candidate and rehearsed derived
changes. It may not claim activation, add `ACTIVE-SPEC:`, or record owner
approval before the owner approves the complete v0.24 bytes and named protected
changes exactly.

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
   bytes and pins, run the archive mutations and full gates, then rerun frozen
   utf8parse/SHA-256/deflate acceptance and the shipped SYS-S10 boundary path.

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
  gate is green at the recorded base.
- Current: establish focused failing controls and implement continuing-edge
  reachability without changing the entailment fragment.
- Next: assemble and verify the complete stable-file candidate, migrate all
  derived pins on the branch, and produce the exact-byte review packet.

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

## Closure

Pending.
