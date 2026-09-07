# The Whitefoot Constitution

This document owns the project's objectives and language-design principles.
The active [kernel specification](../spec/kernel-spec.md) defines accepted
source; [AGENTS.md](../AGENTS.md) defines engineering priorities and the complete
branch-and-main boundary. Principles motivate language changes but do not
silently change existing language rules or add approval steps.

The owner clarified W1, compatibility, and the intended AI collaboration model
on 2026-09-06. The current text incorporates those decisions and the active
specification's machine-checked, erased proof model. Reasons and superseded
choices are recorded in [decision memory](../mcts_mem/whitefoot.md).

## Objectives

**P0 — Performance.** Machine-code performance is the reason to pursue a
systems language. Checked facts and deliberate architecture should enable
efficient implementations without relying on writer-accessible escape hatches.
Measure gains on defined workloads and distinguish algorithm, representation,
lowering, and proof contributions. A performance result for a program does not
by itself establish a benefit from its proof mechanism.

**R0 — The Rust comparison.** A major design decision names its expected or
measured delta over Rust in performance, resistance to unchecked shortcuts
(W3), or default implementation quality (W1). Equivalence on all three leaves
no demonstrated reason for that decision. State the comparison boundary and
the remaining uncertainty; a local win is not an ecosystem-wide claim.

**P1 — AI writability.** The intended writer is AI, with humans approving
requirements and changes. A long-term use is stronger AI designing architecture
and interfaces while many lower-cost agents implement components. That model
motivates local reasoning and composable contracts; its organization and
large-system effectiveness remain to be investigated.

- **W1 — Default performance and architectural guidance.** Restrictions,
  interfaces, reusable components, diagnostics, and taught patterns should
  steer ordinary writers toward efficient, verifiable implementation classes.
  Prevent shortcuts that hide shared mutation or bypass required reasoning.
  Each important restriction needs a usable alternative whose costs and
  limitations are understood. Measure coverage on representative tasks,
  performance under stated conditions, authoring effort, and architectural
  rework. A slow accepted program is a finding to attribute; it is not
  automatically a language defect. W1 does not promise global optimality or
  that a fixed catalog covers every possible program.
- **W2 — Context economy.** An agent should be able to find the current rules
  and relevant interfaces without reading project history. Token counts alone
  are not a gate. Conflicting instructions, repeated facts, long repair loops,
  and unnecessary cross-module knowledge are engineering costs even when the
  context window is large. Model trials measure the tested workflow and help
  expose these costs; they do not prove language safety or universal writability.
- **W3 — No unchecked shortcuts.** Source has no writer-emittable `unsafe`,
  trusted theorem, or runtime proof trap. A required fact must come from the
  specification's machine judgments; human approval and writer confidence are
  not proof. Calls rely on verified contracts or specification-fixed system
  facts. Expected failures use typed outcomes or intended control flow.
  Canonical source makes changes explicit, but the checker cannot recover an
  omitted requirement or prove an unstated property. Weakening a required
  contract to rescue an implementation is a requirements change, not a proof
  repair; independent behavior evidence must still test the intended task.

**Compatibility.** Backward compatibility has lower priority than improving
the language. AI-assisted migration can make breaking changes affordable.
Migration evidence must distinguish mechanical rewriting from changes to
behavior or contracts; easy source edits do not establish semantic preservation.

## Safety and representation commitments

These commitments follow from P0, P1, W3, and R4. They constrain design; their
motivation is not a formal proof that the current compiler implements them.

- **T1 — Memory and thread safety (D1).** Accepted programs must exclude data
  races, use-after-free, dangling references, double-free, and uninitialized
  reads. Ownership supports both this safety boundary and optimization facts.
  Latent memory faults are unsuitable feedback for unattended authors.
- **T2 — No undefined behavior.** The guarantee is conditional on the declared
  trusted computing base. The specification's SCOPE-3 owns that boundary,
  including external resource availability. An implementation defect does not
  amend the guarantee.
- **T3 — Defective executions do not tax correct programs.** Do not withhold
  a proved optimization or overlap merely to reproduce the observables of a
  compiler or trusted-base defect. Proof-required partial operations are
  discharged before lowering and proofs are erased, so there is no source
  proof-failure path to schedule or stabilize. This does not relax required
  safety checks or the semantics of typed errors, intended branches, and
  observable effects. The old derivation from human-approved claims and
  retained traps is superseded by the active proof model.
- **T4 — Resource dependencies are API relations.** Finite resources consumed
  by system operations must be represented by ownership and source-visible
  capacity relations. A release that enables later acquisition must produce
  a relation the checker can use. Overlap must not invent resource exhaustion
  absent from the corresponding sequential execution and then hide it with
  scheduler waits or retries. External changes to host availability belong to
  the specification's explicit outcome or resource boundary. This principle
  guides each resource API; concrete operation contracts live in the spec.

**D17 — Proof-gated representation authority.** The long-term direction is to
admit representation privileges only when a deterministic machine checker
verifies that the exact implementation establishes and preserves every
required invariant. This includes temporary partial initialization without
uninitialized reads and elimination of proved-redundant checks. Missing proof
grants no privilege. A future unproved project primitive, if the specification
admits one, remains in the declared trusted base. Extensions need hostile
soundness evidence. This commitment selects no current syntax, universal
predicate language, or trusted library exemption.

## Design decisions

**Balance.** Evidence decides tradeoffs between performance, authoring cost,
and context economy; where a tradeoff remains, P0 has priority. Required safety
and proof obligations are preserved. Day-to-day research-compiler priorities
are defined in [AGENTS.md](../AGENTS.md#project-goal).

- **R1 — Earn the construct.** A construct must serve P0 or P1. Familiarity or
  human-writing comfort alone is not a reason to add it.
- **R2 — Cuts must preserve useful authoring.** Simplicity alone does not
  justify removing a capability. Test the replacement on the intended task,
  including its proof effort and runtime cost.
- **R3 — Select canonical forms by evidence.** Keep one chosen form per
  construct and justify it against P0 and P1. A form chosen only for minimality
  remains provisional. Regularity is not evidence of an efficient algorithm.
- **R4 — Move failures earlier.** Prefer making defects unrepresentable, then
  rule-citing compile-time rejection with actionable diagnostics. A runtime
  proof trap or hidden fallback cannot substitute for required static proof.
  Recoverable failures remain ordinary program behavior.
- **R5 — Make human exceptions explicit.** Human authorship ergonomics is not
  an independent goal; auditability of the trusted base remains an explicit
  requirement. Clear contracts, representations, and diagnostics also serve
  AI correctness and local reasoning under P1.
- **R6 — Keep the stack open.** Compiler self-hosting and changes to the
  backend or hardware are possible long-term work. Self-hosting tests language
  capability; proving checker or lowering correctness is a separate task.
  Near-term artifacts should not unnecessarily bind the language to one ISA.
