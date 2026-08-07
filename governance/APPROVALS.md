# Protected approval record

This file contains the append-only approval record. It is not a workflow guide;
the sole live governance procedure is [`docs/WORKFLOW.md`](../docs/WORKFLOW.md). Entries through
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

## 2026-08-03 — approval
- owner: approved in session
- reason: Move the sole operational guide from the repository root to `docs/WORKFLOW.md` and make its two workflows explicit. Ordinary project delivery remains the default for project adaptation, already-specified compiler work, defect repair, performance work, and bounded research. The separate specification-change workflow is entered only for a genuine language gap and retains the stronger candidate, evidence, exact-byte approval, synchronized activation, and project-return obligations. Three read-only agent dry runs exercised an unsupported specified capability, a guarded parallelism proposal, and separately approved candidate research; their boundary findings were incorporated without adding another tracker or execution lifecycle.
- boundary: `docs/WORKFLOW.md` SHA-256 `1fc8ff9f0621abfbb9a9124e6fa0fd34cee1423df909906ab6574bdbe6500e14`; reference-only `docs/roadmap.md` SHA-256 `9e403bae50a729179e2fb0802eab7887d87b7b6182ad4b7286c472586f4d278b`; unchanged `docs/current-plan.md` SHA-256 `861b6772a9feee4a504007199f5e5c4d4b66e84fcd28e0a1c910f9cd4e4cb4e3`. The root `WORKFLOW.md` is removed rather than retained as a second authority. Existing bare `WORKFLOW.md` mentions in the immutable active specification and protected conformance material identify this sole document by basename; no protected byte is changed.
- evidence: owner instruction in session 2026-08-03 — “帮我移动一下” — followed by the requirement that ordinary work and specification updates remain distinct and that agents dry-run both paths while the primary agent acts only as the human decision-maker. No specification byte, compiler behavior, conformance source, verdict, expectation kind, runnable status, project selection, or implementation authorization changes.

## 2026-08-05 — approval
- owner: approved in session
- reason: Exact-byte approval of REVIEW CANDIDATE v0.18 (BOUND-1 system-interface first command slice): 25 new rules (`PROG-3`, `EFF-5`, `SYS-1..13`, `HOST-1..3`, `PATH-1..2`, `QUAL-1..3`, `TRAP-1`, `GATE-2`), 13 modified (GRAM-2, GRAM-11, TYPE-2, TYPE-6, OP-1, FN-3, FN-7, EFF-1, EFF-2, EFF-3, STOR-3, PROG-1, DIAG-1), grammar productions +2/modified 3, terminal spellings +3 (`as`, `external`, `blocks`), sections +2, DIAG-1 rank insertion at position 5 with ranks 5/6 renumbered to 6/7. The packet's judgment items were accepted without override: the [SYS-3] syntactic conditional-visibility mechanism is selected (Route C stands; the recorded fallback to a prelude extension lapses unexercised); GATE-2 and TRAP-1 are ratified as additions beyond the dossier §11 inventory; the IoError two-field inline representation (`code: u32`, `origin: u8`, affine), the reserved `as` spelling, the IDENT-plus-closed-table kind/label mechanism, the admitted zero-input command entry, the rejected source call to a kind-declaring entry, and the fn_bind exclusion of system operations stand as drafted; the SCOPE-3 TCB extension remains dropped. The unlabelled entry and all v0.17-accepted program behavior are unchanged; no protected conformance source, verdict, expectation kind, or runnable status changes.
- boundary: approved candidate `governance/spec-evolution/kernel-spec-v0.18-candidate.md` SHA-256 `307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28`, to be installed byte-for-byte as `spec/kernel-spec-v0.18.md` through the guarded activation procedure with every derived artifact updated in the same work.
- evidence: owner instruction "批了" on 2026-08-05 following presentation of the exact-approval packet (candidate SHA-256; semantic delta; native verifier results — staged contract verified 64 productions / 74 decisions / 75 terminal predicates, active v0.17 unchanged 62/72/72; both gates green, 360 tests 0 failed; the eight-item judgment list). Hostile integration review (17 findings, all applied) at `85c0f5c`; staged grammar-path verifier landed at `a9c6e1a` under task 0005; architecture selection and review evidence under `research/investigations/system-capability-architecture/`.

