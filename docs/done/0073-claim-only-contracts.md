# 0073 — Claim-only runtime trap surface and static contracts

Status: DONE (v0.33 activated and merged under the owner's exact-byte approval,
2026-08-20).

Owner: lead. Workspace: `codex/0073-claim-only-contracts` in the isolated
`<scratch-root>/whitefoot-0073-claim-only` worktree. Base:
main `e5b30704831c03a6555aa5a08d049558e468477e` after batch 0072 closure.
Registered: 2026-08-19 under the ACTIVE Current Plan (W1–W6).

## Authority

The owner-approved Direction Outline revision 43 and then-ACTIVE
`docs/current-plan.md` selected this undertaking on 2026-08-19. The batch may
prepare, implement, migrate, test, document, and audit the complete candidate
without an intermediate approval pause. Specification and protected-compliance
bytes remained unactivated candidates until the owner approved their exact
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

- `spec/kernel-spec.md` remained a marked v0.33 candidate over active v0.32
  until the separately approved final activation recorded below.
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

The scoped candidate completed at `CANDIDATE v0.33`, 3,204 lines and 395,671
bytes, with SHA-256
`024a7752a88daf8799f637d95401fb73e25e257b118b3b78d4733b397c3db3c2`.
The owner approved that exact candidate and WORKFLOW activation changed only
its declared status line. The installed `ACTIVE v0.33` bytes are 3,204 lines
and 395,586 bytes at SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`;
the outgoing v0.32 bytes are immutable at `spec/kernel-spec-v0.32.md`.
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
or invocation wiring changed. The final protected tree is
`882e691cf456758c456509a057ab6328c1f58a88`; the manifest SHA-256 is
`f6b7cda7d523837c5ae1ddf3115ac82afabc1d13d9a4e7ddff4a24591b85c609`.

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
- Main-ancestry reconciliation and exit-audit repairs: `4ddde782`,
  `4bb3236c`, `a911681a`, `1dc7e6a1`, `06a625d3`, `08548904`,
  `df057f5d`, `65bfbab6`.
- Activation and closure: the atomic activation/integration commit carrying
  this record move.

The pre-reconciliation no-merge commit sequence is the repository range
`dde2f6af^..f1d8a2cd`; the post-reconciliation list records the merge and every
later repair explicitly. The grouped list identifies the load-bearing slices
rather than duplicating every fixture and documentation commit.

## Verification

- Native grammar identity: 74 productions, 93 decisions, 105 terminal
  predicates; generated tables match the installed specification.
- Specification identity: the specification digest independently matches the
  derivation ledger, `compiler/src/spec.rs`, and
  `compiler/src/spec_identity.rs`; after activation the installed digest and
  chain tail independently match active v0.33.
- Ordinary evidence: all-target compilation, formatting, Clippy, focused
  contract/integer/allocation/system/target suites, 48/48 real-program tests,
  and 10/10 cost-shape tests pass.
- Protected integrity: 499 manifest IDs and 499 source files form an exact
  bijection; coverage is 135/135; the canonical adapter reports
  `Pass=498 Skip=1 Fail=0` in 204.66 seconds.
- Design memory: `npx mcts-mem lint mcts_mem` reports 98 clean nodes and zero
  fact-file violations.
- Final repository gate: `make check` passes on the reconciled candidate tree.
- Activated repository gate: `make check` passes with the v0.32 archive,
  v0.33 chain tail, regenerated identity, and installed protected evidence.

## Adversarial audit dispositions

- Integer-domain normalization was moved into the shared goal authority so
  exact sites, requirements, calls, claims, and contradiction closure cannot
  disagree.
- SYS-3 declaration inventory is installed in every unit before entry-form
  validation; focused resolution tests pin both universal name visibility and
  the unchanged FN-7 rejection of an invalid entry.
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
Merge `4ddde782` records that main as an ancestor while preserving the sole
claim-only plan and batch 0073 record. The final owner act then archives v0.32,
installs and records active v0.33, moves this record unchanged in number to
`docs/done/`, and fast-forwards main. It deliberately does not open 0074 or
include generic-container work. That direction may begin only under a separate
owner-approved plan and a fresh 0074 record against this resulting main.

## Final disposition

The atomic activation/integration commit carrying this move installs the
owner-approved v0.33 language, its generated identities, and the already
audited protected evidence together. `ACTIVE-SPEC: v0.33` chains
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`
to the archived v0.32 digest
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`.
Batch 0073 is closed with no unresolved correctness, evidence, or governance
finding; generic-container and batch 0074 remain outside this integration.
