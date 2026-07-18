# The Whitefoot Constitution

Adopted 2026-07-05; amended 2026-07-05 (priority structure) and 2026-07-07 (D2a). This document supersedes and grounds all owner directives; the individual rulings (D0–D4, D0a/D1a/D2a) live in `optimizer-language-research/notes/user-directives.md`. Decision log: `optimizer-language-research/implementation/decision-gates.md`.


Structured objective, in priority order:

**P0 — PERFORMANCE (the reason to exist):** machine-code performance ranks above the remaining goals. **The Rust test (R0): if a decision leaves us equivalent to Rust on performance + cheat-proofness, the decision failed — "use Rust" was cheaper. Every major decision names its delta over Rust.** R0 reading (owner-affirmed 2026-07-08): a decision satisfies R0 by naming a delta over Rust on ANY of P0 (machine performance), W3 (cheat-proofness), or W1 (weak-writer robustness), not machine performance only; equivalence to Rust on all three is the failure condition.** Current deltas of record: more optimizer-visible facts than rustc emits (per-node numeric modes, exact effect rows, region-explicit borrows, checked laws), W3 (Rust's unsafe is writer-accessible everywhere), no debug/release semantic divergence.

**P1 — AI-WRITABILITY (what "AI writes it without problems" means):**
- **W1 — weak-writer robustness**: correctly writable from the in-context spec by LOW-capability models, not just frontier ones. All codegen experiments run across model capability tiers; a construct only strong models use correctly fails W1.
- **W2 — context economy**: spec/teaching pack fits limited context. DEPRIORITIZED per D2a (2026-07-07): windows are growing; token counts are measured, never gating; the regularity invariants survive under W1 grounding.
- **W3 — cheat-proofness**: the writer cannot hack around the checker — no writer-emittable unsafe or trust; contracts cannot be weakened to make a failing body pass; exhaustiveness cannot be silenced; checks are elidable only by proof; failures trap with reports, never silently; canonical bytes leave nowhere to hide edits. Some AIs cheat when stuck; cheating is made unrepresentable, not detected later.

**STANDING THEOREMS (derived commitments — in full force; revisitable only by refuting a premise, never by preference):**

- **T1 — Memory and thread safety (D1)**: data races, use-after-free, dangling references, double-free, and uninitialized reads are unrepresentable in accepted programs. NOT an axiom — deduced twice from the goals: (a) from P1 via R4/W1/W3 — an unattended writer cannot debug latent memory bugs, runtime failure is the worst AI feedback channel, and a cheating writer must not paper over them; (b) from P0 — ownership/exclusivity is the optimizer's noalias fact base (F001) and race-freedom keeps those proofs sound (a race falsifies compiler reasoning retroactively). Natural experiment: Rust (treatment) vs C/C++ (control) — one type system yields both safety and optimizer facts. 
- **T2 — No-UB envelope**: accepted programs have no undefined behavior, conditional on the declared TCB (round-3 Layer 4). Derived from T1's premises plus R4: silent corruption is the forbidden failure mode.

**BALANCE RULE:** W1/W2 and non-floor aspects of P0 trade off; no decision may claim to optimize all simultaneously. Evidence decides each case; where genuine conflict remains after evidence, P0 wins. Theorems stand while their premises stand — they are conclusions, not preferences, and are never traded against W1/W2 convenience.

Decision rules R1-R6 remain in force under this ordering:

- **R1 — Earn your place.** A construct exists only if it serves P0 or P1. Serving human authorship counts for nothing.
- **R2 — A cut that harms AI codegen is a wrong cut.** Simplicity is never a sufficient reason. Precedent: generics (round-2 checker-collapse); natural experiment: Go pre-1.18.
- **R3 — One way to say anything, and the survivor is chosen by evidence** for P0+P1 among candidates, measured under W1 (weak writers). Minimality-selected forms are PROVISIONAL.
- **R4 — Shift-left everything.** Unrepresentable > check-time rejection with rule-citing diagnostics > runtime trap > (forbidden) silent corruption.
- **R5 — Human exceptions are explicit.** Readability is a non-goal; auditability of the trusted base (D0a) is the deliberate exception.
- **R6 — The stack is negotiable long-horizon** (compiler, possibly self-improving; hardware); near-term, artifacts must not marry one backend or ISA.


Subordinate owner directives (D0–D4) are instances of this constitution.
