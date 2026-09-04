# Specification and conformance approval record

This append-only file records historical approvals and exact specification or
conformance content approved under rule 4 of
[`docs/WORKFLOW.md`](../docs/WORKFLOW.md). It is not a workflow guide. Every
earlier plan, batch, packet, staged-approval, or protected-change procedure
described below is historical and superseded; none imposes a current approval
or work requirement. Ordinary merges still require owner approval under rule 2
but need no entry here unless they also change specification or conformance
content.

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

## 2026-08-07 — approval (FLOOR-5 and the remaining dossier lane)
- owner: approved in session
- reason: (1) the FLOOR-5 spelling batch (`governance/spec-evolution/spelling-relief-candidate.md`, READY after three adversarial rounds — 24 rules at 46 sites, 69 productions) approved for activation through the grammar-path + migration cycle; (2) the remaining DOSSIER §8 items all approved as a lane: taint gate (subject-position), counted range loop, requires-as-goal, ensures, deny-claims partition; (3) the taint gate is ADVANCED in priority on the 0035 acceptance evidence (three canonical-Huffman sites became aborting claims where the design demands Err branches, because no gate exists); (4) ENT-5 loop-rule fix drafted pending the owner's word after the written explanation.
- boundary: drafting and implementation proceed; each spec activation still takes its own exact-byte entry.
- evidence: owner instruction "从语法到剩下的5个都极其重要。批准了，开始做吧" + "2.提前" (2026-08-07).

## 2026-08-07 — approval (ENT-5 loop-rule fix)
- owner: approved in session after the written explanation
- reason: the loop rule's kill scan considers only kill events on edges that can reach a later iteration head; edges leaving the loop or the function (return, break, propagate's error edge) are excluded, since no later head observes their kills. Lead-accepted drafting detail (O1): the reachability form rather than the enumerated statement-kind form — sound, needs no enumeration, and additionally drops a `set`'s kill when only a return follows. Honest correction of record: this is NOT fully monotone — discharge and redundancy only widen, but a surviving pre-loop fact can newly supply a claim's exact negation, so CLM-2 refutation can newly reject a program that traps on every execution reaching it (ENT-1's already-enumerated non-monotone edge).
- boundary: the ENT-5 site only; the provenance gate drafted beside it is split out pending measurement.
- evidence: owner instruction "OK，我同意这个改法，批准了。循环体return回边就这么做吧" (2026-08-07).

## 2026-08-07 — approval (O8 precedent: what discharges a needs-evidence register entry)
- owner: approved in session ("完全同意")
- reason: an R3-PROVISIONAL register entry is discharged by argument alone when the deciding criterion is itself mechanically checkable (T1 reconstructibility, T3 uniqueness, T4 class-globality all verify by machine) — a writer-tier experiment there is ceremony. When the criterion is which spelling a writer gets right more often, argument is insufficient and measured data is required. FLOOR-5's three settlements (no-if, prefix arithmetic, the body half of the annotation mandate) fall in the first class. Precedent governs the remaining eleven entries.
- boundary: register discharge only; no spec rule changes by this entry.
- evidence: lead proposal + owner instruction "完全同意" (2026-08-07).

## 2026-08-07 — scope amendment (activated candidates removed)
- owner: lead action under the standing overnight delegation; owner ratification pending
- reason: the thirteen candidate files `kernel-spec-v0.10-candidate.md` through `kernel-spec-v0.22-candidate.md` were deleted after each was verified byte-identical (`cmp`) to its installed `spec/kernel-spec-vN.md`. They were 2.2 MB of parallel versions of an append-only directory, which the hygiene rule forbids; the approved bytes survive unchanged in `spec/`, and each prior exact-byte approval entry remains valid by SHA-256 against the installed file. `kernel-spec-v0.23-candidate.md` is in flight and untouched. Going forward a candidate is deleted at activation once the installed bytes are verified identical.
- boundary: twelve files. Refined on landing: the candidate for the CURRENTLY ACTIVE version is retained, because a live check consumes it — `compiler/src/bin/spec.rs` now compares the installed specification against the approved candidate bytes rather than against itself, which was a tautology. The rule going forward: at activation, delete the candidate of the version being superseded, never the one just installed.
- evidence: process-design workflow finding (M5); per-file `cmp` verification.

## 2026-08-07 — approval (stable active-specification filename)
- owner: approved in session
- reason: the law amendment drafted at `governance/spec-evolution/stable-spec-filename-proposal.md` is approved — the active specification moves to the stable path `spec/kernel-spec.md`, edited directly on task branches, with the superseded bytes archived flat at `spec/kernel-spec-vN.md` at each activation. All eight mandatory amendments from the adversarial judgment are conditions of adoption: flat archive (no `spec/released/`), computed SHA-256 digest, chained `ACTIVE-SPEC:` approval record, landed-state archive-integrity gate in `make check`, two-path grammar verifier, linear activation only (no `-X ours`/`-X theirs` on the specification), archive-creates-or-fails, and the status word inside the approved bytes. The law text in `CLAUDE.md`/`AGENTS.md` and `docs/WORKFLOW.md` is amended so that the ARCHIVED versioned specifications remain absolutely immutable and hook-enforced, while the active file at the stable path is mutable by design with its integrity carried by the digest chain and the archive gate rather than by the filename.
- boundary: the switchover rides the first small activation with no EBNF change (the approved ENT-5 loop-rule fix), never a standalone commit and never the FLOOR-5/v0.23 activation, which proceeds unchanged. Task 0039's prerequisites land and go green first.
- evidence: owner instruction "批" (2026-08-07) after the drafted proposal and the four-lens adversarial judgment.

## Activation chain

One line per specification activation, in order, so the active identity is a
recorded fact rather than a constant someone remembered to move. The format is
exact: `ACTIVE-SPEC: <version> <sha256-installed> <sha256-superseded>`, with
`-` where no earlier version was approved by exact bytes. `whitefoot-spec`
parses these and refuses to build a specification whose installed bytes,
version label, or predecessor disagree with the chain, so an activation that
forgets a line, or writes the wrong digest into one, fails the gate.

These lines add no approval. Every digest below is the exact-byte value the
owner approved in an exact-byte entry in this ledger; the chain begins at v0.9
because that is where exact-byte approval began. A new activation appends its
line in the same change that installs the specification.

ACTIVE-SPEC: v0.9 bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68 -
ACTIVE-SPEC: v0.10 71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9 bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68
ACTIVE-SPEC: v0.11 050e110c8c5eb3143c9d3f54968a9df9125f1d4b5991f527b8a15938a4292fbc 71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9
ACTIVE-SPEC: v0.12 e2d5566379891454c090e037bd45c5f1a8df90ba23506a0f83ce9aaa03b41463 050e110c8c5eb3143c9d3f54968a9df9125f1d4b5991f527b8a15938a4292fbc
ACTIVE-SPEC: v0.13 ed93cc43a6a224725f813b1adfc4c19fbb64dc5ab294b25d924392d2959b77cd e2d5566379891454c090e037bd45c5f1a8df90ba23506a0f83ce9aaa03b41463
ACTIVE-SPEC: v0.14 31c09313363304f405c8db1191d1982e3625b86788bf953ec3bb169648466e9f ed93cc43a6a224725f813b1adfc4c19fbb64dc5ab294b25d924392d2959b77cd
ACTIVE-SPEC: v0.15 3c924095b2c21f123b7137556f72dbe87275838682c1965e6caf399dd24d13bd 31c09313363304f405c8db1191d1982e3625b86788bf953ec3bb169648466e9f
ACTIVE-SPEC: v0.16 f93264fb4df6994a408e1327c6e8643c34b4aea55fba4b1a0b3dab3501ceb942 3c924095b2c21f123b7137556f72dbe87275838682c1965e6caf399dd24d13bd
ACTIVE-SPEC: v0.17 19642ffb0ad9c7146a84762ada192ed2a25dc446a93c4d060aa29d9a99f69c93 f93264fb4df6994a408e1327c6e8643c34b4aea55fba4b1a0b3dab3501ceb942
ACTIVE-SPEC: v0.18 307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28 19642ffb0ad9c7146a84762ada192ed2a25dc446a93c4d060aa29d9a99f69c93
ACTIVE-SPEC: v0.19 01fb10d2d61cc87cce72cc98071eda98c7411fdc95af4ef29b79ac9a49cb5398 307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28
ACTIVE-SPEC: v0.20 b082ef3fa8d2ee630b7e5b6ecb55ff004ed2473c566040150a1297a61b312dc1 01fb10d2d61cc87cce72cc98071eda98c7411fdc95af4ef29b79ac9a49cb5398
ACTIVE-SPEC: v0.21 3c63a6274047ee2f7eceac7ec6b03d0b84d42fb87cc13da7e6b80ed5b934df9f b082ef3fa8d2ee630b7e5b6ecb55ff004ed2473c566040150a1297a61b312dc1
ACTIVE-SPEC: v0.22 b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8 3c63a6274047ee2f7eceac7ec6b03d0b84d42fb87cc13da7e6b80ed5b934df9f
ACTIVE-SPEC: v0.23 e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5 b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8
ACTIVE-SPEC: v0.24 53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86 e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5
ACTIVE-SPEC: v0.25 c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab 53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86
ACTIVE-SPEC: v0.26 18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476 c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab
ACTIVE-SPEC: v0.27 bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f 18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476
ACTIVE-SPEC: v0.28 08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09 bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f
ACTIVE-SPEC: v0.29 0b7aa8ccee958ba85613c51535165dcbf7ac12db556b2210d2f1aac0d39e6cc3 08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09

## 2026-08-09 — OWNER EXACT-BYTE APPROVAL: v0.23, and its activation
- owner: **the owner's exact-byte approval of the digest below.** It was taken
  by the lead and relayed to this executor for installation; this executor did
  not witness it and records it as reported, which is stated so the trail shows
  where the approval entered rather than implying a second witness. The entry
  immediately above warned that approval must not be inferred from it — this is
  the entry it was waiting for.
- APPROVED BYTES:
  `e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5`, verified
  by this executor with `shasum -a 256` against
  `governance/spec-evolution/kernel-spec-v0.23-candidate.md` before anything was
  installed, and again against the installed `spec/kernel-spec-v0.23.md`, with
  `cmp` reporting the two files byte-identical. Approval covers exactly those
  bytes; a changed byte returns to review.
- SCOPE: numbered rules +0/-0; thirty-four existing rules modified at sixty-two
  verbatim-anchored sites; grammar productions 65 + 4 = 69; sixteen operator
  terminal spellings; the accepted-program set changing as one canonical
  respelling, three deliberate narrowings, and one expressible form removed
  whose effect on the accepted set is not established.
- INSTALLED AT THE VERSIONED PATH, deliberately. `spec/kernel-spec-v0.23.md`,
  not the stable filename. The approved stable-filename proposal's §5 sequences
  that switchover onto the first activation with NO EBNF change, which is the
  ENT-5 loop-rule fix, never this one.
- THE CHAINED LINE BELOW IS AN OWNER APPROVAL RECORD. It is written because the
  owner approved and for no other reason; writing one to make a gate green is
  forbidden, and the two activation-gated tests were red before this commit
  precisely so that nothing else could close them.
- STANDING ACTIVATION ITEM, not a v0.23 finding: the three guards at
  `compiler/src/backend/qualification.rs` hard-code the active version string.
  They read `!= "v0.23"` and therefore pass here — verified by reading all
  three, not assumed — but every future activation must repoint them. They fail
  CLOSED when stale, so a missed update loses qualification silently rather
  than loudly, which is why it has recurred at every bump since v0.19.
- boundary: one specification file installed, the pins that name it, the
  regenerated grammar tables, the roadmap's authority line and revision, and
  this ledger. No conformance case, no manifest row, no compiler behaviour.
- evidence: the digest recomputed from the file rather than copied from any
  report; `cmp` on the installed bytes; the gate figures in the activation
  commit message, with failure sets by name.

## 2026-08-09 — OWNER EXACT-BYTE APPROVAL: v0.24 ENT-5 and stable-path switchover
- owner: **the owner, in conversation, 2026-08-09** — “批准”. The reply was
  given directly against the immediately preceding exact request naming all
  three approval objects below; it is not inferred from the earlier overnight
  delegation or from the superseded v0.22-anchored ENT-5 approval.
- APPROVED BYTES: active `spec/kernel-spec.md`, version v0.24, SHA-256
  `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
  Relative to immutable v0.23, the complete file has exactly three reviewed
  diff hunks: title, the new Status paragraph plus byte-preserved v0.23 Prior
  paragraph, and ENT-5's final-paragraph replacement. A changed byte returns to
  exact review.
- SEMANTIC SCOPE: a pre-loop fact is removed at an iteration head only by a
  continuing kill whose structural normal-control successor can reach a later
  head of that same loop without leaving its body. Return, propagated-error,
  and current/enclosing break edges are non-continuing; ordinary fallthrough,
  else-free false edges, and nested-loop continuations keep their real kills.
  Rules +0/-0 and grammar remains 69 productions, 84 decisions, 93 terminal
  predicates.
- PROTECTED CORPUS: approve the exact changes in review commit `00e6ce4`:
  rewrite `ent5-neg-loop-rule-drops-preloop-fact` so an else-free false edge
  carries a continuing kill while preserving runnable `reject OP-4`; and add
  runnable accepted case `ent5-pos-return-does-not-kill-loop-head-fact`. No
  existing verdict, cited rejection rule, or runnable status changes.
- S10 DISPOSITION: accept as item-4 revalidation the real boundary path
  producing `taken <= room` plus focused actual-index obligations consuming
  all four S10 transfer producers and covering invalidation. The preregistered
  raw-deflate driver itself has no natural entailment obligation consuming that
  relation; no sentinel access, extra copy loop, or other evidence-shaped
  behavior is added, and no end-to-end consumer is claimed.
- STABLE-PATH SWITCH: v0.23 remains byte-identical at immutable
  `spec/kernel-spec-v0.23.md`; v0.24 is active only at
  `spec/kernel-spec.md`. This first switch creates no archive, creates no
  `spec/kernel-spec-v0.24.md`, and writes no `ARCHIVE-SPEC: v0.23` record.
- boundary: the exact specification bytes, ENT-5 implementation and tests,
  named protected corpus changes, stable-path identity/pins/law, derivation
  binding, and live authority. O11, provenance activation, later
  obligation-discharge features, and wfgrep are outside this approval.
- evidence: review implementation `00e6ce4`; exact packet and frozen candidate
  acceptance `7e47130`; independent semantic and exact-byte reviews; synthetic
  activation rehearsal recorded by `0e88b86` and atomicity audit `6f22a7b`.

## 2026-08-09 — OWNER EXACT-BYTE APPROVAL: v0.25 counted `u64` range
- owner: **the owner, in conversation, 2026-08-09** — “同意。批准 v0.25
  c0b3c279（含上述 protected prose 变更）。” The reply followed the complete
  Chinese owner walkthrough of the language surface, execution and cleanup,
  S11 proof boundary, real SHA-256 result, accepted-set impact, protected
  corpus delta, stable-file installation, limitations, and exact full digest.
  It is an explicit approval, not an inference from the earlier lane approval.
- APPROVED BYTES: active `spec/kernel-spec.md`, version v0.25, SHA-256
  `c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`.
  The outgoing immutable v0.24 archive is byte-identical to the previously
  active stable file and has SHA-256
  `53495b9c47b92942876c90931d0296c752855954564ebf7435a549c48cb2dc86`.
  A changed specification or archive byte returns to exact review.
- SEMANTIC SCOPE: add exactly one ascending, unit-stride, half-open counted
  source form, `for @label binder in lower..upper { ... }`, over once-captured
  `own u64` term-or-constant endpoints. The compiler-updated binder is
  body-local and source-immutable. Normal fallthrough cleans up before the
  representable hidden increment; `break`, `return`, and propagated error do
  not increment. S11 establishes only the finite structural body-entry bounds
  `lower_capture <= binder < upper_capture`; it adds no general induction or
  exit postcondition, and ordinary `loop_stmt` semantics remain unchanged.
  Numbered rules remain 128; the native grammar is 70 productions, 85
  decisions, and 96 terminal predicates.
- ACCEPTED-SET IMPACT: the new counted form and its S11 facts widen the set.
  The only v0.24-source narrowing is that exact lowercase `for` and `in` leave
  IDENT eligibility; the frozen 448-file census found no live declaration or
  use collision. Compound `..` preserves every prior single-dot numeric and
  member partition.
- PROTECTED CORPUS: approve the reviewed source-doc line and manifest
  reason/doc rederivation for `gram6-pos-no-operators`; its id, cited GRAM-6
  rule, `Run(0)` verdict, status, and behavior remain unchanged. Also approve
  three additive cases: `ent2-neg-counted-subscript-endpoint`,
  `ent3-pos-s11-counted-range-run`, and `set1-neg-counted-binder-write`. No
  other protected source, verdict, rule, or status changes.
- REAL-PROGRAM EVIDENCE: exactly three SHA-256 index loops use the counted
  form, four claims disappear, and all 9/9 subscript obligations discharge
  without S2/S3. The worker becomes `pure`, emits no `wf_trap`, and retains
  both the direct `3128432319_u32` result and the sustained runtime oracle.
- STABLE-PATH INSTALLATION: v0.25 remains at `spec/kernel-spec.md`; the exact
  outgoing bytes are newly archived at immutable
  `spec/kernel-spec-v0.24.md`. No `spec/kernel-spec-v0.25.md` is created; that
  archive is created only when a later approved version supersedes v0.25.
- boundary: the exact specification and archive bytes, counted-range compiler
  implementation and tests, named protected prose plus three additive cases,
  generated data, active pins, derived material, writer documentation, and
  live authority. Requires-as-goal, provenance activation, ensures, claim
  ledger, deny-claims, O11, and wfgrep are outside this approval.
- owner-process instruction: before every future specification approval
  request, first present an owner-facing explanation at this level, then stop
  and wait for the owner's explicit response. Never combine the walkthrough
  and the request with continued activation work.
- evidence: task 0047's independently reviewed exact candidate and archive;
  native grammar verification; focused counted tests 37/37; frozen acceptance;
  and the complete ignored adapter tally `Pass=393 Fail=1 Skip=13`, whose sole
  failure remains the pre-existing OWN-3 unsupported case.

## 2026-08-09 — OWNER EXACT-BYTE APPROVAL: v0.26 atomic `requires` goals
- owner: **the owner, in conversation, 2026-08-09** — “批准 v0.26 18aa00e3
  （含上述 9 个 protected source 变更、8 条 manifest doc 变更，并归档
  v0.25 c0b3c279）。” The reply followed the complete Chinese owner walkthrough
  of the ordinary-call proof boundary, atomic goal identity, signed evidence,
  S4/body effects, real process-entry checks, O3 metadata, accepted-set impact,
  protected and real-program changes, stable-file action, limitations, exact
  digests, and verification. It is an explicit approval after the required
  hard wait, not an inference from the earlier direction selection.
- APPROVED BYTES: active `spec/kernel-spec.md`, version v0.26, SHA-256
  `18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
  The outgoing immutable v0.25 archive is byte-identical to the previously
  active stable file and has SHA-256
  `c0b3c279f4c20d06da17ef7ac0e4ec882c8a76c560f62cce47d5b4fd4ac6beab`.
  A changed specification or archive byte returns to exact review.
- SEMANTIC SCOPE: preserve the complete FN-8 copy-local plus one pure, total,
  non-trapping Bool-check declaration surface as one finite typed atomic goal,
  and admit it on generic function declarations. An ordinary call proves its
  exact substituted goal after actual obligations and borrow feasibility but
  before transfer and callee-effect kills; refuted or unproved calls reject,
  and no ordinary fallback or executable callee prologue remains. S4 supplies
  the proved goal to the body. Signed opaque facts follow exact support, kill,
  join, loop, and contradiction rules; only an exact comparison root may
  project to L0, and Boolean DAGs are never decomposed or composed.
- ENTRY AND EFFECT BOUNDARY: a requirement is a signature obligation and
  contributes no callee effect. Explicit source checks and claims retain their
  ordinary `traps` contribution. The two implemented process wrappers each
  evaluate the complete requirement exactly once after setup and before owner
  transfer: false preserves the OP-5 record with zero body calls, while true
  transfers each owner once to one body call. No FFI, export, foreign stub,
  owner-taking helper, optimizer assumption, or duplicate cleanup is added.
- ACCEPTED-SET IMPACT: ordinary calls lacking proof of the complete instantiated
  goal and effect rows padded solely for the former prologue narrow. Generic
  functions with requirements, exact `pure` rows for otherwise-pure required
  bodies, and all-derivable unreachable states exposed by signed contradiction
  widen. Existing nongeneric FN-8 shapes, enum equality, the complete base64
  capacity DAG, copy-only clause-local restrictions, ordinary body checks and
  claims, process-entry behavior, and every non-requirement operation remain.
- PROTECTED CORPUS: approve exactly nine source changes:
  `ent3-pos-s4-requires-fact`, `fn3-neg-requires-member`,
  `fn8-neg-requires-missing-traps`, `fn8-pos-requires-eeq`,
  `fn8-pos-requires-name-reuse`, `fn8-pos-requires-run`,
  `fn8-trap-requires-false`, `x-base64-rfc-vectors-run`, and
  `x-requires-output-capacity-run`. Approve exactly eight corresponding
  manifest `doc` changes; every id, rules list, verdict, status, and runtime
  subject remains as reviewed. The two noncopy-local FN-8 controls remain
  byte-identical. No other protected source or manifest semantic field changes.
- REAL-PROGRAM EVIDENCE: base64 keeps the exact full capacity DAG and adds the
  same complete caller evidence before each of three calls. `percent_decode`,
  `utf8parse`, and `raw_deflate_dynamic_decode` change only one exact effect-row
  line apiece where the former prologue was the sole read contributor. The
  frozen buckets remain UTF-8 `33/22/11/0`, SHA-256 `9/9/0/0`, complete
  DEFLATE `29/11/18/0`, and dynamic DEFLATE `24/11/13/0`; the same five
  DEFLATE claims are redundant, sixteen are retained, and none is refuted.
  All three `store_dynamic_length` calls prove their goal in the retained
  unasserted and S4-blinded rewalk, with the distance claim unchanged.
- HELD PROVENANCE BOUNDARY: checked metadata retains the finite subject-only
  requirement-to-protected-leaf bridge, dependency summaries, counterfactual
  rewalks, fixed-point call composition, and deterministic predecessors. v0.26
  emits no provenance rejection and does not activate the held PRV gate.
- STABLE-PATH INSTALLATION: v0.26 remains only at `spec/kernel-spec.md`; the
  exact outgoing bytes are newly archived at immutable
  `spec/kernel-spec-v0.25.md`. No `spec/kernel-spec-v0.26.md` is created.
- boundary: the exact specification and archive bytes, requires-goal compiler
  implementation and tests, the nine named protected source changes and eight
  manifest-doc changes, three named real-program effect-row updates, generated
  data, active pins, derived material, writer documentation, live authority,
  and the activation-following paired design re-decision. Provenance rejection,
  ensures, O11, claim ledger, deny-claims, FFI, and wfgrep are outside this
  approval.
- activation sequence: atomically install this approved packet first; then use
  that commit's real eight-character identity in the immediately following
  paired MCTS re-decision. Installed acceptance, the complete gate, and MCTS
  lint precede task closure and stage 5b.
- evidence: task 0048's exact candidate/archive reviews; independent compiler,
  protected, packet, acceptance, and MCTS-shape reviews; native grammar
  `70/85/96`; grammar-table identity; focused and complete compiler/program
  tests; conformance coverage `128/128`; and the complete ignored adapter tally
  `Pass=393 Fail=1 Skip=13`, whose sole failure remains the pre-existing OWN-3
  unsupported case.

## 2026-08-10 — OWNER EXACT-BYTE APPROVAL: v0.27 provenance gate
- owner: **the owner, in conversation, 2026-08-10** — “批准 v0.27 bbd72500
  （含上述 11 个 raw-DEFLATE claim→branch 迁移、store_dynamic_length Result
  与 3 个 propagate、仅四条 traps 删除、16 个 additive conformance case，
  并归档 v0.26 18aa00e3）。” The reply followed the complete Chinese owner
  walkthrough and the required hard wait. It is an explicit approval of the
  exact bytes and named protected changes, not an inference from the earlier
  Stage 5b direction or plan approval.
- APPROVED BYTES: active `spec/kernel-spec.md`, version v0.27, SHA-256
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.
  The outgoing immutable v0.26 archive is byte-identical to the previously
  active stable file and has SHA-256
  `18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
  A changed specification or archive byte returns to exact review.
- APPROVED CONSUMER MIGRATION: exactly eleven raw-DEFLATE claims become real
  value branches with the reviewed existing error mappings;
  `store_dynamic_length` returns `Result<unit, InflateError>` and exactly three
  callers use `propagate`. The only `traps` deletions are from
  `store_dynamic_length`, `decode_length`, `copy_distance`, and `decode_fixed`;
  every other effect row, error mapping, and runtime oracle remains unchanged.
- PROTECTED CORPUS: approve exactly sixteen additive provenance conformance
  cases and their additive manifest rows. No existing conformance case source,
  manifest row, or annotation changes.
- STABLE-PATH INSTALLATION: v0.27 remains only at `spec/kernel-spec.md`; the
  exact outgoing bytes are newly archived at immutable
  `spec/kernel-spec-v0.26.md`. No `spec/kernel-spec-v0.27.md` is created.
- boundary: this approval covers the exact v0.27 specification and outgoing
  v0.26 archive bytes, the named raw-DEFLATE migration, and the sixteen
  additive conformance cases. Stage 8a is not approved; this commit performs
  no MCTS activation and creates no v0.27 archive.

## 2026-08-15 — OWNER EXACT-BYTE APPROVAL: v0.28 verified normal-return postconditions
- owner: **the owner, in conversation, 2026-08-15** — “按上述 SHA 激活 v0.28
  specification，并加入上述 manifest SHA 与 14 个 source SHA 所定义的受保护
  conformance 候选；随后在当前单一 codex/0047 开发线上执行一次原子化 main
  集成，写入批准/ACTIVE-SPEC 链及普通 closure 记录，并运行完整激活后
  gates。” The reply followed the revised exact packet and its hard wait. It
  explicitly approves the named specification, protected corpus, canonical
  runner identity, archive action, chain entry, and atomic integration.
- APPROVED BYTES: active `spec/kernel-spec.md`, version v0.28, SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`.
  The outgoing immutable v0.27 archive is byte-identical to the previously
  active stable file and has SHA-256
  `bbd7250084123bbce3267f741f30f6c12efc73c341ff8d361dd1b19d9502090f`.
  A changed specification or archive byte returns to exact review.
- PROTECTED CORPUS: approve the append-only manifest at SHA-256
  `8fada5059b57d563ab00a1c1c305dcd5810201ea2c507ee00a4137102bfc18f3`;
  its fourteen-row suffix is
  `d388c6d01ad1de4a294bef0d64fb3074544fb90f2c0c3a4177f7dea587579ab6`.
  Approve exactly these additive source identities, with no prior case or row
  modification, deletion, rename, or reorder:
  `fn9-pos-plain-direct-result` `b3ce74fada6ea840b39d319952ee6b4a0393eb5cd6c2cff6c7faaaf2ddea8a7e`;
  `fn9-pos-ok-selected-receiver` `ad5983b34a0d050b8404248e83d4607ed05af3b4120eaba8f4193023bf5ec79b`;
  `fn9-neg-no-selected-normal-exit` `bccccff8b2725b4bc57a7fc676a045c279057930073d8cc1792ecdd610d3d776`;
  `fn9-neg-unproved-selected-return` `401716a140d4702c31866363f72665df3aa503e379feb8043f5712a101c6fc46`;
  `fn9-neg-entry-image-kill` `8c95f20c44fcf083d60e3e62b5aacd7199a7026006eb7898bf8af6a225cd2515`;
  `fn9-neg-same-scc-summary` `b9adaabea6f31847c7c9495ecd3b94e2a1c3f28d3043e753abdde69364dd013f`;
  `ent3-pos-stage8b-bit-sources` `348aa8a014bf7204dfbc17061234b5076188393b0be308fe3274699ab8c50be1`;
  `ent3-neg-stage8b-local-one` `7fe3f057ab5ca803efb52450c6c92ec9d2d5db0937478f4b0e2dff776467e9bb`;
  `fn9-pos-direct-set-receiver` `f989f805cdfa18d490a795fa050dea0c898a0dd64c067248bb5e407d52c27880`;
  `fn9-neg-named-outcome-no-publication` `4a678d82a9e637f26ae2aae4d33efdebeea9bf28eac5a65a4d30e0310b1a37b9`;
  `ent5-pos-value-if-delivery-join` `1aed6d6be263899f04decd6508966f31199c6bbfd20db40c847d0eeaa587eb5e`;
  `ent5-neg-value-match-no-delivery` `22eba0419d73aaa20a3b21bf78dfba2b9adb9e9159f8f5ed595ee0d0d5e9354b`;
  `prv2-neg-complete-only-postcondition` `34ddc5f80b2a571b32775846419920ea92e7c41d6b5e60426304f982eda54995`;
  and `prv2-pos-postcondition-b-summary`
  `8d87708df6572fb90b09234ba9d51a3cd1ea05dc6f356f3131f5182776abc9fa`.
- CANONICAL RUNNER IDENTITY: approve changing only
  `tests/conformance/runner.py`'s `ACTIVE_SPEC_SHA256` from the outgoing v0.27
  digest to the approved v0.28 digest. The resulting complete runner SHA-256
  is `369fc2f100d679f9ce815087f9533315b95c81750e9d60d1660993f5ea290072`.
  Runner logic, collection, verdict interpretation, adapter, baseline, gate
  wiring, and invocation remain unchanged.
- SEMANTIC AND CONSUMER SCOPE: install the reviewed FN-9 surface, complete/U/B
  selected-exit proofs, bounded S7 sources, earlier-SCC postcondition
  publication, four closed result-receiver routes, `value_if`-only delivery,
  failure-atomic provenance publication, and the exact five-source 14/20/A10
  real-consumer migration. No general assignment equality, solver, induction,
  runtime fallback, new optimizer authority, alternate lowering, host/runtime
  ABI, or additional caller repair is approved.
- INSTALLED EVIDENCE: the additive result is 437 cases and 30 unchanged
  annotations, rule coverage 132/132, and native adapter
  `Pass=423 Fail=1 Skip=13`; the sole failure remains the pre-existing OWN-3
  `own3-pos-outlives-store` unsupported boundary.
- STABLE-PATH INSTALLATION: v0.28 remains only at `spec/kernel-spec.md`; the
  exact outgoing bytes are newly archived at immutable
  `spec/kernel-spec-v0.27.md`. No `spec/kernel-spec-v0.28.md` is created.
- boundary: exact specification/archive/protected/runner identity bytes plus
  the reviewed H0–H5 compiler and consumer implementation, generated data,
  active pins, approval chain, derivation binding, ordinary documentation,
  task closure, MCTS synchronization, full gates, and one atomic main squash.
  Stage 9a is ordinary compiler/tooling work under the ACTIVE plan; Stage 9b
  still requires its later independent exact specification and protected
  approval.

## 2026-08-15 — OWNER EXACT-BYTE APPROVAL: v0.29 strict no-claim partition

- owner: **the owner, in conversation, 2026-08-15** — “上述精确
  specification、archive、ledger、manifest、九个 source、wfgrep marker 和
  未来 runner identity；接受 frontend verifier 在批准前保持红色、批准后才
  实施并在原子激活前转绿的时序例外。” The reply followed the exact held
  packet and its repeated hard wait. It approves the named bytes and the
  disclosed post-approval implementation sequence; it does not waive any
  activation gate or permit a changed candidate byte.
- APPROVED SPECIFICATION: stable `spec/kernel-spec.md`, version v0.29,
  SHA-256
  `0b7aa8ccee958ba85613c51535165dcbf7ac12db556b2210d2f1aac0d39e6cc3`.
  The outgoing immutable v0.28 archive is byte-identical to the installed
  v0.28 authority and has SHA-256
  `08897c51f0ccb8e7d19edb558229b1ef17b33971cd291b701d4f270ee536ce09`.
  The approved derivation ledger has SHA-256
  `7f2b277c3bafa8d9448f4b16b9ba0066b26668beb804cc31ee05d5c655b22806`.
- PROTECTED CORPUS: approve the append-only manifest at SHA-256
  `2dbd4f4242f82142c4c39578d8ca3e70ca6528bc3c2f5c540d0d548ee8fc1ee2`;
  its exact nine-row suffix is
  `ebea9a792345d3b86e3f3d62b4f12631012c0083eb924ccf9fe137979ad7bbef`.
  Approve exactly these additive runnable source identities, with no existing
  case, row, annotation, verdict, status, rule citation, rename, deletion, or
  reorder:
  `clm3-pos-transitive-value-branch`
  `779b5b21aee3f5bd7c6d73d336d3045905ee872c3e7f461bc3085bbb05792614`;
  `clm3-neg-direct-unreachable-claim`
  `9f76ccc551c582c559ed2ca9e7d173f0e05775fb2333c91baaf2235594290f78`;
  `clm3-neg-generic-first-import`
  `cf36fd37b8ca3c972f14118e17c2971179f6a34fb429665e44814ce2c3806028`;
  `clm3-pos-upward-near-miss`
  `34d31132121e19da9185f40a24c35c7fec3baaafedd3728853747007a3533930`;
  `clm3-neg-mutual-scc-import`
  `df6b3048ffc9589c4cfc5b76ac2849bdbce0c0b97da432d284c7f3753b5b66fa`;
  `clm3-neg-generated-wrapper-check`
  `13a7d553dc07a0a89fa181091cdb7655dccf513f1b6c9930971d9aef753157ee`;
  `clm3-neg-body-check-bounds`
  `6abc4f3a4ad34561b8d75c04d75845c5c338a9db32f775a05afc3b01e67a94a0`;
  `clm3-neg-body-check-requires`
  `0be65125314e4ef0b2ea478f3eae3d139917971cc6727dd1f27a4de7e5de7cc7`;
  and `clm3-neg-transitive-check-summary`
  `be1445597569e920034c1d82c40fd6dd9ebc2ef88382d9f0009c417658ad5396`.
- AUTHENTIC CONSUMER: approve only the `deny_claims` prefix on wfgrep's
  `report_failure`, producing complete-file SHA-256
  `fb2f3b44160a947d7adca9fc9b5af851b446a7bcfc179ede4f8c689b21033904`.
  Its body, callers, output, error, cleanup, status, runtime checks, facts-off
  behavior, and non-upward claim boundary remain activation oracles.
- CANONICAL RUNNER IDENTITY: approve changing only
  `tests/conformance/runner.py`'s `ACTIVE_SPEC_SHA256` from the v0.28 digest to
  the approved v0.29 digest. The resulting complete runner SHA-256 is
  `aead0b55b7fe7f52cee36fac74025d46dba8d0de19654a62341ce695b0e62f3e`.
  Runner logic, collection, verdict interpretation, adapter, baseline, gate
  wiring, and invocation remain unchanged.
- SEMANTIC SCOPE: add the optional fixed `deny_claims` declaration prefix and
  CLM-3's finite outgoing concrete-call/SCC closure. Direct or imported claims
  reject deterministically; demanded protected leaves, ordinary requirements,
  and marked program-start requirements must discharge in the already-produced
  U view. The marker never propagates upward, reads no Stage 9a ClaimLedger as
  acceptance authority, and adds no runtime check, body, solver, fact source,
  effect, ABI field, foreign adapter, lowering, optimizer license, or serialized
  identity.
- SEQUENCING EXCEPTION: the owner explicitly accepts the disclosed red
  archive-to-candidate frontend verifier before implementation. Frontend,
  generated tables, semantics, and ordinary tests are implemented only after
  this approval; the production verifier and every complete activation gate
  must become green before the atomic activation. The projected corpus result
  `Pass=432 Fail=1 Skip=13` and exact Stage 9b diagnostics remain expectations
  until measured post-implementation.
- boundary: the exact held commit
  `4e4707317206a103cdb29d2f1d076d8f9807a90f` binds the approved candidate.
  Post-approval ordinary implementation, generated data, active pins, the
  chained `ACTIVE-SPEC:` line, lifecycle and authority closure, MCTS
  synchronization, complete gates, and one atomic main integration may now
  proceed. Any approved specification, archive, ledger, manifest, source,
  verdict, status, rule, wfgrep marker, or runner-identity byte change returns
  to exact review.

## Pre-approval specification digests, as found

The nine versions before exact-byte approval began have no approved digest;
none was ever recorded for them. These lines are a measurement, taken with
`shasum -a 256` on 2026-08-07 against the files as they stand in the
repository. They are **not** approvals and they do not retroactively approve
anything: they record what is on disk so `make check` can detect a released
specification changing underneath the project from now on. If one of these
files was already wrong before this measurement, this pins the wrong bytes —
that is the honest limit of a digest recorded after the fact, and it is why
these carry a different prefix from the approved chain above.

ARCHIVE-SPEC: v0.0 717d1e1025e42b9122e972cac44c7e3e55acab6c146d9b4152c599e37330520e
ARCHIVE-SPEC: v0.1 cf38fbf881450399ce41bc987369061d16a0321e1db713c9be233325ca83d310
ARCHIVE-SPEC: v0.2 1393aeca4faabe0bd73649c61250c2b36c12152490bd1272ec5d3e5c26f34d8c
ARCHIVE-SPEC: v0.3 771bdf907edfa01f40fca1fd97ea92a7e6fff072444e796c2e7da3311b7841b4
ARCHIVE-SPEC: v0.4 30baf9cefa1ab179f39a1d5a0a660e37a4bce42a762574e7eb4098bc2505f3e1
ARCHIVE-SPEC: v0.5 f41128b7e5cc7ecad1447ee0b45bdd3004d681c5048a08a5edc1d5fd0b8aa01a
ARCHIVE-SPEC: v0.6 95ae3e1eec48109e1c55c65c2a3e3ddecccd6192c30c0d034e5b7931f10e535e
ARCHIVE-SPEC: v0.7 212a2224d9d69ed58b0cb4cb3e8137572e2f06c6e1326698b1c6793ff0f04481
ARCHIVE-SPEC: v0.8 d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8

## 2026-08-07 — condition interpretation (computed digest, C1b)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: the stable-filename approval names a "computed SHA-256 digest" as a condition of adoption. Task 0039 implemented the const-fn form, measured it, and found it costs ~12s of constant evaluation PER CRATE (library build 1s -> 12s, compiler gate ~40s -> 87s) because a `const` is re-evaluated in every crate that reads it, and it requires `#![allow(long_running_const_eval)]` in five crate roots. The condition is satisfied in substance by the shipped form: the digest is computed from the embedded bytes at runtime (~1ms) and the `whitefoot-spec` gate plus a unit test reject any disagreement with the recorded constant, so an activation that installs a specification without moving the identity fails immediately. What the literal form would add is only that the constant is not transcribed at activation; what it costs is a permanent doubling of the gate. The condition's wording is therefore read as "the digest is verified by computation against the recorded value at every gate run", not "the constant is itself a compile-time computation".
- boundary: C1b's implementation only; every other condition of the stable-filename approval stands unchanged.
- evidence: task 0039's measurement, recorded in `docs/done/0039-spec-identity-integrity.md`.

## 2026-08-08 — approval (FLOOR-5 conformance verdict-meaning breaks)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: four cases whose meaning cannot survive the spelling batch, ruled on the R4 ladder (a check-time rejection promoted to unrepresentable keeps its concern in stronger form, and the case is repurposed rather than retired):
  (1) `fn2-neg-implicit-instantiation` — rejects a table operation that omits a required instantiation argument; after A1 no table operation carries one, so the error is unspellable. REPURPOSE as a positive pin of the derivation: the argument-free operation is valid and its type derives from its operands.
  (2) `x-typ-bool-cmp-result-as-int` — rejects binding a Bool result to an `own i32` let; A3 deletes the annotation the disagreement needed. REPURPOSE as a positive pin: the comparison result binds as Bool by derivation.
  (3) `x-match-give1-wrong-type` — its premise is the deleted declared mode/type, but its CONCERN (arms delivering disagreeing types) survives in the new GIVE-1 as the agreement judgment. REWRITE to test agreement: two arms delivering different types reject under GIVE-1. Same concern, stronger form.
  (4) `gram9-neg-nested-call` — GRAM-9's prohibition survives and nested CALLS remain spellable; only the arithmetic-shaped nesting loses its surface. Keep the case testing a nested call form that still exists; do not invent a parenthesization surface for infix, which the batch deliberately does not add.
  In every case the executor verifies the ruling against the migrated bytes before writing, and reports rather than proceeds if a ruling does not hold.
- boundary: those four cases and their manifest rows.
- evidence: exec-0038's round-1 report; lead verification of the FORM-2 brace-pair finding at `compiler/src/syntax/parser/finalize/engine.rs:479-484`.

## 2026-08-08 — ruling amendment (x-match-give1-wrong-type surface)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: the 2026-08-08 ruling said to rewrite `x-match-give1-wrong-type` so that two arms delivering different types reject under GIVE-1. Verified against the real bytes by the executor, that fails: the case's scrutinee is `ilt(...)`, which is Bool, and the candidate's GRAM-6 makes a Bool-scrutinee `match` a hard error citing GRAM-6 at the scrutinee node — so the rewrite would assert GIVE-1 and earn GRAM-6, failing its own citation. AMENDED: keep the `match` form and give it a genuine ENUM scrutinee, so GRAM-6 forms no candidate and GIVE-1 genuinely owns the rejection. Rejected alternative: rewriting it as a `value_if`, which is the natural migration of these bytes but drops GIVE-1 agreement coverage in the match form entirely — the four new cases all sit on `if`, so match-form coverage would otherwise vanish from the corpus.
- ALSO MANDATED, from the same finding: the migration must ASSERT zero surviving Bool-scrutinee matches rather than rely on the parse. All 262 `True()`-arm matches become `if`/`else`; a missed one does not fail to parse — it becomes a GRAM-6 rejection, which is a SILENT verdict change.
- boundary: that one case's surface, plus the migration assertion.
- evidence: executor verification against the migrated bytes; the same executor confirmed all nine migration figures by two independent methods and resolved the 378/379 discrepancy to the unique argument-free table-op call site.

## 2026-08-08 — ruling (prelude variant constructors join the retained-argument class)
- owner: lead ruling under the standing delegation; the owner's pending byte-exact approval of the v0.23 candidate covers the resulting bytes
- reason: A3 deletes the body-`let` annotation, and for the nullary prelude variant constructions that annotation was the SOLE supply of the binding's type. `check_construct` resolves the prelude `Option`/`Result` constructors from the expectation and never consults written type arguments, and `let_stmt` is the only construct position whose expectation came from an annotation (`set` uses the target type, call arguments use declared parameter types). So after A3 `let x = None();` has no legal spelling anywhere: `None()` carries no payload to derive from, and `Option` is not in TYPE-5's retained-argument class. Scope measured: 1 corpus site and 4 compiler inline fixtures — small in count, total in kind.
  RULED: option (a) — the prelude variant constructors join TYPE-5's retained-argument class and `check_construct` honours written type arguments (`None<buffer<u8>>()`, `Some<u8>(value: v)`). This is what the class's own rationale already argues: it exists for the cases where no operand can supply the type, which is exactly `None()`. Rejected: deriving the nominal from the payload, which cannot handle `None()` at all; and keeping an expected-type channel for the `let` initializer, which contradicts TYPE-5's own "no binder's type depends on an expected type" in the same paragraph.
  NOTE FOR THE RECORD: review finding F3 certified the retained-argument class "total against the complete operation table". That certification was sound and its SCOPE was the gap — prelude variant constructors are not operations, so they were never in the set being checked. Three review rounds and seven sweep patterns missed this class for that reason.
- boundary: TYPE-5's retained-argument class and the `check_construct` argument path; the candidate digest and its three pins re-key accordingly.
- evidence: exec-0038's trace of where the `expected` hint flows, with the sites at semantic/check/expressions.rs:820,843 and the five affected fixtures.

## 2026-08-08 — ruling (O1 evaluation basis; no split)
- owner: stated in session
- reason: (1) EVALUATION BASIS CORRECTED. The lead had framed the infix-comparison mechanism choice around owner preference for or against a marker. The owner corrects: the decision follows the project's written principles, not taste — a spelling is good when it is clean, not hacky, and better achieves the goals (token economy, a writer's natural shape). Applied: R1 rejects a construct that serves neither P0 nor P1 but only the parser; the residue axis flags that a marker relocates rather than removes the defect the reviewer already named (`ilt`/`igt` as the one piece of surface whose shape is explained by the implementation rather than the language — under a marker it is the generic-call surface instead); W2 weights the ~1533 type-argument sites' token cost without gating on it; W1 favours the shape a writer produces without knowing the ambiguity exists. Ranking therefore: zero-marker preferred IF sound; a marker is honest machinery and acceptable if the adversarial review shows the token-class mechanism is not.
  (2) NO SPLIT of the FLOOR-5 batch: the corpus is machine-checked by parse plus the FORM-2 canonical audit, so a migration defect fails loudly rather than silently; the residual risk lives in the specification text, where the defect-discovery rate is falling across sweeps. Migrating 3700-odd sites twice is the larger cost.
