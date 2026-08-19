# Current Plan — claim-only runtime trap surface and static contracts

Status: ACTIVE (owner approval in conversation, 2026-08-19: roadmap revision
43 and “Claim-only runtime trap surface and static contracts” are approved as
the ACTIVE Current Plan.) Candidate specification and protected-compliance
bytes are prepared and fully verified on the branch; they activate only after
the owner approves their exact identities.

Derived from Direction Outline revision 43 and main at
`e5b30704831c03a6555aa5a08d049558e468477e`. Supersedes the completed
searching-wfgrep plan in place. Active language authority: v0.32 at
`spec/kernel-spec.md`, SHA-256
`5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`.

## Objective

Finish the doctrine that v0.32 applies only in selected families: `claim` is
the sole writer-authored runtime safety backstop, while every other hazardous
condition is a deterministic proof obligation, a total operation, or a typed
expected outcome. Replace the statement-shaped requires/ensures surface with
one erased static contract model, remove the program-entry exception, and
carry every remaining non-claim language trap family through the ordinary
compiler path to a fully tested, unactivated v0.33 candidate.

## Workstreams

- **W1 — one callable world and one contract surface.** Keep exactly one
  uncallable `command fn main`, with optional explicitly labelled capabilities,
  a mandatory named `ExitStatus` result, and no contract. Every function and
  contract-member result is named. One `contract` block admits erased
  declaration-before-use `define` abbreviations followed by independent
  `requires` and `ensures` clauses; selected `Result` postconditions retain an
  explicit route and payload binder. No contract construct executes or lowers.
- **W2 — plural proof semantics and unreachable contradiction.** Prove every
  requirement against the same caller pre-transfer state, then supply all of
  them as independent S4 sources. Prove every selected relation at every
  matching return in complete, unasserted, and S4-blinded views, and publish an
  SCC's summaries atomically. A checker-detected contradictory requirement set
  is legal uninhabited code: retain its source audit and ABI, but lower only an
  `unreachable` stub so ex-falso never emits an unchecked body.
- **W3 — integer-domain obligations.** Remove the five bare-infix and four
  named `.trap` runtime modes as trap carriers. Bare/dotless exact arithmetic
  requires its exact domain predicate to be proved; total `defined` queries
  make the condition writable by a branch, contract, or claim. Checked,
  wrapping, and saturating result semantics stay value-producing and unchanged.
- **W4 — allocation and system-range obligations.** Replace both allocation
  byte-size traps with a target-independent type-stride-ceiling obligation and
  target qualification check. Replace the six active range traps, plus batch
  0072's candidate `open_file` row, with half-open start/end obligations and
  endpoint success results. Expected host, path, UTF-8, and content failures
  remain typed values; OOM, target addressability, and a broken compiler/runtime
  remain explicit resource or TCB boundaries rather than language traps.
- **W5 — migration and durable authority.** Migrate real programs, inline
  compiler fixtures, ordinary tests, and the exact protected conformance
  candidate. Compose batch 0072's held `open_file`, strict-retirement, and
  Linux-qualification deltas into v0.33. Supersede stale prose in place and
  record every genuine re-decision, including entry enforcement, writer trap
  surface, exact arithmetic spelling, and system declaration visibility, in
  MCTS-Mem.
- **W6 — verification and owner packet.** Verify candidate grammar with the
  compiler's own frontend, run accepted-set and artifact differentials, derive
  effects and trap inventories independently, run focused hostile controls,
  full compiler and repository gates, protected conformance, target/code-shape
  checks, and an adversarial batch audit. Deliver the exact spec SHA-256,
  complete diff and impact inventory, protected before/after audit, verifier
  output, commits, and unresolved findings in one approval packet.

## Boundaries and invariants

This is one normal safe-Rust compiler path, not a compatibility layer. There is
no writer `unsafe`, hidden runtime fallback, unchecked assume, operation-name
special case, or second proof engine. Claims always execute even when the
checker can prove them. A claim on an unconditionally external constrained
subject cannot replace the real value branch required by the installed
provenance policy. Facts-off acceptance remains correct.

Specification and protected-compliance edits are marked candidates and remain
unactivated. The completed branch and exact owner packet are the approval
boundary; candidate preparation, compiler implementation, ordinary tests,
documentation, MCTS maintenance, verification, and audit do not pause for an
intermediate approval.

## Acceptance

- The canonical grammar admits only the named-result, command-only entry, and
  unified static-contract surface selected above.
- Every reachable source call proves every requirement before transfer; every
  accepted reachable hazardous operation is proved, total, or guarded by an
  explicit executed claim or typed outcome.
- Checked semantic and IR models have no entry-requirement, integer,
  allocation-size, or affected system-range trap carrier; emitted code has no
  corresponding `wf_trap` edge or DIAG-3 record.
- A complete independent inventory finds claims as the only remaining
  writer-reachable language trap sites, while checked/wrap/saturating values,
  recoverable host failures, cleanup, and TCB/resource behavior retain their
  specified results.
- Candidate grammar, compiler, real programs, protected conformance, repository
  gates, MCTS lint, and adversarial audit are green, and the final packet is
  sufficient for an exact-byte owner decision without follow-up reconstruction.

## Exclusions

No FFI, export adapter, function values, dynamic dispatch, general SMT or
linear-arithmetic solver, frozen target ABI, catchable internal invariant
failure, generalized proof certificate, activation, merge to main, or
unrelated wfgrep capacity/performance repair is authorized by this plan.