## 2026-08-06 — approval
- owner: approved in session
- reason: Exact-byte approval of REVIEW CANDIDATE v0.19 (arg_get parameter-spelling repair). One existing rule modified: SYS-2, renaming `arg_get`'s value parameter from `index` to `position`, because `index` is a fixed GRAM-5 atom excluded from IDENT by FORM-3, making every complete legal `arg_get` call underivable under GRAM-11+SYS-2's required call-site spelling. Declaration-record preorder, ordinals, and counts unchanged; no other semantics change; the sweep of every SYS-2 parameter, field, operation, label-tail, and binder spelling against the 67 fixed terminals shows this is the sole collision. No protected conformance source, verdict, expectation kind, or runnable status changes.
- boundary: approved candidate `governance/spec-evolution/kernel-spec-v0.19-candidate.md` SHA-256 `7dda56d84094275f5ee2b0fa99bdcbcef83b158bb84acec62333aad8f87f7ec5`, to be installed byte-for-byte as `spec/kernel-spec-v0.19.md` with the derived material (compiler catalog spelling and coupled test, derivation-record amendment note, outline clearance) in the same work.
- evidence: owner authorization "授权" (2026-08-06) for the batch after the defect presentation (task 0007 finding, reproduction and rename control, sole-collision sweep), then exact-byte instruction "0.19批了" (2026-08-06) after presentation of the SHA-256, the one-line semantic diff, the grammar-preserving verifier result (64/74/75), and the derived-material inventory.

## 2026-08-06 — approval (supersedes the same-day v0.19 entry above)
- owner: approved in session
- reason: Exact-byte re-approval of the revised REVIEW CANDIDATE v0.19 after the first approved revision was found to omit the machine-checked META-5 `Selection ground:` header element (caught by the `whitefoot-spec` integrity gate at installation; the premature red-gate commit was dropped before anything else landed). The sole difference from the previously approved bytes is the added Selection ground sentence; the semantic delta is unchanged (SYS-2 `arg_get` parameter `index` → `position`).
- boundary: approved candidate `governance/spec-evolution/kernel-spec-v0.19-candidate.md` SHA-256 `01fb10d2d61cc87cce72cc98071eda98c7411fdc95af4ef29b79ac9a49cb5398`, installed byte-for-byte as `spec/kernel-spec-v0.19.md` with the derived material in the same work.
- evidence: owner instruction "批了" (2026-08-06) after presentation of the incident, the one-sentence diff, the new SHA-256, and the green verifier result.

## 2026-08-06 — approval
- owner: approved in session
- reason: Two protected-surface conformance rulings from the first full adapter run (task 0014). (1) The 41 protected case sources that are incomplete compilation units (no `fn main`, verified per case) receive a mechanical completion: one trivial entry (`fn main() -> own unit pure { return unit; }` or the minimal admissible variant) appended, every other byte unchanged, so each case again tests its declared rule; the two `accept` cases whose expectations were contradictory as fragments become satisfiable. (2) The 35 cases whose `runnable` status overclaims against the unimplemented borrow-mode-parameter capability are corrected to `pending` with reasons naming that capability, expectations unchanged, on the recorded per-case evidence. No expectation, rule citation, or verdict changes under either ruling.
- boundary: the 41 case sources and 35 manifest status fields exactly as enumerated by task 0014's adapter run (`docs/done/0014-first-slice-conformance-execution.md`); execution recorded under task 0019.
- evidence: owner instruction "都批" (2026-08-06) after a plain-language presentation of both rulings and their alternatives; first full corpus run Pass=242 Fail=123 with the four-bucket classification.

