# 0066 — Stage 9a deterministic claim ledger

- **Status:** `DONE` (2026-08-15)
- **Authority:** the ACTIVE Current Plan, Workstream 9a `deterministic claim
  ledger`, under Direction Outline `PROOF-8`
- **Registration:** `6e3a545`
- **Landed implementation:**
  `e04d3acad80e1260c4f1aee24d8f45cba5140d84`

## Outcome

The checked program now exposes one deterministic, read-only ledger for every
named claim. Each entry preserves bundle-local source identity, source
spelling, name, predicate, justification, lifecycle disposition and retained
lifecycle proof when redundant or refuted, plus every exact non-lifecycle
retained-root use and S3 premise ID that traverses the existing finalized
function-local derivation DAG. Bounds
and call uses additionally carry their exact existing provenance inventories;
missing or duplicate required mappings fail closed as an internal semantic
compiler failure. A claim-free program publishes an empty ledger without a
source or root scan.

No second semantic walk or closure, copied proof graph, serialized artifact,
portable identity, optimizer input, lowering consumer, acceptance change,
diagnostic change, runtime change, specification change, or
protected-compliance change was introduced.

One necessary internal accounting field was added:
`DerivationMetrics::claim_lifecycle_roots`. The new lifecycle roots already
required by this task must be counted for retained-size and root-class
accuracy; the existing metric classes and their meanings are unchanged, and
no acceptance or lowering path consumes the new field. This is the precise
correction to the live record's overbroad shorthand that proof metrics would
remain byte-for-byte unchanged.

The ledger itself enumerates the complete installed real-source populations:
UTF-8 `2`, four-source raw-DEFLATE `12`, and wfgrep `8`, with stable ordering
and exact nonempty premise links across repeated analyses. Focused synthetic
coverage includes retained, redundant, refuted, contradiction, kill, join,
loop, generic-instance, ordinary-call, zero-argument call, direct-result, and
hostile missing-map cases. The non-heavy entailment selection passed `133/133`;
the frozen-real owning test passed `1/1` in `414.00s`, against Stage 8b
consumer commit `5fd017b46973e5cbf990fe3fc92a2cc20a76f91c` at `412.36s`
(about `0.4%`).

The complete compiler gate is green: library `816/816`, grammar `9/9`,
generated grammar tables `1/1`, migration `36/36`, specification integrity
`10/10`, canonical corpus `3/3`, and real programs `32/32`; the known
conformance integration remains deliberately ignored in this gate and rustdoc
passes with warnings denied. The frontend still identifies active v0.28, all
132 rules, and all 20 activation-chain links.

The repository-root `make check` also completed successfully after the
real-program group passed `32/32` in `2102.97s`, ending both
`WHITEFOOT COMPILER GATE GREEN` and
`WHITEFOOT GATE GREEN (active compiler + independent evidence)`. The separate
`make conformance-run` finished in `199.25s` with the exact unchanged boundary
`Pass=423 Fail=1 Skip=13`; it exited nonzero (`make` `2`, test process `101`)
only because `own3-pos-outlives-store` still expects `Run(0)` while reaching
`Unsupported(RegionsAndBorrows)`. No other protected verdict moved.

Stage 9a is terminal. Stage 9b candidate preparation is next, but its exact
specification and protected-conformance bytes still require the independent
owner approval recorded by the Current Plan. This file is frozen coordination
history, not current authority.
