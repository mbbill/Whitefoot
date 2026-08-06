# Current Plan

Status: ACTIVE — the owner approved this plan on 2026-08-06 ("都批"),
together with the two conformance rulings its Work item 4 executes. The
previous milestone (v0.18/v0.19 activation and the first command slice,
tasks 0004-0018) completed on 2026-08-06.

Derived from: [Direction Outline revision 12](roadmap.md), items `CAND-8`,
`PERF-1`, `BOUND-1`, `VERIFY-2`, and `PROOF-1`

## Goal

Measure what the completed first slice is worth and consolidate the compiler
capabilities it exposed, without widening the project: establish the
zero-change PERF-1 baseline of the exact frozen sequential `wfgrep` slice
against a preregistered comparator, implement the two compiler capabilities
the slice's own execution demonstrated missing, and execute the owner's
rulings on the conformance findings — so the next widening decision
(directory traversal, and with it the deliberately open backing-lifetime
rule) is taken on measured ground.

Per the owner's framing, the deliverable is knowledge: an attributed
baseline (or an honest inability to attribute), each finding closed or
converted into a named direction input, and negative results retained.

## Work

1. **Zero-change wfgrep baseline (PERF-1).** Preregister the envelope —
   pinned corpus with digests, frozen `tests/programs/wfgrep.wf` bytes,
   comparator (`grep -h -F` at a pinned build; the RG-BASE noise findings
   bind the precision protocol), timed phases, statistic and materiality
   rules — then measure, profile, and attribute the first material
   divergence per the layer chain before any widening. The scalar newline
   scan retaining its bounds trap (task-0016 correction) is the
   preregistered first attribution suspect, feeding PROOF-1 only if the
   measured loss lands there.
2. **Borrow-mode parameters for system nominal types** (unsupported
   specified capability, task-0015 finding): implement on the normal path,
   then decompose `wfgrep`'s ~500-line `main` into helpers as the witness
   that the capability composes; the §9.1 gates must hold unchanged on the
   refactored program.
3. **Rule-id plumbing for pre-semantic rejections** (task-0014 bucket 1, 45
   cases): populate `rule_id` at Lexing/CanonicalSource/Parsing/Resolution
   stops; the conformance lane consumes it.
4. **Conformance rulings execution** (buckets 2-4, after the owner rules):
   the 41 incomplete-unit protected sources, the 35 runnable→pending status
   corrections, and the 2 divergence investigations (each investigated to a
   compiler-defect fix with regression or a protected-expectation finding
   returned to the owner).
5. **Return and replace.** Record baseline results in RESULTS, update the
   outline, and replace this plan naming either the first attributed
   performance blocker or the widening proposal (traversal + the
   backing-lifetime decision) as the next selection.

## Verification

- The baseline claim is only as wide as the preregistered envelope; parity
  and losses are retained as findings; no timing claim precedes correctness
  and comparable work.
- Work items 2-3 are ordinary delivery (no specification change): gates
  green before and after; the refactored wfgrep passes the same oracle and
  §9.1 gates byte-for-byte on behavior.
- Bucket executions touch protected material only per the owner's explicit
  rulings, recorded in `governance/APPROVALS.md` where protected bytes
  change.

## Done when

- a preregistered baseline result (win, parity, or attributed loss) is
  recorded in RESULTS with its envelope;
- system-type borrow parameters are implemented with the helper-decomposed
  wfgrep witness green;
- the conformance lane is green or its remaining red is owner-ruled and
  recorded; and
- this plan is replaced naming the next slice or blocker.

## Not in this stage

- No directory traversal, ignore stack, parallelism, or new system
  families; no STORE-2 reopening (its witness is recorded and waits for the
  widening proposal).
- No optimizer or proof implementation; PROOF-1 enters only on an attributed
  material loss at the retained-check layer.
- No specification change of any kind.

## Parallel research

None proposed.
