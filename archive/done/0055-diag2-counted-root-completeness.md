# 0055 — DIAG-2 counted-root completeness

- **Status:** `DONE` (2026-08-13)
- **Authority:** the `ACTIVE` Current Plan's existing-DIAG-2 trust repair,
  under active v0.27 DIAG-2, ENT-3 S11, ENT-4, and ENT-5
- **Landed commit:**
  `491446af053bfe8db95941e6093b30f4ff9cfb7a`

## Outcome

The canonical entailment walk now retains one source-ordered
`CountedDerivationSet` for every checked counted statement. Each set contains
the five normative S11 relations and exactly eight directed atomic roots:
both directions of the two endpoint-capture equalities and binder
initialization equality, plus the lower non-strict and upper strict body-entry
bounds.

All eight atomic roots enter the existing sole `DerivationLedger` root
channel. Existing finalization retains their transitive parents and remaps
their IDs together with every other mandatory root; there is no second
semantic walk, closure, retention list, verifier, or pre-remap identity.

The counted preheader records endpoint and capture identity after closure and
before continuing kills. Body-entry roots are recorded after that existing
kill boundary. Kills still remove ordinary live facts and parents, while each
normative S11 root group remains observationally retained even when unused or
later invalidated. Same-walk encountered/completed counters make a missing or
duplicate counted group an internal compiler defect.

The implementation touched only five existing entailment implementation/test
files. It changes no source acceptance, lowering, runtime behavior, fact
family, loop semantics, specification, protected evidence, real consumer, or
canonical gate wiring.

## Evidence and validation

- The test-only structural checker validates source occurrence identity,
  normalized relations, proof points, parent typing, remapping, root
  cardinality, reachability, and deterministic source order. Mutation controls
  fail when a root is deleted or duplicated, its path or relation changes, a
  snapshot marker or same-conclusion parent is corrupted, or a killed parent
  is retained.
- Hostile coverage includes unused, zero-trip, reversed, singleton, and
  maximum-edge ranges; mutable, projected, interleaved, and dereferenced
  endpoints; both endpoint writes; binder updates; early return; matching and
  enclosing breaks; nesting; contradictory and mixed joins; ordinary-loop
  near misses; and legal concrete generic instances with const-dependent
  value endpoints.
- A paired control confirms that an ordinary query-derived consequence does
  not survive the same mutable kill without the counted-preheader snapshot.
  Counterfactual rewalk remains narrow and does not publish counted roots.
- `tests/programs/sha256_abc.wf` has exactly three counted groups, fifteen
  semantic S11 relations, and twenty-four directed atomic roots, together with
  exact roots for its nine accepted bounds obligations. UTF-8, all four
  raw-DEFLATE sources, and wfgrep retain complete bounds, call, and counted
  roots without changed outcomes.
- Twenty repeated compilations of the concrete generic fixture produced a
  byte-identical normalized ledger. Targeted tests passed 44/44 and the final
  focused entailment suite passed 112/112.
- The first post-change compiler-gate invocation stopped at formatting only.
  Formatting changed only the same five authorized files; the complete gate
  was rerun from the beginning and passed 718/718 library tests and 30/30 real
  programs, ending `WHITEFOOT COMPILER GATE GREEN`. The real-program group
  completed in 1072.83 seconds.
- The final repository gate passed archive integrity for 28 recorded
  specifications, 23/23 independent runner tests, 131/131 rule coverage,
  718/718 library tests, and 30/30 real programs. Its real-program group
  completed in 1043.83 seconds and it ended
  `WHITEFOOT GATE GREEN (active compiler + independent evidence)`.
- Independent review of the final implementation found no P0, P1, or P2
  correctness, authority, completeness, sequencing, or proportionality issue.
- Active specification SHA-256 remained
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`;
  protected acceptance evidence remained
  `271abdf48dcb71e7698f8f1e1d5c18c23adf256115278e5f7ec7ca25226d7df3`.

## Remaining dependency

This task closes only counted-root retention. Task 0056 must independently
audit the complete task-0054/task-0055 derivation set, measure its bounded
cost, pass complete gates, and install exact owner-approved canonical evidence.
The existing-DIAG-2 trust prerequisite is not terminal until task 0056 closes
terminal-positive.

Tasks 0051 and 0052 must refresh once onto this task's terminal closure
revision and rerun their complete matrices. Their withdrawn candidate must not
be installed. Only after their new combined protected evidence is exactly
approved, installed, and both tasks are terminal-positive may task 0053 be
claimed.

No Stage 8b candidate work is permitted until tasks 0053 and 0056 are both
terminal-positive. Stage 8b then remains subject to its separate exact
specification and protected-evidence owner-approval gate.
