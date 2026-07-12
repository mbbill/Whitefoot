# Decision gates

| State | Meaning | Promotion requirement |
|---|---|---|
| research_only | Topic identified but insufficient evidence | primary sources gathered |
| fetched_not_verified | Source fetched but claim not adversarially checked | evidence card + verifier pass |
| workflow_confirmed | Source-grounded in workflow | scope/caveats attached |
| implementation_mapping_needed | Semantics not tied to IR/pass consumers | capability/pass map row |
| prototype_required | Mechanism plausible but untested | lowering sketch + validation plan |
| design_candidate | Ready for debate, not final | evidence, contract, threat model, validation, ABI/runtime impact |
| blocked_by_semantics | Source contract unclear | semantic contract ledger row |
| blocked_by_runtime | Runtime/ABI constraints unresolved | runtime/ABI dossier row |
| rejected_or_refuted | Claim failed or design not viable | preserve reason and source |

A feature cannot become a language default merely because it is optimizer-attractive; it must pass the relevant gates.

## Round-1 debate states (2026-07-01)

| Topic | State after round 1 | Leading candidate exists |
|---|---|---|
| violation-semantics | research_needed | yes |
| aliasing-model | design_candidate_later | yes |
| arrays-loops | prototype_needed | yes |
| dispatch-generics | design_candidate_later | yes |
| static-vs-profile | research_needed | yes |
| numeric-semantics | no_decision | yes |
| concurrency-model | research_needed | yes |
| ir-strategy | research_needed | yes |

## Round-2 feature-essentialism states (2026-07-02)

| Feature | Verdict |
|---|---|
| generics | keep_essential |
| type-inference | needs_evidence |
| dynamic-dispatch | replace_with_alternative |
| closures | replace_with_alternative |
| metaprogramming | replace_with_alternative |
| interfaces | needs_evidence |
| syntax | replace_with_alternative |
| error-handling | replace_with_alternative |
| modules | replace_with_alternative |
| memory-automation | replace_with_alternative |

Decided language law from round 2 (floors that constrain all future rounds): zero implicit value-changing conversions; one numeric mode per arithmetic node; canonical-form-only surface; closed-world whole-program compilation; no exceptions/unwinding (Result + trap=abort); no implicit dynamic dispatch (exhaustive match is the core dispatch); no GC/pervasive RC (checked ownership + explicit storage contracts); no in-language metaprogramming (closed constant-expression sublanguage only); monomorphization-only generics with explicit instantiation.

## Load-bearing card verification (2026-07-02)

D001, D002, D003, D005, D007, J004, A001, A002, A006 adversarially verified against re-fetched primary sources: CONFIRMED with verbatim quotes (see `notes/card-verification-2026-07-02.jsonl`). C004 CONFIRMED with amended sources — the data-race guarantee required adding nomicon/races.html; scope sharpened (data races prevented, general race conditions not). Consequence: the round-2 generics verdict (conditioned on D001) and the five verdicts conditioned on checked ownership/C004 keep their evidentiary footing. Remaining unverified provisional cards: the rest of the A/D/C/J series not in this load-bearing set, plus D009/D010 flagged in round 2.

## Round-3 safety-envelope states (2026-07-02)

| Topic | Outcome |
|---|---|
| unsafe-hatch | design_candidate_later |
| check-policy | design_candidate_later |
| ffi-attenuation | research_needed |
| proof-burden | design_candidate_later |

Decided now (see round3-synthesis-safety-envelope.md): proven-else-checked for D1-critical checkable facts in ALL build modes; no free-form writer-emittable unsafe anywhere; exactly one gated fact-boundary construct family with a per-fact obligation ledger; checks artifact-surfaced on the drop-op template; conservative no-emit fallback where no check form exists; overflow classified non-D1-critical (wrap fallback legal, unproven nsw forbidden); mandatory conservative FFI declaration frames + barrier semantics; foreign-shared memory pre-declared at allocation site; writer proof burden = round-2 floors exactly, nothing writer-stated trusted unchecked; solver may only promote performance facts, never gate acceptance.

Formal spine amendment adopted: "no third state" restated as "no WRITER-EMITTABLE third state; toolchain-gated ledger entries are the sole trusted-assertion class."

PENDING OWNER RULINGS (arbitrate hatch form): (a) D1 quantifier — unconditional theorem over all accepted source vs Rust-class conditional-on-confined-trusted-base; (b) D0 boundary — may humans author toolchain primitives and review gate requests.

## D2 corrected ruling (2026-07-02, owner)

Program-text verbosity is acceptable — benefits of explicit facts outweigh generation tokens. The constraint is spec compactness: few uniform rules, no special cases, stated once. The round-4 verbosity-budget topic is thereby DECIDED BY OWNER (uniform verbose-everywhere annotation stands; elision/inference gains no D2 support — it now costs spec budget instead of saving program budget). Round-2 "verbosity is free" is restored for program text.

## Owner rulings D1a/D0a (2026-07-02)

- D1a: Rust-class conditional envelope CONFIRMED, gated on checker-core feasibility (owner condition: rustc-scale borrow checker effort is unacceptable; simplified checker required). Blocking gate: checker-core prototype.
- D0a: gated-channel content is AI-authored, human-approved (initially); revisit after gate-efficacy experiments.
- Checker simplification levers adopted as design requirements (see notes/checker-feasibility-findings.jsonl): explicit regions/borrows over inference; soundness-over-completeness (checker may reject sound-but-hard patterns and demand restructuring); lexical borrows before NLL-class flow sensitivity; adopt a formalized minimal calculus (Featherweight-Rust/Oxide/Austral-class) rather than inventing rules; dynamic-oracle cross-check (Miri-style interpreter over canonical IR in debug builds).

## Round-4 teachability states (2026-07-02, v2 under corrected D2)

| Topic | Outcome |
|---|---|
| spec-budget | research_needed |
| familiarity | research_needed |
| compiler-as-teacher | research_needed |

