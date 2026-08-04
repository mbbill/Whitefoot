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

## 2026-07-23 — approval
- owner: approved in session
- reason: Correct five protected conformance discrepancies without changing v0.14. Add the `allocates(heap)` effect exhibited by `buffer_new` to `pending-op9-buffer-new` and `op4-trap-index-oob`; make `type2-pos-buffer-tagonly` test the legal `buffer<Bool>` type and its affine transfer without calling the primitive-only `buffer_new` row; retain projected-copy coverage in `own1-pos-match-projected-copy` while using a primitive buffer element admitted by that row; and change `fn8-neg-requires-non-bool-check` from TYPE-5 to the OP-5 rejection explicitly selected by FN-8. The five statuses and expectation kinds remain unchanged; only the last case's expected rule changes.
- boundary: corrected source SHA-256 values are `pending-op9-buffer-new` `989ceff1b6ac87351a0d099ea70e55b9f25c0243b704402257c2af988269d2f5`, `op4-trap-index-oob` `d836c74fb6213fb836e1a601d88d514f14aeab3a077a0a1f998b8766003f03c5`, `type2-pos-buffer-tagonly` `a2eff4ad6383e31c2d215564432c9536472543460fcdd0d955d592b51daaf4ac`, and `own1-pos-match-projected-copy` `dff17036062faaeaada0a32a5dfe4136bbb48e50f6ad35673a094b12e58ed9b9`; unchanged `fn8-neg-requires-non-bool-check` source is `0e45fdbc844a122f615384a79f9ff08d02a6bc7aa3e05945344ee0a80c4fcb90`; resulting `tests/conformance/manifest.jsonl` is `d8a0730244e75c4cc2508b8029130deb37e68f6e957e9477e4c9eeab4957b83f`.
- evidence: owner instruction “1 approved” immediately following presentation of the exact five-case correction boundary; `make conformance` covers all 93 active v0.14 rules after the correction.

## 2026-07-23 — approval
- owner: approved in session
- reason: Approve exact v0.15 candidate SHA-256 `3c924095b2c21f123b7137556f72dbe87275838682c1965e6caf399dd24d13bd` for append-only activation and synchronized compiler, conformance, and live-document updates. The change removes the undefined language-level array frame limit and adds STOR-6: ordinary facts-off target materializations, complete generated target objects, runtime allocator arguments, indices, and scaled offsets must be exactly representable under the selected target without narrowing, storage-class changes, or optimizer dependence. Static target-layout failure and dynamic target-domain failure remain non-language failures; source rejection, language effects, OP-4 and OP-9 traps, DIAG-3 records, and `array_new` purity are unchanged. No existing conformance source, verdict, expectation kind, or runnable status changes.
- evidence: `governance/spec-evolution/kernel-spec-v0.15-candidate.md`; native grammar verifier passed 62 productions, 72 decisions, and 72 terminal predicates; exact rule-body audit found eight modified rules plus STOR-6; three hostile reviewers passed the exact bytes; owner instruction “approved” immediately following presentation of the exact hash and no-protected-change boundary.

## 2026-07-23 — approval
- owner: approved in session
- reason: Approve exact v0.16 candidate SHA-256 `f93264fb4df6994a408e1327c6e8643c34b4aea55fba4b1a0b3dab3501ceb942` for append-only activation and synchronized compiler, additive conformance, derivation, live-document, and design-memory updates. The change closes the nongeneric static source-contract family with complete declared-order bindings, exact concrete type and callable-signature equality, normalized exact effect-capability equality after region alpha-renaming, coherent concrete conformance keys, and mandatory checked-law discharge. Source-contract generic bounds, member calls, dispatch objects, runtime or ABI representations, effect subtyping, and optimizer authority remain absent. No existing conformance source, verdict, expectation kind, or runnable status changes.
- evidence: `governance/spec-evolution/kernel-spec-v0.16-candidate.md`; native grammar verifier passed 62 productions, 72 decisions, and 72 terminal predicates; exact rule-body audit found only TYPE-6, STOR-1, FN-2, FN-3, FN-4, FN-5, DIAG-1, DIAG-2, and DIAG-3 changed; three hostile reviewers passed the exact bytes after resolving circular generic-bound admission, concrete type identity, resolver/semantic diagnostic order, and semantic effect normalization; owner instruction “approved” immediately following presentation of the exact hash and no-protected-change boundary.

