# 0073 — Claim-only runtime trap surface and static contracts

Owner: lead. Workspace: `codex/0073-claim-only-contracts` in the isolated
`/Users/bytedance/do_not_scan/whitefoot-0073-claim-only` worktree. Base:
main `e5b30704831c03a6555aa5a08d049558e468477e` after batch 0072 closure.
Registered: 2026-08-19 under the ACTIVE Current Plan (W1–W6).

## Authority

The owner-approved Direction Outline revision 43 and ACTIVE
`docs/current-plan.md` selected this undertaking on 2026-08-19. The batch may
prepare, implement, migrate, test, document, and audit the complete candidate
without an intermediate approval pause. Specification and protected-compliance
bytes remain unactivated candidates until the owner approves their exact
identities in the final packet.

## Scope

- W1: named results, one uncallable command entry, and one erased `contract`
  block with symbolic `define`, plural `requires`/`ensures`, and selected-result
  routes.
- W2: independent pre-transfer goals, atomic plural postcondition publication,
  explicit uninhabited disposition, and ABI-preserving unreachable lowering for
  checker-detected contradictory requirements.
- W3: all nine integer trap forms become proof-required exact operations with
  total writable domain predicates; no integer runtime trap carrier remains.
- W4: allocation fit and half-open system endpoint obligations replace the two
  allocation-size and six active range traps, and apply equally to batch 0072's
  held `open_file` candidate.
- W5: one coherent v0.33 candidate composed with batch 0072's strict-retirement
  and Linux-qualification deltas; ordinary and protected corpus migration;
  current documentation and MCTS-Mem re-decisions.
- W6: grammar and candidate verifiers, accepted-set and artifact differentials,
  complete compiler/repository/conformance gates, target and cost-shape locks,
  independent trap inventory, adversarial audit, and exact owner packet.

## Approval classes

- `spec/kernel-spec.md` remains a marked v0.33 candidate over active v0.32.
- Every addition, deletion, rename, source rewrite, manifest/verdict change, or
  collection/wiring change under protected conformance is recorded exactly in
  the final before/after audit and does not activate on this branch.
- Ordinary compiler tests, program fixtures, implementation, non-protected
  documentation, bounded probes, and MCTS-Mem maintenance proceed under the
  ACTIVE plan.

## Invariants

One safe-Rust compiler path; no writer unsafe, hidden check, unchecked assume,
operation-name special case, second proof engine, or facts-on acceptance
dependency. Claims always execute. Externally controlled protected subjects
still require the installed provenance-approved value branch. Expected host
and content failure stays typed; resource exhaustion and TCB failure are not
relabelled as language traps.

## Out of scope

Activation, merge to main, FFI or export adapters, dynamic calls, general SMT,
a frozen target ABI, catchable invariant failure, generalized certificates,
and batch 0072's unrelated wfgrep capacity and performance follow-up.

## Outcome

The scoped candidate is complete and remains unactivated. Its specification is
`CANDIDATE v0.33`, 3,204 lines and 395,671 bytes, with SHA-256
`024a7752a88daf8799f637d95401fb73e25e257b118b3b78d4733b397c3db3c2`.
The compiler has one named-result/static-contract path, lowers contradictory
requirements to one ABI-preserving `unreachable` body, and carries no
writer-reachable language trap site except an executed claim. Exact integer,
allocation-fit, and system-range sites are static obligations; checked,
wrapping, saturating, typed host failure, resource, and TCB boundaries retain
their distinct semantics.

The protected corpus moved from 490 to 499 sources: 468 existing paths were
modified, 21 were replaced by renamed subjects, and the nine approved v0.33
cases were added. The one excluded source,
`type6-neg-dup-variant.wf`, remains byte-identical. No runner, collection, gate,
or invocation wiring changed.

## Landed slices

- Plan and boundary: `dde2f6af`, `3094a712`.
- Specification, grammar, and qualification: `55a75434`, `67e94272`,
  `930ca85d`, `1f5bbaa5`, `0fb44628`, `736b06ca`.
- Static contracts, named results, command-only entry, and claim-only IR:
  `b8ccecbb`, `f2a6d60e`, `9f0cccfd`, `d5ca20b8`, `4e62c911`.
- Integer, allocation, system-range, target-boundary, and `open_file` work:
  `22efda80`, `59cc27c7`, `a09a3601`, `55140d50`, `b46bba04`,
  `f942aac9`, `3c5847a3`, `6a799224`, `5c475516`, `a567e5fa`,
  `a5b0f5b1`, `378a9f39`, `54c32b36`.
- Ordinary migration and current prose/design memory: `4d853af5` through
  `0c73b6d9`, plus `627f824d` through `eee66c21`.
- Exact evidence and protected candidate: `99401e3c`, `2757f1a2`,
  `d476a320`, `c9a20baa`, `ed133d08`, `6b8d9760`, `4a160bcf`,
  `f1d8a2cd`.

The complete no-merge commit sequence is the repository range
`dde2f6af..f1d8a2cd`; the grouped list above identifies the load-bearing
slices rather than duplicating every fixture and documentation commit.

## Verification

- Native grammar identity: 74 productions, 93 decisions, 105 terminal
  predicates; generated tables match the candidate specification.
- Candidate identity: the specification digest independently matches the
  derivation ledger, `compiler/src/spec.rs`, and
  `compiler/src/spec_identity.rs`; the active chain still ends at v0.32.
- Ordinary evidence: all-target compilation, formatting, Clippy, focused
  contract/integer/allocation/system/target suites, 48/48 real-program tests,
  and 10/10 cost-shape tests pass.
- Protected integrity: 499 manifest IDs and 499 source files form an exact
  bijection; coverage is 135/135; the canonical adapter reports
  `Pass=498 Skip=1 Fail=0` in 204.66 seconds.
- Design memory: `npx mcts-mem lint mcts_mem` reports 98 clean nodes and zero
  fact-file violations.
- Final repository gate: `make check` passes on the reconciled candidate tree.

## Adversarial audit dispositions

- Integer-domain normalization was moved into the shared goal authority so
  exact sites, requirements, calls, claims, and contradiction closure cannot
  disagree.
- Allocation and system audits repaired generic layout ceilings, zero-length
  arrays, endpoint provenance, host-count and record sanitization, target
  component limits, exact system IR identities, no-follow regular-file
  validation, and the one-attempt provisional-close policy.
- Contract audits repaired strict-root occurrence identity, plural SCC
  publication, empty/define-only coverage, and the uninhabited lowering fence.
- Final qualification, protected-tree, and whole-branch audits are clean. The
  protected audit independently reproduced the approved 489-path-set digest,
  the frozen-source digest, all before/after counts, and unchanged runner
  blobs. No unresolved correctness, evidence, or governance finding remains.

## Integration boundary

Main's later `c8d5db2f` commit prematurely assigned the generic-container work
the same batch number and ACTIVE-plan slot. Owner direction selected
claim-only first, retaining 0073, and generic-container afterward as 0074.
Merge `4ddde782` records latest main as an ancestor while preserving the sole
claim-only ACTIVE plan and sole ongoing 0073 record. It deliberately does not
open an ongoing 0074 before a generic-container plan is ACTIVE. After v0.33 is
approved, activated, merged, and this record moves unchanged in number to
`docs/done/`, the generic-container direction may become the sole ACTIVE plan
and open a fresh 0074 record against the resulting main.

Activation, archive creation, the `ACTIVE-SPEC` chain record, and merge remain
the one final owner-approved act; none is present in this candidate.
