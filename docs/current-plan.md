# Current Plan

Status: ACTIVE — the owner approved this plan, the 19-item amendment
bundle, and the v0.20 batch authorization on 2026-08-06 ("都批"). The
previous milestone (tasks 0023-0025) completed on 2026-08-06.

Derived from: [Direction Outline revision 14](roadmap.md), items `CAND-8`,
`PERF-1`, `FLOOR-1`, `PROOF-1`, `STORE-1`, and `VERIFY-2`

## Goal

Answer the measured latency-floor question and finish the corpus: determine
whether the compiler can lower the trap-carrying byte walk to a wide
(SIMD/word) stride without weakening any required check — by ordinary
lowering if legal, by an honest recorded finding naming the missing proof
or language mechanism if not — and execute the owner's protected-source
rulings plus the v0.20 gap batch so the conformance lane reaches a fully
settled state.

## Work

1. **Check-aware wide-scan lowering slice (PERF-1/FLOOR).** Preregister,
   then investigate lowering the fused walk to a wide stride: the emitted
   loop may process W bytes per step only if every required bounds/trap
   obligation is preserved observably (a trap must still fire at the exact
   byte and with the exact record). Candidate routes to evaluate honestly:
   pure lowering transforms on the existing checked IR; a
   compiler-derived per-block obligation hoist that provably preserves
   trap identity; or the recorded conclusion that a verified fact family
   (PROOF-1-class) is required first. A credited win reruns the frozen
   baseline; a negative names the exact obstruction with a witness.
2. **Protected-source rulings execution** (on the owner's approval of the
   19-item bundle): the amendments land with per-case verification; the
   lane's remaining red becomes exactly the OWN-6-gap cases.
3. **v0.20 micro spec batch** (on the owner's authorization): OWN-6
   return-position reborrow disposition, TYPE-7/OWN-1 simultaneous-
   rejection ordering, and the OWN-1 return-position question if the owner
   prefers the carve-in over the source edit — drafted, verified, and
   exact-approved through the specification-change workflow.
4. **Return and replace**: rerun the baseline after item 1, record, update
   the outline, and replace this plan (candidates: the traversal-widening
   proposal with the backing-lifetime decision, or the next attributed
   cause).

## Verification

- Item 1 preserves facts-off correctness and every required check; trap
  identity is oracle-tested (exact byte, exact record) on hostile inputs;
  §9.1 gates and the wfgrep oracle hold on every accepted change.
- Items 2-3 touch protected material and specification bytes only under
  their recorded approvals.

## Done when

- the wide-scan question has a credited win with a rerun baseline or a
  recorded negative naming the missing mechanism;
- the corpus lane is fully settled (green plus only owner-acknowledged
  open gaps); and
- this plan is replaced naming the next selection.

## Not in this stage

- No traversal, parallelism, or new system families; no STORE-2 growth
  mechanism; no PROOF-1 implementation beyond what item 1's finding
  explicitly justifies proposing.

## Parallel research

The owner's separate obligation-discharge investigation proceeds
independently in its own records; this plan neither sequences nor depends
on it.