- boundary: the O1 mechanism decision and batch composition; the mechanism itself still awaits the adversarial judgment.
- (3) CORRECTNESS OVER CHURN, stated as a standing priority: defects are cleared rather than worked around or minimized, and the size of the resulting change is not a reason to prefer a smaller repair. Applied immediately to the open items — the prelude-constructor hole takes the mandatory retained-argument form covering both `let` and `give` even though it re-keys the candidate digest and all three pins; any residual the O1 mechanism leaves is closed rather than tolerated as a wart.
- evidence: owner statements 2026-08-08 ("我其实没有喜欢不喜欢…这些原则和目标都写在那里…不分批也还好，语料本来也是要编译器检查的" and "缺陷肯定是要清掉，改动大没事，要正确").

## 2026-08-08 — ruling (O1 mechanism decided; deferred to the next version)
- owner: approved in session ("同意推迟")
- reason: adversarial judgment on making `<`/`>` infix. (1) THE LEAD'S MECHANISM IS UNSOUND as stated — `unit` is simultaneously a TYPE-1 primitive and a FORM-5 literal, and CONST-1 admits a bare IDENT as a const targ from two sources, so one token after `<` never suffices; the lead's ordering rule is rejected outright by all three lenses (vacuous where the collision lives — 5 of 6 const-generic declarations in the repository are const-only — and it leaks into declarations). (2) THE OWNER'S RULING IS ACHIEVABLE WITH ZERO MARKERS by factoring one level deeper: consume `<` first, then decide on the two tokens after it. A targ is always followed by `,` or `>`; a comparison operand is always followed by FOLLOW(expr) = {`;`, `else`, `because`, `{`} — structurally disjoint. Measured 77 productions / 98 decisions / 0 clashes on a harness reusing the repository's own generator. (3) THE MARKER IS SIMPLER, stated plainly as a fact: +1 grammar line against +8 productions and +14 decisions, no node-kind fragmentation. It is nonetheless rejected on principle: a marker required in call position but not type position is a context-dependent spelling of one construct (`Result<i32, E>` vs `f::<i32>(x)`), which META-2 forbids; and its cost is permanent and paid by every writer on every generic call, whereas the factoring is paid once, moving complexity from the language into the compiler. (4) HARD VERIFICATION CONDITION: the result rests on one out-of-tree harness and must be re-derived IN-TREE with `whitefoot-grammar-tables` at zero conflicts, with five named programs still parsing, before any prose is drafted; failing that the ruling reverts to the marker without further debate.
- (5) BATCH PLACEMENT: NOT folded into FLOOR-5. It is a different kind of change — it restructures `expr`, amends GRAM-1's node-kind law, introduces the first position-sensitive FORM-2 rule, and makes `>` a lexer invariant, invalidating the batch's approved 69-production count and the O1/O5/O9/R2/EX-1 dispositions. FLOOR-5 lands as approved; `<`/`>` infix opens the next version. FOUR BINDING CONDITIONS so the deferral costs nothing: restate O1 as "mechanism ruled, deferred for batch hygiene" rather than as settled retention; keep `<` and `>` reserved against any other use; do not close FORM-2's attachment sets, O5, O9, or EX-1's bytes in a form presuming no operator ever joins those sets; and make the corpus migration scripted and repeatable so the second pass is a re-run rather than a re-do.
- evidence: owner instruction "嗯，同意推迟。开工吧" (2026-08-08) after the judgment was presented with the marker's simplicity stated.

## 2026-08-08 — ruling (canonical renderer and the migration tool's home)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: the delta's §5 says migration is "printer-driven", but the compiler has no printer — `canonical/` is an auditor whose only public entry answers "are these bytes canonical?". As written the migration must compute every byte of spacing and indentation itself, including the `} else {` join line no v0.22 production produced. RULED: BUILD THE RENDERER. It is a modest addition over existing machinery (`build_gap_styles` already computes the canonical layout per token boundary; `bytes_match` already carries indentation off `format_depth`), and it makes the migration canonical BY CONSTRUCTION — a textual pre-pass that need only parse, plus a render pass — rather than byte-perfect by hand. Three grounds: the owner's standing correctness-over-churn priority; binding condition 4 of the O1 deferral, which requires the migration to be re-runnable because the next version re-runs it; and at least two further spelling batches queued (`<`/`>` infix, arithmetic-mode dissolution). This does not reverse the owner's 2026-08-07 process ruling, which removed the REQUIREMENT that migration be printer-driven; it did not forbid building one, and the calculus has since changed.
  HOMES, no new repository root entry: the renderer joins `compiler/src/syntax/parser/finalize/canonical/` beside the auditor; the migration tool is a bin under `compiler/src/bin/`, the precedent the grammar-table generator set. The renderer ships WIRED TO A GATE per the hygiene rule: `make check` verifies that every canonical corpus file re-renders to itself byte-for-byte, so the corpus is canonical by construction rather than by assertion.
- boundary: those two homes and that gate; no new root entry, no Python in the compiler path.
- evidence: exec-0038's measurement that `canonical/` has no emitter and that the existing gap-style and depth machinery supplies what a renderer needs.

## 2026-08-08 — ruling (box_new nominal interning: fix the checker, not the language)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: after A3 deletes the annotation, `box_new(v)` fails because `CheckedNominalKind::Box` instances are interned at exactly one site and only from a WRITTEN `box<T>` type, while checking is `&self` and so can only look one up. Reproduced with a control pair whose only difference is whether `box<T>` is spelled elsewhere in the unit; the failing program yields `InvalidResolution`, a COMPILER failure reported where the workflow forbids reporting compiler capability as invalid source. Measured scope: 4 corpus files call `box_new`, 2 of which would lose their only `box<T>` spelling.
  RULED (b): INTERN BOX NOMINALS LAZILY DURING CHECKING. Rejected (a) — adding `box_new` to TYPE-5's retained-argument class so the writer spells `box_new<u64>(value)`: the type here is fully derivable from the operand, unlike the `None()` case where no information exists, so (a) would let an implementation limitation select language behaviour, which CLAUDE.md forbids outright; it would also re-key the candidate and all three pins and force STOR-2's amended "`box_new(v)` returns `own box<T>` for `v`'s exact type T" to move, when that sentence is the correct answer. Rejected (c) — a pre-pass deriving operand types — as not closing the gap, since an operand's type can itself be derived.
  THE DISTINCTION IS THE PRECEDENT: the `None()` blocker was a language gap (no information available) and was repaired in the language; this is an implementation limitation (information available, structure cannot record it) and is repaired in the compiler. Complexity moves out of the language, matching the O1 ruling's direction.
- boundary: the nominal-interning path and `check_box_new`; no delta change, no candidate re-key, no pin move. Sequenced after M2 and before M3 completes, since the migration produces the failing shape in two files.
- evidence: exec-0038's control-pair reproduction and site measurement, recorded in docs/ongoing/0038-floor5-semantic-and-migration.md.

## 2026-08-08 — ruling (the 20 pre-semantic reject cases in the FLOOR-5 migration)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: 20 of the 420 corpus files are `expect: reject` at or before parsing (four do not even lex), so a parse-and-render migration cannot process them by construction. Re-rendering one would silently destroy it — `x-form-form2-tab-indent.wf` exists to be rejected for tab indentation and canonical rendering fixes indentation, so the migrated case would test nothing and still be green, which no gate catches. This is a property of the pre-semantic rejects only: the manifest's other 177 reject cases are rejected after parsing and migrate normally.
  RULED, and neither blanket option is safe: leaving them at v0.22 spelling is NOT automatically safe either, because a v0.22 annotation is itself a GRAMMAR error under v0.23, so a case whose recorded rejection fires AT the grammar stage may now cite the annotation's rule instead of its own — a silent verdict change of exactly the kind the batch must not make.
  THE RULE: verify each of the 20 individually against the v0.23 compiler and compare the CITED RULE against its manifest row. (a) A case that still rejects for its recorded rule — those failing lexically or at FORM-2, before the annotation can matter — stays at v0.22 spelling, unmigrated, with the reason recorded per case. (b) A case whose citation changes is RESTATED against v0.23 so it rejects for its own rule again, keeping its concern. Do NOT migrate them textually without rendering: for the cases that test a byte-level property, a textual edit risks perturbing the very property under test.
  The verification is mechanical — run the compiler on the unmigrated file and compare the citation to the manifest — so this is a measurement per case, not a judgement per case.
- boundary: those 20 cases and their manifest rows; the other 400 migrate normally.
- evidence: exec-0038's `--check` run over all 420 files (400 parse and render), and its verification that all 20 are manifest `reject` rows rather than an inference from filenames.

## 2026-08-08 — ruling (reject-err2-nonexhaustive: change of witness)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: the case records ERR-2 (match exhaustiveness) and now cites a different rule. Its scrutinee is Bool, and under v0.23 a Bool-scrutinee match is rejected by GRAM-6 before exhaustiveness is ever consulted, so ERR-2's concern cannot be reached through a Bool match at all. RULED: RESTATE with a source ENUM scrutinee — a user enum whose match omits a variant — and verify it cites ERR-2 again. This is a change of WITNESS rather than of spelling, and therefore a larger step than the seven annotation deletions, but the concern, the rule and the verdict are unchanged; the Bool scrutinee was only a convenient witness. Leaving it would mean the case still rejects, still passes its must-reject check, and ERR-2 loses its negative coverage silently — the failure mode this batch has been eliminating case by case. If other ERR-2 negative cases exist that is a bonus, not a substitute: coverage attributed to this case must not vanish without a replacement.
  SECOND CASE, no action: `type5-neg-match-non-enum.wf` has a scalar scrutinee, so GRAM-6 never applies and TYPE-5 still fires as recorded. Leave it.
  NOTE ON HOW BOTH WERE FOUND: both parse and fail semantically, so no parse-based sweep could surface them. Only the mandatory zero-surviving-Bool-match assertion did, which is the argument for having required an assertion rather than a report.
- boundary: that one case's source and manifest row.
- evidence: exec-0038's assertion run over migrated output, with the citation observed rather than inferred.

## 2026-08-08 — rulings (M3b's four findings)
- owner: lead rulings under the standing delegation; owner ratification pending
- reason:
  (1) `form2-neg-noncanonical-ws.wf` — its entire content under test is an isolated indentation, which canonical rendering normalizes away, so the migrated file compiles cleanly where its manifest demands a FORM-2 rejection. RULED: leave unmigrated at v0.22 spelling, verify it still cites FORM-2 (its violation is caught at the canonical audit, before the annotation can mask it), and record the reason per case — the same disposition as the 13 pre-semantic cases. GENERALIZED, because this one was missed by the earlier sweep: the exclusion set is not "files the tool's --check refuses" but "cases whose recorded rule is a byte-level property that rendering normalizes" — enumerate them from the MANIFEST by rule (FORM-2, FORM-4 and any other layout or lexical rule), not from tool behaviour.
  (2) Two cases whose concern died with the deleted bytes. `x-typ-bool-cmp-result-as-int` is ALREADY ruled (2026-08-08: repurpose as a positive pin of the derivation) and is M4's work, not a new question. `fn2-neg-eeq-implicit-type` is a NEW member of the same class — v0.22 rejected the missing type argument, v0.23 deletes that argument by design — and takes the same disposition: REPURPOSE as a positive pin that the argument-free operation is valid and its type derives from its operands.
  (3) Six cases reject under a different rule with the verdict kind preserved. RULED per case, by the rule already established for the 20: restate minimally so the recorded rule fires again, keeping the witness where possible; where the concern is genuinely unreachable under v0.23, it becomes a finding-(2)-class repurpose with its own ledger line rather than a silent citation change. Report each with recorded rule, observed rule and disposition.
  (4) `ent5-pos-join-keeps-common-bound` newly REJECTING with TYPE-6 DeclarationCollision is NOT accepted as a disposition: a positive case that starts rejecting is a verdict change and is presumed a migration defect until shown otherwise. Investigate and fix the cause; do not restate the case to match the new behaviour.
  (5) Finding 4 is not a ruling but a DEFECT: infix has no checker path in `return` position, so 38 of the 52 remaining adapter failures are `InvalidCanonicalTree` — an internal compiler failure where the workflow requires a source rejection or an accepted program. OP-1 (ii) landed for the `let` path only. Fix it, and enumerate the expression positions exhaustively rather than fixing `return` alone — a predecessor established that the grammar has nine, and infix must be checked at every one.
- boundary: those cases and the infix checker path; no delta change and no candidate re-key.
- evidence: exec-0038g's M3b run, with each finding carrying a reproduction and the three oracles predicted before measurement.

## 2026-08-08 — ruling (the requires-block `let` under A3)
- owner: lead ruling under the standing delegation; owner ratification pending
- reason: after A3 there is no way to write a requires-block `let` — with the annotation it no longer parses (GRAM-4), without it `validate_requires_let` fails `InvalidCanonicalTree` because it requires a `Mode` child and then a `Type` child, which are exactly the two parts A3 deletes. All 13 remaining adapter `InvalidCanonicalTree` failures are this. Four controls isolate it, and infix inside a requires block already works, so it is a distinct defect sharing a symptom.
  RULED, and neither option the executor named is needed: the structural pass simply STOPS REQUIRING the two children that no longer exist. FN-8's own text already fixes the mode — "each let introduces a fresh clause-local OWN copy value" — so no mode derivation is owed, and the type comes from ordinary checking of the initializer exactly as a body `let`'s does. The FN-8 structural pass validates SHAPE (zero or more lets with an ordinary right-hand side, then exactly one check) and does not need a type to do it; the new shape is `let IDENT = rhs;`.
  REJECTED: reordering validation after checking, because FN-8's structural pass is deliberately early so a shape violation is reported before any child semantic error can win, and reordering would change which diagnostic wins — an observable behaviour change. ALSO REJECTED: a separate local derivation for the requires-block binding, because two derivations that must agree is the defect pattern this project has been replacing with shared code all week.
- boundary: `validate_requires_let` and its structural checks; no delta change, no candidate re-key, no diagnostic reordering.
- evidence: exec's four-control isolation, and FN-8's own clause-local sentence in the active specification.

## 2026-08-08 — correction to the requires-block `let` ruling above
- owner: lead correction under the standing delegation; owner ratification pending
- reason: the ruling recorded above was wrong in its stated ground and incomplete in its scope. Both were found by the executing agent and reported rather than absorbed.
  WRONG GROUND: the ruling named `validate_requires_let` as FN-8's early structural pass and rejected reordering on the theory that it would change which diagnostic wins. The early shape pass is in fact `check_requires_blocks` (`compiler/src/resolution/engine/admission.rs`), which runs during resolution and already reads neither annotation; `requires.rs::validate_requires_*` is a later semantic-subset pass. The conclusion held — shape enforcement lives in a different pass and was never at risk — but for a different reason than the one given, so the rejection of "reorder" was argued from the wrong function.
  INCOMPLETE SCOPE: the ruling asserted the deleted `Type` child owed nothing but typing. It was doing double duty. It also carried the sole enforcement of FN-8's copy restriction (`if !self.is_copy_type(self.parse_type(ty)?)?`), so removing the read removed that rejection with nothing replacing it, while FN-8 still requires the clause local to be an "own COPY value". Reachable and measured: `let xs = array_new<i32, 4>(0_i32);` and `let raised = x +checked 1_i32;` both exit 0, with `array<i32,4>` non-copy per `is_copy_type` (`semantic/check/nominals.rs:90`) and `Result<i32, Overflow>` non-copy per `CheckedNominal::is_copy` (`semantic/model.rs:288`). Reachable non-copy families are `array_new`, the `checked` arithmetic family, and the partial `cvt` family; `slice_of` is not reachable, needing a borrow operand the subset pass rejects and a region a clause cannot bring into scope.
  RULED: the compiler is the wrong side of this discrepancy and the specification text stands. The copy judgment is re-established from the type `check_statement` already derives under [TYPE-5] on the next line of `check_requires` — reusing the checker's own derived answer, which is not the second derivation the original ruling rejected.
- also ruled, from the same round: a second pre-existing defect in the same pass is legitimately in scope, having been proven by rebuild on both sides rather than argued. `validate_requires_condition` read only the expression's own `atom` and never the operator row or right operand, because `expr := atom infix_tail?`, so a trapping operator or a `move`, borrow, or subscripted operand escaped in the requires CHECK position (`check a <= xs[1_u64] else trap …` exited 0 before the fix and rejects FN-8 after). Admission now shares `CheckedIntegerOperation::traps` (`semantic/model.rs:412`) with the ordinary checker, because the bare `+ - * / %` forms carry the trapping mode with no `.trap` suffix and a spelling filter cannot see them.
- boundary: `validate_requires_*` and its probes; no spec, candidate, delta, or manifest change. `fn8-neg-requires-eeq-payload-enum` is explicitly excluded — it shares a pre-existing OP-1/OWN-1 precedence question with `op1-neg-eeq-payload-enum`, which has no requires block and was already failing.
- evidence: adapter 28 → 16 fail with nothing newly failing (Pass=359/28/14 → 371/16/14); lib 308/262 → 318/253; every FN-8-family negative still rejecting with its recorded rule and kind, set diff empty in both directions; `coverage (kernel-spec-v0.23-candidate.md): 128/128 rules covered`. All figures recomputed by the executor on branch `task/0038-floor5-semantic-and-migration`.

## 2026-08-08 — correction to the entry above: the escape was one operand form, not four
- owner: lead correction; the executing agent retracted its own commit message and I had already relayed the inflated version into the entry above, so the error is mine to fix here.
- what the entry above says: that "a trapping operator or a `move`, borrow, or subscripted operand escaped in the requires CHECK position". Three of those four are wrong.
- measured at `efb5242`, before any of the round's commits:
  - SUBSCRIPTED operand: real, and the only one. `check a <= xs[1_u64] else trap "…";` inside a requires block compiled clean (exit 0) and rejects `FN-8 InvalidRequires` after. Cause: `expr := atom infix_tail?`, so `validate_requires_condition` read the expression's own `atom`, validated it, returned Ok, and never read the operator or the tail's atom.
  - TRAPPING OPERATOR: NOT a hole, provably closed before the change. `check x + 1_i32 else trap …` rejects `OP-5 InvalidCheckCondition` at exit 1, because a check condition must be Bool and trapping arithmetic returns an integer, so OP-5 fires first.
  - `move` operand: NOT demonstrated. The probe used a u64, so `OWN-1 MoveOfCopy` rejected it for being a copy type, which shows nothing about an FN-8 escape.
  - BORROW operand: NOT measured. The probe did not parse (GRAM-3, malformed parameter syntax). Recorded as unmeasured rather than guessed.
- the `traps()` addition in `8ccd4d8` is nevertheless REQUIRED rather than prospective, and for a reason the retraction understates: the same commit newly admits the infix spelling in a clause `let`, and the pre-existing subset filter recognizes only a `.trap` suffix and the `{buffer_new, box_new, arena_new}` names. Bare `+ - * / %` carry the trapping mode with NO suffix, so admitting infix without sharing `CheckedIntegerOperation::traps` would have opened a new hole in the same commit — FN-8 admits only a "non-trapping, total" row. Verified by the executor's own probe: `let raised = x + 1_i32;` rejects `FN-8 InvalidRequires` with the shared predicate in place.
- standing lesson, recorded because this is the second time in one session: a peer's report is a claim, and relaying it into this ledger converts a claim into an authority. The subscript escape carried a rebuild-on-both-sides reproduction and survived; the other three carried prose and did not. Only the reproduced half should have been written here.

## 2026-08-08 — rulings (M3c's twelve non-migration residuals)
- owner: lead rulings under the standing overnight delegation; owner ratification pending
- reason: the M3c inline-fixture migration moved the library gate 319/253 -> 533/39 with 214 tests fixed and zero newly failing, and correctly refused to close the remaining 37, none of which is a fixture-spelling problem. Ruled from its per-test enumeration, group by group. The governing principle throughout is the one the conformance dispositions established earlier the same day: ask whether the newly cited rule is the test's SUBJECT or an accident of what fired first, and restate the source so the recorded rule fires again rather than editing the expectation.
  (a) `slice_of` still demanding its deleted arguments, 13 tests — the compiler is the wrong side and the specification stands; tracked and assigned separately. Its citation of FN-2, which [DIAG-1] reserves for a user-generic call, is not to be repaired inside that fix.
  (b) `if`/`else` branch blocks not opening declaration scopes, 14 tests — ALREADY FIXED at `8dc6a50` on the integration trunk by a concurrent unit that found the identical defect from the conformance side. M3c's base predates it, verified both ways. No ruling owed; a rebase clears it. If any of the 14 survives the rebase the two defects are not the same one, which would be a finding.
  (c) two pre-existing capability gaps (`Unsupported { OwnershipJoin }`, `Unsupported { RegionsAndBorrows }`) — an unimplemented feature is not a source-language rejection and must not rewrite a normative expectation. The tests stay failing with the reason recorded; note that the OwnershipJoin gap is HIDING a negative, since that case is expected to reject and never reaches its rejection.
  (d1) `operation_call_shapes_keep_their_exact_rule_owners` — RESTATE both assertions onto an operation row that keeps its callee name. The first has no v0.23 expression on a respelled row, because [OP-7]'s one-spelling rule moves the 20 respelled rows out of the callee-name inventory entirely, but the CONCERN — a wrongly spelled operation call earning its exact rule — is still expressible on any row that keeps its name. Restating preserves coverage; removing the assertion deletes it.
  (d2) `driver::tests::compiler_independent_negative_cases_keep_their_semantic_rule` — NOT disposable and NOT to be derived from the manifest. The hard-coded table is a deliberate second witness whose whole purpose is to catch a manifest edit, so deriving it would destroy what it exists for. Its failure is a SYMPTOM of the tracked FN-2/[DIAG-1] discrepancy, which is why the manifest row is already `pending`; it clears when that discrepancy is resolved and stays failing with a recorded reason until then.
  (d3) `result_construction_…` reaching TYPE-5 TypeMismatch — same cause as the adapter's `x-give-result-aggregate`; folded into that investigation rather than treated as its own item.
  (d4) `region_bearing_buffer_content_rejects_under_stor5` reaching OP-1 — RESTATE so STOR-5 fires again, exactly as the six masked conformance cases were restated. Verify per case that STOR-5 is what the specification assigns to the violation that remains; if the concern cannot be restated at all, that is a finding to report rather than an expectation to edit.
  (e) the two coverage fixtures that lost the only form producing a role — RESTATE onto a surviving position. `LexicalUseRole::TypeRegion` came only from a deleted `let` annotation, and signatures keep their written types, so a signature-borne `slice<'r, T>` restores it. For `LexicalUseRole::Type` on `HostString`, find the surviving position that produces the role; if NO position in v0.23 produces it, that is a finding about the resolver's role model and must be reported, not papered over by deleting the assertion.
  (f) the two tests built from `OPERATION_FAMILIES` — their shared premise is now false BY DESIGN, since [OP-1] states that infix resolution consults no name domain and an operator token is never a declaration, callee IDENT, or OPNAME, so the 20 respelled families have no lexical use at all. RE-SCOPE into two assertions rather than migrating or deleting: the callee-path assertion runs over the families that keep a name, and a new assertion states that a respelled family has no lexical use. That converts a broken test into coverage of the property the respelling introduced.
  (g) the one fixture left unmigrated behind (a) — lands with that fix, as reported. A fixture migration with no passing test to verify it is what this batch has been avoiding.
