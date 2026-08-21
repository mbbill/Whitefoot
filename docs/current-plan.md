# Current Plan — proof-derived parallelism

Status: PROPOSED (branch `par/proof-derived-parallelism`, 2026-08-21). This
proposal authorizes no execution. Batch 0074's work was carried out under the
owner's chartering direction of 2026-08-21 recorded in
`research/investigations/proof-derived-parallelism/DESIGN.md` §0; this plan is
the durable sequencing that direction implies, and it becomes ACTIVE only with
the owner's approval at merge.

Derived from Direction Outline revision 46 and main at
`4f01bab6`. Supersedes the completed claim-only trap-surface plan in place.
Active language authority: v0.33 at `spec/kernel-spec.md`, SHA-256
`fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`; the branch
carries a v0.34 CANDIDATE adding [PAR-1], activated only by the merge approval.

## Objective

Make Whitefoot's existing proofs pay for concurrency. The compiler already
derives, for acceptance, exactly the facts a legality judgment for overlapped
execution needs: resolved places, exact effect rows, the [OWN-7] overlap
relation, the [EFF-2] call-boundary projection, and the call graph. Read a
permission off those proofs, state the law that a permitted overlap is
unobservable, and let a runtime decide profitability. No writer construct
declares parallelism, no resource is a language concept, and no accepted
program changes.

## Workstreams

- **W1 — compute lane v1 (landed on the branch; the first item this plan
  covers).** The permission judgment P over sibling call pairs, the
  non-normative `--par-ledger` developer output, default-off runtime
  actualization behind `WF_WORKERS`, the spec CANDIDATE v0.34 [PAR-1] rule,
  compiler tests including each named counterexample shape, and the measured
  demo. Recorded in `docs/ongoing/0074-proof-derived-parallelism.md`.
- **W2 — the I/O concurrency lane (sequenced first among the remaining
  work).** A completion-based family for overlapping host waits. This is where
  the investigation found the profit: 2.83x on the dominant term of a
  directory-walking workload that is 86% I/O, against a compute-lane delta of
  roughly 0.15% of the same frame — with the recorded caveat that part of that
  2.83x is an artifact of the measuring machine's security daemon and must be
  re-measured before the number is used to justify anything. It is a separate
  packet: it needs its own spec question
  (`blocks` rows are exactly what W1's row gate refuses), its own runtime, and
  its own owner approval. Nothing in W3 or W4 precedes it.
- **W3 — the `pal` marker.** The writer-visible structural obligation of
  PAL.md §6: non-authoritative, never gating legality, and therefore a grammar
  plus FORM-table plus teaching-text packet of its own. It buys the writer a
  gradient the ledger currently supplies by hand.
- **W4 — permission growth.** Indexed-loop permission (Tier A) and
  buffer-view splitting (Tier B), each with its recorded hazard ([OWN-9]
  granularity; the c2-F4 aliasing case), and claim-bearing actualization with
  the trap-arbitration ruling already on file. Each widening is a [PAR-1]
  amendment, which is the cost W1's necessary-condition form imposes and the
  first thing to revisit if it bites.

## Boundaries and invariants

Permission is never an obligation: a build with no runtime, or with
`WF_WORKERS` unset, is byte-identical in behavior to today, and every test
that passes sequentially must pass under overlap. Worker count, schedule, and
thread identity are outside the language and outside every rule. Acceptance is
untouched in both directions — P consults typing, rows, places, the CFG, and
the call graph, never the entailment fact state, so facts-on and facts-off
behavior are identical by construction. No arbitration machinery, parked lane,
or coordinator is built while eligible regions are claim-free.

## Acceptance

- A real program compiles, is judged, hands work to lanes, and produces
  byte-identical output at every worker count, with the granted-lane count
  measured rather than assumed.
- Each of the four permission conditions denies its own named counterexample,
  and each denial cites the condition that actually judged it.
- The ledger explains every analyzed site, so a sequentialized region is
  visible rather than silent.
- The repository gate is green with the candidate declared, and the owner
  packet carries the exact candidate SHA-256, diff, impact inventory,
  verifier output, and the protected coverage annotation [PAR-1] needs.

## Exclusions

No writer parallelism construct, no thread or task type, no heartbeat
profitability policy, no `reduce`-clause regrouping, no arbitration for
claim-bearing regions, no generic-container work, and no protected conformance
change beyond the single [PAR-1] coverage annotation the merge approval must
name.
