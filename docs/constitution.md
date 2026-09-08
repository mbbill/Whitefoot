# The Whitefoot Constitution

Whitefoot is a programming language designed as a harness for AI agents.

AI agents are the intended primary authors. Humans set requirements and judge
whether the resulting software serves its purpose. The language supplies
constraints, guidance, and machine judgments within which agents construct,
verify, and revise programs.

Changing the author changes the design space. Human ease of writing and
familiarity need not govern it. More explicit source, restrictive forms, or
harder proofs can be worthwhile when they serve the objectives below. Agents
must still have enough information and effective feedback to complete the
intended work. Difficulty and inability to express a required program are
different costs.

This purpose does not uniquely determine Whitefoot's objectives or mechanisms.
The objectives are choices about the systems we want to build. The principles
guide decisions under stated conditions; their usefulness and the designs
selected under them remain open to technical argument and experiment.

## Chosen objectives

**P1 — Effective agent authorship.** Agents should be able to construct,
verify, maintain, and evolve useful systems programs. Adequate information,
coherent rules, and effective feedback are necessary. A particular model's
success or failure is evidence under its task and assistance conditions, not
a permanent limit on what the language should permit.

**P0 — Performance.** Aim for efficient systems programs. Important costs and
correctness facts should be available when architecture and implementation
are chosen, so that efficient algorithms, representations, and execution are
practical outcomes of ordinary development.

**T1 — Memory and execution safety (D1).** Accepted programs must exclude
memory corruption, data races, uninitialized reads, silent overflow, and
other unproved partial operations. Required safety is a constraint on a
successful design, not a score that an unrelated performance gain can offset.

**T2 — Defined behavior and an explicit trust boundary.** Accepted programs
must have no undefined behavior within the declared guarantees. State the
assumptions about the compiler, runtime, host, and foreign components on which
those guarantees depend. A defect in their implementation does not amend the
guarantee. The [language specification](../spec/kernel-spec.md) defines the
exact judgments and trusted boundary.

## Tradeoffs

**Balance.** Preserve required safety and the ability to express the intended
program. Within that boundary, prefer runtime performance over ease of source
authorship when a real tradeoff remains. Extra writing, proof, and checking
effort can be acceptable; it must still leave a feasible development process.
Today's model cost or context window does not set a permanent language ceiling.

**R5 — Human responsibilities.** Human authorship ergonomics is not an
independent objective. Understanding requirements, reviewing observable
behavior, and auditing the trusted boundary remain necessary human tasks.
Clarity that serves those tasks or effective agent work has value.

**Compatibility.** Backward compatibility has lower priority than improving
the language. When AI-assisted migration reduces the cost of change, breaking
changes become more attractive. Mechanical rewriting does not establish that
behavior or contracts were preserved; those obligations remain.

## Conditional design principles

**W1 — Guide ordinary implementations toward good performance.** For the
intended workload, use constraints and usable guidance to steer writers
toward efficient implementation classes before expensive architectural
rework is needed. An important restriction needs a viable route for the
required programs. This objective promises neither global optimality nor a
closed catalog covering every program.

**W2 — Make relevant information available.** When work is local, its rules
and interfaces should be understandable without reconstructing unrelated
implementations or project history. Reduce hidden premises and conflicting
instructions. Extra explicit source can improve local reasoning; shorter
text alone is not the objective.

**W3 — Make guarantees binding on the writer.** A writer must not be able to
waive a required guarantee or turn an unproved obligation into accepted code
through an unchecked escape. Required proof receives machine judgment, not
trust in the author. A weaker requirement is a different task; successful
checking cannot recover omitted requirements or certify an unstated property.

**W4 — Support independently assigned work.** Where systems are developed
through separate components, their interfaces should expose the obligations
needed to implement and use them without reconstructing each other's
internals. Local guarantees help only to the extent that those obligations
express the intended behavior. This does not select one agent organization
or establish whole-system correctness or performance from local proofs alone.

**R1 — Justify added mechanisms.** A construct must serve the chosen
objectives. Familiarity, novelty, or ease of specification alone is
insufficient. A useful existing mechanism needs no novelty to earn its place.

**R2 — Preserve capability when simplifying.** Reducing language or checker
complexity is useful when the replacement still supports the intended
programs, their safety obligations, and their performance needs. A smaller
language that forces an unsuitable architecture has not achieved this goal.

**R3 — Use regularity where it helps.** Prefer predictable forms when they
reduce ambiguity, conflicting interpretations, or repair effort. Regularity
alone does not establish efficient programs, adequate expressivity, or the
superiority of one exact syntax or formatting convention.

**R4 — Give useful feedback before execution.** Prefer making prohibited
states unrepresentable, then rejecting them with actionable explanations.
An executable fallback cannot substitute for required static proof. Expected
input and environment failures remain intended program behavior.

**T3 — Use established facts without charging for impossible failures.**
When a fact is proved before execution, use it to remove unnecessary work or
permit efficient execution. Preserving the observables of a defect outside
the guarantee is not a reason to withhold a valid optimization. Intended
errors, effects, and required safety remain part of the semantics.

**T4 — Expose consequential resource relations.** When resource constraints
affect safe composition or performance, make the relevant obligations
available at the interfaces where decisions depend on them. Concrete capacity
representations, release protocols, and scheduling policies are design
choices whose costs and guarantees need separate grounds.

**D17 — Justify representation privileges.** An implementation seeking a
privilege on the strength of an invariant must establish and preserve that
invariant for the exact representation. Missing proof grants no such
privilege. The proof vocabulary and any remaining trusted implementation
boundary require their own justification.

**R6 — Keep surrounding architecture open.** Avoid unnecessary dependence
on a particular instruction set or software stack. The intended author does
not prescribe one operating-system shape, layering scheme, or reuse model.
