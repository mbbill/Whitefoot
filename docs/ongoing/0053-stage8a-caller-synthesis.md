# 0053 — Stage 8a caller synthesis

- **Status:** `IN PROGRESS` (2026-08-13)
- **Owner / workspace:** Codex executor /
  `/Users/bytedance/do_not_scan/whitefoot-0053-stage8a-caller-synthesis`, branch
  `codex/0053-stage8a-caller-synthesis`
- **Base revision:**
  `30b0ccc10d394dcce3403aaf49d149aea82f741d`
- **Authority:** the ACTIVE Current Plan selected 2026-08-12 and derived from
  Direction Outline revision 32, plus current Direction Outline revision 34
  item `PROOF-8`, for the complete fourteen-call `read_bits` and twenty-call
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

1. Claim only after the integration commit that moves tasks 0051 and 0052 to
   terminal-positive history and lands both local-witness sections in the
   research acceptance record. Cite that full commit and both sections. The
   same closure baseline records task 0056 terminal-positive; it does not gate
   this caller census.
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
6. Append the full table and PASS/STOP synthesis to the research acceptance
   record through ordinary lead review, then close this task. Preserve exact
   commands, source identities, byte-stable ordering, limitations, and the
   distinction between measured facts and any future repair. Only after this
   task and the independent task 0056 are terminal-positive may the separate
   Stage 8b exact specification/protected-conformance activation candidate be
   prepared.

## Scope and expected touch set

- Persistent: this task record and
  `research/investigations/obligation-discharge/ACCEPTANCE.md` only.
- Temporary: read-only dump or focused state harnesses below
  `/Users/bytedance/do_not_scan`; no production compiler or consumer bytes.

## Dependencies and integration order

Tasks 0051 and 0052 must be terminal-positive with their refreshed research
sections landed. Claim from the commit that closes those tasks and record its
full SHA as the premise. Task 0056 is independently terminal-positive on that
baseline; Stage 8b candidate work may begin only after task 0053 also becomes
terminal-positive.

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

## Progress

- **Claimed:** from the exact closure commit
  `30b0ccc10d394dcce3403aaf49d149aea82f741d`, which lands the refreshed 0051
  and 0052 research sections, moves both tasks to terminal-positive history,
  and records task 0056 terminal-positive independently.
- **Current:** refresh the frozen source identities and enumerate all 14
  `read_bits` and 20 `append_slice` occurrences from checked real units.
- **Next:** build the typed, deterministic caller map, reconcile every
  downstream goal, and stop on any census or flow-class divergence.

## Done-when

The research acceptance record contains the complete reproducible 14/20 map,
the two delivery seams, and an honest Stage 8a PASS or STOP result, and this
task is terminal before any Stage 8b activation candidate begins.