- boundary: `compiler/src` tests and fixtures only. No specification byte, no manifest row, no conformance case, and no edited expectation anywhere in this group.
- evidence: M3c's per-test enumeration with reproductions and controls on `task/0038-m3c-inline-fixtures`; the (b) fix verified reachable from the trunk and unreachable from M3c's base by `git log --oneline <ref> --grep`.

## 2026-08-08 — ruling (the FN-2/TYPE-5 citation defect is one defect with two symptoms)
- owner: lead ruling under the standing overnight delegation; owner ratification pending
- reason: two units reached the same rule from unrelated directions — the conformance dispositions from `fn2-neg-eeq-implicit-type`, and the M3c fixture migration from `slice_of` — and each reported it as its own blocker. Read together against the specification they are ONE defect, and the specification is unambiguous, so this is not a spec question to investigate but a compiler defect to fix.
  THE AUTHORITY, verified by the lead's own command against the candidate rather than relayed: "The cited rule is the rule selected by the callee's class: [FN-2] for a user-generic call, [SYS-2] for a system operation's region arguments, and, for a table operation, the rule [OP-2] selects — OP-1 or TYPE-5." Citation is driven by WHAT IS BEING CALLED.
  THE MEASURED BEHAVIOUR IS INVERTED IN BOTH DIRECTIONS. `slice_of` is a table operation — it carries its own operation-table row over `array<T, N>` and `buffer<T>` — so a rejection there must cite OP-1 or TYPE-5, and the compiler cites FN-2. A user-generic instantiation must cite FN-2, and the compiler cites TYPE-5 for a missing, a wrong-count, and a wrong-kind argument alike, locating the wrong-kind case at the `targ` rather than at the `call`. The single underlying error is that the compiler selects the cited rule from the KIND OF ARGUMENT PROBLEM instead of from the callee's class.
  RULED: the specification stands and the compiler is repaired. Implementation convenience never selects language behaviour, and a spec/compiler discrepancy stops the affected work for investigation — that investigation is now complete and its answer is that no spec change is owed.
- separability, which matters for sequencing: `slice_of` carries TWO distinct defects and only one is a citation. The primary one is an ACCEPTANCE defect — after A1 deletes its arguments, `slice_of(&'v data)` is a program the specification accepts and the compiler rejects; the wrong FN-2 citation is visible only because of that rejection and disappears with it. The user-generic side is PURELY a citation defect: the program is correctly rejected and only the cited rule and node are wrong. So the acceptance fix and the citation fix are separate changes and are tracked separately; neither is to be folded into the other.
- boundary: compiler citation selection and `slice_of` argument admission. No specification byte, no manifest row, no conformance case.
- evidence: `slice_of`'s operation-table row and DIAG-1's callee-class sentence, both read directly from the v0.23 candidate on `task/0038-floor5-semantic-and-migration`; the two units' reproductions with controls in the round records.

## 2026-08-08 — ruling (`slice_of_keeps_nonflat_element_arguments_in_the_op1_domain`)
- owner: lead ruling under the standing overnight delegation; owner ratification pending
- reason: the test's subject was a WRITTEN nonflat element argument, and A1 deletes `slice_of`'s written arguments, so that subject has no v0.23 expression. The executor then established that the derived path cannot reach the same violation either: a non-copy struct element, a generic element, a nested array element, and an `array_new` of a struct are each rejected earlier by TYPE-2 or OP-1 on the array type itself, and in every case a control with the `slice_of` line DELETED fails identically — which is what shows the earlier rejection is not caused by `slice_of`. `array<T, N>` and `buffer<T>` therefore already require a flat T, and the element reaching `slice_of` is flat by construction.
  RULED, and the two halves go different ways. The compiler's derived-element OP-1 BRANCH STAYS: a source rejection is correct if the case is ever reachable, and "could not be shown reachable" is not "proven unreachable". The TEST IS RETIRED: a test that cannot be written for an unreachable branch is not coverage, and a permanently failing test is worse than none because it becomes noise that masks real failures. No coverage is lost, because the surviving OP-1 concern — that a written argument is now itself the rejection — is already carried by the new `slice_of_derives_its_region_and_rejects_a_written_argument`, which discriminates (the pre-fix binary gives the opposite verdict on both its fixtures).
  REJECTED: restating it onto another fixture. The executor tried that first and its own control caught the result passing for the wrong reason, which is the outcome this batch has repeatedly paid to avoid.
  REQUIRED with the retirement: a note at the branch site in the compiler recording that the branch is deliberately untested and why, so that widening `array<T, N>`'s element types re-opens the question instead of leaving a silent orphan.
- boundary: one library test retired, one compiler comment added. No specification byte, no manifest row, no conformance case, no compiler behaviour change.
- evidence: four probes with matched line-deleted controls, in the round record on `task/0038-m3c-inline-fixtures`.

## 2026-08-08 — ruling (STOR-5 versus OP-1 on a `buffer_new` region-bearing element)
- owner: lead ruling under the standing overnight delegation; owner ratification pending
- reason: the executor moved a STOR-5 assertion from `buffer_new` to `box_new` on the ground that "[STOR-5]'s own sentence names `box_new` and `arena_new`, not `buffer_new`". That reading is half right, and the half it misses is what decides the disposition. STOR-5 carries TWO relevant sentences. Its general prohibition explicitly enumerates "struct field, enum variant payload, `array`/`buffer` element, or `box`/`arena` content", so STOR-5 DOES own a buffer element. Only its SUBSTITUTION sentence — the one fixing the diagnostic anchor at an operation call's `targ` — names `box_new` and `arena_new` alone, and that is the sentence the candidate modified when `box_new` lost its `targ`.
  What actually changed is a consequence of [OP-9] deleting `buffer_new`'s written argument, not of any STOR-5 edit: the element is now derived from the fill operand, and the flat-element requirement rejects a region-bearing fill citing OP-1 before STOR-5 is reached. A region-bearing buffer element is therefore STILL REJECTED — the accepted-program set is unchanged — so this is a CITATION MOVE, not a fourth narrowing, and it does not disturb the candidate's §5 accounting of one respelling plus three deliberate narrowings.
  RULED: the move of the assertion to `box_new` STANDS, since the rule and kind fire there at the operand atom the rule names, observed rather than inferred. No test is owed for STOR-5 reaching a `buffer_new` element, because a stricter rule subsumes that path — the same disposition as the unreachable nonflat-element branch ruled earlier today, and for the same reason: a test that cannot be written for an unreachable path is not coverage.
  REQUIRED, and this is the part that outlives the test: the citation move must be NAMED in the owner review packet. The candidate's delta records the STOR-5 substitution edit and records `buffer_new` losing its argument, but nowhere states that the two together move a region-bearing buffer element's citation from STOR-5 to OP-1. A reviewer approving bytes should be told which rule now rejects what, even when the accepted set is untouched.
- boundary: one library assertion relocated; a note owed to the review packet. No specification byte, no manifest row, no conformance case, no acceptance change.
- evidence: v0.22 STOR-5 at `spec/kernel-spec-v0.22.md:299`, both sentences read directly by the lead; the executor's observed verdicts on the relocated assertion.
## 2026-08-08 — ruling (M3b's ten conformance dispositions, carried out and verified)
- owner: lead ruling under the owner's standing overnight delegation; owner ratification pending
- reason: the 2026-08-08 M3b rulings above are hereby CARRIED OUT, with each disposition verified against the specification before it was written rather than against what the compiler prints. THE COUNT IS TEN, not the nine relayed in the executor brief: finding 1 (one case) + finding 2 (two cases) + finding 3 (six cases) + finding 4 (one case). Recomputed from the adapter's own failure list — 28 baseline failures = 13 requires-block cases (a separate branch's defect), the 10 here, and 5 out-of-scope (`fn1-pos-returned-slice-inputs-run`, `fn1-pos-returned-slice-const-run`, `x-give-result-aggregate`, `own3-pos-outlives-store`, `own5-neg-slice-value-match`).
  (a) THE CAUSE OF TWO FINDINGS WAS ONE COMPILER DEFECT, not a migration defect. `ScopeBuild` opened a lexical scope for `loop_stmt`, `region_stmt`, and `arm`, and none for `if_stmt` or `value_if`, so every branch `let` declared into the enclosing block. [GRAM-4] hangs both `stmt*` sequences off one conditional node, which is why a walk keyed on child productions cannot separate them; the brace pairs already on the node can. Fixed at `compiler/src/resolution/scopes.rs`. This closes finding 4 (`ent5-pos-join-keeps-common-bound` compiles, exit 0) AND dissolves one member of finding 3: `ent2-neg-expired-spelling-inherits-nothing` cites its recorded OP-4 again, with NO manifest change. The brief's instruction to restate that row OP-4 -> TYPE-6 is REFUSED and must not be revived: writing it would have recorded a compiler defect as normative expectation. [TYPE-6] admits both programs verbatim — "Disjoint expired lexical scopes may reuse an ordinary value or label spelling" — while still forbidding the live shadow, which a control confirms is still rejected.
  (b) FINDING 1 RESTATED, manifest row unchanged. `form2-neg-noncanonical-ws` keeps the migrated line with its four-space indentation restored and rejects `CanonicalSource/Source [FORM-2]`, observed. The 2026-08-08 ruling's "leave unmigrated" disposition does not hold and is superseded: unmigrated the file cites GRAM-4, because a v0.22 annotation is itself a v0.23 grammar error. THE CLASS IS NOW CLOSED BY RULE, as that ruling directed: the migration tool reads the manifest and keeps every case whose required verdict cites a `FORM-*` rule, which is exactly §2 "Canonical form" (FORM-1..FORM-7 plus the LEX-1 policy rule no case can assert). MEASURED, and the brief's guess that this is a one-member set is wrong: the rule names SIXTEEN of the 401 case files. It covers all 12 FORM-family members of the retired 20-name hand list, adds the one that list missed, and adds the three FORM-7 cases — which rendering does in fact preserve, since terminal interiors keep their bytes, but whose subject is still a literal's spelling. Splitting the family to exclude them would be a per-rule hand list again, which is the defect being replaced. The other 8 of the old 20 are refused by the tool because they do not parse, which needs no list.
  (c) FIVE CASES RESTATED IN SOURCE, recorded rule and manifest verdict unchanged, each observed: `x-typ-bool-cmp-result-as-int` TYPE-5 (a `set` target's type is still written, and TYPE-5 owns `set` exactness verbatim); `type7-neg-implicit-read` TYPE-7 (a declared parameter type is still an expected value type); `x-typ-match-foreign-variant` TYPE-6 (source-enum scrutinee, the same change of witness the ERR-2 case took); `op1-neg-eeq-payload-enum` OP-1 (a payload-carrying enum is affine, so the deleted argument left bare operands earning OWN-1 first; moving both restores OP-1's domain judgment); `form2-neg-noncanonical-ws` FORM-2 as above.
  DIVERGENCE STATED PLAINLY: the brief ruled three of these as manifest citation changes (TYPE-7 -> FN-1, TYPE-6 -> GRAM-6, OP-1 -> OWN-1) and one as a repurpose-or-retire. In each the compiler's citation is literally conforming — FN-1 does own an unreachable statement, GRAM-6 does own a Bool-scrutinee `match`, OWN-1 does own a bare affine place — but in each the cited violation is an INCIDENTAL or EARLIER-FIRING one and the case's own concern is still expressible in v0.23. Restating the source is therefore the disposition the ruling of record already fixed for this class ("restate minimally so the recorded rule fires again, keeping the witness where possible"), and it keeps the negative coverage a citation change would have deleted silently. Where the two instructions agree — a concern that genuinely died — the citation moves; see (d).
  (d) TWO CITATIONS MOVED, source unchanged: `own1-neg-match-move-through-borrow` OWN-1 -> OWN-5, because OWN-5 states the prohibition verbatim ("Content reached through any borrow may never be moved: `move` requires a place rooted at an own-mode binding") and OWN-1 never states it — this row was wrong before the migration and is main's single red case; and `x-match-give1-wrong-type` TYPE-5 -> GIVE-1, carrying out the recorded 2026-08-08 amendment rather than the brief's TYPE-5 -> EFF-2, since its `traps` row is leftover incidental content while delivery-set agreement is the surviving concern the rewritten GIVE-1 owns. Both docs updated to describe what the case now asserts.
  (e) ONE CASE NOT CHANGED, and the reason is a compiler defect rather than a decision: `fn2-neg-eeq-implicit-type` still accepts. FN-2 does retain negative content under v0.23 — explicit instantiation arguments, and the region-free `targ` requirement — so this is NOT the specification finding the brief said to stop on. But the surviving content matching this case's concern is the missing mandatory instantiation argument, and [DIAG-1] requires "[FN-2] for a user-generic call" at the `call` node, while the compiler cites TYPE-5 for a missing, a wrong-count, and a wrong-kind user-generic type argument alike, and locates the wrong-kind case at the `targ` rather than the call. That is a spec/compiler discrepancy, which stops the affected work; it is the same gap already recorded as the `pending` reason on `fn2-neg-implicit-instantiation`. The remaining FN-2 negative content, the region-bearing `targ`, is already covered by two live passing cases, so restating onto it would add no coverage. Nothing was written.
- boundary: the ten cases named above, their manifest rows, `compiler/src/resolution/scopes.rs`, and the migration tool's exclusion criterion. No specification byte, no candidate re-key, no change to any case's `status`, and no change to the five out-of-scope failures or the 13 requires-block cases.
- evidence: every verdict below was produced by `./compiler/target/debug/whitefootc --emit-llvm tests/conformance/cases/<id>.wf` with the exit code read from `$?` directly. Adapter `make conformance-run`: 359/28/14 -> 368/19/14, nine cases resolved, zero newly failing, the failure-set diff recorded in `docs/ongoing/0038-floor5-semantic-and-migration.md`. `make -C compiler check` exit 2 both before and after with the failing set byte-identical at 262 and passing 308 -> 309; `make check` exit 2 at the same step, corpus structure 18/18, coverage 128/128 rules with 0 uncovered. Exclusion measured by one tool invocation per file over all 401 cases: 16 kept, 8 refused, 377 unchanged, 0 rewritten.

## 2026-08-08 — rulings (the three conformance verdicts the citation fix moves)
- owner: lead rulings under the standing overnight delegation; owner ratification pending
- authority, verified by the lead's own command against the candidate at line 220 rather than relayed — [TYPE-5]'s own sentence assigns all three callee classes: "Call sites state explicitly exactly what their callee class requires: type, region, and const arguments for user generics [FN-2]; region arguments for system operations [SYS-2]; and, for exactly the retained-argument table operations — `cvt` and `reinterpret`, `array_new`, `arena_new`, and `finf`/`fnan` — the written arguments their rows fix". A region argument on a user function is therefore FN-2's, not TYPE-5's.
  (1) `type5-neg-wrong-region-arg-count` — RESTATE THE MANIFEST CITATION, TYPE-5 -> FN-2. Its recorded rule was wrong before this change and independently of it, exactly like `own1-neg-match-move-through-borrow`'s OWN-1 -> OWN-5, which was `main`'s single red case. The case id keeps its `type5-` prefix: an id is a stable identifier, not a claim, and the same precedent renamed nothing.
  (2) `type5-neg-shared-for-uniq-arg` and `x-fn-own-arg-for-ref-param` — RESTATE THE SOURCE, NOT THE CITATION. Their subject is a mode or type mismatch at an argument; both omit the mandatory region argument and are rejected for that first, so the recorded rule never fires. Before the fix the masking citation happened to be TYPE-5 and matched the row by coincidence; the fix did not break these cases, it made a pre-existing mask visible. Write the mandatory region argument so each case reaches its own concern, then RECORD THE OBSERVED VERDICT — do not assume it returns to TYPE-5, and verify against what the specification assigns to the violation that remains. If a case becomes ACCEPTED once the region argument is written, its concern died with the deleted bytes and it needs the other disposition, reported rather than restated.
  (3) `fn2-neg-eeq-implicit-type` — the lead's prediction that a citation fix would clear it was WRONG, and the executor said so rather than leaving it looking open. Its source `return eeq(left, right);` is legal v0.23 and exits 0, so its concern died with the bytes A1 deletes; it is round 8's finding-2 class and never was a citation defect. Disposition: RETIRE, conditional on an enumeration first — name which live case now carries each surviving piece of FN-2 negative content (the arity case moves to FN-2 under ruling 1; the region-bearing `targ` half was already reported covered by two live passing cases). If any FN-2 negative content would be left uncovered, repurpose onto it instead of retiring.
  (4) `fn2-neg-implicit-instantiation.wf`'s `pending` reason is STALE and must not simply be deleted. It reads "the active compiler does not yet implement the complete generic-instantiation judgment and its FN-2 diagnostic path"; that path now exists. The real blocker is that the case's violation is no longer expressible, which is ruling 3's class. Re-disposition it on the true reason and correct the recorded one; a stale reason left in place is worse than none, because it sends the next reader after a defect that has been fixed.
- boundary: three conformance cases and their manifest rows, plus one `pending` reason. No specification byte and no candidate re-key.
- evidence: the executor's before/after table with a correct call as the control in each callee class, and its verification that the region-argument omission is a real violation rather than an artefact of the fix.

## 2026-08-08 — rulings (#24's three, and a finding that bears on the byte approval)
- owner: lead rulings under the standing overnight delegation; owner ratification pending. THE THIRD ITEM IS FLAGGED FOR THE OWNER DIRECTLY, not merely for ratification.
  (1) `x-give-result-aggregate` and its lib sibling — RECLASSIFIED, and my "wrong rejection on a positive case" was wrong. It is UNFINISHED MIGRATION: [TYPE-5] mandates written prelude-constructor arguments in every position, and `whitefoot-migrate` writes them in exactly two — an annotated `let` and a `return` — with no rule for `give` or `propagate`, so it left those bare and they are now illegal. Controls: `give Ok(value: 1_u64);` earns TYPE-5 while `give Ok<u64, u64>(value: 1_u64);` exits 0, same for `propagate`. AUTHORIZED: complete the migration at all three measured sites (`x-give-result-aggregate.wf:3,5` and `semantic/tests.rs:737`) AND teach the tool the two missing positions, because a re-run would otherwise re-break them. Both halves land together: they share one cause, and fixing the lib half alone would strand the conformance half and leave the pinned failure set half-stale. This completes a migration; it changes no verdict.
  (2) `own5-neg-slice-value-match` — RECLASSIFIED from capability gap to MASKED NEGATIVE, and the executor's discriminator is what earns it: holding everything fixed but the join, two branches moving the SAME binding reach `Semantics/Source [OWN-5]` while two branches moving DIFFERENT bindings reach `Unsupported { OwnershipJoin }`. So the OWN-5 rejection is implemented, correct and reachable at `control.rs:523`; `join_states` merely runs first. This is the FOURTH masking instance in this batch. AUTHORIZED as an ordering change rather than capability work — a slice-valued delivery is prohibited outright, so no join outcome could make it legal and judging OWN-5 first can only convert an Unsupported into the correct rejection. REQUIRED: verify rather than argue that claim — run the full corpus and library and confirm nothing newly rejects, since a reorder that changes acceptance would be a different thing entirely.
  (3) `own3-pos-outlives-store` — MY CLASSIFICATION WAS WRONG AND SO WAS THE EXECUTOR'S. It is neither a pre-existing capability gap nor a located predicate defect to widen. The case's doc states its whole subject: "an enclosing region outlives an inner one; a `'r` borrow satisfies an `'s` destination". That destination is the written annotation `let q: &'s i32 = &'r a;`, in which the ANNOTATION NAMES A DIFFERENT REGION THAN THE RIGHT-HAND SIDE. A3 deletes it, so the migrated program reads `let q = &'r a;` and means something else — the case's subject is no longer expressible at all, which is round 8's finding-2 class; the `Unsupported { RegionsAndBorrows }` it now reaches is an artefact of the changed program, not the defect.
  THE FINDING, AND IT IS FOR THE OWNER: **A3's premise is that a binder's mode and type are exactly what its right-hand side produces, and this site is a counterexample** — a v0.22 program in which the annotation legally named a region the right-hand side did not. So v0.23 removes the ability to state a destination region for a local binding, and the candidate's §5 accounts for the accepted-program set as "one canonical respelling plus three deliberate narrowings" WITHOUT listing this one. Either it is a fourth narrowing that must be named before bytes are approved, or the premise needs restating; it cannot stay unstated. The executor's predicate analysis (`borrow_holder_scope_supported` at `borrows.rs:894` testing scope-parent identity where `region_outlives` at `borrows.rs:883` is the relation OWN-3 means) remains a real and separately valid observation about the compiler, and is registered on its own — but it is not this case's cause and widening it would not fix this case.
- boundary: three migration sites and the migration tool; one ordering change with a no-new-rejection verification; no verdict restated, no manifest row edited, no specification byte.
- evidence: the executor's controls for (1) and (2), each holding everything fixed but the variable under test; for (3), the lead's own `git show` of the pre-migration bytes against the migrated ones, and the case's own doc string naming the destination.

## 2026-08-08 — ruling (the emptied-case class, and a sweep it makes mandatory)
- owner: lead ruling under the standing overnight delegation. THE SWEEP AND ITS CONSEQUENCE ARE FOR THE OWNER DIRECTLY, because they bear on what a byte approval is being shown.
- reason: an executor found `fn2-pos-explicit-instantiation` PASSING while testing nothing. Its subject is that explicit instantiation arguments are written and monomorphized; A1 respelled them away, its source now reads `let a = 40_i32 + 2_i32;` with no instantiation argument anywhere, and its doc still claims one. It is the same class as `fn2-neg-implicit-instantiation`, retired in the same round after an enumeration showed it carried none of FN-2's four live content pieces — except that one was RED and this one is GREEN.
  THE GENERAL DEFECT, which is worth more than either case: **a migration that deletes a construct silently empties every case whose subject was that construct, and roughly half of those go green rather than red.** v0.23 deletes three constructs — written type arguments on operations, the `let` mode and type annotation, and the Bool-scrutinee `match`. Negative cases whose subject died turn red and this batch caught them one by one. Positive cases whose subject died turn GREEN and nothing in the repository looks for them. That is a selection effect in our own process, not bad luck: we found exactly the half that failure surfaces.
  RULED: a corpus sweep for this class is a PRECONDITION of the v0.23 byte approval, not a follow-up. The tell is a case whose `doc` names a construct its source no longer contains; cross-check that the source still contains the construct its manifest rule is about. Report every hit with its file and the construct that left; do not restate anything found — dispositions are the lead's and some are the owner's.
  CONSEQUENCE FOR THE PACKET, which must be stated plainly rather than left implicit: the corpus's green is partly unearned, and the adapter's figure does not distinguish "passes because the rule holds" from "passes because the case no longer tests its concern". A reviewer approving bytes is entitled to know which kind of green they are being shown.
- also ruled, same round:
  (a) `fn2-neg-implicit-instantiation` RETIRED, and the enumeration is what earned it: it carries none of FN-2's four live content pieces, and its own content — a generic operation's missing instantiation argument — is not FN-2's content under v0.23 at all, since a table operation carries no written argument and [DIAG-1] gives it [OP-2]'s selection. Dead rather than redundant, so retiring drops nothing. Case file and manifest row both go, since there is no `retired` status and every row pairs with a file.
  (b) The driver witness table's `x-match-give1-wrong-type` entry said TYPE-5 where the manifest and the compiler both say GIVE-1 — a desync left by the M3b ruling that moved that citation with the source unchanged. It had been invisible because the now-retired dead entry failed in front of it: the FIFTH masking instance this batch. Corrected BY HAND against the ruling and never derived from the manifest, which is correct — that table is a deliberate second witness and deriving it would destroy what it exists for. All 21 entries were re-checked rather than only the one that surfaced; exactly one disagreed.
  (c) A wrong-kind instantiation argument is FN-2 content with NO conformance case, only a library test, while the coverage gate counts FN-2 covered on its other four carriers. With SYS-2's identical shape this is now a pattern rather than an incident: **the coverage metric is per-rule, not per-content-piece**, so "128/128 rules covered" does not mean every piece of a rule's content has a case. The packet must say so.
- boundary: one case retired with its row, one witness-table entry corrected, one sweep authorized as report-only. No verdict restated, no specification byte.

## 2026-08-08 — rulings (the emptied-case sweep's results, and a third class it discovered)
- owner: lead rulings under the standing overnight delegation; the packet statement at the end is for the owner directly.
- result, report-only as instructed: two EMPTIED cases (`fn2-pos-explicit-instantiation` known, `type5-pos-explicit` new), three SUBJECT-SHIFTED, thirteen doc-stale with subject intact, one false positive, plus the red half retired in round 17.
  (a) A THIRD CLASS EXISTS THAT NEITHER THE RULING NOR THE BRIEF ANTICIPATED: **subject shifted**. `op1-neg-eeq-integer`, `op1-neg-ene-integer` and `op1-neg-ineg-unsigned` were written about a WRITTEN TYPE ARGUMENT and now reject on an OPERAND DOMAIN. They are harder to see than an emptied case in a specific way worth recording: an emptied case can be caught by asking whether its rule still fires, whereas these still fire AND cite the correct rule, so a rule-level audit passes them. Only reading the doc against the source finds them. Each still tests something OP-1 owns, so nothing is broken — but the corpus records three concerns it no longer covers, and no reader can tell.
  (b) BOTH EMPTIED CASES AND ALL THREE SHIFTED ONES GET THE ENUMERATION PRECONDITION before any disposition, the move that has now changed an outcome twice: name what live content each rule still has and which case carries each piece, then dispose. Do not infer a case's content from its name or its rule column. For `type5-pos-explicit` specifically, note that BOTH halves of its stated subject — a `let` stating full mode and type, and a call stating its type argument — are deliberately deleted by v0.23, so it may be dead rather than restatable; the surviving positions that still state these things are signatures [FN-1] and the retained-argument class, and whether restating onto those preserves this case's concern or merely duplicates another case is exactly what the enumeration must answer.
  (c) THE THIRTEEN DOC-STALE CASES SPLIT. Four are the `ieq`/`ine` ones whose docs the infix-comparison reversal will make correct again: LEAVE THEM, do not fight a concurrent change. The nine arithmetic ones do not self-correct and their docs are authorized to be corrected — doc text only, with each case's rule and kind verified unchanged afterwards. A doc that lies about its own source is what made this sweep hard in the first place.
  (d) `tests/programs/` is NOT a gap to close. The executor correctly flagged that its twenty files carry no manifest doc so the tell does not exist there — but those files assert behaviour (they compile and run and produce correct output), not language-rule coverage, so an emptied subject does not have the same meaning. Recorded so the exclusion is deliberate rather than ambiguous.
- METHOD, worth keeping: the first pass returned 97 candidates by substring matching (`ile` inside "while", `ine` inside "line") and the executor DISCARDED that list rather than reporting it, on the ground that a sweep whose hits are mostly artefacts is worse than none because it buries the real ones. Word boundaries cut it to 19, each read by hand against its pre-migration bytes. This is the third tooling trap of this class in one day, after `-E` treating `\b` as a literal `b` and ugrep refusing a bounded-repetition pattern outright.
- FOR THE REVIEW PACKET, adopted close to the executor's own words because they are exact: after this sweep the corpus's green differs from earned green by **at least two cases, plus three that changed what they test**. "At least" is load-bearing and must not be dropped — the mechanical tell only catches a doc that names a CONSTRUCT, and `type5-pos-explicit` was found by reading because "types match exactly" names none, so the true count may be higher and this is not exhaustive. And the asymmetry must be stated as what it is: the red half was caught one case at a time over four rounds while the green half required a deliberate search, which is a property of our process rather than of the migration.

## 2026-08-08 — ruling (what "128/128 rules covered" actually asserts) and the GRAM-6 rank hole
- owner: lead ruling under the standing overnight delegation; the packet wording is for the owner directly.
- MEASURED, and verified by the lead reading `tests/conformance/runner.py` rather than relayed: `tagged |= set(c["rules"])` at line 315 and `covered = by_case | annotated` at line 327. **A rule counts as covered when any case merely NAMES it** in its `rules` list. Not when a case's expected verdict cites it, not when any case would fail if the rule stopped being enforced, and with no notion of a rule's separate content pieces. The runner already computes and prints both exercise sets — `[+109/-45]` at line 365 — but `covered` requires neither.
  Figures: 128 reported covered (110 by case, 30 by annotation); 109 with a positive exercise; **45 with a negative citing them; 83 covered with no negative at all; 19 with neither**.
  THE PACKET FORM, and the qualification is as important as the number. A negative case fails if a rule stops REJECTING, so for those 83 **a rule that silently stopped rejecting would not be caught by this corpus**, and "128/128 covered, 0 uncovered" does not distinguish that from a rule with a real negative. But 83 IS NOT A DEFECT COUNT and must not be presented as one: many of those rules have no negative form at all — GRAM-1..5 are grammar productions exercised positively, the DIAG rules constrain diagnostic shape, EX-1 is a worked example. The honest claim is about what the metric asserts, not that 83 rules are unguarded. The executor's judgement here is adopted verbatim in substance: state it as "the number counts naming" rather than as a gap tally, because a gap tally will read as alarmism and be discounted, and the real point will be lost with it. This is the second instance of the shape the wrong-kind FN-2 gap showed — per-rule, not per-content-piece.
- SYS-2 itself is landed correctly and the ordering hazard I flagged turned out to be covered by an existing machine check, which is better than the verification I asked for: `definition_rank` is checked against the active specification's definition order, so SYS-2's rank is FIXED at 38 by where the candidate defines it, with CLM-1 and CLM-2 shifting up. **The check passing is the evidence the placement is right rather than merely unique.** No verdict moved; the adapter gains only the new negative.
- RULED on the omission found in passing: `SemanticRule::Gram6` has a `definition_rank` but is absent from the rank test's `ALL` array, so 40 variants have ranks and 39 are checked. ADD IT — and if adding it fails, that failure is the finding and the point of the check. A check that silently omits one of its subjects is worse for that subject than no check at all, because it reports "all verified" while one is not, and GRAM-6 is the v0.23 conditional rule, the newest and likeliest to be wrong.
  ALSO REQUIRED: make the omission structurally impossible rather than fixing this instance. Derive the list through an exhaustive `match` over the enum so a new variant cannot be added without the compiler demanding its entry. This is not new machinery — it is using the type system in place of a hand-maintained array, which is what this project's own rules prefer over a script that re-implements what the compiler already enforces.
- boundary: the rank test, the rule inventory, one new negative case with its row, and a packet obligation. No specification byte, no verdict restated.

## 2026-08-08 — rulings (EX-1's drift, the requires-clause `eeq` pincer, and a diagnostic that misdirects)
- owner: lead rulings under the standing overnight delegation; owner ratification pending.
- FIRST, A LEAD ERROR TO RECORD BECAUSE IT ALMOST BECAME A FINDING: I reported that `ex1-pos-worked-example.wf` was a SUPERSET of EX-1's block and therefore could not be diffed. It is a verbatim copy. I had extracted the specification side inside an arbitrary `sed` line window and the case side without one, so one extraction was truncated and the other was not — and I read the difference between my two methods as a property of the two artefacts. The case's real divergence was four bytes. **Comparing two things pulled by different methods produces a difference that belongs to the methods.** The superset I was thinking of is a different case, `run-ex1-value-match.wf`, which names no EX-1 in its rules and must not be pinned to the block.
  (1) EX-1 IS NORMATIVE BYTES, verified rather than inferred: §19 is titled "Worked example (normative bytes)", the rule requires byte-exact canonical form, and [SCOPE-2] accepts a program only if it satisfies every rule — so the §7 digest pin is a consequence of that status, not the evidence for it. The block is the correct side, because `sign_of` returns from the BRANCHES of an `if`/`else` chain after GRAM-6 and "arms" is v0.22's word; `git log -S` places the drift at candidate assembly rather than at migration, and the migration tool never touches string interiors, correctly. The case is now byte-identical to the block, verified by the lead's own extraction and `diff`.
  (2) THE CHECKER IS AUTHORIZED, having been correctly reported rather than built. It is a test, not machinery: byte equality between the fenced block following EX-1's sentence — the only one in §19 — and the case the manifest identifies, with exactly one row naming EX-1 so no filename is hard-coded. Both halves are already in memory (`ACTIVE_KERNEL_SPEC_TEXT` and the existing `include_bytes!`). Record its stated limit with it: **it pins the copy to the block, never the block to reality.** This project forbids machinery no experiment needs; it does not forbid a test for a defect that just occurred and would have reached an owner approval.
  (3) `fn8-neg-requires-eeq-payload-enum` IS RETIRED, and the enumeration is what earns it rather than the failure. Six spellings measured, none reaching OP-1 — bare operands and both borrow forms earn OWN-1 `BareAffineUse`, `move` operands and a clause-local binding earn FN-8 `InvalidRequires`, and a `const` operand is GRAM-3 — against a control where the same clause over a TAG-ONLY enum exits 0, which is what shows the clause admits `eeq` and the payload is the variable. Reported honestly as six spellings rather than as a proof over all forms. The OP-1 content is not at risk: the sibling `op1-neg-eeq-payload-enum` already reaches OP-1 from a function body by moving both operands — the exact repair a clause forbids — so what this case uniquely added was only "a clause cannot widen the domain", which is measured true and already delivered by two other rules.
  REQUIRED WITH THE RETIREMENT, so the lane's figure is not read as progress it is not: the review packet must state that the final adapter count includes one case **retired as unreachable rather than fixed**.
  (4) A LIVE DIAGNOSTIC DEFECT FELL OUT AND IS SEPARATELY TRACKED. OWN-1's `mechanical_fix` reads "write `move p` for the affine place", and that exact repair is rejected by FN-8, which names `move` in its own prohibition list. **Inside a requires clause the compiler sends the writer from one hard error to another with no third spelling.** A mechanical fix that leads to a second rejection is worse than none, because it is a wrong instruction rather than a missing one, and it survives whichever way the case is disposed.
- boundary: one conformance case aligned, one retired with its row, one test authorized, one defect registered. No specification byte.

