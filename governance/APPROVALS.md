# Protected approval record

This file contains the append-only approval record. It is not a workflow guide;
the sole live governance procedure is [`WORKFLOW.md`](../WORKFLOW.md). Entries through
the hash-baseline era remain unchanged historical records, followed by the
newer exact-artifact and protected-change approvals.

## 2026-07-18 — approval
- owner: approved in session
- reason: Governance baseline: establish the spec/test guard at the committed main state (commit c18013b); no guarded content changed. Owner-directed governance lockdown 2026-07-18.
- baseline: 0e876fd68b1da613de96364ba1d5ce33ccebe7c3ea508b0ad0d2dc06f9709749

## 2026-07-18 — approval
- owner: approved in session
- reason: Owner-approved bounded-reborrow relaxation v0.6->v0.7 (statement-scoped child reborrow; OWN-5/6/9/12 + new STOR-5; PATTERNS P4). Approved in session 2026-07-18 after the recorded investigation at optimizer-language-research/implementation/reborrow-investigation/ (DOSSIER; model-check RESULTS 1M programs/0 violations; FR reconciliation; fact-channel review PASS-WITH-CONDITIONS; PACKET; V0.7-DELTA-DRAFT). Guarded changes: new spec kernel-spec-v0.7.md; three conformance META annotations re-versioned to v0.7. No test or oracle weakened.
- baseline: 48cc850aabdbd730792f2f973d85f13896cbea01e3eb2f387963e9f3d29b1db6