## 2026-07-23 — approval
- owner: approved in session
- reason: Correct the protected `stor2-pos-box-new` and `stor3-pos-box-drop-region` sources to declare the `allocates(heap)` effect exhibited by `box_new`, then strengthen both statuses from `pending` to `runnable` now that the general box allocation, dereference, and compiler-derived cleanup path is implemented. Their `run` exit-0 expectations, rule assignments, and documentation remain unchanged; no specification or other conformance case changes.
- boundary: corrected source SHA-256 values are `stor2-pos-box-new` `f09bcf48071bd14d8f835f8fd40bac83aed1df1ee8b9d6d611d58774663194ff` and `stor3-pos-box-drop-region` `4ae736aa0c0fa9d6b6cf749c86b1eaa3d279f272df84b9ebfb24a37c83d8bcc4`; resulting `tests/conformance/manifest.jsonl` is `73fe9696c20bbd4bc7bbbcf494e66114fc629a54e89acb42aa60ec121f8dbe56`.
- evidence: compiler implementation commit `8abfd54`; both corrected cases compiled through the public `whitefootc` command and executed with exit 0 before approval; owner instruction “approved” immediately following presentation of this exact protected boundary.

## 2026-07-23 — approval
- owner: approved in session
- reason: Approve exact v0.17 candidate SHA-256 `19642ffb0ad9c7146a84762ada192ed2a25dc446a93c4d060aa29d9a99f69c93` for append-only activation and synchronized compiler, additive conformance, derivation, live-document, and design-memory updates. Direct `own slice<'r, T>` results gain signature-carried finite storage-origin ceilings and call-site substitution so alias and effect provenance survives calls without body inspection. Region-bearing function and nominal generic arguments become FN-2 source rejections; region-bearing `box` and `arena` content becomes a STOR-5 source rejection; borrow-mode direct-slice function and contract-member results become FN-1 source rejections. Direct nested-slice type formation remains valid but unsupported, and lifetime, runtime representation, ABI, traps, and required runtime safety checks remain unchanged. No existing conformance source, verdict, expectation kind, or runnable status changes.
- evidence: `governance/spec-evolution/kernel-spec-v0.17-candidate.md`; native grammar verifier passed 62 productions, 72 decisions, and 72 terminal predicates; exact rule-body audit found only TYPE-6, OWN-5, OWN-7, STOR-1, STOR-5, FN-1, FN-2, FN-3, FN-4, FN-5, EFF-2, DIAG-2, and DIAG-3 changed; multiple hostile-review rounds rejected and repaired fresh-arena origins, undefined joins and effect substitution, borrowed-descriptor result ambiguity, contract-member coverage, and an accidental nested-slice rejection before two independent reviewers passed the exact final bytes; owner instruction “approved” immediately following presentation of the exact hash and no-protected-change boundary.

## 2026-07-23 — approval
- owner: approved in session
- reason: Delete the obsolete protected STOR-5 `grammar-invariant` coverage annotation. In v0.17, region-bearing stored content is writer-emittable and receives real STOR-5 program judgments, including the new `box_new` and `arena_new` rejection cases; retaining a non-program annotation would be factually false and redundant. No conformance source, verdict, expectation kind, status, rule assignment, or program-case documentation changes.
- boundary: only the single STOR-5 coverage-annotation record is deleted; resulting `tests/conformance/manifest.jsonl` SHA-256 is `4a8cbc40a82732ae3df7b7ee4e0c3d52471d15f69af9994fcdbe411730e5d55a`.
- evidence: owner instruction “approved” immediately following presentation of the exact metadata-only deletion, unchanged protected-case boundary, and resulting manifest hash.