## 2026-08-08 — correction: the retirement above is NOT the lead's to execute
- owner: correction by the lead; **this one needs the owner's word before anything is removed**.
- reason: the entry above says `fn8-neg-requires-eeq-payload-enum` "IS RETIRED". The executing agent declined to carry it out and was right to. `CLAUDE.md` is explicit: tests may be added freely, but existing conformance material is changed or removed "only with owner agreement and an approval-ledger entry" — two conditions, and I supplied only the second. The overnight standing delegation covered decisions while the owner was away; the owner is now present and interacting, so a removal of protected evidence is theirs to agree to rather than mine to rule. The case, its row and its verdict are untouched and it still shows in the adapter's failing set, which is the correct state while an ask is outstanding.
  ALSO AFFECTED, and disclosed rather than left quiet: `fn2-neg-implicit-instantiation` was removed earlier today under the same standing delegation, so it carries the same defect in its authorization. Its reasoning is recorded in full and is ratifiable, but it is an already-executed removal awaiting agreement rather than a settled one.
- THE EVIDENCE IMPROVED WHILE THIS WAS OPEN, and the decisive measurement is better than the lead's reasoning. I argued the OP-1 content was safe because the sibling `op1-neg-eeq-payload-enum` reaches OP-1 from a FUNCTION BODY. The executor found `fn8-neg-requires-eeq-integer.wf`: the same clause, the same shape, the same bare operands over `own u32`, reaching **OP-1 `InvalidOperation`** — verified by the lead's own run. Because `u32` is copy, no OWN-1 candidate forms and the domain judgment is the first thing that can fail, **so the concern is already covered in exactly the disputed position** rather than merely elsewhere. The payload variant is a second instance of a claim already carried, unreachable for a reason independent of that claim.
  The other two dispositions are refuted on evidence rather than taste: restating outside the clause duplicates the sibling, and restating onto the requires-subset boundary is refuted by measurement, since `eeq(move left, move right)` over `own u32` rejects FN-8 identically — the `move` prohibition is TYPE-INDEPENDENT, so a payload enum there would be decorative.
- WHAT WAS ADDED INSTEAD IS FREE AND CLOSES A REAL HOLE. FN-8's prohibition list names nine forms; the corpus carried negatives for four, and **`move` had none**. `fn8-neg-requires-move-operand` is a one-token differential whose control is the evidence: `ieq(left, right)` in the clause compiles at exit 0, `ieq(move left, move right)` rejects FN-8, with operation, domain and clause shape legal in both — so the moved operand is provably the SOLE violation. That is precisely what the payload-enum case could never be, since it carried two violations at once. Adapter Pass 387 → 388; negative exercises 44 → 45, which moves §1.1's metric by a real negative rather than by a rule being named.
- recorded and deliberately not chased, one observation with no claim attached: `eeq(move flag, True())` in a clause cites FORM-3 at the `True` token rather than GRAM-9.

## 2026-08-08 — ruling (the migration destroys exactly the negatives that matter, and how to catch it)
- owner: lead ruling under the standing overnight delegation; owner ratification pending.
- THE HAZARD, verified by the lead's own run rather than relayed: `whitefoot-migrate --check` on the newly added `op1-neg-written-argument-on-deargumented-row.wf` reports **1 file, 1 changed, 1 argument list**. Re-running the tool would delete the written argument list, **which is that case's entire violation**, leaving a case that passes while testing nothing. The `.wf` hold-back keys on a `FORM-*` required verdict and this case cites OP-1; the `migrate: keep` marker is Rust-only.
  THE GENERAL FORM, and it is the mechanism behind §1.2 rather than a new problem: **any negative whose violation IS a construct the migration deletes cannot survive the migration.** That is precisely what emptied `fn2-pos-explicit-instantiation` and `type5-pos-explicit`. The tool is not misbehaving — it is doing its job to a file whose subject is the thing being removed.
  RULED: do NOT widen the hold-back list, and do not add this case to it by name. A hold-back must decide statically which cases to skip, and the deciding property — "the violation is the construct being deleted" — is exactly the thing this batch established is not machine-readable, since it lives in a case's subject rather than in its bytes. A hand list of names is the defect that was already replaced once by deriving the FORM-* class from the manifest.
  RULED INSTEAD: **a post-migration verdict differential.** After any corpus migration, re-run every case and compare its actual verdict against its manifest row; a negative that now passes, or that rejects under a different rule, had its violation deleted or moved. This DETECTS rather than prevents, which is the correct trade here — the tool is run deliberately, detection is exact where prevention is a guess, and the same check would have caught both §1.2 cases automatically at the moment they were emptied instead of four rounds later under a deliberate search. It is the same shape as the verdict-differential rule already standing for citation moves.
- SECOND HAZARD, accepted rather than engineered around: two of the three newly added cases name their CONTROL in the doc (`cvt<u8, i64>`, `pick<Held>`), which the body deliberately lacks, so the emptied-case tell will read them as false positives. Naming the control is what makes a one-token differential legible to a reader, and the tell cannot distinguish a construct named as a contrast from one named stalely. Both are good practices in conflict; the resolution is that the tell's output is hand-verified anyway, so the class is documented as an expected false positive rather than either practice being changed. Recorded in the sweep's criteria so the next sweeper does not rediscover it as three defects.
- also recorded: the −44 → −45 negative-count move earlier attributed to `fn8-neg-requires-move-operand` was **SYS-2 arriving through a rebase**; FN-8 already had seven reject carriers and the case could not have moved it. The lead repeated the wrong figure approvingly. The corrected version is stronger than the claim it replaces — closing three real content gaps moves the coverage figure by **exactly zero**, which demonstrates §1.1 instead of restating it.

## 2026-08-08 — correction: the post-migration verdict differential was the wrong ruling, twice over
- owner: lead correction. The executing agent measured before building and the measurement is what stopped it building; nothing was wired and nothing added.
- FIRST, IT ALREADY EXISTS. "Re-run every case and compare against its manifest row" is the conformance adapter, which already runs in the repository gate. Demonstrated by mutation rather than read off the source: temporarily removing the violation from the new FN-2 case moved the adapter 391/2/13 → 390/3/13 and named the case; restored, tree clean. The moved-citation half is evidenced by the adapter's own live output, `want Reject("OP-1") reached Reject(Some("OWN-1"))`. **I ruled the construction of a check that was already running.**
- SECOND, IT WOULD HAVE CAUGHT NEITHER CASE I INVOKED IT FOR. Both §1.2 emptied cases are POSITIVES that kept their verdict: `fn2-pos-explicit-instantiation` is `run 0` before its subject died and `run 0` after, and the adapter passes it. A verdict differential has nothing to compare. So the ruling was wrong on the mechanism and wrong on the purpose at the same time, and — the executor's phrase, which is the whole lesson — **it would have been built, run green, and been believed.**
- THE POPULATION NOTHING WATCHES IS A DIFFERENT ONE. A verdict that moves *with its manifest row following* is invisible to the adapter by construction, since the adapter compares actual against declared and both moved together. That is the shape `CLAUDE.md` names a governance breach — editing a verdict to go green — precisely because nothing mechanical stops it.
  Diffing every DECLARED verdict across the batch base finds exactly one: `type5-neg-wrong-region-arg-count`, `reject TYPE-5` → `reject FN-2`, which is authorized by the ruling at APPROVALS.md:517 and reported as found and clean rather than as a suspicion. Verified by the lead's own diff.
- RULED: build **the declared-verdict diff across a revision range** instead. No compiler invocation, no case execution, no knowledge of any case's subject. Its first run yields one hit and that hit is clean, which is the right state to wire from — **a new check whose first run is noisy teaches people to ignore it.**
- THE LIMIT, and it is round 26's shape a second time: **a verdict is the wrong instrument for a question about subjects**, exactly as a source program was the wrong instrument for a question about the checker. Naming which instrument fails is what stops the next person building the version that cannot work.
- lead process note: my own probe of the emptied case used a `whitefootc` binary built before the infix reversal and reported a stale `UnresolvedUse` on `ieq`. The claim was confirmed instead from my own earlier adapter run. A stale binary answers the question you asked about a tree that no longer exists.

## 2026-08-08 — third correction to the same ruling: three populations were conflated
- owner: lead correction, prompted by the executor reading the ledger entry rather than my message, which is the protocol working as intended — a defect in an authorization was found by someone consulting the authorization.
- The correction at `0027e8e` said the original ruling was "wrong on the mechanism and wrong on the purpose". That is too coarse and it obscures what was actually wrong. Measured from the manifest: `fn2-pos-explicit-instantiation` and `type5-pos-explicit` are both `kind: run`; `type5-neg-wrong-region-arg-count` is `kind: reject`. **The original entry states a general form over NEGATIVES and then instantiates it with two POSITIVES.**
- THE THREE POPULATIONS, separated, because the ruling ran them together:
  (A) **A negative whose violation is a construct the migration deletes.** The general form is correct about this class — but such a case then COMPILES where its row demands a rejection, so the conformance adapter already catches it and always did. Nothing new was needed.
  (B) **A positive whose subject is deleted.** It keeps passing, its verdict never moves, and no verdict-based check of any kind can see it. These are §1.2's two cases, and they are what the ruling was justified by. A verdict differential would have caught **zero** of them.
  (C) **A declared verdict that moves with its manifest row following.** Invisible to the adapter by construction, since it compares actual against declared and both moved together. This is the real gap, it is not empty — exactly one member this batch, `type5-neg-wrong-region-arg-count`, authorized and clean — and note it is a NEGATIVE, which is the population the general form correctly describes.
  So the general form was right, the instances belonged to (B), the check needed for (A) already existed, and the check worth building addresses (C). The decision — do not widen the hold-back, detect rather than prevent, a hold-back cannot decide a property living in a case's subject — survives all of this unchanged.
- WHY THIS MATTERS IN A LEDGER SPECIFICALLY, in the executor's words: whoever builds the ruled check will report it working, and the two cases that motivated it will still be green. An authorization whose justification names the wrong instances licenses work that then appears to succeed.
- This is the third correction to one ruling and each has been sharper than the last. Recorded rather than tidied, because the sequence is the evidence that the ledger is being read adversarially rather than filed.

## 2026-08-08 — ruling (the declared-verdict diff, and why it stays out of the gate)
- owner: lead ruling under the standing overnight delegation; owner ratification pending.
- BUILT AND RUN, NOT WIRED, as instructed. `runner.py verdicts <revision>` diffs declared verdicts across a range with no compiler invocation, no case execution and no knowledge of any case's subject. Its first run against the batch base reports **1 moved, 1 removed, 5 added, 406 total** — reproduced by the lead's own run rather than accepted:
  `MOVED   type5-neg-wrong-region-arg-count  reject TYPE-5 -> reject FN-2`  (authorized at APPROVALS.md:517)
  `REMOVED fn2-neg-implicit-instantiation    reject FN-2`
  **THE REMOVAL IS THE ITEM I DISCLOSED BY CONSCIENCE THIS MORNING.** The check's first run independently rediscovered the one thing that was visible only because I chose to look and to remember. Had it existed, that disclosure would have been mechanical. That is the strongest evidence available that it watches something real, and it is worth more than any argument for the check.
- RULED: it stays OUT of `make check`, and the executor's reasoning is why. A legitimate restatement makes it red, and the only way to hold a gate green through a legitimate change is a known-failures mechanism — which this record already forbids, on the ground that such a mechanism outlives the reason for it and ends up masking real failures. So it is an **operator check for migration and review time, read against the ledger**, not a gate. A check whose correct state is sometimes red must not be a gate; that is not a weakness of this check but the shape of what it measures.
- ITS LIMIT IS ASSERTED BY A TEST RATHER THAN DOCUMENTED, which is the better form and is adopted as the standard for this class: it cannot see an emptied case, because a positive whose subject was deleted still declares and still reaches the verdict it always did. That is now a failing-if-violated assertion, so **the limit cannot quietly stop being true**. A documented limit decays silently; an asserted one cannot.
- A DEFECT THE EXECUTOR CAUGHT IN ITS OWN WORK, recorded because the lesson is general and because of where it happened. Four new tests were appended after the `__main__` guard and were therefore never collected — and the suite still printed `OK`. What caught it was the **test count** not moving, 18 where 22 was expected. **A test that never runs and a test that passes are indistinguishable by the result line; only the count separates them.** It nearly shipped into the check whose entire purpose is catching that family, which is the sharpest available illustration that this class does not spare the people looking for it.
- boundary: one operator check added beside the coverage check, reading the corpus rather than the language. No gate wiring, no case, no verdict, no rule byte.

## 2026-08-08 — OWNER AGREEMENT: the four case removals and the twelve in-language doc corrections
- owner: **the owner, in conversation, 2026-08-08** — "上面那些需要我决定的,感觉没啥问题,就按这样来吧", given against the five itemized decisions with their evidence. This supplies the condition `CLAUDE.md` requires alongside a ledger entry, and it retroactively completes the authorization of the one removal already executed.
- AGREED:
  (1) `fn8-neg-requires-eeq-payload-enum` REMOVED. Six spellings measured, none reaching its recorded OP-1; `fn8-neg-requires-eeq-integer` reaches OP-1 in the same clause and the same shape over `own u32`, so the concern is covered in the disputed position itself.
  (2) `fn2-neg-implicit-instantiation`'s already-executed removal RATIFIED. Its source migrated to `let a = 40_i32 + 2_i32;` at exit 0; the violation died with the deleted bytes and its content is carried elsewhere.
  (3) `fn2-pos-explicit-instantiation` REMOVED. Passes while testing nothing; its doc names `iadd.trap<i32>` and `imul.wrap<i32>`, neither present in the body.
  (4) `type5-pos-explicit` REMOVED. Same class; both halves of its stated subject — a `let` stating full mode and type, and a call stating its type argument — are deleted by v0.23.
  (5) The twelve in-language `doc "…"` corrections AGREED, with the eight unverified ones to be read by hand first. These change case BYTES rather than a description, which is why they needed agreement.
- CONDITIONS CARRIED INTO EXECUTION: every removal is accompanied by the adapter figure before and after and a failure-SET diff by name; no verdict of any surviving case is edited; each doc correction is followed by a re-run confirming that case's rule and kind are unchanged; and the review packet states that the final adapter count includes cases **retired as unreachable or as testing nothing, rather than fixed**.

## 2026-08-08 — corrections carried out with the owner-agreed removals
- owner: lead corrections; the owner's agreement above governs the substance.
- THE DOC COUNT IS THIRTEEN, NOT TWELVE. The ledger entry and the review packet both said twelve, taken from one unit's report; a second unit re-derived the sweep independently and read every hit by hand, finding thirteen, and the sweep's own task record also says thirteen. Verified by the lead from the landed diff: sixteen case files changed — **thirteen one-line doc edits plus the three removed files**. Ten name a v0.22 arithmetic OPNAME where the body writes the operator; three name a de-argumented row with its written argument. The off-by-one was mine and is corrected here rather than in place, so the sequence stays visible.
- EXECUTION EVIDENCE, per step rather than in aggregate: adapter `391/2/13` → `391/1/13` → `390/1/13` → `389/1/13`, with the failure SET identical at every step after the first, where only the removed name left. **No surviving verdict moved at any step.** `Pass` fell by exactly the two passing cases; the third was failing, so it left `Fail`. Coverage held at 128/128, so no rule lost its only carrier. The final lane is **389/1/13 with one failure, `own3-pos-outlives-store`** — the A3 counterexample, which is the open wording question and nothing else.
- CONSEQUENCE BEYOND THE CORPUS, reported rather than absorbed: `type5-pos-explicit` was `include_bytes!`d as one of eight execution samples in `compiler_independent_scalar_cases_execute_through_host_llvm`, so that test now runs seven. Its named-argument and infix shapes remain covered by `gram11-pos-named-args` and `op1-pos-table-op`. A removal reaching outside the corpus is exactly the kind of thing an aggregate figure hides.
- THE VERDICT CHECK GOT ITS FIRST REAL USE ON THE CHANGE IT WAS BUILT FOR, and passed: `verdicts 8df0e29` reports the four removals and the one authorized move and **nothing else** — silent on all thirteen doc edits, which is correct, since a doc edit is not a verdict change. A check whose first application is the change that motivated it is the only way to learn whether it says what its author believes.
- A PERMANENT FALSE-POSITIVE CLASS FOR THE DOC SWEEP, recorded so it is not rediscovered as defects. The executor withdrew its own earlier warning that two new cases would trip the tell — they do not, because the pattern matches the call head and both spellings are present in those bodies. The real residue is four hits in two classes that **will never clear**: three type constructors (`array<`, `box<`, `arena<`), which a doc may name without the body spelling them identically, and `op7-pos-name-convention`'s deliberate illustration, whose subject is the naming convention itself so an operator has nothing to illustrate it with.

## 2026-08-08 — OWNER APPROVAL: v0.23 content approved; exact-byte digest pending one landing change
- owner: **the owner, in conversation, 2026-08-08** — "看起来都没问题,都是很直观很正常的修改。批了", given against an itemization of everything in v0.23 that changes which programs are legal, plus the review packet's account of what the evidence does and does not establish.
- WHAT WAS SHOWN AND APPROVED — the substance, not a digest:
  (1) The three deliberate narrowings §5 records: a Bool-scrutinee `match` is rejected in favour of `if` (mechanically respellable, corpus already converted); a value initializer whose delivery set is empty is rejected, with the statement form and no binding as the repair; and `if` leaves IDENT eligibility, a measured-empty class at zero declarations across 610 files.
  (2) The fourth item, an EXPRESSIBLE FORM REMOVED whose effect on the accepted set is not established: v0.22 admitted an annotation naming a destination region its right-hand side did not (`let q: &'s i32 = &'r a;`), and A3's deletion removes the ability to state a destination region for a local binding. One site corpus-wide out of 1954 annotated bindings. Approved with the lead's wording choice — named as a removed expressible form rather than asserted as a fourth narrowing, because a borrow at the outer region satisfies an inner destination by outlives, so the equivalent program may always be writable.
  (3) That the remainder of the delta — 62 sites across 34 rules — is respelling: the same programs under shorter spellings, with A1's operation type arguments, A3's `let` annotations, and the infix arithmetic forms deleting redundant text rather than changing acceptance.
  (4) The review packet's disclosures, which are the honest account of what the green signals mean: coverage counts NAMING rather than exercise, so 83 of 128 rules have no negative and a rule that silently stopped rejecting would not be caught; the corpus's green is partly unearned, with at least two cases passing while testing nothing and three testing a different concern than they record; two changes are not in the delta's own account; and four independent pincers exist where two defensible rules jointly leave a writer no legal spelling.
- **THE EXACT-BYTE APPROVAL IS NOT YET RECORDED AND MUST NOT BE INFERRED FROM THIS ENTRY.** The candidate is at `5037bd85…d791fd` as this is written, and one approved change is still landing — the A3 premise clause and §5's naming of item (2), on branch `task/0045-a3-wording`. Those bytes change the digest. An approval recorded against bytes that then change is void, and the compiler's own gate fails closed on exactly that mismatch, which is the mechanism working rather than a formality.
  SEQUENCE: the wording change lands, the lead recomputes the digest independently rather than copying an executor's figure, the owner is shown the final digest for confirmation, and only then is the `ACTIVE-SPEC:` line written. Writing that line to make a gate green is forbidden; it is an owner approval record.

## 2026-08-09 — attribution correction on the v0.23 activation entry, in the lead's own voice
- The activation entry records the owner's approval as taken by the lead and relayed to the executing agent, which is accurate but leaves the trail's first link in someone else's words. Correcting it here so the record shows where the approval entered and who is answerable for it.
- **I took the approval directly.** The owner reviewed an itemization of everything in v0.23 that changes which programs are legal — the three narrowings, the one removed expressible form with the wording I chose for it, and the statement that the remaining 62 sites are respelling — together with the review packet's account of what the green signals do and do not establish. They replied "看起来都没问题,都是很直观很正常的修改。批了", then confirmed it covered these bytes when asked. The executing agent did not witness that exchange and correctly said so rather than writing it as if it had.
- The digest that entered the chain, `e09b32ed…aace0f5`, was recomputed by me at the moment of approval and again before the activation, and by the executing agent twice more, one of those against the installed file with `cmp` confirming byte-identity to the candidate. **Four independent derivations, plus a filesystem comparison, before the `ACTIVE-SPEC:` line was written.**
- `make check` DOES NOT exit 0 after this activation, and the executing agent was right to refuse to report it as green. Two failures survive and neither is closable by an activation: `general_borrows_…` is a pre-existing `RegionsAndBorrows` capability gap, and `own3-pos-outlives-store` is the A3 counterexample the approved bytes deliberately name. **`docs/planned/0040`'s step 7 claimed both gates exit 0 while its own ready-state section named the capability gap as surviving — the two could not both hold, and the ready state is the accurate one.** A task record that contradicts itself is the same defect class this batch spent a day removing from the corpus; recorded rather than quietly reconciled.

## 2026-08-09 — ruling (the archive-integrity gate under the stable-filename model)
- owner: lead ruling on a governance shape the executing agent correctly refused to settle itself; owner ratification pending.
- THE GAP, found in orientation and not mentioned by the approved proposal's §5: `spec-archive-integrity` walks every recorded specification line and requires `spec/kernel-spec-<version>.md` to exist and hash to it. Under the stable model the ACTIVE version's bytes live at `spec/kernel-spec.md`, so the gate would look for `spec/kernel-spec-v0.24.md`, not find it, and red the activation commit. The reverse direction is safe: the stable file is not matched by the hook's `spec/kernel-spec-v*.md` pathspec, which is exactly why condition 1 chose a flat archive.
- RULED: **the activation writes an `ARCHIVE-SPEC:` line for the outgoing version**, and the gate resolves by LINE TYPE rather than by position. A version carrying an `ARCHIVE-SPEC:` line is at its versioned path; the one version carrying only an `ACTIVE-SPEC:` line is at the stable path.
  WHY THIS RATHER THAN "the last line resolves to the stable path": that alternative is ORDER-DEPENDENT, and an append-only ledger whose meaning depends on which line is last is fragile in exactly the way this project keeps paying for — a line appended in the wrong place silently changes what the gate checks. Type-dependence is order-independent, and the chain stays append-only either way.
  REQUIRED, and it is what makes the shape worth the change: the gate asserts that **exactly one recorded version lacks an `ARCHIVE-SPEC:` line**. Zero means the activation forgot to install the stable file; two means it forgot to archive the outgoing one. A botched activation is then caught by the gate rather than discovered later, which is the whole reason this record exists.
- ALSO RULED, on condition 8: the approved bytes must read `Status: ACTIVE vN` before approval. The ENT-5 candidate reads `Status: CANDIDATE, OWNER-APPROVED`, and there is no way to satisfy the condition without changing approved bytes — but the candidate's own header already forces re-approval on any byte change, and four of its bytes are independently stale (Status citing v0.22 and `b133b793`, roadmap revision 18, §2's prior-version name, and the provisional number resolving to v0.24). So both close in one pass, and the sequencing is not a compromise.
  **v0.23 does not satisfy condition 8 either** — its installed bytes read `Status: REVIEW CANDIDATE v0.23` while being the active specification. That was correct when written, since the proposal was approved but not adopted, and it becomes wrong at adoption. Its bytes are already installed and the archive is immutable, so v0.23 is GRANDFATHERED: the condition binds bytes approved from the switchover onward. Recorded rather than retro-fixed, because retro-fitting a released specification is the one thing the append-only rule exists to prevent.
- SEQUENCING AFFIRMED as the executor proposed, and the first step is the load-bearing one: **fix the gate first, on the current model, green, in its own commit.** A gate change proved correct before the file model moves is the difference between one failure mode and two interacting ones. The activation itself stays one commit, because between any two of its parts the tree has one version at two paths or a digest that does not match its ledger line.

## 2026-08-09 — ruling (O11 comes out of the ENT-5 activation)
- owner: lead ruling; owner ratification pending.
- `docs/current-plan.md` queued the ENT boolean-composition correction (O11) to be "drafted alongside the approved ENT-5 loop fix". That pairing was decided before the specification-filename switchover was sequenced onto this same activation.
- RULED: **O11 does not ride this activation.** The switchover was deliberately routed onto "the first activation with no EBNF change" precisely so a file-model change would not be paired with a large semantic one. Adding a second semantic correction to the vehicle chosen for being small undercuts the reason it was chosen. ENT-5's loop fix plus the switchover is already two independent failure surfaces in one commit; three is a different kind of risk, and the one this sequencing exists to avoid.
- O11 is drafted separately and rides a later activation. Nothing about it is urgent: the loop rule is the measured dominant cause of the deflate divergence and O11 is not.
- Also recorded, because it is the kind of thing that is invisible later: the executing agent's context was exhausted and auto-compacted mid-slice. It reported this itself, unprompted, and drew the distinction that matters — its orientation FINDINGS survived, the EVIDENCE behind them did not, so by this project's own standing rule they are claims until re-run. It also noticed that its branch sat at a base the integration tip had moved past, so every anchor it had checked was checked against superseded bytes. **An agent applying the evidence rule to its own memory is the behaviour this record exists to encourage**, and the slice was handed to a fresh agent on that basis rather than on any doubt about its work.