## 2026-07-18 — approval
- owner: approved in session
- reason: Owner-approved region-retention checkpoint (THE-PLAN Phase-2 next step, salvaged from parked_edits, 2026-07-18): fix conformance case x-typ-value-where-borrow-param.wf malformed region param [r]->['r] + explicit region arg (same TYPE-5 reject verdict; required now that stage-0 democ correctly enforces FORM-3 region spelling). Also pins the additive test_codegen.py region-arg tests. No expected verdict weakened; the case still rejects at TYPE-5.
- baseline: 48843fae8d276b06bd3c5f61e5e08b1c440055ed42ad7b529b52a30410c359d8

## 2026-07-19 — approval
- owner: approved in session
- reason: Owner-approved v0.7->v0.8 tag-only enum equality: exact eeq/ene delta and guarded conformance-META updates in enum-equality-investigation/V0.8-DELTA-DRAFT.md
- baseline: 3222e16164d319ba4367368aeabe7396d4360d144e07fd68e819b3afc2f54630

## 2026-07-20 — approval
- owner: approved in session
- reason: Owner-approved retirement of prototype/democ/test_codegen.py with the archived democ implementation on 2026-07-20; test bytes preserved under archive/toolchains/self-hosting-2026-07-20/democ/test_codegen.py; kernel v0.8, conformance cases and verdicts, frozen oracle digests, and active prototype/checker/test_checker.py unchanged.
- baseline: 9d4ff925668a3341543d555c5243ef0b74ca5e7e275617ff4808d90c290dc48a

## 2026-07-21 — approval
- owner: approved in session
- reason: Owner-approved exact v0.9 installation packet at commit 7fbb018: install candidate SHA-256 bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68; approve protected syntax repair 724dbb970c8ce7ede7a52daf3ad2c9286b7872137e83f495fbf845df75252479; apply patches A 4b626ff44a9bc3cec96e41d9f3fa93b937a36397b7970b9310d39039cf8eb1f2, B 62916bfc1bcc9e4eaa0461c33015cb30a2abe113f3aebcc807a3b8c492c0d54a, and C ae48711659c881ab2e3ca4794641ffae948ed52a2e1bdf62f61da764c7be48a6 in order; final manifest 0eff27bfb87ca14086f31f4b171d72c9eb1a49072aa4563a3f7c937d0b8bb90c; append ledger amendment f29b326f446aa9e5f512d079f1dbd14e641e6d840f18b69faab0ea39950e52a0; switch the live target and create v0.9 evidence while preserving v0.8. No expected-verdict, runnable-status, frozen-oracle, or existing reference-test change. Investigation: grammar-verifier/proposal/DELTA.md and SUCCESSOR-HOSTILE-REVIEW.md.
- baseline: bb7ce5ea5b3b2a169b259bcffc7add3234e89b50aa689d5f9df5a93a91325622

## 2026-07-22 — approval
- owner: approved in session
- reason: restructure: co-located gate on new tests/ + governance/ layout
- baseline: 4ad22f40e7a0931a541a54b73f46c417da242efa2ca0d0a8cf60a1e40ee46b7d

## 2026-07-22 — ledger scope amendment

The hash-baseline guard described above was retired on 2026-07-22. Its entries
remain unchanged as the exact audit record for the period in which it operated.
Later approvals record the approved artifact or change boundary directly and do
not invent a retired baseline hash.

Owner rulings before this separate ledger began on 2026-07-18 are preserved in
`archive/governance/directives.md` and in the versioned records indexed by
`archive/governance/decision-log.md`. They are not duplicated here because many of them
authorized research or selected a direction without approving a protected
specification or test change.

## 2026-07-21 — approval
- owner: approved in session
- reason: Approve the exact Phase-5 successor proposal SHA-256 `7fc48cc30f94d25be5be1106e3265d92c1b0cdf2bfea5a7a17759a12f3cf092d` and generated v0.10 candidate SHA-256 `71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9`. The approval selected the reviewed language delta and architecture consequences but did not itself install v0.10 or authorize resolver implementation.
- evidence: `archive/governance/decisions/v0.9.md` (`PHASE5-SUCCESSOR-OWNER-APPROVAL`)

## 2026-07-22 — approval
- owner: approved in session
- reason: Correct the protected `fn7-neg-two-mains` evidence to v0.10 TYPE-6 duplicate-declaration behavior and authorize preparation, hostile review, and owner presentation of the bounded v0.11 semantic-closure candidate. This did not approve candidate activation before exact-byte review.
- evidence: `archive/governance/decisions/v0.10.md` (`V010-DUPLICATE-MAIN-CORRECTION-AND-V011-SEMANTIC-CLOSURE-AUTHORIZATION`)

## 2026-07-22 — approval
- owner: approved in session
- reason: Select `propagate` as the sole ERR-3 Result-forwarding spelling, with no `try` compatibility alias, and approve exact v0.11 candidate SHA-256 `050e110c8c5eb3143c9d3f54968a9df9125f1d4b5991f527b8a15938a4292fbc` for append-only activation with synchronized compiler, conformance, and reference-model updates.
- evidence: `archive/governance/decisions/v0.11.md` (`V011-PROPAGATE-SPELLING-CANDIDATE-FREEZE` and `V011-OWNER-APPROVED-ACTIVATION`)

## 2026-07-22 — approval
- owner: approved in session
- reason: Remove the stale `KNOWN GAP` wording from the protected `xfail-own1-bare-affine-use` conformance source now that the compiler implements its existing OWN-1 rejection. The program, manifest verdict, rule assignment, and reference expectation are unchanged.
- evidence: owner approval immediately following the nominal-data compiler slice at commit `58e9c43`

## 2026-07-22 — approval
- owner: approved in session
- reason: Repair the repo-wide consistency findings without changing a released specification, an existing conformance verdict, or a runnable status. Correct stale conformance explanations and coverage annotations to the active v0.11 architecture, archive the unmanifested duplicate const fixture, replace nonexistent-tool claims with native checks, repair live documentation and design-memory drift, and prepare—but do not activate—the existing v0.12 candidate with current governance references.
- evidence: owner instruction “I think we need to fix all the issues” following the complete repository consistency review

## 2026-07-22 — approval
- owner: approved in session
- reason: Correct five malformed protected expectations whose `accept` kind caused their nested runtime result to be ignored. `op2-pos-sat-mode`, `const2-pos-array-lookup`, `own1-pos-tagonly-copy`, `type2-pos-buffer-tagonly`, and `type2-pos-twostate-enum-i1` now require the already-documented successful execution result (`run`, exit 0). Their sources, rule assignments, runnable statuses, and language behavior are unchanged. The manifest validator now rejects fields that do not belong to the declared expectation kind.
- evidence: owner instruction “forget about 0.12. make sure everything else is good” following the consistency report that identified these exact five malformed expectations

## 2026-07-22 — approval
- owner: approved in session
- reason: Retire the active `tests/reference/` Python model and preserve it intact under `archive/tests/reference/`. The model consumes its own historical toy AST and does not exercise or compare with the Rust compiler, so it is removed from the active language-change workflow and repository gate. No released specification, compiler behavior, conformance source, verdict, status, or codegen policy changes.
- evidence: owner instruction “just move it to the archive and fix all the references”

## 2026-07-22 — approval
- owner: approved in session
- reason: Approve exact v0.12 candidate SHA-256 `e2d5566379891454c090e037bd45c5f1a8df90ba23506a0f83ce9aaa03b41463` for append-only activation. The change adds SET-1 copy-place assignment and synchronizes target ordering, storage, ownership, effects, diagnostics, and governance references. No existing conformance source, expected verdict, or runnable status is changed; the activation adds focused cases and switches live identities.
- evidence: `governance/spec-evolution/kernel-spec-v0.12-candidate.md`; native grammar verifier passed 62 productions, 72 decisions, and 72 terminals; exact rule-body audit found fifteen modified rules plus SET-1; two hostile reviews passed before the owner approved the exact hash.

## 2026-07-22 — approval
- owner: approved in session
- reason: Approve exact v0.13 candidate SHA-256 `ed93cc43a6a224725f813b1adfc4c19fbb64dc5ab294b25d924392d2959b77cd` for append-only activation. The direct `propagate` operand becomes a consuming context for a bare affine place rooted in a live own-mode binding, while explicit `move`, same-E checking, borrow restrictions, cleanup, effects, runtime behavior, and required checks remain unchanged. Also approve source-only repairs, with every existing verdict and runnable status unchanged: write required affine returns explicitly, correct unexhibited `traps` rows to `pure`, add missing minimal `main` declarations to the affected propagation cases, and give the three affected Result match fixtures fresh GRAM-10 binders.
- evidence: `governance/spec-evolution/kernel-spec-v0.13-candidate.md`; native grammar verifier passed 62 productions, 72 decisions, and 72 terminals; the exact rule-body audit found only TYPE-6, OWN-1, STOR-1, FN-4, ERR-3, DIAG-2, and DIAG-3 changed; owner instruction “approved” immediately following presentation of the exact hash and protected repair boundary.

## 2026-07-22 — approval
- owner: approved in session
- reason: Approve exact v0.14 candidate SHA-256 `31c09313363304f405c8db1191d1982e3625b86788bf953ec3bb169648466e9f` for append-only activation and synchronized compiler and conformance updates. The change closes the existing three-row `ineg` judgment: wrapping minimum remains minimum, trapping minimum follows OP-2 and the existing mandatory empty-message record, checked minimum returns `Err(Overflow())`, and every other input returns its exact mathematical negation. No existing conformance source, verdict, or runnable status changes; activation adds focused cases and switches live identities.
- evidence: `governance/spec-evolution/kernel-spec-v0.14-candidate.md`; native grammar verifier passed 62 productions, 72 decisions, and 72 terminals; the exact rule-body audit found only TYPE-6, STOR-1, OP-2, FN-4, DIAG-2, and DIAG-3 changed; owner instruction “approved” immediately following presentation of the exact hash and no-protected-change boundary.
