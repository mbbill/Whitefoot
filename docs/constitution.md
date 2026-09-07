# The Whitefoot Constitution

Whitefoot is a programming language designed as a harness for AI agents.

The primary authors are AI agents; humans set objectives, approve changes,
and judge whether the resulting software serves its intended purpose. The
language supplies constraints and guidance within which agents construct
programs, together with machine judgments on the obligations those programs
must satisfy.

Changing the author changes the design tradeoffs. Explicit proofs, detailed
interfaces, restrictive representations, and verbose source may be worthwhile
when they improve correctness or performance. Human ease of writing and
familiarity are not independent objectives. Agents must still have enough
information and effective feedback to complete the intended work; difficulty
alone and an impossible or incoherent task are different problems.

This premise opens a design space. High performance and machine-checked safety
are Whitefoot's chosen objectives within it. They are not the only values an
AI-oriented language could choose, and they do not uniquely determine a proof
system, syntax, representation, or collaboration model. Concrete designs need
technical reasons and evidence; uncertainty and viable alternatives remain
part of an honest decision.

## Objectives

**P0 — Performance.** Aim for efficient systems programs by making important
costs and correctness facts available when architecture and implementation
are chosen. Restrictions and checked facts should enable good algorithms,
representations, and generated code without writer-accessible escape hatches.
Measure performance on defined workloads and distinguish algorithm,
representation, lowering, and proof contributions. A fast program does not
by itself establish that its proof mechanism caused the gain.

**R0 — The comparison with existing systems.** Assess the language and major
competing design directions against effective existing approaches, including
Rust, in performance, resistance to unchecked shortcuts (W3), and default
implementation quality (W1). State the comparison boundary, expected benefit,
and uncertainty. Reusing an existing mechanism can serve the whole design;
every individual construct need not outperform its counterpart. Novelty alone
has no value, and a local win does not establish an ecosystem-wide advantage.

**P1 — Agent writability.** Agents must be able to construct, verify, and
revise useful programs under the language's constraints. Local reasoning,
composable interfaces, and actionable feedback serve this objective. Verbosity
or difficulty for a human author is not evidence against it; failure by a
particular model is evidence about that model, task, and available assistance.
It can expose a language problem without proving one by itself.

- **W1 — Default performance and architectural guidance.** Restrictions,
  interfaces, reusable components, diagnostics, and taught patterns should
  steer ordinary writers toward efficient, verifiable implementation classes.
  Prevent shortcuts that hide shared mutation or bypass required reasoning.
  Each important restriction needs a usable alternative whose costs and
  limitations are understood. Measure representative coverage, runtime cost,
  repair effort, and architectural rework. A slow accepted program is a
  finding to attribute; it is not automatically a language defect. W1 does
  not promise global optimality or a catalog covering every possible program.
- **W2 — Available information and local reasoning.** An agent should be able
  to find the current rules and relevant interfaces without reconstructing
  project history or unrelated implementations. Additional explicit source
  can reduce uncertainty. Minimize conflicting instructions, hidden premises,
  and unnecessary nonlocal knowledge; do not minimize tokens at their expense.
  A context-window size or today's model cost does not set a permanent language
  limit. Measure repair and coordination costs under stated conditions.
- **W3 — No unchecked shortcuts.** Source has no writer-emittable `unsafe`,
  trusted theorem, or runtime proof trap. A required fact must come from the
  specification's machine judgments; human approval and writer confidence are
  not proof. Calls rely on verified contracts or specification-fixed system
  facts. Expected failures use typed outcomes or intended control flow.
  The checker cannot recover an omitted requirement or prove an unstated
  property. Weakening a required contract to rescue an implementation changes
  the requirement; it is not a proof repair. Evidence about intended behavior
  must remain independent of the implementation being judged.
- **W4 — Composable responsibility.** Aim for interfaces that let an agent
  implement a component and callers rely on its checked guarantees without
  reconstructing each other's internals. Relevant obligations can include
  behavior, ownership, effects, and resource relations; each guarantee has
  the scope its contract and proof system actually express. Successful local
  proofs alone do not establish adequate requirements, good system
  architecture, or end-to-end performance. Separating architecture and
  interface design from parallel component implementation is one collaboration
  model to investigate, not the only organization this objective permits.

**Compatibility.** Backward compatibility has lower priority than improving
the language. AI-assisted migration can make breaking changes affordable.
Migration evidence must distinguish mechanical rewriting from changes to
behavior or contracts; easy source edits do not establish semantic preservation.

## Safety and representation commitments

These are chosen guarantees and design commitments. Their motivation does not
constitute a proof that a compiler or runtime implements them. The
[language specification](../spec/kernel-spec.md) defines their exact source
judgments and trusted boundary.

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
  observable effects.
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

**Balance.** Preserve required safety and proof obligations. Within that
boundary, prefer runtime performance over ease of source authorship when a
real tradeoff remains. Extra writing, proof, or checking effort can be
acceptable, but its cost and the ability to complete intended programs must
be assessed. Present-day model limitations inform experiments; they do not
settle what a future agent can write.

- **R1 — Earn the construct.** A construct must serve P0 or P1. Familiarity or
  human-writing comfort alone is not a reason to add it.
- **R2 — Cuts must preserve useful authoring.** Simplicity alone does not
  justify removing a capability. Test the replacement on the intended task,
  including its proof effort and runtime cost.
- **R3 — Select canonical forms by evidence.** Keep one chosen form per
  construct and justify it against P0 and P1. A form chosen only for minimality
  remains provisional. Regularity does not prove efficiency, and choosing one
  form does not prove that every alternative is unsuitable.
- **R4 — Move failures earlier.** Prefer making defects unrepresentable, then
  rule-citing compile-time rejection with actionable diagnostics. A runtime
  proof trap or hidden fallback cannot substitute for required static proof.
  Recoverable failures remain ordinary program behavior.
- **R5 — Make human exceptions explicit.** Human authorship ergonomics is not
  an independent goal. Human judgment of requirements and behavior, and
  auditability of the trusted base, remain necessary. Clear contracts,
  representations, and diagnostics also serve AI correctness and local
  reasoning under P1.
- **R6 — Keep the stack open.** Self-hosting, alternative backends, hardware,
  and surrounding software architecture are open to investigation. Self-hosting
  tests language capability; proving checker or lowering correctness is a
  separate task. A future use does not establish one necessary operating-system
  shape, software layering, or reuse model. Avoid unnecessary dependence on a
  particular ISA or surrounding stack.
- **R7 — Separate grounds from conclusions.** Distinguish an objective, a
  consequence under stated assumptions, a selected mechanism, and an observed
  result. A rationale can justify trying a design without proving it uniquely
  necessary. State relevant alternatives, uncertainty, and what evidence would
  reopen the choice. Experiments determine what works under their conditions;
  changed conditions can change the decision. A rejected design's actual
  failure still needs an answer: improved authoring ability does not repair a
  soundness counterexample. Preserve the intended problem when comparing
  solutions, or make the changed requirement explicit.