## 2026-08-09 — rulings (O11's status, the verdict flip it causes, and the split affirmed)
- owner: lead rulings; owner ratification pending. The verdict flip in (2) is flagged for the owner directly when O11 reaches approval.
  (1) **O11 IS NOT AN APPROVED SPECIFICATION CHANGE, and the task description calling ENT-5 and O11 "both approved semantics fixes" is false.** What carries owner approval is O11's POSITION IN THE QUEUE — `docs/current-plan.md`'s "Queued after FLOOR-5", whose entire text is "See the research note; drafted alongside the approved ENT-5 loop fix." There is no candidate file, no drafted delta, no anchor, no digest, and no exact-byte approval. It must be drafted from nothing and take its own owner approval. (A `grep` for O11 in this ledger now returns four hits; every one is the lead's own sequencing ruling of the same day, none is an approval of O11's content — the executor's measurement was taken before those lines existed and its conclusion stands.)
  (2) **O11 FLIPS A DECLARED CONFORMANCE VERDICT.** `tests/conformance/cases/ent3-neg-bor-no-comparison-origin.wf` is declared `reject` on OP-4 and its whole subject is that `bor` establishes nothing. Under the correction the else edge of `bor(below, above)` yields both `symbol >= 0` and `symbol < 4`, and `len(table) = 4` is an [ENT-2] implicit fact for `array<u8, 4>`, so the OP-4 obligation discharges and the program is accepted. RULED ON SHAPE, execution deferred to O11's own batch: the case is **rewritten as a positive whose subject is that the conjunctive read discharges the guard — not deleted.** That is a better case than the one it replaces, because it exercises the corrected rule rather than merely recording the absence of the old one. The verdict change is protected material and goes to the owner WITH O11's byte approval, in the same sitting, so the owner is not asked to approve bytes and separately discover what they do to the corpus.
  (3) **THE SPLIT IS AFFIRMED, and the executor's reason is better than mine.** I ruled O11 out because three failure surfaces in one commit is a different kind of risk. The stronger reason: pairing puts the switchover behind O11's drafting, review and owner approval — and **a small prose-only v0.25 is a genuinely useful first exercise of the stable-file model, whereas pairing makes the first stable-file activation also the first one nobody has rehearsed.** Adopted.
  (4) **O11's blast radius is larger than its own record says, and includes a rule nobody had named: CLM-2.** CLM-2 uses "a `band` result" as its worked example of a predicate with NO comparison origin, which the correction makes false. So O11 touches ENT-3's definition and sources AND CLM-2 — two rules, not one. The executor reports 6 sites; the lead's own count of lines carrying the phrase is 5, and the discrepancy is unresolved rather than adjudicated here: **re-measure it when O11 is drafted, and do not carry either figure forward unchecked.**
  (5) Recorded because it decides a drafting question taste would have gotten wrong: the grammar forbids a call as an argument (`atom := literal | "move" place | place | borrow_expr`), so a connective's operands can only ever be IDENTs and nesting connectives can only go through a `let`. A depth-1 definition therefore reads **no corpus site at all** — the one live case uses two indirection steps. The definition must recurse over the connective tree with one `let` step admitted at each node.

## 2026-08-09 — rulings (#29's OWN-3 gap: the standing red is a pincer, and one half is free)
- owner: lead rulings; owner ratification pending. The predicate question in (3) is escalated rather than decided.
- FIRST, A CORRECTION TO MY OWN TASK TEXT: I parked #29 as "off the critical path". **On the measurement it is not** — it is one of exactly two things standing between `make check` and green, and the executing agent said so rather than accepting the framing it was handed.
  (1) THE GAP IS CONFIRMED AND IT IS TWO OF THREE DISJUNCTS. [OWN-3] admits outlives-or-equals as *equal*, *strictly encloses*, or *caller-supplied over local*. `borrow_holder_scope_supported` (`borrows.rs:894-910`) reduces to `holder_scope.parent() == Some(region_declaration(region).scope())` — immediately-encloses, exactly one level. It admits the first disjunct and rejects the other two, reachable from source in six lines with controls that isolate it (nested regions borrowing the OUTER reach Unsupported; the same program borrowing the INNER exits 0). It raises `Unsupported(RegionsAndBorrows)`, the explicit-capability form, never a misreported source rejection — so it is a capability gap, not a wrong verdict.
  (2) **THE STANDING `make check` RED IS A PINCER, WHICH IS WHY IT HAS SURVIVED EVERY PASS.** `general_borrows_keep_their_escape_read_and_exclusivity_rejections` fails its second assertion at a node path **byte-identical to the probe's**, pinning the site to this predicate rather than to any of the other thirteen `RegionsAndBorrows` raises. But its expectation is ALSO stale: the fixture read `let q: &'s i32 = x;` — verbatim the A3-removed annotated form — and the migration deleted the annotation, so no OWN-4 violation remains to find. **Both halves must move together; fixing either alone leaves the test red.**
  RULED: **the fixture repair is authorized now.** It is a library test, not conformance material, so `CLAUDE.md`'s owner-agreement condition does not bind it — and the repair is a REVERT rather than an invention: `fn leak['r0](x: &'r0 i32) -> &'r0 i32 pure { region 's { return &'s deref(x); } }` is the text the file carried before `03ee2e8`, and it rejects with exactly `Own4/InvalidBorrowLifetime`. Restoring it restores the subject the migration destroyed, which is the same disposition class as the emptied conformance cases. **The obvious `let`-shaped variant is NOT a substitute** — it rejects `Own14/InvalidReborrowPosition`, because [OWN-6] admits a reborrow only as a call-argument atom, so the witness must be in return position. That measured distinction is what makes this a revert rather than a guess.
  (3) THE PREDICATE WIDENING IS NOT AUTHORIZED, AND THE QUESTION HAS INVERTED. Region exit (`control.rs:636-683`) ends ordinary borrow loans by BINDING SCOPE rather than region identity; only slice loans are region-keyed. And **the slice path already accepts the exact shape the scalar predicate forbids** — `region 'o { region 'i { let s = slice_of(&'o a); … } }` exits 0, because a slice binding's mode is `Own` and the predicate returns true immediately. So the configuration is already live on the path carrying MORE region-keyed bookkeeping, not less.
  That is evidence and not proof, and it changes what must be answered first: not "is widening safe" but **"why is the scalar path narrower than the slice path that already does this"**. Until that is answered, widening would be changing a borrow-checker predicate without knowing why it was narrow — and this project's rule is that a check is removed only by a machine-verified proof, never by an argument that it looks unnecessary.

## 2026-08-09 — CORRECTION: my archive-gate ruling was unsatisfiable and would have corrupted a prefix's meaning
- owner: lead correction. The executing agent refused to implement the ruling and reported why instead of back-filling the ledger to fit it. That refusal is the finding.
- WHAT I RULED, at `137af3e`: a version carrying an `ARCHIVE-SPEC:` line is at its versioned path, the one version carrying only an `ACTIVE-SPEC:` line is at the stable path, and the gate asserts exactly one recorded version lacks an `ARCHIVE-SPEC:` line.
- WHY IT CANNOT WORK, measured by the lead's own commands: `governance/APPROVALS.md` carries **15 `ACTIVE-SPEC:` lines (v0.9–v0.23) and 9 `ARCHIVE-SPEC:` lines (v0.0–v0.8) over version sets that are DISJOINT** — `comm -12` on the two sorted sets returns nothing. **The prefixes mark PROVENANCE, not location.** `ACTIVE-SPEC:` is the approved activation chain, one line per activation, retained forever; it means "this version was activated", never "this version is active now". `ARCHIVE-SPEC:` is an after-the-fact `shasum` of pre-chain specifications that never had exact-byte approval, and the ledger's own prose says they carry a different prefix *because* of that difference in provenance.
  So today **fifteen** versions lack an `ARCHIVE-SPEC:` line, not one. And the shape is unsatisfiable in either direction: back-fill v0.9–v0.22 and exactly one version lacks the line, but the resolution rule then sends v0.23 to a stable file that does not exist; back-fill v0.23 as well and zero versions lack it, so the assertion reds. **Worse than unsatisfiable, it would have made one prefix mean two unrelated things** — and given future activation-written lines, for owner-approved bytes, the prefix whose stated meaning is "not an approval, measured after the fact".
- RULED INSTEAD, adopting the executor's shape, which keeps both properties I actually wanted: **make the assertion conditional on the stable file's existence, and take the discriminator from the stable file's own version token rather than from a line prefix.**
  `spec/kernel-spec.md` ABSENT → every recorded version must have `spec/kernel-spec-<version>.md`. Today that is 24 versions and it is exactly the check that already runs, so it is **correct and non-vacuous now**, which is what the original instruction demanded and my shape could not deliver.
  `spec/kernel-spec.md` PRESENT → **exactly one** recorded version lacks a versioned file; the stable file's own first-line version token must name that version; and it must hash to that version's recorded digest.
  Zero still means the stable file was never installed. Two still means an archive was forgotten. Nothing depends on which line is last, so the order-independence that drove the original ruling survives. And **no ledger line is written to make a gate green**, which is why stopping was right: back-filling the record to satisfy a gate is the exact shape `CLAUDE.md` names a governance breach.
- ACCEPTED, from the same round: §2's closing sentence — "These bytes are non-authoritative until … active-target installation complete" — cannot stand beside `Status: ACTIVE vN`, and restating it as a condition on installation rather than deleting it keeps the closure record. It changes a sentence every prior version carried, which is why it was raised rather than absorbed. Also accepted: condition 8's target is §2's proposed SPECIFICATION header, not the candidate document's own header as my ruling said; both were corrected, so the conflation changed no action.
- CORRECTIONS TO THE RECORD, both self-reported: the §4 anchor is **547 bytes, not the 470** first reported — an extraction one wrapped line short, whose conclusion survives because a substring matching once implies its superstring matches at most once. And the anchor check is now **whole-line exact rather than substring**, because a substring test passes just as happily when the specification's paragraph has grown a clause the candidate does not know about.
- AND THE ONE I WOULD KEEP IF ONLY ONE SURVIVED: proving the grammar check can fail, the executor's **first attempt at the deliberate break changed zero lines**, so its green tested nothing — caught by the diff count, not by the result line. That is the emptied-test class arriving inside the verification built to prevent it, found by the person building it.

## 2026-08-10 — OWNER PROCESS APPROVAL: high-level plans and autonomous task execution
- owner: **the owner, in conversation, 2026-08-10** — “我的想法是一个计划可以拆成很多个执行item，没执行的放在planned里面，执行中的放在ongoing里面。这些都不需要我批准，你拿了一个以后就可以去执行，执行完了以后吧文档放到合适的目录里就好了。” The owner further required approval for “大的方向性计划” and said that when work needs “修改spec，或者需要修改conformance之类的重要合规测试，那么这需要向我说明，解释，然后得到我的审批。” After the resulting workflow model was restated, the owner instructed: “好的。那你可以更新一下workflow。它也有点过长了，把有用的留下来”。
- process boundary: `docs/current-plan.md` is one owner-approved high-level undertaking that may decompose into many autonomous `planned` / `ongoing` / `done` tasks. The lead may register and execute bounded prerequisites, repairs, and research side tasks without another owner round when they support rather than change that plan. A material change to the plan's direction, boundary, acceptance, or stop condition returns to `PROPOSED` and owner review.
- protected boundary: every specification batch and every addition or change to protected conformance or equivalent compliance evidence remains explanation-first and requires explicit owner approval of the named candidate boundary. Ordinary compiler implementation, ordinary tests, task lifecycle, integration, documentation, and closure inside the active plan do not.
- historical effect: this replaces the rolling one-small-step interpretation that repeatedly sought owner selection for Stage-level execution items. Historical plan, specification, and protected-evidence approvals remain records of their own exact changes; this entry approves no new language or conformance byte.

## 2026-08-17 — OWNER APPROVAL: batch 0068 merge, P1 gate wiring, batch-loop process law
- owner: **the owner, in conversation, 2026-08-17** — "可以，开干吧。……不过在这之前似乎要把目前的分支提到main。我看了你之前给我的那个文档，感觉ok。" The reply followed the batch review
  document (docs/done/0068) enumerating every commit and its approval class,
  and the explain-code walkthrough page of the complete main..spec-rework
  diff.
- APPROVED: merging branch spec-rework to main at merge commit `657abaf`;
  the P1 protected gate-wiring commits (`196ce0f` candidate mode,
  `526225e` candidate composition, `75899f2` sub-id coverage semantics,
  `d1cb4a1`/`09e4cff` gate collection changes ordered by the owner in
  conversation); the root-entry law restoration (`4bf6f79`); the batch-loop
  process law replacing per-task lifecycle coordination (`6f2e6f8`),
  direction stated by the owner: 快速推进,轻流程,批尾对抗审计兜底.
- NOT APPROVED BY THIS ENTRY: activation of the v0.30 candidate. The
  candidate rides main as a declared candidate under the lineage rules; its
  activation requires a separate exact-byte approval naming SHA-256
  db2b4b6906f6309a4fe04568fa5c2beb0fecfae72405591872e6e9c6c70c5ef2.
- boundary: the merge commit and the named commits; no conformance verdict
  changed in this batch; the coverage-semantics commit restores the
  pre-batch denominator (133) rather than altering it.

## 2026-08-17 — OWNER EXACT-BYTE APPROVAL: v0.30 structured representation profile
- owner: **the owner, in conversation, 2026-08-17** — "批了", replying to the
  explicit activation request naming candidate SHA-256
  `db2b4b6906f6309a4fe04568fa5c2beb0fecfae72405591872e6e9c6c70c5ef2`, after
  reviewing the batch 0068 document and the full-diff walkthrough page.
- APPROVED BYTES: the v0.30 candidate at that digest. Content: the
  structured representation migration — header changelog evicted,
  sentence-per-line, [ENT-3.Sk] sub-ids, wf- fences, stale self-reference
  and plan-vocabulary sweeps. No semantic change from v0.29: 133 rules,
  126 operation-table rows byte-identical, grammar unchanged, coverage
  denominator and split unchanged; content preservation proven by the
  one-shot transform verifier recorded at commit 0ba366d.
- ACTIVATION DELTA: per the candidate-mode workflow, activation replaced
  exactly the one declared status line
  `Status: CANDIDATE v0.30 supersedes v0.29 0b7aa8cc…` with
  `Status: ACTIVE v0.30 (2026-08-17; structured representation profile; no
  semantic change from v0.29).` and changed nothing else; the resulting
  active digest recorded in the chain is
  `5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1`.
- boundary: the stable file, its new v0.29 archive (verified against the
  prior chain tail before installation), the chain line below, identity
  constants and generated module, and the prose authority lines the
  digest-sync gate enumerates. No conformance verdict changes.
ACTIVE-SPEC: v0.30 5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1 0b7aa8ccee958ba85613c51535165dcbf7ac12db556b2210d2f1aac0d39e6cc3

## 2026-08-18 — OWNER APPROVAL: batch 0070 merge and its protected conformance candidates
- owner: **the owner, in conversation, 2026-08-18** — "好了。现在0070批准",
  replying after the batch review in docs/ongoing/0070 (morning review,
  ten-agent exit audit with dispositions, final state lib 908/908, adapter
  Pass=460 Skip=1 Fail=0, coverage 134/134) and the full explain-code
  walkthrough page of the batch, with an extended design Q&A on SET-2,
  OP-2, O11, and the reborrow extension conducted in conversation.
- APPROVED: merging branch caps-batch to main at merge commit `4f71b224`;
  the protected conformance candidate commits it carries — `915eeab3`
  (repair three defective cases), `2db1a3cf` (op3-neg-exact-dotted verdict
  corrected to cite OP-1), `a314189a` (promote verified pending cases to
  runnable), `21434167` (migrate 14 cases to the v0.31 arithmetic rule),
  `f53b52a6` (stor1 case rename off the now-reserved name), `1c8d873a`
  (16 new v0.31 cases), and the O11 positive rewrite with its visible
  `ent3-neg-bor-no-comparison-origin` removal — plus the spec-candidate
  amendment `7ceb6ba2`.
- DECISIONS FOLDED INTO THIS APPROVAL (defaults as prepared): DELTA-DIAG1
  not applied (retained as a recorded research delta); the conformance
  adapter `#[ignore]` wiring untouched (reason text corrected only);
  enum-const families remain DEFERRED with recorded reasons.
- NOT APPROVED BY THIS ENTRY: activation of the v0.31 candidate, which is
  the separate exact-byte entry below.

## 2026-08-18 — OWNER EXACT-BYTE APPROVAL: v0.31 batch-0070 capability delta
- owner: **the owner, in conversation, 2026-08-18** — "好了。现在0070批准",
  the batch approval whose recorded single act (docs/ongoing/0070, "Owner
  batch approval = the merge") is the merge plus this activation, naming
  candidate SHA-256
  `6657c3420bef8678be2e43bdf941d6db0095a4ccfece46c557beba26d09b07f4`.
- APPROVED BYTES: the v0.31 candidate at that digest. Content: [SET-2]
  affine-place replacement (`replace_let_rhs`), the [ENT-6]
  constant-operand overflow obligation family, the reborrow extension
  (signature-determined provenance, grandchild composition), [ENT-3]
  signed Boolean decomposition, one-operation const arithmetic with
  struct-typed consts, and the #35 position-conditional repair; rule
  inventory 134, grammar 74/96/99 two-path verified.
- ACTIVATION DELTA: per the candidate-mode workflow, activation replaced
  exactly the one declared status line
  `Status: CANDIDATE v0.31 supersedes v0.30 5ed21019…` with
  `Status: ACTIVE v0.31 (2026-08-18; batch-0070 capability delta: [SET-2]
  affine-place replacement, the [ENT-6] constant-operand overflow
  obligation family, the reborrow extension, [ENT-3] signed Boolean
  decomposition, and one-operation const arithmetic with struct-typed
  consts; rule inventory 134).` and changed nothing else; the resulting
  active digest recorded in the chain is
  `ea4b8ad4a56fbf43f3c98b91fc667da0b693c75b81807250a36454e03a197f1c`.
- boundary: the stable file, its new v0.30 archive (verified against the
  prior chain tail before installation), the chain line below, identity
  constants and generated module, and the prose authority lines the
  digest-sync gate enumerates. Conformance changes ride the batch-0070
  merge entry above, not this entry.
ACTIVE-SPEC: v0.31 ea4b8ad4a56fbf43f3c98b91fc667da0b693c75b81807250a36454e03a197f1c 5ed210190737b2aa53a91dc901f07d02344669eeb6d6660224602872331204d1

## 2026-08-18 — OWNER APPROVAL: batch 0071 merge and its protected conformance candidates
- owner: **the owner, in conversation, 2026-08-18** — "批准", replying to the
  batch review and owner packet in docs/ongoing/0071 (morning review, the
  four-finder adversarial exit audit with every disposition, final state
  lib 952/0, adapter Pass=489 Skip=1 Fail=0, coverage 135/135, canonical
  corpus 3/3). The overnight direction that authorized the work without
  blocking was the owner's, 2026-08-18: "把代码,测试,所有的事情都推进到位".
- APPROVED: merging branch batch-0071 to main at merge commit `73ede975`;
  the protected conformance candidate commits it carries — `819850e9`
  (138 leg-A migrations), `7e8d03f4` (EX-1 reproduced byte-for-byte),
  `de91fffb` (OP-5 to CLM-1 citations), `7409c7cd` (11 subject + 6
  trap-expecting decision cases), `6d1e7929` (7 refutation-guard repairs),
  `951f8221` (26 new cases and the two verdict flips), `188306f3`,
  `f55d4c13`, `6445b932`, `d2c1e209`, `39746f41` (audit repairs, SYS-14
  observability, strict-in-U and borrowed-arena coverage).
- DECISIONS APPROVED WITH THE PACKET: (1) ratification of the traversal
  plan expansion — the ACTIVE plan's W2 authorized three deltas and W1 a
  gap report, and the lead folded the [SYS-14] surface into the same
  candidate under the overnight no-blocking direction; (2) the #50 rule
  widening beyond the owner's literal sentence (a same-region parameter of
  the other borrow kind, and any parameter whose written type names the
  result region, also leave the source undetermined), on v0.31's own
  soundness argument; (3) the five verdict flips and seventeen citation
  rows enumerated as decision rows in the marked commit messages; (4) the
  trap-case consolidation onto the one legal always-false claim spelling.
- NOT APPROVED BY THIS ENTRY: activation of the v0.32 candidate, which is
  the separate exact-byte entry below.

## 2026-08-18 — OWNER EXACT-BYTE APPROVAL: v0.32 trap-endpoint and traversal delta
- owner: **the owner, in conversation, 2026-08-18** — "批准", the batch
  approval whose recorded single act (docs/ongoing/0071, "THE SINGLE ACT")
  is the merge plus this activation, naming candidate SHA-256
  `efaf0ec4e2d7c31518f4e817faa55fcb412f8a8cec542883b4c051917b06e1f3`.
- APPROVED BYTES: the v0.32 candidate at that digest. Content: check
  dissolution (#47) — the body `check_stmt` leaves the [GRAM-4] statement
  alternation while the production survives as the contract final admitted
  at `requires_entry`/`ensures_entry`, [OP-5] refits, [ENT-3.S2] retires
  into a self-contained S3; the [ENT-6] divisor-class division obligation
  family (#48) with signed two-variable sites retaining their trap because
  the safe condition is a disjunction the fragment cannot state; the
  [FN-1] declaration-site borrow-result provenance judgment (#50) with
  [OWN-6]'s binding-side rejection deleted as unreachable; and the
  [SYS-14] directory-traversal surface (nominals +2, constructors +3,
  operations +3, 192 declaration records). Rule inventory 135 (+[SYS-14]);
  grammar 74/96/99 two-path verified with three right-hand sides changed.
- ACTIVATION DELTA: per the candidate-mode workflow, activation replaced
  exactly the one declared status line
  `Status: CANDIDATE v0.32 supersedes v0.31 ea4b8ad4…` with
  `Status: ACTIVE v0.32 (2026-08-18; batch-0071 trap-endpoint and
  traversal delta: …; rule inventory 135).` and changed nothing else; the
  resulting active digest recorded in the chain is
  `5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5`.
- boundary: the stable file, its new v0.31 archive (verified against the
  prior chain tail before installation), the chain line below, identity
  constants and generated module, and the prose authority lines the
  digest-sync gate enumerates. Conformance changes ride the batch entry
  above, not this entry.
ACTIVE-SPEC: v0.32 5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5 ea4b8ad4a56fbf43f3c98b91fc667da0b693c75b81807250a36454e03a197f1c

## 2026-08-20 — OWNER APPROVAL: batch 0073 v0.33 exact-candidate preparation boundary
- owner: **the owner, in conversation, 2026-08-20** — “批准准备 v0.33 exact
  candidate，范围如下”, followed by the six exact scope clauses recorded
  below. This is preparation authority, not final-byte, activation, or merge
  approval.
- SPECIFICATION CANDIDATE: approve drafting `spec/kernel-spec.md` so QUAL-2
  states all four target guarantees, including `DirectoryRelativeResolution`;
  synchronize `spec/derivation/derivation-ledger.md` and regenerate
  `compiler/src/spec.rs` and `compiler/src/spec_identity.rs` for the resulting
  candidate bytes.
- TARGET REVIEW: approve moving the one `REVIEWED_FOR` tripwire in
  `compiler/src/backend/qualification.rs` from `operation_row` to the
  unconditional `command_entry_row`, reviewing and bumping it to v0.33, and
  adding ordinary regressions without changing the selected target rows,
  symbols, or ABIs.
- PROTECTED CORPUS: base tree
  `5a7bb1e2446742f749ba9794cd356b7225da17a0`; approve modifying or renaming the
  489 existing `.wf` cases other than `type6-neg-dup-variant.wf`, whose exact
  sorted path-set SHA-256 is
  `2dd889d79b4470906aa43c1987ffdf1cf233af80eda5aa520d95a9ddda5ba070`, and
  synchronizing `tests/conformance/manifest.jsonl`.
- PROTECTED ADDITIONS: approve exactly
  `v033-neg-missing-result-binding`, `v033-pos-shared-contract-define`,
  `v033-pos-uninhabited-contract`, `v033-neg-exact-domain-unproved`,
  `v033-run-exact-domain-claim`, `v033-neg-allocation-fit-unproved`,
  `v033-run-system-nonzero-next`, `v033-run-open-file-regular`, and
  `v033-run-open-file-directory`.
- EQUIVALENT EVIDENCE: approve synchronizing dependent endpoint, effect, and
  bridge baselines in `compiler/src/semantic/tests/provenance.rs`, and the
  optimizer-tail-merged provisional-close expectation in
  `compiler/src/backend/tests/cost_shape.rs`.
- EXCLUDED: no conformance runner, collection, or gate wiring change; no
  specification activation, `ACTIVE-SPEC:` record, or archive; and no merge to
  main. The final candidate bytes, SHA-256, complete diff, and verifier results
  return for a separate owner decision.
- SUPPLEMENTAL CANDIDATE AUTHORITY: after receiving the complete protected
  before/after inventory, the owner restated on 2026-08-20 that work on the
  isolated branch was authorized without intermediate approval — “分枝上你随便
  改不要等我审批” — while retaining the final activation/merge approval. This
  authorizes the enumerated candidate-only dispositions: four SYS-8 runtime
  trap rows become static SYS-8 rejections; the SYS-14 portable record uses its
  approved little-endian u16 length; retired program-kind and outside-kind
  subjects are renamed/reclassified under FORM-1, FN-7, SYS-2, or TYPE-5 as
  reported; old arithmetic-trap subjects become explicit retained `.defined`
  claims; endpoint and claim identities/docs are synchronized; and the nine
  approved v0.33 additions receive the audit repairs reported in that same
  inventory. No path outside the approved 489-file set and nine additions is
  admitted by this supplement.
- STILL EXCLUDED: this supplement does not approve activation, archive or
  digest-chain installation, runner/collection/gate wiring, or merge to main.
  Those remain the final exact-candidate decision.

## 2026-08-20 — OWNER EXACT-BYTE APPROVAL: v0.33 claim-only runtime trap surface and static contracts
- owner: **the owner, in conversation, 2026-08-20** — “批准以候选 SHA-256
  024a7752a88daf8799f637d95401fb73e25e257b118b3b78d4733b397c3db3c2
  激活 Whitefoot v0.33，并按 WORKFLOW 原子完成：归档 v0.32、切换 ACTIVE
  状态、写入激活链、重新生成 identity/grammar、落地 protected evidence、
  关闭 batch 0073 并合入 main。generic-container 不在本次合入；0073
  关闭后再以 0074 开始。”
- APPROVED BYTES: the complete v0.33 candidate at SHA-256
  `024a7752a88daf8799f637d95401fb73e25e257b118b3b78d4733b397c3db3c2`,
  3,204 lines and 395,671 bytes. Its native grammar identity is 74
  productions, 93 decisions, and 105 terminal predicates. The exact protected
  evidence is the audited 499-source/499-manifest bijection with 135/135 rule
  coverage, `Pass=498 Skip=1 Fail=0`, protected tree
  `882e691cf456758c456509a057ab6328c1f58a88`, and manifest SHA-256
  `f6b7cda7d523837c5ae1ddf3115ac82afabc1d13d9a4e7ddff4a24591b85c609`.
- ACTIVATION DELTA: per the candidate-mode workflow, activation replaces
  exactly the declared candidate status line with `Status: ACTIVE v0.33` and
  changes no other approved specification content. The resulting active bytes
  are 3,204 lines and 395,586 bytes at independently recomputed SHA-256
  `fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.
- ATOMIC BOUNDARY: archive byte-identical outgoing v0.32 as
  `spec/kernel-spec-v0.32.md`; install the active bytes and chained record;
  regenerate specification identity and grammar data; land the already
  approved protected and equivalent evidence; update current authority prose;
  move batch record 0073 from `docs/ongoing/` to `docs/done/`; and fast-forward
  main only after the post-activation gate is green. The generic-container
  direction, batch 0074, and any new plan are explicitly outside this act.
ACTIVE-SPEC: v0.33 fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f 5ea3927aef20d08e1c9c80a50242628f2c469974261b68c696ee2db3934e6bf5

## 2026-08-22 — merge-time approval content: four-rule governance and conformance wiring
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That one exact-revision
  approval is both rule 2's merge approval and rule 4's approval of the content
  below; this record creates no separate approval step.
- SPECIFICATION: `spec/kernel-spec.md` is byte-identical to `main`, SHA-256
  `fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`.
- CONFORMANCE BOUNDARY: five existing files are modified; no conformance file
  is added, deleted, or renamed, and no case source, expected verdict, runnable
  status, rule assignment, or program-case documentation changes.
  - `Makefile`:
    `6b821cded2c9f4dad4e187e06ed95d8b195684f1077ff2e59b50b8d5086735c4`
    -> `d75a8f4c0e04cd51871fcd71aa966730af6716158dbfb4123e9fd3d63ed008e9`.
  - `compiler/tests/conformance/adapter.rs`:
    `e0539b329fcd1f1d9aee1c995fe420101b26de8f3f390b1a1ffcb418dd12d270`
    -> `e4c2447d15ffd39f1a73b5267ef8e2a024dffd2afca445931a13de7de5e305c1`.
  - `tests/conformance/manifest.jsonl`:
    `f6b7cda7d523837c5ae1ddf3115ac82afabc1d13d9a4e7ddff4a24591b85c609`
    -> `4728c7da805223043f120898354ec172d083680c7e7db29e27784a11bb8340ff`.
  - `tests/conformance/runner.py`:
    `3e6ee1913de54d14db7e1c5f26ac61cd22e55991fe4fb8584f8ae09596ee909e`
    -> `368a341ac0c57fdb12ca597545f1cb1a8628b92a1caac28ebc44f0111b4b6bfa`.
  - `tests/conformance/test_runner.py`:
    `71560dbb81bdcdc43317bc9c4996b64b742ad50f8489bdab2de14554274545ac`
    -> `42bc4a0ad2908c59693fa85107466f5856987ad21c9f2fc2e054ddcac38d3890`.
- CONTENT: root `make check` now invokes the complete native adapter and every
  maintained executable test; the adapter's ignored-test contract documents
  that wiring; manifest annotations describe machine-checkable coverage
  without adding a review workflow; the verdict-diff command reports additions
  as well as moves and removals; and its new regression proves an addition exits
  nonzero and is printed. The exact revision's successful `make check` result
  is the rule 3 evidence for this same boundary.

## 2026-08-22 — merge-time approval content: v0.34 claim locality
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That one exact-revision
  approval is both rule 2's merge approval and rule 4's approval of the content
  recorded here; this record creates no separate approval step.
- SPECIFICATION: activate Whitefoot v0.34 at exact SHA-256
  `cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03`.
  It supersedes active v0.33 at SHA-256
  `fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.33.md`.
- CONFORMANCE BOUNDARY: relative to merge base
  `0728d7c66e45832d132ae499066a0da8c6865896`, the exact
  `tests/conformance` Git tree changes from
  `d1fa24b618eb7296e62c0d9b59818839bf9cc277` to
  `a481fb614a26dd12cb87e5056e0a6bff5d959937`. The no-renames
  name/status stream has SHA-256
  `5f453a0270fe0b58f4120bcce1a3a59184efc8b3562ca504a78bbef6f6c740f9`:
  198 existing case sources and the manifest are modified, two case sources
  are added, and nothing is deleted or renamed.
  - The cases tree changes from `576ef3f77d397630bb8a7acb8667bf84906120af`
    to `152918ce85c50f58b83b17f73224def0213d4b71`; the manifest SHA-256
    changes from `4728c7da805223043f120898354ec172d083680c7e7db29e27784a11bb8340ff`
    to `8170ce7a90da9172040a0ccfb86859eb912d532c45be60f67335b2c330a3ec0d`.
  - Added `clm1-neg-system-result-claim-locality.wf` at SHA-256
    `d106b2a730418655f318efe608f42e98327d70e4157886d7648792282bf98c7a`
    and `clm1-neg-user-result-claim-locality.wf` at SHA-256
    `9b3708aa992817209be909b472e745e41fd9e0a6594500304d87e07e10a23278`;
    both reject at CLM-1.
  - Fourteen declared verdicts move: `accept-sysname-near-lookalike` and
    `ent1-pos-instantiation-judged-at-value` from accept to run 0;
    `clm1-trap-false-claim-aborts` and
    `clm1-trap-false-claim-not-refutable` from trap to reject CLM-2;
    `clm2-pos-redundant-claim-advisory` from accept to reject CLM-2; and
    `clm1-trap-runtime-violation`, `eff4-pos-trap-aborts`,
    `err4-trap-claim-domain-violation`, `scope4-pos-claim-traps`,
    `x-arith-claim-catches-wrapped-overflow-traps`,
    `x-arith-idiv-trap-signed-two-variable-traps`,
    `x-arith-loop-multiply-defined-claim-traps`,
    `x-form-form7-overflow-trap-canonical`, and
    `x-integ-loop-product-overflow-traps` from trap to run 0. No declared
    verdict is removed.
- CONFORMANCE CONTENT: parser-shaped claim statements change from 410 in 197
  files to 95 in 74 files. Runtime oracles become ordinary control or typed
  outcomes; false, redundant, and non-local claims become source errors or are
  replaced by true same-function residuals; all 62 claims in accepted/runnable
  sources have exact five-field records and real static consumers. The corpus
  grows from 499 to 501 cases and the native adapter's expected complete result
  is 500 passes, one intentional skip, and no failure. The exact revision's
  successful canonical `make check` is the rule 3 evidence for this same
  boundary.
ACTIVE-SPEC: v0.34 cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03 fc6b5a109e56b4bcd93d30ef934d3c78eca9bddafd640d30c10649e9ba62d08f

## 2026-08-23 — merge-time approval content: v0.35 proof-derived parallelism ([PAR-1], [PAR-2])
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. The owner's merge approval of
  2026-08-23 ("合并吧", repeated as "我是说你直接合并,我批准了" after the
  campaign report) is rule 2's merge approval and rule 4's approval of the
  content recorded here; this record creates no separate approval step.
- SPECIFICATION: activate Whitefoot v0.35 at exact SHA-256
  `645b22b19bdfcf51683b9b10c7fd9109fc4029e9687df30e09e871daf84eb769`.
  It supersedes active v0.34 at SHA-256
  `cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.34.md`. The activated body is the combined candidate
  of the loop-permission batch record (recipe digest
  `f99bb580eec570c62ee6df414aa324039d3b1e55b0cd72ec04b033d73e43bcfd` in
  CANDIDATE form; the ACTIVE digest above differs only by the status line),
  adding [PAR-1] v2 and [PAR-2]: 137 rules, grammar-preserving (74
  productions, 93 decisions, 105 terminal predicates, natively verified).
- CONFORMANCE BOUNDARY: relative to merge base
  `18d332e7`, the only `tests/conformance` change is two annotation lines
  appended to the annotation block of `manifest.jsonl` — the [PAR-1] and
  [PAR-2] coverage annotations whose exact bytes are recorded in
  `docs/ongoing/0078-loop-permission.md` (the lines carrying
  `"covered_by": "compiler-permission-judgment"`). The manifest SHA-256
  changes from
  `8170ce7a90da9172040a0ccfb86859eb912d532c45be60f67335b2c330a3ec0d` to
  `114a56c0a575d0dca2b97bfe6c9f0de873e237315646b640698956b90ed3cc0d`.
  No case source is added, modified, deleted, or renamed; coverage becomes
  137/137 with zero corpus delta.
- DEFAULT-BEHAVIOR FLAG, approved with this merge: an unset `WF_WORKERS`
  in a `--par` binary asks for one lane per logical CPU instead of running
  sequentially; explicit `0`/`1` keep the sequential world. The time and
  depth dimensions of the change are measured and recorded in
  `docs/ongoing/0077-night-par-ceiling.md` and
  `docs/ongoing/0078-loop-permission.md`.
ACTIVE-SPEC: v0.35 645b22b19bdfcf51683b9b10c7fd9109fc4029e9687df30e09e871daf84eb769 cb747505cb043ac0c71861f4fe2df0e159b7b877ff920bc7a31ec60c454ddb03

## 2026-08-24 — merge-time approval content: v0.36 loan-conditioned overlap permission ([PAR-1], [PAR-2] amendments)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step.
- SPECIFICATION: activate Whitefoot v0.36 at exact SHA-256
  `fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62`.
  It supersedes active v0.35 at SHA-256
  `645b22b19bdfcf51683b9b10c7fd9109fc4029e9687df30e09e871daf84eb769`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.35.md`. The rule count is unchanged at 137 and the
  grammar is untouched. [PAR-1] gains the loans half of its disjointness
  condition, the window-statement conditions its "exactly when" list never
  stated, the system-operation clause, and an abandoned-continuation
  sentence scoped to what survives an abort; [PAR-2] gains the same loans
  condition and combination-tree sentences that admit the identity-seeded,
  commutative split the compiler actually emits. The permitted-overlap set
  only narrows; no acceptance or conformance verdict moves.
- CONFORMANCE BOUNDARY: relative to the v0.35 activation boundary, no
  `tests/conformance` content is added, modified, deleted, or renamed;
  coverage remains 137/137 with zero corpus delta.
ACTIVE-SPEC: v0.36 fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62 645b22b19bdfcf51683b9b10c7fd9109fc4029e9687df30e09e871daf84eb769

## 2026-08-27 — merge-time approval content: v0.37 unified-state completion I/O ([EFF-5] rewrite, `effect_path`, [PAR-1] direct system calls)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step.
- SPECIFICATION: activate Whitefoot v0.37 at exact SHA-256
  `ee9f12ec9356267c13b536e962288ebbffa0b3507cfac0a5345f99e8dce53619`.
  It supersedes active v0.36 at SHA-256
  `fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.36.md`. Per the specification's own META-5 delta
  declaration: the numbered rule count is unchanged at 137, and runtime-trap
  families, entry forms, contract block forms, and exception clauses are
  unchanged. Grammar productions are +1/-0 (75 remain): `reads` and `writes`
  stop accepting REGIONID, and the new `effect_path` production admits only
  formal-rooted static struct paths. Unique fixed lowercase grammar atoms are
  net -2 because `external` and `blocks` become ordinary IDENT spellings.
  Writer operation spellings are +4/-3: `read_at`, `open_directory_source`,
  and `directory_next` replace the cursor-shaped spellings, and `reserve_file`
  is added. Two opaque system nominal spellings are added (`FileFactory` and
  `FilePermit`) while one changes from `DirectoryList` to `DirectorySource`.
  System operations are +1 and declaration records +4, from 199 to 203,
  because positioned read adds `file_offset`, each of four open operations
  adds one `permit` parameter, file reservation adds one operation, one
  region, and one parameter, and the backend-only `Interrupted` and
  `WouldBlock` outcome constructors and their four fields are removed.
  Existing effect, system, release, trap, and overlap rules are amended:
  [EFF-5] is rewritten so ordinary owned places, exact effect paths, and loans
  decide coexistence rather than a global sequential external-call order, and
  direct system calls participate in [PAR-1] under the same effect, loan,
  dataflow, and exit permission as user calls. No writer-visible world region,
  capability category, blocking call family, callback, task, future, or
  scheduling marker is added. The selection ground is the first-principles
  derivation in `research/investigations/io-model/FIRST-PRINCIPLES.md`.
- CONFORMANCE BOUNDARY: relative to the v0.36 activation boundary at `main`
  tip `eab81a335addfb0ae060735771d4e98891dec2ea`, `tests/conformance` content
  changes as follows. Added: two case files,
  `tests/conformance/cases/accept-sysfile-two-permits-shared-directory.wf`
  and `tests/conformance/cases/reject-sysfile-permit-used-twice.wf`.
  Modified: ninety-six files, being ninety-five case files under
  `tests/conformance/cases/` and `tests/conformance/manifest.jsonl`. Deleted:
  none. Renamed: none. Within `tests/conformance/manifest.jsonl`, two case
  records are added for the two added cases and twenty-one records are
  modified in place, taking the case-record count from 421 to 423 with no
  record removed. The twenty-one in-place modifications are three
  rule-annotation records and eighteen case records. The rule-annotation
  changes are: [CAP-1] moves from `gated-family` to `compiler-ownership-model`
  because the rule now states that no separate capability category exists;
  [SYS-7]'s portable class count changes from thirty to twenty-eight with the
  removal of the `Interrupted` and `WouldBlock` constructors; and [EFF-5]'s
  annotation reason is rewritten from sequential external-call ordering to
  coexistence decided by owned places, exact effect paths, and loans, observed
  by the runnable redirected-output case. The eighteen case-record changes are
  rule-list and doc-text updates for the renamed operations
  (`read_once`/`list_once` to `read_at`/`directory_next`), the
  `DirectoryList`-to-`DirectorySource` spelling change, the explicit `Args`
  state path in the effect rows, the `FilePermit` parameter on the open
  operations, and the five-row command standard-input table; no `expect`
  verdict changes in any of them. Coverage remains 137/137 rules.
ACTIVE-SPEC: v0.37 ee9f12ec9356267c13b536e962288ebbffa0b3507cfac0a5345f99e8dce53619 fd57cfc4bfcf685f14b073c98e149c8a44a201dc79fbd76075ebd49a87995c62

## 2026-08-27 — merge-time approval content: v0.38 staged loop permission ([PAR-3])
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step.
- SPECIFICATION: activate Whitefoot v0.38 at exact SHA-256
  `5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d`.
  It supersedes active v0.37 at SHA-256
  `ee9f12ec9356267c13b536e962288ebbffa0b3507cfac0a5345f99e8dce53619`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.37.md`. Per the specification's own META-5 delta
  declaration: numbered rules are +1/-0, from 137 to 138, the added rule being
  [PAR-3]. Grammar productions are unchanged at 75, and unique fixed lowercase
  grammar atoms, writer operation spellings, opaque system nominal spellings,
  runtime-trap families, entry forms, contract block forms, exception clauses,
  and the 203 system operations and declaration records are all unchanged.
  [PAR-3] is a second permission over the iterations of a loop, distinct from
  [PAR-2]'s counted permission and sharing none of its apparatus: it cuts the
  body of any `for_stmt` or `loop_stmt` at the first `may-suspend` submission
  and admits executing the remainder of one iteration with overlapping
  execution against the prologue of a later one, requiring no accumulator,
  combination tree, identity element, or index range. Its conditions are formed
  entirely from [PAR-1]'s existing footprint and loan machinery; it admits
  replicating a copy-element place under a byte-coverage condition; it states
  that prologue executions do not overlap one another and that the remainder's
  writes to storage rooted outside the body, together with its reads of such
  storage the body writes, occur in iteration order; it places the edge the
  submitting statement takes on that submission's own outcome in the remainder
  rather than the prologue, so a `propagate` at the cut leaves from the
  remainder; it distinguishes the host resources a system operation of the loop
  creates from the execution resources an implementation spends on overlapping;
  and it carries [PAR-1]'s erroneous-execution clauses, including that no
  correct path reads a trap latch. Its replication clause states which bytes a
  footprint *may* write and which count as written: a contract that fixes only
  which bytes of a buffer may have changed establishes no written byte, so the
  coverage condition can be met only from a contract that states the change
  exactly. [SYS-8] states the may-have-changed form for the range-bearing
  operations and the exact form for the copy pair, and this sentence is what
  keeps the coverage condition from reading the first as the second. [PAR-1]
  and [PAR-2] each gain one cross-reference sentence and no condition. [SYS-2]
  is amended in one sentence to name the release milestone of the name an open
  borrows — `open_read`'s `path`, `open_file`'s and `open_directory`'s `name` —
  which is published before target transfer because forming the request copies
  the admitted code-unit range into compiler-owned storage; every other
  applicable fact is still published by the exactly-once terminal transition.
  That is one entry in the milestone structure [SYS-2] already carries, and it
  is the fact both shipped adapters already publish at submission. No other
  ownership, effect, release, or trap rule changes, and no writer-visible
  depth, window, queue, batch operation, task, future, callback, cancellation
  handle, or scheduling marker is added. The [PAR-3] judgment shipped in this
  same revision does not consume that earlier milestone: it reads every
  retained borrow as retained to `terminal`, which is strictly conservative
  against the amended contract and is the only direction a permission judgment
  may err in. No accepted program becomes rejected: the permitted-overlap set
  only widens, so no conformance verdict moves. The selection ground is the
  staged loop-pipeline design derived against the completion model and the
  io-completion benchmark's own measurements.
- CONFORMANCE BOUNDARY: relative to the v0.37 activation boundary at `main`
  tip `79b2966562e1da8de541feedfd5855d0ef4a3c30`, `tests/conformance` content
  changes as follows. Added: seven case files under
  `tests/conformance/cases/`, being
  `accept-par3-staged-iteration-own-scratch.wf`,
  `accept-par3-staged-loop-with-prologue-break.wf`,
  `accept-par3-staged-denied-hoisted-scratch.wf`,
  `accept-par3-staged-denied-read-before-write.wf`,
  `accept-par3-staged-denied-carried-scratch-byte.wf`,
  `accept-par3-staged-denied-opaque-cursor.wf`, and
  `accept-par3-staged-denied-exit-in-remainder.wf`. Modified: one file,
  `tests/conformance/manifest.jsonl`. Deleted: none. Renamed: none. Within
  `tests/conformance/manifest.jsonl`, eight records are added and no record is
  modified or removed: one [PAR-3] rule-annotation record, and seven case
  records for the seven added cases, taking the case-record count from 503 to
  510 and the rule-annotation count from 34 to 35. Every added case declares
  the `accept` verdict, which is the only verdict a permission rule that
  refuses nothing can carry; each case's `rules` list names [PAR-3] beside the
  rules its source genuinely exercises. No pre-existing `expect` verdict
  changes. Coverage moves from 137/137 to 138/138 rules. Outside
  `tests/conformance`, exactly one conformance-evidence file changes:
  - `compiler/tests/conformance/adapter.rs`:
    `9f4d7d9661883d10c5c4004c38c7b1563d1932e34b8d7484bd98cff3684abce9`
    -> `0c640b5de84ac4f7026b92c4b832b5dc836439a1526c4f5edcf94cc7c682e063`
    (the native compile-and-run adapter links emitted modules through the
    driver's `HOST_LINK_LIBRARIES` constant, one link recipe with the shipped
    driver; no verdict logic changes).
ACTIVE-SPEC: v0.38 5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d ee9f12ec9356267c13b536e962288ebbffa0b3507cfac0a5345f99e8dce53619

## 2026-08-28 — merge-time approval content: v0.39 claim-authority narrowing ([CLM-1])
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step.
- SPECIFICATION: activate Whitefoot v0.39 at exact SHA-256
  `b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516`.
  It supersedes active v0.38 at SHA-256
  `5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.38.md`. Per the specification's own META-5 delta
  declaration: numbered rules are +0/-0 and remain 138; grammar productions
  remain 75; and unique fixed lowercase grammar atoms, writer operation
  spellings, opaque system nominal spellings, runtime-trap families, entry
  forms, contract block forms, exception clauses, and the 203 system
  operations and declaration records are all unchanged. The single changed
  content is one paragraph of [CLM-1]'s claim-authority state. Where v0.38
  joined a selector's witness into every binder, delivered value, and storage
  write whose reaching definition was selected by that edge — naming
  `value_if`, `value_match`, ordinary match, `give`, loop-carried updates, and
  post-join state — v0.39 joins it into each matching binder that edge's arm
  introduces, each value `value_if` or `value_match` delivers along it, and,
  at each ordinary match reconvergence, loop head, and loop exit the selector
  reaches, exactly those components whose reaching definition on one incoming
  edge is a different definition occurrence from their reaching definition on
  another. Standing on a boundary-selected edge is no longer itself a
  selection: an ordinary binding, computed value, or storage write whose own
  operands are every one Local is Local, inside the selected arm and in
  post-join state alike. The clause that a `value_if` or `value_match` delivery is selected in
  every case is retained verbatim in effect, so selecting constants on the two
  arms or the same local value on both arms still does not declassify the
  delivered value. No other [CLM-1] clause changes, no [CLM-2] or [CLM-3]
  clause changes, and [PRV-1] provenance — which never included control
  dependence — is untouched, as are the component tree, the unconditional
  call-result seed, the witness identity and tie-break, the protected
  families, and the constrained subjects. This is an accepted-set widening:
  every component keeps at least the authority a previously accepted program
  gave it, so no accepted program becomes rejected, while a program whose
  claim was refused only because a boundary result selected the edge reaching
  a purely local definition is now accepted. The selection ground is
  evidence-selected: the differential-fuzz campaign recorded in
  `docs/done/0097-differential-fuzz.md`, all 63 of whose rejections over 2004
  generated programs were that one shape, and whose minimized pair differs by
  a single `return` line.
- CONFORMANCE BOUNDARY: relative to `main` tip
  `10b76c66a02bfff21d9842fd9b1f35d0b9855ce4`, `tests/conformance` content
  changes as follows. Added: seven case files under
  `tests/conformance/cases/`, being
  `accept-clm1-local-claim-after-boundary-exit.wf`,
  `accept-clm1-local-claim-after-boundary-join.wf`,
  `accept-clm1-local-claim-inside-selected-arm.wf`,
  `reject-clm1-claim-on-selected-payload.wf`,
  `reject-clm1-claim-on-delivered-selection.wf`,
  `reject-clm1-claim-on-storage-written-under-selection.wf`, and
  `reject-clm1-claim-on-loop-carried-update.wf`. Modified: one file,
  `tests/conformance/manifest.jsonl`. Deleted: none. Renamed: none. Within
  `tests/conformance/manifest.jsonl`, seven records are added and no record is
  modified or removed, taking the case-record count from 510 to 517 and
  leaving the rule-annotation count at 35. No pre-existing `expect` verdict
  changes, and coverage stays at 138/138 rules. The three added `accept`
  cases are the verdict the amended [CLM-1] sentence moves: the two members of
  the differential-fuzz minimized pair, which v0.38 separated by one `return`
  line, and a claim over a parameter-derived value inside a selected arm. The
  four added `reject` cases are the selections the amended sentence retains:
  the matching binder the arm introduces, a `value_if` delivery whose two arms
  both give a literal below the bound, storage one arm wrote and the other did
  not, and loop-carried state a boundary-selected counted loop updated. Every
  pre-existing case was additionally compiled with both the base compiler at
  `100b37cd` and this revision's compiler: all 510 produce byte-identical LLVM
  IR apart from the QUAL-1 banner's version token, or byte-identical
  diagnostics. Outside `tests/conformance`, no conformance-evidence file
  changes: the adapter, the runners, and the collection wiring are untouched.
ACTIVE-SPEC: v0.39 b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516 5a43c7638bd5839d77829836518374f9a169eb953d9c1edbd66b87815aedfb2d

## 2026-09-03 — merge-time approval content: v0.40 source-carried proof ([INV-1], [PRF-1]; [CLM-1..3], [PRV-1..3], [TRAP-1], [DIAG-3], [SCOPE-4] retired)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step, and nothing in this record asserts
  that the approval has been given.
- SPECIFICATION: activate Whitefoot v0.40 at exact SHA-256
  `15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168`.
  It supersedes active v0.39 at SHA-256
  `b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.39.md`. Per the specification's own META-5 delta
  declaration: numbered rules are +2/-9 and 131 remain, the added rules being
  [INV-1] and [PRF-1] and the retired ids, never reused, being CLM-1, CLM-2,
  CLM-3, DIAG-3, PRV-1, PRV-2, PRV-3, SCOPE-4, and TRAP-1; grammar productions
  are +8/-1 and 82 remain, the added productions being `affine_add_op`,
  `affine_expr`, `affine_factor`, `affine_term`, `for_binding`,
  `header_invariant`, `invariant_stmt`, and `proof_use` and the removed one
  being `claim_stmt`; unique fixed lowercase grammar atoms are +2/-4 and 54
  remain, adding `invariant` and `use` and removing `because`, `claim`,
  `deny_claims`, and `traps`; runtime-trap families are +0/-1 and 0 remain;
  entry forms are +0/-1 and 1 remains; and writer operation spellings, opaque
  system nominal spellings, contract block forms, exception clauses, and the
  203 system operations and declaration records are unchanged. The changed
  content replaces v0.39's runtime claim path with one source-carried proof
  surface: every writer-reachable partial operation is proved before
  execution, so the claim statement, its exact runtime record, the claim trap,
  and with them the `deny_claims` entry form are gone. One proof-only
  `invariant` declaration serves both a loop header and an ordinary program
  point; a counted `for` header encloses its binding followed by zero or more
  induction relations, an ordinary `loop` may enclose only induction
  relations, neither form admits a trailing comma, and a header relation
  carries no certificate block. A local invariant may carry ordered `use`
  steps with explicit proof-domain multipliers and named earlier invariants;
  each use is proved independently in the same entering state, adds nothing to
  that state, and publishes only the checked outer target. The automatic
  affine rule is complete for exactly the direct, every coefficient-one
  single-premise, every coefficient-one unordered premise-pair including
  self-pairs, and final closed-L0-image families, and larger or specially
  weighted combinations are writer-directed. Exact operation and subscript
  value identities enter later Goals only after their own nested domain
  obligations succeed, with occurrence-local evaluated-value identities
  covering values outside that admitted structural fragment. Value images,
  source facts, branch joins, pre-kill closure, counted exhaustion, function
  requirements and postconditions, and operation domains consume one
  deterministic source ProofContext; selected-target layout and address and
  parallel permission keep their own deterministic checker domains and consume
  the conclusions source proof retains rather than repeating it or creating a
  second source-acceptance authority. Every admitted family runs to its
  specification-fixed completion without solver, heuristic stopping,
  elapsed-time verdict, cumulative work budget, runtime fallback, or
  compiler-generated proof replay. The selection ground the specification
  states is that every writer-reachable partial operation must be proved
  before execution, while resource availability remains the explicitly
  deferred boundary [SCOPE-3].
- CONFORMANCE BOUNDARY: relative to `main` tip
  `4b08c7589839c9dbf20f8e7a678bac0c062b5fb0`, `tests/conformance` content
  changes as follows. Under `tests/conformance/cases/`, 34 files are added,
  102 are modified, 60 are deleted, and 5 are renamed with their content
  changed; git's own rename detection reports exactly those five pairings.
  Because a case id is its file name, each renamed file retires one declared
  id and introduces another, so at the level of declared case ids the boundary
  is 39 added and 65 removed. Added:
  - `tests/conformance/cases/ent3-pos-s5-set-commit-image.wf`
  - `tests/conformance/cases/ent5-pos-close-before-scope-kill.wf`
  - `tests/conformance/cases/ent5-pos-close-before-set-strengthening.wf`
  - `tests/conformance/cases/ent5-pos-project-middle-before-set.wf`
  - `tests/conformance/cases/fn8-neg-external-actual-without-fact.wf`
  - `tests/conformance/cases/fn8-pos-external-actual-after-branch.wf`
  - `tests/conformance/cases/form3-neg-retired-mode-suffix.wf`
  - `tests/conformance/cases/inv1-neg-backedge-unproved.wf`
  - `tests/conformance/cases/inv1-neg-base-unproved.wf`
  - `tests/conformance/cases/inv1-neg-repeated-name.wf`
  - `tests/conformance/cases/inv1-pos-automatic-two-premise-backedge.wf`
  - `tests/conformance/cases/inv1-pos-ordinary-loop-cursor-set-image.wf`
  - `tests/conformance/cases/inv1-pos-ordinary-loop-guarded-cursor.wf`
  - `tests/conformance/cases/op2-pos-division-requires-nonzero.wf`
  - `tests/conformance/cases/op2-pos-guarded-subtraction-l0-bridge.wf`
  - `tests/conformance/cases/op2-pos-unsigned-literal-division-images.wf`
  - `tests/conformance/cases/op4-neg-external-index-without-fact.wf`
  - `tests/conformance/cases/op4-pos-external-index-after-branch.wf`
  - `tests/conformance/cases/op4-pos-midpoint-automatic.wf`
  - `tests/conformance/cases/prf1-neg-duplicate-use.wf`
  - `tests/conformance/cases/prf1-neg-explicit-factor-one.wf`
  - `tests/conformance/cases/prf1-neg-fact-killed-by-write.wf`
  - `tests/conformance/cases/prf1-neg-inexact-combination.wf`
  - `tests/conformance/cases/prf1-neg-redundant-use-block.wf`
  - `tests/conformance/cases/prf1-neg-unproved-premise.wf`
  - `tests/conformance/cases/prf1-pos-explicit-affine-proof.wf`
  - `tests/conformance/cases/prf1-pos-explicit-factor.wf`
  - `tests/conformance/cases/prf1-pos-integer-tightening-midpoint.wf`
  - `tests/conformance/cases/run-invariant-exact-sum.wf`
  - `tests/conformance/cases/s7-pos-signed-remainder-ranges-run.wf`
  - `tests/conformance/cases/s7-pos-unsigned-remainder-bound-run.wf`
  - `tests/conformance/cases/x-eff-allocating-fn-calls-only-pure.wf`
  - `tests/conformance/cases/x-eff-pure-combined-with-allocation.wf`
  - `tests/conformance/cases/x-eff-pure-fn-calls-allocating-fn.wf`
  Modified:
  - `tests/conformance/cases/accept-par3-staged-denied-carried-scratch-byte.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-exit-in-remainder.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-hoisted-scratch.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-opaque-cursor.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-read-before-write.wf`
  - `tests/conformance/cases/accept-par3-staged-iteration-own-scratch.wf`
  - `tests/conformance/cases/accept-sysrelease-return-unit-declared.wf`
  - `tests/conformance/cases/const1-neg-eval-overflow.wf`
  - `tests/conformance/cases/const1-neg-noninteger.wf`
  - `tests/conformance/cases/const2-neg-set.wf`
  - `tests/conformance/cases/eff1-neg-wrong-order-row.wf`
  - `tests/conformance/cases/eff2-neg-declared-unexhibited.wf`
  - `tests/conformance/cases/eff2-neg-undeclared-exhibited.wf`
  - `tests/conformance/cases/eff2-pos-discharged-division-site-pure-row.wf`
  - `tests/conformance/cases/eff3-pos-pure-fn.wf`
  - `tests/conformance/cases/ent2-neg-counted-subscript-endpoint.wf`
  - `tests/conformance/cases/ent3-pos-band-check-decomposition.wf`
  - `tests/conformance/cases/ent3-pos-s11-counted-range-run.wf`
  - `tests/conformance/cases/ent4-pos-contradictory-state-discharges.wf`
  - `tests/conformance/cases/ent4-pos-disequality-strengthens.wf`
  - `tests/conformance/cases/ent4-pos-transitivity-discharges.wf`
  - `tests/conformance/cases/ent5-neg-kill-on-write.wf`
  - `tests/conformance/cases/ent5-pos-join-keeps-common-bound.wf`
  - `tests/conformance/cases/ent5-pos-value-if-delivery-join.wf`
  - `tests/conformance/cases/err2-neg-missing-variant.wf`
  - `tests/conformance/cases/err3-neg-error-type-mismatch.wf`
  - `tests/conformance/cases/err3-pos-propagate.wf`
  - `tests/conformance/cases/err4-pos-recoverable-value.wf`
  - `tests/conformance/cases/fn3-neg-signature-effect-mismatch.wf`
  - `tests/conformance/cases/fn5-pos-match-dispatch.wf`
  - `tests/conformance/cases/fn6-neg-polymorphic-recursion.wf`
  - `tests/conformance/cases/fn7-neg-two-mains.wf`
  - `tests/conformance/cases/fn8-neg-requires-noncopy-local.wf`
  - `tests/conformance/cases/form1-neg-unknown-construct.wf`
  - `tests/conformance/cases/form3-neg-opname-bad-suffix.wf`
  - `tests/conformance/cases/form3-neg-typeid-fn-name.wf`
  - `tests/conformance/cases/form4-neg-comment.wf`
  - `tests/conformance/cases/form5-neg-missing-suffix.wf`
  - `tests/conformance/cases/form7-neg-leading-zero.wf`
  - `tests/conformance/cases/form7-neg-out-of-range.wf`
  - `tests/conformance/cases/gram9-neg-nested-call.wf`
  - `tests/conformance/cases/op1-neg-written-argument-on-deargumented-row.wf`
  - `tests/conformance/cases/op2-neg-division-constant-zero-divisor.wf`
  - `tests/conformance/cases/op2-neg-division-obligation-undischarged.wf`
  - `tests/conformance/cases/op2-neg-overflow-obligation-undischarged.wf`
  - `tests/conformance/cases/op2-pos-division-obligation-discharged.wf`
  - `tests/conformance/cases/op2-pos-overflow-obligation-discharged.wf`
  - `tests/conformance/cases/op2-pos-wrap-untouched-by-dissolution.wf`
  - `tests/conformance/cases/op4-neg-index-undischarged.wf`
  - `tests/conformance/cases/op4-pos-index-discharged.wf`
  - `tests/conformance/cases/op9-pos-buffer-new.wf`
  - `tests/conformance/cases/own1-neg-index-atom-after-move.wf`
  - `tests/conformance/cases/prog1-pos-closed-unit.wf`
  - `tests/conformance/cases/reject-gram11-unnamed-call.wf`
  - `tests/conformance/cases/reject-sys14-list-end-beyond-buffer.wf`
  - `tests/conformance/cases/reject-syshost-copybytes-end-beyond-buffer.wf`
  - `tests/conformance/cases/reject-syshost-copybytes-start-after-end.wf`
  - `tests/conformance/cases/run-syshost-copybytes-toosmall-unchanged.wf`
  - `tests/conformance/cases/run-syshost-copyutf8-invalid-unchanged.wf`
  - `tests/conformance/cases/run-syshost-copyutf8-toosmall-unchanged.wf`
  - `tests/conformance/cases/run-syspath-dotdot-preserved.wf`
  - `tests/conformance/cases/scope2-pos-accepted.wf`
  - `tests/conformance/cases/set1-neg-counted-binder-write.wf`
  - `tests/conformance/cases/sys14-list-handle-affine.wf`
  - `tests/conformance/cases/sys14-list-handle-unique.wf`
  - `tests/conformance/cases/sys14-no-path-from-bytes.wf`
  - `tests/conformance/cases/type2-pos-twostate-enum-i1.wf`
  - `tests/conformance/cases/type5-neg-index-offset-type.wf`
  - `tests/conformance/cases/type5-pos-index-element-type-derived.wf`
  - `tests/conformance/cases/type6-pos-distinct-names.wf`
  - `tests/conformance/cases/type7-neg-index-box-holder.wf`
  - `tests/conformance/cases/type7-neg-index-reference-holder.wf`
  - `tests/conformance/cases/x-arith-idiv-reject-constant-zero-divisor.wf`
  - `tests/conformance/cases/x-array-const-checksum-run.wf`
  - `tests/conformance/cases/x-array-mutable-checksum-run.wf`
  - `tests/conformance/cases/x-base64-rfc-vectors-run.wf`
  - `tests/conformance/cases/x-buffer-borrowed-columns-run.wf`
  - `tests/conformance/cases/x-buffer-mutable-checksum-run.wf`
  - `tests/conformance/cases/x-child-reborrow-run.wf`
  - `tests/conformance/cases/x-crc32-standard-vector-run.wf`
  - `tests/conformance/cases/x-eff-dup-reads-effect.wf`
  - `tests/conformance/cases/x-eff-trailing-comma-row.wf`
  - `tests/conformance/cases/x-eff-writes-missing-region.wf`
  - `tests/conformance/cases/x-enum-borrow-payload-live.wf`
  - `tests/conformance/cases/x-enum-multiwidth-dispatch.wf`
  - `tests/conformance/cases/x-enum-payload-give.wf`
  - `tests/conformance/cases/x-enum-stmt-payload-check.wf`
  - `tests/conformance/cases/x-fn-cross-fn-call-chain.wf`
  - `tests/conformance/cases/x-fn-mutual-recursion-runs.wf`
  - `tests/conformance/cases/x-integ-coin-borrow-match-score-twice.wf`
  - `tests/conformance/cases/x-integ-give-in-statement-match-rejected.wf`
  - `tests/conformance/cases/x-match-err2-value-match-missing-err.wf`
  - `tests/conformance/cases/x-match-give1-give-in-stmt-match.wf`
  - `tests/conformance/cases/x-match-gram10-binder-not-fresh.wf`
  - `tests/conformance/cases/x-match-gram10-out-of-order-fields.wf`
  - `tests/conformance/cases/x-option-byte-scanner-run.wf`
  - `tests/conformance/cases/x-requires-output-capacity-run.wf`
  - `tests/conformance/cases/x-result-buffer-transform-run.wf`
  - `tests/conformance/cases/x-struct-construct-read-field.wf`
  - `tests/conformance/cases/x-struct-nested-field.wf`
  - `tests/conformance/cases/x-struct-of-buffers-checksum-run.wf`
  - `tests/conformance/cases/x-struct-set-field.wf`
  Deleted:
  - `tests/conformance/cases/accept-clm1-local-claim-after-boundary-exit.wf`
  - `tests/conformance/cases/accept-clm1-local-claim-after-boundary-join.wf`
  - `tests/conformance/cases/accept-clm1-local-claim-inside-selected-arm.wf`
  - `tests/conformance/cases/clm1-neg-repeated-claim-name.wf`
  - `tests/conformance/cases/clm1-neg-system-result-claim-locality.wf`
  - `tests/conformance/cases/clm1-neg-user-result-claim-locality.wf`
  - `tests/conformance/cases/clm1-pos-passing-claim-establishes-fact.wf`
  - `tests/conformance/cases/clm1-trap-false-claim-aborts.wf`
  - `tests/conformance/cases/clm1-trap-false-claim-not-refutable.wf`
  - `tests/conformance/cases/clm1-trap-runtime-violation.wf`
  - `tests/conformance/cases/clm2-neg-refuted-claim.wf`
  - `tests/conformance/cases/clm2-pos-redundant-claim-advisory.wf`
  - `tests/conformance/cases/clm3-neg-body-check-bounds.wf`
  - `tests/conformance/cases/clm3-neg-body-check-requires.wf`
  - `tests/conformance/cases/clm3-neg-direct-unreachable-claim.wf`
  - `tests/conformance/cases/clm3-neg-generic-first-import.wf`
  - `tests/conformance/cases/clm3-neg-mutual-scc-import.wf`
  - `tests/conformance/cases/clm3-neg-static-conjunction-unproved.wf`
  - `tests/conformance/cases/clm3-neg-transitive-check-summary.wf`
  - `tests/conformance/cases/clm3-pos-transitive-value-branch.wf`
  - `tests/conformance/cases/clm3-pos-upward-near-miss.wf`
  - `tests/conformance/cases/eff1-pos-pure-and-traps-rows.wf`
  - `tests/conformance/cases/eff2-neg-division-class-site-pure-row.wf`
  - `tests/conformance/cases/eff2-neg-propagate-hidden-trap.wf`
  - `tests/conformance/cases/eff2-pos-declared-traps-exhibited.wf`
  - `tests/conformance/cases/eff2-pos-propagate-declared-trap.wf`
  - `tests/conformance/cases/eff4-pos-trap-aborts.wf`
  - `tests/conformance/cases/fn8-neg-requires-missing-traps.wf`
  - `tests/conformance/cases/fn8-neg-strict-outside-caller-unproved-requirement.wf`
  - `tests/conformance/cases/form3-neg-reserved-mode-field.wf`
  - `tests/conformance/cases/op2-pos-division-claim-backstop.wf`
  - `tests/conformance/cases/prv1-pos-control-write-address-nontaint.wf`
  - `tests/conformance/cases/prv1-pos-payload-sibling-isolation.wf`
  - `tests/conformance/cases/prv2-neg-complete-only-postcondition.wf`
  - `tests/conformance/cases/prv2-neg-direct-system-result.wf`
  - `tests/conformance/cases/prv2-neg-entry-system-result-bridge.wf`
  - `tests/conformance/cases/prv2-neg-mutual-demand.wf`
  - `tests/conformance/cases/prv2-neg-nonexact-goal.wf`
  - `tests/conformance/cases/prv2-neg-recursive-demand.wf`
  - `tests/conformance/cases/prv2-neg-two-hop-bridge.wf`
  - `tests/conformance/cases/prv2-pos-allocation-equality-call.wf`
  - `tests/conformance/cases/prv2-pos-postcondition-b-summary.wf`
  - `tests/conformance/cases/prv2-pos-seedless-mutual.wf`
  - `tests/conformance/cases/prv3-neg-external-claim-conjunction.wf`
  - `tests/conformance/cases/prv3-neg-external-claim.wf`
  - `tests/conformance/cases/prv3-neg-read-offset-taint.wf`
  - `tests/conformance/cases/prv3-pos-external-bound-only.wf`
  - `tests/conformance/cases/prv3-pos-external-branch.wf`
  - `tests/conformance/cases/prv3-pos-internal-claim.wf`
  - `tests/conformance/cases/reject-clm1-claim-on-delivered-selection.wf`
  - `tests/conformance/cases/reject-clm1-claim-on-loop-carried-update.wf`
  - `tests/conformance/cases/reject-clm1-claim-on-selected-payload.wf`
  - `tests/conformance/cases/reject-clm1-claim-on-storage-written-under-selection.wf`
  - `tests/conformance/cases/run-ex2-loop-exact-claims.wf`
  - `tests/conformance/cases/scope4-pos-claim-traps.wf`
  - `tests/conformance/cases/v033-run-exact-domain-claim.wf`
  - `tests/conformance/cases/x-arith-idiv-trap-signed-two-variable-traps.wf`
  - `tests/conformance/cases/x-eff-pure-combined-with-traps.wf`
  - `tests/conformance/cases/x-eff-pure-fn-calls-traps-fn.wf`
  - `tests/conformance/cases/x-eff-traps-fn-calls-only-pure.wf`
  Renamed:
  - `tests/conformance/cases/err4-trap-claim-domain-violation.wf` -> `tests/conformance/cases/err4-checked-domain-violation.wf`
  - `tests/conformance/cases/x-arith-claim-catches-wrapped-overflow-traps.wf` -> `tests/conformance/cases/x-arith-wrapping-overflow-observed.wf`
  - `tests/conformance/cases/x-arith-loop-multiply-defined-claim-traps.wf` -> `tests/conformance/cases/x-arith-loop-checked-multiply-overflow.wf`
  - `tests/conformance/cases/x-form-form7-overflow-trap-canonical.wf` -> `tests/conformance/cases/x-form-form7-checked-overflow-canonical.wf`
  - `tests/conformance/cases/x-integ-loop-product-overflow-traps.wf` -> `tests/conformance/cases/x-integ-loop-product-checked-overflow.wf`
  No retained case id changes its declared verdict:
  `python3 tests/conformance/runner.py verdicts origin/main` reports 0 moved,
  65 removed, 39 added, 491 total. Within `tests/conformance/manifest.jsonl`,
  case records move from 517 to 491 and rule-annotation records from 35 to 32.
  Declared verdict totals move from 79 accept, 265 reject, and 173 run to 69
  accept, 244 reject, and 178 run. Coverage moves from 138/138 rules to
  131/131 rules: the active corpus covers 112 rules by case and 32 rules by
  annotation, the two channels together covering all 131 rules and overlapping
  where both evidence forms apply, against 115 by case and 35 by annotation
  over v0.39's 138 rules. Every removed id is a case of the claim,
  claim-locality, provenance, or runtime-trap surface the retired rules
  defined, including the `traps` effect rows and the `deny_claims` strict-caller
  path that surface carried. The 39 added ids are the [INV-1] header- and
  body-invariant cases, the [PRF-1] written-proof cases, the entailment,
  contract, remainder-range, and subscript cases for facts a proof must now
  supply before a partial operation, effect-row cases restated over allocation
  rather than traps, the [FORM-3] case for the removed `.trap` suffix, and the
  checked or observed successors of the five renamed overflow cases. The 102
  modified cases keep their ids and their declared verdicts; their sources are
  migrated to the v0.40 surface.
  Outside the case directory, the exact conformance runner, adapter,
  collection, and gate-wiring boundary is:
  - `Makefile`:
    `5fe097a9d4d07657ecb89537ac22b6ef500d6bc97f05c0e68609d8fb85536f97`
    -> `b2a0a74fc7690540adc76eded00db3d173dc4814fa5f6a83e0cf6bdf133818e1`
    (the `conformance-run` and core-limit comments drop the retired trap
    expectation; no stage, target, or dependency changes);
  - `compiler/Makefile`:
    `7c3646b4db551dc8b230ca3501e62beab4bb035fcca55059fb0cbec5df59176d`
    -> `f599c1e0084581e994bcd3f7b76d53a7e06ef0b25f46f64ba34825b38e235d77`
    (the sampling-module list drops the deleted `backend::tests::trap_latch`
    module and the core-limit comment is restated);
  - `compiler/tests/canonical_corpus.rs`:
    `7ab174fa181233b01827f60f8f2d650c0fd23d8c56504b67924413633bd59fac`
    -> `939738cd729ea87f5280519bf9f80220bba044959d4f2ed5f0e23add492d65b0`
    (the exclusion-coverage test drops the retired `Trap` expectation from the
    expectations it enumerates; the [EX-1] selection rule is unchanged);
  - `compiler/tests/conformance/adapter.rs`:
    `0c640b5de84ac4f7026b92c4b832b5dc836439a1526c4f5edcf94cc7c682e063`
    -> `b022470c0bdb440a672bdcd2007d07706dc30de2534bbd999b4fe1df5ba76893`
    (an invocation that ends without an exit status is now a harness stop
    rather than a `Trap` verdict; nothing else selects a path);
  - `compiler/tests/conformance/corpus.rs`:
    `a2e4818ba9e4f4e2b079eb53635c9842397800af1339b974ab461003f22f9e9c`
    -> `1c5bde97f65f564f72ef6bc1fff5de11793b3e938e38ef3fd79669208a19e5b4`
    (the `Trap` verdict and `{"kind":"trap"}` expectation leave the parsed
    verdict space; the remaining match rule is unchanged);
  - `tests/conformance/manifest.jsonl`:
    `96fab2a2261355309d1a667aa220d7e8d96b76ad8d59453cec9ec257d34a0bcd`
    -> `2f01b2601f36295f19d02e8e053a4d82b98e482834e1fcae5e0f72c4028ba5d1`;
  - `tests/conformance/runner.py`:
    `368a341ac0c57fdb12ca597545f1cb1a8628b92a1caac28ebc44f0111b4b6bfa`
    -> `32ffd7607c1d23fcb19813c6d81a08b1ba02f7a6ceb4e1083af57821a242edef`
    (the `trap` verdict kind and its schema row are removed, so an `arrange`
    belongs to a `run` case only);
  - `tests/conformance/test_runner.py`:
    `42bc4a0ad2908c59693fa85107466f5856987ad21c9f2fc2e054ddcac38d3890`
    -> `46beed39eb0c235b914d56d203d2545c8e5c605df66960254891fafa4709e3f4`
    (the arrangement tests follow that schema change).
  No other conformance adapter, runner, collection, manifest, gate-integrity,
  or gate-wiring file changes; `compiler/tests/conformance.rs` and
  `compiler/tests/conformance/json.rs` are byte-identical to `main`.
ACTIVE-SPEC: v0.40 15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168 b4d8e01eecd81bdda9c632093873d604ddfbd64d979a4884472907e456d69516

## 2026-09-03 — merge-time approval content: v0.41 integer comparison symbols and the call-site `::` delimiter (21 rules respelled; none added or retired)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step, and nothing in this record asserts
  that the approval has been given.
- SPECIFICATION: activate Whitefoot v0.41 at exact SHA-256
  `899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761`.
  It supersedes active v0.40 at SHA-256
  `15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.40.md`. The candidate that preceded activation declared
  `CANDIDATE v0.41 supersedes v0.40` and hashed to
  `55ee571e7f342471b16078da05fa8b3bfdab11fbe6819a925baad83687966c06`;
  activation flipped that status line to `ACTIVE v0.41` and changed no other
  byte. Per the specification's own META-5 delta declaration: numbered rules
  are +0/-0 and 131 remain; grammar productions are +1/-0 and 83 remain, the
  added production being `compare_op`; unique fixed lowercase grammar atoms
  are +0/-0 and 54 remain; compound punctuation tokens are +5/-0 and 8
  remain, the added tokens being `==`, `!=`, `<=`, `>=`, and `::`; token
  bytes are +1/-0, `!` entering the alphabet inside `!=` only; writer
  operation spellings are +6/-6, the six integer comparisons `ieq` `ine`
  `ilt` `ile` `igt` `ige` becoming `==` `!=` `<` `<=` `>` `>=` and thereby
  leaving `DotlessOperationNames` and `ReservedLowerNames`; and opaque system
  nominal spellings, runtime-trap families (0 remain), entry forms (1
  remains), contract block forms, the 203 system operations and declaration
  records, and exception clauses are unchanged. The changed content carries
  the second FLOOR-5 spelling batch under the owner rulings of 2026-09-03.
  Integer comparison joins integer arithmetic as an `infix` expression: one
  `compare_op` over two atoms, integer-only exactly as the arithmetic symbols
  are, while float and tag-only enum comparison, the bit family, the shifts,
  and every unary operation keep their prefixed names. A call writes its type
  and region arguments after the `::` delimiter, `cvt::<u8, u32>(w)` and
  `open_file::<'f, 'n>(...)`, so that `IDENT "<"` begins only a comparison and
  every grammar decision keeps its two-token bound; constructors and type
  position are unchanged, and DIAG-1's row-1 and row-2 attributions are keyed
  on `(IDENT, "::")`. In proof position the four ordered symbols replace
  `ile`, `ilt`, `ige`, and `igt` as the relation of a header invariant, a
  local invariant, and a relation-form use step; a multiplied relation-form
  use step is parenthesized, `use 3 * (a <= b);`, and a bare one is not;
  equality and disequality are not invariant relations. FORM-2 gives `::`
  both attachments, a `compare_op` `<` or `>` neither, and the stated space
  between a use multiplier and its `(`. Twenty-one rules and the header move
  at forty-one verbatim-anchored sites recorded in
  `governance/spec-evolution/comparison-symbols-v041-candidate.md`. No rule's
  semantics changes: every comparison origin, contract-clause root, invariant
  relation, and diagnostic attribution is keyed on the same operation
  identities under their new spellings, and the accepted-program set is
  unchanged up to respelling. The selection ground the specification states
  is evidence selection under the FLOOR-5 spelling rule (T1–T4 and its
  measured tiebreaks): the six integer comparisons are the most frequent
  operation class in the corpus, the positional comparison call was the last
  direction-sensitive positional form, v0.40's proof surface had already made
  the ordered comparisons relations over infix affine operands, and the `<`
  collision that cancelled the v0.23 comparison row is dissolved by the `::`
  delimiter without widening any parser decision beyond two tokens.
- CONFORMANCE BOUNDARY: relative to `main` tip
  `a97edf9ae0f3cbd01813c115db9041f997b0aa2b`, `tests/conformance` content
  changes as follows. Under `tests/conformance/cases/`, 5 files are added,
  275 are modified, none are deleted, and none are renamed. Added:
  - `tests/conformance/cases/gram4-neg-multiplied-use-relation-bare.wf`
  - `tests/conformance/cases/gram5-neg-type-application-without-delimiter.wf`
  - `tests/conformance/cases/gram5-pos-comparison-operators.wf`
  - `tests/conformance/cases/inv1-neg-equality-relation.wf`
  - `tests/conformance/cases/prf1-pos-multiplied-relation-use.wf`
  Modified:
  - `tests/conformance/cases/accept-par3-staged-denied-carried-scratch-byte.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-exit-in-remainder.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-hoisted-scratch.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-opaque-cursor.wf`
  - `tests/conformance/cases/accept-par3-staged-denied-read-before-write.wf`
  - `tests/conformance/cases/accept-par3-staged-iteration-own-scratch.wf`
  - `tests/conformance/cases/accept-par3-staged-loop-with-prologue-break.wf`
  - `tests/conformance/cases/accept-syseff-pure-immutable-only.wf`
  - `tests/conformance/cases/accept-sysfile-two-permits-shared-directory.wf`
  - `tests/conformance/cases/accept-sysname-near-lookalike.wf`
  - `tests/conformance/cases/const1-neg-eval-overflow.wf`
  - `tests/conformance/cases/const1-neg-noninteger.wf`
  - `tests/conformance/cases/const1-neg-runtime-arithmetic-mode.wf`
  - `tests/conformance/cases/const1-pos-array-size.wf`
  - `tests/conformance/cases/const1-pos-forwarded-arithmetic.wf`
  - `tests/conformance/cases/const2-pos-array-lookup.wf`
  - `tests/conformance/cases/const2-pos-struct-const.wf`
  - `tests/conformance/cases/eff2-pos-discharged-division-site-pure-row.wf`
  - `tests/conformance/cases/eff3-pos-pure-fn.wf`
  - `tests/conformance/cases/ent1-neg-instantiation-judged-at-value.wf`
  - `tests/conformance/cases/ent1-pos-instantiation-judged-at-value.wf`
  - `tests/conformance/cases/ent2-neg-expired-spelling-inherits-nothing.wf`
  - `tests/conformance/cases/ent2-neg-no-fact-across-call.wf`
  - `tests/conformance/cases/ent2-pos-array-length-survives-root-replace.wf`
  - `tests/conformance/cases/ent3-neg-stage8b-local-one.wf`
  - `tests/conformance/cases/ent3-pos-band-check-decomposition.wf`
  - `tests/conformance/cases/ent3-pos-bor-guard-decomposition.wf`
  - `tests/conformance/cases/ent3-pos-s1-branch-fact.wf`
  - `tests/conformance/cases/ent3-pos-s11-counted-range-run.wf`
  - `tests/conformance/cases/ent3-pos-s4-requires-fact.wf`
  - `tests/conformance/cases/ent3-pos-s5-set-commit-image.wf`
  - `tests/conformance/cases/ent3-pos-stage8b-bit-sources.wf`
  - `tests/conformance/cases/ent4-neg-nonstrict-bound-underivable.wf`
  - `tests/conformance/cases/ent4-pos-contradictory-state-discharges.wf`
  - `tests/conformance/cases/ent4-pos-disequality-strengthens.wf`
  - `tests/conformance/cases/ent4-pos-transitivity-discharges.wf`
  - `tests/conformance/cases/ent5-neg-callee-uniq-buffer-replace-kills-length.wf`
  - `tests/conformance/cases/ent5-neg-join-takes-weakest-bound.wf`
  - `tests/conformance/cases/ent5-neg-kill-on-write.wf`
  - `tests/conformance/cases/ent5-neg-loop-rule-drops-preloop-fact.wf`
  - `tests/conformance/cases/ent5-neg-value-match-no-delivery.wf`
  - `tests/conformance/cases/ent5-pos-close-before-scope-kill.wf`
  - `tests/conformance/cases/ent5-pos-close-before-set-strengthening.wf`
  - `tests/conformance/cases/ent5-pos-join-keeps-common-bound.wf`
  - `tests/conformance/cases/ent5-pos-project-middle-before-set.wf`
  - `tests/conformance/cases/ent5-pos-return-does-not-kill-loop-head-fact.wf`
  - `tests/conformance/cases/ent5-pos-value-if-delivery-join.wf`
  - `tests/conformance/cases/ent6-neg-join-one-arm-advances-accumulator.wf`
  - `tests/conformance/cases/ent6-pos-join-value-if-lifted-addend.wf`
  - `tests/conformance/cases/err1-pos-result-value-match.wf`
  - `tests/conformance/cases/err2-pos-exhaustive-match.wf`
  - `tests/conformance/cases/err3-pos-propagate.wf`
  - `tests/conformance/cases/err3-pos-propagation.wf`
  - `tests/conformance/cases/err4-pos-recoverable-value.wf`
  - `tests/conformance/cases/ex1-pos-worked-example.wf`
  - `tests/conformance/cases/fn1-neg-result-provenance-two-same-kind-called.wf`
  - `tests/conformance/cases/fn1-pos-result-provenance-distinct-regions.wf`
  - `tests/conformance/cases/fn1-pos-returned-slice-const-run.wf`
  - `tests/conformance/cases/fn1-pos-returned-slice-inputs-run.wf`
  - `tests/conformance/cases/fn1-pos-signature-driven-call.wf`
  - `tests/conformance/cases/fn2-neg-function-region-bearing-targ.wf`
  - `tests/conformance/cases/fn2-neg-wrong-kind-instantiation-argument.wf`
  - `tests/conformance/cases/fn3-neg-requires-member.wf`
  - `tests/conformance/cases/fn5-pos-match-dispatch.wf`
  - `tests/conformance/cases/fn6-neg-polymorphic-recursion.wf`
  - `tests/conformance/cases/fn6-pos-recursion.wf`
  - `tests/conformance/cases/fn7-pos-single-main.wf`
  - `tests/conformance/cases/fn8-neg-entry-contract.wf`
  - `tests/conformance/cases/fn8-neg-external-actual-without-fact.wf`
  - `tests/conformance/cases/fn8-neg-requires-control.wf`
  - `tests/conformance/cases/fn8-neg-requires-local-in-body.wf`
  - `tests/conformance/cases/fn8-neg-requires-move-operand.wf`
  - `tests/conformance/cases/fn8-neg-requires-no-check.wf`
  - `tests/conformance/cases/fn8-neg-requires-noncopy-cvt-local.wf`
  - `tests/conformance/cases/fn8-neg-requires-noncopy-local.wf`
  - `tests/conformance/cases/fn8-neg-requires-partial-op.wf`
  - `tests/conformance/cases/fn8-neg-requires-set.wf`
  - `tests/conformance/cases/fn8-neg-requires-user-call.wf`
  - `tests/conformance/cases/fn8-pos-external-actual-after-branch.wf`
  - `tests/conformance/cases/fn8-pos-requires-name-reuse.wf`
  - `tests/conformance/cases/fn8-pos-requires-run.wf`
  - `tests/conformance/cases/fn9-neg-entry-image-kill.wf`
  - `tests/conformance/cases/fn9-neg-named-outcome-no-publication.wf`
  - `tests/conformance/cases/fn9-neg-no-selected-normal-exit.wf`
  - `tests/conformance/cases/fn9-neg-same-scc-summary.wf`
  - `tests/conformance/cases/fn9-neg-unproved-selected-return.wf`
  - `tests/conformance/cases/fn9-pos-direct-set-receiver.wf`
  - `tests/conformance/cases/fn9-pos-ok-selected-receiver.wf`
  - `tests/conformance/cases/fn9-pos-plain-direct-result.wf`
  - `tests/conformance/cases/form3-pos-lexical-classes.wf`
  - `tests/conformance/cases/gram1-pos-lookahead.wf`
  - `tests/conformance/cases/gram10-pos-named-binders.wf`
  - `tests/conformance/cases/gram11-pos-named-args.wf`
  - `tests/conformance/cases/gram4-pos-stmts.wf`
  - `tests/conformance/cases/gram5-pos-exprs-places.wf`
  - `tests/conformance/cases/gram5-pos-recursive-place-projection.wf`
  - `tests/conformance/cases/gram6-pos-no-operators.wf`
  - `tests/conformance/cases/gram7-pos-two-productions.wf`
  - `tests/conformance/cases/gram9-neg-nested-call.wf`
  - `tests/conformance/cases/gram9-pos-three-address.wf`
  - `tests/conformance/cases/inv1-neg-backedge-unproved.wf`
  - `tests/conformance/cases/inv1-neg-base-unproved.wf`
  - `tests/conformance/cases/inv1-neg-repeated-name.wf`
  - `tests/conformance/cases/inv1-pos-automatic-two-premise-backedge.wf`
  - `tests/conformance/cases/inv1-pos-ordinary-loop-cursor-set-image.wf`
  - `tests/conformance/cases/inv1-pos-ordinary-loop-guarded-cursor.wf`
  - `tests/conformance/cases/op1-neg-enum-ordering.wf`
  - `tests/conformance/cases/op1-neg-ieq-bool.wf`
  - `tests/conformance/cases/op1-neg-ieq-tag-enum.wf`
  - `tests/conformance/cases/op1-neg-ine-bool.wf`
  - `tests/conformance/cases/op1-neg-ine-tag-enum.wf`
  - `tests/conformance/cases/op1-neg-written-argument-on-deargumented-row.wf`
  - `tests/conformance/cases/op1-pos-table-op.wf`
  - `tests/conformance/cases/op2-neg-div-wrap.wf`
  - `tests/conformance/cases/op2-pos-division-constant-divisor-total.wf`
  - `tests/conformance/cases/op2-pos-division-minus-one-divisor-bounded.wf`
  - `tests/conformance/cases/op2-pos-division-obligation-discharged.wf`
  - `tests/conformance/cases/op2-pos-division-remainder-same-obligation.wf`
  - `tests/conformance/cases/op2-pos-division-requires-nonzero.wf`
  - `tests/conformance/cases/op2-pos-guarded-subtraction-l0-bridge.wf`
  - `tests/conformance/cases/op2-pos-ineg-modes.wf`
  - `tests/conformance/cases/op2-pos-overflow-obligation-discharged.wf`
  - `tests/conformance/cases/op2-pos-sat-mode.wf`
  - `tests/conformance/cases/op2-pos-unsigned-literal-division-images.wf`
  - `tests/conformance/cases/op2-pos-wrap-untouched-by-dissolution.wf`
  - `tests/conformance/cases/op3-neg-exact-dotted.wf`
  - `tests/conformance/cases/op4-neg-external-index-without-fact.wf`
  - `tests/conformance/cases/op4-pos-external-index-after-branch.wf`
  - `tests/conformance/cases/op4-pos-midpoint-automatic.wf`
  - `tests/conformance/cases/op6-neg-cvt-identity.wf`
  - `tests/conformance/cases/op6-pos-cvt-checked.wf`
  - `tests/conformance/cases/op6-pos-cvt-total.wf`
  - `tests/conformance/cases/op7-neg-missing-prefix.wf`
  - `tests/conformance/cases/op7-pos-name-convention.wf`
  - `tests/conformance/cases/op8-pos-integer-family.wf`
  - `tests/conformance/cases/op8-pos-u64-shift-u32.wf`
  - `tests/conformance/cases/op9-pos-buffer-new.wf`
  - `tests/conformance/cases/own1-neg-bare-uniq-copy.wf`
  - `tests/conformance/cases/own1-neg-move-of-copy.wf`
  - `tests/conformance/cases/own1-pos-match-projected-copy.wf`
  - `tests/conformance/cases/own1-pos-tagonly-copy.wf`
  - `tests/conformance/cases/own10-pos-local-region.wf`
  - `tests/conformance/cases/own11-pos-loop-inner-region.wf`
  - `tests/conformance/cases/own12-neg-alias-uniq-args.wf`
  - `tests/conformance/cases/own12-pos-distinct-uniq-args.wf`
  - `tests/conformance/cases/own13-pos-let-match-give.wf`
  - `tests/conformance/cases/own2-pos-three-modes.wf`
  - `tests/conformance/cases/own3-pos-outlives-store.wf`
  - `tests/conformance/cases/own5-pos-read-through-holder.wf`
  - `tests/conformance/cases/own6-pos-callresult-borrow-chain.wf`
  - `tests/conformance/cases/own6-pos-callscoped-temp.wf`
  - `tests/conformance/cases/own7-pos-distinct-noverlap.wf`
  - `tests/conformance/cases/pre1-pos-prelude-enums.wf`
  - `tests/conformance/cases/prf1-neg-duplicate-use.wf`
  - `tests/conformance/cases/prf1-neg-explicit-factor-one.wf`
  - `tests/conformance/cases/prf1-neg-fact-killed-by-write.wf`
  - `tests/conformance/cases/prf1-neg-inexact-combination.wf`
  - `tests/conformance/cases/prf1-neg-redundant-use-block.wf`
  - `tests/conformance/cases/prf1-neg-unproved-premise.wf`
  - `tests/conformance/cases/prf1-pos-explicit-affine-proof.wf`
  - `tests/conformance/cases/prf1-pos-explicit-factor.wf`
  - `tests/conformance/cases/prf1-pos-integer-tightening-midpoint.wf`
  - `tests/conformance/cases/prog1-pos-closed-unit.wf`
  - `tests/conformance/cases/reject-sys14-list-end-beyond-buffer.wf`
  - `tests/conformance/cases/reject-syseff-declared-unexhibited.wf`
  - `tests/conformance/cases/reject-sysfile-permit-used-twice.wf`
  - `tests/conformance/cases/reject-syshost-copybytes-end-beyond-buffer.wf`
  - `tests/conformance/cases/reject-syshost-copybytes-start-after-end.wf`
  - `tests/conformance/cases/reject-syshost-copybytes-start-beyond-buffer.wf`
  - `tests/conformance/cases/run-ex1-value-match.wf`
  - `tests/conformance/cases/run-invariant-exact-sum.wf`
  - `tests/conformance/cases/run-sysarg-count-and-get.wf`
  - `tests/conformance/cases/run-sysdir-open-notfound.wf`
  - `tests/conformance/cases/run-sysfile-empty.wf`
  - `tests/conformance/cases/run-sysfile-exact.wf`
  - `tests/conformance/cases/run-sysfile-multichunk.wf`
  - `tests/conformance/cases/run-sysfile-short.wf`
  - `tests/conformance/cases/run-syshost-copybytes-toosmall-unchanged.wf`
  - `tests/conformance/cases/run-syshost-copyutf8-invalid-unchanged.wf`
  - `tests/conformance/cases/run-syshost-copyutf8-toosmall-unchanged.wf`
  - `tests/conformance/cases/run-syshost-nontext-argv-bytes-roundtrip.wf`
  - `tests/conformance/cases/run-syshost-nontext-argv-utf8-invalid.wf`
  - `tests/conformance/cases/run-sysout-basic-write.wf`
  - `tests/conformance/cases/run-sysout-redirect-same-sink-order.wf`
  - `tests/conformance/cases/run-syspath-absolute-rejected.wf`
  - `tests/conformance/cases/run-syspath-dotdot-preserved.wf`
  - `tests/conformance/cases/run-syspath-relative-basic.wf`
  - `tests/conformance/cases/s7-pos-signed-remainder-ranges-run.wf`
  - `tests/conformance/cases/s7-pos-unsigned-remainder-bound-run.wf`
  - `tests/conformance/cases/scope3-pos-defined-run.wf`
  - `tests/conformance/cases/set1-pos-local-and-field-copy.wf`
  - `tests/conformance/cases/set2-neg-arena-replace-target.wf`
  - `tests/conformance/cases/set2-neg-region-bearing-target.wf`
  - `tests/conformance/cases/set2-pos-affine-field-replace.wf`
  - `tests/conformance/cases/set2-pos-box-descriptor-replace.wf`
  - `tests/conformance/cases/stor1-pos-frame-resident.wf`
  - `tests/conformance/cases/stor2-pos-box-new.wf`
  - `tests/conformance/cases/stor3-pos-box-drop-region.wf`
  - `tests/conformance/cases/stor4-neg-arena-escape.wf`
  - `tests/conformance/cases/stor4-pos-arena-confined.wf`
  - `tests/conformance/cases/stor5-neg-arena-new-region-bearing.wf`
  - `tests/conformance/cases/sys14-directory-release.wf`
  - `tests/conformance/cases/sys14-entry-kind-closed.wf`
  - `tests/conformance/cases/sys14-list-handle-affine.wf`
  - `tests/conformance/cases/sys14-list-handle-unique.wf`
  - `tests/conformance/cases/sys14-list-outcome-exhaustive.wf`
  - `tests/conformance/cases/sys14-list-zero-range.wf`
  - `tests/conformance/cases/sys14-open-directory-component.wf`
  - `tests/conformance/cases/sys14-open-directory-empty-name.wf`
  - `tests/conformance/cases/sys14-open-directory-success.wf`
  - `tests/conformance/cases/sys2-neg-wrong-region-arg-count.wf`
  - `tests/conformance/cases/type1-pos-i32-unit.wf`
  - `tests/conformance/cases/type2-pos-enum.wf`
  - `tests/conformance/cases/type2-pos-twostate-enum-i1.wf`
  - `tests/conformance/cases/type4-pos-cvt.wf`
  - `tests/conformance/cases/type5-neg-shared-for-uniq-arg.wf`
  - `tests/conformance/cases/type5-neg-wrong-region-arg-count.wf`
  - `tests/conformance/cases/type6-pos-distinct-names.wf`
  - `tests/conformance/cases/type7-neg-match-reference-call.wf`
  - `tests/conformance/cases/type7-pos-deref.wf`
  - `tests/conformance/cases/v033-pos-shared-contract-define.wf`
  - `tests/conformance/cases/v033-pos-uninhabited-contract.wf`
  - `tests/conformance/cases/v033-run-open-file-directory.wf`
  - `tests/conformance/cases/v033-run-open-file-regular.wf`
  - `tests/conformance/cases/v033-run-system-nonzero-next.wf`
  - `tests/conformance/cases/x-arith-iadd-wrap-overflow-to-negative.wf`
  - `tests/conformance/cases/x-arith-isub-wrap-min-roundtrip-runs.wf`
  - `tests/conformance/cases/x-arith-loop-checked-multiply-overflow.wf`
  - `tests/conformance/cases/x-arith-wrapping-overflow-observed.wf`
  - `tests/conformance/cases/x-array-const-checksum-run.wf`
  - `tests/conformance/cases/x-array-mutable-checksum-run.wf`
  - `tests/conformance/cases/x-base64-rfc-vectors-run.wf`
  - `tests/conformance/cases/x-borrow-two-shared-reads-run.wf`
  - `tests/conformance/cases/x-borrow-uniq-shared-call-args-overlap.wf`
  - `tests/conformance/cases/x-borrowed-pool-tree-run.wf`
  - `tests/conformance/cases/x-buffer-borrowed-columns-run.wf`
  - `tests/conformance/cases/x-buffer-mutable-checksum-run.wf`
  - `tests/conformance/cases/x-child-reborrow-run.wf`
  - `tests/conformance/cases/x-const-scalar-u64-width.wf`
  - `tests/conformance/cases/x-crc32-standard-vector-run.wf`
  - `tests/conformance/cases/x-enum-borrow-payload-live.wf`
  - `tests/conformance/cases/x-enum-multiwidth-dispatch.wf`
  - `tests/conformance/cases/x-enum-payload-give.wf`
  - `tests/conformance/cases/x-enum-stmt-payload-check.wf`
  - `tests/conformance/cases/x-enum-twostate-result-payload.wf`
  - `tests/conformance/cases/x-fn-cross-fn-call-chain.wf`
  - `tests/conformance/cases/x-fn-mutual-recursion-runs.wf`
  - `tests/conformance/cases/x-fn-own-arg-for-ref-param.wf`
  - `tests/conformance/cases/x-form-form5-op-arg-missing-suffix.wf`
  - `tests/conformance/cases/x-give-result-aggregate.wf`
  - `tests/conformance/cases/x-gram-combo-flat-call-construct-match.wf`
  - `tests/conformance/cases/x-gram-nested-op-in-construct-field.wf`
  - `tests/conformance/cases/x-gram-nested-ucall-in-call-arg.wf`
  - `tests/conformance/cases/x-integ-checked-overflow-diverts-to-err.wf`
  - `tests/conformance/cases/x-integ-coin-borrow-match-score-twice.wf`
  - `tests/conformance/cases/x-integ-loop-product-checked-overflow.wf`
  - `tests/conformance/cases/x-integ-sign-weight-accumulate.wf`
  - `tests/conformance/cases/x-integ-traffic-light-state-machine.wf`
  - `tests/conformance/cases/x-match-give1-give-in-stmt-match.wf`
  - `tests/conformance/cases/x-match-give1-nested-value-match.wf`
  - `tests/conformance/cases/x-nominal-multifield-payload-run.wf`
  - `tests/conformance/cases/x-option-byte-scanner-run.wf`
  - `tests/conformance/cases/x-ownmove-copy-reused-affine-consumed-once.wf`
  - `tests/conformance/cases/x-requires-output-capacity-run.wf`
  - `tests/conformance/cases/x-result-buffer-transform-run.wf`
  - `tests/conformance/cases/x-struct-construct-read-field.wf`
  - `tests/conformance/cases/x-struct-cross-fn.wf`
  - `tests/conformance/cases/x-struct-mixed-width.wf`
  - `tests/conformance/cases/x-struct-nested-field.wf`
  - `tests/conformance/cases/x-struct-of-buffers-checksum-run.wf`
  - `tests/conformance/cases/x-struct-set-field.wf`
  - `tests/conformance/cases/x-typ-bool-cmp-result-as-int.wf`
  - `tests/conformance/cases/x-typ-uniq-deref-write-roundtrip.wf`
  - `tests/conformance/cases/x-typ-value-where-borrow-param.wf`
  - `tests/conformance/cases/x-wc-chunk-summary-run.wf`
  Deleted: none.
  Renamed: none.
  No retained case id changes its declared verdict:
  `python3 tests/conformance/runner.py verdicts origin/main` reports 0 moved,
  0 removed, 5 added, 503 total. Within `tests/conformance/manifest.jsonl`,
  case records move from 498 to 503 and rule-annotation records remain 32.
  Declared verdict totals move from 69 accept, 248 reject, and 181 run to 69
  accept, 251 reject, and 183 run. Coverage remains 131/131 rules: the corpus
  covers 113 rules by case and 32 rules by annotation, against 112 by case and
  32 by annotation on `main`. The 5 added ids pin the respelled rules:
  `gram5-pos-comparison-operators` (all six operators and a delimited call
  run to exit 0), `gram5-neg-type-application-without-delimiter` (a bare
  `cvt<u8, u32>(...)` parses as a comparison and fails at `u8`, which DIAG-1
  row 3 attributes to FORM-3), `gram4-neg-multiplied-use-relation-bare` (a
  multiplied relation-form use step without parentheses does not derive),
  `prf1-pos-multiplied-relation-use` (`use 3 * (a + b <= c + d)` closes a
  factor-three target), and `inv1-neg-equality-relation` (`==` as an
  invariant relation is rejected citing INV-1). The 275 modified cases keep
  their ids and their declared verdicts; their sources are respelled to the
  v0.41 surface by a one-shot token rewriter that is not shipped, and the two
  cases that reached `main` after the branch was cut,
  `set2-neg-arena-replace-target` and `set2-pos-box-descriptor-replace`, are
  respelled on the merge of `main` into the branch.
  Outside the case directory, the exact conformance runner, adapter,
  collection, and gate-wiring boundary is:
  - `tests/conformance/manifest.jsonl`:
    `7fcc8df2b350459e1524eb75a5d3d44c502eb9f00a98ad51c10c74439b4ae0a9`
    -> `85adc40a7c7250ee46dff76bd9354b61075f774fa77ba3d485ddc513b5d88e2c`
    (five case records added; the `doc` prose of records that named a retired
    operation spelling respelled; no verdict, rule, status, or coverage field
    of a retained record changes).
  No other conformance adapter, runner, collection, manifest, gate-integrity,
  or gate-wiring file changes; `Makefile`, `compiler/Makefile`,
  `compiler/tests/canonical_corpus.rs`, `compiler/tests/conformance.rs`,
  `compiler/tests/conformance/adapter.rs`,
  `compiler/tests/conformance/corpus.rs`,
  `compiler/tests/conformance/json.rs`, `tests/conformance/runner.py`, and
  `tests/conformance/test_runner.py` are byte-identical to `main`.
ACTIVE-SPEC: v0.41 899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761 15ec2f6f475a7b70fb2654026ec3b6ef79afca3bd588fb38f22005d6637c0168

## 2026-09-03 — merge-time approval content: v0.42 one canonical region spelling ([FORM-8]) (1 rule added; 132 remain)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this record
  creates no separate approval step, and nothing in it asserts that the
  approval has been given.
- SPECIFICATION: activate Whitefoot v0.42 at exact SHA-256
  `6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26`.
  It supersedes active v0.41 at SHA-256
  `899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.41.md`. The candidate that preceded activation declared
  `CANDIDATE v0.42 supersedes v0.41` and hashed to
  `8cf0b9142eaee1e48734fd29d572d27f2e6f9faf0a6a6518d011124786bc2e37`;
  activation flipped that status line to `ACTIVE v0.42` and changed no other
  byte. Per the specification's own META-5
  delta declaration: numbered rules are +1/-0 and 132 remain, the added rule
  being [FORM-8] and no id retired; grammar productions are +0/-0 and 83
  remain, with `mode`, `borrow_expr`, `region_stmt`, and the `slice` and
  `arena` alternatives of `type` each gaining one optional REGIONID decision;
  and fixed lowercase atoms, compound punctuation tokens, token bytes, writer
  operation spellings, opaque system nominal spellings, runtime-trap families,
  entry forms, contract block forms, exception clauses, and the 203 system
  operations and declaration records are unchanged. The changed content makes
  every region spelling forced. A REGIONID is written exactly where the
  document does not otherwise fix the region it denotes: a declaration writes a
  name only to relate two of its own positions or to name an output-position
  region no parameter position determines, and the region parameter list then
  holds exactly the written names in order of first written occurrence; a
  `borrow_expr` writes its region only when it is not the innermost enclosing
  `region_stmt`'s; a `region_stmt` writes its name only when its body still
  references it; and a call's `::` type application writes exactly the callee
  region parameters no parameter position determines, every other one being the
  least region of the actual arguments at the positions naming it, so a call
  whose regions are all determined writes no `::` application at all. The
  seventeen [SYS-2] declaration records are re-rendered in that same form,
  changing spelling only — no signature identity, parameter name, order, mode,
  type, effect, or count moves. No liveness, outlives, exclusivity,
  storage-duration, provenance, effect, or confinement judgment changes, and
  the accepted-program set is unchanged up to respelling. The selection ground
  the specification states is minimality under [FORM-1] and the owner ruling
  of record that one semantics has one spelling, with the rule decidable from
  the owning declaration's own text at every position.
- CONFORMANCE BOUNDARY: relative to `main` tip
  `30602914`, `tests/conformance` content changes as follows. Under
  `tests/conformance/cases/`, 8 files are added, 123 are modified, and 3 are
  deleted; git reports no rename. `manifest.jsonl` is modified: the three
  deleted ids leave it, the eight added ids enter it, and two retained ids get
  a new `doc` sentence. Added:
  - `tests/conformance/cases/form8-neg-elided-related-region.wf`
  - `tests/conformance/cases/form8-neg-elided-result-region.wf`
  - `tests/conformance/cases/form8-neg-unreferenced-region-block-name.wf`
  - `tests/conformance/cases/form8-neg-written-determined-region-argument.wf`
  - `tests/conformance/cases/form8-neg-written-innermost-borrow-region.wf`
  - `tests/conformance/cases/form8-neg-written-unrelated-parameter-region.wf`
  - `tests/conformance/cases/form8-pos-elided-regions-run.wf`
  - `tests/conformance/cases/form8-pos-related-pair-written.wf`

  Deleted, each because the language rule it tested no longer exists:
  - `tests/conformance/cases/type5-neg-wrong-region-arg-count.wf` — a user call
    stating two region arguments where the callee declared one. [FORM-8] gives
    a call exactly one legal region-argument list, so "wrong count" is no
    longer a distinct fault; `form8-neg-written-determined-region-argument`
    carries the surviving content, that writing a determined region argument is
    rejected.
  - `tests/conformance/cases/sys2-neg-wrong-region-arg-count.wf` — the same
    fault for a system operation. Every [SYS-2] region occupies one parameter
    position, so the legal list is empty and the same replacement covers it.
  - `tests/conformance/cases/reject-sys2-args-count-missing-region.wf` — a
    system call omitting a required region argument. Under [FORM-8] omitting it
    is the required spelling, so the program this case declared invalid is now
    the canonical one; `form8-pos-elided-regions-run` accepts and runs that
    shape.

  The 123 modified case sources are the mechanical [FORM-8] re-spelling and
  nothing else, except two whose content is restated because the old spelling
  became unwritable:
  - `own3-neg-undeclared-signature-region` — the undeclared region moves from a
    parameter mode to a body `borrow_expr`. A signature position can no longer
    carry an undeclared name, because [FORM-8] admits a written name only when
    the region parameter list holds it; the body borrow is where an undeclared
    spelling is still reachable, and the case still reaches [OWN-3].
  - `x-borrow-own-param-escape-no-return` — the caller-supplied region gains a
    second position (a borrow parameter and the result share it) so that
    [FORM-8] writes it at all. The escape under test is unchanged: a borrow of
    an own local into a caller-supplied region, with nothing returning it, and
    the case still reaches [OWN-10].

  No other case changes its expectation kind, cited rule, or runnable status,
  and coverage remains complete at 132/132 rules with [FORM-8] covered by case.
ACTIVE-SPEC: v0.42 6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26 899437ecf48691b9bc436c86a56ccc2a47fc4eb9290d546010296db7808c5761

## 2026-09-03 — merge-time approval content: v0.43 loop-body regions and the [ENT-6] join repair (0 rules added; 132 remain)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this record
  creates no separate approval step, and nothing in it asserts that the
  approval has been given.
- SPECIFICATION: install Whitefoot v0.43 as a work-branch CANDIDATE at exact
  SHA-256
  `1708dd2b64b93c88d1dfc23acd340c853b03ca240b9b86ac43fbc91e1c0b2081`.
  Its status line declares `CANDIDATE v0.43 supersedes v0.42
  6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26`. A
  candidate is not an activated identity: this record adds no `ACTIVE-SPEC:`
  line, writes no `spec/kernel-spec-v0.42.md` archive, and leaves the chain
  tail at v0.42, so canonical `make check` stops the revision at
  `spec-archive-integrity` with the candidate message until a later activation
  archives the outgoing bytes and records the chain line. Per the
  specification's own META-5 delta declaration: numbered rules are +0/-0 and
  132 remain, with no id added or retired; grammar productions are +0/-0 and
  83 remain, with no production added, removed, or given a new decision; and
  fixed lowercase atoms, compound punctuation tokens, token bytes, writer
  operation spellings, opaque system nominal spellings, runtime-trap families,
  entry forms, contract block forms, exception clauses, and the 203 system
  operations and declaration records are unchanged. [OWN-3], [OWN-11],
  [FORM-8], and [ENT-6] are amended, and the candidate carries **two
  independent amendments**.

  The first makes every loop body a region block. The body of a `loop_stmt` or
  a `for_stmt` introduces one unnamed local region whose block is that body
  [OWN-3, OWN-11], so a `borrow_expr` written directly in that body takes that
  region, is written bare [FORM-8], and dies with the iteration — exactly the
  guarantee [OWN-11] already made, now obtained with no writer ceremony. The
  region has no REGIONID and no position can name it, so nothing outside the
  body reaches it. Because that region exists, a `region_stmt` that is a loop
  body's only statement has exactly the body as its block, is a second
  spelling of one region, and is a hard error citing [FORM-8]; the mechanical
  repair is to delete the block, keep its statements as the loop body, and
  elide every REGIONID that named it. A `region_stmt` the body writes another
  statement beside is not that second spelling and stays legal: its block is a
  strict part of the body, and [OWN-6] admits a statement-scoped child
  reborrow under a region whose block does not extend beyond the enclosing
  statement, which a one-statement block inside a longer body satisfies and
  the body's own region does not. The one exception to the rejection is a
  block some `targ` region argument inside it must write its name at, because
  no implicit region has a name that position could carry. A writer decides
  the whole rule by reading the loop body alone, asking only whether the body
  writes anything beside the block. No liveness, outlives, exclusivity,
  storage-duration, provenance, effect, or confinement judgment changes; the
  accepted-program set changes only by removing the redundant spelling and
  admitting the bare one.

  The second repairs [ENT-6]'s control-flow join, which was not associative. A
  delta atom minted by an earlier join counted as an ordinary nonconstant term
  at the next one, so two nested joins lost a binding image that one flat join
  over the same branches kept, and a three-way demux was accepted written as a
  `match` and refused written as nested `if`/`else` with no semantic
  difference between them. At a join every input image is now normalized
  first: each delta atom an earlier join minted is folded back into the
  constant interval it stands for, that atom's coefficient times its interval
  added to the input's constant, leaving one non-delta nonconstant form and
  one closed constant interval. Where the normalized inputs share one
  nonconstant form the join is that form plus one fresh delta atom over the
  hull of their constant intervals; otherwise the binding still receives one
  fresh full-type atom. Delta atoms remain ordinary shared atoms everywhere
  except a join, so a relation formed over one after a join still holds at the
  next. An input carrying no delta atom normalizes to its own constant, so the
  rule is the v0.42 rule wherever no join has run. The repair only adds
  images, so no program v0.42 accepted is refused, and acceptance no longer
  depends on the shape of the control join. The rule stays source-structural
  and decidable from the rule alone.
- CONFORMANCE BOUNDARY: relative to this batch's base, the v0.42 activation
  commit `c168ab2a` on `batch/0118-region-elision`, `tests/conformance`
  content changes as follows. Under `tests/conformance/cases/`, 6 files are
  added, 3 are modified, and none is deleted; git reports no rename.
  `manifest.jsonl` is modified: the six added ids enter it and no existing
  line changes. Added:
  - `tests/conformance/cases/own11-pos-loop-body-region.wf`
  - `tests/conformance/cases/form8-neg-region-block-is-the-loop-body.wf`
  - `tests/conformance/cases/form8-pos-narrower-loop-region-block.wf`
  - `tests/conformance/cases/ent6-pos-nested-join-keeps-the-flat-image.wf`
  - `tests/conformance/cases/ent6-pos-join-shape-independence.wf`
  - `tests/conformance/cases/ent6-neg-nested-join-does-not-invent-a-bound.wf`

  Modified, with the reason each source moves:
  - `own11-neg-borrow-outer-region` — the borrow now writes the outer region
    name. Its old elided spelling denoted the region introduced outside the
    loop; under [OWN-11] an elided borrow in a loop body takes the body's own
    region, so the old source is an accepted program and the outer region has
    to be named for the fault to be written at all. The normative content is
    unchanged: a borrow inside a loop naming a region introduced outside it is
    rejected citing OWN-11.
  - `accept-par3-staged-denied-hoisted-scratch` and
    `accept-par3-staged-denied-read-before-write` — each had a `region 'f`
    block as its `for` body's only statement. [FORM-8] now rejects that
    spelling, so the block is deleted, its statements become the loop body,
    and `&'f cwd` becomes `&cwd` under the body's own region. Both keep their
    declared expectation, cited rule, and staged-permission verdict.

  No other case changes its expectation kind, cited rule, or runnable status,
  and coverage remains complete at 132/132 rules.

## 2026-09-04 — merge-time approval content: activate v0.43 (loop bodies as region blocks and the associative image join; no rule added or retired; 132 remain)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step, and nothing in this record asserts
  that the approval has been given.
- SPECIFICATION: activate Whitefoot v0.43 at exact SHA-256
  `037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`.
  It supersedes active v0.42 at SHA-256
  `6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.42.md`. The candidate that preceded activation was
  installed on `main` by the record above of 2026-09-03: it declared
  `CANDIDATE v0.43 supersedes v0.42` and hashed to
  `1708dd2b64b93c88d1dfc23acd340c853b03ca240b9b86ac43fbc91e1c0b2081`;
  activation flipped that status line to `ACTIVE v0.43` and changed no other
  byte, so the META-5 delta declaration, the two independent amendments
  ([OWN-3], [OWN-11], [FORM-8], [ENT-6] amended; numbered rules +0/-0, 132
  remain), and the conformance boundary recorded there are the content
  activated here. This record adds the `ACTIVE-SPEC:` chain line below and
  the archive of the outgoing bytes, and nothing else.
- CONFORMANCE BOUNDARY: this merge adds, modifies, deletes, or renames no
  conformance case, manifest row, adapter, runner, or collection wiring; the
  boundary recorded with the candidate on 2026-09-03 is unchanged.
ACTIVE-SPEC: v0.43 037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951 6b935d2ea7729876fc96533b5559f6f58598e335b4b5cffad86cc4782c0eed26

## 2026-09-04 — merge-time approval content: kernel specification v0.44 CANDIDATE (the fact machinery: contract vocabulary, operand denotation, and publication; numbered rules +4, 136 remain)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step, and nothing in this record asserts
  that the approval has been given.
- SPECIFICATION: install the Whitefoot kernel specification v0.44 CANDIDATE at
  `spec/kernel-spec.md`, whose status line declares
  `CANDIDATE v0.44 supersedes v0.43 037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`
  and whose complete bytes hash to SHA-256
  `1240d9ff604276f96b954f0524c973c8ab7490ef63b91c7f7c6b8c2d57181b3b`.
  A candidate is not an activated identity: this record adds no `ACTIVE-SPEC:`
  line, archives no bytes, and leaves the activation chain tail at v0.43. The
  bytes above are the exact content approved; activation, if the owner takes
  it, is its own later record that flips the status line and archives the
  outgoing v0.43 bytes.
- CONTENT: four numbered rules are added and none is retired, so 136 remain.
  [MSR-5] gives a `requires_clause` and an `ensures_clause` a `clause_expr`
  whose operands are an `atom`, a `call`, or a `construct`, which is what
  admits a measure of a place as a clause operand on either side of the
  comparison; the measure formers are table data with one row in this version,
  `len(P)`, admitted exactly where [ENT-2] clause (b) admits a length term.
  [MSR-3] states one denotation per operand position keyed on the parameter's
  mode, adds the compiler-owned **call datum** an `own` operand denotes when a
  relation is read at a caller, and makes a `&uniq` parameter's measure
  inadmissible in a source-declared `ensures`. [CALL-4] states the contract
  vocabulary over the one result a `fn_decl` declares and records the deferred
  widenings. [CALL-6] states publication once — substitution, instantiation
  point, establishment point, destination, and support — adds [ENT-3]'s source
  S13, and refuses at the declaration a contract whose published relations are
  contradictory at their establishment point. Grammar productions +1
  (`clause_expr`); `requires_clause` and `ensures_clause` change to take it;
  no token, no fixed atom, and no system record changes. [GRAM-6], [FN-8],
  [FN-9], [ENT-2], [ENT-3], and [ENT-5] are amended in place.
- CONFORMANCE BOUNDARY: relative to this batch's base, the v0.43 activation
  merge `69771591` on `main`, `tests/conformance` content changes as follows.
  Under `tests/conformance/cases/`, 6 files are added, none is modified, and
  none is deleted; git reports no rename. `manifest.jsonl` is modified: the
  six added ids enter it and no existing line changes. Added:
  - `tests/conformance/cases/msr5-pos-two-measure-clause.wf`
  - `tests/conformance/cases/msr3-pos-own-operand-call-datum.wf`
  - `tests/conformance/cases/msr3-neg-uniq-state-measure-in-ensures.wf`
  - `tests/conformance/cases/call6-neg-contradictory-published-relations.wf`
  - `tests/conformance/cases/call6-pos-routed-relation-over-a-call-datum.wf`
  - `tests/conformance/cases/call4-neg-measured-result-not-admitted.wf`

  No existing case changes its expectation kind, cited rule, or runnable
  status, and no case flips a verdict. Coverage remains complete over the
  numbered rules the corpus is required to cover.
- CORPUS MIGRATION, outside the conformance boundary: [MSR-3] refuses a
  measure of a `&uniq` state parameter in an `ensures`, and two programs under
  `tests/programs/` wrote exactly that clause. `wfgrep.wf` and
  `raw_deflate_boundary.wf` each declared
  `fn append_slice(destination: &uniq buffer<u8>, ...)` with
  `define capacity = len(deref(destination)); ensures result <= capacity;`,
  which publishes a bound on a caller's object through the one parameter mode
  from which the callee could have replaced it. Both take the capacity as an
  `own u64` operand instead and state the connection as
  `requires capacity == len(deref(destination));`, which is the restructuring
  the diagnostic names; each body keeps binding the destination's own length
  for its writes, so it proves its own subscript with no help from the
  contract. Every caller passes the length it already had in scope. Neither
  program's behaviour, published diagnostics, or exit codes change, and the
  integration tests that execute both still pass. `docs/patterns.md` carries
  the writer form as P21 and corrects P16's define-per-measure spelling.

## 2026-09-04 — merge-time approval content: activate v0.44 (the fact machinery: contract vocabulary, operand denotation, and publication; numbered rules +4/-0 relative to v0.43, 136 remain)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. That merge approval is rule
  2's approval and rule 4's approval of the content recorded here; this
  record creates no separate approval step, and nothing in this record asserts
  that the approval has been given.
- SPECIFICATION: activate Whitefoot v0.44 at exact SHA-256
  `5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049`.
  It supersedes active v0.43 at SHA-256
  `037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951`;
  those outgoing bytes are preserved byte-for-byte as
  `spec/kernel-spec-v0.43.md`. The candidate that preceded activation was
  installed on `main` by the record above of 2026-09-04: it declared
  `CANDIDATE v0.44 supersedes v0.43` and hashed to
  `1240d9ff604276f96b954f0524c973c8ab7490ef63b91c7f7c6b8c2d57181b3b`;
  activation flipped that status line to `ACTIVE v0.44` and changed no other
  byte, so the META-5 delta declaration, the four added rules ([MSR-3],
  [MSR-5], [CALL-4], [CALL-6]; numbered rules +4/-0, 136 remain; grammar
  productions +1, `clause_expr`), the amendments in place to [GRAM-6],
  [FN-8], [FN-9], [ENT-2], [ENT-3], and [ENT-5], and the conformance boundary
  recorded there are the content activated here. This record adds the
  `ACTIVE-SPEC:` chain line below and the archive of the outgoing bytes, and
  nothing else.
- CONFORMANCE BOUNDARY: this merge adds, modifies, deletes, or renames no
  conformance case, manifest row, adapter, runner, or collection wiring; the
  boundary recorded with the candidate on 2026-09-04 is unchanged.
ACTIVE-SPEC: v0.44 5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049 037c9e69b271a7ae212bd71fa2e79c74a3bf4b2115c0418f4908a24b0a9f6951

## 2026-09-04 — merge-time approval content: kernel specification v0.45, the container and resource design (batch/0127-containers; record written in full at merge time)
- EFFECT: this record becomes effective only when the owner approves the exact
  revision containing it for merge into `main`. It is a work-branch draft
  until then; its SPECIFICATION digest below is refreshed as the branch edits
  the active file, and its CONTENT and CONFORMANCE BOUNDARY sections are
  completed before the merge.
- SPECIFICATION: v0.45 at the digest on the chain line below supersedes v0.44
  at `5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049`,
  whose outgoing bytes are preserved byte-for-byte as `spec/kernel-spec-v0.44.md`.
- CONTENT (B2, the proof surface): v0.45 additionally lands the containers
  design's [MSR] proof surface. Numbered rules +4/-0 (140 remain), being
  [MSR-1], [MSR-2], [MSR-4] and [MSR-6]; grammar productions +0/-0 and no
  grammar atom is added by this batch; writer operation spellings +3/-0, being
  the [OP-1] readers `cap`, `room` and `head` over `array<T, N>`,
  `slice<'r, T>` and `buffer<T>`; exception clauses +0/-1, being [ENT-5]'s
  element-position carve-out, removed rather than narrowed. [OP-1], [OP-4],
  [OP-7], [TYPE-6], [ENT-2], [ENT-5], [ENT-6], [INV-1] and [MSR-5] are amended
  in place. [MSR-1] makes `len(P)`, `cap(P)`, `room(P)` and `head(P)` [ENT-2]
  terms over an admitted measure place formed with field selections, `deref`
  wrappings and subscripts, states that a measure table exists and gives every
  measured type a row with one of exact, bounded or absent per cell, carries
  that table as a `wf-measures` fence, and fixes the logical coordinate system
  and its injectivity sentence. [MSR-2] states the support of a measure term as
  descriptor storage rather than a root binding, restates the kill as an
  ordinary [ENT-5] event over that storage, fixes the standing facts every
  measured value carries, and appends `len + room = cap` to [ENT-6]'s automatic
  affine premises as two inequalities with empty support. [MSR-4] states once
  the complete ordered derivation of a numeric goal in six steps and retires
  [ENT-6]'s per-family route grants, each family keeping its normalization;
  [ENT-6] carries one compiler-owned immutable affine atom per live measure
  term. [MSR-6] admits an in-scope const generic as a [TYPE-6] `pbase` and with
  it as an [ENT-2] `for_stmt` endpoint and an [MSR-5] clause operand.
- CONSEQUENCE OF THE THREE READER SPELLINGS: `cap`, `room` and `head` are
  dotless IDENT-shaped operation spellings, so [OP-1] adds all three to
  `ReservedLowerNames` and no source declaration may be spelled with one of
  them any more. The corpus used all three as ordinary binding spellings. This
  merge therefore renames those bindings in the corpus's own sources -- `room`
  to `spare`, `head` to `front`, `cap` to `limit` -- in 118 snapshot cases, 30
  conformance cases, 7 `tests/programs` sources and the compiler's embedded
  test sources. Every renamed program keeps the verdict it recorded: no
  `tests/snapshot/index.tsv` row and no `tests/conformance/manifest.jsonl`
  expectation is edited by this merge. Before the rename the two corpora
  reported 28 conformance failures and 72 snapshot flips, every one of them a
  [FORM-3] `ReservedName` rejection on one of the three spellings and none of
  them a semantic verdict change; after it both corpora are back to their
  recorded verdicts.
- CONTENT (B1b, multi-return): v0.45 lands multi-return. Numbered rules +0/-0 and
  grammar productions +0/-0 (84 remain); five productions change — `fn_decl`
  takes an ordered result list, `result_route` an optional ordinal binder,
  `let_stmt` a parenthesized binder list over a `call`, `set_stmt` a
  parenthesized target list over a `call`, and `return_stmt` one expression per
  declared result — and one fixed lowercase grammar atom is added, `is` (55
  remain). [FORM-2], [TYPE-5], [SET-1], [FN-1], [FN-9], [CALL-4], [CALL-6] and
  [ENT-3] are amended in place; no rule id is added or retired and [ENT-3]
  gains no source label. [FN-1] states the result ordinal and reads every result
  judgment per ordinal; [TYPE-5] derives binder i and target i from ordinal i;
  [SET-1] commits a target list in written order over pairwise distinct roots;
  [CALL-4] takes the ordinal-named route `when b is V(f: r):`, its
  omitted-binder condition, its ambiguity refusal at the declaration, and the
  two [ENT-3.S12] destinations only a multi-result contract exercises.
- CONFORMANCE BOUNDARY: this merge ADDS thirteen conformance cases and their
  thirteen manifest rows, MODIFIES the source of 30 existing cases by renaming
  a binding whose spelling [OP-1] now reserves (listed under CONSEQUENCE
  above; no expectation, rule citation, status or manifest row of any of them
  changes), and deletes or renames none; it changes no adapter,
  runner, or collection wiring. The added case ids are
  `s16-pos-result-list-reaches-both-let-binders`,
  `s16-pos-result-list-reaches-both-set-targets`,
  `call4-pos-route-names-a-result-ordinal`,
  `call4-pos-omitted-route-binder-with-one-enum-ordinal` (each expecting run,
  exit 0) and `call4-neg-ambiguous-route-over-two-enum-ordinals` (expecting
  reject, CALL-4). B2 adds `msr1-pos-the-four-measure-readers`,
  `msr1-pos-subscript-obligation-against-len`,
  `msr2-pos-sibling-field-write-keeps-the-measure`,
  `msr2-pos-element-write-keeps-the-measure`,
  `msr4-pos-capacity-requirement-discharges-a-length-obligation`,
  `msr6-pos-const-generic-as-a-value` and
  `msr6-pos-const-generic-in-a-clause-and-an-endpoint` (each expecting run,
  exit 0) and `msr2-neg-descriptor-write-kills-the-measure` (expecting reject,
  OP-4). Before this merge the corpus holds 520 cases with the native adapter
  reporting Pass=518, Xfail=1, Skip=1; after it the corpus holds 533 with the
  adapter reporting Pass=531, Xfail=1, Skip=1. The one xfail
  (`ent5-neg-callee-uniq-buffer-replace-kills-length`) and the one skip are
  unchanged in id, expectation, and status: [MSR-2] restated the granularity of
  a measure kill over storage, while the classification of what a projected
  callee write touches is [CALL-3]'s and is not in this merge.
- CONTENT (B4, liveness and the one commit rule): v0.45 lands the containers
  design's `[LIV]` family. Numbered rules +2/-0 (142 remain), being [LIV-1] and
  [LIV-2]; grammar productions +0/-0 (84 remain); `set_stmt` changes again,
  its parenthesized target list now taking one `call` or a value list of one
  expression per target, which is the owner's decision D2 of 2026-09-03 that
  `set (p, x) = x, p;` is the swap and that the language gets no exchange
  operation. [OWN-1], [OWN-11], [SET-1], [STOR-1], [STOR-3] and [TYPE-5] are
  amended in place. [LIV-1] makes a binding's live-or-dead status a property of
  a program point: every predecessor of every join and every loop head agrees
  on it, a disagreement is a hard error naming the binding and the two
  predecessors, and because the status agrees every scope-exit release is
  unconditional. [OWN-11]'s prohibition on moving an outer binding into a loop
  body becomes that rule's per-iteration reading of the same agreement at its
  own loop head, over the structural backedge, so a body that moves such a
  binding and reinitializes it before the backedge is admitted and a body that
  leaves it dead is not. [LIV-2] states one `set` commit for one or more
  targets: every target is resolved before the right-hand side, a `move` of a
  target place or of a place reached through one is that target's read-out and
  kills no root, every target is dead through the evaluation, and all targets
  are reinitialised at one commit, under three conditions — the dead target,
  whose live affine case keeps [STOR-1]'s error and its one restructuring;
  pairwise non-overlapping targets on [OWN-7]'s terms, which replaces [SET-1]'s
  pairwise-distinct-roots placeholder; and exact arity and type. [OWN-1] revives
  a dead binding only through such a commit at that complete binding. One clause
  of [LIV-2] is DEFERRED with a stated zero delta: a target identifier that
  resolves to no binding, which would introduce one as a `let` does.
- CONFORMANCE BOUNDARY (B4): this batch ADDS nine conformance cases and their
  nine manifest rows, and modifies, deletes and renames none; it changes no
  adapter, runner, or collection wiring, and no existing case's source,
  expectation, rule citation or status. The added ids are
  `liv1-neg-branches-disagree-on-liveness` (expecting reject, LIV-1),
  `liv1-neg-loop-leaves-an-outer-binding-dead` (expecting reject, OWN-11),
  `liv2-neg-two-subscripts-of-one-run` (expecting reject, LIV-2),
  `liv1-pos-loop-moves-and-restores-an-outer-binding` and
  `liv2-pos-read-out-at-a-binding-a-field-and-a-deref` (run, exit 0),
  `liv2-pos-read-out-keeps-the-root-and-its-other-fields` (run, exit 3),
  `liv2-pos-swap-and-rotation` (run, exit 5),
  `liv2-pos-two-fields-of-one-root` (run, exit 9) and
  `liv2-pos-subscript-targets-commit-their-own-ordinals` (run, exit 2). Before
  this batch the corpus holds 533 cases with the native adapter reporting
  Pass=531, Xfail=1, Skip=1; after it the corpus holds 542 with the adapter
  reporting Pass=540, Xfail=1, Skip=1. The one xfail
  (`ent5-neg-callee-uniq-buffer-replace-kills-length`) and the one skip are
  unchanged in id, expectation and status, and the recorded-verdict snapshot
  corpus reports Pass=491, Flip=0: no verdict of either corpus moved.
- CONTENT (B5, linearity, the release and the destructuring forms): v0.45 lands
  the containers design's `[PROV-6]`. Numbered rules +1/-0 (143 remain), being
  [PROV-6]; grammar productions +3/-0 (87 remain), being `dispose_stmt`,
  `region_param` and `linearity_bound`; unique fixed lowercase grammar atoms
  +3/-0 (58 remain), being `dispose`, `linear` and `affine`, which [FORM-3]
  therefore excludes from IDENT, so no source declaration, contract member or
  callee may be spelled any of the three any more. `stmt` gains `dispose_stmt`,
  `let_stmt` gains the destructuring consume `let N(f1: b1, ..., fk: bk) = move
  v;`, `struct_decl` and `enum_decl` each gain an optional `linear` modifier,
  `gparam`'s optional bound gains a linearity class beside its marker TYPEID,
  and `region_params`'s members become `region_param`. These four spellings are
  [S12], [S13], [S18] and [S32] as the owner adopted them on 2026-09-03 and
  2026-09-04, and `dispose p;` takes no `using` list. [FORM-2], [TYPE-2],
  [TYPE-3], [OWN-1], [LIV-1], [STOR-3], [FN-2] and [EFF-2] are amended in place.
  [PROV-6] makes a value linear in a scope exactly when it owns, at any depth,
  a value whose release action requires a capability that scope does not hold or
  a value of a nominal declared `linear`, and affine there otherwise; one
  release graph is what the compiler-derived release and `dispose p;` both walk;
  a consume of a proper sub-place of a linear value is refused unless the same
  statement's commit reinitialises it; and a written `affine` or `linear` bound
  states the class a declaration is written for, checked at every instantiation.
  In this version the ambient heap is the only store whose reclamation is a
  release and it is not a value, so every scope holds it: nothing is linear here
  by the capability criterion, every memory-reclamation action keeps the empty
  row [EFF-2] already fixed, and no walk resolves a provider place. Two clauses are
  DEFERRED with stated zero deltas. The first is the refusal of a type whose
  release graph has a cycle: landing it retires
  `tests/programs/recursive_tree.wf`, a heap-backed recursive enum this compiler
  accepts today and three live tests compile, and that verdict move is the
  owner's disposition rather than a checker capability, so this merge does not
  make it. The second is the linearity-bound check over a region argument, which
  decides nothing while every store a region names is reclaimed by its own
  region release; the bound is read at the declaration and its type axis is
  checked at every instantiation in both directions.
- CONFORMANCE BOUNDARY (B5): this batch ADDS sixteen conformance cases and their
  sixteen manifest rows, MODIFIES one existing case's source, and deletes and
  renames none; it changes no adapter, runner, or collection wiring, and no
  existing case's expectation, rule citation or status. The added ids are
  `prov6-pos-dispose-runs-the-release-walk-early` and
  `prov6-pos-linearity-bound-on-a-region-parameter` (run, exit 0),
  `prov6-pos-linearity-bound-admits-the-instantiation` (run, exit 2),
  `prov6-pos-linear-value-moved-out-whole` (run, exit 3),
  `prov6-pos-commit-reinitialises-the-consumed-sub-place` (run, exit 5),
  `prov6-pos-destructuring-consume-discharges-the-obligation` (run, exit 7),
  `prov6-pos-dispose-writes-the-operands-storage-origin` (run, exit 9),
  `prov6-neg-linear-value-not-consumed`,
  `prov6-neg-linear-value-partially-consumed`,
  `prov6-neg-dispose-of-a-view`,
  `prov6-neg-dispose-of-a-modifier-linear-node`,
  `prov6-neg-dispose-without-a-capability-leaf`,
  `prov6-neg-linear-modifier-on-a-tag-only-enum` and
  `prov6-neg-linearity-bound-refuses-the-instantiation` (each expecting reject,
  PROV-6), `prov6-neg-dispose-through-a-shared-borrow` (reject, OWN-1) and
  `prov6-neg-dispose-without-the-declared-write` (reject, EFF-2). The one
  MODIFIED case is `reject-syseff-pure-member-binds-release`: its contract
  member and conformance binding were spelled `dispose`, which [FORM-3] now
  excludes from IDENT, so both are renamed `release`. Its id, expectation
  (reject, FN-3), rule citation, status and doc are unchanged, and the rule
  under test — a `pure` member bound to a function that exhibits a category
  only through release — is untouched: the before/after boundary of that case
  is exactly the member spelling. Before this batch the corpus holds 542 cases
  with the native adapter reporting Pass=540, Xfail=1, Skip=1; after it the
  corpus holds 558 with the adapter reporting Pass=556, Xfail=1, Skip=1. The one
  xfail (`ent5-neg-callee-uniq-buffer-replace-kills-length`) and the one skip are
  unchanged in id, expectation and status, and the recorded-verdict snapshot
  corpus reports Pass=491, Flip=0: no verdict of either corpus moved.
- CONTENT (B7a, the brand, the runs and the kernel declaration domain): v0.45
  lands the containers design's `[PROV-1]`, `[BLK-0]`, `[BLK-1]`, `[BLK-2]` and
  `[BLK-3]`. Numbered rules +5/-0 (148 remain); grammar productions +0/-0 (87
  remain), with `struct_decl` and `enum_decl` each gaining an optional
  `region_params` [S20]; unique fixed lowercase grammar atoms +0/-0 (58 remain);
  compiler-owned nominal type spellings +4/-0, being `Vector`, `FixedVector`,
  `Heap` and `Arena` [S1-S4], each contributing one nominal-type entry and one
  constructor entry of the same spelling; writer operation spellings +9/-0, being
  the kernel-domain operations `seq_fixed`, `seq_arena`, `seq_arena_proved`,
  `seq_heap`, `arena_frame`, `seq_place`, `seq_place_front`, `seq_take` and
  `seq_take_front` [S7-S9], which are declaration records rather than [OP-1]
  table rows and therefore add nothing to `ReservedLowerNames`; kernel
  declaration records +44/-0, being nine operations, twenty-three type, const
  and region parameters, and twelve value parameters. [TYPE-2], [TYPE-3], [TYPE-5], [TYPE-6], [CONST-1],
  [OWN-1], [FN-2], [FN-7], [OP-1], [OP-4], [OP-9], [MSR-1], [MSR-2], [MSR-3],
  [ENT-3.S13], [STOR-1], [STOR-3], [STOR-5], [PROV-6], [PROG-1], [SYS-3] and
  [DIAG-1] are amended in place. [PROV-1] makes a store's identity a region,
  puts that region in the type of every value the store backs, admits at most
  one reserving occurrence per region, and resolves every elided store brand by
  one rule with two clauses. [BLK-0] states that the container and store
  operations are one compiler-owned generic declaration domain admitted to every
  unit on [SYS-3]'s terms, each operation one complete signature record whose
  written arguments are decided per argument, whose value arguments are named,
  and every one of whose rows is complete over every measure it writes on every
  exit; [ENT-3.S13]'s population is extended to those rows. [BLK-1] states the
  two runs and the one window, the logical subscript whose obligation is against
  `len`, and the affine element domain the window makes sound; [MSR-1]'s table
  gains three rows and its first *bounded* cell, a run's `head`. [BLK-2] and
  [BLK-3] are the inventory: four formation rows, one frame reservation, and the
  four boundary operations, each taking its run by value and handing it back.
  Eight clauses are DEFERRED with stated deltas, five of them added by this
  batch: [FN-7]'s `command.heap` standard-input row, whose label tail is the
  spelling [EFF-1] fixes as the allocation atom of `allocates(heap)` and which
  [FORM-3] therefore excludes from IDENT, so it lands with that atom's
  retirement; [BLK-2]'s `arena_extent` row together with the per-activation
  refusal its soundness needs; [STOR-5]'s confinement judgment over an admitted
  store-branded position and the `&uniq` parameter refusal that goes with it,
  costing one further numbered rule; and [CALL-4]'s three v0.44 admissions — a
  result of measured type, a measure over a result place, and a route over any
  variant of any returned enum — which stay deferred because a measured result
  has no destination to be instantiated over until a kernel row hands a run
  back. The retirement of `array<T, N>` with its `array_new` row in favour of
  the `FixedVector<T, n>` const form [S34] is recorded as DEFERRED for the same
  reason: the corpus's ninety-nine `array_new` occurrences have no replacement
  until a formation row and a source `filled` helper both compile, and retiring
  the spelling before then would leave the specification and the compiler
  disagreeing about a hundred accepted programs.
- CONFORMANCE BOUNDARY (B7a): this batch ADDS eleven conformance cases and their
  eleven manifest rows, and modifies, deletes and renames none; it changes no
  adapter, runner, or collection wiring, and no existing case's source,
  expectation, rule citation or status. The added ids are
  `prov1-pos-a-store-branded-run-in-a-field`,
  `prov1-pos-a-nominal-declares-a-store-region` and
  `blk1-pos-both-runs-are-nameable-types` (each expecting accept),
  `prov1-neg-a-provider-in-a-stored-position` (reject, STOR-5),
  `prov1-neg-an-extent-elides-its-store-region` (reject, FORM-8),
  `prov1-neg-a-source-nominal-collides-with-a-container` and
  `blk0-neg-a-source-function-collides-with-a-kernel-row` (each reject, TYPE-6),
  `blk1-neg-a-construct-names-a-run` and
  `blk1-neg-a-construct-names-a-provider` (each reject, BLK-1), and
  `blk2-pos-a-formation-row-builds-an-empty-run` and
  `blk3-pos-a-boundary-row-moves-the-back-boundary` (each expecting accept,
  status `pending`). The two pending rows carry the reason the corpus schema
  requires: a call to a kernel-domain row is an explicit unsupported capability
  in this toolchain, because the window lowering [BLK-1] fixes and the relation
  publication [CALL-6] carries for those rows are not implemented; the
  expectation each records is the verdict the specification requires and is not
  weakened by that status. Before this batch the corpus holds 558 cases with the
  native adapter reporting Pass=556, Xfail=1, Skip=1; after it the corpus holds
  569 with the adapter reporting Pass=565, Xfail=1, Skip=3. The one xfail
  (`ent5-neg-callee-uniq-buffer-replace-kills-length`) is unchanged in id,
  expectation and status, the two added skips are the two pending rows named
  above, and the recorded-verdict snapshot corpus reports Pass=491, Flip=0: no
  verdict of either corpus moved.
- CONTENT (B7a2, the frame-resident run at execution): no numbered rule is added
  or retired (148 remain) and no grammar production, atom, token byte, entry
  form, contract block form, system operation, kernel declaration record or
  compiler-owned nominal spelling changes. [FN-9] and [CALL-4] are amended in
  place, and [CALL-4] lands the first two of its own three v0.44 DEFERRED
  admissions: a declared result may be of measured type [MSR-1], and a measure
  over that bare result place is a clause operand, instantiated at that
  ordinal's own destination [ENT-3.S12] and queried at a selected return over
  the place that return hands back. [FN-9] states the same admission at its
  operand list and at its unrouted-clause condition. The third admission — a
  route over any variant of any returned enum — and the projected result place
  formed with field-selection `psuffix`es, `deref` wrappings or subscripts stay
  DEFERRED with their stated zero deltas, so the count of DEFERRED clauses is
  unchanged at eight. Nothing else in the specification moves: what this batch
  otherwise lands is the compiler catching up to [BLK-0], [BLK-1], [BLK-2],
  [BLK-3], [MSR-4] and [CALL-6] as v0.45 already states them. A call to a
  kernel-domain row over a `FixedVector<T, n>` is now checked from the row's own
  signature record, its written arguments are judged per argument, its
  requirement is submitted at the call and discharged under [MSR-4], its call
  datums are minted at [ENT-3.S13]'s point, and its declared relations are
  published at [CALL-4]'s destinations under [CALL-6]; the window, its
  subscript at `(head + i) mod cap`, the four boundary operations and the run's
  [STOR-3] release action all lower and execute. A diagnostic arising in that
  domain cites BLK-0 and names the operation and the position of the
  requirement in that row's own declared list, which is what a compiler-owned
  record has instead of a source clause node, so no fabricated source location
  enters [DIAG-1]. The store half is not lowered: `arena_frame`, `seq_arena`,
  `seq_arena_proved` and `seq_heap` resolve, check and judge, and a call to one
  stops as an explicit unsupported capability, so no `Vector<'s, T>` and no
  `Arena<'s, bytes, align>` value exists at run time.
- CONFORMANCE BOUNDARY (B7a2): this batch ADDS six conformance cases and their
  six manifest rows, MODIFIES one existing case's source and one existing
  case's status, deletes and renames none, and changes no adapter, runner, or
  collection wiring. The added ids are
  `blk3-pos-a-queue-preserves-its-order`,
  `blk3-pos-a-deque-moves-both-boundaries` and
  `blk1-pos-a-wrapped-window-subscripts-against-len` (each run, exit 0),
  `blk0-neg-a-row-requirement-is-not-discharged` and
  `blk0-neg-a-written-argument-the-row-does-not-declare` (each reject, BLK-0),
  and `call6-pos-a-row-relation-establishes-at-a-caller` (accept). Two existing
  rows change status only: `blk2-pos-a-formation-row-builds-an-empty-run` and
  `blk3-pos-a-boundary-row-moves-the-back-boundary` move from `pending` to
  `runnable` and drop the `reason` the schema requires of a pending row,
  because the capability that reason named now exists; each keeps its id, its
  source, its expectation (accept) and its rule citation exactly. The one
  MODIFIED case is `call4-neg-measured-result-not-admitted`. Its id, its
  expectation (reject, FN-9), its rule citation and its status are unchanged,
  and its before/after boundary is exactly two things: its declared effect row
  moves from `reads(taken)` to `pure`, and its two `doc` lines and its manifest
  `doc` say that the refusal is now the FN-9 proof rather than the FN-9
  admission. The effect row was always wrong under [EFF-2] — a clause
  contributes no effect and the body only hands the parameter back — and the
  declaration's earlier FN-9 admission rejection was reached first, so the
  correction is what the same program has always required rather than a
  weakening: with the measured-result admission landed, the program is still
  refused, now because its parameter arrives with no known length and the
  relation it declares has no premise at the selected return. Before this batch
  the corpus holds 569 cases with the native adapter reporting Pass=565,
  Xfail=1, Skip=3; after it the corpus holds 575 with the adapter reporting
  Pass=573, Xfail=1, Skip=1. The one xfail
  (`ent5-neg-callee-uniq-buffer-replace-kills-length`) is unchanged in id,
  expectation and status, the two skips that disappeared are the two rows named
  above, and the recorded-verdict snapshot corpus reports Pass=491, Flip=0: no
  verdict of either corpus moved.
ACTIVE-SPEC: v0.45 3e0b071836ca0c98ba4017f4b03408fe3f829c477cee834100805018cd3b4c1a 5ef144bfa9f85e9d2a412e053e43b83d250b804acb2f3d409f4d4367301fa049