## 2026-08-06 — approval
- owner: approved in session
- reason: One-token amendment to the protected conformance source `gram5-pos-recursive-place-projection.wf`: task 0019's bucket-4 classification found its `run 0` expectation unsatisfiable as written — the case omits the explicit region instantiation argument FN-2 requires at its region-generic call — and adding `<'projection>` at that call makes the program compile and run to its declared exit. The expectation, rules, and every other byte are unchanged.
- boundary: the one call-site token in `tests/conformance/cases/gram5-pos-recursive-place-projection.wf`; executed at landing under the ACTIVE plan's Work item 3.
- evidence: owner instruction "都批准" (2026-08-06) after presentation; classification and reproduction at `docs/done/0019-conformance-rulings-execution.md`.

## 2026-08-06 — approval
- owner: approved in session
- reason: (1) The 19-item protected-source amendment bundle from the 0024/0025 investigations, each a mechanical one-or-two-token repair of a case defect with the compiler behavior verified correct: five GRAM-10 binder renames (binder spelled equal to its paired field, forbidden by three GRAM-10 sentences); three OP-family verdict corrections to OP-1 (resolution precedes semantics, OP-2/OP-7/OP-8 unreachable for an unknown OPNAME by construction); one region-parameter addition unmasking EFF-1 (`x-eff-dup-reads-effect`); one `return move item;` repair (`own1-pos-return-affine-contextual-move`; OWN-1's closed consumption list); three authored `pure` rows gaining the `reads('r)` their borrowed reads attribute; six missing region arguments (TYPE-5). (2) Authorization of a v0.20 micro specification batch for the two recorded gaps — the OWN-6 disposition of reborrow forms outside call-argument position, and the ordering of simultaneously established TYPE-7/OWN-1 rejections — through the full specification-change workflow with exact byte approval still required. (3) Selection of the wide-scan-and-settlement plan as ACTIVE.
- boundary: the 19 enumerated case sources/manifest verdicts per docs/done/0024 and docs/done/0025; the v0.20 candidate when presented; docs/current-plan.md ACTIVE.
- evidence: owner instruction "都批" (2026-08-06) after the plain-language presentation of all three items.

## 2026-08-07 — approval
- owner: approved in session
- reason: (1) Exact-byte approval of the v0.20 specification batch: `governance/spec-evolution/kernel-spec-v0.20-candidate.md` at SHA-256 `b082ef3fa8d2ee630b7e5b6ecb55ff004ed2473c566040150a1297a61b312dc1` — OWN-14 returned reborrows, the OWN-13 arm-scoped child reborrow closing the OWN-13/OWN-5 payload-binder contradiction, and the DIAG-1 same-node citation rank; drafted disposition A on both sub-choices. (2) Four protected-source completions unmasked by task 0027: trivial `fn main` appended to `own1-pos-match-copy-payload-reuse`, `own1-neg-match-move-through-borrow`, and `own5-neg-match-borrow-affine-payload-move` (the 41-source mechanical-completion shape), and the second bare affine return in `own1-pos-return-affine-contextual-move` repaired to `return move holder;`.
- boundary: those exact candidate bytes for activation as `spec/kernel-spec-v0.20.md`; the four enumerated source edits and nothing else.
- evidence: owner instruction "全批" (2026-08-07) after the plain-language packet.

## 2026-08-07 — approval
- owner: approved in session
- reason: (1) Rulings O1–O16 on the obligation-discharge batch-1 candidate, each resolving to the lead-recommended option (notably: v0.21 target; `index_get` dotless spelling; four-field claim trap record; Section D rides in the batch; the u64 offset-typing fix lands now). (2) Adoption of the sixteenth modified rule: the OP-1 non-consuming place-operand sentence for `len`/`slice_of`/`index`/`index_get` bases. (3) Approval of `governance/spec-evolution/obligation-discharge-batch1-candidate.md` into the activation pipeline: claim construct (CLM-1/2), the L0 entailment fragment (ENT-1..6), OP-4 caller-side discharge with `index_get`, and the SYS count-bound postconditions, after the adversarial-review cycle closed all findings (F1–F11, re-verify at commit 9a4ff9f).
- boundary: the candidate as finalized with rule sixteen adopted; exact-byte approval of the full-document v0.21 candidate remains a separate entry at the guarded version bump, after the compiler grammar-path extension (ruling O1).
- evidence: owner instructions "没问题" (rulings, 2026-08-07) and "没问题，开始吧" (adoption + pipeline approval, 2026-08-07) after the plain-language sittings.

## 2026-08-07 — approval (sequencing amendment)
- owner: approved in session
- reason: ruling on the task-0030 blocker (retired staged-grammar mechanism): option 3, atomic activation — the compiler grammar-path extension, the v0.21 full-document candidate, and every identity pin move together on one reviewed task branch; the verifier shows 65/77 green on the branch and 64/74/75 on main until merge. Amends ruling O1's "extension before candidate generation" to "concurrent on the activation branch". Table data by mirroring the else-row pattern for because (claim_stmt is grammar-isomorphic to check_stmt), with a one-shot generator deleted after use as fallback. No staged dual-contract rebuild.
- boundary: task 0030 re-scoped accordingly; step-4 exact-byte approval of the v0.21 candidate remains a separate owner entry before installation.
- evidence: owner instruction "嗯，3" (2026-08-07) after the three-option presentation.

## 2026-08-07 — approval (exact byte)
- owner: approved in session
- reason: WORKFLOW step-4 exact-byte approval of the v0.21 specification batch: `governance/spec-evolution/kernel-spec-v0.21-candidate.md` at SHA-256 `3c63a6274047ee2f7eceac7ec6b03d0b84d42fb87cc13da7e6b80ed5b934df9f` — 128 rules (CLM-1/2, ENT-1..6 added; fifteen rules modified at sixteen sites), obligation-discharge batch 1, grammar path verified 65/74/77 on task/0030 with main at 64/74/75.
- boundary: those exact candidate bytes for activation as `spec/kernel-spec-v0.21.md`; integration of task/0030-grammar-path-extension.
- evidence: owner instruction "approved" (2026-08-07) after the step-4 presentation.

## 2026-08-07 — approval (process simplification)
- owner: approved in session
- reason: corpus/test migration rules relaxed — edits (scripted or manual) are permitted; the owner reviews the result, same treatment as spec bytes. Standing simplification going forward: spec changes and test changes both go to owner review; no canonical-emitter tooling is built for migration at this stage. Existing law unchanged: conformance verdict meanings never shift silently.
- boundary: task 0031 item (4) re-scoped to reviewable respell; future respell batches inherit this process.
- evidence: owner instruction "算了，把规矩放宽一点。可以改，但改完了要给我看，我审批…未来spec和测试都是给我review就好了" (2026-08-07).

## 2026-08-07 — approval (conformance repurpose)
- owner: approved in session
- reason: `type5-neg-index-element-type` repurposed — v0.22 deletes stated index element types, making the tested error unspellable; the case becomes a positive pin of the new element-type derivation (`items[0_u64]` binds u8). Same concern, stronger form (R4: check-time rejection promoted to unrepresentable).
- boundary: that one case source + manifest row, within task 0031.
- evidence: owner instruction "改用途" (2026-08-07).

## 2026-08-07 — approval (exact byte, v0.22)
- owner: review delegated to lead ("帮我过一下好了"); lead verified and approved
- reason: WORKFLOW step-4 exact-byte approval of `governance/spec-evolution/kernel-spec-v0.22-candidate.md` at SHA-256 `b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8` — index surface settlement (subscript psuffix, element-type derivation, index_get removal, S8 parked, ENT-1 scoping), 128 rules. Review packet accepted: 137 files, corpus respell verdict-meaning-preserving (adapter 364/1/14 -> 365/1/14, same single pre-existing fail), repurposed + additive cases per rulings, `=[` cvalue attachment accepted as-ruled (revisit at FLOOR-5). Lead spot-checks: hash both sides, utf8parse zero old-form sites, patterns.md clean.
- boundary: those exact bytes installed as `spec/kernel-spec-v0.22.md`; integration of task/0031-v022-grammar-and-respell.
- evidence: owner instruction "帮我过一下好了" (2026-08-07) after the packet presentation.

## 2026-08-07 — standing overnight delegation
- owner: granted in session before sleeping
- reason: full decision delegation to the lead for the night — reviews (including 0034's corpus-migration diff and any executor deliverable), integration, and sequencing proceed without owner stops; nothing blocks awaiting owner input. Owner ratifies or fixes forward after waking.
- boundary: the active implementation ladder (0033-0035) and its integration; spec ACTIVATIONS remain out of scope for the night (FLOOR-5 stays frozen at READY for an awake review).
- evidence: owner instruction "别停了，一直跑吧，你帮我决定。别block了…如果真有问题的话到时候再修" (2026-08-07).

## 2026-08-07 — approval (overnight delegation: 0034 conformance changes)
- owner: lead-approved under the recorded overnight delegation; owner ratification pending
- reason: the OP-4 flip's conformance consequences — op4-pos-index repurposed to op4-neg-index-undischarged (+ discharged sibling), op4-trap-index-oob repurposed to clm1-trap-runtime-violation (runtime-trap concern relocated to the claim per the R4 ladder), nine verdict-meaning-preserving lane migrations (incl. the eff2 pair's trap-carrier swap, old carrier inexpressible), runner ACTIVE_SPEC pin advanced to v0.22 with six interim compiler-test annotations pending 0035's independent cases. Adapter 364/1/14 -> 366/1/14; coverage 128/128.
- boundary: the enumerated cases and manifest rows in task/0034 commit b382cf8.
- evidence: 0034 report + lead review, per the 2026-08-07 standing overnight delegation.
