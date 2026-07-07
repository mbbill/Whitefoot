# Debate round 1 — verdict index (2026-07-01)

Eight topics, each debated by three assigned-position advocates, one adversarial critic, and one judge, all citing corpus evidence cards. Full transcripts: `round1-full-journal-wf_74f9eedf-c49.jsonl`; raw structured output: `round1-verdicts-raw.json`.

Outcome labels: research_needed / prototype_needed / evidence_conflict / design_candidate_later / no_decision. A leading candidate is named even when the outcome is not design_candidate_later.

| Topic | Outcome | Leading candidate (abridged) | Detail |
|---|---|---|---|
| violation-semantics | `research_needed` | Position 3 (graduated obligations), amended post-critique: compiler-proven contracts assumable in every build mode; human-asserted contracts only in per-fact... | [round1-violation-semantics.md](round1-violation-semantics.md) |
| aliasing-model | `design_candidate_later` | Position 1 — checked ownership/exclusive-mutation in the type system as the core non-interference discharge mechanism, expected to land as the core of a hybr... | [round1-aliasing-model.md](round1-aliasing-model.md) |
| arrays-loops | `prototype_needed` | Structured-arrays-first as the default surface, explicitly reframed as the top layer of a composition: a structured/affine-checked core over a contract-beari... | [round1-arrays-loops.md](round1-arrays-loops.md) |
| dispatch-generics | `design_candidate_later` | Advocate 1's skeleton — guaranteed static specialization as the core's only implicit path, with dynamic dispatch available solely through an explicit, restri... | [round1-dispatch-generics.md](round1-dispatch-generics.md) |
| static-vs-profile | `research_needed` | Static correctness contracts as the default channel, plus firewalled offline profile data restricted to cost decisions (position 2) — leading on the strength... | [round1-static-vs-profile.md](round1-static-vs-profile.md) |
| numeric-semantics | `no_decision` | P1's defensible core as a default-agnostic surface architecture: a small vocabulary of named, truthfully-lowered numeric modes (wrapping, checked, and unsafe... | [round1-numeric-semantics.md](round1-numeric-semantics.md) |
| concurrency-model | `research_needed` | Position 1 (type-enforced data-race freedom via Send/Sync-like sharing capabilities plus exclusive borrowing) as the sharing-model core — with the residue qu... | [round1-concurrency-model.md](round1-concurrency-model.md) |
| ir-strategy | `research_needed` | A verifier-checked, semantics-preserving IR above LLVM that carries ownership/non-interference/effect (and provisionally shape/loop) facts structurally and d... | [round1-ir-strategy.md](round1-ir-strategy.md) |

## Cross-topic observations

- Two topics reached `design_candidate_later`: **aliasing-model** (checked ownership/exclusive mutation in the type system) and **dispatch-generics** (guaranteed static specialization core with explicit, restricted dynamic dispatch). These are the closest to decidable.
- Judges repeatedly flagged the same coupling: violation semantics, concurrency/race semantics, and aliasing guarantees cannot be finalized independently — safe-code non-interference proofs quantify over race-free executions, so the memory model is a prior decision.
- `numeric-semantics` ended `no_decision`: the surface architecture (named truthful overflow/FP modes) is agreeable, but the *default* is a policy choice the evidence cannot settle.
- Common evidence gaps across topics: no quantified performance deltas for any contract regime, missing primary C++/Java memory-model text, no purity-check evidence, unverified direct-extraction cards (A/D/C/J), and no prototype-level validation that emitted facts survive lowering and are consumed by passes.
- The critic phase materially changed positions: several advocate claims were struck as uncarded extrapolations (e.g., misread N003, stretched F007), which is exactly the discipline the corpus caveats demanded.
