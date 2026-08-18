# Check dissolution — protected conformance inventory (list only)

Batch 0071, task #47, executor E3. This is the complete inventory of
every `tests/conformance/` case whose source touches `check` and every
manifest row touching `check`/OP-5, each with a proposed disposition for
the protected v0.32 candidate the lead prepares. **No conformance byte is
edited by this task**; every disposition below is a proposal requiring
the owner's exact-byte approval on the protected surface.

Classification method: contract finals (the mandatory final `check_stmt`
of a `requires`/`ensures` block) are counted separately from body checks,
because dissolution keeps the contract-final syntax; the verdict engine
compares run-versus-trap and rejection rules only, never trap-record
bytes, so a leg-A migration (body check to named claim) preserves every
runnable verdict while the record bytes on a failing path change.

## Summary

- 188 case files touch `check`; 462 cases total.
- 25 cases: `keep-unchanged` — only contract finals; zero candidate bytes.
- 152 cases: `leg-A migration` — body checks are scaffolding; verdict
  unchanged. Six of these expect a trap and need a refutation review
  (a migrated claim whose exact negation the closed facts derive would
  turn the expected runtime trap into a CLM-2 rejection; none is
  expected to, because every such predicate is runtime-computed, but the
  candidate must verify each). Where a manifest row lists OP-5 among the
  exercised rules for scaffolding checks, that row's rules update to
  CLM-1.
- 11 cases: `subject` — OP-5/check or the CLM-3 check/claim asymmetry is
  the tested surface itself; each needs an owner decision to re-author
  as the CLM analogue or retire:
  - op5-pos-check-pass, op5-neg-check-fails, trap-op5-check-fails,
    scope4-pos-check-traps, x-arith-check-catches-wrapped-overflow-traps
    (OP-5 statement behavior; CLM-1 analogues partially exist already,
    e.g. clm1-trap-runtime-violation);
  - prv3-neg-external-check (a body check is not a PRV-3 repair; the
    v0.32 analogue asserts the same for the claim, which
    clm-family cases may already cover);
  - ent3-pos-band-check-decomposition (S2 decomposition; re-author on
    S3/claim or S1/branch — the recorded goal-decomposition analysis);
  - clm3-neg-body-check-bounds, clm3-neg-body-check-requires,
    clm3-neg-transitive-check-summary, clm3-pos-transitive-value-branch
    (CLM-3 cases whose fixtures use body checks inside or beside strict
    closures; under v0.32 a body check does not parse, and a claim in a
    demanded closure rejects — each case needs re-authoring against the
    strengthened `deny_claims` meaning).
- 4 manifest rows touch check only in prose (doc text), with no check in
  the case source: clm2-pos-redundant-claim-advisory,
  form7-neg-out-of-range, op2-pos-overflow-obligation-discharged,
  trap-syshost-copybytes-offset-beyond-buffer. Disposition: doc-prose
  update only where the candidate's CLM-2/OP-5 rewording makes the
  sentence stale; no verdict or source change.

A grammar-level note for the candidate: after dissolution a body `check`
fails at parse (GRAM-4 statement selection), so every `subject` case that
keeps a body check as its own subject must either move under the
candidate's new expected rejection or be retired; `keep-unchanged` cases
are genuinely byte-stable because the contract-final spelling survives.

## Per-case table