Decided now: the spec becomes a SINGLE machine-checked, version-pinned CI artifact under regularity invariants (one spelling per construct; zero context-dependent rules; empty exception lists; each fact stated once; no sugar/inference in the writable surface); every design proposal must declare its spec delta (a spec-delta column is hereby part of this gate document's discipline); kernel (D1a ownership calculus + ledger + FFI frames + capabilities) is priced FIRST; no numeric token cap has cut authority until the calibration audit + spec-size ablation run; no-shadow-spec rule (writer-facing schemas count as spec mass); teaching pack is a generated, tested build artifact; spec-primary pedagogy with repair loop as reinforcement; conservative-extension law for any future partition.

Pending owner rulings (procedural): canonical counting procedure (tokenizer set, rule individuation, normative boundary); D2 binding unit (total description vs resident context).

## Calibration audit v0 (2026-07-02)

Kernel spec v0 drafted (`/Users/bytedance/Dev/xlang/spec/kernel-spec-v0.md`, 59 rules) and priced: 3208 tokens (cl100k_base), 3218 (o200k_base). Projected full teaching pack (spec + examples + diagnostics vocabulary): ~7-9k tokens. Spec-delta discipline starts from this baseline. See notes/calibration-audit-v0.json.

## Checker-core prototype (D1a gate) — 2026-07-02

PASSED initial gate: the D1a-simplified ownership calculus (affine ownership, lexical regions, borrow exclusivity, escape checking, reject-when-unsure) implemented in ~230 lines Python over a toy canonical AST; 16/16 tests (10 negative asserting exact rule IDs per DIAG-1, 6 positive). Evidence for frontend-scale effort per the owner condition on D1a. Remaining risk is rule soundness at scale, not volume — formalized-calculus adoption (Featherweight Rust/Austral-class) required before ratification. See notes/checker-prototype-report.json and /Users/bytedance/Dev/xlang/prototype/checker/.

## kernel-spec v0.1 delta (2026-07-02, META-5 declaration)

Revision under spec-critique round 1 (63 findings: 13 blocking, ~37 major, rest minor; 37 missing rules; archived at debates/spec-critique-round1-raw.json).

- Spec delta: +~18 rules (57 -> ~75 unique; recount pending rule-individuation procedure); tokens 3208 -> 6310 (cl100k); +0 alternate spellings; exception clauses reduced to 0 (STOR-2 default clause deleted; OP carve-outs became table data).
- Blocking fixes: OWN-4 direction inverted (unsound, fixed); OWN-10 added (borrow-storage duration; the dangling own-param return case); OWN-5 restated over resolved overlapping places; OWN-6 holder/resolution defined; move-through-borrow banned; copy/affine classification added; OWN-11 loop back-edge rule; STOR storage-by-type (no default clause); arena confinement; FN-4 law discharge restricted to stated-and-checked or gated ledger; Bool made prelude-enum-only; SCOPE-3 made conditional (Layer-4); EFF-2 syntactic exhibits; EFF-3 pure-elimination restricted (no termination assumption); prelude made normative; grammar closed (all nonterminals defined); conformance declarations and deref places added; kernel pricing scope corrected (CAP/GATE/LEDGER stubs added and counted).
- Deferred with recorded deltas: ERR-3 Result propagation; typed operation tables; env-struct exactness diagnostics; report field schemas; constant-expression sublanguage; stated-and-checked vocabulary beyond check.
- Checker updated to match: OWN-10 + holder resolution implemented; 19/19 tests; the v0 positive test that embodied the dangling case is now a negative test.
- Ratification preconditions unchanged: formal-calculus reconciliation (section 5), effect-exemplar carding (section 9), owner rule-individuation and counting-procedure rulings.

## kernel-spec v0.2 delta (2026-07-03, META-5 declaration)

Lexicon revision per owner rulings (D3): `&mut` renamed `&uniq` throughout; LEX-1 added (lexicon policy: invariant-naming, no backend-IR vocabulary at the surface, divergence census for borrowed keywords; two-axis mode vocabulary DEFERRED). Spec delta: +1 rule (LEX-1), tokens 6310 -> 6502 (cl100k), 0 new spellings (rename replaces a spelling), 0 exceptions. Artifact: /Users/bytedance/Dev/xlang/spec/kernel-spec-v0.2.md (v0.1 retained as history). Checker diagnostics wording aligned; 19/19 tests pass. Backlog: Rust-divergence audit of the full spec lexicon.

## Constitution adopted (2026-07-05)

THE GOAL + rules R1-R6 recorded in notes/user-directives.md; grounds D0-D3. Immediate consequences: (a) D1 re-grounded as shift-left for the AI loop (R4), not a standalone value; (b) R3 audit opened — constructs selected for spec-minimality rather than validated best-for-AI are now PROVISIONAL: first flagged item is the loop form (bare loop+break chosen for grammar minimality; counted-loop alternatives never evaluated for AI codegen error rates or provability); (c) Go pre-generics recorded as the R2 precedent case.

## Constitution amended (2026-07-05, owner)

THE GOAL restructured: floors (D1 + W3 cheat-proofness) / P0 performance with the Rust test R0 (every major decision names its delta over Rust) / P1 = W1 weak-writer robustness + W2 context economy; balance rule with P0 as post-evidence tie-breaker. Consequences: (a) all planned codegen experiments must add a weak-model tier (W1); (b) W3 unifies previously scattered anti-cheat rules (gating, no wildcards, proof-only elision, trap-not-silent, canonical bytes) under one named floor; (c) the running constitution-audit workflow evaluated the pre-amendment text — its findings must be re-read against this version.

## Constitution audit integrated (2026-07-05)

Full results: debates/constitution-audit.md. 22 decided items re-grounded with R-citations. Adopted immediately:

1. META-5 EXTENSION (resolves the structural D2-R3 tension): every spec delta now declares its SELECTION GROUND — evidence-selected vs minimality-selected — alongside rules/tokens/spellings/exceptions. Spec-delta PRICES candidates; AI-codegen + performance evidence SELECTS them. Minimality-selected forms are automatically PROVISIONAL.
2. PROCEDURAL BREACH acknowledged: TYPE-5 (interior annotation mandate) and FN-3/conform (interfaces replacement) shipped as normative while their round-2 verdicts sit at needs_evidence. Both are now marked R3-provisional in the spec status header; their pre-registered experiments (redundancy-independence; interfaces back-fill) are BLOCKING ratification preconditions.
3. R3-PROVISIONAL REGISTER (13 items) recorded in the spec status header: loop form, conditional form (no-if — never debated), statement-only match, prefix arithmetic surface, TYPE-5 interior, TYPE-6 no-shadowing, FN-5 env-structs, FN-3 contracts, FORM-1/2 byte-format choices (reject-vs-canonicalize untested), FORM-4 no-comments, FORM-5 decimal-only literals, checker levers (OWN-3/8/11 rejection-rate unmeasured), deref/index prefix forms.
4. Ratification preconditions EXTENDED with the R3 validation debts (AI-codegen validation harness; loop/conditional/syntax/redundancy experiments across W1 model tiers; N001/N002/D009/D010 card verification; R1 justification pass on human-residue rules; check-loop latency budget; CONST-1 delivery or evidence per R2).
5. Round-3 'wrap fallback legal' wording ANNOTATED: legal as an explicit per-node mode only; any future defaulting rule instantiating it would violate R4 + META-2.

## D4 recorded (2026-07-06)

Rewrite-first, FFI-narrow (owner). ffi_abi_runtime dossier RESCOPED: C-ABI out-calls, buffer pinning/foreign_shared, unwind-abort at boundary, single-threaded-entry contract for exports; JNI/GCHandle/cgo rich-interop precedent study reduced to the pinning/pointer-escape rules only; foreign-thread callback problem deferred behind the entry contract. Round-3 wall composite unaffected (quarantine + declaration frames were already the leading candidate); the C002-at-the-wall question is now scoped to foreign_shared buffers only.

## Card verification round 2 + formal-calculus candidates (2026-07-06)

Discharged from the ratification debt: N001, A005, C005, C002, D009, D010 CONFIRMED verbatim (notes/card-verification-2026-07-06.jsonl). N002 PARTIALLY_VERIFIED (nsw->poison corroborated by the verified UB manual; wraps-modulo verbatim quote blocked by LangRef fetch size — not refuted; pending sectioned source). Remaining unverified load-bearing: J001/J002, A003.

Formal-calculus dossier (K003/K004): **Featherweight Rust selected as the section-5 reconciliation target** — soundness-proven, lightweight, Java reference implementation, validated by 500B-program model checking and rustc fuzzing (which found a real rustc bug — evidence that adversarial validation of borrow checkers works and that even production checkers have bugs). Oxide recorded as the NLL-upgrade reference (its fully-annotated-types premise matches our no-inference direction). Reconciliation task: map OWN-1..13 onto FR rules, prove-or-fix divergences, and adopt FR-style model checking for our checker.

## static-vs-profile resolved by layering (2026-07-06, owner)

Owner's layering question dissolved the round-1 topic (research_needed since 2026-07-01):
- Language half DECIDED: no runtime speculation machinery (no JIT/deopt/OSR/safepoint semantics — already law), and no writer-stated performance hints ([[likely]]/#[cold]-class rejected under R1: unverifiable data-habit assertions; measured-beats-asserted; zero checkable invariant).
- Toolchain half OUT OF LANGUAGE SCOPE, recorded as two policy lines: (1) profiles are cost inputs, never acceptance inputs — acceptance remains decidable from the canonical artifact alone (DIAG-2); (2) any profile used in a build is a declared, content-addressed build input (reproducibility).
- Residual research item (descriptive CFI / stack-attribution without F008-class codegen constraints) moves to the toolchain track; it feeds trap-report telemetry, not language design.
- Area-6 closing question about guarded value speculation in AOT binaries reclassified: an LLVM pass-pipeline question inside already-stated semantics, not a design question.

## D2a recorded (2026-07-07)

W2 deprioritized (owner): token counting stays as measurement, never gate; regularity invariants retained under W1 grounding; spec-size ablation and resident-vs-total ruling deprioritized; rule-individuation ruling now needed only for the ratchet, not for cap arithmetic.

## Constitution relocated (2026-07-07)

Authoritative text moved to /Users/bytedance/Dev/xlang/CONSTITUTION.md (top-level, discoverable; owner could not find it in notes/). user-directives.md now holds only the D-rulings plus a pointer; D2a folded into the constitution W2 clause. Historical gate entries referencing the old location remain as history.

## Constitution restructured: floors -> standing theorems (2026-07-07, owner)

Memory/thread safety (D1) and the no-UB envelope are NOT constitutional axioms; they are STANDING THEOREMS derived from the goals — from P1 (R4 shift-left, W1 undebuggable-at-runtime, W3 no-papering-over) and independently from P0 (F001 ownership=noalias facts; race-freedom keeps proofs sound). Rust-vs-C/C++ recorded as the natural experiment. W3 moved under P1 as a component of AI-writability. Practical force unchanged: theorems hold while premises hold, revisitable only by refuting a derivation, never by preference. No downstream decision changes; round-3 envelope and D1a/D0a rulings stand on the theorem exactly as they stood on the floor.

## Derivation ledger built (2026-07-07, owner-mandated)

spec/derivation-ledger-v0.2.md: all 75 rules traced against the current constitution. Result: 36 derived / 39 derived_existence_only / 0 underived. META-6 added (every rule must carry a ledger entry; orphaned chains auto-flag; underived rules may not ratify). Notable: (a) zero underived — no rule exists for literally no reason, but the 39 existence-only entries confirm the audit: roughly half the spec's specific FORMS await their R3 experiments; (b) new register gaps found by the ledger itself: FORM-3 sigil choices, FORM-6 unit token (zero provenance), FN-6's over-strong polymorphic-recursion criterion, FN-7 no-globals (uncarded plausibility), OP-2's ineg.wrap exclusion (two's-complement wrapping negation IS sound modular arithmetic — the div/rem rationale does not cover it; fix or justify); (c) weakest-chain list is the priority re-grounding queue; META-4 and several FORM rules are W2-only post-D2a and need W1 re-grounding.

## kernel-spec v0.3 delta (2026-07-07, META-5/6 declaration)

Ledger-fix revision. Selection grounds: evidence-selected (OP-2 negation fix — semantic argument; META-4 W1 re-grounding), recorded-rationale (FORM-6, FN-6, FN-7, FORM-3 census entry). Spec delta: +1 op-table row (ineg.wrap), +0 rules, ~+550 tokens (6502->7059), 0 new spellings, 0 exceptions. Ledger stats moved 36/39/0 -> 38/37/0 (derived/existence-only/underived; OP-2 and FORM-6 promoted; META-4 was already derived and only re-grounded). Living ledger renamed to spec/derivation-ledger.md (v0.2-named file retained as history). Remaining from weakest-chain queue: FORM-4 no-comments experiment, GRAM-7 statement-only-match A/B, TYPE-6 shadowing evidence, OWN-9 noalias benchmark carding, DIAG-3 schema delivery — all registered.

## v0.3.1 delta (2026-07-07)

ERR-3 propagation (try_stmt) + ERR-4 classification added — closes the R4-load-bearing deferral. Evidence-selected (R4/W1/W3 chain recorded in-rule). +2 rules, +1 production. ROADMAP.md added at repo root.

## Spec-CI online (2026-07-07)

tools/spec_ci.py enforces META-1 uniqueness, cross-reference integrity, META-6 ledger coverage, META-3 exception scan against the latest spec version. First run caught META-6 missing its own ledger entry (fixed, self-referentially) and validated itself by correctly flagging v0-era defects when mis-pointed. Ledger: 41 derived / 36 existence-only / 0 underived (77 rules). Run before every spec change.

## Checker extended: OWN-11/OWN-12/OWN-6-temporaries (2026-07-07)

292 lines, 26/26 tests. Rule-precision preserved: intra-call conflicts cite OWN-12, not OWN-5 (DIAG-1). Remaining before FR reconciliation: OWN-13, slices, copy/affine expression-level enforcement.

## Demo compiler online (2026-07-07)

prototype/democ/democ.py: micro-subset source -> parse -> prototype ownership checker -> LLVM IR -> clang -O2. Demonstrated end-to-end: (1) dangling-borrow program REJECTED citing OWN-10 before codegen; (2) &uniq/& lower to noalias/noalias-readonly, iadd.wrap to unflagged add (N002), iadd.trap to sadd.with.overflow + trap block (SCOPE-4); (3) MEASURED P0 payoff: optimized code performs 1 load with ownership facts vs 2 without (read-after-store of a distinct-by-checker location eliminated) — the F001/Area-1 claim now reproduced by our own pipeline. Owner ruling recorded: the checker/compiler prototypes are temporary; the endgame is self-hosting (bootstrap ladder: host-language demo -> real compiler -> compiler in xlang as the ultimate R0 dogfood).

## Build plan M0-M4 ratified (2026-07-07)

Order rationale recorded: M0 (FR reconciliation) precedes M1 (reference compiler) because the compiler core implements §5 and rule changes would rework it; M1+M2 constitute the instrument (M3) that the empirical spec questions require; research track parallel. democ 1-vs-2-loads native measurement noted as partial discharge of the OWN-9 benchmark debt.

## M1 first increment: EX-1-class program compiles AND RUNS (2026-07-07)

democ grown to: enums, match (Bool/Result/user-enum scrutinees, binders), check-else-trap, region stmts, doc fields, cross-fn calls, iadd.checked->Result pair repr, runnable main. examples/ex1.xl (sign_of + main with checked arithmetic, Result match, two runtime checks, cross-fn call) compiles to native and RUNS: exit 0, both checks pass. Regressions green (twice_read OK, dangle REJECTED OWN-10 exit 1). SPEC FINDING by construction: GRAM-4 says `match place` but spec EX-1 matches an EXPRESSION (`match ilt<i32>(x, 0_i32)`) — GRAM-4/EX-1 contradiction; fix queued for v0.4 (widen scrutinee to expr, or canonicalize EX-1 to bind-then-match). Known prototype approximations recorded in democ source: match-arm checker mapping is sequential (OWN-13 move precision TODO), effects blob unvalidated (EFF-2 TODO).

## kernel-spec v0.4 delta (2026-07-07, META-5/6)

GRAM-4 match scrutinee: place -> expr (fixes GRAM-4/EX-1 contradiction). Selection ground: evidence-adjacent — bind-then-match alternative rejected under R3/W1 (taxes the only conditional idiom with an invented temporary per use; weak-writer naming burden); democ already implements expr scrutinees. OWN-13 += owned-temporary clause. +0 rules, ~+90 tokens, 0 spellings, 0 exceptions. Ledger chains for GRAM-4/OWN-13 unchanged in kind.

## democ enforces ERR-2 (2026-07-07)

Non-exhaustive match now REJECTED with rule ID (examples/nonexhaustive.xl: "have [True], need [False, True]", exit 1). W3 floor (exhaustiveness cannot be silenced) is enforced, not aspirational. Regressions green: ex1 runs exit 0; dangle OWN-10 exit 1.

## M1 increment 2 + trap-elimination measured (2026-07-07)

democ: loop/break codegen, mutable own locals (alloca, mem2reg-cleaned), full isub/imul wrap/trap/checked families. ex2.xl (loop-summed 0..4 with iadd.trap counter/accumulator + two checks) compiles and runs exit 0. MEASUREMENT: IR contains 5 overflow-checked ops; clang -O2 output is mov w0,#0; ret — every trap branch eliminated BY PROOF (0 b.vs), loop constant-folded, checks statically discharged. The Area-2 three-tier prediction (induction traps are free) confirmed in native code; the whole guarded program folded to return 0. Regressions green (ex1 runs, twice_read compiles, nonexhaustive ERR-2 exit 1).

## M0 reconciliation memo drafted (2026-07-07)

spec/fr-reconciliation-m0.md: OWN-1..13 + STOR-4 mapped to Featherweight Rust; verdicts: 8 equivalent-or-stricter (sound by restriction), 2 extensions beyond FR (calls, sums) with named proof obligations, 0 weaker-than-FR rules found. Two structural theorems recorded: T-A singleton provenance (no borrow rebinding => FR path-sets degenerate to our one-place model — the load-bearing D1a simplification, a language choice not a checker shortcut) and T-B arm isolation. Section 5 status: reconciled-modulo-OBL-0..3 (OBL-0 = verbatim paper check, blocked on fetch; OBL-1/3 land with M2 harness; OBL-2 is a proof note). Not yet ratified — honest gate.

## M2 online: oracle + model checker; OBL-1/OBL-2 progress (2026-07-07)

OBL-2 discharged (OWN-12 = OWN-5 closed under simultaneity; proof note in memo). M2 instruments built: independent oracle interpreter (oracle.py) + generative model checker (modelcheck.py). Run: 20,000 programs, 14,905 accepted, 0 soundness violations; over-rejection = 2.9% of rejections oracle-clean — first measurement of the OWN-8/D1a lever cost (audit item "rejection-rate unmeasured" now has a number for the base fragment). OBL-1 partially discharged; OBL-0 (verbatim FR paper pass) and OBL-3 (arm isolation) remain.

## OBL-3 discharged; match lands in checker (2026-07-07)

Arm isolation + conservative join implemented (28/28 tests, incl. the false-rejection fix and the join negative). democ uses real match nodes. All-paths oracle (<=256 paths). Model check with match generation: 20k programs, 0 soundness violations, over-rejection 1.1%. Section-5 ratification now blocks ONLY on OBL-0 (verbatim FR paper verification).

## Model-check hardened; perf pinned; parser regression caught (2026-07-07)

30k programs (calls/params/caller-horizon oracle): 0 soundness violations; true over-rejection 7.2% (all OWN-10 temporaries — measured, refinement recorded). perf_regress.py pins the 1-vs-2-loads noalias win — and its first run caught a REAL parser regression (deref swallowed as user call; IR referenced undefined @deref; earlier smoke check missed it by testing democ exit only, not clang). Fixed; all regressions green including perf pin. Lesson recorded: regression checks must assert the artifact, not the tool exit code.

## OWN-13 fully implemented (2026-07-07)

Binder modes now DERIVED per OWN-13: own scrutinee moves (post-match use rejected OWN-1); borrow-mode scrutinee stays live, binders become aliasing borrows of its content (uniq binder conflicts with root re-borrow: rejected OWN-5); expression scrutinees are owned temporaries. Oracle mirrors the dynamics (own-scrutinee move observable). 31/31 tests; 30k model check clean chain (exit 0 => 0 soundness violations); true over-rejection improved again 7.2% -> 5.5%; democ ex1/ex2 run; perf pin intact. Checker OWN coverage now complete: OWN-1..13 all implemented and generatively tested.

## OBL-0 DISCHARGED — §5 formally reconciled (2026-07-07)

FR preprint fetched, archived in sources/, and verbatim-verified (page-anchored quotes in the memo): all five memo claims CONFIRMED, plus exact alignments (Def 3.6 copy classification = OWN-1 verbatim; FR itself bans shadowing = TYPE-6; OWN-5 clause set matches T-Move/T-Copy/T-MutBorrow/T-ImmBorrow rule-for-rule; Def 3.21 containment = OWN-4 direction letter-for-letter). The single permissiveness delta located precisely: FR supports borrow reassignment with retyping (p.25 ex.17) — the feature whose removal makes T-A hold; our model is a strict sound subset of FR state space. ALL M0 OBLIGATIONS DISCHARGED (OBL-1 at fragment scope). §5 ratification now awaits only owner sign-off.

## v0.4.1 delta + lexicon census (2026-07-07)

DIAG-3 field schemas delivered in-spec (trap/check/lifetime/check-density tables) — closes the audit-flagged R4-load-bearing deferral; ~+300 tokens, 0 rules, evidence-selected (R4 grounding recorded at flag time). D3 lexicon census recorded (notes/lexicon-census.md): 8 PASS, 1 HOLD (region sigil, kept provisionally), 0 FAIL — with the reusable errs-toward-rejection principle: borrowed names are safe when every prior-driven misuse lands as checker rejection, never accepted-but-wrong. D3 audit backlog item substantially discharged.

## R0 reading affirmed + v0.6 polish (2026-07-08)
Owner affirmed R0 credits a delta on ANY of P0 / W3 / W1 (not machine-performance only); equivalence to Rust on all three is the failure condition. Recorded in CONSTITUTION.md. v0.6 polish rulings: user-fn calls use NAMED arguments in declared order (GRAM-11) — reordering rejected (FORM-1 one byte sequence; names are R4 checked-redundant facts, symmetric to GRAM-8 construction), table-operation calls stay positional; enum variant constructor names are GLOBALLY UNIQUE (TYPE-6, closed world); `reinterpret` extended to same-width int<->int resign (OP-1/OP-8), giving bit-level resign a home distinct from value-preserving `cvt`; built-in Int/Float numeric conformance licenses op-table rows + 0_T/1_T for bound gparams (FN-3). make check green (89 rules). Deferred: give-keyword spelling confirmed `give`; scrutinee pre-bind-vs-inline FORM-1 residue flagged for a separate ruling.

## Post-audit diagnosis on the "no gain over Rust" dead end (2026-07-09)

The negative audit is VALID for what it measured and INCOMPLETE for what it concluded. It benchmarked only the fact channels democ already emits (parameter noalias from borrow modes; arithmetic definedness) — precisely the channels where rustc emits the SAME facts, so parity was the corpus's own prediction (F001: rustc gets noalias from &mut/&). Constitution P0 names FOUR delta channels; democ emits ~1.5. Untested channels, all structurally absent from Rust:

1. REGION-SCOPED INTRA-FUNCTION ALIAS METADATA (F003 alias.scope/noalias): rustc emits noalias at function boundaries only; interior loans are invisible to LLVM. Our checker knows every loan region -> emit scoped metadata inside bodies. No Rust source channel exists.
2. EFFECT ROWS -> FUNCTION ATTRIBUTES: pure -> memory(none)+nounwind; reads-rows -> memory(read)/per-arg readonly. Declared+checked=guaranteed, vs LLVM attributor heuristics that give up across cycles/opacity; Rust has no source channel.
3. CHECKED-LAW CHANNEL (FN-4): associativity/commutativity of USER ops -> reassociation/vector reduction of user-defined operations — impossible for LLVM on calls, inexpressible in Rust; AND the autopar postmortem's named missing facts (element injectivity, checked commutativity) are law-channel citizens. The ai-native-parallelism plan-bundle design (untrusted AI plans + compiler verification/guards) is round-3's gated-obligation architecture reinvented for parallel plans — convergent validation; formalize bundles as ledger-family artifacts.

PLAN RATIFIED (phases): 0 hygiene (section-5 status reconciliation pending owner ratification word; docs at v0.6) -> A M3-unblock democ subset (buffer/index/len OP-9, try/ERR-3, bytes; pool/handle ruling) -> B build the three channels + DISCRIMINATING benchmarks vs real rustc with pre-committed thresholds (interior-loan kernel with non-inlinable boundaries; call-heavy pure chains vs Rust±fat-LTO; user-op reduction; law-guarded scatter) -> C run the M3 distributional sprint as designed (tiers/budgets) -> R0 decision: self-host iff (>=1 robust B delta) OR (C W1 win); else stop/pivot (named options: verified-facts frontend; upstream scoped-loan metadata to rustc; research artifact).

Honest risks pre-registered: channel 1 may be recovered by LLVM post-inlining (benchmark must use opaque boundaries; result may be modest); channel 2 competes with ThinLTO attributor inference; channel 3 is uncontested structurally but needs a consumer pass; W1 sprint may show parity too — in which case R0 stop/pivot applies honestly.

## Channel 2 BUILT + MEASURED (2026-07-09)

democ emits effect-row function attributes (pure->memory(none)/argmem-read; traps->inaccessiblemem: write; nounwind) PLUS a derived-totality tier (loop-free + total callees + trap-free => willreturn; sound, writer-free) — added after measuring that memory(none) alone hoists nothing (LICM needs willreturn; EFF-3 honesty gated the channel). Four-way benchmark at 2e9 iters, opaque-boundary callee: xlang-with-facts 0.00s (O(1): hoist + strength-reduce) vs xlang-control 1.47s vs Rust-cross-crate-noLTO 1.49s vs Rust-fatLTO 0.00s. Verdict: complexity-class win over Rust's default build shape; tie with Rust's most expensive config; guarantee-vs-heuristic and facts-survive-opaque-boundaries are the durable deltas (=> LTO-grade optimization at per-file build cost; feeds FFI-frame future). make check green incl. new totality path. First of three unbuilt channels now built; scoped-alias metadata (channel 1) and FN-4 law consumer (channel 3) remain.

## Phase A progress: buffer stack lands (2026-07-09)

buffer<T> end-to-end: OP-9 buffer_new (u64 size-overflow trap before alloc, fill loop), OP-4 bounds-checked index as a place (read+write, trap OOB), len as non-consuming place-read operand, {ptr,i64} cross-fn ABI, and a then-implicit move-at-call approximation for own affine args. Conformance 179->183 PASS (3 pending activated + new OOB-trap case); two case files had spec-WRONG effect rows (pure/missing traps) — the implementation correctly rejected its own test suite per OP-4/OP-9 exhibits; cases corrected. M3 blocker `buffer_index_kernel` UNBLOCKED. Remaining M3 blockers: try/ERR-3, byte-parser surface (buffer<u8> now exists — needs only u8 literal ergonomics check), arena/pool ruling (STOR-1 already REJECTS index-pools; arena codegen missing). Channel-1 benchmark now possible once slice_of/loan metadata lands. AMENDMENT 2026-07-10: the implicit call conversion was removed; OWN-1 requires `move` explicitly and both type and flow layers now see the same transfer.

## ERR-3 try lands (2026-07-09)

try-propagation implemented end-to-end: parse (let..= try e), TYPE layer (Result payload types preserved through parse_type/ttype; checked ops now return Result<T,Overflow|DivError> composites; same-E enforced exactly per spec — the m3-flagged false-positive is structurally prevented; erased-E rejected conservatively), FLOW layer (try/return/own-args consume affine operands — implicit-move-at-consumption per FR T-Move), codegen (try -> synthesized Ok-bind/Err-re-return match; enumv aggregate packer; %Result/%Option {i32,i64} ABI; call returns spilled to slots). Conformance 183->185 (ERR-3 pos run + same-E neg). M3 blocker error_propagation_chain UNBLOCKED. Remaining M3 blockers: checked_integer_parser surface (buffer<u8> exists; needs an end-to-end byte-parser attempt), arena_ast_builder (arena codegen + STOR-1 pool ruling).

## Phase A COMPLETE minus arena (2026-07-09)

All three remaining M3 references land and run: checked_integer_parser (parse_u64 over buffer<u8>, 6 cases incl. u64::MAX; codegen bug found by bisection — prelude Err construct dropped payloads, fixed), error_propagation_chain (try chains, wrong-error-kind discrimination), buffer_index_kernel (fill/checked-sum 523776, OOB expressed as bounds-guarded branch; executed-trap variant lives in conformance op4-trap-index-oob). Harness gains per-language expected (expected_xlang {exit:0}): xlang has no print surface, traps discriminate failure — negative-tested (mutated sum => fail exit -5). Lexer hardening en route: signed literals (-1_i64 silently lexed as 1_i64 before), FORM-1 catch-all hard error, @label token class that had never been lexed. Reference matrix: Rust 7/7, xlang 6/6-runnable + 1 pending (arena_ast_builder, gated on STOR-1 pool/handle OWNER RULING, not code). M3 model-evidence blocker is now dominant. Next per plan: Phase B channel 1 (region-scoped alias metadata from loans; needs slice_of in democ; benchmark = split-halves vs Rust split_at_mut) and channel 3 (FN-4 law consumer), then Phase C model sprint.

## Channel 1 BUILT + MEASURED (2026-07-09)

Scoped-alias metadata from ownership provenance lands in democ WITHOUT new spec surface: OWN-1/T1 (affine buffers pairwise-disjoint) + OWN-2/5 (&uniq exclusivity) + T-A (singleton provenance) license per-provenance-class !alias.scope/!noalias on every buffer-element and struct-memory access rooted at a borrow/own param, plus dereferenceable/align from borrow validity. En route democ gained: `.` token + psuffix places (deref(s).a), post-deref field model, buffer-in-struct fields ({ptr,i64} layout, construct/set/read), move-expr parsing (GRAM-5 gap — checker demanded what democ couldn't parse), imin/imax lowering, borrow-of-struct-local fix. Benchmark (experiments/scoped-alias-channel): 8-column SoA kernel, opaque boundary, vs rustc -O3 three shapes. Result: xlang-facts vectorizes with ZERO guards (121 asm lines); Rust obvious shape vectorizes via loop-versioning (29 runtime guards, 2132 lines) — ties at n>=32 (pre-registered "recovered by runtime checks" risk REAL for long trips), loses 2.0x at n=8, 1.18x at n=16; even expert innerfn (the only safe-Rust noalias idiom) loses 1.17x at n=8. Durable deltas: short-trip perf, 17x code size, static-O(1)-vs-quadratic-runtime disambiguation, W1 obvious-shape-is-fast at every n. make check gains pin #2 (soa_kernel must vectorize guard-free with facts, not without). Channel 3 (FN-4 law consumer) remains. Follow-ups: wider-struct bail-point sweep; aligned-alloc large-n re-measure.

## Channel 3 BUILT + MEASURED (2026-07-09)

FN-4 checked-law channel lands end-to-end: democ parses contract/law/conform (FN-3 signature validation; closed LAWNAME table), statically discharges laws by the one honest demo shape (bound fn body = single table op whose law is OP-8 table data, signedness-aware), and consumes them: reduction loops over proved assoc+comm+identity ops are reassociated into 4 block-interleaved accumulators seeded with the proved identity (interleave licensed by assoc+comm, seed by identity; original loop = scalar tail; rewrite runs post-check, codegen-level, gated on facts flag). En route: SPEC ERRATA found+fixed — FORM-3's closed OPNAME mode set omitted `sat` while OP-2/OP-7/OP-8 define .sat ops (Tier-0 internal contradiction class); FORM-3 now {wrap,trap,checked,sat,strict} in spec+democ, iadd/isub.sat lowered (llvm.[su]{add,sub}.sat), imin/imax lowered + added to dotless table. Benchmark (experiments/checked-law-channel): u64 sat-add fold, opaque boundary, vs rustc -O3. facts 0.156 ns/elem vs control 0.511 vs rust-obvious 0.512 (3.3x) vs rust-expert-4acc 0.159 (tie) at n=65536, identical sinks. The W3 jewel: rust-expert is an UNCHECKED human assertion — the same shape over signed sat-add compiles silently to garbage in Rust; xlang REFUTES the stated law compile-time (fn4-neg-law-refuted-signedness). Conformance 185->192 (fn3/fn4 pendings activated after fixing their spec-wrong effect rows — third instance of that case-file bug class; 3 new FN-4 discharge/refute cases + op2 sat case). make check pin #3 guards the reassociation. ALL THREE PHASE-B CHANNELS NOW BUILT+MEASURED with differentiated deltas: ch1 short-trip+17x-code-size, ch2 O(n)->O(1) at opaque boundaries, ch3 3.3x + refutation-of-false-laws. R0 condition ">=1 robust B delta" is arguably met by ch2+ch3; Phase C (model-tier W1 sprint) is the remaining evidence leg and is user-gated (needs real weak/middle/strong model runs).

## Phase C READY (2026-07-09)

Phase C (model-tier W1 sprint) is now one command per tier: m3/harness/trial.py drives generation + fixed repair budget around ANY external model CLI (--gen-cmd, prompt on stdin / program on stdout), machine-output-only feedback per the DECISION_SPRINT protocol, JSONL per attempt + per-trial summary, sources archived under m3/submissions/generated/<tier>/<language>/. The W1 test material is m3/prompts/xlang-spec-excerpt.md (167-line writer's excerpt; worked example VALIDATED end-to-end against democ — validation caught a democ subset gap: payload-carrying enum variants inside Err() are not erased into the Result word; excerpt teaches nullary error variants; gap recorded for later) and rust-guardrails.md. Dry-run verified: pass path (mock emits reference -> PASS first-shot) and repair path (mock emits broken program -> 3 attempts recorded, FAIL summary). Owner action to run Phase C: supply weak/middle/strong --gen-cmd values, e.g. `python3 m3/harness/trial.py --language xlang --tier weak --gen-cmd '<model-cli>' --trials 3 --repairs 3 --out weak-xlang.jsonl` x {xlang,rust} x 3 tiers, then score.py with --require-decision-ready. Remaining owner gates: STOR-1 pool/handle ruling (arena task), section-5 ratification word, R0 after Phase C.

## R0 self-host gate REWORDED (2026-07-09, owner-approved)

Old gate: "self-host iff (>=1 robust Phase-B delta) OR (Phase-C W1 win); else stop/pivot."
New gate (per D5 — Phase C deprioritized because model capability is improving faster than the weak-writer test depreciates): "self-host iff the Phase-B deltas are shown to be FREQUENT, not merely real: a channel-pattern frequency study over real corpora must show the patterns (opaque hot calls for ch2; alias-guard loop versioning for ch1; custom/manual reassociation idioms for ch3) occur at rates that make the measured deltas project-visible, weighted together with the qualitative W3/determinism case; else stop/pivot." Phase C becomes optional validation (harness + excerpt stay shelf-ready). The constitution's R0 per-decision reading is unchanged (W1 remains a valid delta axis; only its empirical sprint is deprioritized). Next unit of work: the frequency study — concrete proxies: (a) rustc -Rpass-analysis=loop-vectorize memcheck remarks + versioned-loop code-size delta over a crate corpus, (b) surviving non-inlined call sites in hot functions under default release builds, (c) syntactic mining for custom folds / saturating accumulation / hand-rolled multi-accumulator idioms.

## Owner design review: deep-write access + lifetime shape (2026-07-09)

Owner probed two soft spots. (1) Lifetime shape: regions+effects cannot infer the problem away (MLKit precedent: pure region inference degenerated, GC added inside regions; Cyclone hybrid) — design target is phase-shaped-majority coverage: nested region staircases + static-nursery promotion (inner-region alloc, explicit move-out for survivors — expressible today with regions+affine moves) + box for the interleaved residue, with allocates('r)/allocates(heap) rows making the split signature-visible. Card Koka/Cyclone/MLKit (§9 backlog) BEFORE speccing v0.7 arenas. (2) Deep-write: conceded that v0 is STRICTER than Rust in one axis — uniq borrows are affine with no reborrow (T-A), so deep call chains cannot implicitly carry &uniq down and keep it; options are linear threading (verbose, no tuples in v0) or restructuring. Blessed idiom recorded: command-buffer / write-intent pattern — deep code is reads('p)/pure and returns write intents as values; ONE shallow fn is writes('p) and applies them under the single &uniq. Checkable via effect rows; parallelism-ready (D1). Genuine dissolves vs Rust: no receiver methods -> no whole-object borrows (OWN-7 prefix rule permits cross-call disjoint field borrows, incl. &uniq one + & another); &uniq subsumes read + value copy-outs -> no long-lived interior refs. Relief valves if evidence demands (carded, evidence-first): Rust-style reborrow (D1a-rejected for checker complexity), split_uniq disjoint views by construction, checked Cell-for-copy-types. Honest residue: in-place mutation interleaved with traversal of the same structure stays awkward-or-rejected (OWN-8 bet).

## Owner review: totality economics + AoS path + evidence bar (2026-07-10)

(1) TOTALITY IS INFECTIOUS — owner flag, must not be forgotten. willreturn
requires may-abort-free: ONE trapping op in a leaf (e.g. a bounds check in a
low-level API) strips derived totality from every transitive caller, killing
call-hoisting/CSE for the whole tower above it. Rust suffers identically
(panic paths block LLVM willreturn inference) but offers no lever; we have
four, only one built: (a) BUILT — non-trapping modes exist for all arithmetic
(.wrap/.sat/.checked); the unavoidable trap source is bounds checks; (b)
CONST-TRIP TIER (unbuilt): a loop with literal-bounded trip count is
willreturn — sound, cheap, extends compute_total beyond loop-free; (c)
PROOF-ELIDED CHECKS (spec'd in OP-4, unbuilt): when OUR checker/proof artifact
proves in-bounds, the trap is not emitted at all and the fn re-enters the
total tier — proof-driven elision is the totality-restoration lever, not just
a speed lever; (d) DIAGNOSTIC TIER (unbuilt): surface "this fn is N traps away
from total" / totality-blocker attribution so writers (and the pattern
doctrine) can steer hot towers toward total leaves. Backlog-registered;
priority rises with any real port study.

(2) AoS PATH: v0 buffers hold copy primitives only, forcing SoA (3 columns for
Ast). SoA is doctrine-blessed (P2) and usually the FASTER layout, but
buffer<PodStruct> (AoS) needs a COPY-STRUCT TIER (a struct of copy fields may
be declared copy) — smaller than the SS5-blocked affine-element cluster;
carded as the v0.x path when AoS is genuinely wanted.

(3) EVIDENCE BAR RAISED (owner): the conventions-erode/W3 argument and the
channel deltas remain HYPOTHESES until shown on a non-trivial program. Agreed
two-legged plan: leg A = the queued channel-pattern frequency study over real
Rust corpora; leg B = PORT STUDY — reimplement a small real Rust program in
xlang (target must fit the subset: compute-bound, fixed-size data, byte-level
IO at most; candidate class: bytecode VM / protocol codec / table-driven
interpreter, i.e. compiler-adjacent workloads) and measure end-to-end: runtime
vs rustc -O3, code size, guard/versioned-loop counts, emitted-fact census,
and audit metrics (what fraction of behavior is signature-determined).

## Owner steer: interpreters are dispatch-bound (2026-07-10)

Owner (direct wasm-VM implementation experience): interpreter handlers are
trivial (add.i32, local.get); the pressure is DISPATCH. A naive switch-in-loop
compiles to one shared indirect branch — same machine code from every
language, so no channel delta there; the fast shapes are threaded dispatch
(computed goto) and musttail chains (wasm3/protobuf/Deegen style,
musttail + preserve_none), which C owns and Rust cannot express (no stable
guaranteed TCO, no computed goto). Consequences: (1) VM interpreter DROPPED as
a leg-B channel showcase — it would measure dispatch codegen we have not
built; leg-B candidates stay fannkuch (index compute) + codec class (blocked
on const-array codegen). (2) CARDED as candidate CHANNEL 4 — "blessed
interpreter pattern": lower the naive loop{match-on-opcode} source shape to
threaded/musttail dispatch in our own codegen (emit musttail+preserve_none
chains or tail-duplicated indirectbr from the pattern). Delta over Rust: real
and structural (inexpressible there). Delta over C: parity with expert
hand-threaded C, win over naive C. P0+W1 shaped: the obvious shape becomes the
fast shape — pattern doctrine's thesis applied to control flow. Needs carding
(BTB behavior, SimplifyCFG re-merging hazards, preserve_none availability)
before any spec/impl work. (3) Nuance kept on record: channels are not
irrelevant to interpreters — keeping pc/sp/stack headers in registers across
handler bodies is an ALIASING fact (channel 1's exact shape: state struct
behind &uniq, stack/locals/linear-memory as buffer fields), and wasm linear
memory is bounds-checked buffer indexing (OP-4 elision territory). Dispatch
dominates only when the opcode stream defeats prediction.

## Leg-B pilot: binary-trees port MEASURED + VERIFIED (2026-07-10)

Full numbers + caveats in experiments/port-study/binary-trees/RESULTS.md. Adversarial 3-lens panel (equivalence/fairness/claims) confirmed equivalence and fairness (facts/nofacts diff is metadata-only; calloc parity; xlang carries strictly more checks — conservative direction) and CORRECTED the headline: the 12x-vs-Box number is a shape effect present in Rust-vs-Rust too (13.5x); the honest claims are (a) FLOOR-RAISING — v0's borrow rules make the slow per-node-alloc design unrepresentable and steer the port to the fast SoA shape; (b) facts worth 1.45x on a real program; (c) checked-semantics tax ~11% vs identical-shape Rust WITH facts (61% without) — part of the 11% is trapping arithmetic Rust doesn't do, i.e. more safety, not worse codegen; (d) "only expressible shape" retracted — batch-1's greenlit OWN-6/OWN-14 reborrow deltas will legalize the recursive arena shape when implemented, at which point this pilot should be re-run with it. Panel also flagged: single-author references, default allocator on the Box variant, and Benchmarks-Game-rules caveat (game requires per-node alloc semantics; claim is about the workload, not the leaderboard). Verification cost note: panel ran on the inherited top-tier model BEFORE the owner's model-tiering directive; future panels go to opus-tier.

## Safe-direction pilot: wc port MEASURED (2026-07-10)

Direction reset per owner (avoid filter-sensitive domains; pure performance/correctness framing; log every step for rewind durability). wc chosen (zero new subset features needed) -> built same-day: xlang kernel (count_lines vectorizable scan + count_all state machine, results via &uniq out-borrow — struct-return hit the arm64 sret ABI mismatch, out-borrow is the ABI-clean idiom) + C slurp driver. Correctness: byte-identical vs system wc; 45/45 fuzz-diff under LC_ALL=C. Perf (426MB warm): full counts 0.28s vs GNU 0.48s (1.7x) vs BSD 0.54s (1.9x); -l 0.10s vs BSD 0.33s but GNU memchr 0.05s (honest 2x gap, hand-tuned SIMD vs our naive shape). TOTALITY ECONOMICS VINDICATED EMPIRICALLY: iadd.trap counters -> 0 vector ops; bounded-counter .wrap switch -> 24 vector ops, 2x on -l. Also fixed en route: _llty(Bool) was i32, breaking Bool-typed give-slots (make check still green, 192 PASS). Next: base64 (pulls const arrays). Durability protocol: commit + gates line per step.

## wc ladder completed vs uutils + memchr ceiling (2026-07-10)

uutils(Rust) wc installed + measured: DEFAULT full counts xlang 0.27s beats GNU 0.48 (1.8x), BSD 0.54 (2.0x), uutils 0.56 (2.1x — the Rust rewrite is SLOWEST on the default path while its -l is fastest at 0.03s via bytecount SIMD: obvious-vs-fast-shape distribution, live in a shipped product). -l gap decomposed: slurp-driver page-fault overhead ~half (memchr-with-our-driver 0.09 vs GNU-chunked 0.05), kernel shape ~half. Queued: chunked driver; count-matches blessed pattern (assoc+comm reduction, channel-3 family, bytecount-class lowering).

## CONST-2 const items implemented (2026-07-10)

Gap #1 (D7a) closed: const arrays + scalar consts end-to-end. democ: parse `const name: type = cvalue;` (array literal or scalar), array<T,N> type retained through parse_type/ttype, emit `@__const_<name>` private constant globals, const-array index -> bounds-checked GEP into the global, len -> static N, scalar const -> literal fold. checker: consts seeded into TypeChecker.env + ownership Checker.bindings (is_const flag); move/set/&uniq of a const rejected [CONST-2]; const-eligibility enforced (primitive or array<primitive,N> only — box/buffer/arena/slice/struct/enum rejected). Conformance 192->196 (const2-pos-item activated + const2-pos-array-lookup, const2-neg-noneligible, const2-neg-set added; pending-const2-item superseded). NOTE: const names are IDENTs (lowercase per FORM-3), not TYPEIDs — the old placeholder cases used `LIMIT`/`BAD` which are spec-wrong; corrected. Deferred: CONST-1 const-expr-as-array-size (needs array_new codegen + const-generic forwarding — separate from const items); const-array shared-borrow (&'r const / slice_of) — OWN-10 const clause exists but base64 needs only index. Unblocks the table-driven codec class; next: base64.

## Safe-direction pilot #2: base64 encode MEASURED (2026-07-10)

First const-array consumer. base64 encode via `const b64: array<u8,64>` alphabet, byte-identical to system base64 on all RFC 4648 vectors + 300/300 fuzz. Perf (384MB warm): xlang 0.23s vs BSD 0.20 (platform-tuned, 15% ahead) vs GNU 0.36 (1.6x) vs uutils-Rust 0.36 (1.6x). facts-neutral (single buffer). Honest framing: codec = obvious-shape-is-fast, so parity-at-C-speed + safety is the headline, not speed. Implemented to ship it: `&uniq buffer<u8>` params (lowered {ptr,i64}-by-value, element writes caller-visible via shared data ptr — the codec out-buffer idiom). Findings for pattern doctrine / advertising honesty: (1) ANF verbose for bit-twiddling (90 lines vs C's ~15); (2) whole-fn no-shadowing forces globally-unique locals across sibling blocks (suffix per arm). Next: base64 DECODE (input validation = the CVE-relevant, stronger-safety direction).

## D9 (2026-07-10): Confidence gate BEFORE compiler/feature investment

Owner fear (explicit): finishing the real compiler + a large project only to find xlang == Rust on performance. A pre-registered confidence gate now precedes further compiler/feature investment. Two legs pushed in parallel:
- LEG A (analyze existing Rust — FREQUENCY): the decision-relevant leg for the owner's specific fear (large project ties Rust). Proxies over real crates: alias-guard loop-versioning count + versioned-loop code-size (channel-1 sites), surviving non-inlined hot calls under default release build (channel-2 sites), custom/manual reassociation & hand-multi-accumulator idioms (channel-3 sites). Output: how OFTEN winning patterns occur => whether a large project wins at aggregate.
- LEG B (build with xl — CEILING): the decisive result is a REAL (non-micro) program where xlang-with-facts beats BEST-EFFORT SAFE Rust by a margin safe Rust cannot close without unchecked code. Honest current state: NO port so far shows this — binary-trees (facts 1.45x over own control, but same-shape Rust still faster), wc (algorithm win, not fact-channel), base64 (parity, facts-neutral). Microbenchmarks show channel deltas; no real program has yet. Channel 3 (checked-law reassociation) is the strongest "safe Rust structurally cannot do this" candidate; channel 1 (multi-buffer scoped alias) second.

PRE-REGISTERED BAR to justify building the real compiler as a performance play: >=1 real-program leg-B win over best-effort safe Rust that is fact-channel-attributable, AND leg-A frequency showing the pattern is not vanishingly rare. If neither clears after bounded effort: honest conclusion = xlang ~= Rust on raw speed; real differentiators are qualitative (safety-by-construction, reproducibility, AI-authorability, spec compactness) — changes the pitch and go/no-go (R0-fail options: verified-facts frontend / linted-Rust pivot), not a hidden failure.

BSD-gap answer (measured): base64 encode-only is 2.5 GB/s SCALAR (0 vector ops); ~152ms of the 230ms 384MB wall is encode. BSD ~200ms whole-run implies its encoder beats 152ms => BSD uses a wider precomputed table (12-bit->2-char, halving iterations), not better I/O. Our per-sextet scalar lookup is the naive shape; a wide-table variant is a blessed-pattern opportunity. The 15% is algorithm (table width), not language overhead.

democ tightening status (owner asked): of 4 totality levers, only (d) DIAGNOSTIC TIER shipped (`--totality`). Still unbuilt: (b) const-trip willreturn tier, (c) proof-elided bounds checks, (e) tier over-strictness (compute_total demands a fully `pure` row; memory-only rows are termination-irrelevant and a loop-free+trap-free fn with reads/writes/allocates should still qualify). Deferred until the confidence gate justifies more compiler investment.

## Totality tier RELAXED + three soundness holes closed (2026-07-10)

Lever (e) shipped on owner green-light: compute_total now requires loop-free + trap-free (row) + total callees — memory rows no longer block willreturn. Adversarial review (opus, per tiering directive) REFUTED the first version end-to-end and found: (1) CHECKER HOLE — _exhibits_traps had no `try` case, so an index place inside a try scrutinee never forced `traps` into the row (EFF-2 violation, pre-existing; weaponized by the relaxation into willreturn-on-aborting-fn, IR-verified). Fixed; conformance +2 (eff2-neg-try-hidden-trap / eff2-pos-try-declared-trap), 198 PASS. (2)+(3) democ _has_loop/_calls were blind to give-match arms (loop or non-total call hidden in a give-match earned willreturn even under the OLD pure-only tier — pre-existing unsoundness). Fixed; perf_regress pin #4 (totality_pins.xl must emit zero willreturn). Verified safe by the review: FN-4 rewrite ordering (only touches already-loopy fns), exhaustive-match unreachable blocks (dynamically dead), unknown callees (block by default). Lesson recorded: fact channels get adversarial review BEFORE ship, not after — this one caught a real IR-level unsoundness a green make check missed.

## Leg-B candidate: ordered chunk-summary wc REFUTED (2026-07-10)

Tested the proposed real channel-3 workload: represent full C-locale wc as map byte chunks -> ordered associative reduction over Summary(lines, words, bytes, first_space, last_space). Algebra HOLDS: external experimental artifact exhaustively checked identity/split equivalence and 39,651,821 associativity triples; xlang+Rust differential harness passed 1,284 arbitrary-split verdicts. But this does NOT clear D9. The merge is associative and NON-COMMUTATIVE, outside democ's current scalar assoc+comm consumer; consuming it would require an order-preserving tree rewrite + parallel runtime. Ceiling test used external pthread/scoped-thread orchestration: on the 421.2MB corpus, median 1t/8t kernel times were xlang 241.9/39.2ms, xlang-no-facts 220.4/41.2ms, C 133.6/24.8ms, safe Rust 135.0/24.1ms. xlang summarize assembly is identical facts/no-facts, so there is zero channel attribution; safe Rust expresses the same shape and is 1.6-1.8x faster. Code-shape finding: xlang loop vectorizes width 2 x interleave 4 while equivalent C/Rust use width 16; hoisting first_space out improved xlang ~311->~220ms but did not close. VERDICT: preserve experiments/port-study/wc-chunk-summary as negative evidence; do NOT build associative-only law discharge/threading for this candidate. Investigate the width-2 lowering independently only if a later confidence-gate target needs this classifier shape.

## OWN-1 AMENDED: tag-only enums are copy; chunk-wc reaches C/Rust parity (2026-07-10)

Owner-directed fix for the width-2 lowering deficiency, executed as spec delta + both checker layers + democ, with derivation recorded in spec/derivation-ledger.md (affinity of resource-free values was minimality-selected R3, bought zero safety, taxed every boolean; evidence = the measured 1.6-1.8x classifier gap). Spec: OWN-1 copy class += tag-only enums; move-of-copy now a FORM-1-style hard error (one spelling: copies are used bare). Checker: type layer _is_copy consults enum table + rejects move-of-copy; return/try/own-arg auto-move conversions skip copies; flow layer Binding.ty + copy_enums guard the match-scrutinee consume. democ: i1 let-slots (mutable Bool), band/bor/bxor/bnot lowered in i1 (were entirely missing), b-ops added to dotless table. Conformance: 4 negative cases migrated (their affine specimen Sign() became legal — payload added to preserve intent), +2 amendment cases, 200 PASS. RESULT: chunk_wc scan rewritten in i1 dataflow vectorizes at width 16 (29 x16b ops, was 0) -> 134ms == C 132.6 == Rust 133.9 at 1 thread (was 220-242ms); parity at all thread counts; facts/nofacts asm byte-identical. The D9 verdict on the workload is UNCHANGED (no fact attribution — Rust expresses the same algorithm); the codegen debt is retired. wc port updated for move-of-copy and re-fuzzed 45/45.

## Owner audit: four nearby cases fixed / recorded (2026-07-10)

(1) wc count_all rewritten in i1 dataflow: 0.28 -> 0.23s warm full counts (2.1x GNU; owner's 0.17-0.18 was kernel-only, slurp driver costs ~0.04s), fuzz 45/45 green. (2) 2-VARIANT TAG-ONLY USER ENUMS now lower i1 end-to-end (module registry _TAGONLY2 threaded through _llty/_field_ll/_tybytes/_size_align/construct/gen_match): ScanState probe = width-16, 130-132ms = parity with Bool/C/Rust (owner measured 34% penalty as i32; eliminated). 3+-variant tag-only enums stay i32 (i8 possible later; recorded, unmeasured need). (3) buffer<Bool> fixed twice over: _tybytes 4->1 byte/elem AND ttype elem was hard-coded prim (buffer<Bool> never typechecked; now structural) + TYPE-2 parenthetical updated to match the OWN-1 amendment. Conformance +2 (202 PASS). (4) PROOF-ELISION EVIDENCE recorded, tier still carded not built: provably-in-range iadd.trap reductions stay scalar (buffer_index_kernel 0..1023 sum); base64 hot loop retains ~18 bounds branches with LLVM reporting early-exits blocking vectorization — the controlled bounds-elision experiment (elide exactly the checks a checker proof covers, measure, verify) is the designed next step BEFORE claiming any gain (table lookups may still bound SIMD). Doctrine note per owner ("allowing bad code to exist violates the rules"): case 2 was a compiler-side violation (equivalent shapes, unequal speed) — fixed; case 1 was writer-side (the doctrine answer is teach the i1 dataflow pattern + the totality lint); case 4 is the language-side answer (writers should KEEP .trap and the compiler earns the speed via proof — pushing writers to .wrap is the wrong fix).

## PAUSE: history rewrite + consolidation (2026-07-10, owner-approved)

Repo history REWRITTEN (git-filter-repo + stale codex checkpoint-ref removal): .git 380MB -> 3.8MB; stripped the two benchmark corpora (wc big.txt 421MB, base64 big.bin 384MB — both regenerable per RESULTS), a committed cargo target/ tree, playwright snapshots, compiled binaries and fuzz dirs. Research journals and the FR paper kept. CONSEQUENCE: all commit hashes cited in gates entries BEFORE this line are pre-rewrite labels — historical text, no longer resolvable; post-rewrite history is authoritative from here. Force-push to origin (github.com/mbbill/xlang) left to the owner. Hygiene: stale checker/democ headers rewritten to current subset; experiments/README.md index added. CONSOLIDATION: THE-PLAN.md created — single current-state map (beliefs, rulings digest, honest evidence ledger incl. non-wins, ranked bets: proof-elision design > leg-A frequency study > channel-4 card > coreutils ladder, pre-registered pivot clause, standing process rules). PATTERNS.md +P7 (branchless i1 classifier) +P8 (traps to the boundary) — closes the owner-audit case-1 residue as doctrine.

## Bet 1 ceiling MEASURED + OP-4 proof-tier design card (2026-07-10)

Experiment-only democ flag --elide-bounds-experiment (ceiling probe, default off, make check untouched): base64 encode 2.44 -> 4.2 GB/s (1.7x), branches 41 -> 9, byte-identical outputs; still zero SIMD (shuffle algorithm not vectorizer-discoverable — the audit's caution confirmed; elision value is scalar). CEILING JUSTIFIES THE TIER. DESIGN CARD (build next): (PROOF-1) structural dominating-guard prover — the checker already verifies the canonical loop idioms (P2/P7 shapes); recognize `guard cmp(i, n) dominates index(b, i)` with n bound to len(b), plus offset algebra under `rem >= k` guards; elided checks emit no trap branch. (PROOF-2, then awaiting owner ratification) a checked concrete-function entry clause computes and checks the capacity relation once at the callee boundary, so every invocation is covered even when no xlang call site exists; its passed fact licenses elision of callee-body capacity checks. Static caller discharge is not required for acceptance and is not part of the first slice. (PROOF-3 correction under the normative EFF-2 rule) proof-elided checks remain in the syntactic exhibit set, so effect rows do not tighten and derived `willreturn` does not return through this route; this conservatism keeps acceptance and signatures optimization-stable. HONEST ADVERSARY pre-registered for the eventual benchmark: Rust's assert-up-front idiom (a dominating `assert!` lets LLVM elide some later checks) and unsafe `get_unchecked` (the escape we refuse); the claim to test is parity with the one-boundary-assert shape while retaining xlang's deterministic proof report, not a caller-site advantage. Discipline note: writers keep `.trap` everywhere; elision is EARNED by proof, never granted by mode-switching (P8).

## PROOF-1 shipped + base64 local fraction measured (2026-07-10)

Structural bounds discharge now has one codegen source of truth (`index.proof`)
and a byte-transparent per-site report at the exact `index_addr` lowering
boundary. The parity runner gates exact eligible/proved/retained counts per
function; C/Rust report N/A; ceiling elision remains distinct from proof.
Corpus: 24 prior guard/mask cases plus 26 derived-range cases = 50 total,
covering direct guards, fixed-stride remainder induction/tails, masked const
indexes through unsigned widening, exact mixed-site classification, and
wraparound/direct-or-aliased mutation/lexical-scope/wrong-buffer/wrong-bound adversaries. The key soundness
premise for `rem=len-i` is explicit: i starts at zero and its sole mutation is
the exact stride increment; the guard alone is UNSOUND under unsigned wrap.

REAL-WORKLOAD RESULT: base64 `encode` has 27 lowered sites. PROOF-1 proves 15
(6 source + 9 alphabet) and retains 12, all output-capacity writes. On the
384MB encode-only M4 harness (five-sample medians), no-facts 2.50 GB/s / 153.9
ms, local facts 2.93 GB/s / 131.2 ms, perfect ceiling 4.23 GB/s / 90.9 ms.
Thus local proof is 1.17x and recovers ~36% of removable time; 139/139
boundary-biased facts/nofacts and system-base64 differentials pass. The 9
alphabet checks were already optimized away; the six source checks cause the
measured gain. PROOF-2 is now sharply scoped: check once at the callee entry that
`len(out) >= 4*ceil(len(src)/3)` and connect the passed fact to i=3k/o=4k. No spec surface
was invented at that step; owner ratification still remained required then.

PRE-COMMIT ADVERSARIAL AUDIT: three initially accepted holes were found and
fixed before shipping: writes through uniq-borrow holders now invalidate proof
dependencies; masked facts use lexical environments and cannot leak between
sibling bindings that reuse a spelling; and bare affine call arguments no
longer become type-layer-only implicit moves. Exact negative gates cover all
three, with dedicated OWN-1 positive/negative conformance cases for calls.

## PROOF-2 checked `requires` first slice selected (2026-07-11)

Owner approved proceeding with the checked form. The selected semantic unit is
an optional concrete-function prologue after the effect row:
`requires { let_stmt* check_stmt }`. It executes at callee entry on every
invocation; failure traps, success contributes one dominated fact, and ordinary
call acceptance never depends on a caller proof. The clause is not `assume`, is
not a trusted-ledger member, and its explicit check has OP-5 semantics. The proof
tier may remove only downstream implicit checks justified by that fact. The
first slice is deliberately absent from contract `fn_sig`: inventing
refinement/subtyping or partial-law semantics before the FN-3/FN-4 evidence gate
would violate R3/OWN-8's reject-when-unsure direction.

Constitution chain: P0/R0 select the feature class because the measured base64
ceiling is 1.7x, PROOF-1 leaves exactly 12 output-capacity checks, and its local
subset already measures 1.17x. W3 fixes checked-not-trusted semantics; OP-5
supplies the existing stated-and-checked channel. W3/T1 plus the observed
direct-C entry path select callee-boundary coverage rather than reliance on a
set of known xlang callers; R4 then ranks its runtime trap above the forbidden
silent-corruption fallback when no static proof exists. T2 and OWN-8 justify
the closed pure/total operation-table ANF sublanguage. EFF-2 remains conservative: both the retained boundary check and
any proof-elided downstream source check continue to exhibit `traps`, so rows
and totality attributes do not change after optimization. R3 makes only the
surface spelling provisional.

META-5 delta: +1 normative rule (FN-8; 89 -> 90), +1 grammar production
(`requires_block`), +1 optional occurrence on `fn_decl`, +0 changes to
`fn_sig`, +1 reserved surface word/spelling (`requires`), +0 exceptions, and
+1 R3-provisional-register item; +524 `cl100k_base` tokens in the normative
spec (14,692 -> 15,216, measured with tiktoken 0.12.0). Selection ground is split explicitly:
feature existence, callee-entry execution, always-retained check, and
concrete-only scope are evidence/semantics-selected; the block spelling is
minimality-selected because it reuses the existing ANF `let` and OP-5 `check`
nodes. Completing experiment: compare first-parse/repair behavior and emitted
code for this spelling against any credible single-predicate alternative; the
semantics and W3 floor are not candidates in that A/B.

Prototype boundary: democ emits and backend-pins the checked entry branch, and
its external structured report gates each eliminated index. Gated FFI frames,
DIAG-2 proof references embedded in a canonical artifact, and DIAG-3
machine-readable trap reports are still unimplemented toolchain layers; the
direct-C probe is evidence that entry enforcement is necessary, not a claim
that the gated FFI/reporting stack has shipped.

## PROOF-2 implemented, adversarially gated, and measured (2026-07-11)

The prototype now parses, checks, scopes, and lowers FN-8 at callee entry. The
base64 recognizer accepts exactly the checked non-wrapping relation
`N <= 3*floor(C/4)`, then separately verifies the zero-based `i+=3/o+=4`
induction and mutually exclusive tail-1/tail-2 groups before marking an output
site `output-capacity-lockstep`. It proves all 27 base64 sites (6 source, 9
alphabet, 12 output); facts-off retains all 27 while running the identical
entry check. The optimized facts and perfect-index-elision variants both have
77 instructions and one retained trap path.

The new family gates 28 cases and 42 sites: 5 fully proved positives, 2 exact
mixed classifications, and 21 retained near-misses covering relation/target,
zero bases, 3:4 strides, increment order, offset four, conditional drift,
tail shape/mutual exclusion, direct and aliased mutation, output-buffer
reborrowing, sibling scope, and interprocedural fail-closed behavior. Combined
PROOF-1/2 corpus: 78 cases. A pre-commit audit found and closed: user-call alias
invalidation, shared-for-uniq mode substitution, affine uniq-borrow copying,
whole-buffer-replacement acceptance without storage semantics, backend removal
of explicit tautological checks, incomplete FN-3 signature matching, and an
FN-8 whitelist broader than implemented lowerings.

M4 five-sample median: same-source facts-off 2.480 GB/s / 154.9 ms, PROOF-2
4.233 GB/s / 90.7 ms, perfect index-elision ceiling 4.215 GB/s / 91.1 ms.
Thus the checked proof path is 1.71x and reaches the ceiling within noise.
Correctness: 139/139 deterministic boundary-biased facts/nofacts/Python
reference differentials; an exact-capacity direct-C call succeeds and a
one-byte-under call traps at entry. All six verification layers are green,
including the 10,000-program independent model check (zero accepted
violations) and 223 runnable conformance cases.

## PROOF-2 adversarially reviewed: SOUND with 1 hole + 3 hardenings, all fixed (2026-07-11)

3-lens hostile panel (induction-math on top tier; validation-bypass + elision-overreach on opus; ~45 attacks, all probes actually compiled and run) on FN-8 requires + output-capacity-lockstep (c0b3ef8). CORE VERDICT SOUND: every induction side-condition is pinned in code and was verified line-by-line (relation shape incl. wrap-impossibility of 3*(C>>2); i=o=0 base; single increment sites after all stores; guard literal pinned to stride; isub wrap-safety via i<=n induction; offsets pinned to o..o+3 with budget math; tail arms tied to ieq(tail,1)/ieq(tail,2) with independent pad-group arithmetic; binding stability incl. src!=out and param modes). ~30 exploit mutants all correctly flip to retained/rejected. Independently: capacity relation re-derived tight (12 passes / 11 traps at N=9); RFC-identical outputs on all tail shapes; C-caller undersized-buffer traps at boundary pre-write. FIXED FROM PANEL: (HOLE) doc-only requires clause vanished before validation (docs filtered -> empty list -> treated as no-requires); parser now defaults requires=None, checker rejects present-but-empty clauses [FN-8]; conformance +1 (fn8-neg-doc-only-clause). (WG-1) offset-former now pins tyargs=[u64] + literal ty (narrow-wrap laundering defense; unexploitable today, fails closed anyway). (WG-2) _linear_statements blacklist -> WHITELIST (doc/let/set/check/expr) so unknown future statement kinds fail closed. (Corpus gaps) +3 gate cases n23/n24/n25 (guard-literal low/high, requires shift amount) — the two pinned literals the corpus never mutated. Six layers green, 224 conformance, corpus 81 cases 0 failures 0 debts.

## base64 ladder FLIPPED by the proof tier (2026-07-11)

HISTORICAL PRE-ADVERSARY SNAPSHOT. Re-measured post PROOF-1/2 (384MB, byte-identical): xlang-with-proofs 0.16s now FASTEST — BSD 0.21 (was ahead of us at 0.20 vs our 0.23), GNU 0.36, uutils-Rust 0.36. Kernel 4.09 GB/s vs 2.46 control = 1.66x from proofs alone, 97% of the elision ceiling, at full checked semantics. This established a D9-relevant FACT-ATTRIBUTABLE real-program delta for the obvious indexed shape: facts on/off isolates the proof tier, and the compared uutils/indexed Rust shapes retained checks. It did **not** establish that expert safe Rust could not restructure the checks away. At this ledger point the assert-up-front adversary was still owed; the two entries below discharge it and supersede any broader Rust comparison.

## requires/check-accounting design REVIEWED (2026-07-11)

Three-lens panel (repo-reality, governance red-team, complexity audit — opus tier) + synthesis on requires-check-accounting-design.md; full review in requires-check-accounting-REVIEW.md. VERDICT: accept the analysis, reject the first-slice scope. Facts verified accurate (only corpus-count staleness). FOUR BLOCKING DEFAULT-FLIPS: (B1) approval identity per-site dependency-cone, never whole-artifact (whole-artifact => per-commit invalidation => rubber-stamping approver => laundered debt with false provenance — worse than no gate); (B2) retained-check debt narrowed to OBLIGATION-BACKED debt only (analyzer-derived obligation unmet => fail; no-obligation-derivable => safety floor, passes; indeterminate/credit-without-proof => fail — reject-when-unsure applied to accounting); (B3) explicit-unused NEVER a hard failure (would incentivize deleting defensive checks — the one forbidden incentive); (B4) versioned not auto-accounted and dropped from slice 1 (cheapest-route-to-credit + W1 cliffs + measured 17x code-size precedent). FIRST SLICE: §5 obligation analyzer + §11 diagnostics wired into the EXISTING parity gate; approvals as GATE-1 instances; §9/10 writer doctrine to PATTERNS.md now. Everything else deferred with named evidence triggers. Q13 answered: QOI decode is the decoder experiment. Q17: constant-time out of scope entirely.

## Owed adversary run: assert-up-front REFUTED; expert iterators tie; D9 decider moves to QOI (2026-07-11; numeric comparison superseded below)

The initial fixed-order base64 run correctly found that Rust's up-front assert recovers zero inner bounds checks and that expert `chunks_exact/zip` restructuring reaches the fastest measured check-free scalar performance class. Its cross-harness ranking was not controlled, however: it compared one Rust five-pass aggregate with an xlang reference from a different run, in different fixed-order thermal contexts, and the Rust chunks function silently truncated short output and discarded tails. The 384MB timed input was divisible by three with ample output, so the API mismatch was latent rather than the cause of the timing correction; the unsupported cross-harness ranking was the defect. The apparent ~5% Rust lead and its attribution to loop-shape quality are **withdrawn**. The structural rulings survive: base64 does not clear D9's strict bar; the durable claim is W1 distributional (xlang obvious shape + one checked relation reaches expert-safe performance, while Rust obvious shape has no local annotation fix) plus boundary trap protection; QOI decode remains the leg-B decider; the leg-A frequency study remains the other gate leg.

## Controlled base64 adversary correction: practical parity, no 5% residual (2026-07-11)

Rebuilt xlang PROOF-2 and four full-tail Rust candidates into one executable; the primary safe chunks candidate plus assert/unsafe use the same checked entry-capacity relation, while naive deliberately remains the ordinary-bounds-check control. The harness requires 27 proved / 0 retained sites, verifies exact-capacity output for every length 0..257, and retains the existing 139-case differential/foreign-boundary gate. Evidence protocol: 384MB, native M4 codegen, 30 fresh-process Williams blocks across three cycles, shared buffers/clock within each block, every ordinal position and ordered first-order pair balanced six times inside isolated blocks, deterministic block-order shuffle, all samples retained, descriptive 10,000-resample process-block bootstrap interval, and a predeclared plus-or-minus-2% practical-equivalence band. Medians: xlang-obvious+requires 4.285 GB/s; Rust naive 2.673; Rust assert-up-front 2.677; full-semantics safe Rust chunks 4.297; unsafe indexed Rust 4.111. Primary XL/Rust-chunks throughput ratio 0.997, descriptive interval 0.994..0.999; cycle medians 0.997/0.994/0.999 and XL-first/Rust-first medians 0.997/0.995. This is **practical parity under the 2% rule**, not exact equality and not a meaningful xlang codegen deficit. Assert-up-front remains refuted dynamically and in assembly; expert safe Rust still structurally removes the checks, so D9/QOI and W1 rulings are unchanged. Canonical evidence: `experiments/port-study/base64/RESULTS.md`; raw CSV SHA-256 `ebea523dda82e7e7d3156da1dbb982a58fa5672a1c2bf751dbfd5a488fca7a20`; metadata sidecar SHA-256 `1b83368d7127f96304c97bd65d29aa16406f0ea28387ca2c4a2fa1c651316f84`.

## Obligation-driven PROOF-2 first slice implemented (2026-07-11)

The reviewed first slice is now concrete. PROOF-2's dependency is reversed:
the compiler derives the closed supported 3:4 lockstep obligation and candidate
sites from the body, independently normalizes the checked FN-8 requirement,
then compares the two. Only the same exact body + exact requirement pair
accepted by the old recognizer may apply the existing
`output-capacity-lockstep` marker. The external site report now carries
obligation status/exactness, requirement relation, canonical first missing
fact, and deterministic first failed premise. Facts-off runs the same analysis
but applies no marker, so diagnostics—not only site enumeration—are a control
oracle.

All 37 output-capacity cases now pin those states and reasons in the existing
parity gate, including exact mixed-site distributions and exact missing-fact
objects. The real base64 gate separately pins 12 sufficient/equivalent output
obligations plus 15 unrelated source/table sites. Pre/post compiler comparison
over all 88 corpus programs in both facts modes found 176/176 identical
acceptance results and byte-identical LLVM IR. Focused PROOF-2, all-bounds,
base64's 139-case differential/foreign-boundary verifier, the facts-independent
regression pin, and `make check` are green (224 conformance cases; 90/90 rules;
all six layers). P9 in PATTERNS.md now teaches the reviewed doctrine: a
`requires` predicate is an actual invalid-call boundary, not a common case or
worst-case allocator hint; expected shortage is a recoverable value.

Scope remains deliberately narrow: no warning has become a language error, no
guard versioning or counterexample search was added, and no approval/promotion
authority is implemented. The next major implementation is the obligation-
backed checked-automation promotion gate from review B2, using the existing
parity artifact rather than a parallel policy system.

## Obligation-backed checked-automation bounds-v1 build subgate implemented (2026-07-11)

The existing parity harness now carries review B2's bounds-v1 build policy without
changing source acceptance. Every implicit bounds site starts with incomplete
obligation analysis. The compiler iterates a schema-versioned registry containing
`output-capacity-lockstep-v1`, records the exact successful analyzer set, and
only then permits an affirmative `not-applicable` result. Valid frontend proofs
are `automatically-accounted`; retained sites explicitly outside the analyzer
families are the passing `intrinsic-dynamic` safety floor; derived obligations
with missing/mismatched facts are `hard-finding`; failed premises, unknown or
incomplete provenance, matched-but-retained sites, ceiling mode, and unverified
backend credit are `unaccounted`. Hard and unaccounted findings fail the build
subgate while leaving every runtime check intact. Malformed reports/compiler
errors are a distinct exit-2 class, not policy findings.

Base64/facts is the first root. Dual repository pins duplicate case, variant,
function, source, source SHA-256, and `closed-unit` scope in the manifest and
harness review set. `--promotion` requires the full corpus plus default
manifest, rejects all case/tag/mode filters, and verifies that every pin ran.
Accounting uses the unfiltered whole compilation-unit report, so same-unit
helper extraction cannot hide debt. Base64 passes with 27 automatically
accounted sites and zero findings. Ordinary corpus `maturity: gate` remains
diagnostic maturity, never build-root authority; 15 capacity cases pin policy
pass/fail distributions in facts mode. Protected external owner review is still required for coordinated
changes to both pins; the repository mechanism alone is not GATE-1 authority.

Adversarial review found a class of laundering and coverage escapes before
landing. `n27` perturbs remainder syntax; n28–n31 move output state through a
cursor field, aggregate path, owned buffer, or helper-return alias; n32 moves an
owned alias; and n33 exposed a real indexed-borrow lowering bug that previously
ignored the index. The compiler now covers every nonliteral indexed write plus
accesses through current unique-reference roots, lowers indexed borrows through
the checked element-address path, and fails unrecognized candidates closed.
FN-4 cloning had also copied source obligation metadata into generated indexes;
generated origins now start incomplete. A final-AST invariant requires every
enumerated index origin to emit exactly one report record, with matching total
cardinality. The callable analyzer registry supplies the recorded completion
set. These defenses close the enumerated current-AST attacks, not arbitrary
user control/`Result` rewrites, imported calls, or fixed-literal forms.

Evidence: 21 policy/authority/provenance unit tests; 44 PROOF-2 diagnostic cases;
94 total bounds cases; 190/190 byte-identical report/no-report compiler runs
over all 95 corpus `.xl` files in both facts modes; 15 review-pinned policy
oracles; base64 27/27; approvals hard-fixed to `[]`; facts-off and ceiling roots
rejected; invalid/duplicate/unknown analyzer states rejected. `make parity` is
the dedicated authenticated bounds-v1 subgate run.
The slice remains authorization-free: review B1 per-site dependency-cone
identity must exist before any retained-site approval, and repository
permissions/review must protect coordinated changes to the separately pinned
root set; that external control is required before claiming GATE-1. The saved
JSON separately records validated invocation, pass/fail verdict, policy, roots,
and oracle digest. B3 explicit checks, B4 versioning, domain records, backend
provenance, imported/transitive closure, overflow/allocation/FFI accounting, and
counterexamples remain deferred. There is also no automated known-hot-outside-
scope tripwire or approval-governance telemetry; root reconciliation is manual
and approvals remain absent. The next experiment remains caller-owned streaming
QOI decode.
