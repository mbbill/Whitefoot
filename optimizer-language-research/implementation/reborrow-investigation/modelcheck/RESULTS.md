# Model-check results — minimal statement-scoped reborrow rule

Date: 2026-07-18. Harness: `modelcheck_reborrow.py`. Verifies `MINIMAL-RULE.md` rev 2.
This discharges **OBL-4 item 1** (statement-scoped suspension) for the admitted fragment at
the model level. It is evidence for the owner-gated spec decision, not a spec change.

## Method

FR-inherited differential method (K003): generate random small programs; run the
minimal-rule **checker**; for every **accepted** program run an **independent operational
oracle**. The oracle models each borrow as a memory claim with *operational* liveness (not
the rule's suspension) and raises a violation if two simultaneously-usable claims overlap
the same place and at least one is exclusive (`uniq`). An oracle violation on an accepted
program is a checker soundness bug. The oracle shares no acceptance logic with the checker.

The generator deliberately emits the two escape vectors the adversarial review surfaced:
- **borrow-returning callees** (`callee_ret = 'borrow'`) with the result bound — the launder
  that broke rev 1;
- **caller-supplied and multi-statement `'c` regions** (`region_kind in {caller, outer}`).

## Self-tests (all pass)

The oracle is proven **non-vacuous**: the launder program is run against a deliberately
broken rev-1 checker (which accepts it), and the oracle **catches** the resulting aliasing.
Then the same program is run against rev 2, which **rejects** it (OWN-6).

```
ok  wfc-shape accepted                                  (reborrow field, own callee, local 'c, unbound)
ok  wfc-shape oracle-clean
ok  launder rejected by rev-2 (OWN-6)                    (borrow-returning callee + bound result)
ok  oracle catches launder aliasing (non-vacuous)        (broken rev-1 accepts -> oracle flags)
ok  overlapping uniq siblings rejected (OWN-7)
ok  disjoint-field uniq siblings accepted + clean        (the frontend.wf:707 shape)
ok  literal-vs-variable uniq index siblings rejected (OWN-7)
ok  shared child from uniq parent accepted
ok  caller-supplied 'c rejected (OWN-4)
```

## Random run (deterministic seed 20260718)

```
programs=1000000 accepted=190835 rejected=809165
SOUNDNESS violations on accepted: 0        <-- the result
launder attempts generated: 363153; rejected by checker: 363153 (100.0%)
rejections by rule: OWN-6 392268, OWN-4 366060, OWN-7 50837
rejected-but-oracle-clean (over-rejection): 307205
```

(200k-program run identical in character: 0 soundness violations, 100% launder rejection.)

## Reading

- **0 soundness violations across 1,000,000 programs.** No program the minimal rule accepts
  produces two simultaneously-usable overlapping exclusive claims. The no-alias invariant
  F001 holds for the admitted fragment in this model — the performance-preservation claim.
- **100% of the 363,153 launder attempts are rejected**, all at OWN-6 (the rmode-non-borrow
  admission condition). The miscompile rev 1 admitted is closed.
- **Over-rejection is conservative and expected.** 307,205 rejected programs run
  oracle-clean — programs the strict rule rejects (borrow-returning callee without an actual
  escape, benign caller-`'c`) that would not in fact alias. This is fail-closed (over-reject,
  never under-reject) and does not affect wfc: wfc's shape (local region, `own`-returning
  callee, disjoint siblings) is always accepted (coverage discharge, `MINIMAL-RULE.md` §7).

## Honest scope and what remains

- This is a **focused abstract model** of the reborrow rule (places, uniq/shared claims,
  statement-scoped children, the escape vectors), not the full-language production checker.
  It targets the reborrow soundness invariant directly and independently. Folding the rule
  into `prototype/checker/checker.py` and extending the shared model-check oracle with
  exclusivity is **production-integration work for after the spec lands**, not part of this
  evidence step.
- The result is "no accepted program aliases in 1M random draws," a strong failure-to-break,
  not a closed-form proof. It complements, and does not replace, the two remaining evidence
  pieces: the **Featherweight-Rust reconciliation** (checker stays singleton-rooted) and the
  **hostile no-alias fact-channel review** (re-derive `noalias` under §4; attack the escape
  clauses directly; recommend a structural exclusivity assertion in the checker).

## Reproduce

```
python3 modelcheck_reborrow.py 1000000
```
Exit 0 iff zero soundness violations. Deterministic seed; re-run is bit-stable.
