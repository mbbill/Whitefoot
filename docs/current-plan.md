# Current Plan — claim-only runtime trap surface and static contracts

Status: COMPLETE (v0.33 installed 2026-08-20). This is the technical record of
that completed undertaking. It neither authorizes nor blocks work on a branch;
the live branch and `main` rules are in `docs/WORKFLOW.md`.

Derived from Direction Outline revision 43 and main at
`e5b30704831c03a6555aa5a08d049558e468477e`. Supersedes the completed
searching-wfgrep plan in place. Active language authority: v0.33 at
`spec/kernel-spec.md`, SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.

## Objective

Install the doctrine that v0.32 applied only in selected families: `claim` is
the sole writer-authored runtime safety backstop, while every other hazardous
condition is a deterministic proof obligation, a total operation, or a typed
expected outcome. Replace the statement-shaped requires/ensures surface with
one erased static contract model, remove the program-entry exception, and
carry every remaining non-claim language trap family through the ordinary
compiler path to the tested and active v0.33 specification.

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
- **W6 — verification evidence.** Verify candidate grammar with the compiler's
  own frontend, run accepted-set and artifact differentials, derive effects and
  trap inventories independently, run focused hostile controls, full compiler
  and repository tests, conformance, and target/code-shape checks. Preserve the
  exact specification identity and the evidence needed to explain the result.

## Boundaries and invariants

This is one normal safe-Rust compiler path, not a compatibility layer. There is
no writer `unsafe`, hidden runtime fallback, unchecked assume, operation-name
special case, or second proof engine. Claims always execute even when the
checker can prove them. A claim on an unconditionally external constrained
subject cannot replace the real value branch required by the installed
provenance policy. Facts-off acceptance remains correct.

The v0.33 work used branch-candidate specification bytes until its recorded
merge. That is historical execution detail, not a current approval or workflow
requirement.

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
- Installed grammar, compiler, real programs, conformance, and repository tests
  establish the completed v0.33 result.

## Exclusions

The completed v0.33 undertaking did not include FFI, an export adapter,
function values, dynamic dispatch, a general SMT or linear-arithmetic solver,
a frozen target ABI, catchable internal invariant failure, a generalized proof
certificate, unrelated wfgrep capacity/performance repair, or generic-container
work. This historical scope statement does not restrict branch work.