## 2026-07-27 — approval
- owner: approved in session
- reason: Approve the 2026-07-27 roadmap reorientation and the W1 constitution amendment. `docs/roadmap.md` fixes the finite Phase 10 completion set as the sole Phase 11 entry gate, adds Phase 11 (declared parallelism) superseding and deleting the non-authoritative untracked `docs/parallelism.md` note per its own disposition clause, renumbers optional hardening to Phase 12, and defers the headline artifact ladder, the take/replace-versus-sealed-kernel storage decision, contract member calls (P5), and the constant-time `secret` effect with unchanged re-entry bars. `docs/constitution.md` rewords W1 from low-capability-model writability to floor robustness (default shape is optimal shape: an accepted program has been forced onto a fast shape, and the writer's only alternative is a program that does not compile) and updates R0's third leg accordingly. No specification byte, conformance source, verdict, expectation kind, or runnable status changes.
- boundary: amended `docs/constitution.md` SHA-256 `4dfb64f622f7cf037c404e74cd5c1c17ea95f74425a1457db17ee2d67087b3a2`; reoriented `docs/roadmap.md` SHA-256 `b23d47ba61de9d32eb49d9f5d83d92e165793ede0e1538f12eba7525d29fa37a`.
- evidence: owner instructions in session 2026-07-27 — "we should drop w1, or reframe it around default shape is optimal"; "what we can do right now is to finish things that's unlikely to affect this. then, we move on to research and implement parallelism"; and the explicit go for the amendment and commit — each following presentation of the exact plan and change summary; full repository gate green after the change.

## 2026-08-01 — approval
- owner: approved in session
- reason: Approve the live authority placement of the already-selected D17 representation-invariant proof lane. `docs/constitution.md` records the long-term checked lane: an exact project or user implementation gains only the narrowly scoped representation privileges whose complete invariants and obligations a deterministic machine checker verifies, while an unproved project kernel, if a future active specification admits one, remains an explicit trusted boundary. `docs/roadmap.md` separates that principle from D16's deferred sealed-catalog/storage mechanism and records that v0.17 has no representation-invariant proof language, checker, partial-initialization path, privileged operation, or authorized implementation work. The exact proof mechanism, concrete operations and semantics within the selected privilege classes, any additional privilege class, re-entry work, and schedule remain deferred. This reconciles live authority with the existing D17 ruling; it does not re-decide that durable design. No specification byte, compiler behavior, conformance source, verdict, expectation kind, runnable status, or current implementation authorization changes.
- boundary: amended `docs/constitution.md` SHA-256 `c316d968b7c8912e750b57c963d946019846138226ac7b7de81618d632bbb519`; placed `docs/roadmap.md` SHA-256 `9667974a4b6e0ff43d036c503757c42fc4e60e8abf7d545f9f58a6e6d7bbbcd1`.
- evidence: owner instruction “ok” on 2026-08-01 after presentation of the exact boundary: move D17's long-term principle into current project law, record its deferred mechanism and re-entry boundary in the roadmap, and do not start proof implementation.

## 2026-08-01 — approval
- owner: approved in session
- reason: Promote two retained archive-audit findings as future problem constraints, not inherited solutions. Phase 11 construct design must exercise a runtime-count dynamic fan-out witness whose workers share-read outer state, either supporting it soundly or deliberately rejecting it; the archived fixed-spawn plus `par.for_chunks` answer carries no preference. Reopening the deferred take/replace-versus-sealed-kernel storage decision must account for six independent dimensions: growable dense storage and affine backing replacement; move-out, failure, cleanup, and destruction ordering; partial initialization and sparse occupancy; stable versus recyclable identity; invalidation, stale-handle detection, and check-elision authority; and multi-place access, iteration, and relocation under active loans. The non-authoritative audit supplies provenance only. This changes no Phase 11 entry or sequence, selects no construct, runtime, storage mechanism, or OWN-11 carve-out, and authorizes no current specification or compiler work. No specification byte, compiler behavior, conformance source, verdict, expectation kind, or runnable status changes.
- boundary: promoted `docs/roadmap.md` SHA-256 `85c91653c82978869adbe7479782aca72bfdd7a2a5b7f1aea6ce4cf5d98a607b`; synchronized `research/archive-promotion-audit.md` SHA-256 `b272eccbc87386fed980cf5fdc057db0a69f2bb4af869cc1c74bf875bcdd820a`.
- evidence: owner instruction “ok” on 2026-08-01 after presentation of the two exact promotions: add dynamic fan-out to the Phase 11 required cases and make the deferred storage decision carry the retained operation-level checklist, without restoring archived mechanisms or implementation work.

## 2026-08-03 — approval
- owner: approved in session
- reason: Replace the phase checklist and experiment-first execution order with a living Direction Outline, one status-bearing Current Plan, project-derived milestones, and explicitly approved bounded parallel research. The outline records directions, progress, dependencies, evidence, and candidate-project mappings but never schedules work. `PROPOSED` plans authorize nothing; only an `ACTIVE` plan or separate owner approval authorizes execution. Candidate-project pressure determines when an outline item matters, while specification gaps continue through the guarded append-only language-change branch. This supersedes the 2026-07-27 Phase 10 completion set as the sole Phase 11 entry gate and the older phase-order rulings only as current sequencing; it does not amend W1, D17, the active specification, protected evidence, or their historical approval records.
- boundary: `docs/roadmap.md` SHA-256 `616ef7109e81e375c1f7da47c0ec3b3f3eb6a972f325b513157d5083022ff8ee`; `docs/current-plan.md` SHA-256 `861b6772a9feee4a504007199f5e5c4d4b66e84fcd28e0a1c910f9cd4e4cb4e3`; `WORKFLOW.md` SHA-256 `2180430e86d8bb43eaf12ff5e755d1c6f4ec6fa362da2db675e36fd8fa36816c`.
- evidence: owner instructions in session 2026-08-03 selected the model in which the outline decides what exists and is known, candidate projects decide when it matters, the Current Plan defines the next executable slice, and independent research may fill named gaps; the owner then confirmed that the model looked good and instructed the repository to be organized accordingly. No specification byte, compiler behavior, conformance source, verdict, expectation kind, or runnable status changes.
