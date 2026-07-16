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

## Leg-A throwaway pilot complete; bounds proof remains the bet (2026-07-12)

The owner explicitly chose a quick directional estimate over a publication-
grade frequency platform. The completed pilot scanned 30 popular source crates
and 12 download-ranked command-line crates. Raw source signals were common but
overinclusive: manual review accepted no current xlang scoped-alias or
checked-law opportunity, and the strict reassociation miner found zero
qualifying sites.

The useful positive result came from optimized IR. Of three selected builds,
`comrak` retained 101 first-party bounds-panic edges across 40 functions and
`inferno` retained 15 across two first-party functions; `crc64fast-nvme` was a
zero control. A sampled `comrak` workload put `Parser::process_line`,
`Parser::open_new_blocks`, and `html::escape` above 2% self CPU, all with
surviving checks. Audit found a few current-PROOF-1-shaped cases and a larger
family needing generalized relational or callee-postcondition proofs.

One byte-identical `html::escape` intervention removed a logically redundant
check but was neutral over 35 alternating measurements (172.971 ms checked vs
173.909 ms unchecked median). Thus the pilot is **not** a whole-program win and
does not clear D9. It is a provisional go to stop frequency analysis and test a
cluster of relational proofs in a real workload; alias/law frequency is not the
next priority. Full caveats and raw directional counts are in
`experiments/frequency-study/RESULTS.md`.

## Repo restructured for agent onboarding (2026-07-12, owner-directed)

Clean-base pass: (1) CLAUDE.md created at root — auto-loaded agent onboarding: read order (THE-PLAN -> gates tail), both verification gates, standing rules (durability protocol, adversarial-review-before-ship, proof-not-weakening, tiering, plain-language reporting), layout map, current focus. (2) README.md rewritten as a router table. (3) archive/ created: DECISION_SPRINT.md + ROADMAP.md (superseded by THE-PLAN), research era (debates/sources/matrices/synthesis), corpus-era experiments (codegen-vs-rust-c, regions-effects-vs-safe-rust, ai-native-parallelism), and m3 (shelf-ready per D5); archive/README.md explains each. Verified zero live tool/gate references into archived trees before moving; active-doc references fixed; historical log text left untouched (pre-archive paths are labels, precedent per the history rewrite). (4) THE-PLAN status: build track ACTIVE — compiler/ hosts xlc, the production compiler written in xlang (SoA-tape per P2, democ as stage 0, self-parse gate); bet-1 adversary resolved (assert-refuted / expert-iterators-tie / QOI is the leg-B decider). Both gates green post-move.

## Restructure errata (2026-07-12)

