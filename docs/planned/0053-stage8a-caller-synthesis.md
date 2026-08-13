# 0053 — Stage 8a caller synthesis

**Planned task.** Decomposed from the `ACTIVE` Current Plan selected
2026-08-12, Workstream 8a `Caller audit`. It is not yet claimed; claiming fills
in status, owner, workspace, and base revision and moves this number unchanged
to `docs/ongoing/`. Delete it if a replacement plan does not explicitly carry
this scope.

- **Authority:** Direction Outline revision 33 item `PROOF-8` and the ACTIVE
  Current Plan's complete fourteen-call `read_bits` and twenty-call
  `append_slice` inventory requirement

## Goal

Produce one exact, typed, deterministic caller map for all 34 real calls using
only the two positive local witnesses from tasks 0051 and 0052. Fix the
smallest Stage 8b caller-repair boundary without implementing postcondition
syntax, general assignment equality, variable-addition entailment, or any
other production mechanism.

## Direction and invariants

- The census is exactly fourteen `read_bits` calls across the four-source
  raw-DEFLATE unit and twenty `append_slice` calls across wfgrep and the
  raw-DEFLATE boundary.
- Each row records source/function/concrete-instance identity, checked call
  occurrence, actuals, receiving shape, hypothetical fact, exact live support,
  kills/scope exits/joins, first consuming obligation or requirement,
  `discharged` or `unproved`, and the one planned repair.
- `read_bits` facts begin on the `Ok` payload. Current `set` creates no RHS
  equality; payload scope exit must remove the fact from the outer binding.
- A direct `append_slice` result may be modeled only as the future verified
  normal-result relation instantiated onto its direct receiving target. This
  is not a general `set` rule.
- The wfgrep host-copy element write must preserve the length relation; the
  subsequent scalar `set length = length +wrap copied` must kill it, and S7
  must not synthesize a variable-plus-variable replacement.

## Method

1. Claim only after tasks 0051 and 0052 have terminal positive results and
   cite their landed commits and canonical acceptance sections.
2. Recompute the frozen source identities and enumerate all calls directly
   from the checked real unit, not from an old table.
3. Walk the three admitted flow classes: `Ok payload -> set -> scope exit`,
   direct postcondition result -> receiving target, and append result -> host
   element write -> copied count -> scalar set -> join.
4. Use disposable checked-tree or `FactState` probes only where a row cannot be
   established directly from retained checked metadata. Remove them before
   integration.
5. Reconcile every call and downstream goal. Expected results are fourteen
   payload-delivery gaps and append `19 discharged / 1 unproved`; the sole
   append gap is the wfgrep separator after host copy. Any refutation or other
   unproved path is a blocker.
6. Append the full table and PASS/STOP synthesis to the existing acceptance
   record, explicitly stating that a PASS still grants no Stage 8b authority
   before DIAG-2 is terminal and exact protected approval is obtained.

## Scope and expected touch set

- Persistent: this task record and
  `research/investigations/obligation-discharge/ACCEPTANCE.md` only.
- Temporary: read-only dump or focused state harnesses below
  `/Users/bytedance/do_not_scan`; no production compiler or consumer bytes.

## Dependencies and integration order

Tasks 0051 and 0052 must both be terminal positive results. Integrate after
0051 then 0052. It may run while the later DIAG-2 tasks continue, but Stage 8b
requires this task and all DIAG-2 tasks terminal on one refreshed baseline.

## Validation

- The checked-tree census is exactly 14/20 and every source call has exactly
  one primary row.
- Every relation is well typed after hypothetical formal/result substitution.
- The fourteen payload facts expire exactly at the current delivery seam.
- The append map is exactly 19/1, with the known host-copy seam as the sole
  unproved row and no refutation.
- Repeated clean walks reproduce byte-identical ordering and counts.
- Temporary bytes are absent; focused checks and `make check` pass.

## Stop condition

Stop if the census differs, any substitution is ill typed, a fourth flow class
appears, a direct result needs general `set` equality, any refutation occurs,
any caller besides the fourteen immutable deliveries and one wfgrep value
branch needs repair, or a solver, third fact source, variable-offset S7,
recognizer, or writer assertion is required.

## Done-when

The canonical acceptance record contains the complete reproducible 14/20 map,
the exact two delivery seams, and an honest Stage 8a PASS or STOP result.
