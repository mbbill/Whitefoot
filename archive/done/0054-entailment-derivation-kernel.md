# 0054 — entailment derivation kernel

- **Status:** `DONE` (2026-08-13)
- **Authority:** the `ACTIVE` Current Plan's existing-DIAG-2 trust repair,
  under active v0.27 DIAG-2 and ENT-4/ENT-5
- **Landed commit:**
  `0e9a206188d8cc37ec3bb248889e42109122246a`

## Outcome

The canonical entailment pass now retains one function-local dense shared-DAG
derivation ledger inside each `FunctionEntailment`. The existing source,
closure, strengthening, join, materialization, and query operations record
canonical parents as they run; derivation retention does not rerun closure or
add another semantic walk.

Every accepted subscript and every discharged ordinary-call goal retains one
exact root, including concrete goal substitution and contradiction discharge.
Failed and refuted outcomes retain no positive root. Joins name every reaching
predecessor in ordinal order, kills remove facts and their live parents
together, and finalization retains and remaps only the transitive closure of
mandatory roots and their events.

The implementation adds no serialization, portable identity, replay protocol,
ProofFlow, shadow verifier, lowering authority, source acceptance, diagnostic,
runtime, specification, protected-evidence, or real-program change. It touched
only the five existing entailment implementation files and their ordinary
test module.

## Evidence and validation

- A test-only structural walker validates every retained node kind, arithmetic,
  parent typing and order, acyclicity, event identity, inventories,
  root/outcome cardinality, exact metadata, root reachability, and the absence
  of positive roots on failed outcomes.
- Hostile coverage includes direct and implicit bounds, transitivity,
  subsumption, equality, strict-derived disequality, opaque and projected
  goals, both contradiction paths, predecessor-complete joins,
  materialization, kills, calls, generics, recursion, borrows, loops, and 20
  deterministic repeated compilations.
- The focused entailment suite passed 106/106 tests. The final compiler gate
  passed 712/712 library tests and 30/30 real programs, ending
  `WHITEFOOT COMPILER GATE GREEN`; the real-program group completed in
  955.60 seconds.
- The final repository gate passed 23/23 independent runner checks, 131/131
  rule coverage, 712/712 library tests, and 30/30 real programs, ending
  `WHITEFOOT GATE GREEN`; its real-program group completed in 1029.76 seconds.
- Independent review found no P0/P1 correctness, authority, completeness, or
  proportionality issue.
- Active specification SHA-256 remained
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`;
  protected acceptance evidence remained
  `271abdf48dcb71e7698f8f1e1d5c18c23adf256115278e5f7ec7ca25226d7df3`.

## Remaining dependency

Task 0055 must build on this exact implementation and retain every normative
S11 root group for every counted statement: five semantic roots and eight
directed atomic-bound roots per occurrence, including unused and zero-trip
occurrences. Task 0056 then audits completeness and measures bounded cost. The
DIAG-2 trust prerequisite is not terminal until both tasks close, and Stage 8b
remains blocked on terminal-positive tasks 0053 and 0056.