The parity gate broke silently during the archive pass (pipeline exit-status masking — tail swallowed make's failure; the clean-base commit landed red). Root cause: codegen-vs-rust-c is a LIVE fixture (codegen-parity.json pins the splitmix xlang/C/Rust triple by path — my pre-move grep missed JSON at repo root). Restored to experiments/ with a DO-NOT-ARCHIVE note in the index; both gates re-verified green. Process lesson recorded: never chain commit after a piped gate — verify with explicit exit codes; and grep ALL file types (json manifests included) before moving anything a gate might pin.

## Design tree constructed (2026-07-12)

mcts_mem/ now holds the MCTS-Mem design tree: 28 nodes (17 live, 11 alt) recovering the project's re-decisions and weighed alternatives from this log, user-directives, the spec revision history, the derivation record, the FR memo, experiment RESULTS, and the review docs; lint clean; every Moves hash resolves in post-rewrite history. Built by a dedicated construction agent, then live-context reviewed by the coordinating session (21 of the 26 files present at review time read; recorded whys checked against session memory of the OWN-1/totality/PROOF-2/requires decisions; no corrections needed). Coverage rulings of record: D5/D7/D9 are process-not-design and enter as root facts, not nodes; the v0.1 soundness inversions (OWN-4 direction, arena escape) are pitfalls, not weighed alternatives; GRAM-4 bind-then-match and the GRAM-7 helper-fn idiom are real superseded forms with .alt homes; pre-repo decisions (D1a checker class, mut->uniq) anchor to the initial import commit because pre-rewrite hashes are unresolvable. Two construction errata: the interim "added mcts_mem" commit was an accidental partial snapshot from a blocked session (truncated pattern-doctrine node; completed in the follow-up commit), and its push made one surface-form fact committed before a planned dedup — append-only wins, the fact stays. Maintenance from here follows the mcts-mem discipline: a re-decision moves the superseded form into .alt/ with paired verbatim whys in the same change as the code.

## D9a: primary gate is the default AI-written xlang floor vs shipped Rust (2026-07-12, owner re-ruling)

The owner revised the comparison target after the expert-safe-Rust base64 run.
Expert Rust is a constructed ceiling: Rust's expressiveness lets a specialist
restructure many kernels until they reach the same machine-code class. That is
valuable ceiling evidence, but it does not measure the product claim. Existing
widely used libraries also make ordinary readability and maintenance
trade-offs, while xlang is intended to make a good shape the default output of
an AI writer. The primary score is therefore one fixed low-tier model's first
correctness-green xlang artifact against one pre-registered, unmodified shipped
Rust library at an exact version, default features, and ordinary release build.

The protocol freezes the API, corpus, correctness gate, repair budget,
measurement order, and success band before generation. The model may see only
the sanitized xlang teaching pack, task contract, compiler/checker diagnostics,
and failing correctness cases. It may not see the Rust source, prior xlang
base64 source, performance results, profiler data, IR/assembly, performance
hints, or human source edits; there is one trajectory and no fastest-of-N
selection. The first green source, complete trace, and hash are frozen before
measurement. The same source is then built facts-on and facts-off for
attribution. Stronger xlang models and expert Rust are allowed only after that
freeze as gradient/ceiling evidence.

The owner caught a target-selection leak before the protocol was frozen:
base64 has already been used to develop and tune xlang's proof tier and has a
complete handwritten/expert comparison in this repository. A fresh model run
there could smoke-test the harness, but cannot be the new primary evidence.
Target selection therefore excludes workloads previously used to tune xlang,
as well as near-duplicates that exercise the same fixed-ratio proof family.
The first scoring target remains to be pre-registered after a fresh-target
audit. Any broad claim still requires a second independently pre-registered
shipped-library replication.

This is a narrow override of D5: the broad tier-distribution sprint stays
deprioritized, but one exact Luna/Terra-class low-tier run is mandatory. The
leg-A frequency pilot is contextual evidence rather than the primary score.
This supersedes D9's best-effort-safe-Rust adversary as the primary scoring
rule; it does not retract the base64 expert-parity measurement, which remains
valid ceiling evidence.

## D9a target lock: percent-decode versus GPT-5.6-Terra (2026-07-12, owner-selected)

The owner selected the fresh primary workload from a deliberately varied
shortlist. Lock the shipped comparator to crates.io `percent-encoding` 2.3.2,
exact requirement `=2.3.2`, default features (`std` -> `alloc`), registry
checksum `9b4f627cb1b25917193a259e49bdad08f671f8d9708acfd5fe0a8c1455d87220`,
and the public `percent_decode(&[u8]) -> Iterator<Item = u8>` API. The source
anchor is rust-url commit `91377f48bf35011d042aa5abef9e7f2a0a625aaa`, but the
published artifact records a dirty VCS tree, so the Cargo lock plus registry
checksum is authoritative. The frozen Rust adapter may only consume that public
iterator sequentially into the same caller-owned output shape; it may not
reimplement or specialize the decoder.

Lock the writer to model slug `gpt-5.6-terra`, Codex CLI 0.144.0, reasoning
effort `medium`, normal service tier, one trajectory, and at most three
machine-feedback repair turns. A task-independent read-only `/tmp` probe
confirmed the exact slug and settings are available; it did not expose the
experiment task or repository. No target prompt is sent until the task,
teaching pack, correctness corpus, benchmark order, and score band are committed
to the experiment protocol. This workload is variable-output, malformed-input
observable, fully expressible in the current democ subset, and absent from prior
xlang experiments; no target-driven compiler change is allowed before freeze.

## D9a pre-freeze launcher incident: no candidate; identity retired (2026-07-12)

While the percent-decode protocol was still uncommitted and under hostile
audit, a reviewer invoked the preregistration launcher with `--help`. The draft
launcher incorrectly ignored argv and attempted its then-fixed run path. Codex
adapter exit 70 produced zero model-output bytes and an empty candidate; the
evaluator never ran and no source froze. We inspected only process/result
metadata and artifact sizes/hashes, not model stderr or candidate contents.

The retired identity `runs/primary-terra-medium` will never be reused; the
hash-only incident record is committed under the target experiment, and its
opaque draft artifacts were removed. The official identity is now
`runs/primary-terra-medium-preregistered`. The launcher rejects every argument,
requires all model/correctness/benchmark inputs clean and tracked at `HEAD`, and
locks tool binaries/versions before sending the corrected, newly hashed prompt.
Therefore the incident is a pre-freeze harness failure, not a sampled candidate
or a restart of the official one-trajectory score.

This incident explicitly supersedes the earlier forward-looking statement in
the target-lock entry that no target prompt would be sent before commit: the
draft invocation did send its older prompt, but returned zero model-output
bytes and created no candidate or evaluator observation.  The corrected prompt
and official run identity remain unsent until their preregistration commit.

## D9a percent-decode preregistration frozen (2026-07-12, before generation)

The complete default-floor protocol is now frozen for one
`gpt-5.6-terra` trajectory with reasoning effort `medium` and exact Codex
`service_tier="default"`; this exact value clarifies the earlier phrase “normal
service tier.”  The target is `percent-encoding` 2.3.2 public
`percent_decode`, consumed by the minimal caller-owned-output adapter.  The
6,764-byte base prompt is SHA-256
`554050441f265d5c290756209a4a911f91004b1d55d73e6dd6a42bdc40d00dc7`.
The first correctness-green source in the single initial-plus-three-repair
trajectory freezes immediately, with no performance feedback or human source
edit.  The official identity remains
`runs/primary-terra-medium-preregistered`; it does not yet exist at this point.

Freeze audit: generic generation tests 12/12; Rust baseline tests 6/6; harness
tests 7/7; strict C boundary syntax; malformed-source repair classification;
full tool/SDK, Cargo-config, crate/archive/tree, input-hash, run-tree archive,
and invalid-rerun validation; benchmark smoke marked `not_a_score`; `make check`
and `make -C compiler check` both green.  No official model output or scoring
measurement was produced during these checks.

## D9a Terra default-floor source frozen (2026-07-12, before measurement)

The sole preregistered `gpt-5.6-terra`/medium/default-service trajectory ran
under identity `runs/primary-terra-medium-preregistered`.  Round 0 failed to
compile on a missing statement terminator.  Round 1 compiled but failed the
stable differential case `%0A` (`0a` expected, `00` returned).  Round 2 was the
first correctness-green candidate and froze immediately; no fourth model call
was made.  Frozen source SHA-256:
`b67dd2912ba907d64e38fc1044f52a305824b3d9141043a901027b64b94e00bd`.

The frozen manifest validates the exact three-round trace and first-green
identity.  An independent post-freeze evaluator invocation again reports
compile green and `correct cases=153014`.  At this entry no proof report, IR,
assembly, or performance result has been used to alter the source, and the
preregistered scoring campaign has not yet run.

## D9a percent-decode primary score: default xlang wins, facts neutral (2026-07-12)

The one preregistered campaign completed without rerun: 30 fresh processes,
all 90 samples retained, 256 MiB fixed corpus, AC power identity stable, and no
recorded power/thermal transition (the thermal probe was consistently
unavailable).  Median throughput was 629.484 MiB/s xlang facts-on, 620.343
MiB/s facts-off, and 383.456 MiB/s shipped Rust.  The paired facts-on/Rust
median ratio is 1.6533 with stratified bootstrap interval [1.6309, 1.6668]: a
predeclared **meaningful xlang win**.  Facts-on/facts-off is 1.0100 with interval
[1.0047, 1.0196], inside the ±2% **practical parity** band.

Attribution matters: facts-on and facts-off proof reports are byte-identical,
with all six bounds sites retained and none proved.  This is therefore not a
proof-elision win.  It is the intended default-floor result: the first green
low-tier-AI xlang shape beat the ordinary released `percent-encoding` 2.3.2
public-iterator path.  Post-result assembly shows the Rust adapter making an
out-of-line `PercentDecode::next` call per yielded byte while the xlang helpers
fold into one call-free loop.  Expert Rust could plausibly close that gap with
inlining/LTO/restructuring, but those are ceiling experiments and cannot alter
this primary score.  Canonical evidence and caveats are in
`experiments/default-floor/percent-decode/RESULTS.md`; raw SHA-256
`a659855f0d3fa909874f861f6d7ec7401653a877a7c0bd0d1544d4333054324b`,
analysis SHA-256
`24cfe1e9a3aed9d647d7ae9f684f178fe62c363bff1afa405a1dc245376bd5b9`.

D9a is satisfied for this target, not generalized.  The next evidence step is
one independently preregistered shipped-library replication; an optional
post-primary Rust inlining ceiling may explain mechanism but is not required
to preserve the default-floor conclusion.

## D9a utf8parse replication preregistered (2026-07-12, before generation)

The second shipped-library target is frozen independently against ordinary
`utf8parse` 0.2.2 defaults and its public `Parser`/`Receiver` API.  The one-shot
task maps valid scalars and invalid-sequence events into caller-owned `u32`
storage.  The sole generation identity is again one exact
`gpt-5.6-terra`/medium/default-service trajectory with an initial response and
at most three repairs; first correctness-green freezes immediately.

The deterministic gate has 84,041 cases.  Rust/oracle preflight, facts-on, and
facts-off each run every case at exact `src.len` capacity and at `src.len + 32`;
a subprocess boundary additionally requires every smaller capacity to trap
before writing.  The primary score is 30 fresh processes over a fixed 128 MiB
four-class corpus, with the same balanced ordering, paired ratios, stratified
bootstrap, and +/-2% verdict band as the first target.  The scope is explicitly
one-shot and synthetic-corpus-specific, not a claim about chunked streaming or
typical terminal text.

Prompt SHA-256 is
`81f023e583987d4610f15faa529b6481805dc4094fda2168146cf9ea9e9c903a`;
protocol SHA-256 is
`e786e949d1500c605ef37195dadc60e25380d4e02b7afc9f660d1cf5e062d960`.
Pre-freeze evidence: private reference candidate passed the full locked
evaluator (`correct cases=84041`); Rust/harness tests and Clippy are green;
strict C boundary and Python syntax checks are green; 16 MiB orchestrator smoke
completed all six non-scoring order blocks with identical counts/digests; root
`make check` and `make -C compiler check` are green.  No official model output,
proof report, IR, assembly, or timing result existed at this entry.

## D9a utf8parse Terra source frozen (2026-07-13, before measurement)

The sole preregistered `gpt-5.6-terra`/medium/default-service trajectory ran
under identity `runs/primary-terra-medium-preregistered`.  Round 0 compiled and
passed all 84,041 locked correctness cases, so it froze immediately; no repair
prompt or second model call was made.  Frozen source SHA-256:
`3bb7951995120d24c651b6750ac9ba4c7a8fb9b61bc02fb5d219e64e640c1535`.

The frozen manifest validates the exact one-round trace, every archived
artifact hash, and the first-green identity.  An independent post-freeze
evaluator invocation again reports compile green and `correct cases=84041`.
At this entry no proof report, IR, assembly, or performance result has been
used to alter the source, and the preregistered scoring campaign has not yet
run.

## D9a utf8parse score: default-floor win replicated (2026-07-13)

The first scoring attempt was invalidated at block 27 when the observed power
source changed from AC Power to Battery Power.  Its 28 completed raw rows and
all logs remain archived; none entered the result.  The protocol-authorized
full rerun binds that invalid campaign's metadata hash and the transition
reason, restarted at block 0 in a fresh append-free directory, and completed
all 30 processes and 90 samples under stable Battery Power with no recorded
power or thermal transition.

Median throughput in the valid campaign was 333.105 MiB/s xlang facts-on,
319.673 MiB/s facts-off, and 280.474 MiB/s shipped Rust.  The preregistered
paired facts-on/Rust median ratio is 1.0980 with stratified bootstrap interval
[1.0849, 1.1448], a predeclared **meaningful xlang win**.  Facts-on/facts-off
is 1.0051 with interval [0.9856, 1.0349], **inconclusive against the +/-2%
band** and unable to change the primary verdict.

Attribution remains negative for proof elision.  The facts-on and facts-off
reports are byte-identical, retain all 11 bounds sites, and prove none; their
optimized instruction bodies are also identical.  Rust's generic public
`Parser`/`Receiver` path monomorphizes and inlines into a call-free branch DFA,
so this result is not a dynamic-dispatch penalty either.  The defensible static
mechanism is different state numbering, control-flow layout, and source-loop
lowering for two implementations of the same DFA semantics; it does not imply
that expert Rust cannot choose a different layout.

This independently reproduces the default-floor win on a second shipped
library, with a much smaller effect than percent-decode.  D9a's requested
replication is therefore complete, but the claim remains limited to these two
first-green sources, frozen synthetic workloads, machine, and toolchain.  The
utf8parse corpus is deliberately equal-weight and malformed-heavy, and the
adapter is one-shot rather than a persistent streaming parser.  Canonical
evidence is in `experiments/default-floor/utf8parse/RESULTS.md`; valid raw
SHA-256 `71297ba0bfc2bc7b8af1f29b33f7c4769cb6c706f8a0c3f7c855363b86404ab3`,
analysis SHA-256
`1a16044495b3e4d2e41c6f49d6f32cf9390d95a3b40f5f3e8c278077e115ff1c`.

## D9a cross-target synthesis frozen; experiment expansion stops (2026-07-13)

The two separately preregistered target results now form the complete D9a
default-floor evidence set.  Report the target-specific paired throughput
ratios, never an average or pooled effect: percent-decode 1.6533 with
descriptive 95% bootstrap interval [1.6309, 1.6668], and one-shot utf8parse
1.0980 with interval [1.0849, 1.1448].  The targets were selected rather than
randomly sampled, the second was chosen after the first result, and both share
one model, compiler/checker, M4 host, and harness design.  This is cross-target
replication, not an ecosystem prevalence estimate or statistically independent
replication.

The supported claim is that, on these two frozen shipped-library tasks, the
first correctness-green `gpt-5.6-terra`/medium xlang sources beat the released
crates' ordinary public paths through minimal safe adapters.  All 6 and 11
reported xlang bounds sites respectively remained checked, so neither result
is a proof-elision win.  The evidence does not support “xlang is generally
faster than Rust,” “AI xlang is automatically optimal,” or any claim about the
expert-Rust ceiling.

Canonical wording, methodology, and caveats are frozen in
`experiments/default-floor/RESULTS.md`.  D9a's required second target is
satisfied; no third benchmark is needed for this gate.  Any later measurement
of either source is secondary sensitivity evidence and cannot replace its
primary campaign.  The experiment track now yields priority back to the xlc
self-hosting build track.

## E0.1 detached candidate evidence archived; prototype remains rejected (2026-07-13)

The exact unconditional candidate reviewed in
`experiments/data-layout-owning-sequence/HOSTILE_REVIEW_1.md` is now durable as
`DETACHED_CANDIDATE.patch`.  It is a 57,547-byte binary diff against Git
`58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`, changes four files by
`+968/-21`, and has SHA-256
`bed070414f9552ea105857404d6d1296b98542a28cc65fa6899a197830e6774e`.
An isolated worktree at the recorded parent accepts the archive under
`git apply --check`.

Archiving does not change the hostile-review verdict: the prototype is
semantically rejected, contains no scored timing evidence, and grants no
production implementation, specification, xlc-migration, or default-teaching
authorization.  The main toolchain still has no E0.1 candidate semantics or
feature flag.

## OWN-1 index-atom flow omission recorded after mainline repair (2026-07-13)

E0.1 hostile review exposed a pre-existing mainline checker defect: an index
atom could read a field through an already-moved affine binding without entering
ownership flow.  Commit `38d642e` makes index atoms pass the normal
liveness/readability checks and adds the negative conformance case
`own1-neg-index-atom-after-move`.  This is an independent current-language bug
fix, not evidence for Flat records and not authorization for E0.1 production
semantics.

## GRAM-9 constructor drift and recursive projection drift repaired (2026-07-13)

Both executable frontends now enforce the selected strict three-address grammar:
a constructor is an expression and cannot occupy a call-argument or construction-
field atom position.  Auditing the pre-repair repository found 770 existing
violations: 745 in the 20 self-hosted compiler source files, 24 in 14 conformance
sources, and one embedded stage-0 fixture.  Of the compiler violations, 744
nullary Copy enum constructors are now bound once immediately before use; the
remaining violation was `AstConstructor()` inside the invalid constructor-as-atom
parser branch and disappeared with that branch.  The conformance migration
similarly binds 19 Copy values and binds then explicitly moves five affine
aggregate/payload values.  The self-hosted compiler pays 744 additional source
lines; this is measured maintenance evidence for any later GRAM-9 redesign, not
permission to ignore the current rule.  Existing performance and code-shape gates
remain authoritative for the generated program.

The same conformance pass repaired stage 0's recursive place suffix handling.
Its tokenizer deliberately retains dotted lower-case tokens, but the parser had
treated the suffix token in `index<T>(...).inner.value` as one field.  It now
expands every segment, while xlc's already-correct lexer/parser is pinned to two
successive field nodes.  Two negative GRAM-9 cases and one runnable recursive-
projection case make both boundaries durable.  These are current-language parser
repairs, not E0.1 record-storage semantics.

## Current-language expression-context ownership/type seams repaired (2026-07-13)

The E0.1 hostile review exposed a family of pre-existing checker seams after the
initial index liveness/readability repair at `38d642e`.  The complete repair keeps
the current language unchanged: projected places are classified by their projected
type; index atoms receive one ordinary ownership and type judgment; `index<T>`
checks its stated element type and exact u64 offset; and reference, box, and arena
holders require the explicit `deref(.)` spelling in index, match, try, and referent-
return contexts.

Match now consumes affine own scrutinees, copies tag-only scrutinees, rejects
explicit `move` of Copy or through a borrow, and requires an enum type.  Specialized
Result/Option/user-enum payload types enter flow; shared and exclusive borrowed
payload binders preserve ancestry and per-field aliasing, so derived access is legal,
siblings remain disjoint, and the parent holder remains frozen.  Contextual own
returns consume affine places without stealing through a borrow.  Try and const
bindings likewise retain concrete type metadata.  Conformance review also exposed
that the prototype type table had never encoded OP-6; it now implements the exact 29
total conversion pairs and returns `Result<Dst, NarrowError>` for every other
distinct numeric pair.

Twenty-five manifest/source gates cover the repaired seams: 19 negative diagnostics,
five runnable/accepted positives, and one checker-positive aggregate-payload case
that remains pending only because the disposable democ ABI cannot lower aggregate
enum payloads.  Two older fixtures were corrected: borrowed Copy payloads now use
explicit dereference while retaining byte-identical LLVM (SHA-256
`ab236e489742577015d080b06b6eb1a4d5d82c7486f5006da185e9f9df5ed`), and the
double-consumption negative now uses a genuinely affine payload rather than a
ratified Copy tag-only enum.

Hostile review found no remaining checker/conformance finding.  Verification is
94/94 checker units; 10,000 modelchecked programs with zero accepted soundness
violations; conformance 259 PASS / 14 SKIP / 90/90 rules; full `make check` ending
`ALL VERIFICATION LAYERS GREEN`; and clean whitespace checks.  This is an existing-
specification conformance repair, not E0.1 candidate semantics or authorization for
any record-storage design.

## E0.1 owner-review package revised; production design remains open (2026-07-13)

The owner-advisor review is durably dispositioned in
`experiments/data-layout-owning-sequence/REVIEW_RESPONSE.md`.  The exact rejected
prototype source is already archived at `68a55e4`; the initial index liveness repair
is narrowly attributed to `38d642e`; strict GRAM-9 and recursive projection are
repaired at `50a1ddd`; and the complete current-language expression-context checker
repair is `7438e17`.  Disposable executables remain outside the repository, while
every reviewed source artifact is recoverable.

The design recommendation is reopened.  Automatic structural Copy, declarative
`copy struct`, and nominally affine record storage are distinct ownership routes.
The OWN-1 tag-only precedent is answered on memory safety, nominal authority/private-
constructor invariants, and copy-cost visibility.  Recursive recipe, single-level
initializer, builder/initialized-prefix coupling, explicit Repeat/Clone, and Copy-
tier fill are preliminary alternatives with nonzero grammar/type/effect/lowering
prices; none is selected.  Non-builder E0.1a/E0.1b remains serial, while a builder
requires a newly approved coupled protocol.  An opaque kernel sequence would reverse
STOR-1 and therefore needs a full META-5 and design-tree redecision.  PATTERNS remains
a production gate, and float exclusion remains only a limitation of the archived
prototype.

No production specification, storage, xlc-layout, PATTERNS, teaching, or external-
model change is authorized.  The MCTS-Mem tree is unchanged while the owner choice
is open.  If a selected production route supersedes the recorded declarative-copy
path, the same decision must move that path to the appropriate `.alt/` branch with
paired reasons; selecting it instead must update its card with the deciding evidence.

## E0.1a ownership-route paired protocol passed draft hostile review; no candidate work authorized (2026-07-13)

`experiments/data-layout-owning-sequence/OWNERSHIP_ROUTE_PROTOCOL.md` now defines
the owner-review draft for a direct `DECLARATIVE_COPY` versus
`AFFINE_FIXED_BUILDER` screen.  The exact reviewed protocol SHA-256 is
`88d70083f9cf0219d558675b34a42f54c851793125fccebc07c3f48f4aa1b003`.
`OWNERSHIP_ROUTE_HOSTILE_REVIEW.md` records the three independent
ownership/state, benchmark/statistics, and repository-consistency reviews, the
blocking findings, their dispositions, and three final `PASS` verdicts on those
exact bytes.

The protocol compares complete initializer/ownership bundles, uses one direct
candidate contrast, makes `CURRENT` an unchanged-source identity control,
separates intrinsic route contradictions from invalid artifacts, freezes all
scored counts in Lock A, embargoes hidden writer results until campaign-wide
freeze, and fails closed on unmatched semantic/backend transfer provenance.  It
also closes recursive declared-Copy eligibility, builder assignment and cleanup
edges, record-array consequences, weak-writer clustering, current-SoA event
mapping, and the META-5/STOR production handoff.  `RESEARCH.md` now records
primary-source Swift and Move evidence without selecting a route.

Sequential post-change `make check` and `make -C compiler check` both pass; the
root gate reports 90/90 rule coverage, 10,000 model programs with zero accepted
soundness violations, 259 conformance passes/14 skips, and all verification
layers green.  This is not Lock A or a preregistration.  No candidate
implementation, timing, profiling, external disclosure, specification change,
xlc migration, PATTERNS change, default teaching, or production design is
authorized.  The MCTS-Mem tree remains unchanged pending owner selection of the
experimental narrowing.

## General-purpose data-structure capability floor opened; E0.1 candidate work remains paused (2026-07-13)

The initialized-prefix discussion exposed an upstream process failure: E0.1's
fixed-record ownership screen had not first proved that ordinary checked xlang
can express the dense, sparse, relocation, traversal, recursion, and stable-
identity transitions required by representative data structures.  The local
protocol was internally reviewed but its review scope did not include this
neighboring-operation census.  Under the standing owner instruction to stop
implementation and research first, the paired protocol remains paused before
Lock A.

`general-purpose-data-structure-capability-RESEARCH.md` establishes the
non-normative correction.  Its exact reviewed SHA-256 is
`48384d74624c40dad13514232985d39df6fa5910ba5bf513cd39f941440d82c7`.
The report separates protected baselines, mandatory floor contracts,
ordinary-library topology witnesses, held-out anti-special-casing witnesses,
and prevalence-gated options; maps each to a named canary; constrains R3 to
caller-observable contracts; freezes the workload/target and unique-winner
rules needed to interpret “fastest”; and records the finite copied-handle
impossibility boundary.  Public uninitialized payload access and standard-
library-only raw privilege remain rejected.  H-STORE must directly instantiate
the same checked storage transitions available to an unseen ordinary library.

`general-purpose-data-structure-capability-HOSTILE-REVIEW.md` (SHA-256
`b1d99a055372602aac0b8714661a09e9771e60ac66014f8a8ef2f34a625e9467`)
records three independent final `PASS` verdicts on the exact report bytes for
ownership/state/failure soundness, performance and R3 selection, and
specification/MCTS-Mem/process consistency.  Earlier rejected snapshots and
every blocking disposition are preserved in that review.

G0 is the only proposed next research step: freeze the role-mapped operation
registry, semantic contracts, payload/workload/target matrix, selection rule,
META-5 derivability ledger, held-out dependency budgets, and rejection
thresholds.  G0 itself needs review before candidate implementation.  No
language mechanism, collection representation, standard-library placement,
specification or PATTERNS change, xlc migration, scored run, external
disclosure, or default teaching is authorized.  The MCTS-Mem tree remains
unchanged because no production route was selected.

Sequential post-change verification passes: `make check` reports 90/90 rule
coverage, 10,000 model programs with zero accepted soundness violations, 259
conformance passes/14 skips, and `ALL VERIFICATION LAYERS GREEN`;
`make -C compiler check` also passes.

## D11 stages the data-structure capability floor by family; no next work is authorized (2026-07-13)

The owner confirmed xlang's eventual general-purpose systems-language scope and
rejected the initial all-at-once G0 process. The durable ruling is D11 in
`optimizer-language-research/notes/user-directives.md`: a bounded G0-Core freezes
the global registry, coarse contracts, safety/no-tax laws, family dependencies,
exact held-out dependency budgets, reopening rules, and later-gate schemas. Each
family freezes exact semantics, algorithms, workloads, thresholds, soundness
fixtures, construction/tuning protocol, and META-5 derivability only in its own
Lock A immediately before candidate work. Candidate Freeze B remains a distinct
pre-scoring artifact freeze.

The current exact
`general-purpose-data-structure-capability-RESEARCH.md` SHA-256 is
`066c6e411cc0eefec608fb53371e4badfd91017617cefe63178ca101e19191b6`.
It separates family closure from the complete-floor claim: a family that passes
its assigned M/W/H obligations while protecting every B baseline may seek a
separate owner-approved production adoption, but no adoption may expose a public
transition, state topology, trusted fact, or trusted path outside the adopted
family locks. The complete claim waits for every protected B and every mandatory
M, topology W, and held-out H obligation. Append-only and recyclable stable
identity remain separate contracts, and the append-only path pays no generation
or recycling tax.

The first complete staged snapshot,
`3117ab458b0fc9a10f90a80dc8f923a5acfa2b7116bdbbcfc7b06b41a3323134`,
received no P0 finding but received three `REVISE` verdicts. Hostile review found
that candidate-construction effort was not preregistered, gating soundness
fixtures could change without reopening, W closure followed candidate claims,
cross-family safety followed claims rather than implemented/exposed state,
borrow-bearing payload scope was unstated, exact held-out budgets had weakened
to a schema, and stale E0.1 documents still exposed the superseded authorization
path. All findings were repaired. Three independent read-only scopes then
verified the final exact report bytes and returned `PASS` with no P0, P1, or P2
finding for ownership/state/failure soundness, performance/staging/selection,
and Constitution/specification/MCTS-Mem/repository consistency.

`general-purpose-data-structure-capability-HOSTILE-REVIEW.md` preserves the
initial exact review and records the D11 amendment, rejected hash, dispositions,
and final exact-hash passes; its SHA-256 is
`80cae6e2296da389581ed908b695cd0a301adfc399f5421adb8444e96b5b2265`.
The plan, compiler plan, experiment index, and E0.1 report, candidate, protocol,
review, and results surfaces now mark the old paired route as historical input.
G0-Core plus dense Family Lock A are necessary but not sufficient for a later
owner decision to lift the E0.1 pause; neither historical protocol restarts
automatically.

The MCTS-Mem root, pattern-doctrine, and data-model nodes gain sourced rationale
facts for the eventual general-purpose scope, ordinary-library completeness
test, family-by-family adoption boundary, and separate append-only/recyclable
contracts. No Item, Move, node, or `.alt/` path changes: the monolithic G0 was an
unratified research proposal, and no production mechanism, STOR-1 route, or
language design was replaced. `npx mcts-mem lint` passes.

Post-change verification is green. `make check` reports 90/90 rule coverage,
10,000 model programs with zero accepted soundness violations, 259 conformance
passes/14 skips, zero codegen-parity gate failures, and
`ALL VERIFICATION LAYERS GREEN`; the separately invoked
`make -C compiler check` also passes. `git diff --check`, the changed-content
English-only scan, and byte identity of `AGENTS.md` and `CLAUDE.md` pass.

This step authorizes only the corrected research boundary. It authorizes no
G0-Core work, Family Lock A, experiment, candidate or production implementation,
specification or PATTERNS change, xlc migration, scored run, external disclosure,
or default teaching. The next step remains a separate owner discussion.

## D12 authorizes the complete G0-Core census and derivation research; mechanisms remain paused (2026-07-14)

The owner confirmed the general-purpose systems-language objective and
authorized completion of the research program proposed after D11. D12 records
the method: use the stable Rust `core`, `alloc`, and `std` caller surface as a
finite completeness anchor; normalize observable contracts rather than copying
Rust mechanisms; derive a Pareto-small checked capability basis below named
containers; require ordinary no-unsafe library generativity, cross-ecosystem
witnesses, and explicit accounting for every deferred systems domain; and
preserve structural-performance and protected-baseline costs in the minimum.

The exact research charter is
`optimizer-language-research/implementation/minimal-systems-capability/G0-CORE-CHARTER.md`,
SHA-256
`11b017da99da40713038bcbdcdeee0787e31a258b50e9c5dbc0b5ad52edfcc13`.
It pins Rust 1.97.0, released 2026-07-09, to peeled source commit
`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`; freezes inventory,
normalization, derivation, performance-control, review, and completion schemas;
and names the detailed first closure as the sequential, unique-owner,
ordinary-library data-structure floor. The complete systems envelope remains
accounted but not silently claimed by that closure.

This authorization covers source inspection, reproducible census tooling,
contract and capability ledgers, proof sketches, E0.1 traceability,
cross-ecosystem research, and hostile review. It does not authorize Family Lock
A, syntax, a storage substrate, a trusted fact or path, a candidate or
production implementation, a specification or PATTERNS change, xlc migration,
E0.1 restart, a scored candidate run, or default teaching. `THE-PLAN.md` and the
byte-identical repository instructions now expose that exact boundary.

Post-charter verification passes. `make check` reports 90/90 specification
coverage, 10,000 model programs with zero accepted soundness violations, 259
conformance passes/14 skips, zero codegen-parity gate failures, the complete
self-hosted compiler gate, and `ALL VERIFICATION LAYERS GREEN`. `git diff
--check` and byte identity of `AGENTS.md` and `CLAUDE.md` pass.

## Rust 1.97.0 public surface receives a reproducible mechanical census (2026-07-14)

The first D12 research step inventories the exact Rust 1.97.0 public rustdoc
surface from matching compiler and source commit
`2d8144b7880597b6e6d3dfd63a9a9efae3f533d3`. The checked extractor SHA-256 is
`12c4642b8bf848bcf82d81e04a200aa4dc9fe52be9edf2f346fd145ec71bd915`.
It starts at the public `core`, `alloc`, and `std` module indices, records public
items, inherent declarations, and defining-trait declarations, preserves unsafe
and unstable evidence, canonicalizes reexports by normalized source
declaration, and deliberately does not multiply concrete trait implementations.

The detailed output has 16,432 rows: 9,874 stable-safe renderings, 554
stable-unsafe renderings, and 6,004 unstable renderings. Source canonicalization
leaves 5,096 stable-safe and 273 stable-unsafe declarations. The 290-row module
ledger has no missing page and no unresolved external module link. The 28
collapsed `core::arch`/`core::intrinsics` module rows still count and digest
16,888 direct stable and 12,633 direct unstable target-specific entries; this is
explicit compression into the later target/runtime family, not omission.

Output hashes are pinned by
`RUST-1.97.0-CENSUS-MANIFEST.json`: the API inventory is
`e1d59827c606978742419869d89e558f0f000da53f74467bd5c9594c96055888`
and the module ledger is
`5a4707a77b920dfa9de57b1eefcfb08efec08a222a4c4120c6ab1a42052cb4e4`.
An independent narrow extractor's selected array/slice/text/box/collection/
shared-owner/dynamic-borrow counts reproduce exactly: 547 page-local safe and
36 page-local unsafe methods, with two safe and one unsafe duplicate renderings,
for 545/35 canonical declarations. The checked verifier SHA-256 is
`65a9c49a1cba45fd5f37458369f8ce51112b9ac358282f0841545aa3892794e0`
and reports `PASS`.

This is demand and implementation-evidence accounting only. Counts are not
capability counts, do not prove xlang derivability, and select no syntax,
storage state, trusted transition, standard-library API, or production route.

## G0-Core normalizes the data floor and separates semantic obligations from mechanisms (2026-07-14)

The second D12 research step maps all 5,369 canonical stable Rust 1.97.0
declarations, including 5,096 safe and 273 unsafe declarations, to one of the
applicable D01-D25 public systems domains. Every unsafe writer surface remains
explicit `unsafe_evidence_NG`; its underlying checked need routes separately.
The same checked classifier maps all 290 reachable modules, using D26 only as
an explicit holding route for otherwise unclassified modules with zero direct
stable items. The declaration map SHA-256 is
`a3907c0fb0d1c24b9651dd49e6d135095fe444f6771b0d8b278bbc8871808f54`;
the module-domain map SHA-256 is
`1f217998d974999c3f920dc56b6b2836eade87b500377310f0cdb109b568b2cb`.

The detailed data/text/iteration/ownership/lifecycle seed is normalized into
224 contract and evidence clusters. An exact 545-row surface map assigns each
canonical stable-safe inherent declaration to one primary cluster exactly
once; 35 canonical stable-unsafe declarations are retained only in eight
unsafe evidence clusters. The contract census SHA-256 is
`64322a701fe6d84c2631873c35376bc668de7596d7e06d529c7e41db00a7be5a`;
the exact surface map SHA-256 is
`8398121eafd71363c14c3eb44bbc2543b5db9506d27fe0b55294c1e6c8555fa6`.

The semantic result is recorded separately from candidate mechanisms: 45
operational capability obligations, 12 orthogonal proof dimensions, and 16
global lifecycle, fact, generativity, scope, and protected-performance laws.
The capability registry SHA-256 is
`be31ac48a9d3076e7a0cf38a8fe75397851caedd015d868bbfbc4335f34c9ff5`;
the semantic registry SHA-256 is
`4dafb78728d3d78fe9fe9088236f43a7ec806fce2a833c65890f35d6dfb4441a`.
The witness registry freezes seven visible topology witnesses and three exact
held-out contracts without creating hidden source or scored inputs. E0.1 stays
suspended, with every historical ownership, layout, capacity, correctness,
structural, and performance obligation traced into a future family lock.

Focused verification passes: the raw census verifier reports 290 modules and
16,432 detailed rows; the domain classifier reports 5,369 declarations and 290
modules accounted; the contract verifier reports 224 clusters, exact 545/35
seed coverage, no omission, no duplicate primary mapping, and no unknown
contract; and `git diff --check` passes. This step selects no storage spelling,
state representation, checked transition, cleanup mechanism, fact channel,
compiler path, standard container, candidate algorithm, numeric threshold, or
production change.

## The domain ledger is repaired to separate Rust caller safety from xlang admission (2026-07-14)

An independent hostile audit found that the first whole-surface classifier
used `checked_surface` for Rust-safe declarations, left several source-surface
families with imprecise semantic owners, did not assign one terminating
disposition per declaration, and overstated dynamic loading as a settled
non-goal. Those defects could make a complete row count look like a completed
xlang capability decision.

The repaired classifier retains Rust `caller_safety` only as source evidence.
Each of the 5,369 canonical stable declarations now terminates in exactly one
conservative `G0`, `LIB`, `LATER`, `RED`, or `NG` disposition; all 273
Rust-unsafe declarations and 169 Rust-safe raw-pointer, manual-lifetime, leak,
or spare-capacity boundaries remain evidence rather than admitted xlang
surface. The 545 detailed stable-safe seed declarations carry their exact
contract IDs. Macro, keyword, primitive, OS-extension, panic-hook, raw-memory,
and dynamic-loading routes now land in their semantic domain or named later
frame. Only the item-free `core::panicking` module remains in the explicit D26
holding route.

The checked classifier SHA-256 is
`3754a028821376f16143ed8c2ff0aea768b79f2088439bf5a6eca27bd058a3f0`;
the rule table SHA-256 is
`5dfac844dc49d2a6725431dd0424d816ff5841e651d4b4412a805e3a260087ba`;
the declaration map SHA-256 is
`a635f656e2b2dd71fcf9a15e0c85a2ade3efeedaf6ed06ebe3391fb87c770645`;
and the module map SHA-256 is
`7f664088c5a93d9a0ba1a6a9068f2405bd15df03cc131a00f696a0ff5ed67758`.
The regenerated classifier reports 5,096 safe and 273 unsafe declarations and
290 reachable modules accounted; the independent detailed-contract verifier
still reports 224 clusters with exact 545/35 seed coverage; and `git diff
--check` passes.

This repair changes accounting precision only. It selects no xlang surface,
mechanism, trusted path, family closure, specification rule, compiler change,
or production implementation.

## G0-Core closes the pre-review obligation, route, iteration, and witness accounting (2026-07-14)

The D12-authorized synthesis now separates 49 operational obligations, 12
orthogonal proof dimensions, and 16 global laws without selecting one language
primitive per obligation. `BR-STORED` preserves the missing lifetime/provenance
question for borrow-bearing payloads; the first detailed floor remains
explicitly region-free and borrow-free. `IT-COMPOSE` and W-PIPE require reusable
zero-materialization traversal rather than a hand-fused loop. `F-MMIO` preserves
the checked volatile/device-memory need while keeping raw-pointer volatile
spelling inadmissible.

The classifier now has independent surface-evidence and underlying-need axes.
Across all 5,369 canonical stable declarations it records 4,609 safe contract
anchors, 170 safe boundaries, 273 unsafe boundaries, and 317 Rust-only surfaces;
the underlying routes are 1,033 G0 contracts, 302 library contracts, 3,765
later-family contracts, seven exact frame services, 43 redundant declarations,
217 declarations with no independent need, and two true non-goals. The only
true non-goals are `catch_unwind` and `resume_unwind` under EFF-4. Raw, unsafe,
manual-lifetime, leak, partial-initialization, FFI, atomic, pin, and MMIO evidence
retains a named safe displacement instead of being mislabeled as a non-goal.

The detailed census contains 258 normalized rows. A new exact D10 crosswalk
routes all 150 stable iteration/range declarations once (132 iteration, 18
range; 107 contract routes, 43 redundant surfaces), keeps `ExactSize` and
`Fused` separate, and splits the stable range surface into 13 contracts. The
258-row derivation matrix records E=4, P=0, U=10, X=212, FRAME=4, DEFERRED=19,
BOUNDARY=9, and NG=0. BOUNDARY means the Rust spelling is inadmissible while
the checked need remains live; it is not a non-goal.

Visible and held-out witness budgets now use exact closed allowlists. W-POOL
freezes generation exhaustion, retirement, memory history, cross-pool policy,
and returned-borrow ownership. W-ARENA promises owner-tied borrows across later
allocations without importing physical address stability, rejects reset while
borrows live, and prices chunk allocation. H-STORE includes dependent-state and
result-reborrow obligations and receives only the public allocator facade.
H-LRU and H-IPQ freeze exact affine outcomes; H-IPQ must implement heap repair
over selected dense/map public APIs rather than import a finished heap.

Focused verification passes: deterministic classifier regeneration, exact Rust
census, 258-row data-contract verification, 150-row D10 verification, 258-row
derivation verification against all 49 capability IDs, exact witness-ID checks,
`git diff --check`, AGENTS/CLAUDE identity, and the cached MCTS-Mem linter (`lint
clean: 28 nodes, 0 fact files`). The exact aggregate manifest and final
exact-hash hostile reviews remain the next research step.

Key pre-review SHA-256 values are: capability registry
`1aa1a05a670c5f73387b8b96913da7acbdc6e98bebee1fbf7f0e12301729efb7`;
declaration classification
`d8e6530517fa4ad74614c7611721d618834e7bc5002ee5c11953822a88629d6a`;
detailed census
`cc896e31435e5dce573ce58be7ba6bf2a612eac8ece62ebf40089f5ac6e71838`;
D10 crosswalk
`f1e2588e39da498130b175d5f3438da7748e51ae56dc005a51a295e761482bb8`;
derivation matrix
`6832c7aa2e6bebf2f89bbb5948e162a2c0fae4b549ab2efed1fa569339ea5082`;
and witness registry
`1e684589a7897caa31421a4d6eddcebc5307e080625f20982eb180853f8728f0`.

This is research accounting only. It selects no syntax, substrate, runtime
representation, trusted implementation, family closure, specification change,
compiler change, production implementation, E0.1 restart, or default teaching.

## G0-Core capability accounting is complete; mechanisms remain unselected (2026-07-14)

The finite Rust 1.97.0 anchor, non-importable contract clustering, exact
evidence universe, family routing, trait-implementation topology, payload
scope, witness obligations, and combined dependency graph are frozen by the
110-artifact manifest whose SHA-256 is
`f0eced756688affef1732a133c43fb39ab6fc672334dca27b26129ddb5123719`.
The closed accounting contains 276 contract clusters, 1,961 evidence
relations, 334 implementation-topology keys, 378 owning-cluster relations,
49 family obligations, and 294 payload branches. The corrected evidence
policy binds all 97 two-target trait relations to their child-specific
operation gates and full route prerequisites; the remaining 281 relations
have one target. Independent exact-hash hostile reviews of coverage and
provenance, semantic and ownership soundness, and performance and staging all
returned PASS with no P-level finding. `verify_g0_core.py`, `make check`, the
direct `make -C compiler check`, `git diff --check`, AGENTS/CLAUDE identity,
and the MCTS-Mem linter are green.

This completes D12's G0 research authorization. It selects no language
mechanism or representation and authorizes no Family Lock, candidate,
experiment, specification or compiler change, production implementation,
E0.1 restart, xlc migration, or default teaching. The next possible owner
decision is whether to authorize drafting the dense unique-owner Family Lock
A for review; drafting would not authorize implementation or a scored run.

## D13 authorizes the dense unique-owner Family Lock A draft only (2026-07-14)

After reviewing the completed G0-Core result, the owner authorized drafting
the dense unique-owner Family Lock A. The permitted work is exact audit-domain
refinement, member/outcome contracts, finite mechanism descriptions and
reference algorithms, soundness and performance protocol freeze, protected
controls, validators, META-5 and E0.1 accounting, and hostile review. Candidate
construction, Candidate Freeze B, scored or held-out execution, language or
specification decisions, compiler or production implementation, E0.1 restart,
xlc migration, and default teaching remain prohibited. The completed reviewed
draft returns to the owner before any further authorization.

## D13-R1 rejects the first dense Family Lock A draft (2026-07-14)

Independent coverage, soundness, and performance reviewers rejected the first
generated draft before it became a durable lock. The review independently
confirmed only the immutable audit-domain counts: 65 routed clusters, 651
parent evidence identities, 119 dense and 193 non-dense concrete
implementations, 29 additional operation-gate relations, 25 role identities,
49 capability identities, the 39/17/4/5 payload partition, 75 overlay keys,
and one delegated allocation-error row. It found blocking heuristic routing,
mutable-input, outcome, oracle, candidate-lifecycle, fact-channel, ZST,
performance-cell, statistics, protected-control, and META-5 defects. The
failed-draft findings and exact reviewed manifest/report hashes are preserved
in `dense-family-lock-a/DENSE-HOSTILE-REVIEW-ROUND-1.md`; the overwritten
untracked draft bytes are not represented as recoverable evidence. Repair and
new exact-hash hostile review are mandatory. Candidate construction, scoring,
language selection, and production implementation remain unauthorized.

## D13-R2 rejects the first repaired dense coverage authority (2026-07-14)

An independent frozen-input recomputation confirmed the 65-cluster,
651-parent, 119/193 topology, 29-gate, 39/17/4/5 payload, 75-overlay, and
426-selector-child domain facts, but rejected the first repaired coverage
authority. Unanchored selector children still inherited cluster-wide targets
and members; four active BR-STORED clusters lacked required bindings;
O-ROPE-UNIQUE was omitted; real excluded evidence lacked outcome bindings;
the local member model was outside the reviewed dependency set; and coherent
row mutations bypassed validators that checked only mutual consistency. The
exact failed bytes, hashes, mutation canaries, and required repairs are durable
in `dense-family-lock-a/DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-2-FAIL.md` and the
preceding Git snapshot. A fresh exact-hash review is required after repair.
Candidate construction and every production-relevant action remain
unauthorized.

## D13-R3 closes the exact dense coverage authority (2026-07-14)

The repaired dense Family Lock A coverage authority passes exact-byte hostile
review. An independent G0-only oracle reproduced all 65 audit clusters, 426
selector children, 1,400 target terminals, and 780 evidence/member bindings;
all 456 direct identities anchor exactly once; and all 22 mutation attacks are
rejected. Five cross-topology raw identities retain their real gate-local
excluded outcomes without inheriting F-DENSE applicability. The reviewed
sources, generated authorities, immutable G0 inputs, hashes, adjudication, and
strict authorization boundary are recorded in
`dense-family-lock-a/DENSE-COVERAGE-HOSTILE-REVIEW-ROUND-3-PASS.md` at SHA-256
`d8ee4c161e84a3996c0167b54576893074a16775b30994ab8236e79fa63d4798`.
This step closes coverage and provenance only. Candidate construction,
selection, execution, scoring, language or specification decisions, compiler
or production implementation, E0.1 restart, xlc migration, and default
teaching remain unauthorized.

## D13-R4 closes dense contracts and mathematical soundness (2026-07-14)

The exact dense Family Lock A contract and mathematical-soundness layer passes
independent exact-byte hostile review. The frozen model contains 303
member/outcome contracts, a bijective 303-row owner-role registry, the exact
five-by-303 candidate binding product, 97 behavior-preserving adapter groups,
five pairwise-distinct lifecycle mechanisms, eight non-production fact-channel
contracts, and 2,002 deterministic traces. All 32 registered mutations and
three additional coherent hostile mutations fail closed. Every arm uses the
same conditional affine single-live-interval carrier for owning traversal, so
candidate-private cursors, allocators, sealing, or provenance routes cannot
confound the comparison. The reviewed hashes, independent joins, lifecycle and
policy audit, trace partition, and strict boundary are recorded in
`dense-family-lock-a/DENSE-SOUNDNESS-HOSTILE-REVIEW-PASS.md` at SHA-256
`20b6325366c961a5d608066da8acd9a9c19352290fdaa44e3666f2e14430c7c7`.
OD-0, OD-1, OD-3, and OD-4 remain unresolved. This step authorizes no candidate
construction, execution, scoring, selection, production fact channel,
language or specification decision, compiler or production implementation,
E0.1 restart, xlc migration, or default teaching.

## D13-R5 closes the dense performance and statistical protocol (2026-07-14)

The exact Dense Family Lock A performance layer passes independent exact-byte
hostile review. The frozen registry derives all 303 contract dispositions into
97 standalone same-shape Rust operation gates and 520 exact cells, including
502 timed-primary cells, across eight unresolved owner branches and 25 explicit
external blockers. Primary decisions use fixed-n strict raw-integer scheduled-
mixture successes, exact worst-case Poisson-binomial tails, one global Holm
benefit family, reference-only clustered Q/P/M planning power, typed hidden
randomization manifests, and a counted 175,667,428-operation power-engine
ceiling. The verifier rejects all 39 registered mutations; the independent
review additionally rejects eight hostile probes, reconstructs every family,
law, task, multiplicity, partition, randomization, and resource identity, and
regenerates all 29 generated files byte-for-byte. The report preserves six
invalidated coherent-looking freeze histories, including the final stale
median/log-ratio endpoint authority, so none can silently recur. All 39 current
reviewed file hashes and the strict research-only boundary are recorded in
`dense-family-lock-a/DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md` at SHA-256
`83e9135ba26348c6f89423c4710259da76a65381aca2e7d2e54799ef386541ff`.
Post-report `make check` and the separate `make -C compiler check` both pass.
No pilot or candidate observation has occurred. This step authorizes no
reference pilot, candidate construction, Candidate Freeze B, execution,
scoring, held-out access, selection, language or specification decision,
compiler or production implementation, production fact channel, E0.1 restart,
xlc migration, or default teaching.

## D13-R5A closes the corrected dense performance staging protocol (2026-07-15)

The corrected Dense Family Lock A performance layer passes a fresh independent
exact-byte hostile review. It preserves the 303 contract dispositions, 97
same-shape Rust operation gates, 520 exact cells, inference law, selection law,
and counted resource ceiling from D13-R5 while replacing only the incomplete v4
staging map. The v5 registry contains 27 explicit blockers with exact owner-
branch applicability: Mac-local branches have eight direct reference-pilot
prerequisites and 21 cumulative candidate-construction prerequisites; dual-
native branches have nine and 22. The reference pilot must close feasible
before the first candidate prompt. A common exact repository baseline now gates
all 520 cells, the OD-4 Rust reference adapter is separated from later
candidate-side META-5/compiler artifacts, and Candidate Freeze B pins artifacts
produced during authorized construction rather than circularly preceding them.

Independent regeneration reproduced all 29 generated files byte-for-byte and
authenticated the exact 39-file union. The fail-closed verifier rejects all 48
registered mutations; the independent reviewer rejected 14 additional coherent
attacks across stage placement, branch applicability, matrix joins, benefit
families, law/task domains, and resource accounting. The v5 report is
`dense-family-lock-a/DENSE-PERFORMANCE-HOSTILE-REVIEW-PASS.md` at SHA-256
`e42823c8ecf94b2ac5c898c3215c511e9881fd082b7b77a112e98ff3b3b7bfe1`.
The D13-R5 v4 report and commit remain superseded history and cannot supply a
live staging authority.

No pilot or candidate observation has occurred. This correction authorizes no
reference pilot, candidate construction, Candidate Freeze B, execution,
scoring, held-out access, selection, language or specification decision,
compiler implementation, production adoption, production fact channel, E0.1
restart, xlc migration, or default teaching.

## D13-R6 closes Dense Unique-Owner Family Lock A for owner review (2026-07-15)

The research-only Dense Unique-Owner Family Lock A revision
`F-DENSE-LOCK-A-R5` passes independent exact-byte whole-lock hostile review.
The canonical manifest contains 74 artifacts, eight current controls, seven
artifact-class records, and 13 explicit mixed PASS/BLOCKED completion rows. Its
SHA-256 is
`0bff36e75a41575ae16bd51fc12ef5c0fcdb819288aa7755eeea320741a5ad97`;
the build summary is
`a9584bc10ba6414a94d10cce2ba95ff066ef9e0632f8d2167b57b17bd31b9ea2`.
The dossier, builder, and verifier hashes are respectively
`2a7114b82a6cf97d81a6bcf4695cfcd50b28a3b15aa5e2048dbfa039ad5a1f13`,
`02290bc605aeb6c956c114cf2a9fcd6f3d0c434c27ceb2e732adb06d31f75afb`,
and `e610888c724753e83d995b150238d8605cfaf723a3db1fd6054bf9dd7fddf282`.

The durable layer-review identities are coverage
`d8ee4c161e84a3996c0167b54576893074a16775b30994ab8236e79fa63d4798`,
contract/soundness
`20b6325366c961a5d608066da8acd9a9c19352290fdaa44e3666f2e14430c7c7`,
corrected performance v5
`e42823c8ecf94b2ac5c898c3215c511e9881fd082b7b77a112e98ff3b3b7bfe1`,
and whole lock
`27534e0c792daf7970bbf5ee53de0fec2dd77d1e577daf17359ce61260cfff93`.
The final review independently reconstructed the 13-input/93-member E0.1
partition, the 27-blocker owner-branch stage closure, every authority boundary,
and all artifact/reviewer/completion joins. Sixty-five pre-report whole-lock
attacks and 13 report-parser attacks were rejected. `verify_dense_lock.py`,
`make check`, the separate `make -C compiler check`, Python 3.9 compilation,
`git diff --check`, AGENTS/CLAUDE byte identity, and the English-only scan all
pass on the exact reviewed bytes; both repository gates also passed before and
after the external review report was added.

This closes only the D13 research draft. The next action is owner review and
explicit resolution of OD-0 through OD-5. Approval of those decisions alone
still authorizes no reference pilot, candidate construction, Candidate Freeze
B, candidate or held-out execution, held-out access, candidate selection or
scoring, language or specification change or decision, compiler or production
implementation, production adoption, production fact channel, E0.1 restart,
xlc migration, or default teaching. Every operational or production action
requires a later separate owner authorization and its exact prerequisite
closure.

## D14-R1 closes the privileged-gate synthesis for owner review (2026-07-15)

The hostile-reviewed research selects one admission-gate direction and rejects
premature public-basis closure. The direction is a sealed capability admission
verifier with a compiler-embedded base root and one authenticated signed
approval-snapshot predicate for scoped out-of-band capsules. Ordinary source
has no privilege-admission spelling or flag. Proof over existing public checked
semantics remains an ordinary artifact available to every producer; only a new
machine, helper, lowering, backend, ABI, OS, device, or foreign semantic edge
requires a capsule. Every proposition consumed for safety, resource authority,
or optimizer facts must come from static proof, runtime
validation/enforcement, or explicit ledgered trust; ordinary abstract
container correctness is outside that obligation unless promoted to a checked
contract or fact.

The bounded sequential candidate now has nine schematic roles over full,
prefix, interval, and two-range shapes, including shape-preserving swap and
replace. It remains blocked on opaque generative sealing, full/inline adoption,
focus, cleanup, allocation policy, arbitrary relocation, exact static rules,
and protected-path evidence. H-STORE's dense-prefix route establishes payload
safety through checked indexing, so its abstract key/position invariant does
not justify a general proof logic; the frozen `ST-SPARSE` requirement is
recorded for re-adjudication as a possible mechanism overconstraint. The
26-domain companion is an admission/dependency map, not a derivability proof.

Two independent exact-byte hostile reviews report no remaining P0 or P1
finding. The reviewed report, witness derivations, admission map, and review
hashes are respectively
`3f1998623e2278585867f512a20da2d9c218b1b0a114f26b28932a655254acf8`,
`fd334c57d76f064d69d2bd9c6f114c94ff6caad6fbd666c76bf98947de2d3deb`,
`ebab8f2e0592d9524d0fd7db7382d961b60b52bab5e9e519a797b19606cdf4bd`,
and `72fb432f1db15b97faa400592a4fc83c1f86fd121b312bd11e6e70b989b17a92`.

The requested next research is staged: Gate Authentication Lock A and Bounded
Sequential Safety Kernel Lock B only. A general-predicate lock opens only after
a formal counterexample or later separately authorized measurement proves it
necessary; shared and concurrent families remain later. This step selects no
public basis, language or specification rule, compiler feature, standard
library, container, experiment, fact channel, or production action. Stages A
and B, every implementation, and every execution require explicit owner
authorization.

## D14-R2 selects Gate Authentication Lock A at the semantic level (2026-07-15)

The exact-byte hostile-reviewed lock selects one sealed stateful admission
verifier per fixed authenticated semantic toolchain root. Compiler-shipped
entries are authorization records in the embedded genesis snapshot; later
entries use signed sequential successor snapshots. Every privileged use passes
one current-membership, exact evidence, target/frame, implementation-cone,
consumer-cone, final-resolution, and review predicate. Ordinary source has no
privilege-enabling keyword, attribute, module, path, package, flag, plugin,
helper name, or cache route.

The review repaired mandatory dependency-cone binding, three-layer identity
separation, compiler-derived use scope, threshold key continuity, local-only
freshness claims, base/extension unification, revocation scope, ordinary
abstraction classification, and the exact distinction between certified bytes
and an explicitly trusted opaque provider. Proofs and hidden representations
using only existing public semantics remain ordinary. A new root-governed
primitive, machine/helper/lowering/backend edge, ABI/OS/device/foreign
assumption, proof axiom, or fact/effect kind requires an exact admitted entry or
a new semantic root.

The gate lock, completion contract, and hostile review hashes are respectively
`6c104f0cba7a3fe2c3e354be37d4351bd4f9413ab38cdd4bfa35d4cbd6eb4e13`,
`395279c549aa37137d122282eed5c5e4a732521697f41ae406632bd4e2141e4d`,
and `54818aa3045824bdb8b3c922829e2d5c6d7882970776786be62b5fe40f31ccc1`.

This closes the research-level authentication architecture only. Wire format,
cryptographic suite, key custody, protected-state provider, dynamic-provider
mechanism, verifier limits, test vectors, and conformance corpus remain
implementation locks that may not weaken the selected semantics. No public
resource basis, language or specification change, compiler implementation,
capability entry, standard library, experiment, or production action is
authorized by this step.

## D14-R3 proposes the certified-resource architecture and keeps exact closure pending (2026-07-15)

The owner packet proposes one sealed stateful admission verifier per
authenticated semantic toolchain root and one fixed checked public
resource-policy plane for ordinary proof-carrying modules. The candidate basis
separates spatial resource, place identity, lifecycle/control, interference,
external-frame, and final-loaded-code authority. Category-level paper witnesses
cover all 26 systems-demand categories and four independently selected
held-outs. Architecture hostile review found no architecture-shape blocker, but
the proposal remains pending owner selection.

The exact dense ledger is fail-closed rather than complete. Its full 303 by 14
authority expands to 1,773 contexts and 44,689 route obligations: 8,075 required
routes are activated, 35,021 forbidden routes have zero violations, and 340
required obligations across 150 contexts remain unresolved with no positive
D-2 or P-1 credit. The unresolved partition is 168 coarse Convert-route, 24
Convert-callable, 136 allocator-applicability, six IntoOwner ZST/capacity-
reshape, and six IntoBoxed fullness/ZST obligations. Five independent
multiplicity rows preserve one-versus-two carrier lower bounds in 39 admitted
structural-cost contexts without measurement credit. D-2 and P-1 remain
`PENDING`; the D14 completion lock remains open.

Independent authority reproduction, two deterministic exact verifies, and all
thirteen hostile mutations pass. The root and compiler verification gates pass,
and the changed research scope contains no non-English prose or filename. The
owner verdict, architecture decision/review, exact report/review, manifest,
route authority, and multiplicity authority SHA-256 values are respectively
`b8f75ef88d3a9a5f6ec954ac1f4edf4f9b5d1daca446d9af7c191a09d059ff98`,
`94fab00f99909f5211d18e11eb86d596523bb94c77a2f0c71af598bd10551e0d`,
`55ddddea054bcb8edad195ed260d854e34f6fb9d070889745e1ee85a25d2b2a6`,
`dab4e72f4b115f76eb7f1ff8e7981f8545bade96d68041385ed423a396de8164`,
`6d68d0ef13f7184dbf1b4e719f4ae0f42cec4446ee70331aac2a176aedb88aa5`,
`5a8e697e4c1df54b2362cbf08883a20d9daebdd53ee4915867a152542d757aa0`,
`e952e286f7ffc5ee0bd115a32d962dc1fb12f172050769d09540070fca277889`,
and `d40cde6c75682c0750f94f951c78294ba618d38aaccb48ade339287acb0cf081`.

This completed research packet requests only two owner decisions: whether to
select the architecture for further research, and whether to authorize one
bounded source-normalization/reference-trace pass covering Convert,
allocator events, ZST/fullness, all seven trace classifiers, Rotate, and stable
and cached-key sorting. It authorizes no language or specification change,
compiler or verifier implementation, capability entry, standard library,
container, candidate execution, benchmark, E0.1 restart, migration, or default
teaching.

D14-R3a (2026-07-15): synchronized `AGENTS.md` and `CLAUDE.md` with D14-R3; the obsolete OD-0-through-OD-5 next action is removed, both files remain byte-identical, and no research or production conclusion changes.

D14-R4 (2026-07-15): repaired and hostile-reviewed the privileged admission-state comparison. One stage-indexed, source-inaccessible predicate now distinguishes immutable release entries F, stateless signed extension grants C, and stateful successor membership S through exact typed witnesses and an acyclic semantic/authorization/release/use/receipt identity graph. The 42-row pinned Pareto matrix selects C only if existing external-frame templates must accept new approved instances without an authorization-release update, and F otherwise; S's unique local extension-grant currentness is not presently required and its protected state is an added cost. This is conditional research, not an owner or production selection. The baseline verifier passes and rejects 4/4 hostile mutations. Decision, dimensions, matrix, verifier, mutation-test, and hostile-review SHA-256 values are `004318ed7acee04e77dece09c4fc361fa4b19c0c447a52ac5636b553e2aaa74c`, `492444e5d2421159019dcb2a370e6902bc9cc175b7b490b9a26f7387507834b7`, `7e43cc84106f4b35197adf57b35941b5f6fb8891aa052c22d449c60b48e2922b`, `6da14a8fd5d403fd194ce6b804235434ced3eb5e3650120e9017dda88baa0a1e`, `38b30be2c97ff93490a4307b42b0b17e4e3ac88c57bc48e31b2bdbb1180a0580`, and `91e2664258fb1e2611e1ae114e0901f1cea36bc117a99ca4a78c2d6f2ebcd45e`. The bounded no-formula frame-template registry, exact minimal privilege cuts, D6 coverage, G-2-through-G-5 V2 lock, dense D-2, P-1, and owner F/C choice remain fail-closed. No language/specification/compiler/verifier implementation, capability entry, standard library, container, candidate execution, benchmark, E0.1 restart, migration, or default teaching is authorized.

D14-R5 (2026-07-15): added the root `HANDOVER.md` after the owner corrected the research scope. The active question is how a static compiler, runtime, or official-core boundary defines semantics unavailable to ordinary source, followed by derivation of a minimal safe public capability basis and ordinary-library coverage and cost analysis. Cryptographic authorization, F/C/S, signed grants, replay, revocation, key rotation, identity graphs, and independently distributed privileged extensions are historical out-of-scope research, not pending design choices. The handover records completed evidence, current fail-closed obligations, stale active status documents, explicit non-goals, falsifiers, and the exact continuation sequence. It authorizes no production change, implementation, candidate, or experiment.

D10-R1 (2026-07-15): strengthened the byte-identical `AGENTS.md` and `CLAUDE.md` onboarding rule to state directly that all project code and documentation must be written in English, while retaining D10's broader coverage of every new or modified repository artifact. This is a documentation-only clarification and changes no production or research conclusion.

D14-R6 (2026-07-15): corrected the active status layer to the owner's static privilege-definition scope. `THE-PLAN.md`, the byte-identical `AGENTS.md` and `CLAUDE.md`, `user-directives.md`, and `mcts_mem/xlang.md` now require a production-language census and comparison of compiler-, runtime-, and official-core-private mechanism classes before any xlang gate recommendation or public capability-basis derivation. The F/C/S, signed-grant, snapshot, replay, revocation, key-rotation, identity-graph, and independently distributed privileged-extension branch remains exact historical evidence for an out-of-scope authorization problem; no F/C/S choice or cryptographic template registry is pending. The Rust census, 276 coverage clusters, 26-domain ledger, dense obligations, and held-outs remain finite tests rather than a primitive list; 340 dense obligations across 150 contexts and exact D-2/P-1 remain fail-closed. This documentation-only correction authorizes no language/specification/compiler/verifier implementation, capability entry, standard library, container, candidate, experiment, E0.1 restart, xlc migration, production fact channel, or default teaching.

D14-R7 (2026-07-15): completed the primary-source static privilege-definition mechanism census across Rust 1.97.0, pinned Swift, pinned Go, and pinned .NET runtime revisions. The census separates compiler-hard-coded operations, compiler-only declarations, compiler-synthesized modules, official-core compilation modes, and exact native runtime imports; records authority, ordinary-source reachability, safe-call shape, compiler/library/runtime ownership, extension path, runtime and code-shape cost, no-standard-library fit, backend obligations, route multiplicity, audit/TCB load, and P0/W1/W3 consequences; and carries explicit name/path/attribute/flag/symbol/backend bypass falsifiers into the architectural gate comparison. It makes no xlang gate recommendation and authorizes no public capability-basis derivation yet. The census SHA-256 is `2ee4470e3b6f2d395e796783dbe1b698089754bc4ffa6245b51847eb2c9bb9cf`. No language/specification/compiler/verifier implementation, capability entry, standard library, container, candidate, experiment, E0.1 restart, xlc migration, production fact channel, or default teaching is authorized.

D14-R8 (2026-07-15): compared the five static privilege-definition mechanism classes and hostile-reviewed one architectural recommendation pending owner selection: a sealed compiler-embedded primitive registry whose compiler-created declaration identity is the sole semantic authority decision. Ordinary source may call, alias, re-export, and wrap fixed checked declarations but cannot define one through syntax, attributes, names, paths, flags, cached artifacts, symbols, plugins, or semantic descriptors. Direct lowering and exact runtime bodies are subordinate physical implementations selected by the same identity, not additional authority routes. The hostile review attacks sixteen admission and phase-boundary paths, records twelve binding invariants, and passes only at the architectural level; concrete parser, resolver, cache, phase-boundary, runtime/provider, target, constant-evaluation, optimizer-fact, diagnostic, and mutation-test obligations remain production fail-closed. Decision and hostile-review SHA-256 values are `f6a23db71d85887dfbab1e5ede34b6717da0f7d7b2b31c36814d52a675828bfd` and `075ff31e50faa81bfa29ac5d9dd7e61ee7a2ffcd3fa4b19adc91ce9b08254a87`. This selects no public capability row, source spelling, language/specification/compiler/verifier/runtime implementation, standard library, container, experiment, E0.1 restart, xlc migration, production fact channel, or default teaching.

D14-R9 (2026-07-15): re-derived and hostile-reviewed an abstract minimal public capability-basis hypothesis under the static registry gate. The candidate separates three fixed storage leaves (tag-free checked carrier/place formation, exact vacant/live transitions, and recoverable empty physical-root acquisition/release), six ordinary checked mechanisms that add no machine semantics, and independently counted stability, atomic, thread, exact external-event, exact target/device-event, and executable-code leaves. Ordinary predicates and proofs cannot add primitives, effects, facts, rules, lowerings, or foreign assumptions; external and target classes expand to exact compiler-owned rows rather than generic syscall, opcode, ABI-contract, or semantic descriptors. The exact 49/49 capability-obligation crosswalk records paper routes and fail-closed gaps, while replacement, swap, relocation, resize, named containers, allocator policy, synchronization policy, async policy, and buffering remain derived ordinary code. The hostile review passes only for further derivability research; exact proof judgments, external/target inventories, weak-memory/reclamation semantics, final-image validation, all 340 dense D-2 obligations, and P-1 remain fail-closed. Basis, hostile-review, and crosswalk SHA-256 values are `bc5272644fce5ab36d8b99fbee400c21872c7d9f6a4b0742f7ba98c9168711b9`, `4256fa7465b48a14e150ac24bc7eff15c3cda0feee122a8fa894f71987fe4f0c`, and `430e8b534c377ffeb021ff32f8b69b169099a53b2643b3ee5ef81b76ac546cc5`. No language/specification/compiler/verifier/runtime implementation, capability entry, standard library, container, candidate execution, benchmark, E0.1 restart, xlc migration, production fact channel, or default teaching is authorized.

D14-R10 (2026-07-15): completed and hostile-reviewed the static-basis derivability and structural-cost packet. It accounts for all 49 capability obligations and routes all 26 systems domains; constructs ordinary-library paper routes for the complete sequential witness set plus category routes for shared ownership, synchronization, channels, and async; preserves the controlling WITNESS-REGISTRY memory, allocation, traffic, event, and no-tax budgets; and separates exact external/target/runtime leaves from ordinary policy. Concurrent cuckoo migration remains blocked on Q5 memory/reclamation semantics, crash-consistent LSM on exact P7 persistence/filesystem rows, hot-swappable JIT on P9 final-image validation, and sparse GPU residency on exact P7/P8 DMA/device/reset rows. Dense D-2 retains 340 missing obligations across 150 contexts and P-1 remains unmeasured. Three smallest separately authorizable slices are recorded for P1/P2/Q1/Q2 safety, structural no-tax parity, and exact partial/retained external byte events. Packet and hostile-review SHA-256 values are `44629b71325dd4ca8c9b2b3b3b52c38c5261e26933686674c9a479848fbaeb13` and `843c88571b7acf2e7a64b2a78f6d79b29abc84ade9f35c6e702e2cef3f47f803`. No experiment, language/specification/compiler/verifier/runtime implementation, capability entry, standard library, container, benchmark, E0.1 restart, xlc migration, production fact channel, or default teaching is authorized.

D14-R11 (2026-07-15): completed the D14 static-privilege owner decision packet and updated active status to owner-review pending. The packet separates four dispositions: select/revise/reject the sealed compiler-embedded registry research direction; accept/revise/reject the abstract basis hypothesis; authorize none, only the deterministic P1/P2/Q1/Q2 safety slice, or the conditional safety-to-structural/external validation sequence; and leave production work unauthorized. It recommends research selection of the gate and basis, safety modeling first, structural no-tax work only after a hostile-reviewed safety PASS, and one bounded external-row slice after safety. `THE-PLAN.md`, byte-identical `AGENTS.md`/`CLAUDE.md`, and `mcts_mem/xlang.md` now record that the packet is complete without claiming owner or production selection. The owner packet SHA-256 is `d517c3c3be19122edf4ceb489ec29bb0f81d29b628fe8524c7ad3a99b0cb2cf2`. No validation slice, experiment, language/specification/compiler/verifier/runtime implementation, capability entry, standard library, container, benchmark, E0.1 restart, xlc migration, production fact channel, or default teaching is authorized.

D14-R12 (2026-07-15): owner corrected the capability research objective from mechanism-first privilege isolation to performance-first safe expressiveness. The active work must freeze finite cases where current xlang semantics block native representation or impose initialization, zeroing, copying, relocation, metadata, allocation, indirection, checks, code-shape, or machine-event costs; derive common semantics; and compare at least three materially different complete capability sets. Each candidate must expose its exact contents, removal witnesses, performance effects, ordinary-library derivations, checker/compiler/backend/runtime shape, weaker-shape taxes, safety and cleanup obligations, open problems, and falsifiers, followed by an explained pros/cons comparison. Keywords, general language and ownership rules, checked proofs, ordinary libraries, builtins, and exact machine leaves remain candidates. The sealed compiler registry and P1-P9/Q1-Q6 packet are retained as historical evidence, not selected direction; isolation applies only as a conditional implementation constraint. `PERFORMANCE-FIRST-CAPABILITY-RESEARCH-CHARTER.md`, `HANDOVER.md`, `THE-PLAN.md`, byte-identical `AGENTS.md`/`CLAUDE.md`, `user-directives.md`, and `mcts_mem/xlang.md` record the correction. The charter SHA-256 is `fd75eeb26e73b50ac2e3ca668a8897eedc3486aa0f7f4992011d8cafce9c3999`. Exact D-2/P-1 remain fail-closed. No experiment, language/specification/checker/compiler/verifier/runtime implementation, capability entry, standard library, container, benchmark, E0.1 restart, xlc migration, production fact channel, or default teaching is authorized.

D14-R13 (2026-07-15): froze the performance-first expressiveness frontier as 15 conjunctive demand families rather than proposed mechanisms. The ledger routes all 43 capability obligations whose status is not established or protected and separately retains the two protected no-tax controls; every row states its target contract, current rule blocker, forced initialization/copy/movement/allocation/metadata/indirection/check/code-shape/machine-event cost, correctness and cleanup obligations, protected budget, evidence strength, and observable falsifier. Measured scoped-alias, checked-law, and bounds-proof results remain narrow evidence; structural witnesses are not measurements; concurrency, address-stability, external, target, and final-image rows remain weaker deferred-domain obligations. The verifier enforces the closed row range, reference integrity, and complete frontier routing but makes no safety, performance, or domain-completeness proof. Gap ledger, report, and verifier SHA-256 values are `2f28461a2e68e79b29c64093062498efe4d0a650c5cf7d8b36586f288507005a`, `74faf2385f032f03a9aa06046d47ebd4362f7f9028b78ff9a666e0ba97202394`, and `78bf0325ca779051e67f6505dfd5c2183dcfccff42346679303021992f325943`. This selects no primitive, capability set, syntax, language/specification/checker/compiler/verifier/runtime implementation, standard library, container, candidate execution, experiment, benchmark, E0.1 restart, xlc migration, production fact channel, or default teaching.

D14-R14 (2026-07-15): derived three materially distinct paper capability sets from the performance-demand frontier. Candidate A is a proof-indexed resource calculus in which ordinary modules may define protocols reducible to one fixed logic; Candidate B is a bounded orthogonal kernel of tag-free storage, closed topology views, direct place transitions, scoped footprints, handles, lifecycle, cleanup, and fact schemas; Candidate C is a closed family-specialized substrate with compiler-known storage and protocol state machines. The sets share eleven lower-bound capabilities for opaque modules, stateful direct callables, selective immovability, refinement, verified facts, failure/commit algebra, allocation roots, thread/atomic events and a memory model, exact external events, exact target/device events, and executable-image lifecycle. Their 44 capability items each route all 15 gap families and record semantics, combination rules, deletion witnesses, ordinary-library derivations, implementation shape, safety invariants, open failures, and falsifiers. C0-8 through C0-11 table semantics are stated but platform rows remain unenumerated and receive no production-completeness credit. Candidate packet and verifier SHA-256 values are `8f729e7f1654e760df2aa93e9b493c4593973f3b2ed5292f134e5135fe3c8761` and `876425c0515b5437b59afa041a021b65cbe3cf85587c4af2dd2b9ff6b2eec44a`. This selects no candidate, primitive, syntax, language/specification/checker/compiler/verifier/runtime implementation, standard library, container, candidate execution, experiment, benchmark, E0.1 restart, xlc migration, production fact channel, or default teaching.

D14-R15 (2026-07-15): completed a uniform 17-dimension A/B/C comparison with an advantage, liability, trade-off, evidence class, and observable decision rule or falsifier in every cell. The dimensions cover gap coverage, native representation and zero cost, weaker-shape no-tax, semantic/rule count, composition, checker effort, compiler/backend special cases, runtime metadata, failure/cleanup, verified facts, AI stability, specification/teaching size, no-standard-library use, portability, extension control, safety/review, and compile-time/code size. No numeric score is used because one soundness failure, one protected-route tax, or one workload exclusion is disqualifying. There is no evidence-backed winner: B is only the first validation priority, A remains the generality control, and C remains the specialization control. Matrix, comparison report, and verifier SHA-256 values are `91bf85dbb11d8714976571b819a16fb4a31aaf172cfd4f8dc44b7d87bf33216b`, `328b14db02c8a86fe99369e9da2681805823f2d88d1067efea6f5ccbc8e3f0c1`, and `0cc3385519d9164a8f718e0115eb3ab405740167aabb6c3b8c316e3dd652b976`. This authorizes no candidate execution, safety model, structural prototype, AI trial, experiment, benchmark, language/specification/checker/compiler/verifier/runtime implementation, standard library, production fact channel, or default teaching.

D14-R16 (2026-07-15): completed a post-candidate-freeze paper derivability and structural-cost audit over the two protected baselines, eight visible witnesses, and four held-out contracts. The 42 A/B/C cells state ordinary-library routes, structural accounts, and unresolved falsifiers, with no PASS status, implementation, source, trace, execution, or measurement. This is not blind held-out validation because held-out identities and budgets were prior research inputs. H-STORE exposes Candidate B's unresolved public Sparse admission rule for an unrelated library's exact key-position-to-dense-live-payload relation; W-ARENA exposes a shared cross-root provenance obligation across registry-slot and block-owner-token relocation; W-RECUR and W-PIPE retain exact cleanup and abandonment gaps. A has paper expressiveness but no proof-feasibility credit. C retains routes only if its Sparse/access facilities are public substrates rather than named-container recognition. B is not recommendable as a complete capability set unless these definitions close without converging into A or C. Matrix, audit report, and verifier SHA-256 values are `fae20f97e6b9f7e3e83ff153864c85122bb334e4208737e4bec2a8af3a224799`, `ce4195a356752b6c85eb03b0cdb504fbc7d94b2e70c8dfc81ddd564dd1311f2b`, and `99eb4389b5d7064c11f2cc3e85d8feb524b22e44a517fff6908fd9fd9331cbc1`. Exact D-2/P-1 and all runtime claims remain fail-closed. This authorizes no repair, model, prototype, witness construction, held-out execution, experiment, benchmark, language/specification/checker/compiler/verifier/runtime implementation, standard library, production fact channel, or default teaching.

D14-R17 (2026-07-15): completed the performance-first hostile review and owner decision packet. Twenty-two attacks reject selecting any candidate: A remains the generality control, B the first repair-and-validation hypothesis, and C the specialization control. B survives as a distinct minimum only if public Sparse admission, backing-root provenance across owner-token relocation, and recursive/traversal exact cleanup become finite deterministic rules without acquiring A-style general proof authority or C-style family-specific operations; convergence chooses the next research comparison branch rather than a production design. Six separately authorizable requests isolate semantic repair, safety modeling, structural cost, one exact machine-event model, performance measurement, and AI stability. The packet recommends VR-0 paper repair only and keeps every validation and production action unauthorized pending owner selection. Hostile review, validation requests, owner packet, and verifier SHA-256 values are `7a04935e1230a6f05ce7a278ac88a6ec9c694085524c39958193bc1dde32fcbd`, `3bbb543ddc65e3ab8fd292616d376962ff4d177d278b0aadcb2907796e1b4c5d`, `ac98f65a7dc8b43de0a57313585e46785fea602f8fee6a4ce51579ffd825257f`, and `b642c1a3ee107ab5fdd6c0e0808e2536aa97dbf1b1910e8c839866e4c2cd2d98`. Exact D-2/P-1, concurrency, external, target, final-image, syntax/specification/checker/compiler/verifier/runtime, standard-library, migration, production-fact, and default-teaching work remain fail-closed.

D14-R18 (2026-07-15): synchronized active status after the performance-first owner packet. `THE-PLAN.md`, byte-identical `AGENTS.md`/`CLAUDE.md`, and `HANDOVER.md` now state owner-review pending, no evidence-backed winner, A/B/C control roles, B's three blocking definition gaps, and the recommendation of VR-0 paper repair only. The final status verifier rejects stale candidate-comparison-pending wording, nonidentical agent instructions, a selected winner, or implicit model/prototype/measurement authorization. Plan, handover, and status-verifier SHA-256 values are `da1336c9bf0f584e61c3289ad9e0bc9be4ff0b6e03225e729d4795493742e393`, `e21e660fc6b61ed3c02d3ae6e15bbcbd7094f85a1836c742d662d651941bd3e3`, and `ec6960dfd3c6112947c67c0893517ee3fede1d9495786ec68565895187329746`. All six validation requests and every production action remain unauthorized pending owner selection; exact D-2/P-1 remain fail-closed.

D14-R19 (2026-07-15): owner selected Candidate C as the first bounded validation hypothesis, B as the later compression challenge over evidenced families, and A as the generality fallback. `CANDIDATE-C-BOUNDED-VALIDATION-PLAN.md` is the controlling operational contract: the current authorization covers plan durability, Stage 0's frozen C v0 baseline, and Stage 1's exact five-operation Hashbrown paper calibration, followed by a mandatory stop at Gate 1. Each evidence slice is time-boxed and fail-closed; `UNKNOWN` cannot expand scope or receive pass credit. The plan, active plan, handover, owner directive, MCTS node, and status-verifier SHA-256 values are `3a46609f7eeaae559ae2a609c0275058c7933c533ad31d45b4442105b9412c7a`, `e24cd6bb430474c963c4eb36951121640c563728a97c0af70a5e741fc756f035`, `790aa45164044c3ac941efc13a96eb6841308073f95a0e7a925081353c55b63f`, `a641b7b478f8e0e7427a2a5f937c57edeb2ce3d6ea1c029624b64d93c2388cf8`, `059b85a3505796a2a88a2151cca7c17c0b7968fdc5902dd376a2b6b2074829fd`, and `87e54a33dc4c458936fc915d9286cb04fe96f3337bddfdea076a78ad9eec24ef`. No Stage 2, allocator or later project audit, safety model, prototype, candidate execution, benchmark, machine-event model, AI trial, language/specification/checker/compiler/runtime change, standard library, or production work is authorized; exact D-2/P-1 remain fail-closed.

D14-R20 (2026-07-15): completed Candidate C Stage 0 without adding capability. `CANDIDATE-C-V0-AUDIT-BASELINE.md` freezes all eleven C0 entries and C-1 through C-12, their representation charges and forbidden inferences, the only admissible cross-family composition classes, the family-admission rule, eight evidence states, eleven structural-cost dimensions, the audit schema, and ten safety/no-tax invariants. It treats ambiguity as absent authority and records six exact unresolved definition classes for Stage 1: sparse control-to-payload admission, sparse mutation/rehash state machines and cleanup, sparse result provenance, sparse fact schemas, an exact group/SIMD target row, and an exact growth-allocation row. Stage 0 is audit-ready, not semantically complete. Baseline, verifier, and MCTS SHA-256 values are `0515b383cc0ab4add767a778981fbd46926af1428f37e2b7cc848f0fb92ce0d6`, `075c0913fe49c7ec67fddb898625cc5ab84877f4db8d6a346cf57f79d1619f2b`, and `ce5d3274ed91d034e762b6fdf1eb208a393b976116362a57a829ea4b59b7e590`. This authorizes only the already-approved Stage 1 Hashbrown paper calibration and no Stage 2, model, prototype, implementation, execution, benchmark, machine-event work, or production change; exact D-2/P-1 remain fail-closed.

D14-R21 (2026-07-15): completed the bounded Candidate C Stage 1 calibration against official Hashbrown v0.17.1 commit `c62a63a61b7caf2de8f9ecb7b06a66b0ab6bdf3d` and stopped at Gate 1 with `C-REVISE`. The 18-row matrix covers exactly lookup, vacant insertion, replacement, removal, and rehash: 15 rows are `COMPOSITION-GAP` and three are `C0-GAP`. The slice fits the existing C-4 sparse family without project-name recognition, a newly admitted family, forced payload initialization or zeroing, extra owner traffic, unrelated-shape metadata, extra allocation, or an added whole-table scan. Exact reusable sparse control-to-payload admission, transition and partial-progress cleanup, entry/result provenance, and fact schemas remain absent across C-4/C-6/C-7/C-11; the selected group operation and growth allocation remain absent C0-10/C0-7 rows. Matrix, report, audit verifier, active plan, handover, MCTS, and status-verifier SHA-256 values are `c91701196238bd7be6057ae468c12aef06f62bd07c7d170000970408f0f70b18`, `c0af7f7bdede3798d24b0a4c6e755e4827e0296b3457699e079513695cd05e20`, `bcb51749ec9056706339be5e165c74fc28972c34efa098e99a2e68639aeae474`, `59116c0b0bb72bdc333f99bc7115585097813fc99f152c8a6ff4019a8bf00c03`, `78aaba99623322bda445c36a7792d1d0df83c10964beb9d65fca633a491c0dea`, `f7aa3de12c202663e9d02af55ec0c977d54e9de573cbfbdca5c4f37c21b82c2c`, and `c79dc133da78e4d9cfa5db765d484122c24e26aa3dec2aecadc9afcdefb6f5d6`. This is structural paper evidence only: no repair, safety proof, code-shape evidence, performance claim, Stage 2, allocator or later audit, model, prototype, implementation, execution, benchmark, machine-event work, or production change is authorized; exact D-2/P-1 remain fail-closed.

D14-R22 (2026-07-15): owner authorized route 1 after Candidate C Gate 1: a bounded paper design and comparison of repairs for the six known sparse-definition gaps before any allocator audit. `CANDIDATE-C-SPARSE-REPAIR-PLAN.md` is the controlling contract. It freezes exactly three alternatives—operation-closed sparse profiles, one profile-indexed sparse automaton, and an orthogonal factoring compression control—plus two shared exact C0 leaf proposals, exactly fifteen candidate-operation rows over the existing five Hashbrown operations, a 90-minute combined semantic-analysis/comparison limit, and a mandatory Sparse Repair Gate stop. Plan, active plan, handover, owner directive, MCTS, and status-verifier SHA-256 values are `7bb9c81923d4bfe43b5414052a8604c8029ee6d48fb8b6cc4380936542f04155`, `ae94e91f2a4e64da0cc56d1efddbe19158844cc7d1a30633db553610dc7fbd34`, `a29635a32e59932d871ec138afec606ceca131cf0b5ada1270656fd982bba236`, `e03f658ba1946b8623042182bfeee4abc88f7d03962724a9499c0dc37a5590b7`, `035a7756c3c4440515d87a4a9e59b7e6eba4fe5355ffa2b334049b81e696b0fe`, and `dc42e324c790f9d10f92e3c65f88bf88edabccf770adb0d9a354a3112ccd0048`. This authorizes no repair application, Candidate C v0 change, family admission, wider source inspection, Stage 2, other project audit, safety model, implementation, execution, generated-code inspection, benchmark, measurement, AI trial, or production change; exact D-2/P-1 remain fail-closed.

D14-R23 (2026-07-15): completed the bounded Candidate C sparse-repair comparison and stopped at the Sparse Repair Gate with `SPARSE-SELECT: SR-PROFILE` as a further-research hypothesis only. The exact fifteen-row matrix gives five paper-`CLOSED` routes to operation-closed profiles, five to the profile-indexed sparse automaton, and five `CONVERGES-B` routes to orthogonal factoring. The selected hypothesis defines one closed `SPARSE-AUTOMATON-1`, fixed `INSERT-1`, `REPLACE-1`, `REMOVE-1`, `RELOCATE-1`, and `REHASH-1` templates, finite access and fact tables, and separate exact `GROUP-MATCH-1` and `ROOT-ALLOC-1` C0 leaf proposals. It identifies no project-name recognition, writer predicate or cleanup authority, new family admission, forced initialization/zeroing, extra payload traffic, extra allocation, unrelated metadata, indirection, check, scan, synchronization, or asymptotic cost. `SR-CLOSED` loses on avoidable full-catalog duplication per profile; `SR-ORTHOGONAL` is B's later compression direction and becomes A-like if it accepts arbitrary invariants. Matrix, candidate report, verifier, active plan, handover, MCTS, and status-verifier SHA-256 values are `84709f9375aaba476dc01ebd5f7602285f58731014137f8d9ff2425870233cf6`, `8c9d8fe1aac425f91d2e594960d05b3307aec7c4ef57287023351cb83b90d14e`, `346ef3f1d806d7046965e78c601a562cba180d7bc60eeab3b6eb12ef958f3d85`, `428b6e0ad4e61e64a25acb7b2988cc5a17bd90dea0764726dca1d99afc2563f5`, `cfe761e02add906438189142d0158c144bf94ac16382054a29f4d6d1b0bdbc41`, `aa0a9c21cfed65b8bba414caaf85f3222685afdcba1a0fa744df63add83945c0`, and `2085a755dfa5be11a0433de0ee6bf7e8120ff56080a47fce5b9481d6de800ba3`. Candidate C v0 remains unchanged. Formal safety, exact derivability, implementation, code shape, measurement, independent-demand generality, Stage 2, later audit, and production work remain unauthorized and unresolved; exact D-2/P-1 remain fail-closed.

D14-R24 (2026-07-15): owner authorized the same bounded comparative method for Candidate B and required evidence beyond Hashbrown. `CANDIDATE-B-ELEGANT-DESIGN-PLAN.md` freezes fourteen operations across pinned Hashbrown v0.17.1, mimalloc v3.3.2, SQLite 3.53.3, and Crossbeam Epoch 0.9.18 sources; exactly three alternatives (`B-FORMS`, `B-STRATA`, and `B-GRAPHS`); a fourteen-row source-demand audit; exactly 42 candidate-operation routes; explicit capability contents, removal witnesses, performance roles, ordinary-library derivations, unresolved problems, and pros/cons; and a mandatory Candidate B Design Gate stop. A new B rule must serve at least two independent projects, and time-bounded uncertainty stays `UNKNOWN`. Plan, active plan, handover, owner directive, MCTS, status verifier, and agent-instruction SHA-256 values are `46442eb8f2168dc6a172319ea0a57acf2c7ab8d29de7bc33cec96be4ea77a890`, `b68a90e58a5872cbfc473aea45d365cc46840711450600a6aa679f2edd450be0`, `44da99b9daee54efbbd4c75af118809c7b50d70536fcb6f305447b71e347ec18`, `cf4fde28fc68c573e910a742b704c2252080a5c37c374f1c957836c6c30bc4e6`, `54ff49518975eb376e12d1a3bb03de4cd242111120ff56b14ed3232db2860e6f`, `bbeeb47c6be216eb3cb43f7267d1338a8a0ed0172f76b829e391732e3c6c1230`, and `c7a1d406247327220152b9d44f339f3a7bf375f302ec4f94163d4b4eaaae3a75`. This authorizes only read-only primary-source inspection, paper design, comparison, hostile review, deterministic verification, and status durability. It authorizes no additional project or operation, formal safety model, prototype, candidate construction or execution, generated-code inspection, benchmark, measurement, AI trial, language/specification/checker/compiler/verifier/runtime change, standard library, container, or production selection; exact D-2/P-1 remain fail-closed.

D14-R25 (2026-07-15): corrected the mimalloc v3.3.2 source identity before source routing. `5687270e7fbb15d494a46b0d048f978bad973e4f` is the annotated tag object and dereferences to audited source commit `30b2d9d89099bee08e9f67a1ffb3e12e7ba45227`; the Crossbeam Epoch and SQLite tag identities are lightweight commit identities. The corrected Candidate B plan SHA-256 is `4e8dd41fcb0e25ae301ac67550dae80230d5efc9f1891149ba93996361c179a9`. No project, operation, candidate, route, authorization, or research conclusion changed.

D14-R26 (2026-07-15): completed Candidate B Phase 1's exact fourteen-operation read-only source audit across pinned Hashbrown v0.17.1, mimalloc v3.3.2 source commit `30b2d9d89099bee08e9f67a1ffb3e12e7ba45227`, SQLite 3.53.3 canonical Fossil source, and Crossbeam Epoch 0.9.18. The repeated demands are physical roots and partitions, state-indexed interpretation without permanent tags, closed metadata-to-place admission, direct affine and atomic owner transitions, non-escapable partial progress including rollback-required outcomes, physical-root provenance across wrappers and queues, bounded executable cleanup, and separation of shared-access safety from reclamation policy. The evidence adds mimalloc's uninitialized intrusive overlays and local/remote page ownership, SQLite's subrange invalidation and transaction-level repair, and Crossbeam's guard-scoped access, unique retirement, cross-root delayed disposition, and policy-selected quiescence to the existing Hashbrown evidence. Exact VFS/WAL, platform page-map, portable atomic, formal safety, progress, callback termination, generated-code, and performance questions remain open. Report, exact fourteen-row TSV, verifier, and MCTS SHA-256 values are `fbe0dcf1db00875be668da02b342ce13ad8328f4e691ae95b24003e6a8e0c1b5`, `e731a30bbba522a5f2dffd0736410280f97e7c1dd6de391f414e482d78dd1dd4`, `227e300d7dc910c2c65186785ca188deca0169049682112cc15be954f811a227`, and `771f276405a92031ce0cda60631612d89cece426e0de4a0a9e08db496e9e5611`. The audit scores no B alternative and demands no project-specific language form; it authorizes only the already-approved paper architecture and 42-row comparison.

D14-R27 (2026-07-15): completed and independently hostile-reviewed the bounded Candidate B architecture comparison and stopped at the mandatory Design Gate with `B-REVISE`. The exact 42-route matrix gives the original flat `B-FORMS` zero closed and fourteen open rows, the eight-layer project-independent `B-STRATA` hypothesis six closed and eight open rows, and the bounded library-protocol `B-GRAPHS` control six closed and eight open rows. Both revised alternatives close all five Hashbrown operations and complete mimalloc small allocation, including cold collect/extend/retry/null, while full local/remote page disposition, every complete SQLite operation, and every Crossbeam operation remain open. `B-STRATA` compresses physical places, structural liveness, owner transitions, scoped progress/repair, physical-root provenance, finite executable disposition, invalidatable facts, and concurrent custody without project identity; exact page-observer quiescence, heterogeneous erased one-shot actions, and SQLite pager/VFS/WAL/reinitializer rows remain absent. Independent review removed an ordinary-validator-to-liveness authority hole, rejected hot-subpath credit for whole operations, downgraded local-free and SQLite rows with missing end-to-end leaves, and changed overclaimed graph convergence to fail-closed `OPEN`. Design report, matrix, design verifier, active plan, handover, MCTS, status verifier, and agent-instruction SHA-256 values are `d6513fd0bf91ab41a9917f8cdb5dbc1fe48632aa4a704a2ec40bdc19693cc7fc`, `a4b578835030eaf55e8655a3ecdb353604acbffae7018fda16809069fdcbfc62`, `5bc85fd4aa4b232bfbb5b39a7538c04a32dac0550105cbe6dce37f899f451da3`, `2a68bbba8c111c21ad50abde24cc4dcd2800ce63f39dba6c16234bff9586fbc4`, `c2a17df3e9e0cb6e69547f4279dd8d4b7e6531ee39d491642395dc6c47e504bb`, `1a221cfc54484a3c3eba8ce43a93e36da55b0f88d6a4ec16835ab0517f67ffd3`, `4e2f8bf5198bf835c41bdef40f63a12eafa1800ef261096ce3e437cd13884ec9`, and `a0beb08b13c8699a0ff78ac7257eefc7c6b2526388475f0502741f54bf72714f`. Retain `B-STRATA` only as the best-defined revision hypothesis; Candidate C v0 is unchanged, no production winner is selected, exact D-2/P-1 remain fail-closed, and no further audit, safety model, implementation, execution, code-shape inspection, benchmark, measurement, AI trial, language/specification/checker/compiler/verifier/runtime change, standard-library/container work, or production decision is authorized.

D14-R28 (2026-07-15): owner selected B-Strata as the sole capability architecture for a decisive research and conditional landing track. `CANDIDATE-B-STRATA-DECISIVE-PLAN.md` fixes the same four projects and fourteen complete operations, treats the eight strata as analytical jobs rather than preselected primitives, and forces exactly `STRATA-YES` or `STRATA-NO`: no Candidate C pivot, competing B-Graphs design, or open-ended revision verdict is permitted. Phase 1 first falsifies a three-judgment K1 rooted-place, K2 sealed-state, and K3 linear-step kernel against nonforgeable liveness, a shared policy-neutral quiescence theorem for mimalloc and Crossbeam, erased affine one-shot disposition, and SQLite's exact external-event/repair boundary. The plan freezes sole authority producers, a closed exact-leaf output boundary, one global semantic-repair budget, deterministic checking bounds, exact outcome-complete derivations for all fourteen operations, a general operational safety proof not replaced by bounded execution, one shared project-independent prototype path, per-operation erasure/code-shape/performance gates, and an exact production landing proposal on YES. Inconclusive measurement cannot become a third final state: after one frozen nonsemantic implementation correction it must earn performance credit or expose a semantic structural cost or deterministic-lowering infeasibility supporting NO. Three independent hostile reviews of core totality, soundness, and the landing path returned PASS after corrections. Plan, active plan, handover, owner-directive, MCTS, and status-verifier SHA-256 values are `834604478f79135317b3e77cad5e7229b6d65975531ed10bb3881471cfa4c386`, `ceb3038c8fc913efb493b4f64913f053d92b18e62324f4e0bd4454074422caf5`, `524cde1829bd9627a6ddb5109fe569495c3e1f841ed716d151c190ab70bedeae`, `a064649e11318afebdb199b230a9733f25d25440dc720abb37da0818387ddf32`, `328fdf7c7f85e73469d3ce679a430ddddca8d73954f38e8f94265c9b3002be19`, and `b0417356169540863d6f71c166de82a2884be0522373217887b5dfd32c040b2c`. Paper and model gates condition prototypes and measurement; production language, specification, checker, compiler, verifier, runtime, standard-library, container, xlc, migration, teaching, selection, and shipping changes remain unauthorized until a final YES and separate owner review. Exact D-2/P-1 and the 340 unresolved dense obligations remain fail-closed.

PRESERVE (2026-07-16): committed the prior session's in-progress working-tree state exactly as found, for durability only: the demand-substitution propagation across `THE-PLAN.md`, the byte-identical `AGENTS.md`/`CLAUDE.md`, `HANDOVER.md`, `mcts_mem/xlang.md`, and the rewritten `CANDIDATE-B-STRATA-DECISIVE-PLAN.md`; the untracked Phase 1 artifacts `CANDIDATE-B-STRATA-CORE.md`, the six strata ledgers, and `tools/verify_candidate_b_strata_core.py`; the updated `tools/verify_performance_research_status.py`; and the D10-R1 entry. No Phase 1 gate has run, the strata verifier still fails closed on its self-reported lineage-conservation blocker, and this entry states or changes no research conclusion, authorization, or verdict.

D15 (2026-07-16): the owner redirected the capability research to a fresh, fully autonomous derivation. The target: for most systems-programming scenarios, at least one blessed way of writing must reach or exceed the performance of the best existing implementations; the provided form count n must stay small under the binding spec-compactness requirement; line-by-line reproduction of existing software is explicitly not required. Compiler-known forms with disciplined trusted internals are readmitted to the answer space, the D14 B-Strata-only lock no longer constrains the active derivation, and all prior candidate artifacts remain historical evidence and falsifiers. Full ruling text is in `optimizer-language-research/notes/user-directives.md` under D15. This entry authorizes scenario mapping, independent design derivation, hostile review, comparison, and durable recording only; every production change remains separately gated.