| case id | body checks | contract finals | manifest rules | expect | status | proposed disposition |
| --- | --- | --- | --- | --- | --- | --- |
| accept-sysname-lookalike-outside-kind | 1 | 0 | SYS-1, SYS-3 | accept | runnable | leg-A mechanical migration; verdict unchanged |
| clm1-trap-runtime-violation | 1 | 0 | CLM-1, DIAG-3, SCOPE-4 | trap | runnable | leg-A migration; verdict preserved (trap-record bytes change; refutation-review the trapping predicate) |
| clm3-neg-body-check-bounds | 1 | 0 | OP-4,CLM-3,ENT-3 | reject | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| clm3-neg-body-check-requires | 1 | 1 | FN-8,CLM-3,ENT-3 | reject | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| clm3-neg-generated-wrapper-check | 0 | 1 | FN-8,PROG-3,CLM-3,ENT-4 | reject | runnable | keep-unchanged: contract finals only |
| clm3-neg-transitive-check-summary | 1 | 0 | OP-4,CLM-3,ENT-3 | reject | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| clm3-pos-transitive-value-branch | 1 | 3 | CLM-3,FN-2,FN-6,FN-8,FN-9,GIVE-1,ENT-5,PROG-3 | run | runnable | subject: CLM-3 strict case; a body check in a demanded closure becomes a rejecting claim - re-author (owner decision) |
| const1-pos-forwarded-arithmetic | 2 | 0 | CONST-1, FN-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| const2-pos-array-lookup | 2 | 0 | CONST-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| const2-pos-struct-const | 2 | 0 | CONST-2, TYPE-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| eff1-neg-wrong-order-row | 1 | 0 | EFF-1 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| eff1-pos-pure-and-traps-rows | 1 | 0 | EFF-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| eff2-neg-declared-unexhibited | 1 | 0 | EFF-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| eff2-neg-undeclared-exhibited | 1 | 0 | EFF-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| eff2-pos-declared-traps-exhibited | 1 | 0 | EFF-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| eff3-pos-pure-fn | 1 | 0 | EFF-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| eff4-pos-trap-aborts | 1 | 0 | EFF-4 | trap | runnable | leg-A migration; verdict preserved (trap-record bytes change; refutation-review the trapping predicate) |
| ent2-neg-expired-spelling-inherits-nothing | 1 | 0 | ENT-2, ENT-5, OP-4 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| ent2-neg-no-fact-across-call | 1 | 0 | ENT-2, OP-4, ENT-6 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| ent3-neg-stage8b-local-one | 0 | 1 | ENT-3,FN-8 | reject | runnable | keep-unchanged: contract finals only |
| ent3-pos-band-check-decomposition | 2 | 0 | ENT-3, OP-4, ENT-6 | run | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| ent3-pos-bor-guard-decomposition | 1 | 0 | ENT-3, OP-4, ENT-6 | run | runnable | leg-A mechanical migration; verdict unchanged |
| ent3-pos-s11-counted-range-run | 1 | 0 | GRAM-4, GRAM-6, FN-1, ENT-3, OP-4 | run | runnable | leg-A mechanical migration; verdict unchanged |
| ent3-pos-s4-requires-fact | 0 | 1 | ENT-3, ENT-2, FN-8, OP-4 | accept | runnable | keep-unchanged: contract finals only |
| ent3-pos-stage8b-bit-sources | 0 | 2 | ENT-3,FN-8 | accept | runnable | keep-unchanged: contract finals only |
| ent4-neg-nonstrict-bound-underivable | 1 | 0 | ENT-4, OP-4, ENT-6 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| ent4-pos-contradictory-state-discharges | 2 | 0 | ENT-4, OP-4 | accept | runnable | leg-A mechanical migration; verdict unchanged |
| ent4-pos-disequality-strengthens | 2 | 0 | ENT-4, OP-4 | accept | runnable | leg-A mechanical migration; verdict unchanged |
| ent4-pos-transitivity-discharges | 2 | 0 | ENT-4, OP-4 | accept | runnable | leg-A mechanical migration; verdict unchanged |
| ent5-neg-join-takes-weakest-bound | 2 | 0 | ENT-5, OP-4, ENT-6 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| ent5-neg-kill-on-write | 1 | 0 | ENT-5, SET-1, OP-4 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| ent5-neg-loop-rule-drops-preloop-fact | 1 | 0 | ENT-5, OP-4, ENT-6 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| ent5-neg-value-match-no-delivery | 2 | 1 | ENT-5,FN-8 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| ent5-pos-join-keeps-common-bound | 2 | 0 | ENT-5, OP-4 | accept | runnable | leg-A mechanical migration; verdict unchanged |
| ent5-pos-return-does-not-kill-loop-head-fact | 1 | 0 | ENT-5, OP-4 | accept | runnable | leg-A mechanical migration; verdict unchanged |
| ent5-pos-value-if-delivery-join | 4 | 1 | ENT-5,FN-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| err1-pos-result-value-match | 1 | 0 | ERR-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| err2-neg-missing-variant | 1 | 0 | ERR-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| err2-pos-exhaustive-match | 1 | 0 | ERR-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| err3-pos-propagation | 3 | 0 | ERR-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| err4-pos-contract-trap | 1 | 0 | ERR-4 | trap | runnable | leg-A migration; verdict preserved (trap-record bytes change; refutation-review the trapping predicate) |
| err4-pos-recoverable-value | 1 | 0 | ERR-4, ERR-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| ex1-pos-worked-example | 1 | 0 | EX-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn1-pos-returned-slice-const-run | 1 | 0 | FN-1,OWN-5,OWN-10,CONST-2,OP-4 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn1-pos-returned-slice-inputs-run | 2 | 0 | FN-1,OWN-5,OWN-7,OP-4 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn1-pos-signature-driven-call | 1 | 0 | FN-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn3-neg-requires-member | 0 | 1 | FN-3, FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn3-neg-signature-effect-mismatch | 1 | 0 | FN-1, FN-3 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| fn5-pos-match-dispatch | 1 | 0 | FN-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn6-pos-recursion | 1 | 0 | FN-6 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn7-pos-single-main | 1 | 0 | FN-7 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn8-neg-requires-control | 0 | 1 | FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-eeq-integer | 0 | 1 | FN-8,OP-1 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-local-in-body | 0 | 1 | FN-8, TYPE-5 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-missing-traps | 1 | 1 | FN-8, EFF-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| fn8-neg-requires-move-operand | 0 | 1 | FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-non-bool-check | 0 | 1 | FN-8, OP-5 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-noncopy-cvt-local | 0 | 1 | FN-8,TYPE-5,OP-6 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-noncopy-local | 0 | 1 | FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-set | 0 | 1 | FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-trapping-op | 0 | 1 | FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn8-neg-requires-user-call | 0 | 1 | FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn8-pos-requires-eeq | 0 | 1 | FN-8,OP-1,OP-8 | run | runnable | keep-unchanged: contract finals only |
| fn8-pos-requires-name-reuse | 1 | 1 | FN-8, TYPE-6 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn8-pos-requires-run | 1 | 1 | FN-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn8-trap-requires-false | 0 | 1 | FN-8, SCOPE-4 | trap | runnable | keep-unchanged: contract finals only |
| fn9-neg-entry-image-kill | 0 | 1 | FN-9,ENT-5 | reject | runnable | keep-unchanged: contract finals only |
| fn9-neg-named-outcome-no-publication | 0 | 2 | FN-9,ENT-3,FN-8 | reject | runnable | keep-unchanged: contract finals only |
| fn9-neg-no-selected-normal-exit | 0 | 1 | FN-9 | reject | runnable | keep-unchanged: contract finals only |
| fn9-neg-same-scc-summary | 0 | 3 | FN-9,ENT-6 | reject | runnable | keep-unchanged: contract finals only |
| fn9-neg-unproved-selected-return | 0 | 1 | FN-9,ENT-4 | reject | runnable | keep-unchanged: contract finals only |
| fn9-pos-direct-set-receiver | 0 | 2 | FN-9,ENT-3,ENT-5,ENT-6 | accept | runnable | keep-unchanged: contract finals only |
| fn9-pos-ok-selected-receiver | 1 | 2 | FN-9,ENT-3,ENT-5,ENT-6 | run | runnable | leg-A mechanical migration; verdict unchanged |
| fn9-pos-plain-direct-result | 1 | 2 | FN-9,ENT-3,ENT-6 | run | runnable | leg-A mechanical migration; verdict unchanged |
| form3-pos-lexical-classes | 1 | 0 | FORM-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram1-pos-lookahead | 1 | 0 | GRAM-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram10-pos-named-binders | 1 | 0 | GRAM-10 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram11-pos-named-args | 1 | 0 | GRAM-11 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram4-pos-stmts | 1 | 0 | GRAM-4 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram5-pos-exprs-places | 1 | 0 | GRAM-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram5-pos-recursive-place-projection | 1 | 0 | GRAM-5, TYPE-7 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram6-pos-no-operators | 1 | 0 | GRAM-6 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram7-pos-two-productions | 1 | 0 | GRAM-7 | run | runnable | leg-A mechanical migration; verdict unchanged |
| gram9-neg-nested-call | 1 | 0 | GRAM-9 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| gram9-pos-three-address | 1 | 0 | GRAM-9 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op1-neg-written-argument-on-deargumented-row | 1 | 0 | OP-1,TYPE-5 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| op1-pos-bool-enum-equality | 8 | 0 | OP-1,OP-7,OP-8,PRE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op1-pos-table-op | 1 | 0 | OP-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op1-pos-tag-enum-equality | 20 | 0 | OP-1,OP-7,OP-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op2-pos-ineg-modes | 3 | 0 | OP-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op2-pos-sat-mode | 1 | 0 | OP-2, OP-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op2-pos-wrap-untouched-by-dissolution | 1 | 0 | OP-2, OP-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op5-neg-check-fails | 1 | 0 | OP-5 | trap | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| op5-pos-check-pass | 1 | 0 | OP-5 | run | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| op7-pos-name-convention | 1 | 0 | OP-7 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op8-pos-integer-family | 1 | 0 | OP-1, OP-2, OP-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| op8-pos-u64-shift-u32 | 1 | 0 | OP-1, OP-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own1-neg-index-atom-after-move | 1 | 0 | OWN-1 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| own1-pos-match-projected-copy | 2 | 0 | OWN-1, OWN-13, OP-4 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own10-pos-local-region | 1 | 0 | OWN-10 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own11-pos-loop-inner-region | 1 | 0 | OWN-11 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own13-pos-let-match-give | 1 | 0 | OWN-13 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own2-pos-three-modes | 2 | 0 | OWN-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own3-pos-outlives-store | 1 | 0 | OWN-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own5-pos-read-through-holder | 2 | 0 | OWN-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own6-pos-callresult-borrow-chain | 1 | 0 | OWN-6, OWN-5, OWN-14 | run | runnable | leg-A mechanical migration; verdict unchanged |
| own7-pos-distinct-noverlap | 2 | 0 | OWN-7 | run | runnable | leg-A mechanical migration; verdict unchanged |
| pre1-pos-prelude-enums | 1 | 0 | PRE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| prog1-pos-closed-unit | 1 | 0 | PROG-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| prv2-neg-complete-only-postcondition | 1 | 2 | FN-9,ENT-3,PRV-1,PRV-2,FN-8,OP-4,SYS-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| prv2-neg-entry-system-result-bridge | 1 | 1 | PRV-1, PRV-2, FN-8, ENT-3, OP-4, SYS-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| prv2-neg-nonexact-goal | 1 | 1 | PRV-2, FN-8, ENT-3, OP-4 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| prv2-neg-two-hop-bridge | 1 | 3 | PRV-1, PRV-2, FN-8, ENT-6, OP-4, SYS-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| prv2-pos-allocation-equality-call | 0 | 1 | PRV-1, PRV-2, FN-8, ENT-3, OP-4, OP-9 | accept | runnable | keep-unchanged: contract finals only |
| prv2-pos-postcondition-b-summary | 0 | 2 | FN-9,ENT-3,PRV-1,PRV-2,FN-8,OP-4,SYS-2 | accept | runnable | keep-unchanged: contract finals only |
| prv2-pos-seedless-mutual | 1 | 2 | PRV-1, PRV-2, FN-6, FN-8, SYS-2 | accept | runnable | leg-A mechanical migration; verdict unchanged |
| prv3-neg-external-check | 1 | 0 | PRV-1, PRV-3, ENT-3, OP-4, SYS-2 | reject | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| run-ex1-value-match | 2 | 0 | GRAM-10, GRAM-11, GIVE-1, ERR-1, OP-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| run-ex2-loop-trap-folds | 2 | 0 | GRAM-4, OP-1, OP-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| scope3-pos-defined-run | 1 | 0 | SCOPE-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| scope4-pos-check-traps | 1 | 0 | SCOPE-4 | trap | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| set1-pos-local-and-field-copy | 4 | 0 | SET-1,TYPE-5,STOR-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| set2-pos-affine-field-replace | 2 | 0 | SET-2, TYPE-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| stor1-pos-frame-resident | 1 | 0 | STOR-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| stor2-pos-box-new | 1 | 0 | STOR-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| stor3-pos-box-drop-region | 1 | 0 | STOR-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| stor4-pos-arena-confined | 1 | 0 | STOR-4 | run | runnable | leg-A mechanical migration; verdict unchanged |
| trap-op5-check-fails | 1 | 0 | OP-5, SCOPE-4 | trap | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| type1-pos-i32-unit | 1 | 0 | TYPE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| type2-pos-enum | 1 | 0 | TYPE-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| type2-pos-twostate-enum-i1 | 1 | 0 | TYPE-2, OWN-1, PRE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| type6-pos-distinct-names | 1 | 0 | TYPE-6 | run | runnable | leg-A mechanical migration; verdict unchanged |
| type7-pos-deref | 1 | 0 | TYPE-7 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-arith-check-catches-wrapped-overflow-traps | 1 | 0 | OP-5, SCOPE-4 | trap | runnable | subject: OP-5/check is (part of) the tested surface; re-author as CLM analogue or retire (owner decision) |
| x-arith-iadd-checked-overflow-err-arm-runs | 1 | 0 | OP-1, OP-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-arith-iadd-wrap-overflow-to-negative | 1 | 0 | OP-1, OP-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-arith-idiv-trap-zero-divisor-traps | 1 | 0 | OP-2, SCOPE-4 | trap | runnable | leg-A migration; verdict preserved (trap-record bytes change; refutation-review the trapping predicate) |
| x-arith-isub-wrap-min-roundtrip-runs | 1 | 0 | OP-1, OP-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-arith-loop-imul-trap-folds-overflow-traps | 1 | 0 | OP-1, SCOPE-4 | trap | runnable | leg-A migration; verdict preserved (trap-record bytes change; refutation-review the trapping predicate) |
| x-array-const-checksum-run | 1 | 0 | CONST-1,CONST-2,TYPE-2,OP-1,OP-4,SET-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-array-mutable-checksum-run | 1 | 0 | TYPE-2,OP-1,OP-4,SET-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-base64-rfc-vectors-run | 15 | 1 | CONST-1,CONST-2,FN-8,OP-5,OP-6,OP-1,OP-4,OP-9,OWN-1,OWN-3,OWN-4,OWN-5,OWN-10,OWN-12,TYPE-7,EFF-2,SET-1,STOR-3 | run | runnable | leg-A mechanical migration; verdict unchanged (check is scaffolding; where the manifest rules list OP-5, that row updates to CLM-1) |
| x-borrow-two-shared-reads-run | 1 | 0 | OWN-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-borrowed-pool-tree-run | 2 | 0 | TYPE-2,OWN-1,OWN-3,OWN-4,OWN-5,OWN-7,OWN-10,OWN-12,TYPE-7,EFF-2,SET-1,OP-1,OP-4,OP-9,STOR-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-buffer-borrowed-columns-run | 1 | 0 | OWN-3,OWN-4,OWN-5,OWN-7,OWN-10,OWN-12,TYPE-7,EFF-2,SET-1,OP-4 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-buffer-mutable-checksum-run | 1 | 0 | TYPE-2,OP-1,OP-4,OP-9,SET-1,STOR-3,EFF-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-child-reborrow-run | 3 | 0 | OWN-3,OWN-4,OWN-5,OWN-6,OWN-7,OWN-9,OWN-10,OWN-12,TYPE-7,EFF-2,SET-1,OP-4,OP-9,STOR-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-const-scalar-u64-width | 1 | 0 | CONST-2, TYPE-5, FORM-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-crc32-standard-vector-run | 1 | 0 | OP-6,OP-1,OP-4,OP-9,SET-1,STOR-3,EFF-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-eff-dup-reads-effect | 1 | 0 | EFF-1 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-eff-pure-combined-with-traps | 2 | 0 | EFF-1 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-eff-pure-fn-calls-traps-fn | 2 | 0 | EFF-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-eff-trailing-comma-row | 2 | 0 | EFF-1 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-eff-traps-fn-calls-only-pure | 1 | 0 | EFF-2 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-eff-writes-missing-region | 1 | 0 | EFF-1 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-enum-borrow-payload-live | 1 | 0 | OWN-13, GIVE-1, GRAM-10, OP-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-enum-multiwidth-dispatch | 1 | 0 | TYPE-2, TYPE-6, GRAM-10 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-enum-payload-give | 1 | 0 | TYPE-2, GRAM-8, GRAM-10, GIVE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-enum-stmt-payload-check | 2 | 0 | TYPE-2, GRAM-10, OWN-13, OP-1 | run | runnable | leg-A mechanical migration; verdict unchanged (check is scaffolding; where the manifest rules list OP-5, that row updates to CLM-1) |
| x-enum-twostate-result-payload | 6 | 0 | TYPE-2, ERR-1, PRE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-fn-cross-fn-call-chain | 1 | 0 | FN-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-fn-mutual-recursion-runs | 1 | 0 | FN-1, FN-6 | run | runnable | leg-A mechanical migration; verdict unchanged (check is scaffolding; where the manifest rules list OP-5, that row updates to CLM-1) |
| x-give-result-aggregate | 4 | 0 | GIVE-1, ERR-1, TYPE-5, PRE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-gram-combo-flat-call-construct-match | 1 | 0 | GRAM-9, GRAM-11, GRAM-8, GRAM-10 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-integ-checked-overflow-diverts-to-err | 1 | 0 | OP-1, GIVE-1, OWN-13, SCOPE-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-integ-coin-borrow-match-score-twice | 1 | 0 | OWN-13, GIVE-1, OP-1, SCOPE-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-integ-loop-product-overflow-traps | 1 | 0 | OP-1, GIVE-1, SCOPE-3 | trap | runnable | leg-A migration; verdict preserved (trap-record bytes change; refutation-review the trapping predicate) |
| x-integ-sign-weight-accumulate | 1 | 0 | OP-1, OWN-13, SCOPE-3, FN-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-integ-traffic-light-state-machine | 1 | 0 | OWN-13, GIVE-1, OP-1, SCOPE-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-match-err2-value-match-missing-err | 1 | 0 | ERR-2, PRE-1 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-match-give1-nested-value-match | 1 | 0 | GIVE-1, PRE-1, ERR-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-match-gram10-binder-not-fresh | 1 | 0 | GRAM-10 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-match-gram10-out-of-order-fields | 1 | 0 | GRAM-10 | reject | runnable | leg-A mechanical migration; verdict unchanged |
| x-nominal-bool-ops-run | 4 | 0 | OP-1,OP-8,PRE-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-nominal-multifield-payload-run | 4 | 0 | TYPE-2,GRAM-8,GRAM-10,OWN-13,OP-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-option-byte-scanner-run | 3 | 0 | PRE-1,ERR-1,GRAM-8,GRAM-10,OWN-13,OWN-12,TYPE-7,EFF-2,OP-4,SET-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-ownmove-copy-reused-affine-consumed-once | 2 | 0 | OWN-1 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-ownmove-owned-temporary-scrutinee | 2 | 0 | OWN-13 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-requires-output-capacity-run | 2 | 1 | FN-8,OP-5,EFF-2,OWN-12,TYPE-7,OP-4,SET-1 | run | runnable | leg-A mechanical migration; verdict unchanged (check is scaffolding; where the manifest rules list OP-5, that row updates to CLM-1) |
| x-result-buffer-transform-run | 5 | 0 | PRE-1,ERR-1,GRAM-8,GRAM-10,OWN-1,OWN-13,OP-4,OP-9,SET-1,STOR-3,EFF-2 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-struct-construct-read-field | 1 | 0 | TYPE-2, GRAM-8, TYPE-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-struct-cross-fn | 1 | 0 | TYPE-5, GRAM-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-struct-mixed-width | 3 | 0 | TYPE-2, TYPE-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-struct-nested-field | 1 | 0 | TYPE-2, TYPE-5 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-struct-of-buffers-checksum-run | 1 | 0 | TYPE-2,OWN-1,SET-1,OP-4,OP-9,STOR-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-struct-set-field | 1 | 0 | TYPE-5, GRAM-8 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-typ-uniq-deref-write-roundtrip | 1 | 0 | TYPE-7 | run | runnable | leg-A mechanical migration; verdict unchanged |
| x-wc-chunk-summary-run | 8 | 0 | TYPE-2,OWN-1,OWN-3,OWN-4,OWN-5,OWN-7,OWN-10,OWN-12,TYPE-7,EFF-2,SET-1,OP-1,OP-4,OP-9,STOR-3 | run | runnable | leg-A mechanical migration; verdict unchanged |
